//! Live integration tests for `pak_vfs` against a local Torchlight Infinite installation.
//!
//! Configure `TLI_GAME_DIR` and `TLI_AES_KEY` in `.envrc`, then run explicitly:
//! `nix develop --command cargo test -p pak_vfs --test torchlight_infinite -- --ignored --nocapture`

use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Instant;

use pak_vfs::{
    AesKey, CompressionMethod, MountTier, PackageKind, PakVfs, ReadScratch, u32_to_usize,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn tli_game_dir() -> TestResult<PathBuf> {
    let raw = std::env::var("TLI_GAME_DIR").map_err(|err| {
        format!("TLI_GAME_DIR environment variable must be set for live VFS tests: {err}")
    })?;
    Ok(PathBuf::from(raw))
}

fn tli_aes_key() -> TestResult<AesKey> {
    let raw = std::env::var("TLI_AES_KEY").map_err(|err| {
        format!("TLI_AES_KEY environment variable must be set for live VFS tests: {err}")
    })?;
    Ok(AesKey::try_from_hex(&raw)?)
}

#[test]
#[ignore = "requires local TLI_GAME_DIR and TLI_AES_KEY"]
fn mounts_full_tli_vfs_and_verifies_overrides_aliases_packages_and_ranges() -> TestResult {
    let game_dir = tli_game_dir()?;
    let key = tli_aes_key()?;

    let t0 = Instant::now();
    let vfs = PakVfs::open_tli_game_dir(&game_dir, Some(key))?;
    let mount_elapsed = t0.elapsed();

    println!(
        "Mounted {} TLI .pak archives in {:?}: {} files, {} dirs, {} exts, {} aliases, arena = {} bytes ({:.2} MiB)",
        vfs.pak_count(),
        mount_elapsed,
        vfs.file_count(),
        vfs.dir_count(),
        vfs.distinct_extension_count(),
        vfs.mount_alias_count(),
        vfs.arena_bytes(),
        f64::from(vfs.arena_bytes()) / (1024.0 * 1024.0),
    );

    assert!(vfs.pak_count() >= 60);
    assert!(vfs.file_count() > 400_000);
    assert!(vfs.dir_count() > 15_000);
    assert!(vfs.distinct_extension_count() >= 30);
    assert!(vfs.mount_alias_count() >= 15);
    // Deduplicated string arena for 435k files + 19k dirs must remain under 8 MiB (~5.18 MiB actual).
    assert!(vfs.arena_bytes() < 8 * 1024 * 1024);

    let mut scratch = ReadScratch::with_default_limits();
    verify_patch_overrides_and_ue_aliases(&vfs, &mut scratch)?;
    verify_cooked_package_companions(&vfs, &mut scratch)?;
    verify_large_oodle_ranged_and_cursor_reads(&vfs, &mut scratch)?;
    Ok(())
}

fn verify_patch_overrides_and_ue_aliases(vfs: &PakVfs, scratch: &mut ReadScratch) -> TestResult {
    let def_game = vfs
        .find("UE_game/Config/DefaultGame.ini")?
        .ok_or("DefaultGame.ini not found")?;
    let def_meta = vfs.file_meta(def_game);
    let def_pak = vfs.pak_info(def_meta.pak_id());
    assert!(matches!(def_pak.tier(), MountTier::PreMountPatch { .. }));
    vfs.verify_file_sha1(def_game, scratch)?;

    let ver_phys = vfs
        .find("UE_game/Content/Script/Luac/Config/Version.out")?
        .ok_or("Version.out physical path not found")?;
    let ver_ue = vfs
        .find_ue("/Game/Script/Luac/Config/Version.out")?
        .ok_or("Version.out /Game path not found")?;
    assert_eq!(ver_phys, ver_ue);
    let ver_pak = vfs.pak_info(vfs.file_meta(ver_phys).pak_id());
    assert!(matches!(ver_pak.tier(), MountTier::PreMountPatch { .. }));

    let ver_len = u32::try_from(vfs.file_meta(ver_phys).uncompressed_bytes())?;
    let mut ver_bytes = vec![0_u8; u32_to_usize(ver_len)];
    vfs.read_into(ver_phys, scratch, &mut ver_bytes)?;
    assert!(!ver_bytes.is_empty());

    assert!(vfs.resolve_ue_mount_root("/Game").is_some());
    assert!(vfs.resolve_ue_mount_root("/Engine").is_some());
    assert!(vfs.resolve_ue_mount_root("/AntiAddiction").is_some());
    assert!(vfs.find_dir("UE_game/Config")?.is_some());
    assert!(vfs.find_ue_dir("/Game/Script/Luac/Config")?.is_some());
    assert!(vfs.find_ue_dir("/Engine")?.is_some());
    Ok(())
}

fn verify_cooked_package_companions(vfs: &PakVfs, scratch: &mut ReadScratch) -> TestResult {
    let mut found_triple = false;
    let mut path_buf = Vec::new();
    for ubulk_id in vfs.iter_files_with_ext("ubulk").take(128) {
        let Some(pkg) = vfs.package_for_file(ubulk_id) else {
            continue;
        };
        if pkg.kind() == PackageKind::Asset && pkg.uexp().is_some() && pkg.ubulk() == Some(ubulk_id)
        {
            path_buf.clear();
            vfs.write_file_path(pkg.header(), &mut path_buf);
            let full_header_path = std::str::from_utf8(&path_buf)?;
            let without_ext = full_header_path
                .strip_suffix(".uasset")
                .ok_or("expected .uasset header suffix")?;
            assert_eq!(vfs.resolve_package(without_ext)?, Some(pkg));
            assert_eq!(vfs.resolve_package(full_header_path)?, Some(pkg));
            vfs.verify_file_sha1(pkg.header(), scratch)?;
            found_triple = true;
            break;
        }
    }
    assert!(found_triple);
    Ok(())
}

fn verify_large_oodle_ranged_and_cursor_reads(
    vfs: &PakVfs,
    scratch: &mut ReadScratch,
) -> TestResult {
    let mut target_file = None;
    for fid in vfs.iter_files_with_ext("ubulk") {
        let meta = vfs.file_meta(fid);
        if meta.compression_method() == CompressionMethod::Oodle
            && meta.uncompressed_bytes() >= 4 * 1024 * 1024
        {
            target_file = Some((fid, meta.uncompressed_bytes()));
            break;
        }
    }
    let (fid, file_bytes) = target_file.ok_or("expected >= 4 MiB Oodle .ubulk file in TLI")?;

    // Read a 512-byte slice across a 64 KiB block boundary near the middle of the file.
    let mid_offset = (file_bytes / 2) & !0xFFFF_u64;
    let boundary_offset = mid_offset - 128;
    let mut range_buf = [0_u8; 512];
    vfs.read_range_into(fid, boundary_offset, scratch, &mut range_buf)?;

    let mut cursor = vfs.open_cursor(fid, scratch);
    assert_eq!(
        cursor.seek(SeekFrom::Start(boundary_offset))?,
        boundary_offset
    );
    let mut cursor_buf = [0_u8; 512];
    cursor.read_exact(&mut cursor_buf)?;
    assert_eq!(range_buf, cursor_buf);
    Ok(())
}

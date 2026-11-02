//! Positive functional tests for `pak_vfs` discovery, shadowing, hierarchy, aliases, packages, and I/O.

use std::io::{Read, Seek, SeekFrom};

use pak::{ReadScratch, ScratchLimits};

use crate::tests::builder::{TempDir, TestPakEntry, TestPakSpec};
use crate::{
    DirId, MountSource, MountTier, PackageKind, PakVfs, VfsBatchOutcome, VfsError, VfsOpenOptions,
    VfsReadRequest, u32_to_usize,
};

fn read_utf8_file(vfs: &PakVfs, path: &str, scratch: &mut ReadScratch) -> String {
    let fid = vfs
        .find(path)
        .expect("valid path")
        .expect("file must exist");
    let len = u32::try_from(vfs.file_meta(fid).uncompressed_bytes()).expect("fits");
    let mut buf = vec![0_u8; u32_to_usize(len)];
    vfs.read_into(fid, scratch, &mut buf)
        .expect("read must succeed");
    String::from_utf8(buf).expect("valid utf8")
}

fn format_vfs_file_path(vfs: &PakVfs, file: crate::FileId) -> String {
    let mut out = Vec::new();
    vfs.write_file_path(file, &mut out);
    String::from_utf8(out).expect("valid utf8")
}

#[test]
fn tli_three_tier_discovery_and_patch_shadowing() {
    let tmp = TempDir::new("tli_tiers");
    // Unrelated third-party Chromium .pak outside the 3 UE directories must be ignored.
    tmp.write_raw_file(
        "Engine/Binaries/ThirdParty/CEF3/cef.pak",
        b"NOT_AN_UNREAL_PAK",
    );

    let mut p100 = TestPakSpec::new_v12("../../../");
    p100.entries
        .push(TestPakEntry::plain("UE_game/Config", "Ver.ini", b"pre_100"));
    tmp.write_pak(
        "UE_game/TorchLight/Saved/PersistentDownloadDir/PreMountPak/UE_game_100_P.pak",
        &p100,
    );

    let mut p200 = TestPakSpec::new_v12("../../../");
    p200.entries
        .push(TestPakEntry::plain("UE_game/Config", "Ver.ini", b"pre_200"));
    tmp.write_pak(
        "UE_game/TorchLight/Saved/PersistentDownloadDir/PreMountPak/UE_game_200_P.pak",
        &p200,
    );

    create_content_and_engine_tier_paks(&tmp);

    let vfs = PakVfs::open_tli_game_dir(tmp.path(), None).expect("open_tli_game_dir must succeed");
    assert_eq!(vfs.pak_count(), 7);
    let top_pak = vfs.pak_info(vfs.pak_id_at(0).expect("pak 0"));
    assert_eq!(
        top_pak.tier(),
        MountTier::PreMountPatch { patch_version: 200 }
    );
    assert_eq!(top_pak.winning_file_count(), 1);

    let mut scratch = ReadScratch::with_limits(ScratchLimits::DEFAULT);
    assert_eq!(
        read_utf8_file(&vfs, "UE_game/Config/Ver.ini", &mut scratch),
        "pre_200"
    );
    assert_eq!(
        read_utf8_file(&vfs, "UE_game/Content/Patch.txt", &mut scratch),
        "content_patch_50"
    );
    assert_eq!(
        read_utf8_file(&vfs, "UE_game/Content/Order.txt", &mut scratch),
        "chunk_101_9"
    );
    assert_eq!(
        read_utf8_file(&vfs, "Engine/Plugins/Plug.txt", &mut scratch),
        "engine_plug"
    );
}

fn create_content_and_engine_tier_paks(tmp: &TempDir) {
    let mut cp50 = TestPakSpec::new_v12("../../../UE_game/");
    cp50.entries.push(TestPakEntry::plain(
        "Content",
        "Patch.txt",
        b"content_patch_50",
    ));
    tmp.write_pak("UE_game/Content/Paks/UE_game_50_P.pak", &cp50);

    let mut c999 = TestPakSpec::new_v12("../../../UE_game/");
    c999.entries
        .push(TestPakEntry::plain("Content", "Patch.txt", b"chunk_999"));
    tmp.write_pak(
        "UE_game/Content/Paks/pakchunk999-WindowsNoEditor.pak",
        &c999,
    );

    // Reverse ASCII order puts 'pakchunk101-9-' ahead of 'pakchunk101-10-', matching UE_game.log.
    let mut c101_9 = TestPakSpec::new_v12("../../../UE_game/");
    c101_9
        .entries
        .push(TestPakEntry::plain("Content", "Order.txt", b"chunk_101_9"));
    tmp.write_pak(
        "UE_game/Content/Paks/pakchunk101-9-WindowsNoEditor.pak",
        &c101_9,
    );

    let mut c101_10 = TestPakSpec::new_v12("../../../UE_game/");
    c101_10
        .entries
        .push(TestPakEntry::plain("Content", "Order.txt", b"chunk_101_10"));
    tmp.write_pak(
        "UE_game/Content/Paks/pakchunk101-10-WindowsNoEditor.pak",
        &c101_10,
    );

    let mut ep = TestPakSpec::new_v12("../../../");
    ep.entries.push(TestPakEntry::plain(
        "Engine/Plugins",
        "Plug.txt",
        b"engine_plug",
    ));
    tmp.write_pak("Engine/Plugins/Paks/pakchunk777-WindowsNoEditor.pak", &ep);
}

#[test]
fn fresh_install_without_optional_pak_dirs_succeeds() {
    let tmp = TempDir::new("fresh_install");
    let mut base = TestPakSpec::new_v12("../../../UE_game/");
    base.entries
        .push(TestPakEntry::plain("Content", "Boot.bin", b"ok"));
    tmp.write_pak("UE_game/Content/Paks/pakchunk0-WindowsNoEditor.pak", &base);

    let vfs = PakVfs::open_tli_game_dir(tmp.path(), None).expect("must open without PreMountPak");
    assert_eq!(vfs.pak_count(), 1);
    assert_eq!(vfs.file_count(), 1);
}

#[test]
fn heterogeneous_mount_points_and_case_folding() {
    let tmp = TempDir::new("hetero_mounts");
    let mut hi = TestPakSpec::new_v12("../../../UE_game/Content/");
    hi.entries.push(TestPakEntry::plain(
        "Art/Hero",
        "Knight.UASSET",
        b"hi_knight",
    ));
    let hi_path = tmp.write_pak("hi.pak", &hi);

    let mut lo = TestPakSpec::new_v12("../../../");
    lo.entries.push(TestPakEntry::plain(
        "ue_game/content/art/hero",
        "knight.uasset",
        b"lo_knight",
    ));
    lo.entries
        .push(TestPakEntry::plain("Engine\\Config", "Base.ini", b"ini"));
    let lo_path = tmp.write_pak("lo.pak", &lo);

    let mounts = [MountSource::new(&hi_path), MountSource::new(&lo_path)];
    let opts = VfsOpenOptions::new(None).with_verify_index_hashes(true);
    let vfs = PakVfs::open_mounts(&mounts, &opts).expect("open_mounts must succeed");
    assert_eq!(vfs.file_count(), 2);

    let fid = vfs
        .find("ue_game\\CONTENT//art\\HERO/knight.uasset")
        .expect("valid query")
        .expect("must match case-insensitively");
    assert_eq!(
        format_vfs_file_path(&vfs, fid),
        "UE_game/Content/Art/Hero/Knight.UASSET"
    );
    assert_eq!(vfs.file_extension(fid), Some("UASSET"));

    let root = vfs.find_dir("").expect("valid").expect("root exists");
    assert_eq!(root, DirId::ROOT);
    assert_eq!(vfs.dir_name(root), "");
    let hero_dir = vfs
        .find_dir("UE_GAME/content/art/hero/")
        .expect("valid")
        .expect("dir exists");
    let mut dir_buf = Vec::new();
    vfs.write_dir_path(hero_dir, &mut dir_buf);
    assert_eq!(dir_buf, b"UE_game/Content/Art/Hero");
}

#[test]
fn unicode_case_folding_and_hash_bucket_collision_probing() {
    let tmp = TempDir::new("unicode_and_probing");
    let mut spec = TestPakSpec::new_v12("../../../");
    // 1-to-1 Unicode case folding ('Ä' <-> 'ä') plus 300 sibling entries forcing linear-probe chains.
    spec.entries
        .push(TestPakEntry::plain("Über/Ärger", "Öl.TXT", b"utf8_ok"));
    for i in 0..300_u32 {
        spec.entries.push(TestPakEntry::plain(
            &format!("Dense/D_{}", i % 17),
            &format!("item_{i:04}.dat"),
            &i.to_le_bytes(),
        ));
    }
    let p = tmp.write_pak("u.pak", &spec);
    let vfs =
        PakVfs::open_mounts(&[MountSource::new(&p)], &VfsOpenOptions::new(None)).expect("open");

    let uid = vfs
        .find("über/ärger/öl.txt")
        .expect("valid")
        .expect("unicode folded match");
    assert_eq!(format_vfs_file_path(&vfs, uid), "Über/Ärger/Öl.TXT");
    assert!(vfs.find_dir("ÜBER/ärger").expect("valid").is_some());

    for i in 0..300_u32 {
        let q = format!("dense/d_{}/ITEM_{i:04}.DAT", i % 17);
        assert!(vfs.find(&q).expect("valid").is_some());
    }
    assert!(
        vfs.find("dense/d_0/ITEM_9999.DAT")
            .expect("valid")
            .is_none()
    );
}

#[test]
fn stem_deduplication_and_bijective_filenames() {
    let tmp = TempDir::new("stem_dedup");
    let long_stem = "A_Very_Long_Shared_Unreal_Asset_Stem_Name_0123456789";
    let mut spec = TestPakSpec::new_v12("../../../UE_game/Content/");
    for ext in ["uasset", "uexp", "ubulk", "uptnl"] {
        spec.entries.push(TestPakEntry::plain(
            "Pkg",
            &format!("{long_stem}.{ext}"),
            b"x",
        ));
    }
    spec.entries
        .push(TestPakEntry::plain("Pkg", "Hero.v2.uasset", b"v2"));
    spec.entries
        .push(TestPakEntry::plain("Pkg", "Makefile", b"mk"));
    spec.entries
        .push(TestPakEntry::plain("Pkg", ".gitignore", b"gi"));
    let p = tmp.write_pak("p.pak", &spec);

    let vfs =
        PakVfs::open_mounts(&[MountSource::new(&p)], &VfsOpenOptions::new(None)).expect("open");
    // Four companions share `long_stem` once (`52` bytes), so total arena must stay well below 2x52+rest.
    assert!(vfs.arena_bytes() < 150);

    let v2 = vfs
        .find("UE_game/Content/Pkg/Hero.v2.uasset")
        .expect("ok")
        .expect("found");
    assert_eq!(vfs.file_stem(v2), "Hero.v2");
    assert_eq!(vfs.file_extension(v2), Some("uasset"));
    assert_eq!(
        format_vfs_file_path(&vfs, v2),
        "UE_game/Content/Pkg/Hero.v2.uasset"
    );

    for raw_leaf in ["Makefile", ".gitignore"] {
        let full = format!("UE_game/Content/Pkg/{raw_leaf}");
        let id = vfs.find(&full).expect("ok").expect("found");
        assert_eq!(vfs.file_stem(id), raw_leaf);
        assert_eq!(vfs.file_extension(id), None);
        assert_eq!(format_vfs_file_path(&vfs, id), full);
    }
}

#[test]
fn hierarchy_wide_branching_subtree_and_ext_iterators() {
    let tmp = TempDir::new("wide_tree");
    let mut spec = TestPakSpec::new_v12("../../../");
    // 80 sibling subdirectories (> PATH_DEPTH_HARD_MAX = 64) verifies SubtreeFileIter's slice stack.
    for i in 0..80_u32 {
        let ext = if i % 2 == 0 { "png" } else { "PNG" };
        spec.entries.push(TestPakEntry::plain(
            &format!("Icons/Sub_{i:03}"),
            &format!("icon_{i}.{ext}"),
            b"px",
        ));
    }
    spec.entries
        .push(TestPakEntry::plain("Icons", "readme.txt", b"hi"));
    let p = tmp.write_pak("wide.pak", &spec);

    let vfs =
        PakVfs::open_mounts(&[MountSource::new(&p)], &VfsOpenOptions::new(None)).expect("open");
    let icons_dir = vfs.find_dir("Icons").expect("ok").expect("Icons dir");
    let view = vfs.read_dir(icons_dir);
    assert_eq!(view.subdir_count(), 80);
    assert_eq!(view.subdirs().len(), 80);
    assert_eq!(view.subdirs().rev().count(), 80);
    assert!(view.subdir_at(0).is_some());
    assert!(view.subdir_at(80).is_none());
    assert_eq!(view.file_count(), 1);
    assert!(view.file_at(0).is_some());
    assert!(view.file_at(1).is_none());

    let subtree_count = vfs.iter_subtree_files(icons_dir).count();
    assert_eq!(subtree_count, 81);
    assert_eq!(vfs.iter_files().len(), 81);
    assert_eq!(vfs.iter_files().rev().count(), 81);
    assert_eq!(vfs.iter_files_with_ext(".PnG").count(), 80);
    assert_eq!(vfs.iter_files_with_ext("nonexistent").count(), 0);
}

#[test]
fn unreal_mount_aliases_and_plugin_precedence() {
    let tmp = TempDir::new("ue_aliases");
    let spec = build_alias_fixture_spec();
    let p = tmp.write_pak("aliases.pak", &spec);
    let opts = VfsOpenOptions::new(None)
        .with_game_project_name("UE_game")
        .expect("valid project");
    let vfs = PakVfs::open_mounts(&[MountSource::new(&p)], &opts).expect("open");

    assert_eq!(vfs.mount_alias_count(), 4);
    assert_eq!(vfs.iter_mount_aliases().rev().count(), 4);
    assert!(vfs.resolve_ue_mount_root("/Game/").is_some());
    assert!(vfs.find_ue("/Game/UI/Hud.uasset").expect("ok").is_some());
    assert!(
        vfs.find_ue("/Engine/BasicShapes/Cube.uasset")
            .expect("ok")
            .is_some()
    );
    // Physical `Engine/Config/Base.ini` is NOT under `/Engine` (`Engine/Content`), preventing collision.
    assert!(
        vfs.find_ue("/Engine/Config/Base.ini")
            .expect("ok")
            .is_none()
    );
    assert!(vfs.find("Engine/Config/Base.ini").expect("ok").is_some());
    assert!(
        vfs.find_ue("/NoDescPlug/Asset.uasset")
            .expect("ok")
            .is_some()
    );

    let shared_fid = vfs
        .find_ue("/SharedPlug/Which.txt")
        .expect("ok")
        .expect("found");
    let mut scratch = ReadScratch::with_limits(ScratchLimits::DEFAULT);
    let mut buf = [0_u8; 12];
    vfs.read_into(shared_fid, &mut scratch, &mut buf)
        .expect("read");
    assert_eq!(&buf, b"from_project");
}

fn build_alias_fixture_spec() -> TestPakSpec {
    let mut spec = TestPakSpec::new_v12("../../../");
    for (dir, file, body) in [
        ("Engine/Config", "Base.ini", &b"phys_eng"[..]),
        ("Engine/Content/BasicShapes", "Cube.uasset", b"eng_cube"),
        ("Engine/Plugins/SharedPlug", "SharedPlug.uplugin", b"{}"),
        (
            "Engine/Plugins/SharedPlug/Content",
            "Which.txt",
            b"from_engine",
        ),
        ("UE_game/Content/UI", "Hud.uasset", b"game_hud"),
        (
            "UE_game/Plugins/SharedPlug/Content",
            "Which.txt",
            b"from_project",
        ),
        (
            "UE_game/Plugins/NoDescPlug/Content",
            "Asset.uasset",
            b"no_desc",
        ),
    ] {
        spec.entries.push(TestPakEntry::plain(dir, file, body));
    }
    spec
}

#[test]
fn package_resolution_asset_map_and_multidot() {
    let tmp = TempDir::new("packages");
    let mut spec = TestPakSpec::new_v12("../../../UE_game/Content/");
    spec.entries
        .push(TestPakEntry::plain("Art", "Hero.uasset", b"hdr"));
    spec.entries
        .push(TestPakEntry::plain("Art", "Hero.uexp", b"exp"));
    spec.entries
        .push(TestPakEntry::plain("Art", "Hero.ubulk", b"blk"));
    spec.entries
        .push(TestPakEntry::plain("Art", "Hero.uptnl", b"ptn"));
    spec.entries
        .push(TestPakEntry::plain("Art", "Hero.v2.uasset", b"v2h"));
    spec.entries
        .push(TestPakEntry::plain("Art", "Hero.v2.uexp", b"v2e"));
    spec.entries
        .push(TestPakEntry::plain("Maps", "Town.umap", b"maph"));
    spec.entries
        .push(TestPakEntry::plain("Maps", "Town.uexp", b"mape"));
    let p = tmp.write_pak("pkg.pak", &spec);
    let vfs =
        PakVfs::open_mounts(&[MountSource::new(&p)], &VfsOpenOptions::new(None)).expect("open");

    let hero = vfs
        .resolve_ue_package("/Game/Art/Hero")
        .expect("ok")
        .expect("Hero pkg");
    assert_eq!(hero.kind(), PackageKind::Asset);
    assert!(hero.uexp().is_some());
    assert!(hero.ubulk().is_some());
    assert!(hero.uptnl().is_some());
    assert_eq!(
        vfs.resolve_package("UE_game/Content/Art/Hero.uasset")
            .expect("ok"),
        Some(hero)
    );
    assert_eq!(
        vfs.package_for_file(hero.ubulk().expect("ubulk")),
        Some(hero)
    );

    let v2 = vfs
        .resolve_ue_package("/Game/Art/Hero.v2")
        .expect("ok")
        .expect("Hero.v2 pkg");
    assert_eq!(vfs.file_stem(v2.header()), "Hero.v2");
    assert!(v2.uexp().is_some());

    let town = vfs
        .resolve_ue_package("/Game/Maps/Town.umap")
        .expect("ok")
        .expect("Town map");
    assert_eq!(town.kind(), PackageKind::Map);
    assert!(town.uexp().is_some());
}

#[test]
fn ranged_reads_batch_elevator_and_seeded_cursor() {
    let tmp = TempDir::new("io_suite");
    let mut payload = vec![0_u8; 9_000];
    for (idx, b) in (0_u32..9_000).zip(payload.iter_mut()) {
        let low = (idx & 0xFF) as u8;
        *b = low.wrapping_mul(31).wrapping_add(7);
    }
    let mut spec = TestPakSpec::new_v12("../../../");
    spec.entries
        .push(TestPakEntry::oodle("Data", "Blob.bin", &payload, 2048));
    spec.entries
        .push(TestPakEntry::plain("Data", "Raw.bin", b"0123456789ABCDEF"));
    let p = tmp.write_pak("io.pak", &spec);

    let vfs =
        PakVfs::open_mounts(&[MountSource::new(&p)], &VfsOpenOptions::new(None)).expect("open");
    let blob_id = vfs.find("Data/Blob.bin").expect("ok").expect("blob");
    let raw_id = vfs.find("Data/Raw.bin").expect("ok").expect("raw");
    let mut scratch = ReadScratch::with_limits(ScratchLimits::DEFAULT);

    vfs.verify_file_sha1(blob_id, &mut scratch)
        .expect("blob sha1 valid");
    vfs.verify_file_sha1(raw_id, &mut scratch)
        .expect("raw sha1 valid");

    // Cross-block range read spanning blocks 0, 1, and 2.
    let mut window = vec![0_u8; 2500];
    vfs.read_range_into(blob_id, 1900, &mut scratch, &mut window)
        .expect("range read");
    assert_eq!(&window[..], &payload[1900..4400]);

    verify_batch_elevator(&vfs, blob_id, raw_id, &payload, &mut scratch);
    verify_seeded_cursor(&vfs, blob_id, &payload, &mut scratch);
}

fn verify_batch_elevator(
    vfs: &PakVfs,
    blob_id: crate::FileId,
    raw_id: crate::FileId,
    payload: &[u8],
    scratch: &mut ReadScratch,
) {
    // Empty batch must be a total 0-work no-op.
    vfs.read_batch_into(&mut [], scratch, &mut []);

    // 70 requests exercises multi-chunk (> 64) stack-sorted elevator batching + per-item failure isolation.
    let mut bufs: Vec<[u8; 8]> = vec![[0_u8; 8]; 70];
    let mut outcomes: Vec<VfsBatchOutcome> = (0..70).map(|_| VfsBatchOutcome::Pending).collect();
    {
        let mut reqs: Vec<VfsReadRequest<'_>> = (0_u32..70)
            .zip(bufs.iter_mut())
            .map(|(i, dst)| {
                if i == 33 {
                    // Wrong buffer size for whole-file read of 16-byte `raw_id`.
                    VfsReadRequest::whole_file(raw_id, dst)
                } else if i.is_multiple_of(2) {
                    let off = u64::from(i.strict_mul(100));
                    VfsReadRequest::byte_range(blob_id, off, dst)
                } else {
                    VfsReadRequest::byte_range(raw_id, 4, dst)
                }
            })
            .collect();
        vfs.read_batch_into(&mut reqs, scratch, &mut outcomes);
    }
    assert_eq!(bufs.len(), outcomes.len());
    for ((i, buf), outcome) in (0_u32..70).zip(bufs.iter()).zip(outcomes.iter()) {
        if i == 33 {
            assert!(matches!(
                outcome,
                VfsBatchOutcome::Err(VfsError::DestinationSizeMismatch { .. })
            ));
        } else if i.is_multiple_of(2) {
            assert!(matches!(outcome, VfsBatchOutcome::Ok { bytes_read: 8 }));
            let start = u32_to_usize(i.strict_mul(100));
            assert_eq!(&buf[..], &payload[start..start + 8]);
        } else {
            assert!(matches!(outcome, VfsBatchOutcome::Ok { bytes_read: 8 }));
            assert_eq!(&buf[..], b"456789AB");
        }
    }
}

fn verify_seeded_cursor(
    vfs: &PakVfs,
    blob_id: crate::FileId,
    payload: &[u8],
    scratch: &mut ReadScratch,
) {
    let mut rng = fastrand::Rng::with_seed(0x5EED_CAFE_1234_5678);
    let mut cursor = vfs.open_cursor(blob_id, scratch);
    assert_eq!(cursor.len(), 9_000);
    assert!(!cursor.is_empty());

    let mut read_buf = [0_u8; 257];
    for step in 0..200_u32 {
        let target = rng.u64(0..=9_000);
        let new_pos = match step % 3 {
            0 => cursor.seek(SeekFrom::Start(target)).expect("seek start"),
            1 => {
                let delta = i64::try_from(target).expect("fits")
                    - i64::try_from(cursor.position()).expect("fits");
                cursor.seek(SeekFrom::Current(delta)).expect("seek current")
            }
            _ => {
                let delta = i64::try_from(target).expect("fits") - 9_000_i64;
                cursor.seek(SeekFrom::End(delta)).expect("seek end")
            }
        };
        assert_eq!(new_pos, target);
        let ask = rng.usize(1..=read_buf.len());
        let n = cursor.read(&mut read_buf[..ask]).expect("cursor read");
        let start = usize::try_from(target).expect("fits");
        assert_eq!(&read_buf[..n], &payload[start..start + n]);
    }
}

//! Live integration tests against a local Torchlight Infinite installation (`v12` `.pak` dialect).
//!
//! Configure `TLI_GAME_DIR` and `TLI_AES_KEY` in a local untracked `.envrc` (see `.envrc.example`),
//! then run explicitly:
//! `nix develop --command cargo test -p pak --test torchlight_infinite -- --ignored`

use std::path::PathBuf;

use pak::{
    AesKey, CompressionMethod, PakArchive, PakDialect, PakError, PakVersion, ReadScratch,
    u32_to_usize,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const PREMOUNT_REL_DIR: &str = "UE_game/TorchLight/Saved/PersistentDownloadDir/PreMountPak";
const PAKS_REL_DIR: &str = "UE_game/Content/Paks";

fn tli_game_dir() -> TestResult<PathBuf> {
    let raw = std::env::var("TLI_GAME_DIR").map_err(|err| {
        format!("TLI_GAME_DIR environment variable must be set for live game tests: {err}")
    })?;
    Ok(PathBuf::from(raw))
}

fn tli_aes_key() -> TestResult<AesKey> {
    let raw = std::env::var("TLI_AES_KEY").map_err(|err| {
        format!("TLI_AES_KEY environment variable must be set for live game tests: {err}")
    })?;
    let key = AesKey::try_from_hex(&raw)?;
    Ok(key)
}

fn read_verified_entry(
    archive: &PakArchive,
    id: pak::EntryId,
    scratch: &mut ReadScratch,
) -> Result<Vec<u8>, PakError> {
    archive.verify_entry_sha1(id, scratch)?;
    let meta = archive.entry_meta(id);
    let len_u32 = u32::try_from(meta.uncompressed_bytes()).map_err(|_| {
        PakError::DestinationSizeMismatch {
            expected: meta.uncompressed_bytes(),
            actual: u64::from(u32::MAX),
        }
    })?;
    let mut dst = vec![0_u8; u32_to_usize(len_u32)];
    let written = archive.read_entry_into(id, scratch, &mut dst)?;
    assert_eq!(written, meta.uncompressed_bytes());
    Ok(dst)
}

#[test]
#[ignore = "requires local TLI_GAME_DIR and TLI_AES_KEY"]
fn reads_premount_patch_paks_and_multiblock_oodle() -> TestResult {
    let premount_dir = tli_game_dir()?.join(PREMOUNT_REL_DIR);
    let key = tli_aes_key()?;
    let mut scratch = ReadScratch::with_default_limits();

    let patch_a = PakArchive::open(
        &premount_dir.join("UE_game_6164967_P.pak"),
        Some(key.clone()),
    )?;
    assert_eq!(patch_a.dialect(), PakDialect::TorchlightInfiniteV12);
    assert_eq!(patch_a.version(), PakVersion::V12);
    assert_eq!(patch_a.entry_count(), 1);

    let ver_id = patch_a
        .find("Version.out")?
        .ok_or("Version.out not found in UE_game_6164967_P.pak")?;
    let ver_buf = read_verified_entry(&patch_a, ver_id, &mut scratch)?;
    assert_eq!(ver_buf.len(), 313);

    let patch_b = PakArchive::open(&premount_dir.join("UE_game_6238244_P.pak"), Some(key))?;
    assert_eq!(patch_b.dialect(), PakDialect::TorchlightInfiniteV12);
    assert!(patch_b.entry_count() >= 3);
    let mut saw_multiblock_oodle = false;
    for (id, path_opt) in patch_b.iter_entries() {
        let rel_path = path_opt.ok_or("expected directory path in UE_game_6238244_P.pak")?;
        assert_eq!(patch_b.find(rel_path)?, Some(id));
        let meta = patch_b.entry_meta(id);
        if meta.compression_method() == CompressionMethod::Oodle && meta.block_count() > 1 {
            saw_multiblock_oodle = true;
        }
        let decoded = read_verified_entry(&patch_b, id, &mut scratch)?;
        assert_eq!(u64::try_from(decoded.len())?, meta.uncompressed_bytes());
    }
    assert!(saw_multiblock_oodle);
    Ok(())
}

#[test]
#[ignore = "requires local TLI_GAME_DIR and TLI_AES_KEY"]
fn reads_encrypted_oodle_zlib_and_concurrent_pakchunks() -> TestResult {
    let paks_dir = tli_game_dir()?.join(PAKS_REL_DIR);
    let key = tli_aes_key()?;

    let chunk999 = PakArchive::open(
        &paks_dir.join("pakchunk999-WindowsNoEditor.pak"),
        Some(key.clone()),
    )?;
    assert_eq!(chunk999.entry_count(), 130);
    let mut scratch = ReadScratch::with_default_limits();
    let mut encrypted_oodle_decoded: u32 = 0;
    let mut encrypted_raw_decoded: u32 = 0;
    for (id, path_opt) in chunk999.iter_entries() {
        if let Some(rel_path) = path_opt {
            assert_eq!(chunk999.find(rel_path)?, Some(id));
        }
        let meta = chunk999.entry_meta(id);
        let decoded = read_verified_entry(&chunk999, id, &mut scratch)?;
        assert_eq!(u64::try_from(decoded.len())?, meta.uncompressed_bytes());
        if meta.is_encrypted() && meta.compression_method() == CompressionMethod::Oodle {
            encrypted_oodle_decoded += 1;
        }
        if meta.is_encrypted() && meta.compression_method() == CompressionMethod::None {
            encrypted_raw_decoded += 1;
        }
    }
    assert!(encrypted_oodle_decoded >= 16);
    assert!(encrypted_raw_decoded >= 16);

    let chunk0 = PakArchive::open(&paks_dir.join("pakchunk0-WindowsNoEditor.pak"), Some(key))?;
    assert!(chunk0.entry_count() > 100);
    let mut saw_zlib = false;
    for (id, _) in chunk0.iter_entries().take(256) {
        let meta = chunk0.entry_meta(id);
        if meta.compression_method() == CompressionMethod::Zlib {
            let decoded = read_verified_entry(&chunk0, id, &mut scratch)?;
            assert_eq!(u64::try_from(decoded.len())?, meta.uncompressed_bytes());
            saw_zlib = true;
            break;
        }
    }
    assert!(saw_zlib);
    verify_concurrent_reads(&chunk0)?;
    Ok(())
}

fn verify_concurrent_reads(chunk0: &PakArchive) -> TestResult {
    let worker_results: Vec<TestResult> = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(4);
        for worker in 0_u32..4 {
            handles.push(scope.spawn(move || -> TestResult {
                let mut local_scratch = ReadScratch::with_default_limits();
                for step in 0_u32..16 {
                    let idx = (worker * 17 + step * 7) % chunk0.entry_count();
                    let id = chunk0.entry_id_at(idx).ok_or("entry index out of bounds")?;
                    let meta = chunk0.entry_meta(id);
                    if meta.uncompressed_bytes() <= 4 * 1024 * 1024 {
                        let decoded = read_verified_entry(chunk0, id, &mut local_scratch)?;
                        assert_eq!(u64::try_from(decoded.len())?, meta.uncompressed_bytes());
                    }
                }
                Ok(())
            }));
        }
        handles
            .into_iter()
            .map(|h| match h.join() {
                Ok(res) => res,
                Err(_) => Err("worker thread panicked".into()),
            })
            .collect()
    });
    for res in worker_results {
        res?;
    }
    Ok(())
}

//! Exact `0`, `MAX - 1`, `MAX`, and `MAX + 1` boundary transition tests.

use crate::archive::PakArchive;
use crate::error::{KeyParseErrorKind, LimitKind, PakError, WireContext};
use crate::path::compute_ue4_path_hash;
use crate::scratch::{ReadScratch, ScratchLimits};
use crate::tests::builder::{SyntheticEntry, SyntheticPakSpec, TempPakFile, build_synthetic_pak};
use crate::types::{
    AesKey, CompressionMethod, ENTRIES_COUNT_HARD_MAX, FOOTER_V8A_BYTES, MOUNT_POINT_BYTES_MAX,
    OpenOptions, PATH_BYTES_MAX, PATH_UTF16_UNITS_MAX, PakLimits, PakVersion,
};
use crate::u32_to_usize;
use crate::wire::take_fstring_into;

#[test]
fn file_size_footer_minimum_boundaries() {
    let min_usize = u32_to_usize(FOOTER_V8A_BYTES);

    let f0 = TempPakFile::from_bytes(&[]);
    let err0 = PakArchive::open(f0.path(), None).expect_err("0 bytes must fail");
    assert!(matches!(
        err0,
        PakError::FileTooSmall {
            file_bytes: 0,
            min_bytes: FOOTER_V8A_BYTES
        }
    ));

    let f188 = TempPakFile::from_bytes(&vec![0_u8; min_usize - 1]);
    let err188 = PakArchive::open(f188.path(), None).expect_err("188 bytes must fail");
    assert!(matches!(
        err188,
        PakError::FileTooSmall {
            file_bytes: 188,
            min_bytes: FOOTER_V8A_BYTES
        }
    ));

    let f189 = TempPakFile::from_bytes(&vec![0_u8; min_usize]);
    let err189 = PakArchive::open(f189.path(), None).expect_err("189 zero bytes must fail magic");
    assert!(matches!(err189, PakError::InvalidFooterMagic));
}

#[test]
fn aes_key_slice_and_hex_boundaries() {
    assert!(matches!(
        AesKey::try_from_slice(&[]),
        Err(PakError::InvalidAesKey {
            kind: KeyParseErrorKind::InvalidSliceLength { byte_len: 0 }
        })
    ));
    assert!(matches!(
        AesKey::try_from_slice(&[0xAA; 31]),
        Err(PakError::InvalidAesKey {
            kind: KeyParseErrorKind::InvalidSliceLength { byte_len: 31 }
        })
    ));
    let key32 = AesKey::try_from_slice(&[0xAB; 32]).expect("32-byte key valid");
    assert_eq!(format!("{key32:?}"), "AesKey([REDACTED; 32])");
    assert!(matches!(
        AesKey::try_from_slice(&[0xAA; 33]),
        Err(PakError::InvalidAesKey {
            kind: KeyParseErrorKind::InvalidSliceLength { byte_len: 33 }
        })
    ));

    assert!(matches!(
        AesKey::try_from_hex(""),
        Err(PakError::InvalidAesKey {
            kind: KeyParseErrorKind::InvalidHexLength { byte_len: 0 }
        })
    ));
    assert!(matches!(
        AesKey::try_from_hex(&"a".repeat(63)),
        Err(PakError::InvalidAesKey {
            kind: KeyParseErrorKind::InvalidHexLength { byte_len: 63 }
        })
    ));
    let hex64 = "ab".repeat(32);
    assert_eq!(AesKey::try_from_hex(&hex64).expect("64 hex"), key32);
    assert_eq!(
        AesKey::try_from_hex(&format!("  0x{hex64} \n")).expect("0x hex"),
        key32
    );
    assert_eq!(
        AesKey::try_from_hex(&format!("0X{}", hex64.to_uppercase())).expect("0X hex"),
        key32
    );
    assert!(matches!(
        AesKey::try_from_hex(&"a".repeat(65)),
        Err(PakError::InvalidAesKey {
            kind: KeyParseErrorKind::InvalidHexLength { byte_len: 65 }
        })
    ));
    let mut bad_hex = "a".repeat(63);
    bad_hex.push('g');
    assert!(matches!(
        AesKey::try_from_hex(&bad_hex),
        Err(PakError::InvalidAesKey {
            kind: KeyParseErrorKind::InvalidHexDigit
        })
    ));
}

#[test]
fn path_bytes_utf16_units_and_mount_point_boundaries() {
    let max_path = u32_to_usize(PATH_BYTES_MAX);
    compute_ue4_path_hash("", 0).expect("0 path bytes");
    compute_ue4_path_hash(&"a".repeat(max_path - 1), 0).expect("MAX - 1 path bytes");
    compute_ue4_path_hash(&"a".repeat(max_path), 0).expect("MAX path bytes");
    let over_path = compute_ue4_path_hash(&"a".repeat(max_path + 1), 0).expect_err("MAX + 1");
    assert!(matches!(
        over_path,
        PakError::LimitExceeded {
            kind: LimitKind::PathBytes,
            actual: 2049,
            max: 2048
        }
    ));

    let mut out = Vec::new();
    for &units in &[
        0_u32,
        PATH_UTF16_UNITS_MAX - 1,
        PATH_UTF16_UNITS_MAX,
        PATH_UTF16_UNITS_MAX + 1,
    ] {
        let mut wire = Vec::new();
        if units == 0 {
            wire.extend_from_slice(&0_i32.to_le_bytes());
        } else {
            let neg_len = -(i32::try_from(units + 1).expect("fits"));
            wire.extend_from_slice(&neg_len.to_le_bytes());
            for _ in 0..units {
                wire.extend_from_slice(&u16::from(b'x').to_le_bytes());
            }
            wire.extend_from_slice(&0_u16.to_le_bytes());
        }
        let mut slice = wire.as_slice();
        let res = take_fstring_into(
            &mut slice,
            PATH_BYTES_MAX * 2,
            LimitKind::PathBytes,
            WireContext::FString,
            &mut out,
        );
        if units <= PATH_UTF16_UNITS_MAX {
            res.expect("within PATH_UTF16_UNITS_MAX");
            assert_eq!(out.len(), u32_to_usize(units));
        } else {
            assert!(matches!(
                res,
                Err(PakError::LimitExceeded {
                    kind: LimitKind::PathUtf16Units,
                    actual: 2049,
                    max: 2048
                })
            ));
        }
    }
}

#[test]
fn mount_point_bytes_boundaries() {
    let max_mnt = u32_to_usize(MOUNT_POINT_BYTES_MAX);
    for mnt_len in [max_mnt - 1, max_mnt, max_mnt + 1] {
        let mut spec = SyntheticPakSpec::new(PakVersion::V12);
        spec.mount_point = "m".repeat(mnt_len);
        let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&spec));
        let res = PakArchive::open(tmp.path(), None);
        if mnt_len <= max_mnt {
            res.expect("mount point within MOUNT_POINT_BYTES_MAX");
        } else {
            assert!(matches!(
                res,
                Err(PakError::LimitExceeded {
                    kind: LimitKind::MountPointBytes,
                    actual: 1025,
                    max: 1024
                })
            ));
        }
    }
}

#[test]
fn pak_limits_constructor_boundaries() {
    assert!(PakLimits::try_new(0, 1, 1, 1).is_err());
    assert!(PakLimits::try_new(1, 0, 1, 1).is_err());
    assert!(PakLimits::try_new(1, 1, 0, 1).is_err());
    assert!(PakLimits::try_new(1, 1, 1, 0).is_err());

    PakLimits::try_new(1, ENTRIES_COUNT_HARD_MAX - 1, 1, 1).expect("HARD_MAX - 1 valid");
    PakLimits::try_new(1, ENTRIES_COUNT_HARD_MAX, 1, 1).expect("HARD_MAX valid");
    let over = PakLimits::try_new(1, ENTRIES_COUNT_HARD_MAX + 1, 1, 1).expect_err("HARD_MAX + 1");
    assert!(matches!(
        over,
        PakError::LimitExceeded {
            kind: LimitKind::EntriesCount,
            ..
        }
    ));
}

#[test]
fn pak_limits_enforcement_exact_transitions() {
    let mut spec = SyntheticPakSpec::new(PakVersion::V12);
    let mut e1 = SyntheticEntry::plain("A", "One.bin", &[0x11; 64]);
    e1.method = CompressionMethod::Zlib;
    e1.block_size = 32;
    let mut e2 = SyntheticEntry::plain("B", "Two.bin", &[0x22; 64]);
    e2.method = CompressionMethod::Zlib;
    e2.block_size = 32;
    spec.entries = vec![e1, e2];
    let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&spec));

    let lim_entries_low = PakLimits::try_new(1024 * 1024, 1, 100, 1024).expect("lim");
    assert!(matches!(
        PakArchive::open_with_options(
            tmp.path(),
            &OpenOptions::new(None).with_limits(lim_entries_low)
        ),
        Err(PakError::LimitExceeded {
            kind: LimitKind::EntriesCount,
            ..
        })
    ));
    let lim_entries_exact = PakLimits::try_new(1024 * 1024, 2, 4, 64).expect("lim");
    PakArchive::open_with_options(
        tmp.path(),
        &OpenOptions::new(None).with_limits(lim_entries_exact),
    )
    .expect("exact limits must succeed");

    let lim_blocks_low = PakLimits::try_new(1024 * 1024, 2, 3, 64).expect("lim");
    assert!(matches!(
        PakArchive::open_with_options(
            tmp.path(),
            &OpenOptions::new(None).with_limits(lim_blocks_low)
        ),
        Err(PakError::LimitExceeded {
            kind: LimitKind::BlocksTotalCount,
            ..
        })
    ));

    let lim_uncomp_low = PakLimits::try_new(1024 * 1024, 2, 4, 63).expect("lim");
    assert!(matches!(
        PakArchive::open_with_options(
            tmp.path(),
            &OpenOptions::new(None).with_limits(lim_uncomp_low)
        ),
        Err(PakError::LimitExceeded {
            kind: LimitKind::EntryUncompressedBytes,
            actual: 64,
            max: 63
        })
    ));

    let lim_idx_tiny = PakLimits::try_new(8, 2, 4, 64).expect("lim");
    assert!(matches!(
        PakArchive::open_with_options(
            tmp.path(),
            &OpenOptions::new(None).with_limits(lim_idx_tiny)
        ),
        Err(PakError::LimitExceeded {
            kind: LimitKind::IndexBytes,
            ..
        })
    ));
}

#[test]
fn scratch_limits_and_capacity_boundaries() {
    assert!(ScratchLimits::try_new(0, 0).is_err());
    assert!(ScratchLimits::try_new(65, 64).is_err());
    ScratchLimits::try_new(0, 1).expect("0..=1 valid");
    ScratchLimits::try_new(64, 64).expect("64..=64 valid");

    let mut spec = SyntheticPakSpec::new(PakVersion::V12);
    let mut entry = SyntheticEntry::plain("S", "Blob.bin", &[0x42; 128]);
    entry.method = CompressionMethod::Oodle;
    entry.block_size = 2048;
    spec.entries.push(entry);
    let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&spec));
    let archive = PakArchive::open(tmp.path(), None).expect("open");
    let id = archive.entry_id_at(0).expect("id 0");
    let span = u32::try_from(archive.entry_meta(id).disk_span_bytes()).expect("span u32");

    let mut scratch_too_small =
        ReadScratch::with_limits(ScratchLimits::try_new(0, span - 1).expect("limits"));
    // Streaming SHA-1 verification succeeds even when `io_bytes_max` is smaller than the entry.
    archive
        .verify_entry_sha1(id, &mut scratch_too_small)
        .expect("streaming sha1 must succeed with constrained scratch");
    let mut dst = [0_u8; 128];
    let err = archive
        .read_entry_into(id, &mut scratch_too_small, &mut dst)
        .expect_err("span - 1 must fail");
    assert!(matches!(err, PakError::ScratchCapacityExceeded { .. }));

    let mut scratch_exact =
        ReadScratch::with_limits(ScratchLimits::try_new(0, span).expect("limits"));
    archive
        .read_entry_into(id, &mut scratch_exact, &mut dst)
        .expect("exact scratch max must succeed");
    assert_eq!(dst, [0x42; 128]);
}

#[test]
fn scratch_block_bytes_and_range_boundaries() {
    let base = ScratchLimits::DEFAULT;
    assert!(matches!(
        base.with_block_bytes_max(0),
        Err(PakError::LimitExceeded {
            kind: LimitKind::ScratchBlockBytes,
            actual: 0,
            max: 1
        })
    ));

    let mut spec = SyntheticPakSpec::new(PakVersion::V12);
    let mut entry = SyntheticEntry::plain("S", "Range.bin", &[0x7E; 128]);
    entry.method = CompressionMethod::Zlib;
    entry.block_size = 64;
    spec.entries.push(entry);
    let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&spec));
    let archive = PakArchive::open(tmp.path(), None).expect("open");
    let id = archive.entry_id_at(0).expect("id 0");

    let lim_63 = base.with_block_bytes_max(63).expect("63 valid");
    let mut scratch_63 = ReadScratch::with_limits(lim_63);
    let mut partial = [0_u8; 10];
    assert!(matches!(
        archive.read_entry_range_into(id, 4, &mut scratch_63, &mut partial),
        Err(PakError::LimitExceeded {
            kind: LimitKind::ScratchBlockBytes,
            actual: 64,
            max: 63
        })
    ));

    let lim_64 = base.with_block_bytes_max(64).expect("64 valid");
    let mut scratch_64 = ReadScratch::with_limits(lim_64);
    archive
        .read_entry_range_into(id, 4, &mut scratch_64, &mut partial)
        .expect("exact 64B block_bytes_max must succeed");
    assert_eq!(partial, [0x7E; 10]);

    archive
        .read_entry_range_into(id, 118, &mut scratch_64, &mut partial)
        .expect("exact end boundary 118 + 10 == 128");
    assert!(matches!(
        archive.read_entry_range_into(id, 119, &mut scratch_64, &mut partial),
        Err(PakError::OutOfBoundsRange {
            offset: 119,
            byte_len: 10,
            container_bytes: 128
        })
    ));
    assert!(matches!(
        archive.read_entry_range_into(id, u64::MAX, &mut scratch_64, &mut [0_u8; 1]),
        Err(PakError::OutOfBoundsRange {
            offset: u64::MAX,
            byte_len: 1,
            container_bytes: 128
        })
    ));
}

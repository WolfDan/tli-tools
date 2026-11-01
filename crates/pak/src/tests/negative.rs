//! Negative wire-corruption rejections and `#[should_panic]` internal precondition tests.

use crate::archive::PakArchive;
use crate::crypto::PakCipher;
use crate::error::{EncryptedSectionKind, IndexSectionKind, LimitKind, PakError, WireContext};
use crate::scratch::ReadScratch;
use crate::tests::builder::{SyntheticEntry, SyntheticPakSpec, TempPakFile, build_synthetic_pak};
use crate::tests::test_aes_key;
use crate::types::{
    AesKey, BatchReadOutcome, BatchReadRequest, CompressionMethod, EntryId, OpenOptions,
    PATH_BYTES_MAX, PakDialect, PakVersion,
};
use crate::wire::take_fstring_into;

#[test]
fn rejects_v9_frozen_index_and_unsupported_versions() {
    let mut v9_frozen = SyntheticPakSpec::new(PakVersion::V9);
    v9_frozen.is_v9_frozen = true;
    let tmp_frozen = TempPakFile::from_bytes(&build_synthetic_pak(&v9_frozen));
    assert!(matches!(
        PakArchive::open(tmp_frozen.path(), None),
        Err(PakError::FrozenIndexUnsupported)
    ));

    let mut std_bytes = build_synthetic_pak(&SyntheticPakSpec::new(PakVersion::V11));
    let std_ver_off = std_bytes.len() - 221 + 21;
    std_bytes[std_ver_off..std_ver_off + 4].copy_from_slice(&12_u32.to_le_bytes());
    let tmp_std_bad = TempPakFile::from_bytes(&std_bytes);
    assert!(matches!(
        PakArchive::open(tmp_std_bad.path(), None),
        Err(PakError::UnsupportedVersion {
            dialect: PakDialect::StandardUe4,
            raw_version: 12
        })
    ));

    let mut tli_bytes = build_synthetic_pak(&SyntheticPakSpec::new(PakVersion::V12));
    let tli_ver_off = tli_bytes.len() - 222 + 21;
    tli_bytes[tli_ver_off..tli_ver_off + 4].copy_from_slice(&13_u32.to_le_bytes());
    let tmp_tli_bad = TempPakFile::from_bytes(&tli_bytes);
    assert!(matches!(
        PakArchive::open(tmp_tli_bad.path(), None),
        Err(PakError::UnsupportedVersion {
            dialect: PakDialect::TorchlightInfiniteV12,
            raw_version: 13
        })
    ));
}

#[test]
fn rejects_missing_or_wrong_aes_key_and_unaligned_ciphertext() {
    let key = test_aes_key();
    let wrong_key = AesKey::from_bytes([0xFF; 32]);

    let mut enc_idx_spec = SyntheticPakSpec::new(PakVersion::V12);
    enc_idx_spec.key = Some(key.clone());
    enc_idx_spec.encrypt_index = true;
    enc_idx_spec.guid = [0x77; 16];
    let tmp_enc_idx = TempPakFile::from_bytes(&build_synthetic_pak(&enc_idx_spec));

    assert!(matches!(
        PakArchive::open(tmp_enc_idx.path(), None),
        Err(PakError::EncryptionKeyRequired { .. })
    ));
    assert!(matches!(
        PakArchive::open(tmp_enc_idx.path(), Some(wrong_key)),
        Err(PakError::IndexHashMismatch {
            section: IndexSectionKind::PrimaryIndex,
            ..
        })
    ));

    let mut enc_entry_spec = SyntheticPakSpec::new(PakVersion::V12);
    enc_entry_spec.key = Some(key.clone());
    enc_entry_spec.encrypt_index = false;
    let mut e = SyntheticEntry::plain("Sec", "Payload.bin", b"secret-data");
    e.encrypted = true;
    enc_entry_spec.entries.push(e);
    let tmp_enc_entry = TempPakFile::from_bytes(&build_synthetic_pak(&enc_entry_spec));

    let archive_no_key =
        PakArchive::open(tmp_enc_entry.path(), None).expect("metadata-only open without key");
    let id = archive_no_key.entry_id_at(0).expect("id 0");
    let mut scratch = ReadScratch::with_default_limits();
    let mut dst = [0_u8; 11];
    assert!(matches!(
        archive_no_key.read_entry_into(id, &mut scratch, &mut dst),
        Err(PakError::EncryptionKeyRequired { .. })
    ));

    let cipher = PakCipher::new(&key);
    cipher
        .decrypt_in_place(EncryptedSectionKind::EntryPayload, &mut [])
        .expect("0 bytes aligned");
    assert!(matches!(
        cipher.decrypt_in_place(EncryptedSectionKind::EntryPayload, &mut [0_u8; 15]),
        Err(PakError::UnalignedEncryptedSize { byte_len: 15, .. })
    ));
    assert!(matches!(
        cipher.decrypt_in_place(EncryptedSectionKind::EntryPayload, &mut [0_u8; 17]),
        Err(PakError::UnalignedEncryptedSize { byte_len: 17, .. })
    ));
}

#[test]
fn detects_corrupted_payload_sha1_and_corrupt_codec_streams() {
    let mut spec = SyntheticPakSpec::new(PakVersion::V12);
    let e_plain = SyntheticEntry::plain("Check", "Plain.bin", b"integrity-test-bytes");
    let mut e_bad_zlib = SyntheticEntry::plain("Check", "BadZlib.bin", &[0x11; 32]);
    e_bad_zlib.method = CompressionMethod::Zlib;
    e_bad_zlib.block_size = 2048;
    e_bad_zlib.precompressed_blocks = Some(vec![vec![0xFF, 0x00, 0xAA, 0x55]]);

    let mut e_bad_oodle = SyntheticEntry::plain("Check", "BadOodle.bin", &[0x22; 32]);
    e_bad_oodle.method = CompressionMethod::Oodle;
    e_bad_oodle.block_size = 2048;
    e_bad_oodle.precompressed_blocks = Some(vec![vec![0xFF, 0xFF, 0x00, 0x01]]);

    spec.entries = vec![e_plain, e_bad_zlib, e_bad_oodle];
    let mut bytes = build_synthetic_pak(&spec);
    bytes[56] ^= 0x01;
    let tmp = TempPakFile::from_bytes(&bytes);
    let archive = PakArchive::open(tmp.path(), None).expect("open");
    let mut scratch = ReadScratch::with_default_limits();

    let id0 = archive.entry_id_at(0).expect("0");
    assert!(matches!(
        archive.verify_entry_sha1(id0, &mut scratch),
        Err(PakError::PayloadHashMismatch { entry, .. }) if entry == id0
    ));
    let mut wrong_dst = [0_u8; 19];
    assert!(matches!(
        archive.read_entry_into(id0, &mut scratch, &mut wrong_dst),
        Err(PakError::DestinationSizeMismatch {
            expected: 20,
            actual: 19
        })
    ));

    let id1 = archive.entry_id_at(1).expect("1");
    let mut dst32 = [0_u8; 32];
    assert!(matches!(
        archive.read_entry_into(id1, &mut scratch, &mut dst32),
        Err(PakError::ZlibDecompressFailed {
            entry,
            block_index: 0
        }) if entry == id1
    ));

    let id2 = archive.entry_id_at(2).expect("2");
    assert!(matches!(
        archive.read_entry_into(id2, &mut scratch, &mut dst32),
        Err(PakError::OodleDecompressFailed {
            entry,
            block_index: 0
        }) if entry == id2
    ));
}

#[test]
fn rejects_invalid_utf8_utf16_and_referenced_unsupported_codec() {
    let mut out = Vec::new();
    let mut bad_utf8 = Vec::new();
    bad_utf8.extend_from_slice(&3_i32.to_le_bytes());
    bad_utf8.extend_from_slice(&[b'a', 0xFF, 0x00]);
    let mut s1 = bad_utf8.as_slice();
    assert!(matches!(
        take_fstring_into(
            &mut s1,
            PATH_BYTES_MAX,
            LimitKind::PathBytes,
            WireContext::FString,
            &mut out
        ),
        Err(PakError::InvalidUtf8String {
            context: WireContext::FString
        })
    ));

    let mut bad_utf16 = Vec::new();
    bad_utf16.extend_from_slice(&(-2_i32).to_le_bytes());
    bad_utf16.extend_from_slice(&0xD800_u16.to_le_bytes());
    bad_utf16.extend_from_slice(&0_u16.to_le_bytes());
    let mut s2 = bad_utf16.as_slice();
    assert!(matches!(
        take_fstring_into(
            &mut s2,
            PATH_BYTES_MAX,
            LimitKind::PathBytes,
            WireContext::FString,
            &mut out
        ),
        Err(PakError::InvalidUtf16String {
            context: WireContext::FString
        })
    ));

    let mut unref_spec = SyntheticPakSpec::new(PakVersion::V12);
    unref_spec.method_names = ["Zlib", "Oodle", "LZ4", "", ""];
    unref_spec
        .entries
        .push(SyntheticEntry::plain("A", "Ok.txt", b"hello"));
    let tmp_unref = TempPakFile::from_bytes(&build_synthetic_pak(&unref_spec));
    PakArchive::open(tmp_unref.path(), None).expect("unreferenced LZ4 slot allowed");

    let mut ref_spec = SyntheticPakSpec::new(PakVersion::V12);
    ref_spec.method_names = ["LZ4", "Oodle", "", "", ""];
    let mut lz4_entry = SyntheticEntry::plain("A", "Fail.bin", b"hello");
    lz4_entry.method = CompressionMethod::Zlib;
    ref_spec.entries.push(lz4_entry);
    let tmp_ref = TempPakFile::from_bytes(&build_synthetic_pak(&ref_spec));
    match PakArchive::open(tmp_ref.path(), None) {
        Err(PakError::UnsupportedCompressionMethod { name }) => {
            assert_eq!(name.as_str(), "LZ4");
        }
        other => panic!("expected UnsupportedCompressionMethod(LZ4), got {other:?}"),
    }
}

#[test]
fn rejects_compressed_size_exceeding_disk_span() {
    let mut spec = SyntheticPakSpec::new(PakVersion::V8B);
    spec.mount_point.clear();
    spec.entries
        .push(SyntheticEntry::plain("", "a", b"12345678"));
    let mut bytes = build_synthetic_pak(&spec);
    let comp_sz_off = 61 + 4 + 4 + 6 + 8;
    assert_eq!(&bytes[comp_sz_off..comp_sz_off + 8], &8_u64.to_le_bytes());
    bytes[comp_sz_off..comp_sz_off + 8].copy_from_slice(&9_u64.to_le_bytes());
    let tmp = TempPakFile::from_bytes(&bytes);
    let opts = OpenOptions::new(None).with_verify_index_hashes(false);
    assert!(matches!(
        PakArchive::open_with_options(tmp.path(), &opts),
        Err(PakError::CorruptWireLayout {
            context: WireContext::LegacyDirectoryIndex
        })
    ));
}

#[test]
#[should_panic(expected = "EntryId out of bounds")]
fn entry_meta_panics_on_out_of_bounds_entry_id() {
    let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&SyntheticPakSpec::new(
        PakVersion::V12,
    )));
    let archive = PakArchive::open(tmp.path(), None).expect("open");
    let meta = archive.entry_meta(EntryId::new(0));
    assert_eq!(meta.uncompressed_bytes(), 0);
}

#[test]
#[should_panic(expected = "EntryId out of bounds")]
fn entry_path_panics_on_out_of_bounds_entry_id() {
    let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&SyntheticPakSpec::new(
        PakVersion::V12,
    )));
    let archive = PakArchive::open(tmp.path(), None).expect("open");
    let path = archive.entry_path(EntryId::new(0));
    assert!(path.is_none());
}

#[test]
#[should_panic(expected = "EntryId out of bounds")]
fn read_entry_into_panics_on_out_of_bounds_entry_id() {
    let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&SyntheticPakSpec::new(
        PakVersion::V12,
    )));
    let archive = PakArchive::open(tmp.path(), None).expect("open");
    let mut scratch = ReadScratch::with_default_limits();
    archive
        .read_entry_into(EntryId::new(0), &mut scratch, &mut [])
        .expect("must panic on out-of-bounds EntryId before returning");
}

#[test]
#[should_panic]
fn read_batch_into_panics_on_mismatched_outcomes_len() {
    let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&SyntheticPakSpec::new(
        PakVersion::V12,
    )));
    let archive = PakArchive::open(tmp.path(), None).expect("open");
    let mut scratch = ReadScratch::with_default_limits();
    let mut dst = [];
    let mut reqs = [BatchReadRequest::new(EntryId::new(0), &mut dst)];
    let mut outcomes: [BatchReadOutcome; 0] = [];
    archive.read_batch_into(&mut reqs, &mut scratch, &mut outcomes);
}

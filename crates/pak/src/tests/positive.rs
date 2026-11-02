//! Positive round-trip, codec, dialect, lookup, batch, and concurrency tests.

use fastrand::Rng;

use crate::archive::{PakArchive, PakPayloadReader};
use crate::error::PakError;
use crate::path::{PathArenaBuilder, PathLookupTable, compute_ue4_path_hash};
use crate::scratch::ReadScratch;
use crate::tests::builder::{
    SyntheticEntry, SyntheticPakSpec, TempPakFile, build_synthetic_pak, make_lzb16_compressed_pair,
};
use crate::tests::{read_entry_vec, test_aes_key};
use crate::types::{
    BatchReadOutcome, BatchReadRequest, CompressionMethod, EntryId, OpenOptions, PakDialect,
    PakVersion, PathHash, StreamingEntry,
};

#[test]
fn ue4_path_hash_matches_known_tli_vector_and_normalization() {
    let tli_seed: u64 = 0x6537_f1ca;
    let known = compute_ue4_path_hash("Version.out", tli_seed).expect("valid path");
    assert_eq!(known, PathHash::from_raw(0x6c25_ce5a_c8a3_455d));

    let slash_upper = compute_ue4_path_hash("/VERSION.OUT", tli_seed).expect("valid path");
    let backslash_mixed = compute_ue4_path_hash("\\\\version.Out", tli_seed).expect("valid path");
    assert_eq!(slash_upper, known);
    assert_eq!(backslash_mixed, known);

    let nested_a =
        compute_ue4_path_hash("///UE_game\\Config\\DefaultGame.ini", 42).expect("valid path");
    let nested_b = compute_ue4_path_hash("ue_game/config/defaultgame.ini", 42).expect("valid path");
    let nested_seed0 =
        compute_ue4_path_hash("ue_game/config/defaultgame.ini", 0).expect("valid path");
    assert_eq!(nested_a, nested_b);
    assert_ne!(nested_a, nested_seed0);
}

#[test]
fn path_lookup_disambiguates_hash_collision_via_stored_path() {
    let mut arena_builder = PathArenaBuilder::with_capacity(2);
    arena_builder
        .push_path("DirA/Alpha.bin")
        .expect("push alpha");
    arena_builder.push_path("DirB/Beta.bin").expect("push beta");
    let arena = arena_builder.finish(2);

    let colliding_hash = PathHash::from_raw(0xDEAD_BEEF_CAFE_BABE);
    let id0 = crate::types::EntryId::new(0);
    let id1 = crate::types::EntryId::new(1);
    let mut lookup = PathLookupTable::with_entry_capacity(2);
    lookup.insert(colliding_hash, id0);
    lookup.insert(colliding_hash, id1);

    assert_eq!(
        lookup.find("dira\\alpha.bin", colliding_hash, &arena),
        Some(id0)
    );
    assert_eq!(
        lookup.find("/DIRB/BETA.BIN", colliding_hash, &arena),
        Some(id1)
    );
    assert_eq!(lookup.find("DirC/Gamma.bin", colliding_hash, &arena), None);
}

#[test]
fn all_pak_versions_and_dialects_roundtrip() {
    let versions = [
        (PakVersion::V8A, PakDialect::StandardUe4),
        (PakVersion::V8B, PakDialect::StandardUe4),
        (PakVersion::V9, PakDialect::StandardUe4),
        (PakVersion::V10, PakDialect::StandardUe4),
        (PakVersion::V11, PakDialect::StandardUe4),
        (PakVersion::V12, PakDialect::TorchlightInfiniteV12),
    ];
    let mut scratch = ReadScratch::with_default_limits();
    for (version, expected_dialect) in versions {
        let mut spec = SyntheticPakSpec::new(version);
        let plain = SyntheticEntry::plain("Config", "Game.ini", b"[Core]\nVersion=1\n");
        let mut zlib_e = SyntheticEntry::plain("Data", "Table.bin", &[0x5A; 512]);
        zlib_e.method = CompressionMethod::Zlib;
        zlib_e.block_size = 256;
        let mut oodle_e = SyntheticEntry::plain("Content", "Blob.uexp", &[0xA7; 300]);
        oodle_e.method = CompressionMethod::Oodle;
        oodle_e.block_size = 2048;
        spec.entries = vec![plain, zlib_e, oodle_e];

        let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&spec));
        let archive = PakArchive::open(tmp.path(), None).expect("open versioned pak");
        assert_eq!(archive.version(), version);
        assert_eq!(archive.dialect(), expected_dialect);
        assert_eq!(archive.mount_point(), "../../../");
        assert_eq!(archive.entry_count(), 3);
        assert_eq!(archive.entry_id_at(2).map(|id| id.index()), Some(2));
        assert_eq!(archive.entry_id_at(3), None);

        let mut iter = archive.iter_entries();
        assert_eq!(iter.len(), 3);
        let (_, last_path) = iter.next_back().expect("last entry");
        assert_eq!(last_path, Some("Content/Blob.uexp"));
        assert_eq!(iter.len(), 2);

        for (id, path_opt) in archive.iter_entries() {
            assert!(path_opt.is_some());
            archive
                .verify_entry_sha1(id, &mut scratch)
                .expect("sha1 valid");
        }
        let id_ini = archive
            .find("config\\game.ini")
            .expect("find ok")
            .expect("found");
        assert_eq!(
            read_entry_vec(&archive, id_ini, &mut scratch),
            b"[Core]\nVersion=1\n"
        );
    }
}

fn build_v12_codec_matrix_spec(key: crate::types::AesKey) -> (SyntheticPakSpec, Vec<u8>) {
    let mut spec = SyntheticPakSpec::new(PakVersion::V12);
    spec.key = Some(key);
    spec.encrypt_index = true;
    spec.path_hash_seed = 0x1234_5678_9ABC_DEF0;

    let mut rng = Rng::with_seed(0x5EED_2026);
    let multi_payload: Vec<u8> = (0..5000).map(|_| rng.u8(..)).collect();
    let (lzb16_comp, lzb16_plain) = make_lzb16_compressed_pair();

    let e_empty = SyntheticEntry::plain("Zero", "Empty.txt", b"");
    let e_plain = SyntheticEntry::plain("", "Version.out", b"6238244");
    let mut e_enc_raw = SyntheticEntry::plain("Sec", "RawEnc.bin", &[0xCC; 37]);
    e_enc_raw.encrypted = true;

    let mut e_zlib = SyntheticEntry::plain("Zip", "Multi.bin", &multi_payload);
    e_zlib.method = CompressionMethod::Zlib;
    e_zlib.encrypted = true;
    e_zlib.block_size = 1500;

    let mut e_oodle = SyntheticEntry::plain("Ooz", "Kraken.bin", &multi_payload);
    e_oodle.method = CompressionMethod::Oodle;
    e_oodle.encrypted = true;
    e_oodle.block_size = 2048;

    let mut e_lzb16 = SyntheticEntry::plain("Ooz", "Lzb16.bin", &lzb16_plain);
    e_lzb16.method = CompressionMethod::Oodle;
    e_lzb16.encrypted = true;
    e_lzb16.block_size = 2048;
    e_lzb16.precompressed_blocks = Some(vec![lzb16_comp]);

    let mut e_unenc = SyntheticEntry::plain("LegacySlot", "Unencoded.bin", b"unencoded-v12");
    e_unenc.force_unencoded = true;

    let mut e_pruned = SyntheticEntry::plain("Pruned", "Hidden.bin", b"hash-only-payload");
    e_pruned.prune_directory_path = true;

    spec.entries = vec![
        e_empty, e_plain, e_enc_raw, e_zlib, e_oodle, e_lzb16, e_unenc, e_pruned,
    ];
    (spec, multi_payload)
}

#[test]
fn codecs_encryption_unencoded_and_pruned_entries_roundtrip() {
    let key = test_aes_key();
    let (spec, multi_payload) = build_v12_codec_matrix_spec(key.clone());
    let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&spec));
    let archive = PakArchive::open(tmp.path(), Some(key)).expect("open encrypted v12 pak");
    let mut scratch = ReadScratch::with_default_limits();

    assert_eq!(archive.entry_count(), 8);
    assert_eq!(archive.path_hash_seed(), 0x1234_5678_9ABC_DEF0);

    let id_empty = archive.find("Zero/Empty.txt").expect("ok").expect("found");
    assert_eq!(
        archive
            .read_entry_into(id_empty, &mut scratch, &mut [])
            .expect("empty read"),
        0
    );

    let id_enc_raw = archive.find("Sec/RawEnc.bin").expect("ok").expect("found");
    assert_eq!(archive.entry_meta(id_enc_raw).compressed_bytes(), 37);
    assert_eq!(archive.entry_meta(id_enc_raw).disk_span_bytes(), 48);
    assert_eq!(
        read_entry_vec(&archive, id_enc_raw, &mut scratch),
        [0xCC; 37]
    );

    let id_zlib = archive.find("zip/multi.bin").expect("ok").expect("found");
    assert_eq!(
        read_entry_vec(&archive, id_zlib, &mut scratch),
        multi_payload
    );

    let id_oodle = archive.find("ooz/kraken.bin").expect("ok").expect("found");
    assert_eq!(
        read_entry_vec(&archive, id_oodle, &mut scratch),
        multi_payload
    );

    let id_lzb16 = archive.find("Ooz/Lzb16.bin").expect("ok").expect("found");
    assert_eq!(
        read_entry_vec(&archive, id_lzb16, &mut scratch),
        b"TLI!TLI!TPAK_V12"
    );

    let id_unenc = archive
        .find("LegacySlot/Unencoded.bin")
        .expect("ok")
        .expect("found");
    assert_eq!(
        read_entry_vec(&archive, id_unenc, &mut scratch),
        b"unencoded-v12"
    );

    let pruned_hash =
        compute_ue4_path_hash("Pruned/Hidden.bin", archive.path_hash_seed()).expect("hash");
    let id_pruned = archive.find_by_hash(pruned_hash).expect("found by hash");
    assert_eq!(archive.entry_path(id_pruned), None);
    assert_eq!(
        archive.find("pruned\\hidden.bin").expect("ok"),
        Some(id_pruned)
    );
    assert_eq!(
        read_entry_vec(&archive, id_pruned, &mut scratch),
        b"hash-only-payload"
    );
    for (id, _) in archive.iter_entries() {
        archive
            .verify_entry_sha1(id, &mut scratch)
            .expect("sha1 verified");
    }
}

#[test]
fn utf16_fstrings_and_empty_archive_roundtrip() {
    let empty_spec = SyntheticPakSpec::new(PakVersion::V11);
    let empty_tmp = TempPakFile::from_bytes(&build_synthetic_pak(&empty_spec));
    let empty_pak = PakArchive::open(empty_tmp.path(), None).expect("open empty pak");
    assert_eq!(empty_pak.entry_count(), 0);
    assert_eq!(empty_pak.iter_entries().len(), 0);
    assert_eq!(empty_pak.entry_id_at(0), None);
    assert_eq!(empty_pak.find("Version.out").expect("ok"), None);

    let mut utf16_spec = SyntheticPakSpec::new(PakVersion::V12);
    utf16_spec.mount_point = "../../../GameRoot/".to_owned();
    utf16_spec.utf16_mount_point = true;
    let mut entry = SyntheticEntry::plain("Données", "Épreuve_α.bin", b"unicode-utf16-ok");
    entry.utf16_strings = true;
    utf16_spec.entries.push(entry);

    let utf16_tmp = TempPakFile::from_bytes(&build_synthetic_pak(&utf16_spec));
    let archive = PakArchive::open(utf16_tmp.path(), None).expect("open utf16 pak");
    assert_eq!(archive.mount_point(), "../../../GameRoot/");
    let id = archive
        .find("données\\épreuve_Α.bin")
        .expect("find unicode")
        .expect("found unicode entry");
    let mut scratch = ReadScratch::with_default_limits();
    assert_eq!(
        read_entry_vec(&archive, id, &mut scratch),
        b"unicode-utf16-ok"
    );
}

#[test]
fn batch_reads_and_multithreaded_concurrent_reads() {
    let key = test_aes_key();
    let (spec, _) = build_v12_codec_matrix_spec(key.clone());
    let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&spec));
    let archive = PakArchive::open(tmp.path(), Some(key)).expect("open v12 pak");

    let mut scratch = ReadScratch::with_default_limits();
    archive.read_batch_into(&mut [], &mut scratch, &mut []);

    let id_ver = archive.find("Version.out").expect("ok").expect("found");
    let id_lzb = archive.find("Ooz/Lzb16.bin").expect("ok").expect("found");
    let mut buf_ver = [0_u8; 7];
    let mut buf_bad = [0_u8; 3];
    let mut buf_lzb = [0_u8; 16];
    let mut reqs = [
        BatchReadRequest::new(id_ver, &mut buf_ver),
        BatchReadRequest::new(id_lzb, &mut buf_bad),
        BatchReadRequest::new(id_lzb, &mut buf_lzb),
    ];
    let mut outcomes = [
        BatchReadOutcome::Pending,
        BatchReadOutcome::Pending,
        BatchReadOutcome::Pending,
    ];
    archive.read_batch_into(&mut reqs, &mut scratch, &mut outcomes);
    assert!(matches!(
        outcomes[0],
        BatchReadOutcome::Ok { bytes_written: 7 }
    ));
    assert!(matches!(
        outcomes[1],
        BatchReadOutcome::Err(PakError::DestinationSizeMismatch {
            expected: 16,
            actual: 3
        })
    ));
    assert!(matches!(
        outcomes[2],
        BatchReadOutcome::Ok { bytes_written: 16 }
    ));
    assert_eq!(&buf_ver, b"6238244");
    assert_eq!(&buf_lzb, b"TLI!TLI!TPAK_V12");

    std::thread::scope(|scope| {
        for worker_idx in 0_u64..4 {
            let shared = &archive;
            scope.spawn(move || {
                let mut local_scratch = ReadScratch::with_default_limits();
                let mut rng = Rng::with_seed(0xABCD_0000 + worker_idx);
                for _ in 0..32 {
                    let idx = rng.u32(0..shared.entry_count());
                    let id = shared.entry_id_at(idx).expect("valid index");
                    let bytes = read_entry_vec(shared, id, &mut local_scratch);
                    assert_eq!(
                        u64::try_from(bytes.len()).expect("fits"),
                        shared.entry_meta(id).uncompressed_bytes()
                    );
                }
            });
        }
    });
}

#[test]
fn open_streaming_and_payload_reader_roundtrip_legacy_and_split() {
    let mut legacy_spec = SyntheticPakSpec::new(PakVersion::V8B);
    legacy_spec.entries = vec![
        SyntheticEntry::plain("Cfg", "Base.ini", b"v8b-ini"),
        SyntheticEntry::plain("Art", "Tex.bin", b"v8b-tex"),
    ];
    let tmp_legacy = TempPakFile::from_bytes(&build_synthetic_pak(&legacy_spec));
    let mut legacy_seen = Vec::new();
    let (legacy_reader, legacy_mnt) = PakPayloadReader::open_streaming(
        tmp_legacy.path(),
        &OpenOptions::new(None),
        |mnt, item| {
            assert_eq!(mnt, "../../../");
            match item {
                StreamingEntry::Named { id, dir, file } => {
                    legacy_seen.push((id, dir.to_owned(), file.to_owned()));
                }
                StreamingEntry::Pruned { .. } => panic!("unexpected pruned entry in v8b"),
            }
            Ok::<(), PakError>(())
        },
    )
    .expect("open_streaming v8b");
    assert_eq!(&*legacy_mnt, "../../../");
    assert_eq!(legacy_reader.entry_count(), 2);
    assert_eq!(legacy_seen.len(), 2);
    assert_eq!(legacy_seen[0].2, "Cfg/Base.ini");

    let key = test_aes_key();
    let (split_spec, _) = build_v12_codec_matrix_spec(key.clone());
    let tmp_split = TempPakFile::from_bytes(&build_synthetic_pak(&split_spec));
    let mut named_count: u32 = 0;
    let mut pruned_ids: Vec<EntryId> = Vec::new();
    let (split_reader, split_mnt) = PakPayloadReader::open_streaming(
        tmp_split.path(),
        &OpenOptions::new(Some(key.clone())),
        |mnt, item| {
            assert_eq!(mnt, "../../../");
            match item {
                StreamingEntry::Named { .. } => named_count += 1,
                StreamingEntry::Pruned { id } => pruned_ids.push(id),
            }
            Ok::<(), PakError>(())
        },
    )
    .expect("open_streaming v12");
    assert_eq!(&*split_mnt, "../../../");
    assert_eq!(named_count, 7);
    assert_eq!(pruned_ids.len(), 1);
    assert_eq!(split_reader.entry_count(), 8);

    let full_archive = PakArchive::open(tmp_split.path(), Some(key)).expect("open full");
    let converted = full_archive.into_payload_reader();
    let mut scratch = ReadScratch::with_default_limits();
    let mut pruned_buf = [0_u8; 17];
    converted
        .read_entry_into(pruned_ids[0], &mut scratch, &mut pruned_buf)
        .expect("read pruned via payload reader");
    assert_eq!(&pruned_buf, b"hash-only-payload");
}

#[test]
fn read_entry_range_into_across_codecs_encryption_and_block_cache() {
    let key = test_aes_key();
    let (spec, _) = build_v12_codec_matrix_spec(key.clone());
    let tmp = TempPakFile::from_bytes(&build_synthetic_pak(&spec));
    let archive = PakArchive::open(tmp.path(), Some(key)).expect("open v12 pak");
    let mut scratch = ReadScratch::with_default_limits();
    let mut rng = Rng::with_seed(0xCAFE_1234);

    for (id, _) in archive.iter_entries() {
        let full = read_entry_vec(&archive, id, &mut scratch);
        let total = u64::try_from(full.len()).expect("fits");
        archive
            .read_entry_range_into(id, 0, &mut scratch, &mut [])
            .expect("empty at 0");
        archive
            .read_entry_range_into(id, total, &mut scratch, &mut [])
            .expect("empty at end");
        verify_range_windows(&archive, id, &full, &mut scratch, &mut rng);
    }

    verify_cross_archive_cookie_isolation(&mut scratch);
}

fn verify_range_windows(
    archive: &PakArchive,
    id: EntryId,
    full: &[u8],
    scratch: &mut ReadScratch,
    rng: &mut Rng,
) {
    if full.is_empty() {
        return;
    }
    let edges = [
        (0, 1),
        (full.len() - 1, 1),
        (0, full.len()),
        (1.min(full.len() - 1), full.len().saturating_sub(2).max(1)),
    ];
    for (start, len) in edges {
        let mut win = vec![0_u8; len];
        let off = u64::try_from(start).expect("u64");
        archive
            .read_entry_range_into(id, off, scratch, &mut win)
            .expect("edge range");
        assert_eq!(win, &full[start..start + len]);
    }
    for _ in 0..24 {
        let start = rng.usize(0..full.len());
        let max_len = (full.len() - start).min(2200);
        let len = rng.usize(0..=max_len);
        let mut win = vec![0_u8; len];
        let off = u64::try_from(start).expect("u64");
        archive
            .read_entry_range_into(id, off, scratch, &mut win)
            .expect("random range");
        assert_eq!(win, &full[start..start + len]);
    }
}

fn verify_cross_archive_cookie_isolation(scratch: &mut ReadScratch) {
    let mut s1 = SyntheticPakSpec::new(PakVersion::V12);
    let mut e1 = SyntheticEntry::plain("A", "Same.bin", &[0x11; 512]);
    e1.method = CompressionMethod::Zlib;
    e1.block_size = 256;
    s1.entries.push(e1);

    let mut s2 = SyntheticPakSpec::new(PakVersion::V12);
    let mut e2 = SyntheticEntry::plain("A", "Same.bin", &[0x22; 512]);
    e2.method = CompressionMethod::Zlib;
    e2.block_size = 256;
    s2.entries.push(e2);

    let t1 = TempPakFile::from_bytes(&build_synthetic_pak(&s1));
    let t2 = TempPakFile::from_bytes(&build_synthetic_pak(&s2));
    let a1 = PakArchive::open(t1.path(), None).expect("open a1");
    let a2 = PakArchive::open(t2.path(), None).expect("open a2");
    let id0 = EntryId::new(0);

    let mut b1 = [0_u8; 16];
    let mut b2 = [0_u8; 16];
    a1.read_entry_range_into(id0, 8, scratch, &mut b1)
        .expect("a1 range");
    a1.read_entry_range_into(id0, 24, scratch, &mut b1)
        .expect("a1 cached block hit");
    assert_eq!(b1, [0x11; 16]);

    a2.read_entry_range_into(id0, 24, scratch, &mut b2)
        .expect("a2 must miss a1 cached block via reader_cookie");
    assert_eq!(b2, [0x22; 16]);
}

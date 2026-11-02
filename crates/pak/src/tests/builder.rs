//! Deterministic synthetic `.pak` binary generator for unit and boundary testing.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use flate2::{Compress, Compression, FlushCompress};

use crate::align_up_16;
use crate::crypto::{PakCipher, compute_sha1};
use crate::path::compute_ue4_path_hash;
use crate::types::{AesKey, CompressionMethod, PakDialect, PakVersion};
use crate::u32_to_usize;
use crate::wire::{MAGIC_TLI_V12, MAGIC_UE4, inline_header_bytes};

static NEXT_TEMP_ID: AtomicU32 = AtomicU32::new(1);

/// Self-cleaning temporary `.pak` file located under `std::env::temp_dir()`.
pub(crate) struct TempPakFile {
    path: PathBuf,
}

impl TempPakFile {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let seq = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let file_name = format!("tli_pak_test_{pid}_{seq}.pak");
        let path = std::env::temp_dir().join(file_name);
        fs::write(&path, bytes).expect("temp pak write must succeed");
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempPakFile {
    fn drop(&mut self) {
        match fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => panic!("failed to remove temporary test pak {:?}: {err}", self.path),
        }
    }
}

#[derive(Clone)]
pub(crate) struct SyntheticEntry {
    pub(crate) dir: String,
    pub(crate) file: String,
    pub(crate) utf16_strings: bool,
    pub(crate) payload: Vec<u8>,
    pub(crate) precompressed_blocks: Option<Vec<Vec<u8>>>,
    pub(crate) method: CompressionMethod,
    pub(crate) encrypted: bool,
    pub(crate) block_size: u32,
    pub(crate) force_unencoded: bool,
    pub(crate) prune_directory_path: bool,
}

impl SyntheticEntry {
    pub(crate) fn plain(dir: &str, file: &str, payload: &[u8]) -> Self {
        Self {
            dir: dir.to_owned(),
            file: file.to_owned(),
            utf16_strings: false,
            payload: payload.to_vec(),
            precompressed_blocks: None,
            method: CompressionMethod::None,
            encrypted: false,
            block_size: 0,
            force_unencoded: false,
            prune_directory_path: false,
        }
    }
}

pub(crate) struct SyntheticPakSpec {
    pub(crate) version: PakVersion,
    pub(crate) is_v9_frozen: bool,
    pub(crate) mount_point: String,
    pub(crate) utf16_mount_point: bool,
    pub(crate) path_hash_seed: u64,
    pub(crate) key: Option<AesKey>,
    pub(crate) encrypt_index: bool,
    pub(crate) guid: [u8; 16],
    pub(crate) method_names: [&'static str; 5],
    pub(crate) entries: Vec<SyntheticEntry>,
}

impl SyntheticPakSpec {
    pub(crate) fn new(version: PakVersion) -> Self {
        Self {
            version,
            is_v9_frozen: false,
            mount_point: "../../../".to_owned(),
            utf16_mount_point: false,
            path_hash_seed: 0,
            key: None,
            encrypt_index: false,
            guid: [0x11; 16],
            method_names: ["Zlib", "Oodle", "", "", ""],
            entries: Vec::new(),
        }
    }

    pub(crate) fn dialect(&self) -> PakDialect {
        if self.version == PakVersion::V12 {
            PakDialect::TorchlightInfiniteV12
        } else {
            PakDialect::StandardUe4
        }
    }
}

struct EncodedPayload {
    disk_bytes: Vec<u8>,
    block_raw_lens: Vec<u32>,
    rel_blocks: Vec<(u64, u64)>,
    compressed_total: u64,
}

struct WrittenEntryRecord {
    entry: SyntheticEntry,
    header_offset: u64,
    compressed_total: u64,
    raw_method_idx: u32,
    block_raw_lens: Vec<u32>,
    rel_blocks: Vec<(u64, u64)>,
    payload_sha1: [u8; 20],
}

struct RawEntryWriteSpec<'a> {
    dialect: PakDialect,
    offset: u64,
    compressed_size: u64,
    uncompressed_size: u64,
    raw_method: u32,
    sha1: [u8; 20],
    blocks: &'a [(u64, u64)],
    encrypted: bool,
    block_size: u32,
}

/// Compresses `src` using `flate2` Zlib wrapper format.
pub(crate) fn zlib_compress_chunk(src: &[u8]) -> Vec<u8> {
    let mut comp = Compress::new(Compression::fast(), true);
    let cap = src.len().saturating_add(src.len() / 4).saturating_add(256);
    let mut out = vec![0_u8; cap];
    let status = comp
        .compress(src, &mut out, FlushCompress::Finish)
        .expect("zlib compress must succeed");
    assert_eq!(status, flate2::Status::StreamEnd);
    let written = usize::try_from(comp.total_out()).expect("zlib out fits in usize");
    out.truncate(written);
    out
}

/// Encodes `src` (`<= 256 KiB`) as a valid Oodle Kraken uncompressed quantum block (`0x4C 0x06`).
pub(crate) fn oodle_encode_chunk(src: &[u8]) -> Vec<u8> {
    assert!(src.len() <= 0x40000);
    let mut out = Vec::with_capacity(src.len() + 2);
    out.extend_from_slice(&[0x4C, 0x06]);
    out.extend_from_slice(src);
    out
}

/// Builds a genuine LZ-compressed Oodle LZB16 block (`0x8C 0x02`) and its expected plaintext.
pub(crate) fn make_lzb16_compressed_pair() -> (Vec<u8>, Vec<u8>) {
    // 4 literals "TLI!" followed by match code 1 (length 5) at distance 4 -> "TLI!TLI!T" (9 bytes),
    // then final run of 7 literals "PAK_V12" -> 16 bytes total plaintext.
    let commands: &[u8] = &[
        0x14, b'T', b'L', b'I', b'!', 0x04, 0x00, 0x07, b'P', b'A', b'K', b'_', b'V', b'1', b'2',
    ];
    let q_size = u16::try_from(commands.len() - 1).expect("commands fit in u16");
    let q_be = q_size.to_be_bytes();
    let mut compressed = vec![0x8C, 0x02, q_be[0], q_be[1]];
    compressed.extend_from_slice(commands);
    let plaintext = b"TLI!TLI!TPAK_V12".to_vec();
    (compressed, plaintext)
}

pub(crate) fn write_fstring(out: &mut Vec<u8>, text: &str, as_utf16: bool) {
    if text.is_empty() {
        out.extend_from_slice(&0_i32.to_le_bytes());
        return;
    }
    if !as_utf16 {
        let len_with_nul = i32::try_from(text.len() + 1).expect("fstring len fits in i32");
        out.extend_from_slice(&len_with_nul.to_le_bytes());
        out.extend_from_slice(text.as_bytes());
        out.push(0);
        return;
    }
    let units: Vec<u16> = text.encode_utf16().chain(core::iter::once(0)).collect();
    let neg_len = -i32::try_from(units.len()).expect("utf16 units fit in i32");
    out.extend_from_slice(&neg_len.to_le_bytes());
    for unit in units {
        out.extend_from_slice(&unit.to_le_bytes());
    }
}

pub(crate) fn build_synthetic_pak(spec: &SyntheticPakSpec) -> Vec<u8> {
    let cipher = spec.key.as_ref().map(PakCipher::new);
    let dialect = spec.dialect();
    let mut disk = Vec::new();
    let mut records = Vec::with_capacity(spec.entries.len());
    for entry in &spec.entries {
        let rec = append_entry_to_disk(&mut disk, entry, dialect, cipher.as_ref());
        records.push(rec);
    }
    match spec.version {
        PakVersion::V8A | PakVersion::V8B | PakVersion::V9 => {
            append_legacy_index_and_footer(&mut disk, spec, &records, cipher.as_ref());
        }
        PakVersion::V10 | PakVersion::V11 | PakVersion::V12 => {
            append_split_index_and_footer(&mut disk, spec, &records, cipher.as_ref());
        }
    }
    disk
}

fn append_entry_to_disk(
    disk: &mut Vec<u8>,
    entry: &SyntheticEntry,
    dialect: PakDialect,
    cipher: Option<&PakCipher>,
) -> WrittenEntryRecord {
    let header_offset = u64::try_from(disk.len()).expect("disk len fits in u64");
    let raw_method_idx: u32 = match entry.method {
        CompressionMethod::None => 0,
        CompressionMethod::Zlib => 1,
        CompressionMethod::Oodle => 2,
    };
    let encoded = encode_entry_payload(entry, dialect, raw_method_idx != 0, cipher);
    let hashed_len = usize::try_from(encoded.compressed_total).expect("compressed_total fits");
    let payload_sha1 = *compute_sha1(&encoded.disk_bytes[..hashed_len]).as_bytes();
    let uncomp_u64 = u64::try_from(entry.payload.len()).expect("uncomp fits in u64");
    write_unencoded_fpak_entry(
        disk,
        &RawEntryWriteSpec {
            dialect,
            offset: 0,
            compressed_size: encoded.compressed_total,
            uncompressed_size: uncomp_u64,
            raw_method: raw_method_idx,
            sha1: payload_sha1,
            blocks: &encoded.rel_blocks,
            encrypted: entry.encrypted,
            block_size: entry.block_size,
        },
    );
    let expected_hdr = inline_header_bytes(
        dialect,
        raw_method_idx != 0,
        u16::try_from(encoded.rel_blocks.len()).expect("blocks fit in u16"),
    );
    assert_eq!(
        u64::try_from(disk.len()).expect("fits") - header_offset,
        u64::from(expected_hdr)
    );
    disk.extend_from_slice(&encoded.disk_bytes);
    WrittenEntryRecord {
        entry: entry.clone(),
        header_offset,
        compressed_total: encoded.compressed_total,
        raw_method_idx,
        block_raw_lens: encoded.block_raw_lens,
        rel_blocks: encoded.rel_blocks,
        payload_sha1,
    }
}

fn compress_entry_chunks(entry: &SyntheticEntry) -> Vec<Vec<u8>> {
    if let Some(custom) = &entry.precompressed_blocks {
        return custom.clone();
    }
    let chunk_size = if entry.block_size == 0 {
        entry.payload.len()
    } else {
        u32_to_usize(entry.block_size)
    };
    entry
        .payload
        .chunks(chunk_size)
        .map(|slice| match entry.method {
            CompressionMethod::Zlib => zlib_compress_chunk(slice),
            CompressionMethod::Oodle => oodle_encode_chunk(slice),
            CompressionMethod::None => slice.to_vec(),
        })
        .collect()
}

fn encode_entry_payload(
    entry: &SyntheticEntry,
    dialect: PakDialect,
    has_compression: bool,
    cipher: Option<&PakCipher>,
) -> EncodedPayload {
    if !has_compression {
        let mut raw = entry.payload.clone();
        let uncomp_len = u64::try_from(raw.len()).expect("len fits");
        if entry.encrypted {
            let aligned =
                usize::try_from(align_up_16(uncomp_len).expect("align")).expect("aligned fits");
            raw.resize(aligned, 0);
            cipher
                .expect("encrypted entry requires test cipher")
                .encrypt_in_place_for_test(&mut raw);
        }
        return EncodedPayload {
            disk_bytes: raw,
            block_raw_lens: Vec::new(),
            rel_blocks: Vec::new(),
            compressed_total: uncomp_len,
        };
    }
    if entry.payload.is_empty() {
        return EncodedPayload {
            disk_bytes: Vec::new(),
            block_raw_lens: Vec::new(),
            rel_blocks: Vec::new(),
            compressed_total: 0,
        };
    }
    let raw_chunks = compress_entry_chunks(entry);
    let blk_count_u16 = u16::try_from(raw_chunks.len()).expect("blocks fit in u16");
    let hdr_bytes = u64::from(inline_header_bytes(dialect, true, blk_count_u16));
    let mut disk_payload = Vec::new();
    let mut raw_lens = Vec::with_capacity(raw_chunks.len());
    let mut rel_blocks = Vec::with_capacity(raw_chunks.len());
    let mut comp_sum: u64 = 0;
    for mut blk in raw_chunks {
        let start_rel = hdr_bytes + u64::try_from(disk_payload.len()).expect("fits");
        let raw_len = u32::try_from(blk.len()).expect("blk fits in u32");
        raw_lens.push(raw_len);
        let end_rel = start_rel + u64::from(raw_len);
        rel_blocks.push((start_rel, end_rel));
        if entry.encrypted {
            let aligned =
                usize::try_from(align_up_16(u64::from(raw_len)).expect("align")).expect("fits");
            blk.resize(aligned, 0);
            cipher
                .expect("encrypted block requires test cipher")
                .encrypt_in_place_for_test(&mut blk);
        }
        comp_sum += u64::try_from(blk.len()).expect("aligned blk fits");
        disk_payload.extend_from_slice(&blk);
    }
    EncodedPayload {
        disk_bytes: disk_payload,
        block_raw_lens: raw_lens,
        rel_blocks,
        compressed_total: comp_sum,
    }
}

fn write_unencoded_fpak_entry(out: &mut Vec<u8>, spec: &RawEntryWriteSpec<'_>) {
    if spec.dialect == PakDialect::TorchlightInfiniteV12 {
        out.push(0);
    }
    out.extend_from_slice(&spec.offset.to_le_bytes());
    out.extend_from_slice(&spec.compressed_size.to_le_bytes());
    out.extend_from_slice(&spec.uncompressed_size.to_le_bytes());
    out.extend_from_slice(&spec.raw_method.to_le_bytes());
    out.extend_from_slice(&spec.sha1);
    if spec.raw_method != 0 {
        let cnt = u32::try_from(spec.blocks.len()).expect("cnt fits");
        out.extend_from_slice(&cnt.to_le_bytes());
        for &(s, e) in spec.blocks {
            out.extend_from_slice(&s.to_le_bytes());
            out.extend_from_slice(&e.to_le_bytes());
        }
    }
    let flags: u8 = if spec.encrypted {
        if spec.dialect == PakDialect::TorchlightInfiniteV12 {
            0x03
        } else {
            0x01
        }
    } else {
        0x00
    };
    out.push(flags);
    out.extend_from_slice(&spec.block_size.to_le_bytes());
}

fn finalize_section(
    disk: &mut Vec<u8>,
    mut section: Vec<u8>,
    encrypt: bool,
    cipher: Option<&PakCipher>,
) -> (u64, u64, [u8; 20]) {
    if encrypt {
        let len_u64 = u64::try_from(section.len()).expect("fits");
        let aligned = usize::try_from(align_up_16(len_u64).expect("align")).expect("fits");
        section.resize(aligned, 0);
    }
    let hash = *compute_sha1(&section).as_bytes();
    if encrypt {
        cipher
            .expect("encrypted index requires test cipher")
            .encrypt_in_place_for_test(&mut section);
    }
    let offset = u64::try_from(disk.len()).expect("fits");
    let size = u64::try_from(section.len()).expect("fits");
    disk.extend_from_slice(&section);
    (offset, size, hash)
}

fn append_legacy_index_and_footer(
    disk: &mut Vec<u8>,
    spec: &SyntheticPakSpec,
    records: &[WrittenEntryRecord],
    cipher: Option<&PakCipher>,
) {
    let mut idx = Vec::new();
    write_fstring(&mut idx, &spec.mount_point, spec.utf16_mount_point);
    let cnt = i32::try_from(records.len()).expect("fits");
    idx.extend_from_slice(&cnt.to_le_bytes());
    for rec in records {
        let joined = if rec.entry.dir.is_empty() || rec.entry.dir == "/" {
            rec.entry.file.clone()
        } else {
            format!("{}/{}", rec.entry.dir.trim_matches('/'), rec.entry.file)
        };
        write_fstring(&mut idx, &joined, rec.entry.utf16_strings);
        write_unencoded_fpak_entry(
            &mut idx,
            &RawEntryWriteSpec {
                dialect: spec.dialect(),
                offset: rec.header_offset,
                compressed_size: rec.compressed_total,
                uncompressed_size: u64::try_from(rec.entry.payload.len()).expect("fits"),
                raw_method: rec.raw_method_idx,
                sha1: rec.payload_sha1,
                blocks: &rec.rel_blocks,
                encrypted: rec.entry.encrypted,
                block_size: rec.entry.block_size,
            },
        );
    }
    let (idx_off, idx_sz, idx_hash) = finalize_section(disk, idx, spec.encrypt_index, cipher);
    append_footer(disk, spec, idx_off, idx_sz, idx_hash);
}

fn encode_packed_entry(out: &mut Vec<u8>, rec: &WrittenEntryRecord) -> i32 {
    let loc = i32::try_from(out.len()).expect("encoded offset fits in i32");
    let blk_cnt = u32::try_from(rec.block_raw_lens.len()).expect("fits");
    let (bsz_code, explicit_bsz) = if rec.entry.block_size == 0 {
        (0_u32, None)
    } else if (rec.entry.block_size & 0x7FF) == 0 && (rec.entry.block_size >> 11) < 0x3F {
        (rec.entry.block_size >> 11, None)
    } else {
        (0x3F_u32, Some(rec.entry.block_size))
    };
    let enc_bit = u32::from(rec.entry.encrypted);
    let bits: u32 = (1 << 31)
        | (1 << 30)
        | (1 << 29)
        | ((rec.raw_method_idx & 0x3F) << 23)
        | (enc_bit << 22)
        | ((blk_cnt & 0xFFFF) << 6)
        | bsz_code;
    out.extend_from_slice(&bits.to_le_bytes());
    if let Some(bsz) = explicit_bsz {
        out.extend_from_slice(&bsz.to_le_bytes());
    }
    let off32 = u32::try_from(rec.header_offset).expect("off32");
    let unc32 = u32::try_from(rec.entry.payload.len()).expect("unc32");
    out.extend_from_slice(&off32.to_le_bytes());
    out.extend_from_slice(&unc32.to_le_bytes());
    if rec.raw_method_idx != 0 {
        let comp32 = u32::try_from(rec.compressed_total).expect("comp32");
        out.extend_from_slice(&comp32.to_le_bytes());
    }
    if blk_cnt > 0 && (rec.entry.encrypted || blk_cnt != 1) {
        for &blen in &rec.block_raw_lens {
            out.extend_from_slice(&blen.to_le_bytes());
        }
    }
    loc
}

fn append_split_index_and_footer(
    disk: &mut Vec<u8>,
    spec: &SyntheticPakSpec,
    records: &[WrittenEntryRecord],
    cipher: Option<&PakCipher>,
) {
    let mut encoded_blob = Vec::new();
    let mut unencoded_blob = Vec::new();
    let mut unencoded_cnt: i32 = 0;
    let mut locations = Vec::with_capacity(records.len());
    for rec in records {
        if rec.entry.force_unencoded {
            let loc = -(unencoded_cnt + 1);
            unencoded_cnt += 1;
            write_unencoded_fpak_entry(
                &mut unencoded_blob,
                &RawEntryWriteSpec {
                    dialect: spec.dialect(),
                    offset: rec.header_offset,
                    compressed_size: rec.compressed_total,
                    uncompressed_size: u64::try_from(rec.entry.payload.len()).expect("fits"),
                    raw_method: rec.raw_method_idx,
                    sha1: rec.payload_sha1,
                    blocks: &rec.rel_blocks,
                    encrypted: rec.entry.encrypted,
                    block_size: rec.entry.block_size,
                },
            );
            locations.push(loc);
        } else {
            locations.push(encode_packed_entry(&mut encoded_blob, rec));
        }
    }
    let phi_bytes = build_phi_bytes(records, &locations, spec.path_hash_seed);
    let fdi_bytes = build_fdi_bytes(records, &locations);
    let (phi_off, phi_sz, phi_hash) = finalize_section(disk, phi_bytes, spec.encrypt_index, cipher);
    let (fdi_off, fdi_sz, fdi_hash) = finalize_section(disk, fdi_bytes, spec.encrypt_index, cipher);
    let mut primary = Vec::new();
    write_fstring(&mut primary, &spec.mount_point, spec.utf16_mount_point);
    let num_entries = i32::try_from(records.len()).expect("fits");
    primary.extend_from_slice(&num_entries.to_le_bytes());
    primary.extend_from_slice(&spec.path_hash_seed.to_le_bytes());
    primary.extend_from_slice(&1_u32.to_le_bytes());
    primary.extend_from_slice(&phi_off.to_le_bytes());
    primary.extend_from_slice(&phi_sz.to_le_bytes());
    primary.extend_from_slice(&phi_hash);
    primary.extend_from_slice(&1_u32.to_le_bytes());
    primary.extend_from_slice(&fdi_off.to_le_bytes());
    primary.extend_from_slice(&fdi_sz.to_le_bytes());
    primary.extend_from_slice(&fdi_hash);
    let enc_sz = i32::try_from(encoded_blob.len()).expect("fits");
    primary.extend_from_slice(&enc_sz.to_le_bytes());
    primary.extend_from_slice(&encoded_blob);
    primary.extend_from_slice(&unencoded_cnt.to_le_bytes());
    primary.extend_from_slice(&unencoded_blob);
    let (idx_off, idx_sz, idx_hash) = finalize_section(disk, primary, spec.encrypt_index, cipher);
    append_footer(disk, spec, idx_off, idx_sz, idx_hash);
}

fn build_phi_bytes(records: &[WrittenEntryRecord], locations: &[i32], seed: u64) -> Vec<u8> {
    assert_eq!(records.len(), locations.len());
    let mut phi = Vec::new();
    let cnt = u32::try_from(records.len()).expect("fits");
    phi.extend_from_slice(&cnt.to_le_bytes());
    for (rec, &loc) in records.iter().zip(locations.iter()) {
        let rel = if rec.entry.dir.is_empty() || rec.entry.dir == "/" {
            rec.entry.file.clone()
        } else {
            format!("{}/{}", rec.entry.dir.trim_matches('/'), rec.entry.file)
        };
        let hash = compute_ue4_path_hash(&rel, seed).expect("valid test path");
        phi.extend_from_slice(&hash.as_u64().to_le_bytes());
        phi.extend_from_slice(&loc.to_le_bytes());
    }
    phi
}

fn build_fdi_bytes(records: &[WrittenEntryRecord], locations: &[i32]) -> Vec<u8> {
    assert_eq!(records.len(), locations.len());
    let kept: Vec<(&WrittenEntryRecord, i32)> = records
        .iter()
        .zip(locations.iter().copied())
        .filter(|(r, _)| !r.entry.prune_directory_path)
        .collect();
    let mut fdi = Vec::new();
    let dir_cnt = u32::try_from(kept.len()).expect("fits");
    fdi.extend_from_slice(&dir_cnt.to_le_bytes());
    for (rec, loc) in kept {
        write_fstring(&mut fdi, &rec.entry.dir, rec.entry.utf16_strings);
        fdi.extend_from_slice(&1_u32.to_le_bytes());
        write_fstring(&mut fdi, &rec.entry.file, rec.entry.utf16_strings);
        fdi.extend_from_slice(&loc.to_le_bytes());
    }
    fdi
}

fn append_footer(
    disk: &mut Vec<u8>,
    spec: &SyntheticPakSpec,
    idx_off: u64,
    idx_sz: u64,
    idx_hash: [u8; 20],
) {
    disk.extend_from_slice(&spec.guid);
    disk.push(u8::from(spec.encrypt_index));
    let magic = if spec.version == PakVersion::V12 {
        MAGIC_TLI_V12
    } else {
        MAGIC_UE4
    };
    disk.extend_from_slice(&magic.to_le_bytes());
    disk.extend_from_slice(&spec.version.raw_number().to_le_bytes());
    if spec.version == PakVersion::V12 {
        disk.push(0);
    }
    disk.extend_from_slice(&idx_off.to_le_bytes());
    disk.extend_from_slice(&idx_sz.to_le_bytes());
    disk.extend_from_slice(&idx_hash);
    if spec.version == PakVersion::V9 {
        disk.push(u8::from(spec.is_v9_frozen));
    }
    let slots = if spec.version == PakVersion::V8A {
        4
    } else {
        5
    };
    for name in spec.method_names.iter().take(slots) {
        let mut slot = [0_u8; 32];
        let src = name.as_bytes();
        slot[..src.len()].copy_from_slice(src);
        disk.extend_from_slice(&slot);
    }
}

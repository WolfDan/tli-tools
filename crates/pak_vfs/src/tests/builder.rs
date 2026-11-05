//! Deterministic synthetic `.pak` and temporary game-tree generator for `pak_vfs` unit tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use pak::{CompressionMethod, PakDialect, PakVersion};

use crate::u32_to_usize;

static NEXT_TEST_ID: AtomicU32 = AtomicU32::new(1);

const MAGIC_UE4: u32 = 0x5A6F_12E1;
const MAGIC_TLI_V12: u32 = 0x6B2A_56B8;

/// Self-cleaning temporary directory under `std::env::temp_dir()`.
pub(crate) struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub(crate) fn new(label: &str) -> Self {
        let seq = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let dir_name = format!("tli_pak_vfs_{label}_{pid}_{seq}");
        let path = std::env::temp_dir().join(dir_name);
        fs::create_dir_all(&path).expect("temp dir creation must succeed");
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn write_pak(&self, rel_path: &str, spec: &TestPakSpec) -> PathBuf {
        let full = self.path.join(rel_path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("parent dir creation must succeed");
        }
        let bytes = build_test_pak(spec);
        fs::write(&full, bytes).expect("pak write must succeed");
        full
    }

    pub(crate) fn write_raw_file(&self, rel_path: &str, bytes: &[u8]) -> PathBuf {
        let full = self.path.join(rel_path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("parent dir creation must succeed");
        }
        fs::write(&full, bytes).expect("raw file write must succeed");
        full
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        match fs::remove_dir_all(&self.path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => panic!("failed to clean temporary directory {:?}: {err}", self.path),
        }
    }
}

#[derive(Clone)]
pub(crate) struct TestPakEntry {
    pub(crate) dir: String,
    pub(crate) file: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) method: CompressionMethod,
    pub(crate) block_size: u32,
    pub(crate) prune_directory_path: bool,
}

impl TestPakEntry {
    pub(crate) fn plain(dir: &str, file: &str, payload: &[u8]) -> Self {
        Self {
            dir: dir.to_owned(),
            file: file.to_owned(),
            payload: payload.to_vec(),
            method: CompressionMethod::None,
            block_size: 0,
            prune_directory_path: false,
        }
    }

    pub(crate) fn oodle(dir: &str, file: &str, payload: &[u8], block_size: u32) -> Self {
        Self {
            dir: dir.to_owned(),
            file: file.to_owned(),
            payload: payload.to_vec(),
            method: CompressionMethod::Oodle,
            block_size,
            prune_directory_path: false,
        }
    }
}

pub(crate) struct TestPakSpec {
    pub(crate) version: PakVersion,
    pub(crate) mount_point: String,
    pub(crate) entries: Vec<TestPakEntry>,
}

impl TestPakSpec {
    pub(crate) fn new_v12(mount_point: &str) -> Self {
        Self {
            version: PakVersion::V12,
            mount_point: mount_point.to_owned(),
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

struct WrittenTestEntry {
    entry: TestPakEntry,
    header_offset: u64,
    compressed_total: u64,
    raw_method_idx: u32,
    block_raw_lens: Vec<u32>,
}

pub(crate) fn build_test_pak(spec: &TestPakSpec) -> Vec<u8> {
    let dialect = spec.dialect();
    let mut disk = Vec::new();
    let mut records = Vec::with_capacity(spec.entries.len());
    for entry in &spec.entries {
        records.push(append_test_entry(&mut disk, entry, dialect));
    }
    append_split_index_and_footer(&mut disk, spec, &records);
    disk
}

fn append_test_entry(
    disk: &mut Vec<u8>,
    entry: &TestPakEntry,
    dialect: PakDialect,
) -> WrittenTestEntry {
    let header_offset = disk.len() as u64;
    let raw_method_idx: u32 = match entry.method {
        CompressionMethod::None => 0,
        CompressionMethod::Zlib => 1,
        CompressionMethod::Oodle => 2,
    };
    let (disk_bytes, block_raw_lens, rel_blocks) =
        encode_test_payload(entry, dialect, raw_method_idx != 0);
    let compressed_total = disk_bytes.len() as u64;
    let payload_sha1 = sha1_digest(&disk_bytes);
    write_inline_header(
        disk,
        &InlineHeaderSpec {
            dialect,
            offset: 0,
            comp_sz: compressed_total,
            uncomp_sz: entry.payload.len() as u64,
            raw_method: raw_method_idx,
            sha1: payload_sha1,
            blocks: &rel_blocks,
            block_size: entry.block_size,
        },
    );
    disk.extend_from_slice(&disk_bytes);
    WrittenTestEntry {
        entry: entry.clone(),
        header_offset,
        compressed_total,
        raw_method_idx,
        block_raw_lens,
    }
}

fn encode_test_payload(
    entry: &TestPakEntry,
    dialect: PakDialect,
    compressed: bool,
) -> (Vec<u8>, Vec<u32>, Vec<(u64, u64)>) {
    if !compressed || entry.payload.is_empty() {
        return (entry.payload.clone(), Vec::new(), Vec::new());
    }
    let chunk_size = if entry.block_size == 0 {
        entry.payload.len()
    } else {
        u32_to_usize(entry.block_size)
    };
    let chunks: Vec<Vec<u8>> = entry
        .payload
        .chunks(chunk_size)
        .map(oodle_raw_quantum)
        .collect();
    let blk_cnt = u32::try_from(chunks.len()).expect("fits");
    let base_hdr: u64 = if dialect == PakDialect::TorchlightInfiniteV12 {
        54
    } else {
        53
    };
    let hdr_bytes = base_hdr + 4 + u64::from(blk_cnt) * 16;
    let mut out = Vec::new();
    let mut raw_lens = Vec::with_capacity(chunks.len());
    let mut rel_blocks = Vec::with_capacity(chunks.len());
    for blk in chunks {
        let start_rel = hdr_bytes + (out.len() as u64);
        let len_u32 = u32::try_from(blk.len()).expect("fits");
        raw_lens.push(len_u32);
        rel_blocks.push((start_rel, start_rel + u64::from(len_u32)));
        out.extend_from_slice(&blk);
    }
    (out, raw_lens, rel_blocks)
}

fn oodle_raw_quantum(src: &[u8]) -> Vec<u8> {
    assert!(src.len() <= 0x40000);
    let mut out = Vec::with_capacity(src.len() + 2);
    out.extend_from_slice(&[0x4C, 0x06]);
    out.extend_from_slice(src);
    out
}

struct InlineHeaderSpec<'a> {
    dialect: PakDialect,
    offset: u64,
    comp_sz: u64,
    uncomp_sz: u64,
    raw_method: u32,
    sha1: [u8; 20],
    blocks: &'a [(u64, u64)],
    block_size: u32,
}

fn write_inline_header(out: &mut Vec<u8>, hdr: &InlineHeaderSpec<'_>) {
    if hdr.dialect == PakDialect::TorchlightInfiniteV12 {
        out.push(0);
    }
    out.extend_from_slice(&hdr.offset.to_le_bytes());
    out.extend_from_slice(&hdr.comp_sz.to_le_bytes());
    out.extend_from_slice(&hdr.uncomp_sz.to_le_bytes());
    out.extend_from_slice(&hdr.raw_method.to_le_bytes());
    out.extend_from_slice(&hdr.sha1);
    if hdr.raw_method != 0 {
        let cnt = u32::try_from(hdr.blocks.len()).expect("fits");
        out.extend_from_slice(&cnt.to_le_bytes());
        for &(s, e) in hdr.blocks {
            out.extend_from_slice(&s.to_le_bytes());
            out.extend_from_slice(&e.to_le_bytes());
        }
    }
    out.push(0);
    out.extend_from_slice(&hdr.block_size.to_le_bytes());
}

fn append_split_index_and_footer(
    disk: &mut Vec<u8>,
    spec: &TestPakSpec,
    records: &[WrittenTestEntry],
) {
    let mut encoded_blob = Vec::new();
    let mut locations = Vec::with_capacity(records.len());
    for rec in records {
        locations.push(encode_packed_entry(&mut encoded_blob, rec));
    }
    let phi_bytes = build_phi(&locations);
    let fdi_bytes = build_fdi(records, &locations);
    let (phi_off, phi_sz, phi_hash) = append_section(disk, &phi_bytes);
    let (fdi_off, fdi_sz, fdi_hash) = append_section(disk, &fdi_bytes);

    let mut primary = Vec::new();
    write_fstring(&mut primary, &spec.mount_point);
    let num_entries = i32::try_from(records.len()).expect("fits");
    primary.extend_from_slice(&num_entries.to_le_bytes());
    primary.extend_from_slice(&0_u64.to_le_bytes());
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
    primary.extend_from_slice(&0_i32.to_le_bytes());

    let (idx_off, idx_sz, idx_hash) = append_section(disk, &primary);
    append_footer(disk, spec.version, idx_off, idx_sz, idx_hash);
}

fn encode_packed_entry(out: &mut Vec<u8>, rec: &WrittenTestEntry) -> i32 {
    let loc = i32::try_from(out.len()).expect("fits");
    let blk_cnt = u32::try_from(rec.block_raw_lens.len()).expect("fits");
    let (bsz_code, explicit_bsz) = if rec.entry.block_size == 0 {
        (0_u32, None)
    } else if (rec.entry.block_size & 0x7FF) == 0 && (rec.entry.block_size >> 11) < 0x3F {
        (rec.entry.block_size >> 11, None)
    } else {
        (0x3F_u32, Some(rec.entry.block_size))
    };
    let bits: u32 = (1 << 31)
        | (1 << 30)
        | (1 << 29)
        | ((rec.raw_method_idx & 0x3F) << 23)
        | ((blk_cnt & 0xFFFF) << 6)
        | bsz_code;
    out.extend_from_slice(&bits.to_le_bytes());
    if let Some(bsz) = explicit_bsz {
        out.extend_from_slice(&bsz.to_le_bytes());
    }
    let off32 = u32::try_from(rec.header_offset).expect("fits");
    let unc32 = u32::try_from(rec.entry.payload.len()).expect("fits");
    out.extend_from_slice(&off32.to_le_bytes());
    out.extend_from_slice(&unc32.to_le_bytes());
    if rec.raw_method_idx != 0 {
        let comp32 = u32::try_from(rec.compressed_total).expect("fits");
        out.extend_from_slice(&comp32.to_le_bytes());
    }
    if blk_cnt > 1 {
        for &blen in &rec.block_raw_lens {
            out.extend_from_slice(&blen.to_le_bytes());
        }
    }
    loc
}

fn build_phi(locations: &[i32]) -> Vec<u8> {
    let mut phi = Vec::new();
    let cnt = u32::try_from(locations.len()).expect("fits");
    phi.extend_from_slice(&cnt.to_le_bytes());
    for (idx, &loc) in (0..cnt).zip(locations.iter()) {
        let hash = u64::from(idx + 1);
        phi.extend_from_slice(&hash.to_le_bytes());
        phi.extend_from_slice(&loc.to_le_bytes());
    }
    phi
}

fn build_fdi(records: &[WrittenTestEntry], locations: &[i32]) -> Vec<u8> {
    assert_eq!(records.len(), locations.len());
    let kept: Vec<(&WrittenTestEntry, i32)> = records
        .iter()
        .zip(locations.iter().copied())
        .filter(|(r, _)| !r.entry.prune_directory_path)
        .collect();
    let mut fdi = Vec::new();
    let dir_cnt = u32::try_from(kept.len()).expect("fits");
    fdi.extend_from_slice(&dir_cnt.to_le_bytes());
    for (rec, loc) in kept {
        write_fstring(&mut fdi, &rec.entry.dir);
        fdi.extend_from_slice(&1_u32.to_le_bytes());
        write_fstring(&mut fdi, &rec.entry.file);
        fdi.extend_from_slice(&loc.to_le_bytes());
    }
    fdi
}

fn write_fstring(out: &mut Vec<u8>, text: &str) {
    if text.is_empty() {
        out.extend_from_slice(&0_i32.to_le_bytes());
        return;
    }
    let len_with_nul = i32::try_from(text.len() + 1).expect("fits");
    out.extend_from_slice(&len_with_nul.to_le_bytes());
    out.extend_from_slice(text.as_bytes());
    out.push(0);
}

fn append_section(disk: &mut Vec<u8>, bytes: &[u8]) -> (u64, u64, [u8; 20]) {
    let hash = sha1_digest(bytes);
    let offset = disk.len() as u64;
    let size = bytes.len() as u64;
    disk.extend_from_slice(bytes);
    (offset, size, hash)
}

fn append_footer(
    disk: &mut Vec<u8>,
    version: PakVersion,
    idx_off: u64,
    idx_sz: u64,
    idx_hash: [u8; 20],
) {
    disk.extend_from_slice(&[0x22_u8; 16]);
    disk.push(0);
    let magic = if version == PakVersion::V12 {
        MAGIC_TLI_V12
    } else {
        MAGIC_UE4
    };
    disk.extend_from_slice(&magic.to_le_bytes());
    disk.extend_from_slice(&version.raw_number().to_le_bytes());
    if version == PakVersion::V12 {
        disk.push(0);
    }
    disk.extend_from_slice(&idx_off.to_le_bytes());
    disk.extend_from_slice(&idx_sz.to_le_bytes());
    disk.extend_from_slice(&idx_hash);
    for name in ["Zlib", "Oodle", "", "", ""] {
        let mut slot = [0_u8; 32];
        slot[..name.len()].copy_from_slice(name.as_bytes());
        disk.extend_from_slice(&slot);
    }
}

/// Compact pure-Rust FIPS 180-4 SHA-1 implementation for synthetic test `.pak` digests.
fn sha1_digest(data: &[u8]) -> [u8; 20] {
    let mut h = [
        0x6745_2301_u32,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];
    let bit_len = (data.len() as u64) * 8;
    let mut padded = data.to_vec();
    padded.push(0x80);
    for _ in 0..64 {
        if (padded.len() % 64) == 56 {
            break;
        }
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    let (blocks, rem) = padded.as_chunks::<64>();
    assert!(rem.is_empty());
    for block in blocks {
        sha1_compress_block(&mut h, block);
    }
    let mut out = [0_u8; 20];
    let (out_words, out_rem) = out.as_chunks_mut::<4>();
    assert!(out_rem.is_empty());
    assert_eq!(out_words.len(), h.len());
    for (dst_chunk, &word) in out_words.iter_mut().zip(h.iter()) {
        *dst_chunk = word.to_be_bytes();
    }
    out
}

fn sha1_compress_block(h: &mut [u32; 5], block: &[u8; 64]) {
    let mut w = [0_u32; 80];
    let (in_words, in_rem) = block.as_chunks::<4>();
    assert!(in_rem.is_empty());
    assert_eq!(in_words.len(), 16);
    for (slot, chunk) in w.iter_mut().take(16).zip(in_words.iter()) {
        *slot = u32::from_be_bytes(*chunk);
    }
    for i in 16..80_u32 {
        let idx = u32_to_usize(i);
        w[idx] = (w[idx - 3] ^ w[idx - 8] ^ w[idx - 14] ^ w[idx - 16]).rotate_left(1);
    }
    let [mut a, mut b, mut c, mut d, mut e] = *h;
    for (i, &wi) in (0..80_u32).zip(w.iter()) {
        let (f, k) = if i < 20 {
            ((b & c) | ((!b) & d), 0x5A82_7999_u32)
        } else if i < 40 {
            (b ^ c ^ d, 0x6ED9_EBA1_u32)
        } else if i < 60 {
            ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC_u32)
        } else {
            (b ^ c ^ d, 0xCA62_C1D6_u32)
        };
        let temp = a
            .rotate_left(5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(k)
            .wrapping_add(wi);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = temp;
    }
    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
}

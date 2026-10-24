//! Disk-backed flat binary index cache (`TARTPAK6`) and localization table cache (`TARTLNG1`)
//! for sub-millisecond cold-start lookups and SIMD arena scans.
//!
//! # Design (TigerStyle)
//! - **Control Plane:** Computes a 64-bit FNV-1a fingerprint over the AES key and all discovered
//!   `.pak` `(pak_name, is_premount, file_size, mtime_nanos)` tuples. If the cache file in
//!   `/tmp/tartarus-pak-v6-<fingerprint>.idx` exists, `PakVfs::open_game_dir` mounts in `< 0.1 ms`
//!   without opening a single `.pak` file.
//! - **Data Plane:**
//!   1. `get_record(virtual_path)` probes the on-disk open-addressed hash table via a single
//!      `pread64` window and reads the 64-byte [`PakEntryRecord`] in `< 10 µs`.
//!   2. `collect_substring_matches` filters 4 KB arena chunks using a 256-bit (`u64x4`) SIMD
//!      trigram Bloom filter before scanning candidate chunks with `memchr::memmem::Finder`.

use std::cell::OnceCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::simd::u64x4;

use byteorder::{ByteOrder, LittleEndian};

use crate::aes::Aes256KeySchedule;
use crate::error::ExtractError;
use crate::luac::parse_multi_language_bin;
use crate::pak::{
    CompressionKind, PATH_BYTES_MAX, PakArchive, PakEntryRecord, ascii_lowercase_into_simd,
    fnv1a_hash_64, read_exact_at_offset, split_normalized_dir_and_file,
};

const PAK_CACHE_MAGIC: &[u8; 8] = b"TARTPAK6";
const LANG_CACHE_MAGIC: &[u8; 8] = b"TARTLNG1";
const HEADER_LEN: u64 = 64;
const ARENA_HEADER_LEN: u64 = 48;
const SLOT_BYTES: usize = 16;
const RECORD_BYTES: usize = 64;
const DIR_ENTRY_BYTES: usize = 24;
const PROBE_WINDOW_SLOTS: usize = 16;
const CHUNK_TARGET_BYTES: usize = 4096;
const CHUNK_DESC_BYTES: usize = 40;

const _: () = {
    assert!(std::mem::size_of::<PakEntryRecord>() == RECORD_BYTES);
    assert!(HEADER_LEN == 64);
    assert!(SLOT_BYTES == 16);
    assert!(DIR_ENTRY_BYTES == 24);
};

/// 24-byte directory descriptor in the flat cache file.
#[derive(Clone, Copy, Debug)]
pub struct CachedDirEntry {
    pub dir_hash: u64,
    pub dir_off: u32,
    pub dir_len: u16,
    pub first_record_index: u32,
    pub record_count: u32,
}

impl CachedDirEntry {
    fn to_bytes(self) -> [u8; DIR_ENTRY_BYTES] {
        assert!(self.dir_len as usize <= PATH_BYTES_MAX);
        let mut out = [0u8; DIR_ENTRY_BYTES];
        LittleEndian::write_u64(&mut out[0..8], self.dir_hash);
        LittleEndian::write_u32(&mut out[8..12], self.dir_off);
        LittleEndian::write_u16(&mut out[12..14], self.dir_len);
        LittleEndian::write_u16(&mut out[14..16], 0);
        LittleEndian::write_u32(&mut out[16..20], self.first_record_index);
        LittleEndian::write_u32(&mut out[20..24], self.record_count);
        assert_eq!(out.len(), DIR_ENTRY_BYTES);
        out
    }

    fn from_bytes(b: &[u8; DIR_ENTRY_BYTES]) -> Self {
        assert_eq!(b.len(), DIR_ENTRY_BYTES);
        let entry = Self {
            dir_hash: LittleEndian::read_u64(&b[0..8]),
            dir_off: LittleEndian::read_u32(&b[8..12]),
            dir_len: LittleEndian::read_u16(&b[12..14]),
            first_record_index: LittleEndian::read_u32(&b[16..20]),
            record_count: LittleEndian::read_u32(&b[20..24]),
        };
        assert!(entry.dir_len as usize <= PATH_BYTES_MAX);
        entry
    }
}

#[derive(Debug)]
struct LoadedDirTable {
    dirs: Vec<CachedDirEntry>,
    dir_orig_arena: Vec<u8>,
    dir_lower_arena: Vec<u8>,
}

/// Handle to the validated `/tmp/tartarus-pak-v6-<fingerprint>.idx` flat binary index.
#[derive(Debug)]
pub struct PakFastIndex {
    file: File,
    pub fingerprint: u64,
    pub record_count: u32,
    pub dir_count: u32,
    pub table_mask: u32,
    pub table_offset: u64,
    pub records_offset: u64,
    pub dirs_offset: u64,
    pub arenas_offset: u64,
    dir_table: OnceCell<LoadedDirTable>,
}

#[inline]
fn non_zero_hash(h: u64) -> u64 {
    if h == 0 { 1 } else { h }
}

#[inline]
fn encode_record_idx_high_bit(record_idx: u32) -> [u8; 4] {
    assert!(record_idx < (1 << 28));
    let b = [
        0x80 | ((record_idx & 0x7F) as u8),
        0x80 | (((record_idx >> 7) & 0x7F) as u8),
        0x80 | (((record_idx >> 14) & 0x7F) as u8),
        0x80 | (((record_idx >> 21) & 0x7F) as u8),
    ];
    assert!(b[0] >= 0x80 && b[1] >= 0x80 && b[2] >= 0x80 && b[3] >= 0x80);
    b
}

#[inline]
fn decode_record_idx_high_bit(b: &[u8]) -> u32 {
    assert!(b.len() >= 4);
    assert!(b[0] >= 0x80 && b[1] >= 0x80 && b[2] >= 0x80 && b[3] >= 0x80);
    u32::from(b[0] & 0x7F)
        | (u32::from(b[1] & 0x7F) << 7)
        | (u32::from(b[2] & 0x7F) << 14)
        | (u32::from(b[3] & 0x7F) << 21)
}

/// Computes a 64-bit FNV-1a fingerprint over `key` and the ordered list of `archives`.
pub fn compute_vfs_fingerprint(key: &[u8; 32], archives: &[PakArchive]) -> u64 {
    assert_eq!(key.len(), 32);
    assert!(!archives.is_empty());
    let mut buf = Vec::with_capacity(64 + archives.len() * 48);
    buf.extend_from_slice(key);
    buf.extend_from_slice(&(archives.len() as u32).to_le_bytes());
    for arch in archives {
        buf.extend_from_slice(arch.pak_name.as_bytes());
        buf.push(u8::from(arch.is_premount));
        buf.extend_from_slice(&arch.file_size.to_le_bytes());
        buf.extend_from_slice(&arch.mtime_nanos.to_le_bytes());
    }
    let h = non_zero_hash(fnv1a_hash_64(&buf));
    assert_ne!(h, 0);
    h
}

fn working_cache_dir() -> PathBuf {
    let dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    assert!(!dir.as_os_str().is_empty());
    if let Some(parent) = dir.parent()
        && parent.file_name().is_some_and(|n| n == "crates")
        && let Some(workspace_root) = parent.parent()
    {
        return workspace_root.to_path_buf();
    }
    dir
}

fn cache_file_path(fingerprint: u64) -> PathBuf {
    assert_ne!(fingerprint, 0);
    let path = working_cache_dir().join(format!("tartarus-pak-v6-{fingerprint:016x}.idx"));
    assert!(!path.as_os_str().is_empty());
    path
}

#[inline]
fn compute_trigram_bloom(bytes: &[u8]) -> [u64; 4] {
    let mut bloom = [0u64; 4];
    if bytes.len() < 3 {
        return bloom;
    }
    for w in bytes.windows(3) {
        if (w[0] | w[1] | w[2]) < 0x80 {
            let v = u32::from(w[0]) | (u32::from(w[1]) << 8) | (u32::from(w[2]) << 16);
            let h = v.wrapping_mul(0x9E37_79B1);
            let lane = ((h >> 24) & 3) as usize;
            let bit = h >> 26;
            bloom[lane] |= 1u64 << bit;
        }
    }
    assert!(bloom.len() == 4);
    bloom
}

fn build_arena_chunk_index(arena: &[u8]) -> Vec<[u8; CHUNK_DESC_BYTES]> {
    if arena.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(arena.len() / CHUNK_TARGET_BYTES + 2);
    let mut offset = 0usize;
    while offset < arena.len() {
        let rem = arena.len() - offset;
        let len = if rem <= CHUNK_TARGET_BYTES {
            rem
        } else {
            find_last_entry_boundary(&arena[offset..offset + CHUNK_TARGET_BYTES])
        };
        assert!(len > 0);
        let bloom = compute_trigram_bloom(&arena[offset..offset + len]);
        let mut desc = [0u8; CHUNK_DESC_BYTES];
        LittleEndian::write_u32(&mut desc[0..4], offset as u32);
        LittleEndian::write_u32(&mut desc[4..8], len as u32);
        for (i, &w) in bloom.iter().enumerate() {
            LittleEndian::write_u64(&mut desc[8 + i * 8..16 + i * 8], w);
        }
        out.push(desc);
        offset += len;
    }
    assert_eq!(offset, arena.len());
    out
}

fn filter_matching_chunk_ranges_simd(chunks_raw: &[u8], needle: &[u8]) -> Vec<(u32, u32)> {
    assert!(chunks_raw.len().is_multiple_of(CHUNK_DESC_BYTES));
    let needle_bloom = u64x4::from_array(compute_trigram_bloom(needle));
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for desc in chunks_raw.as_chunks::<CHUNK_DESC_BYTES>().0 {
        let b = u64x4::from_array([
            LittleEndian::read_u64(&desc[8..16]),
            LittleEndian::read_u64(&desc[16..24]),
            LittleEndian::read_u64(&desc[24..32]),
            LittleEndian::read_u64(&desc[32..40]),
        ]);
        if (b & needle_bloom) == needle_bloom {
            let start = LittleEndian::read_u32(&desc[0..4]);
            let len = LittleEndian::read_u32(&desc[4..8]);
            if let Some(last) = ranges.last_mut()
                && last.0 + last.1 == start
                && last.1 + len <= 65536
            {
                last.1 += len;
            } else {
                ranges.push((start, len));
            }
        }
    }
    assert!(ranges.len() <= chunks_raw.len() / CHUNK_DESC_BYTES);
    ranges
}

#[inline]
fn strip_ue_pkg_ext(file_lower: &str) -> &str {
    assert!(!file_lower.is_empty());
    for ext in [".uasset", ".uexp", ".ubulk", ".uptnl"] {
        if let Some(stem) = file_lower.strip_suffix(ext)
            && !stem.is_empty()
        {
            return stem;
        }
    }
    file_lower
}

#[inline]
fn extract_stem_search_needle(lower_needle: &str) -> Option<&str> {
    assert!(!lower_needle.is_empty());
    if !lower_needle.is_ascii() {
        return None;
    }
    const PKG_EXTS: [&str; 4] = [".uasset", ".uexp", ".ubulk", ".uptnl"];
    for ext in PKG_EXTS {
        if ext.contains(lower_needle) {
            return None;
        }
    }
    if let Some(dot_pos) = lower_needle.rfind('.') {
        let suffix = &lower_needle[dot_pos..];
        if PKG_EXTS.iter().any(|ext| ext.starts_with(suffix)) {
            let before_dot = &lower_needle[..dot_pos];
            return (!before_dot.is_empty()).then_some(before_dot);
        }
    }
    Some(lower_needle)
}

#[inline]
fn find_last_entry_boundary(slice: &[u8]) -> usize {
    assert!(!slice.is_empty());
    let mut i = slice.len() - 1;
    while i > 0 {
        if slice[i - 1] >= 0x80 && slice[i] < 0x80 {
            return i;
        }
        i -= 1;
    }
    assert!(i == 0);
    slice.len()
}

#[inline]
fn push_ascii_sanitized(dst: &mut Vec<u8>, s: &str) {
    assert!(!s.is_empty());
    let start_len = dst.len();
    for &b in s.as_bytes() {
        dst.push(if b < 0x80 && b != 0 { b } else { b'_' });
    }
    assert_eq!(dst.len() - start_len, s.len());
}

fn merge_record_spans(mut raw_spans: Vec<(u32, u32, bool)>) -> Vec<(u32, u32, bool)> {
    if raw_spans.is_empty() {
        return raw_spans;
    }
    let in_len = raw_spans.len();
    raw_spans.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.2.cmp(&b.2)));
    let mut merged: Vec<(u32, u32, bool)> = Vec::with_capacity(in_len);
    for (mut s, mut c, v) in raw_spans {
        if c == 0 {
            continue;
        }
        let e = s + c;
        if let Some(last) = merged.last_mut() {
            let last_e = last.0 + last.1;
            if e <= last_e {
                continue;
            }
            if s <= last_e && last.2 == v && (e - last.0) <= 4096 {
                last.1 = e - last.0;
                continue;
            }
            if s < last_e {
                s = last_e;
                c = e - s;
            }
        }
        merged.push((s, c, v));
    }
    assert!(merged.len() <= in_len);
    merged
}

impl PakFastIndex {
    /// Attempts to open an existing `/tmp/tartarus-pak-v5-<fingerprint>.idx` file by reading
    /// only its 64-byte header.
    pub fn try_open(fingerprint: u64) -> Option<Self> {
        assert_ne!(fingerprint, 0);
        let path = cache_file_path(fingerprint);
        let file = File::open(&path).ok()?;
        let file_len = file.metadata().ok()?.len();
        if file_len < HEADER_LEN + ARENA_HEADER_LEN {
            return None;
        }
        let mut hdr = [0u8; 64];
        read_exact_at_offset(&file, &mut hdr, 0).ok()?;
        if &hdr[0..8] != PAK_CACHE_MAGIC || LittleEndian::read_u64(&hdr[8..16]) != fingerprint {
            return None;
        }
        let record_count = LittleEndian::read_u32(&hdr[16..20]);
        let dir_count = LittleEndian::read_u32(&hdr[20..24]);
        let table_mask = LittleEndian::read_u32(&hdr[24..28]);
        let table_offset = LittleEndian::read_u64(&hdr[32..40]);
        let records_offset = LittleEndian::read_u64(&hdr[40..48]);
        let dirs_offset = LittleEndian::read_u64(&hdr[48..56]);
        let arenas_offset = LittleEndian::read_u64(&hdr[56..64]);
        if record_count == 0 || dir_count == 0 || arenas_offset + ARENA_HEADER_LEN > file_len {
            return None;
        }
        assert!(table_mask.is_power_of_two() || (table_mask + 1).is_power_of_two());
        Some(Self {
            file,
            fingerprint,
            record_count,
            dir_count,
            table_mask,
            table_offset,
            records_offset,
            dirs_offset,
            arenas_offset,
            dir_table: OnceCell::new(),
        })
    }

    /// Probes the on-disk open-addressed hash table via `pread64` and returns the matching
    /// 64-byte [`PakEntryRecord`] in `< 10 µs`.
    pub fn get_record(&self, virtual_path: &str) -> Option<PakEntryRecord> {
        assert!(!virtual_path.is_empty());
        let (dir_part, file_part) = split_normalized_dir_and_file(virtual_path);
        if file_part.is_empty() || dir_part.len() + 1 + file_part.len() > PATH_BYTES_MAX {
            return None;
        }
        let mut full_lower = [0u8; PATH_BYTES_MAX];
        let full_len = if dir_part.is_empty() {
            ascii_lowercase_into_simd(file_part.as_bytes(), &mut full_lower[..file_part.len()]);
            file_part.len()
        } else {
            let dlen = dir_part.len();
            let flen = file_part.len();
            ascii_lowercase_into_simd(dir_part.as_bytes(), &mut full_lower[..dlen]);
            full_lower[dlen] = b'/';
            ascii_lowercase_into_simd(
                file_part.as_bytes(),
                &mut full_lower[dlen + 1..dlen + 1 + flen],
            );
            dlen + 1 + flen
        };
        let path_hash = non_zero_hash(fnv1a_hash_64(&full_lower[..full_len]));
        assert_ne!(path_hash, 0);
        self.probe_hash_table(path_hash, full_len as u32)
    }

    fn probe_hash_table(&self, path_hash: u64, full_len: u32) -> Option<PakEntryRecord> {
        assert_ne!(path_hash, 0);
        let table_cap = (self.table_mask as usize) + 1;
        assert!(table_cap.is_power_of_two());
        let mut slot = (path_hash as usize) & (self.table_mask as usize);
        let mut window_buf = [0u8; PROBE_WINDOW_SLOTS * SLOT_BYTES];

        for _ in 0..=table_cap / PROBE_WINDOW_SLOTS {
            let count = (table_cap - slot).min(PROBE_WINDOW_SLOTS);
            let byte_len = count * SLOT_BYTES;
            let off = self.table_offset + (slot * SLOT_BYTES) as u64;
            read_exact_at_offset(&self.file, &mut window_buf[..byte_len], off).ok()?;

            for i in 0..count {
                let s = &window_buf[i * SLOT_BYTES..(i + 1) * SLOT_BYTES];
                let shash = LittleEndian::read_u64(&s[0..8]);
                if shash == 0 {
                    return None;
                }
                let rec_idx = LittleEndian::read_u32(&s[8..12]);
                let slen = LittleEndian::read_u32(&s[12..16]);
                if shash == path_hash
                    && slen == full_len
                    && let Some(rec) = self.read_record_by_index(rec_idx)
                {
                    return Some(rec);
                }
            }
            slot = (slot + count) & (self.table_mask as usize);
        }
        None
    }

    fn read_record_by_index(&self, rec_idx: u32) -> Option<PakEntryRecord> {
        if rec_idx >= self.record_count {
            return None;
        }
        let mut buf = [0u8; RECORD_BYTES];
        let off = self.records_offset + u64::from(rec_idx) * (RECORD_BYTES as u64);
        read_exact_at_offset(&self.file, &mut buf, off).ok()?;
        let rec = PakEntryRecord::from_bytes(&buf);
        assert!(rec.size_uncompressed <= 64 * 1024 * 1024 * 1024);
        Some(rec)
    }

    fn read_dir_by_index(&self, dir_idx: u32) -> Option<CachedDirEntry> {
        if dir_idx >= self.dir_count {
            return None;
        }
        let mut buf = [0u8; DIR_ENTRY_BYTES];
        let off = self.dirs_offset + u64::from(dir_idx) * (DIR_ENTRY_BYTES as u64);
        read_exact_at_offset(&self.file, &mut buf, off).ok()?;
        let entry = CachedDirEntry::from_bytes(&buf);
        assert!(entry.dir_len as usize <= PATH_BYTES_MAX);
        Some(entry)
    }

    fn ensure_dir_table(&self) -> Result<&LoadedDirTable, ExtractError> {
        assert!(self.dir_count > 0);
        if let Some(tbl) = self.dir_table.get() {
            return Ok(tbl);
        }
        let mut arena_hdr = [0u8; 48];
        read_exact_at_offset(&self.file, &mut arena_hdr, self.arenas_offset)?;
        let dir_orig_len = LittleEndian::read_u64(&arena_hdr[0..8]) as usize;
        let dir_chunks_count = LittleEndian::read_u32(&arena_hdr[32..36]) as usize;
        let file_chunks_count = LittleEndian::read_u32(&arena_hdr[36..40]) as usize;
        let chunks_bytes = ((dir_chunks_count + file_chunks_count) * CHUNK_DESC_BYTES) as u64;

        let dirs_bytes_len = (self.dir_count as usize) * DIR_ENTRY_BYTES;
        let mut dirs_raw = vec![0u8; dirs_bytes_len];
        read_exact_at_offset(&self.file, &mut dirs_raw, self.dirs_offset)?;
        let mut dirs = Vec::with_capacity(self.dir_count as usize);
        for chunk in dirs_raw.as_chunks::<DIR_ENTRY_BYTES>().0 {
            dirs.push(CachedDirEntry::from_bytes(chunk));
        }

        let orig_off = self.arenas_offset + ARENA_HEADER_LEN + chunks_bytes;
        let mut dir_orig_arena = vec![0u8; dir_orig_len];
        read_exact_at_offset(&self.file, &mut dir_orig_arena, orig_off)?;
        let mut dir_lower_arena = vec![0u8; dir_orig_len];
        ascii_lowercase_into_simd(&dir_orig_arena, &mut dir_lower_arena);

        let _ = self.dir_table.set(LoadedDirTable {
            dirs,
            dir_orig_arena,
            dir_lower_arena,
        });
        assert!(self.dir_table.get().is_some());
        Ok(self.dir_table.get().unwrap())
    }

    /// Scans directories matching `dir_predicate(dir_lower)` and reads only matching directories'
    /// records and filenames from disk.
    pub fn collect_entries<D, F>(
        &self,
        mut dir_predicate: D,
        mut file_predicate: F,
    ) -> Vec<(String, PakEntryRecord)>
    where
        D: FnMut(&str) -> bool,
        F: FnMut(&str, &str) -> bool,
    {
        assert!(self.record_count > 0);
        let Ok(dtbl) = self.ensure_dir_table() else {
            return Vec::new();
        };
        let mut arena_hdr = [0u8; 48];
        if read_exact_at_offset(&self.file, &mut arena_hdr, self.arenas_offset).is_err() {
            return Vec::new();
        }
        let dir_orig_len = LittleEndian::read_u64(&arena_hdr[0..8]);
        let dir_lower_len = LittleEndian::read_u64(&arena_hdr[8..16]);
        let dir_chunks_count = LittleEndian::read_u32(&arena_hdr[32..36]) as usize;
        let file_chunks_count = LittleEndian::read_u32(&arena_hdr[36..40]) as usize;
        let chunks_bytes = ((dir_chunks_count + file_chunks_count) * CHUNK_DESC_BYTES) as u64;
        let file_orig_base =
            self.arenas_offset + ARENA_HEADER_LEN + chunks_bytes + dir_orig_len + dir_lower_len;

        let mut out = Vec::new();
        let mut file_lower_buf = [0u8; PATH_BYTES_MAX];

        for d in &dtbl.dirs {
            let ds = d.dir_off as usize;
            let de = ds + (d.dir_len as usize);
            let dir_lower = std::str::from_utf8(&dtbl.dir_lower_arena[ds..de]).unwrap_or("");
            if !dir_predicate(dir_lower) || d.record_count == 0 {
                continue;
            }
            let dir_orig = std::str::from_utf8(&dtbl.dir_orig_arena[ds..de]).unwrap_or("");
            self.collect_dir_records(
                d,
                (dir_orig, dir_lower),
                file_orig_base,
                &mut file_predicate,
                &mut file_lower_buf,
                &mut out,
            );
        }
        assert!(out.len() <= self.record_count as usize);
        out
    }

    fn collect_dir_records<F>(
        &self,
        d: &CachedDirEntry,
        (dir_orig, dir_lower): (&str, &str),
        file_orig_base: u64,
        file_predicate: &mut F,
        file_lower_buf: &mut [u8; PATH_BYTES_MAX],
        out: &mut Vec<(String, PakEntryRecord)>,
    ) where
        F: FnMut(&str, &str) -> bool,
    {
        assert!(d.record_count > 0);
        let count = d.record_count as usize;
        let mut recs_raw = vec![0u8; count * RECORD_BYTES];
        let rec_off = self.records_offset + u64::from(d.first_record_index) * (RECORD_BYTES as u64);
        if read_exact_at_offset(&self.file, &mut recs_raw, rec_off).is_err() {
            return;
        }
        let first_rec = PakEntryRecord::from_bytes(recs_raw[0..64].try_into().unwrap());
        let last_rec =
            PakEntryRecord::from_bytes(recs_raw[(count - 1) * 64..count * 64].try_into().unwrap());
        let f_start = first_rec.file_name_off as usize;
        let f_end = (last_rec.file_name_off as usize) + (last_rec.file_name_len as usize);
        if f_end < f_start {
            return;
        }
        let mut names_buf = vec![0u8; f_end - f_start];
        if read_exact_at_offset(&self.file, &mut names_buf, file_orig_base + f_start as u64)
            .is_err()
        {
            return;
        }

        for chunk in recs_raw.as_chunks::<RECORD_BYTES>().0 {
            let rec = PakEntryRecord::from_bytes(chunk);
            let rel_s = (rec.file_name_off as usize).saturating_sub(f_start);
            let flen = rec.file_name_len as usize;
            let Some(name_bytes) = names_buf.get(rel_s..rel_s + flen) else {
                continue;
            };
            ascii_lowercase_into_simd(name_bytes, &mut file_lower_buf[..flen]);
            let file_lower = std::str::from_utf8(&file_lower_buf[..flen]).unwrap_or("");
            if !file_predicate(dir_lower, file_lower) {
                continue;
            }
            let file_orig = std::str::from_utf8(name_bytes).unwrap_or("");
            let vpath = if dir_orig.is_empty() {
                file_orig.to_string()
            } else {
                format!("{dir_orig}/{file_orig}")
            };
            out.push((vpath, rec));
        }
    }

    /// Hardware SIMD (`u64x4` trigram Bloom filter + `memchr::memmem::Finder`) substring search
    /// across `dir_lower_arena` and `file_lower_arena`.
    pub fn collect_substring_matches(&self, lower_needle: &str) -> Vec<(String, PakEntryRecord)> {
        assert!(!lower_needle.is_empty());
        if let Some(slash_idx) = lower_needle.rfind('/') {
            let dir_part = &lower_needle[..slash_idx];
            let file_part = &lower_needle[slash_idx + 1..];
            let d_finder = memchr::memmem::Finder::new(dir_part.as_bytes());
            let f_finder = memchr::memmem::Finder::new(file_part.as_bytes());
            return self.collect_entries(
                |d| d_finder.find(d.as_bytes()).is_some(),
                |_, f| file_part.is_empty() || f_finder.find(f.as_bytes()).is_some(),
            );
        }
        let Some(stem_needle) = extract_stem_search_needle(lower_needle) else {
            let finder = memchr::memmem::Finder::new(lower_needle.as_bytes());
            return self.collect_entries(
                |_| true,
                |d, f| finder.find(d.as_bytes()).is_some() || finder.find(f.as_bytes()).is_some(),
            );
        };

        let mut arena_hdr = [0u8; 48];
        if read_exact_at_offset(&self.file, &mut arena_hdr, self.arenas_offset).is_err() {
            return Vec::new();
        }
        let dir_orig_len = LittleEndian::read_u64(&arena_hdr[0..8]);
        let dir_lower_len = LittleEndian::read_u64(&arena_hdr[8..16]);
        let file_orig_len = LittleEndian::read_u64(&arena_hdr[16..24]);
        let dir_chunks_count = LittleEndian::read_u32(&arena_hdr[32..36]) as usize;
        let file_chunks_count = LittleEndian::read_u32(&arena_hdr[36..40]) as usize;
        let dir_chunks_bytes = dir_chunks_count * CHUNK_DESC_BYTES;
        let total_chunks_bytes = (dir_chunks_count + file_chunks_count) * CHUNK_DESC_BYTES;

        let chunks_base = self.arenas_offset + ARENA_HEADER_LEN;
        let dir_orig_base = chunks_base + total_chunks_bytes as u64;
        let dir_lower_base = dir_orig_base + dir_orig_len;
        let file_orig_base = dir_lower_base + dir_lower_len;
        let file_lower_base = file_orig_base + file_orig_len;

        let mut chunks_raw = vec![0u8; total_chunks_bytes];
        if read_exact_at_offset(&self.file, &mut chunks_raw, chunks_base).is_err() {
            return Vec::new();
        }
        let dir_ranges = filter_matching_chunk_ranges_simd(
            &chunks_raw[..dir_chunks_bytes],
            lower_needle.as_bytes(),
        );
        let file_ranges = filter_matching_chunk_ranges_simd(
            &chunks_raw[dir_chunks_bytes..],
            stem_needle.as_bytes(),
        );

        let mut chunk_buf = [0u8; 65536];
        let mut raw_spans: Vec<(u32, u32, bool)> = Vec::with_capacity(64);
        let mut dir_cache: HashMap<u32, String> = HashMap::new();
        let full_finder = memchr::memmem::Finder::new(lower_needle.as_bytes());
        let stem_finder = memchr::memmem::Finder::new(stem_needle.as_bytes());

        self.scan_dir_chunks_simd(
            &full_finder,
            &mut chunk_buf,
            (dir_orig_base, dir_lower_base),
            &dir_ranges,
            &mut dir_cache,
            &mut raw_spans,
        );
        self.scan_file_chunks_simd(
            (&stem_finder, stem_needle.len() < lower_needle.len()),
            &mut chunk_buf,
            file_lower_base,
            &file_ranges,
            &mut raw_spans,
        );

        let merged_spans = merge_record_spans(raw_spans);
        let mut out = Vec::new();
        for span in merged_spans {
            self.emit_matched_stem_group(
                span,
                &full_finder,
                (dir_orig_base, file_orig_base),
                &mut dir_cache,
                &mut out,
            );
        }
        assert!(out.len() <= self.record_count as usize);
        out
    }

    fn scan_dir_chunks_simd(
        &self,
        finder: &memchr::memmem::Finder<'_>,
        buf: &mut [u8; 65536],
        (dir_orig_base, dir_lower_base): (u64, u64),
        ranges: &[(u32, u32)],
        dir_cache: &mut HashMap<u32, String>,
        raw_spans: &mut Vec<(u32, u32, bool)>,
    ) {
        let needle_len = finder.needle().len();
        assert!(needle_len > 0);
        let mut matched_dir_idxs: Vec<u32> = Vec::new();

        for &(start, len) in ranges {
            let n = (len as usize).min(buf.len());
            if read_exact_at_offset(&self.file, &mut buf[..n], dir_lower_base + u64::from(start))
                .is_err()
            {
                continue;
            }
            let mut cursor = 0usize;
            while cursor < n {
                let Some(rel) = finder.find(&buf[cursor..n]) else {
                    break;
                };
                let mut end = cursor + rel + needle_len;
                while end < n && buf[end] < 0x80 {
                    end += 1;
                }
                if end + 4 > n {
                    break;
                }
                matched_dir_idxs.push(decode_record_idx_high_bit(&buf[end..end + 4]));
                cursor = end + 4;
            }
        }
        self.preload_matched_dirs(&matched_dir_idxs, dir_orig_base, dir_cache, raw_spans);
    }

    fn preload_matched_dirs(
        &self,
        matched_dir_idxs: &[u32],
        dir_orig_base: u64,
        dir_cache: &mut HashMap<u32, String>,
        raw_spans: &mut Vec<(u32, u32, bool)>,
    ) {
        if matched_dir_idxs.is_empty() {
            return;
        }
        let min_d = *matched_dir_idxs.iter().min().unwrap();
        let max_d = *matched_dir_idxs.iter().max().unwrap();
        assert!(min_d <= max_d);
        if max_d < self.dir_count && max_d - min_d <= 512 {
            let count = (max_d - min_d + 1) as usize;
            let mut dirs_raw = vec![0u8; count * DIR_ENTRY_BYTES];
            let off = self.dirs_offset + u64::from(min_d) * (DIR_ENTRY_BYTES as u64);
            if read_exact_at_offset(&self.file, &mut dirs_raw, off).is_ok() {
                let first_d = CachedDirEntry::from_bytes(dirs_raw[0..24].try_into().unwrap());
                let last_d = CachedDirEntry::from_bytes(
                    dirs_raw[(count - 1) * 24..count * 24].try_into().unwrap(),
                );
                let o_start = first_d.dir_off as usize;
                let o_end = (last_d.dir_off as usize) + (last_d.dir_len as usize);
                let mut orig_buf = vec![0u8; o_end.saturating_sub(o_start)];
                let _ =
                    read_exact_at_offset(&self.file, &mut orig_buf, dir_orig_base + o_start as u64);
                for &d_idx in matched_dir_idxs {
                    let rel = (d_idx - min_d) as usize;
                    let d = CachedDirEntry::from_bytes(
                        dirs_raw[rel * 24..(rel + 1) * 24].try_into().unwrap(),
                    );
                    if d.record_count > 0 {
                        raw_spans.push((d.first_record_index, d.record_count, false));
                        let ds = (d.dir_off as usize).saturating_sub(o_start);
                        if let Some(s) = orig_buf.get(ds..ds + (d.dir_len as usize)) {
                            dir_cache.insert(d_idx, String::from_utf8_lossy(s).into_owned());
                        }
                    }
                }
                return;
            }
        }
        for &dir_idx in matched_dir_idxs {
            if let Some(d) = self.read_dir_by_index(dir_idx)
                && d.record_count > 0
            {
                raw_spans.push((d.first_record_index, d.record_count, false));
            }
        }
    }

    fn scan_file_chunks_simd(
        &self,
        (stem_finder, verify_full): (&memchr::memmem::Finder<'_>, bool),
        buf: &mut [u8; 65536],
        file_lower_base: u64,
        ranges: &[(u32, u32)],
        raw_spans: &mut Vec<(u32, u32, bool)>,
    ) {
        let needle_len = stem_finder.needle().len();
        assert!(needle_len > 0);
        for &(start, len) in ranges {
            let n = (len as usize).min(buf.len());
            if read_exact_at_offset(
                &self.file,
                &mut buf[..n],
                file_lower_base + u64::from(start),
            )
            .is_err()
            {
                continue;
            }
            let mut cursor = 0usize;
            while cursor < n {
                let Some(rel) = stem_finder.find(&buf[cursor..n]) else {
                    break;
                };
                let mut end = cursor + rel + needle_len;
                while end < n && buf[end] < 0x80 {
                    end += 1;
                }
                if end + 5 > n {
                    break;
                }
                let first_rec_idx = decode_record_idx_high_bit(&buf[end..end + 4]);
                let count = u32::from(buf[end + 4] & 0x7F);
                cursor = end + 5;
                raw_spans.push((first_rec_idx, count, verify_full));
            }
        }
    }

    fn emit_matched_stem_group(
        &self,
        (first_rec_idx, count, verify_full): (u32, u32, bool),
        full_finder: &memchr::memmem::Finder<'_>,
        (dir_orig_base, file_orig_base): (u64, u64),
        dir_cache: &mut HashMap<u32, String>,
        out: &mut Vec<(String, PakEntryRecord)>,
    ) {
        if count == 0 || first_rec_idx + count > self.record_count {
            return;
        }
        assert!(count <= 4096);
        let n = count as usize;
        let mut recs_raw = [0u8; 8 * RECORD_BYTES];
        let mut recs_vec;
        let recs_slice: &mut [u8] = if n <= 8 {
            &mut recs_raw[..n * RECORD_BYTES]
        } else {
            recs_vec = vec![0u8; n * RECORD_BYTES];
            &mut recs_vec
        };
        let rec_off = self.records_offset + u64::from(first_rec_idx) * (RECORD_BYTES as u64);
        if read_exact_at_offset(&self.file, recs_slice, rec_off).is_err() {
            return;
        }
        let first_rec = PakEntryRecord::from_bytes(recs_slice[0..64].try_into().unwrap());
        let last_rec =
            PakEntryRecord::from_bytes(recs_slice[(n - 1) * 64..n * 64].try_into().unwrap());
        let f_start = first_rec.file_name_off as usize;
        let f_end = (last_rec.file_name_off as usize) + (last_rec.file_name_len as usize);
        if f_end < f_start || f_end - f_start > PATH_BYTES_MAX * n {
            return;
        }
        let mut names_buf = vec![0u8; f_end - f_start];
        if read_exact_at_offset(&self.file, &mut names_buf, file_orig_base + f_start as u64)
            .is_err()
        {
            return;
        }
        let mut lower_scratch = [0u8; PATH_BYTES_MAX];

        for chunk in recs_slice.as_chunks::<RECORD_BYTES>().0 {
            let rec = PakEntryRecord::from_bytes(chunk);
            let rel_s = (rec.file_name_off as usize).saturating_sub(f_start);
            let flen = rec.file_name_len as usize;
            let Some(name_bytes) = names_buf.get(rel_s..rel_s + flen) else {
                continue;
            };
            if verify_full {
                ascii_lowercase_into_simd(name_bytes, &mut lower_scratch[..flen]);
                if full_finder.find(&lower_scratch[..flen]).is_none() {
                    continue;
                }
            }
            let dir_orig = self.resolve_dir_orig(rec.dir_index, dir_orig_base, dir_cache);
            let file_orig = std::str::from_utf8(name_bytes).unwrap_or("");
            let vpath = if dir_orig.is_empty() {
                file_orig.to_string()
            } else {
                format!("{dir_orig}/{file_orig}")
            };
            out.push((vpath, rec));
        }
    }

    fn resolve_dir_orig<'a>(
        &self,
        dir_idx: u32,
        dir_orig_base: u64,
        dir_cache: &'a mut HashMap<u32, String>,
    ) -> &'a str {
        assert!(dir_idx < self.dir_count);
        dir_cache.entry(dir_idx).or_insert_with(|| {
            let Some(d) = self.read_dir_by_index(dir_idx) else {
                return String::new();
            };
            let dlen = d.dir_len as usize;
            if dlen == 0 {
                return String::new();
            }
            let mut buf = [0u8; PATH_BYTES_MAX];
            if read_exact_at_offset(
                &self.file,
                &mut buf[..dlen],
                dir_orig_base + u64::from(d.dir_off),
            )
            .is_err()
            {
                return String::new();
            }
            String::from_utf8_lossy(&buf[..dlen]).into_owned()
        })
    }
}

struct TempDirBucket {
    dir_hash: u64,
    dir_orig: String,
    dir_lower: String,
    files: Vec<(String, String, PakEntryRecord)>,
}

/// Builds `/tmp/tartarus-pak-v5-<fingerprint>.idx` from `archives` and returns an open
/// [`PakFastIndex`] handle.
pub fn build_and_save_fast_index(
    fingerprint: u64,
    archives: &[PakArchive],
    key_schedule: &Aes256KeySchedule,
) -> Option<PakFastIndex> {
    assert_ne!(fingerprint, 0);
    assert!(!archives.is_empty());
    let (buckets, total_records) = collect_all_dir_buckets(archives, key_schedule)?;
    if total_records == 0 || buckets.is_empty() {
        return None;
    }

    let table_cap = (total_records * 2).next_power_of_two().max(64);
    let table_mask = (table_cap - 1) as u32;
    let mut hash_table = vec![[0u8; SLOT_BYTES]; table_cap];
    let mut records = Vec::with_capacity(total_records);
    let mut dirs = Vec::with_capacity(buckets.len());
    let mut dir_orig_arena = Vec::with_capacity(buckets.len() * 48);
    let mut dir_lower_arena = Vec::with_capacity(buckets.len() * 52);
    let mut file_orig_arena = Vec::with_capacity(total_records * 24);
    let mut file_lower_arena = Vec::with_capacity(total_records * 14);

    for mut bucket in buckets {
        bucket.files.sort_by(|a, b| {
            strip_ue_pkg_ext(&a.1)
                .cmp(strip_ue_pkg_ext(&b.1))
                .then_with(|| a.1.cmp(&b.1))
        });
        let dir_idx = dirs.len() as u32;
        let dir_off = dir_orig_arena.len() as u32;
        let dir_len = bucket.dir_orig.len() as u16;
        dir_orig_arena.extend_from_slice(bucket.dir_orig.as_bytes());
        if !bucket.dir_lower.is_empty() {
            push_ascii_sanitized(&mut dir_lower_arena, &bucket.dir_lower);
            dir_lower_arena.extend_from_slice(&encode_record_idx_high_bit(dir_idx));
        }
        let first_record_index = records.len() as u32;
        let record_count = bucket.files.len() as u32;

        append_bucket_files_to_arenas(
            dir_idx,
            &bucket.dir_lower,
            bucket.files,
            table_mask as usize,
            &mut hash_table,
            &mut records,
            &mut file_orig_arena,
            &mut file_lower_arena,
        );

        dirs.push(CachedDirEntry {
            dir_hash: bucket.dir_hash,
            dir_off,
            dir_len,
            first_record_index,
            record_count,
        });
    }

    write_fast_index_file(
        fingerprint,
        table_mask,
        &hash_table,
        &records,
        &dirs,
        (
            &dir_orig_arena,
            &dir_lower_arena,
            &file_orig_arena,
            &file_lower_arena,
        ),
    )
    .ok()?;
    PakFastIndex::try_open(fingerprint)
}

#[allow(clippy::too_many_arguments)]
fn append_bucket_files_to_arenas(
    dir_idx: u32,
    dir_lower: &str,
    files: Vec<(String, String, PakEntryRecord)>,
    table_mask: usize,
    hash_table: &mut [[u8; SLOT_BYTES]],
    records: &mut Vec<PakEntryRecord>,
    file_orig_arena: &mut Vec<u8>,
    file_lower_arena: &mut Vec<u8>,
) {
    assert!(!files.is_empty());
    let mut group_stem = String::new();
    let mut group_first_idx = 0u32;
    let mut group_count = 0u8;

    for (file_orig, file_lower, mut rec) in files {
        let rec_idx = records.len() as u32;
        let file_off = file_orig_arena.len() as u32;
        let file_len = file_orig.len() as u16;
        file_orig_arena.extend_from_slice(file_orig.as_bytes());

        let stem = strip_ue_pkg_ext(&file_lower);
        if group_count > 0 && stem == group_stem && group_count < 127 {
            group_count += 1;
        } else {
            if group_count > 0 {
                push_ascii_sanitized(file_lower_arena, &group_stem);
                file_lower_arena.extend_from_slice(&encode_record_idx_high_bit(group_first_idx));
                file_lower_arena.push(0x80 | group_count);
            }
            group_stem.clear();
            group_stem.push_str(stem);
            group_first_idx = rec_idx;
            group_count = 1;
        }

        rec.dir_index = dir_idx;
        rec.file_name_off = file_off;
        rec.file_name_len = file_len;
        records.push(rec);

        let full_len = if dir_lower.is_empty() {
            file_lower.len() as u32
        } else {
            (dir_lower.len() + 1 + file_lower.len()) as u32
        };
        insert_hash_slot(hash_table, table_mask, rec.path_hash, rec_idx, full_len);
    }

    if group_count > 0 {
        push_ascii_sanitized(file_lower_arena, &group_stem);
        file_lower_arena.extend_from_slice(&encode_record_idx_high_bit(group_first_idx));
        file_lower_arena.push(0x80 | group_count);
    }
}

fn insert_hash_slot(
    hash_table: &mut [[u8; SLOT_BYTES]],
    mask: usize,
    path_hash: u64,
    rec_idx: u32,
    full_len: u32,
) {
    assert_ne!(path_hash, 0);
    assert!(mask < hash_table.len());
    let mut slot = (path_hash as usize) & mask;
    while LittleEndian::read_u64(&hash_table[slot][0..8]) != 0 {
        slot = (slot + 1) & mask;
    }
    LittleEndian::write_u64(&mut hash_table[slot][0..8], path_hash);
    LittleEndian::write_u32(&mut hash_table[slot][8..12], rec_idx);
    LittleEndian::write_u32(&mut hash_table[slot][12..16], full_len);
}

fn collect_all_dir_buckets(
    archives: &[PakArchive],
    key_schedule: &Aes256KeySchedule,
) -> Option<(Vec<TempDirBucket>, usize)> {
    assert!(!archives.is_empty());
    let mut buckets: Vec<TempDirBucket> = Vec::new();
    let mut dir_map: HashMap<String, usize> = HashMap::new();
    let mut seen_hashes: HashSet<(u64, u32)> = HashSet::new();
    let mut total_records = 0usize;

    for (pak_idx, archive) in archives.iter().enumerate() {
        archive.scan_all_entries_for_cache(
            pak_idx as u16,
            key_schedule,
            &mut |(dir_orig, dir_lower, dir_hash), (file_orig, file_lower), mut rec| {
                let full_len = if dir_lower.is_empty() {
                    file_lower.len()
                } else {
                    dir_lower.len() + 1 + file_lower.len()
                };
                let mut full_buf = [0u8; PATH_BYTES_MAX];
                if full_len > PATH_BYTES_MAX {
                    return;
                }
                if dir_lower.is_empty() {
                    full_buf[..file_lower.len()].copy_from_slice(file_lower.as_bytes());
                } else {
                    let dl = dir_lower.len();
                    full_buf[..dl].copy_from_slice(dir_lower.as_bytes());
                    full_buf[dl] = b'/';
                    full_buf[dl + 1..full_len].copy_from_slice(file_lower.as_bytes());
                }
                let path_hash = non_zero_hash(fnv1a_hash_64(&full_buf[..full_len]));
                if !seen_hashes.insert((path_hash, full_len as u32)) {
                    return;
                }
                rec.path_hash = path_hash;
                let b_idx = if let Some(&idx) = dir_map.get(dir_lower) {
                    idx
                } else {
                    let idx = buckets.len();
                    dir_map.insert(dir_lower.to_string(), idx);
                    buckets.push(TempDirBucket {
                        dir_hash,
                        dir_orig: dir_orig.to_string(),
                        dir_lower: dir_lower.to_string(),
                        files: Vec::new(),
                    });
                    idx
                };
                buckets[b_idx]
                    .files
                    .push((file_orig.to_string(), file_lower.to_string(), rec));
                total_records += 1;
            },
        );
    }
    assert!(total_records >= buckets.len() || total_records == 0);
    Some((buckets, total_records))
}

fn write_fast_index_file(
    fingerprint: u64,
    table_mask: u32,
    hash_table: &[[u8; SLOT_BYTES]],
    records: &[PakEntryRecord],
    dirs: &[CachedDirEntry],
    (dir_orig_arena, dir_lower_arena, file_orig_arena, file_lower_arena): (
        &[u8],
        &[u8],
        &[u8],
        &[u8],
    ),
) -> std::io::Result<()> {
    assert_ne!(fingerprint, 0);
    assert!(!records.is_empty());
    let final_path = cache_file_path(fingerprint);
    let tmp_path = final_path.with_extension(format!("idx.tmp.{}", std::process::id()));
    let file = File::create(&tmp_path)?;
    let mut w = BufWriter::with_capacity(256 * 1024, file);

    let table_offset = HEADER_LEN;
    let records_offset = table_offset + (hash_table.len() * SLOT_BYTES) as u64;
    let dirs_offset = records_offset + (records.len() * RECORD_BYTES) as u64;
    let arenas_offset = dirs_offset + (dirs.len() * DIR_ENTRY_BYTES) as u64;

    let mut hdr = [0u8; 64];
    hdr[0..8].copy_from_slice(PAK_CACHE_MAGIC);
    LittleEndian::write_u64(&mut hdr[8..16], fingerprint);
    LittleEndian::write_u32(&mut hdr[16..20], records.len() as u32);
    LittleEndian::write_u32(&mut hdr[20..24], dirs.len() as u32);
    LittleEndian::write_u32(&mut hdr[24..28], table_mask);
    LittleEndian::write_u64(&mut hdr[32..40], table_offset);
    LittleEndian::write_u64(&mut hdr[40..48], records_offset);
    LittleEndian::write_u64(&mut hdr[48..56], dirs_offset);
    LittleEndian::write_u64(&mut hdr[56..64], arenas_offset);
    w.write_all(&hdr)?;

    for slot in hash_table {
        w.write_all(slot)?;
    }
    for rec in records {
        w.write_all(&rec.to_bytes())?;
    }
    for d in dirs {
        w.write_all(&d.to_bytes())?;
    }

    let dir_chunks = build_arena_chunk_index(dir_lower_arena);
    let file_chunks = build_arena_chunk_index(file_lower_arena);
    let mut arena_hdr = [0u8; 48];
    LittleEndian::write_u64(&mut arena_hdr[0..8], dir_orig_arena.len() as u64);
    LittleEndian::write_u64(&mut arena_hdr[8..16], dir_lower_arena.len() as u64);
    LittleEndian::write_u64(&mut arena_hdr[16..24], file_orig_arena.len() as u64);
    LittleEndian::write_u64(&mut arena_hdr[24..32], file_lower_arena.len() as u64);
    LittleEndian::write_u32(&mut arena_hdr[32..36], dir_chunks.len() as u32);
    LittleEndian::write_u32(&mut arena_hdr[36..40], file_chunks.len() as u32);
    w.write_all(&arena_hdr)?;
    for c in &dir_chunks {
        w.write_all(c)?;
    }
    for c in &file_chunks {
        w.write_all(c)?;
    }
    w.write_all(dir_orig_arena)?;
    w.write_all(dir_lower_arena)?;
    w.write_all(file_orig_arena)?;
    w.write_all(file_lower_arena)?;
    w.flush()?;
    drop(w);

    fs::rename(&tmp_path, &final_path)?;
    assert!(final_path.exists());
    Ok(())
}

/// Reads or builds a sorted binary localization cache `./tartarus-lang-v1-<lang>-<off>-<sz>.idx`
/// so prefix queries (`s16_gameplay_`) execute via binary search + `pread64` in `< 0.1 ms`.
pub fn read_or_cache_multi_language<F>(
    lang_code: &str,
    rec: &PakEntryRecord,
    prefix_filter: Option<&str>,
    extract_fn: F,
) -> Result<BTreeMap<String, String>, ExtractError>
where
    F: FnOnce() -> Result<Vec<u8>, ExtractError>,
{
    assert!(!lang_code.is_empty());
    let cache_path = working_cache_dir().join(format!(
        "tartarus-lang-v1-{}-{:x}-{:x}.idx",
        lang_code, rec.offset, rec.size_compressed
    ));
    if let Some(map) = try_query_lang_cache(&cache_path, prefix_filter) {
        return Ok(map);
    }

    let raw = extract_fn()?;
    let full_map = parse_multi_language_bin(&raw, None);
    let _ = write_lang_cache(&cache_path, &full_map);

    let Some(prefix) = prefix_filter.filter(|p| !p.is_empty()) else {
        return Ok(full_map);
    };
    let mut filtered = BTreeMap::new();
    for (k, v) in full_map.range(prefix.to_string()..) {
        if !k.starts_with(prefix) {
            break;
        }
        filtered.insert(k.clone(), v.clone());
    }
    assert!(filtered.len() <= full_map.len());
    Ok(filtered)
}

fn try_query_lang_cache(
    cache_path: &Path,
    prefix_filter: Option<&str>,
) -> Option<BTreeMap<String, String>> {
    assert!(!cache_path.as_os_str().is_empty());
    let file = File::open(cache_path).ok()?;
    let mut hdr = [0u8; 32];
    read_exact_at_offset(&file, &mut hdr, 0).ok()?;
    if &hdr[0..8] != LANG_CACHE_MAGIC {
        return None;
    }
    let count = LittleEndian::read_u32(&hdr[8..12]) as usize;
    let idx_off = u64::from(LittleEndian::read_u32(&hdr[12..16]));
    let data_off = u64::from(LittleEndian::read_u32(&hdr[16..20]));
    if count == 0 {
        return Some(BTreeMap::new());
    }

    if let Some(prefix) = prefix_filter.filter(|p| !p.is_empty()) {
        return query_lang_cache_prefix(&file, count, idx_off, data_off, prefix);
    }

    let data_len = LittleEndian::read_u32(&hdr[20..24]) as usize;
    let mut idx_raw = vec![0u8; count * 12];
    let mut data_raw = vec![0u8; data_len];
    read_exact_at_offset(&file, &mut idx_raw, idx_off).ok()?;
    read_exact_at_offset(&file, &mut data_raw, data_off).ok()?;
    decode_lang_slice(&idx_raw, &data_raw, 0, None)
}

fn query_lang_cache_prefix(
    file: &File,
    count: usize,
    idx_off: u64,
    data_off: u64,
    prefix: &str,
) -> Option<BTreeMap<String, String>> {
    assert!(count > 0 && !prefix.is_empty());
    let mut lo = 0usize;
    let mut hi = count;
    let mut key_buf = [0u8; 512];
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let mut entry = [0u8; 12];
        read_exact_at_offset(file, &mut entry, idx_off + (mid * 12) as u64).ok()?;
        let k_off = LittleEndian::read_u32(&entry[0..4]);
        let k_len = (LittleEndian::read_u16(&entry[4..6]) as usize).min(512);
        read_exact_at_offset(file, &mut key_buf[..k_len], data_off + u64::from(k_off)).ok()?;
        if &key_buf[..k_len] < prefix.as_bytes() {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo >= count {
        return Some(BTreeMap::new());
    }
    let window_entries = (count - lo).min(4096);
    let mut idx_raw = vec![0u8; window_entries * 12];
    read_exact_at_offset(file, &mut idx_raw, idx_off + (lo * 12) as u64).ok()?;

    let first_k_off = LittleEndian::read_u32(&idx_raw[0..4]);
    let mut matched = 0usize;
    let mut end_byte = first_k_off;
    for chunk in idx_raw.as_chunks::<12>().0 {
        let v_len = u32::from(LittleEndian::read_u16(&chunk[6..8]));
        let v_off = LittleEndian::read_u32(&chunk[8..12]);
        end_byte = v_off + v_len;
        matched += 1;
    }
    let slice_len = (end_byte - first_k_off) as usize;
    let mut data_raw = vec![0u8; slice_len];
    read_exact_at_offset(file, &mut data_raw, data_off + u64::from(first_k_off)).ok()?;
    decode_lang_slice(
        &idx_raw[..matched * 12],
        &data_raw,
        first_k_off,
        Some(prefix),
    )
}

fn decode_lang_slice(
    idx_raw: &[u8],
    data_raw: &[u8],
    base_data_off: u32,
    prefix: Option<&str>,
) -> Option<BTreeMap<String, String>> {
    assert!(idx_raw.len().is_multiple_of(12));
    let mut out = BTreeMap::new();
    for chunk in idx_raw.as_chunks::<12>().0 {
        let k_off = (LittleEndian::read_u32(&chunk[0..4]) - base_data_off) as usize;
        let k_len = LittleEndian::read_u16(&chunk[4..6]) as usize;
        let v_len = LittleEndian::read_u16(&chunk[6..8]) as usize;
        let v_off = (LittleEndian::read_u32(&chunk[8..12]) - base_data_off) as usize;
        let k_str = std::str::from_utf8(data_raw.get(k_off..k_off + k_len)?).ok()?;
        if let Some(p) = prefix
            && !k_str.starts_with(p)
        {
            break;
        }
        let v_str = std::str::from_utf8(data_raw.get(v_off..v_off + v_len)?).ok()?;
        out.insert(k_str.to_string(), v_str.to_string());
    }
    Some(out)
}

fn write_lang_cache(cache_path: &Path, map: &BTreeMap<String, String>) -> std::io::Result<()> {
    assert!(!cache_path.as_os_str().is_empty());
    let tmp_path = cache_path.with_extension(format!("idx.tmp.{}", std::process::id()));
    let file = File::create(&tmp_path)?;
    let mut w = BufWriter::with_capacity(256 * 1024, file);

    let count = map.len() as u32;
    let idx_off = 32u32;
    let data_off = idx_off + count * 12;
    let mut data_arena = Vec::with_capacity(map.len() * 64);
    let mut idx_arena = Vec::with_capacity(map.len() * 12);

    for (k, v) in map {
        let k_bytes = k.as_bytes();
        let v_bytes = v.as_bytes();
        if k_bytes.len() > u16::MAX as usize || v_bytes.len() > u16::MAX as usize {
            continue;
        }
        let k_off = data_arena.len() as u32;
        data_arena.extend_from_slice(k_bytes);
        let v_off = data_arena.len() as u32;
        data_arena.extend_from_slice(v_bytes);

        let mut entry = [0u8; 12];
        LittleEndian::write_u32(&mut entry[0..4], k_off);
        LittleEndian::write_u16(&mut entry[4..6], k_bytes.len() as u16);
        LittleEndian::write_u16(&mut entry[6..8], v_bytes.len() as u16);
        LittleEndian::write_u32(&mut entry[8..12], v_off);
        idx_arena.extend_from_slice(&entry);
    }

    let mut hdr = [0u8; 32];
    hdr[0..8].copy_from_slice(LANG_CACHE_MAGIC);
    LittleEndian::write_u32(&mut hdr[8..12], (idx_arena.len() / 12) as u32);
    LittleEndian::write_u32(&mut hdr[12..16], idx_off);
    LittleEndian::write_u32(&mut hdr[16..20], data_off);
    LittleEndian::write_u32(&mut hdr[20..24], data_arena.len() as u32);
    w.write_all(&hdr)?;
    w.write_all(&idx_arena)?;
    w.write_all(&data_arena)?;
    w.flush()?;
    drop(w);
    fs::rename(&tmp_path, cache_path)?;
    assert!(cache_path.exists());
    Ok(())
}

impl PakEntryRecord {
    /// Serializes this 64-byte [`PakEntryRecord`] into a fixed 64-byte little-endian array.
    pub fn to_bytes(&self) -> [u8; RECORD_BYTES] {
        assert_eq!(std::mem::size_of::<Self>(), RECORD_BYTES);
        let mut out = [0u8; RECORD_BYTES];
        LittleEndian::write_u64(&mut out[0..8], self.offset);
        LittleEndian::write_u64(&mut out[8..16], self.size_compressed);
        LittleEndian::write_u64(&mut out[16..24], self.size_uncompressed);
        LittleEndian::write_u64(&mut out[24..32], self.path_hash);
        LittleEndian::write_u32(&mut out[32..36], self.block_size_bytes);
        LittleEndian::write_u32(&mut out[36..40], self.dir_index);
        LittleEndian::write_u32(&mut out[40..44], self.file_name_off);
        LittleEndian::write_u16(&mut out[44..46], self.file_name_len);
        LittleEndian::write_u16(&mut out[46..48], self.blocks_count);
        LittleEndian::write_u16(&mut out[48..50], self.pak_index);
        out[50] = self.compression as u8;
        out[51] = u8::from(self.encrypted);
        out[52] = u8::from(self.is_premount);
        assert_eq!(out.len(), RECORD_BYTES);
        out
    }

    /// Deserializes a 64-byte [`PakEntryRecord`] from a fixed 64-byte little-endian array.
    pub fn from_bytes(b: &[u8; RECORD_BYTES]) -> Self {
        assert_eq!(b.len(), RECORD_BYTES);
        let compression = match b[50] {
            0 => CompressionKind::None,
            1 => CompressionKind::Oodle,
            2 => CompressionKind::Zlib,
            _ => CompressionKind::Unknown,
        };
        Self {
            offset: LittleEndian::read_u64(&b[0..8]),
            size_compressed: LittleEndian::read_u64(&b[8..16]),
            size_uncompressed: LittleEndian::read_u64(&b[16..24]),
            path_hash: LittleEndian::read_u64(&b[24..32]),
            block_size_bytes: LittleEndian::read_u32(&b[32..36]),
            dir_index: LittleEndian::read_u32(&b[36..40]),
            file_name_off: LittleEndian::read_u32(&b[40..44]),
            file_name_len: LittleEndian::read_u16(&b[44..46]),
            blocks_count: LittleEndian::read_u16(&b[46..48]),
            pak_index: LittleEndian::read_u16(&b[48..50]),
            compression,
            encrypted: b[51] != 0,
            is_premount: b[52] != 0,
            reserved: [0u8; 11],
        }
    }
}

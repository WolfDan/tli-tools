//! Zero-allocation UE4 UTF-16LE FNV-1a path hashing, contiguous `PathArena`, and flat hash lookup.

use core::hash::Hasher as _;

use fnv::FnvHasher;

use crate::error::{LimitKind, PakError};
use crate::low_u32;
use crate::types::{
    ENTRIES_COUNT_HARD_MAX, EntryId, PATH_BYTES_MAX, PATH_UTF16_UNITS_MAX, PathHash,
};
use crate::u32_to_usize;

const FNV1A_64_OFFSET_BASIS: u64 = 0xCBF2_9CE4_8422_2325;

#[inline]
fn trim_leading_separators(path: &str) -> &str {
    path.trim_start_matches(['/', '\\'])
}

#[inline]
fn normalize_fold_char(ch: char) -> char {
    if ch == '\\' {
        return '/';
    }
    if ch.is_ascii() {
        return ch.to_ascii_lowercase();
    }
    let mut lower = ch.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(single), None) => single,
        _ => ch,
    }
}

/// Computes Unreal Engine's seeded 64-bit FNV-1a digest over normalized, lowercased UTF-16LE bytes.
///
/// Streams 2 bytes per UTF-16LE code unit directly into [`FnvHasher`] with zero heap allocations
/// and zero intermediate stack buffer copies.
pub(crate) fn compute_ue4_path_hash(rel_path: &str, seed: u64) -> Result<PathHash, PakError> {
    let trimmed = trim_leading_separators(rel_path);
    if trimmed.len() > u32_to_usize(PATH_BYTES_MAX) {
        let actual = u64::try_from(trimmed.len()).map_err(|_| PakError::LimitExceeded {
            kind: LimitKind::PathBytes,
            actual: u64::MAX,
            max: u64::from(PATH_BYTES_MAX),
        })?;
        return Err(PakError::LimitExceeded {
            kind: LimitKind::PathBytes,
            actual,
            max: u64::from(PATH_BYTES_MAX),
        });
    }
    let mut hasher = FnvHasher::with_key(FNV1A_64_OFFSET_BASIS.wrapping_add(seed));
    let mut utf16_units: u32 = 0;
    let mut unit_buf = [0_u16; 2];
    for ch in trimmed.chars() {
        let folded = normalize_fold_char(ch);
        let units = folded.encode_utf16(&mut unit_buf);
        let added = u32::try_from(units.len()).map_err(|_| PakError::LimitExceeded {
            kind: LimitKind::PathUtf16Units,
            actual: u64::MAX,
            max: u64::from(PATH_UTF16_UNITS_MAX),
        })?;
        utf16_units = utf16_units.saturating_add(added);
        if utf16_units > PATH_UTF16_UNITS_MAX {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::PathUtf16Units,
                actual: u64::from(utf16_units),
                max: u64::from(PATH_UTF16_UNITS_MAX),
            });
        }
        for &unit in &*units {
            hasher.write(&unit.to_le_bytes());
        }
    }
    Ok(PathHash::from_raw(hasher.finish()))
}

/// Compares two virtual paths under UE4's separator-normalized, case-insensitive equivalence.
#[must_use]
pub(crate) fn eq_ue4_path(a: &str, b: &str) -> bool {
    let a_trim = trim_leading_separators(a);
    let b_trim = trim_leading_separators(b);
    if a_trim.is_ascii() && b_trim.is_ascii() {
        if a_trim.len() != b_trim.len() {
            return false;
        }
        return a_trim
            .as_bytes()
            .iter()
            .zip(b_trim.as_bytes().iter())
            .all(|(&lhs, &rhs)| normalize_ascii_byte(lhs) == normalize_ascii_byte(rhs));
    }
    a_trim
        .chars()
        .map(normalize_fold_char)
        .eq(b_trim.chars().map(normalize_fold_char))
}

#[inline]
const fn normalize_ascii_byte(byte: u8) -> u8 {
    if byte == b'\\' {
        b'/'
    } else {
        byte.to_ascii_lowercase()
    }
}

/// Normalizes a single relative path (`\` -> `/`, leading separators stripped) into `out`.
pub(crate) fn normalize_rel_path_into(raw: &[u8], out: &mut Vec<u8>) -> Result<(), PakError> {
    out.clear();
    let trimmed = trim_leading_sep_bytes(raw);
    push_normalized_bytes(trimmed, out)
}

/// Joins a `FullDirectoryIndex` `(dir_name, file_name)` pair into a canonical relative UTF-8 path.
pub(crate) fn normalize_join_dir_file_into(
    dir: &[u8],
    file: &[u8],
    out: &mut Vec<u8>,
) -> Result<(), PakError> {
    out.clear();
    let dir_trim = trim_leading_sep_bytes(dir);
    let file_trim = trim_leading_sep_bytes(file);
    push_normalized_bytes(dir_trim, out)?;
    if !out.is_empty() && !file_trim.is_empty() && out.last().copied() != Some(b'/') {
        if out.len() >= u32_to_usize(PATH_BYTES_MAX) {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::PathBytes,
                actual: u64::from(PATH_BYTES_MAX) + 1,
                max: u64::from(PATH_BYTES_MAX),
            });
        }
        out.push(b'/');
    }
    push_normalized_bytes(file_trim, out)
}

#[inline]
fn trim_leading_sep_bytes(mut bytes: &[u8]) -> &[u8] {
    while let Some((&first, rest)) = bytes.split_first() {
        if first == b'/' || first == b'\\' {
            bytes = rest;
        } else {
            break;
        }
    }
    bytes
}

fn push_normalized_bytes(src: &[u8], out: &mut Vec<u8>) -> Result<(), PakError> {
    let new_len = out.len().saturating_add(src.len());
    if new_len > u32_to_usize(PATH_BYTES_MAX) {
        let actual = u64::try_from(new_len).map_err(|_| PakError::LimitExceeded {
            kind: LimitKind::PathBytes,
            actual: u64::MAX,
            max: u64::from(PATH_BYTES_MAX),
        })?;
        return Err(PakError::LimitExceeded {
            kind: LimitKind::PathBytes,
            actual,
            max: u64::from(PATH_BYTES_MAX),
        });
    }
    for &b in src {
        out.push(if b == b'\\' { b'/' } else { b });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct PathSpan {
    offset: u32,
    len: u32,
}

/// Contiguous immutable UTF-8 path storage eliminating per-entry `String` heap allocations.
#[derive(Debug)]
pub(crate) struct PathArena {
    text: Box<str>,
    spans: Box<[Option<PathSpan>]>,
}

impl PathArena {
    #[inline]
    #[must_use]
    pub(crate) fn get(&self, id: EntryId) -> Option<&str> {
        let span = self.spans[u32_to_usize(id.index())];
        let PathSpan { offset, len } = span?;
        let start = u32_to_usize(offset);
        let end = start.strict_add(u32_to_usize(len));
        Some(&self.text[start..end])
    }
}

/// Startup builder for [`PathArena`].
pub(crate) struct PathArenaBuilder {
    text: String,
    spans: Vec<Option<PathSpan>>,
}

impl PathArenaBuilder {
    pub(crate) fn with_capacity(entry_cap: u32) -> Self {
        Self {
            text: String::with_capacity(u32_to_usize(entry_cap).saturating_mul(32)),
            spans: Vec::with_capacity(u32_to_usize(entry_cap)),
        }
    }

    pub(crate) fn push_path(&mut self, path: &str) -> Result<(), PakError> {
        let len = u32::try_from(path.len()).map_err(|_| PakError::LimitExceeded {
            kind: LimitKind::PathBytes,
            actual: u64::MAX,
            max: u64::from(PATH_BYTES_MAX),
        })?;
        if len > PATH_BYTES_MAX {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::PathBytes,
                actual: u64::from(len),
                max: u64::from(PATH_BYTES_MAX),
            });
        }
        let offset = u32::try_from(self.text.len()).map_err(|_| PakError::LimitExceeded {
            kind: LimitKind::PathArenaBytes,
            actual: u64::from(u32::MAX) + 1,
            max: u64::from(u32::MAX),
        })?;
        let _end = offset.checked_add(len).ok_or(PakError::LimitExceeded {
            kind: LimitKind::PathArenaBytes,
            actual: u64::from(offset) + u64::from(len),
            max: u64::from(u32::MAX),
        })?;
        self.text.push_str(path);
        self.spans.push(Some(PathSpan { offset, len }));
        Ok(())
    }

    pub(crate) fn push_missing(&mut self) {
        self.spans.push(None);
    }

    pub(crate) fn finish(self, expected_entries: u32) -> PathArena {
        assert_eq!(self.spans.len(), u32_to_usize(expected_entries));
        PathArena {
            text: self.text.into_boxed_str(),
            spans: self.spans.into_boxed_slice(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum HashBucket {
    Empty,
    Occupied { hash: PathHash, entry: EntryId },
}
const _: () = assert!(core::mem::size_of::<HashBucket>() == 16);

/// Power-of-two open-addressed lookup table with <= 50% load factor and bounded linear probing.
#[derive(Debug)]
pub(crate) struct PathLookupTable {
    buckets: Box<[HashBucket]>,
    mask: u32,
    occupied_count: u32,
}

impl PathLookupTable {
    pub(crate) fn with_entry_capacity(entry_count: u32) -> Self {
        assert!(entry_count <= ENTRIES_COUNT_HARD_MAX);
        let min_slots = (entry_count.saturating_mul(2)).max(2);
        let table_len = min_slots.next_power_of_two();
        assert!(table_len >= 2);
        assert!(table_len > entry_count);
        let buckets = vec![HashBucket::Empty; u32_to_usize(table_len)].into_boxed_slice();
        Self {
            buckets,
            mask: table_len - 1,
            occupied_count: 0,
        }
    }

    #[inline]
    fn start_slot(&self, hash: PathHash) -> u32 {
        let raw = hash.as_u64();
        let folded = low_u32(raw ^ (raw >> 32));
        folded & self.mask
    }

    pub(crate) fn insert(&mut self, hash: PathHash, entry: EntryId) {
        let table_len = self.mask.strict_add(1);
        assert!(self.occupied_count < table_len);
        let start = self.start_slot(hash);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let slot = start.wrapping_add(step) & self.mask;
            let slot_idx = u32_to_usize(slot);
            match self.buckets[slot_idx] {
                HashBucket::Empty => {
                    self.buckets[slot_idx] = HashBucket::Occupied { hash, entry };
                    self.occupied_count = self.occupied_count.strict_add(1);
                    assert!(self.occupied_count < table_len);
                    return;
                }
                HashBucket::Occupied {
                    hash: existing_hash,
                    entry: existing_entry,
                } => {
                    if existing_hash == hash && existing_entry == entry {
                        return;
                    }
                    probed = probed.strict_add(1);
                }
            }
        }
        assert!(self.occupied_count < table_len);
        assert!(probed <= self.occupied_count);
    }

    #[must_use]
    pub(crate) fn find(
        &self,
        query_path: &str,
        hash: PathHash,
        arena: &PathArena,
    ) -> Option<EntryId> {
        let table_len = self.mask.strict_add(1);
        let start = self.start_slot(hash);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let slot = start.wrapping_add(step) & self.mask;
            match self.buckets[u32_to_usize(slot)] {
                HashBucket::Empty => return None,
                HashBucket::Occupied {
                    hash: bucket_hash,
                    entry,
                } => {
                    if bucket_hash == hash {
                        match arena.get(entry) {
                            Some(stored) => {
                                if eq_ue4_path(stored, query_path) {
                                    return Some(entry);
                                }
                            }
                            None => return Some(entry),
                        }
                    }
                    probed = probed.strict_add(1);
                }
            }
        }
        assert!(self.occupied_count < table_len);
        assert!(probed <= self.occupied_count);
        None
    }

    #[must_use]
    pub(crate) fn find_by_hash(&self, hash: PathHash) -> Option<EntryId> {
        let table_len = self.mask.strict_add(1);
        let start = self.start_slot(hash);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let slot = start.wrapping_add(step) & self.mask;
            match self.buckets[u32_to_usize(slot)] {
                HashBucket::Empty => return None,
                HashBucket::Occupied {
                    hash: bucket_hash,
                    entry,
                } => {
                    if bucket_hash == hash {
                        return Some(entry);
                    }
                    probed = probed.strict_add(1);
                }
            }
        }
        assert!(self.occupied_count < table_len);
        assert!(probed <= self.occupied_count);
        None
    }
}

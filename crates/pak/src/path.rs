//! Zero-allocation UE4 UTF-16LE FNV-1a path hashing, contiguous `PathArena`, and flat hash lookup.

use core::num::NonZeroU16;

use hashbrown::HashTable;

use crate::error::{LimitKind, PakError};
use crate::types::{
    ENTRIES_COUNT_HARD_MAX, EntryId, PATH_BYTES_MAX, PATH_UTF16_UNITS_MAX, PathHash,
};
use crate::{bounded_len_u32, u32_to_usize};

const FNV1A_64_OFFSET_BASIS: u64 = 0xCBF2_9CE4_8422_2325;
const FNV1A_64_PRIME: u64 = 0x0000_0100_0000_01B3;

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
        (None, _) | (Some(_), Some(_)) => ch,
    }
}

/// Computes Unreal Engine's seeded 64-bit FNV-1a digest over normalized, lowercased UTF-16LE bytes.
///
/// Streams 2 bytes per UTF-16LE code unit directly into the 64-bit FNV-1a accumulator with zero
/// heap allocations and zero intermediate stack buffer copies.
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
    let mut hash_state = FNV1A_64_OFFSET_BASIS.wrapping_add(seed);
    let mut utf16_units: u32 = 0;
    let mut unit_buf = [0_u16; 2];
    for ch in trimmed.chars() {
        let folded = normalize_fold_char(ch);
        let units = folded.encode_utf16(&mut unit_buf);
        let added = bounded_len_u32(units.len());
        utf16_units += added;
        if utf16_units > PATH_UTF16_UNITS_MAX {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::PathUtf16Units,
                actual: u64::from(utf16_units),
                max: u64::from(PATH_UTF16_UNITS_MAX),
            });
        }
        for &unit in &*units {
            for byte in unit.to_le_bytes() {
                hash_state = (hash_state ^ u64::from(byte)).wrapping_mul(FNV1A_64_PRIME);
            }
        }
    }
    Ok(PathHash::from_raw(hash_state))
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
pub(crate) fn normalize_rel_path_into(raw: &str, out: &mut String) -> Result<(), PakError> {
    out.clear();
    let trimmed = raw.trim_start_matches(['/', '\\']);
    push_normalized_str(trimmed, out)
}

/// Joins a `FullDirectoryIndex` `(dir_name, file_name)` pair into a canonical relative UTF-8 path.
pub(crate) fn normalize_join_dir_file_into(
    dir: &str,
    file: &str,
    out: &mut String,
) -> Result<(), PakError> {
    out.clear();
    let dir_trim = dir.trim_start_matches(['/', '\\']);
    let file_trim = file.trim_start_matches(['/', '\\']);
    push_normalized_str(dir_trim, out)?;
    if !out.is_empty() && !file_trim.is_empty() && !out.ends_with('/') {
        if out.len() >= u32_to_usize(PATH_BYTES_MAX) {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::PathBytes,
                actual: u64::from(PATH_BYTES_MAX) + 1,
                max: u64::from(PATH_BYTES_MAX),
            });
        }
        out.push('/');
    }
    push_normalized_str(file_trim, out)
}

fn push_normalized_str(src: &str, out: &mut String) -> Result<(), PakError> {
    let new_len = out
        .len()
        .checked_add(src.len())
        .ok_or(PakError::LimitExceeded {
            kind: LimitKind::PathBytes,
            actual: u64::MAX,
            max: u64::from(PATH_BYTES_MAX),
        })?;
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
    for (idx, part) in src.split('\\').enumerate() {
        if idx > 0 {
            out.push('/');
        }
        out.push_str(part);
    }
    Ok(())
}

const _: () = assert!(PATH_BYTES_MAX < 0xFFFF);

/// Niche-optimized 8-byte `(offset, len)` span into [`PathArena`] where `len + 1` is stored in
/// [`NonZeroU16`] so `Option<PathSpan>` also occupies 8 bytes without any dummy padding field.
#[derive(Clone, Copy, Debug)]
struct PathSpan {
    offset: u32,
    len_plus_one: NonZeroU16,
}
const _: () = assert!(core::mem::size_of::<PathSpan>() == 8);
const _: () = assert!(core::mem::size_of::<Option<PathSpan>>() == 8);

impl PathSpan {
    #[inline]
    #[must_use]
    const fn len(self) -> u16 {
        self.len_plus_one.get() - 1
    }
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
        let span = self.spans[u32_to_usize(id.index())]?;
        let start = u32_to_usize(span.offset);
        let end = start + usize::from(span.len());
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
        assert!(entry_cap <= u32::MAX / 32);
        Self {
            text: String::with_capacity(u32_to_usize(entry_cap * 32)),
            spans: Vec::with_capacity(u32_to_usize(entry_cap)),
        }
    }

    pub(crate) fn push_path(&mut self, path: &str) -> Result<(), PakError> {
        let Ok(len_u16) = u16::try_from(path.len()) else {
            let actual = u64::try_from(path.len()).map_err(|_| PakError::LimitExceeded {
                kind: LimitKind::PathBytes,
                actual: u64::MAX,
                max: u64::from(PATH_BYTES_MAX),
            })?;
            return Err(PakError::LimitExceeded {
                kind: LimitKind::PathBytes,
                actual,
                max: u64::from(PATH_BYTES_MAX),
            });
        };
        let len = u32::from(len_u16);
        if len > PATH_BYTES_MAX {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::PathBytes,
                actual: u64::from(len),
                max: u64::from(PATH_BYTES_MAX),
            });
        }
        let Some(len_plus_one) = NonZeroU16::new(len_u16 + 1) else {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::PathBytes,
                actual: u64::from(len),
                max: u64::from(PATH_BYTES_MAX),
            });
        };
        let offset = bounded_len_u32(self.text.len());
        let _end = offset.checked_add(len).ok_or(PakError::LimitExceeded {
            kind: LimitKind::PathArenaBytes,
            actual: u64::from(offset) + u64::from(len),
            max: u64::from(u32::MAX),
        })?;
        self.text.push_str(path);
        self.spans.push(Some(PathSpan {
            offset,
            len_plus_one,
        }));
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
struct PathHashSlot {
    hash: PathHash,
    entry: EntryId,
}
const _: () = assert!(core::mem::size_of::<PathHashSlot>() == 16);

/// Pre-sized [`HashTable<u32>`] index over `(PathHash, EntryId)` slots with external [`PathArena`] disambiguation.
#[derive(Debug)]
pub(crate) struct PathLookupTable {
    table: HashTable<u32>,
    slots: Vec<PathHashSlot>,
    capacity: u32,
}

impl PathLookupTable {
    pub(crate) fn with_entry_capacity(entry_count: u32) -> Self {
        assert!(entry_count <= ENTRIES_COUNT_HARD_MAX);
        let cap = u32_to_usize(entry_count);
        Self {
            table: HashTable::with_capacity(cap),
            slots: Vec::with_capacity(cap),
            capacity: entry_count,
        }
    }

    pub(crate) fn insert(&mut self, hash: PathHash, entry: EntryId) {
        let raw_hash = hash.as_u64();
        if self
            .table
            .find(raw_hash, |&idx| {
                let slot = self.slots[u32_to_usize(idx)];
                slot.hash == hash && slot.entry == entry
            })
            .is_some()
        {
            return;
        }
        let idx = bounded_len_u32(self.slots.len());
        assert!(idx < self.capacity);
        self.slots.push(PathHashSlot { hash, entry });
        let slots = &self.slots;
        self.table
            .insert_unique(raw_hash, idx, |&i| slots[u32_to_usize(i)].hash.as_u64());
    }

    #[must_use]
    pub(crate) fn find(
        &self,
        query_path: &str,
        hash: PathHash,
        arena: &PathArena,
    ) -> Option<EntryId> {
        let idx = *self.table.find(hash.as_u64(), |&i| {
            let slot = self.slots[u32_to_usize(i)];
            if slot.hash != hash {
                return false;
            }
            match arena.get(slot.entry) {
                Some(stored) => eq_ue4_path(stored, query_path),
                None => true,
            }
        })?;
        Some(self.slots[u32_to_usize(idx)].entry)
    }

    #[must_use]
    pub(crate) fn find_by_hash(&self, hash: PathHash) -> Option<EntryId> {
        let idx = *self
            .table
            .find(hash.as_u64(), |&i| self.slots[u32_to_usize(i)].hash == hash)?;
        Some(self.slots[u32_to_usize(idx)].entry)
    }
}

//! Contiguous deduplicated UTF-8 `StringArena` and interned file extension table (`ExtTable`).

use crate::error::{VfsError, VfsLimitKind};
use crate::path::eq_folded_str;
use crate::types::{ExtSlotId, VfsLimits};
use crate::{bounded_len_u16, bounded_len_u32, u32_to_usize};

/// Compact `(offset, len)` span addressing a valid UTF-8 slice inside [`StringArena`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ArenaSpan {
    pub(crate) offset: u32,
    pub(crate) len: u16,
}
const _: () = assert!(core::mem::size_of::<ArenaSpan>() == 8);

impl ArenaSpan {
    pub(crate) const EMPTY: Self = Self { offset: 0, len: 0 };

    #[inline]
    #[must_use]
    pub(crate) const fn new(offset: u32, len: u16) -> Self {
        Self { offset, len }
    }
}

/// Immutable contiguous UTF-8 buffer storing all directory segment names, per-directory deduplicated
/// file stems, and interned extension strings.
#[derive(Debug)]
pub(crate) struct StringArena {
    text: Box<str>,
}

impl StringArena {
    #[inline]
    #[must_use]
    pub(crate) fn get(&self, span: ArenaSpan) -> &str {
        let start = u32_to_usize(span.offset);
        let end = start + usize::from(span.len);
        &self.text[start..end]
    }

    #[inline]
    #[must_use]
    pub(crate) const fn byte_len(&self) -> u32 {
        bounded_len_u32(self.text.len())
    }
}

pub(crate) struct StringArenaBuilder {
    buf: String,
    bytes_max: u32,
}

impl StringArenaBuilder {
    #[must_use]
    pub(crate) fn with_capacity(initial_cap: u32, bytes_max: u32) -> Self {
        let clamped = initial_cap.min(bytes_max);
        Self {
            buf: String::with_capacity(u32_to_usize(clamped)),
            bytes_max,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) const fn byte_len(&self) -> u32 {
        bounded_len_u32(self.buf.len())
    }

    #[inline]
    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        &self.buf
    }

    #[inline]
    #[must_use]
    pub(crate) fn get(&self, span: ArenaSpan) -> &str {
        let start = u32_to_usize(span.offset);
        let end = start + usize::from(span.len);
        &self.buf[start..end]
    }

    pub(crate) fn push_str(&mut self, s: &str) -> Result<ArenaSpan, VfsError> {
        let Ok(seg_len) = u16::try_from(s.len()) else {
            let actual = u64::try_from(s.len()).map_err(|_| VfsError::LimitExceeded {
                kind: VfsLimitKind::PathBytes,
                actual: u64::MAX,
                max: u64::from(u16::MAX),
            })?;
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::PathBytes,
                actual,
                max: u64::from(u16::MAX),
            });
        };
        let cur_u32 = self.byte_len();
        let next_u64 = u64::from(cur_u32) + u64::from(seg_len);
        if next_u64 > u64::from(self.bytes_max) {
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::ArenaBytes,
                actual: next_u64,
                max: u64::from(self.bytes_max),
            });
        }
        self.buf.push_str(s);
        Ok(ArenaSpan::new(cur_u32, seg_len))
    }

    #[must_use]
    pub(crate) fn finish(self) -> StringArena {
        assert!(self.buf.len() <= u32_to_usize(self.bytes_max));
        StringArena {
            text: self.buf.into_boxed_str(),
        }
    }
}

/// Canonical Unreal Engine file extension classification ordered so package companions sort adjacently.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum KnownExt {
    None = 0,
    Uasset = 1,
    Umap = 2,
    Uexp = 3,
    Ubulk = 4,
    Uptnl = 5,
    Uplugin = 6,
    Other = 7,
}

impl KnownExt {
    #[must_use]
    pub(crate) fn classify(ext: Option<&str>) -> Self {
        let Some(s) = ext else {
            return Self::None;
        };
        if s.eq_ignore_ascii_case("uasset") {
            Self::Uasset
        } else if s.eq_ignore_ascii_case("umap") {
            Self::Umap
        } else if s.eq_ignore_ascii_case("uexp") {
            Self::Uexp
        } else if s.eq_ignore_ascii_case("ubulk") {
            Self::Ubulk
        } else if s.eq_ignore_ascii_case("uptnl") {
            Self::Uptnl
        } else if s.eq_ignore_ascii_case("uplugin") {
            Self::Uplugin
        } else {
            Self::Other
        }
    }

    #[inline]
    #[must_use]
    pub(crate) const fn is_package_member(self) -> bool {
        match self {
            Self::Uasset | Self::Umap | Self::Uexp | Self::Ubulk | Self::Uptnl => true,
            Self::None | Self::Uplugin | Self::Other => false,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct ExtEntry {
    offset: u32,
    len: u16,
    canonical_slot: ExtSlotId,
}
const _: () = assert!(core::mem::size_of::<ExtEntry>() == 8);

impl ExtEntry {
    #[inline]
    #[must_use]
    const fn span(self) -> ArenaSpan {
        ArenaSpan::new(self.offset, self.len)
    }
}

/// Immutable table of exact case-preserved file extension spellings (`35` slots in Torchlight Infinite).
#[derive(Debug)]
pub(crate) struct ExtTable {
    entries: Box<[ExtEntry]>,
}

impl ExtTable {
    #[inline]
    #[must_use]
    pub(crate) fn get<'a>(
        &self,
        slot: Option<ExtSlotId>,
        arena: &'a StringArena,
    ) -> Option<&'a str> {
        let id = slot?;
        let entry = self.entries[usize::from(id.index())];
        Some(arena.get(entry.span()))
    }

    #[inline]
    #[must_use]
    pub(crate) fn canonical_for(&self, slot: Option<ExtSlotId>) -> Option<ExtSlotId> {
        let id = slot?;
        Some(self.entries[usize::from(id.index())].canonical_slot)
    }

    /// Resolves `query_ext` (stripping optional leading `.`) to its case-insensitive equivalence class
    /// representative [`ExtSlotId`], enabling `O(1)` 2-byte per-file comparison in [`crate::ExtFileIter`].
    #[must_use]
    pub(crate) fn find_canonical_slot(
        &self,
        query_ext: &str,
        arena: &StringArena,
    ) -> Option<ExtSlotId> {
        let clean = match query_ext.strip_prefix('.') {
            Some(stripped) => stripped,
            None => query_ext,
        };
        if clean.is_empty() {
            return None;
        }
        for entry in &self.entries {
            let stored = arena.get(entry.span());
            if eq_folded_str(stored, clean) {
                return Some(entry.canonical_slot);
            }
        }
        None
    }

    #[inline]
    #[must_use]
    pub(crate) const fn count(&self) -> u16 {
        bounded_len_u16(self.entries.len())
    }
}

#[derive(Clone, Copy)]
struct StagingExt {
    span: ArenaSpan,
    slot: ExtSlotId,
    canonical_slot: ExtSlotId,
    known: KnownExt,
}

pub(crate) struct ExtBuilder {
    text: String,
    entries: Vec<StagingExt>,
}

impl ExtBuilder {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            text: String::with_capacity(512),
            entries: Vec::with_capacity(64),
        }
    }

    #[inline]
    #[must_use]
    const fn count(&self) -> u16 {
        bounded_len_u16(self.entries.len())
    }

    #[inline]
    #[must_use]
    fn slice(&self, span: ArenaSpan) -> &str {
        let start = u32_to_usize(span.offset);
        let end = start + usize::from(span.len);
        &self.text[start..end]
    }

    pub(crate) fn intern(
        &mut self,
        ext: Option<&str>,
        limits: VfsLimits,
    ) -> Result<Option<ExtSlotId>, VfsError> {
        let Some(ext_str) = ext else {
            return Ok(None);
        };
        for entry in &self.entries {
            if self.slice(entry.span) == ext_str {
                return Ok(Some(entry.slot));
            }
        }
        let folded_match = self.find_folded_canonical(ext_str);
        let cur_count = self.count();
        if cur_count >= limits.exts_count_max() {
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::ExtsCount,
                actual: u64::from(limits.exts_count_max()) + 1,
                max: u64::from(limits.exts_count_max()),
            });
        }
        let slot = ExtSlotId::try_from_index(cur_count).ok_or(VfsError::LimitExceeded {
            kind: VfsLimitKind::ExtsCount,
            actual: u64::from(cur_count) + 1,
            max: u64::from(limits.exts_count_max()),
        })?;
        let Ok(ext_len) = u16::try_from(ext_str.len()) else {
            let actual = u64::try_from(ext_str.len()).map_err(|_| VfsError::LimitExceeded {
                kind: VfsLimitKind::PathBytes,
                actual: u64::MAX,
                max: u64::from(u16::MAX),
            })?;
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::PathBytes,
                actual,
                max: u64::from(u16::MAX),
            });
        };
        let off_u32 = bounded_len_u32(self.text.len());
        let next_bytes = u64::from(off_u32) + u64::from(ext_len);
        if next_bytes > u64::from(limits.arena_bytes_max()) {
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::ArenaBytes,
                actual: next_bytes,
                max: u64::from(limits.arena_bytes_max()),
            });
        }
        self.text.push_str(ext_str);
        let canon = match folded_match {
            Some(existing_canon) => existing_canon,
            None => slot,
        };
        self.entries.push(StagingExt {
            span: ArenaSpan::new(off_u32, ext_len),
            slot,
            canonical_slot: canon,
            known: KnownExt::classify(Some(ext_str)),
        });
        Ok(Some(slot))
    }

    fn find_folded_canonical(&self, ext_str: &str) -> Option<ExtSlotId> {
        for entry in &self.entries {
            if eq_folded_str(self.slice(entry.span), ext_str) {
                return Some(entry.canonical_slot);
            }
        }
        None
    }

    #[inline]
    #[must_use]
    pub(crate) fn get_str(&self, slot: Option<ExtSlotId>) -> Option<&str> {
        let id = slot?;
        Some(self.slice(self.entries[usize::from(id.index())].span))
    }

    #[inline]
    #[must_use]
    pub(crate) fn known_for(&self, slot: Option<ExtSlotId>) -> KnownExt {
        match slot {
            Some(id) => self.entries[usize::from(id.index())].known,
            None => KnownExt::None,
        }
    }

    pub(crate) fn finish(self, arena: &mut StringArenaBuilder) -> Result<ExtTable, VfsError> {
        let mut entries = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            let span = arena.push_str(self.slice(entry.span))?;
            entries.push(ExtEntry {
                offset: span.offset,
                len: span.len,
                canonical_slot: entry.canonical_slot,
            });
        }
        Ok(ExtTable {
            entries: entries.into_boxed_slice(),
        })
    }
}

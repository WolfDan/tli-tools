//! Non-generic Zstandard block decompression and iterative key-hierarchy walkers.

use crate::bin::error::{BinError, BinLimitKind, WireContext};
use crate::bin::types::{
    AffixEffectRow, AffixEffectTable, BinLimits, BinTable, KEY_DEPTH_COUNT_MAX, KeySchema,
    MultiLanguageRow, MultiLanguageTable,
};
use crate::bin::wire::{
    RawAffixEffectFixed, RawBlockDesc, RawChunkHeader, RawMapHeader, WireCursor,
};
use crate::{FixedVec, bounded_len_u32, u32_to_usize};
use arrayvec::ArrayVec;
use core::hash::Hash;
use rustc_hash::{FxBuildHasher, FxHashMap};

const KEY_DEPTH_MAX_USIZE: usize = u32_to_usize(KEY_DEPTH_COUNT_MAX);
const INITIAL_MAP_CAP_MAX: u32 = 65_536;

#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "Block index is pre-validated < CHUNK_BLOCKS_COUNT_MAX (4096) <= u16::MAX"
)]
const fn block_idx_to_u16(idx: u32) -> u16 {
    assert!(idx <= u16::MAX as u32);
    idx as u16
}

/// Reusable Zstandard decompression context and bounded block buffer for `*_ChunkA0.bin` files.
pub struct BinScratch {
    limits: BinLimits,
    dctx: zstd_safe::DCtx<'static>,
    block_buf: FixedVec<u8>,
}

impl core::fmt::Debug for BinScratch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BinScratch")
            .field("limits", &self.limits)
            .field("block_buf_cap", &self.block_buf.cap())
            .finish_non_exhaustive()
    }
}

impl BinScratch {
    /// Allocates a reusable `BinScratch` bounded by `limits`.
    #[must_use]
    pub fn with_limits(limits: BinLimits) -> Self {
        Self {
            limits,
            dctx: zstd_safe::DCtx::default(),
            block_buf: FixedVec::with_capacity(limits.block_uncompressed_bytes_max()),
        }
    }

    /// Returns the active [`BinLimits`] governing this scratch buffer.
    #[inline]
    #[must_use]
    pub const fn limits(&self) -> BinLimits {
        self.limits
    }

    #[inline(never)]
    fn decompress_block(
        &mut self,
        block_index: u16,
        frame: &[u8],
        expected_uncompressed: u32,
    ) -> Result<&[u8], BinError> {
        let resized = self.block_buf.resize_within_cap(expected_uncompressed, 0);
        assert!(resized);
        let written = self
            .dctx
            .decompress(self.block_buf.as_mut_slice(), frame)
            .map_err(|code| BinError::ZstdDecompressFailed {
                block_index,
                code_raw: code as u64,
            })?;
        if written != u32_to_usize(expected_uncompressed) {
            return Err(BinError::DecompressedSizeMismatch {
                block_index,
                expected_bytes: expected_uncompressed,
                actual_bytes: written as u64,
            });
        }
        Ok(self.block_buf.as_slice())
    }
}

/// Primary key type that can be materialized from the iterative hierarchy walker state.
pub trait BinKey: Eq + Hash + Sized {
    /// Key hierarchy schema (`Int1..Int4`, `Str1`, `StrInt1..StrInt3`).
    const SCHEMA: KeySchema;

    /// Constructs the key from the current hierarchy path.
    fn from_hierarchy(str_key: &str, int_keys: &[i32; 4]) -> Self;
}

impl BinKey for i32 {
    const SCHEMA: KeySchema = KeySchema::Int1;

    #[inline]
    fn from_hierarchy(_str_key: &str, int_keys: &[i32; 4]) -> Self {
        int_keys[0]
    }
}

impl BinKey for (i32, i32) {
    const SCHEMA: KeySchema = KeySchema::Int2;

    #[inline]
    fn from_hierarchy(_str_key: &str, int_keys: &[i32; 4]) -> Self {
        (int_keys[0], int_keys[1])
    }
}

impl BinKey for (i32, i32, i32) {
    const SCHEMA: KeySchema = KeySchema::Int3;

    #[inline]
    fn from_hierarchy(_str_key: &str, int_keys: &[i32; 4]) -> Self {
        (int_keys[0], int_keys[1], int_keys[2])
    }
}

impl BinKey for (i32, i32, i32, i32) {
    const SCHEMA: KeySchema = KeySchema::Int4;

    #[inline]
    fn from_hierarchy(_str_key: &str, int_keys: &[i32; 4]) -> Self {
        (int_keys[0], int_keys[1], int_keys[2], int_keys[3])
    }
}

impl BinKey for Box<str> {
    const SCHEMA: KeySchema = KeySchema::Str1;

    #[inline]
    fn from_hierarchy(str_key: &str, _int_keys: &[i32; 4]) -> Self {
        str_key.into()
    }
}

impl BinKey for (Box<str>, i32) {
    const SCHEMA: KeySchema = KeySchema::StrInt1;

    #[inline]
    fn from_hierarchy(str_key: &str, int_keys: &[i32; 4]) -> Self {
        (str_key.into(), int_keys[1])
    }
}

impl BinKey for (Box<str>, i32, i32) {
    const SCHEMA: KeySchema = KeySchema::StrInt2;

    #[inline]
    fn from_hierarchy(str_key: &str, int_keys: &[i32; 4]) -> Self {
        (str_key.into(), int_keys[1], int_keys[2])
    }
}

impl BinKey for (Box<str>, i32, i32, i32) {
    const SCHEMA: KeySchema = KeySchema::StrInt3;

    #[inline]
    fn from_hierarchy(str_key: &str, int_keys: &[i32; 4]) -> Self {
        (str_key.into(), int_keys[1], int_keys[2], int_keys[3])
    }
}

/// Trait implemented by generated `Config*Row` structs to decode leaf columns without drop ladders.
pub trait BinRow: Default {
    /// Populates `self` in place from `cur` at a leaf row position.
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError>;
}

#[inline(never)]
fn validate_block_descs(
    descs: &[RawBlockDesc],
    payload_len: usize,
    limits: BinLimits,
) -> Result<(), BinError> {
    let payload_bytes = payload_len as u64;
    let block_count = bounded_len_u32(descs.len());
    for (idx, desc) in descs.iter().enumerate() {
        let block_idx_u32 = bounded_len_u32(idx);
        let block_index = block_idx_to_u16(block_idx_u32);
        let start = desc.offset_compressed.get();
        let end = if block_idx_u32 + 1 < block_count {
            descs[idx + 1].offset_compressed.get()
        } else {
            bounded_len_u32(payload_len)
        };
        if start > end || u64::from(end) > payload_bytes {
            return Err(BinError::InvalidBlockRange {
                block_index,
                start_offset: start,
                end_offset: end,
                payload_bytes,
            });
        }
        let uncomp = desc.size_uncompressed.get();
        if uncomp > limits.block_uncompressed_bytes_max() {
            return Err(BinError::LimitExceeded {
                kind: BinLimitKind::BlockUncompressedBytes,
                actual: u64::from(uncomp),
                max: u64::from(limits.block_uncompressed_bytes_max()),
            });
        }
        if uncomp == 0 && start != end {
            return Err(BinError::NonEmptyFrameForEmptyBlock {
                block_index,
                compressed_bytes: end - start,
            });
        }
    }
    Ok(())
}

#[inline(never)]
fn parse_chunk_header(
    buf: &[u8],
    limits: BinLimits,
) -> Result<(&[RawBlockDesc], &[u8], u32), BinError> {
    let mut cur = WireCursor::new(buf, limits);
    let hdr = cur.take::<RawChunkHeader>(WireContext::ChunkHeader)?;
    let block_count = hdr.block_count.get();
    if block_count > limits.block_count_max() {
        return Err(BinError::LimitExceeded {
            kind: BinLimitKind::ChunkBlocksCount,
            actual: u64::from(block_count),
            max: u64::from(limits.block_count_max()),
        });
    }
    let descs =
        cur.take_slice_elems::<RawBlockDesc>(block_count, WireContext::BlockDescriptorTable)?;
    let payload = cur.remaining();
    validate_block_descs(descs, payload.len(), limits)?;
    Ok((descs, payload, hdr.hash_buckets.get()))
}

#[inline(never)]
fn open_decompressed_block<'s>(
    scratch: &'s mut BinScratch,
    descs: &[RawBlockDesc],
    payload: &[u8],
    block_idx: u32,
) -> Result<Option<(WireCursor<'s>, ArrayVec<u32, KEY_DEPTH_MAX_USIZE>)>, BinError> {
    let idx = u32_to_usize(block_idx);
    let desc = &descs[idx];
    let uncomp = desc.size_uncompressed.get();
    if uncomp == 0 {
        return Ok(None);
    }
    let start = u32_to_usize(desc.offset_compressed.get());
    let end = if idx + 1 < descs.len() {
        u32_to_usize(descs[idx + 1].offset_compressed.get())
    } else {
        payload.len()
    };
    let limits = scratch.limits();
    let block_bytes =
        scratch.decompress_block(block_idx_to_u16(block_idx), &payload[start..end], uncomp)?;
    let mut cur = WireCursor::new(block_bytes, limits);
    let top_count = cur.read_u32(WireContext::DecompressedBlock)?;
    if top_count > limits.rows_count_max() {
        return Err(BinError::LimitExceeded {
            kind: BinLimitKind::RowsCount,
            actual: u64::from(top_count),
            max: u64::from(limits.rows_count_max()),
        });
    }
    let mut frames = ArrayVec::new();
    let push_res = frames.try_push(top_count);
    assert!(push_res.is_ok());
    Ok(Some((cur, frames)))
}

#[inline(never)]
fn step_hierarchy<'a>(
    cur: &mut WireCursor<'a>,
    schema: KeySchema,
    frames: &mut ArrayVec<u32, KEY_DEPTH_MAX_USIZE>,
    str_key: &mut &'a str,
    int_keys: &mut [i32; 4],
) -> Result<bool, BinError> {
    let depth = usize::from(schema.depth());
    while !frames.is_empty() {
        let level = frames.len() - 1;
        if frames[level] == 0 {
            frames.pop();
            continue;
        }
        frames[level] -= 1;
        if level == 0 && schema.outer_is_str() {
            let bytes = cur.read_bytes_slice(WireContext::KeyHierarchy)?;
            *str_key = core::str::from_utf8(bytes)?;
        } else {
            int_keys[level] = cur.read_i32_ctx(WireContext::KeyHierarchy)?;
        }
        if level + 1 < depth {
            let _bucket_meta = cur.read_u32(WireContext::KeyHierarchy)?;
            let child_count = cur.read_u32(WireContext::KeyHierarchy)?;
            if child_count > cur.limits().rows_count_max() {
                return Err(BinError::LimitExceeded {
                    kind: BinLimitKind::RowsCount,
                    actual: u64::from(child_count),
                    max: u64::from(cur.limits().rows_count_max()),
                });
            }
            let push_res = frames.try_push(child_count);
            assert!(push_res.is_ok());
        } else {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Parses a Zstandard-compressed `*_ChunkA0.bin` buffer into a [`BinTable<K, Row>`].
pub fn parse_chunk_bin_table<K: BinKey, Row: BinRow>(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<BinTable<K, Row>, BinError> {
    let limits = scratch.limits();
    let (descs, payload, hash_buckets) = parse_chunk_header(buf, limits)?;
    let init_cap = hash_buckets
        .min(limits.rows_count_max())
        .min(INITIAL_MAP_CAP_MAX);
    let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(init_cap), FxBuildHasher);
    let mut total_rows: u32 = 0;
    let block_count = bounded_len_u32(descs.len());

    for block_idx in 0..block_count {
        let Some((mut cur, mut frames)) =
            open_decompressed_block(scratch, descs, payload, block_idx)?
        else {
            continue;
        };
        let mut str_key = "";
        let mut int_keys = [0i32; 4];
        while step_hierarchy(
            &mut cur,
            K::SCHEMA,
            &mut frames,
            &mut str_key,
            &mut int_keys,
        )? {
            if total_rows == limits.rows_count_max() {
                return Err(BinError::LimitExceeded {
                    kind: BinLimitKind::RowsCount,
                    actual: u64::from(total_rows) + 1,
                    max: u64::from(limits.rows_count_max()),
                });
            }
            total_rows += 1;
            let key = K::from_hierarchy(str_key, &int_keys);
            let mut row = Row::default();
            row.read_fields(&mut cur)?;
            if map.insert(key, row).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::KeyHierarchy,
                });
            }
        }
        if !cur.is_empty() {
            return Err(BinError::TrailingBytes {
                context: WireContext::DecompressedBlock,
                remaining_bytes: cur.remaining().len() as u64,
            });
        }
    }
    Ok(BinTable::from_map(map))
}

/// Parses an uncompressed `Maps/<Map>/config_*.bin` buffer into a [`BinTable<K, Row>`].
pub fn parse_map_bin_table<K: BinKey, Row: BinRow>(
    buf: &[u8],
    limits: BinLimits,
) -> Result<BinTable<K, Row>, BinError> {
    let mut cur = WireCursor::new(buf, limits);
    let hdr = cur.take::<RawMapHeader>(WireContext::MapContainerHeader)?;
    let top_count = hdr.top_count.get();
    if top_count > limits.rows_count_max() {
        return Err(BinError::LimitExceeded {
            kind: BinLimitKind::RowsCount,
            actual: u64::from(top_count),
            max: u64::from(limits.rows_count_max()),
        });
    }
    let init_cap = top_count.min(INITIAL_MAP_CAP_MAX);
    let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(init_cap), FxBuildHasher);
    let mut frames = ArrayVec::new();
    let push_res = frames.try_push(top_count);
    assert!(push_res.is_ok());
    let mut str_key = "";
    let mut int_keys = [0i32; 4];
    let mut total_rows: u32 = 0;

    while step_hierarchy(
        &mut cur,
        K::SCHEMA,
        &mut frames,
        &mut str_key,
        &mut int_keys,
    )? {
        if total_rows == limits.rows_count_max() {
            return Err(BinError::LimitExceeded {
                kind: BinLimitKind::RowsCount,
                actual: u64::from(total_rows) + 1,
                max: u64::from(limits.rows_count_max()),
            });
        }
        total_rows += 1;
        let key = K::from_hierarchy(str_key, &int_keys);
        let mut row = Row::default();
        row.read_fields(&mut cur)?;
        if map.insert(key, row).is_some() {
            return Err(BinError::DuplicateKey {
                context: WireContext::KeyHierarchy,
            });
        }
    }
    if !cur.is_empty() {
        return Err(BinError::TrailingBytes {
            context: WireContext::MapContainerEof,
            remaining_bytes: cur.remaining().len() as u64,
        });
    }
    Ok(BinTable::from_map(map))
}

/// Parses an uncompressed `config_multi_language*.bin` localization table.
pub fn parse_multi_language_table(
    buf: &[u8],
    limits: BinLimits,
) -> Result<MultiLanguageTable, BinError> {
    let mut cur = WireCursor::new(buf, limits);
    let hdr = cur.take::<RawMapHeader>(WireContext::MultiLanguageRecord)?;
    let count = hdr.top_count.get();
    if count > limits.rows_count_max() {
        return Err(BinError::LimitExceeded {
            kind: BinLimitKind::RowsCount,
            actual: u64::from(count),
            max: u64::from(limits.rows_count_max()),
        });
    }
    let mut map = FxHashMap::with_capacity_and_hasher(
        u32_to_usize(count.min(INITIAL_MAP_CAP_MAX)),
        FxBuildHasher,
    );
    for _ in 0..count {
        let key1 = cur.read_str_ctx(WireContext::MultiLanguageRecord)?;
        let key2 = cur.read_str_ctx(WireContext::MultiLanguageRecord)?;
        let author = cur.read_str_ctx(WireContext::MultiLanguageRecord)?;
        let text = cur.read_str_ctx(WireContext::MultiLanguageRecord)?;
        if map
            .insert((key1, key2), MultiLanguageRow { author, text })
            .is_some()
        {
            return Err(BinError::DuplicateKey {
                context: WireContext::MultiLanguageRecord,
            });
        }
    }
    if !cur.is_empty() {
        return Err(BinError::TrailingBytes {
            context: WireContext::MultiLanguageRecord,
            remaining_bytes: cur.remaining().len() as u64,
        });
    }
    Ok(BinTable::from_map(map))
}

/// Parses `config_affix_effect.bin` into an [`AffixEffectTable`].
pub fn parse_affix_effect_table(
    buf: &[u8],
    limits: BinLimits,
) -> Result<AffixEffectTable, BinError> {
    let mut cur = WireCursor::new(buf, limits);
    let hdr = cur.take::<RawMapHeader>(WireContext::AffixEffectRecord)?;
    let count = hdr.top_count.get();
    if count > limits.rows_count_max() {
        return Err(BinError::LimitExceeded {
            kind: BinLimitKind::RowsCount,
            actual: u64::from(count),
            max: u64::from(limits.rows_count_max()),
        });
    }
    let mut map = FxHashMap::with_capacity_and_hasher(
        u32_to_usize(count.min(INITIAL_MAP_CAP_MAX)),
        FxBuildHasher,
    );
    for _ in 0..count {
        let fixed = cur.take::<RawAffixEffectFixed>(WireContext::AffixEffectRecord)?;
        let trigger_json = cur.read_str_ctx(WireContext::AffixEffectRecord)?;
        let condition1_json = cur.read_str_ctx(WireContext::AffixEffectRecord)?;
        let condition2_json = cur.read_str_ctx(WireContext::AffixEffectRecord)?;
        let pre_effect_json = cur.read_str_ctx(WireContext::AffixEffectRecord)?;
        let effect_json = cur.read_str_ctx(WireContext::AffixEffectRecord)?;
        let param1 = cur.read_i32_ctx(WireContext::AffixEffectRecord)?;
        let id = fixed.id.get();
        let sub_id = fixed.sub_id.get();
        let row = AffixEffectRow {
            trigger_json,
            condition1_json,
            condition2_json,
            pre_effect_json,
            effect_json,
            id,
            sub_id,
            sub_id_inner: fixed.sub_id_inner.get(),
            id_inner: fixed.id_inner.get(),
            param0: fixed.param0.get(),
            param1,
        };
        if map.insert((id, sub_id), row).is_some() {
            return Err(BinError::DuplicateKey {
                context: WireContext::AffixEffectRecord,
            });
        }
    }
    if !cur.is_empty() {
        return Err(BinError::TrailingBytes {
            context: WireContext::AffixEffectRecord,
            remaining_bytes: cur.remaining().len() as u64,
        });
    }
    Ok(BinTable::from_map(map))
}

const CHUNK_LOCALE_SUFFIXES: [&str; 19] = [
    "_cn", "_de", "_en", "_es", "_fr", "_hk", "_id", "_ja", "_jp", "_ko", "_kr", "_pt", "_ru",
    "_th", "_tr", "_tw", "_vi", "_vn", "_zh",
];

/// Normalizes a `*_ChunkA0.bin` file stem to its canonical `config_*` table stem by stripping
/// `_ChunkA0` and any trailing two-letter locale suffix (`_CN`, `_EN`, `_ja`, etc.).
#[must_use]
pub fn normalize_chunk_bin_stem(raw_stem: &str) -> &str {
    let s = match raw_stem.strip_suffix("_ChunkA0") {
        Some(stripped) => stripped,
        None => raw_stem,
    };
    if s.len() > 3
        && let Some((base, tail)) = s.rsplit_once('_')
        && tail.len() == 2
    {
        let suffix_with_underscore = &s[base.len()..];
        if CHUNK_LOCALE_SUFFIXES
            .iter()
            .any(|loc| suffix_with_underscore.eq_ignore_ascii_case(loc))
        {
            return base;
        }
    }
    s
}

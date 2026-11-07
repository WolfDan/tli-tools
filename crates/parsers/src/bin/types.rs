//! Core domain types, limits, and `O(1)` table containers for `.bin` binary configs.

use crate::bin::error::{BinError, BinLimitKind};
use crate::bounded_len_u32;
use core::borrow::Borrow;
use core::hash::Hash;
use rustc_hash::FxHashMap;
use std::collections::hash_map::{Iter, Keys, Values};

/// Hard ceiling on compressed block count inside a single `*_ChunkA0.bin` header.
pub const CHUNK_BLOCKS_COUNT_MAX: u32 = 4_096;

/// Hard ceiling on uncompressed byte size of a single Zstandard block (`16 MiB`).
pub const BLOCK_UNCOMPRESSED_BYTES_MAX: u32 = 16 * 1024 * 1024;

/// Hard ceiling on total rows or records in a single `.bin` table (`2,000,000`).
pub const TABLE_ROWS_COUNT_MAX: u32 = 2_000_000;

/// Hard ceiling on byte length of a single wire string or byte slice (`1 MiB`).
pub const STRING_BYTES_MAX: u32 = 1_048_576;

/// Hard ceiling on element count of a single array or map column (`65,536`).
pub const COLLECTION_COUNT_MAX: u32 = 65_536;

/// Maximum outer key hierarchy depth (`1..=4`).
pub const KEY_DEPTH_COUNT_MAX: u32 = 4;

/// Maximum explicit work-stack depth when decoding `FConfigAffix` expression trees.
pub const AFFIX_STACK_DEPTH_MAX: u32 = 32;

/// Hard ceiling on total expression tree nodes decoded in a single `FConfigAffix` column.
pub const TREE_NODES_COUNT_MAX: u32 = 4_096;

/// Validated resource limits governing `.bin` container decompression and row decoding.
///
/// Fields are private to enforce `1 <= limit <= *_MAX` invariants at construction time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinLimits {
    block_count_max: u32,
    block_uncompressed_bytes_max: u32,
    rows_count_max: u32,
    string_bytes_max: u32,
    collection_count_max: u32,
    tree_nodes_count_max: u32,
}

impl BinLimits {
    /// Default production limits capable of decoding every `.bin` table in the game corpus.
    pub const DEFAULT: Self = Self {
        block_count_max: CHUNK_BLOCKS_COUNT_MAX,
        block_uncompressed_bytes_max: 2 * 1024 * 1024,
        rows_count_max: TABLE_ROWS_COUNT_MAX,
        string_bytes_max: STRING_BYTES_MAX,
        collection_count_max: COLLECTION_COUNT_MAX,
        tree_nodes_count_max: TREE_NODES_COUNT_MAX,
    };

    /// Constructs a validated `BinLimits`, rejecting `0` or values exceeding hard ceilings.
    pub const fn try_new(
        block_count_max: u32,
        block_uncompressed_bytes_max: u32,
        rows_count_max: u32,
        string_bytes_max: u32,
        collection_count_max: u32,
        tree_nodes_count_max: u32,
    ) -> Result<Self, BinError> {
        if block_count_max == 0 || block_count_max > CHUNK_BLOCKS_COUNT_MAX {
            return Err(BinError::InvalidLimits {
                kind: BinLimitKind::ChunkBlocksCount,
                value: block_count_max as u64,
                max: CHUNK_BLOCKS_COUNT_MAX as u64,
            });
        }
        if block_uncompressed_bytes_max == 0
            || block_uncompressed_bytes_max > BLOCK_UNCOMPRESSED_BYTES_MAX
        {
            return Err(BinError::InvalidLimits {
                kind: BinLimitKind::BlockUncompressedBytes,
                value: block_uncompressed_bytes_max as u64,
                max: BLOCK_UNCOMPRESSED_BYTES_MAX as u64,
            });
        }
        if rows_count_max == 0 || rows_count_max > TABLE_ROWS_COUNT_MAX {
            return Err(BinError::InvalidLimits {
                kind: BinLimitKind::RowsCount,
                value: rows_count_max as u64,
                max: TABLE_ROWS_COUNT_MAX as u64,
            });
        }
        if string_bytes_max == 0 || string_bytes_max > STRING_BYTES_MAX {
            return Err(BinError::InvalidLimits {
                kind: BinLimitKind::StringBytes,
                value: string_bytes_max as u64,
                max: STRING_BYTES_MAX as u64,
            });
        }
        if collection_count_max == 0 || collection_count_max > COLLECTION_COUNT_MAX {
            return Err(BinError::InvalidLimits {
                kind: BinLimitKind::CollectionCount,
                value: collection_count_max as u64,
                max: COLLECTION_COUNT_MAX as u64,
            });
        }
        if tree_nodes_count_max == 0 || tree_nodes_count_max > TREE_NODES_COUNT_MAX {
            return Err(BinError::InvalidLimits {
                kind: BinLimitKind::TreeNodesCount,
                value: tree_nodes_count_max as u64,
                max: TREE_NODES_COUNT_MAX as u64,
            });
        }
        Ok(Self {
            block_count_max,
            block_uncompressed_bytes_max,
            rows_count_max,
            string_bytes_max,
            collection_count_max,
            tree_nodes_count_max,
        })
    }

    /// Returns the maximum permitted block count in a `*_ChunkA0.bin` header.
    #[inline]
    #[must_use]
    pub const fn block_count_max(&self) -> u32 {
        self.block_count_max
    }

    /// Returns the maximum permitted uncompressed byte size of a single block.
    #[inline]
    #[must_use]
    pub const fn block_uncompressed_bytes_max(&self) -> u32 {
        self.block_uncompressed_bytes_max
    }

    /// Returns the maximum permitted row count in a single table.
    #[inline]
    #[must_use]
    pub const fn rows_count_max(&self) -> u32 {
        self.rows_count_max
    }

    /// Returns the maximum permitted byte length of a single wire string or byte slice.
    #[inline]
    #[must_use]
    pub const fn string_bytes_max(&self) -> u32 {
        self.string_bytes_max
    }

    /// Returns the maximum permitted element count of an array or map column.
    #[inline]
    #[must_use]
    pub const fn collection_count_max(&self) -> u32 {
        self.collection_count_max
    }

    /// Returns the maximum permitted expression tree node count in an `FConfigAffix` column.
    #[inline]
    #[must_use]
    pub const fn tree_nodes_count_max(&self) -> u32 {
        self.tree_nodes_count_max
    }
}

/// Primary-key hierarchy layout of a `.bin` table (`depth` `1..=4`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySchema {
    /// 1D `i32` key.
    Int1,
    /// 2D `(i32, i32)` key.
    Int2,
    /// 3D `(i32, i32, i32)` key.
    Int3,
    /// 4D `(i32, i32, i32, i32)` key.
    Int4,
    /// 1D `Box<str>` key.
    Str1,
    /// 2D `(Box<str>, i32)` key.
    StrInt1,
    /// 3D `(Box<str>, i32, i32)` key.
    StrInt2,
    /// 4D `(Box<str>, i32, i32, i32)` key.
    StrInt3,
}

impl KeySchema {
    /// Returns the hierarchy nesting depth (`1..=4`).
    #[inline]
    #[must_use]
    pub const fn depth(self) -> u8 {
        match self {
            Self::Int1 | Self::Str1 => 1,
            Self::Int2 | Self::StrInt1 => 2,
            Self::Int3 | Self::StrInt2 => 3,
            Self::Int4 | Self::StrInt3 => 4,
        }
    }

    /// Returns `true` if level `0` of the key hierarchy is a length-prefixed string.
    #[inline]
    #[must_use]
    pub const fn outer_is_str(self) -> bool {
        matches!(
            self,
            Self::Str1 | Self::StrInt1 | Self::StrInt2 | Self::StrInt3
        )
    }
}

/// Strongly-typed `.bin` configuration table backed by an `O(1)` [`FxHashMap<K, Row>`].
#[derive(Debug, Clone, PartialEq)]
pub struct BinTable<K: Eq + Hash, Row> {
    rows: FxHashMap<K, Row>,
}

impl<K: Eq + Hash, Row> BinTable<K, Row> {
    /// Wraps a populated [`FxHashMap<K, Row>`] into a [`BinTable`].
    #[inline]
    #[must_use]
    pub const fn from_map(rows: FxHashMap<K, Row>) -> Self {
        Self { rows }
    }

    /// Returns the number of rows in the table.
    #[inline]
    #[must_use]
    pub fn len(&self) -> u32 {
        bounded_len_u32(self.rows.len())
    }

    /// Returns `true` if the table contains no rows.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Returns a reference to the underlying `O(1)` [`FxHashMap<K, Row>`].
    #[inline]
    #[must_use]
    pub const fn rows(&self) -> &FxHashMap<K, Row> {
        &self.rows
    }

    /// Returns an iterator over the table's keys.
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, Row> {
        self.rows.keys()
    }

    /// Returns an iterator over the table's row structs.
    #[inline]
    pub fn values(&self) -> Values<'_, K, Row> {
        self.rows.values()
    }

    /// Looks up `key` in `O(1)` time and returns the corresponding `Row`.
    #[inline]
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&Row>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.rows.get(key)
    }

    /// Iterates over `(&K, &Row)` pairs in the table.
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, Row> {
        self.rows.iter()
    }
}

/// 2D or 3D floating-point vector column (`<dim: u8>` followed by `f64[2]` or `f64[3]`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VecDim {
    /// 2D vector (`dim == 2`, or `dim == 0` zeroed 2D vector).
    Vec2([f64; 2]),
    /// 3D vector (`dim == 3`).
    Vec3([f64; 3]),
}

impl Default for VecDim {
    #[inline]
    fn default() -> Self {
        Self::Vec2([0.0, 0.0])
    }
}

/// Single localized text entry in a `config_multi_language*.bin` table.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MultiLanguageRow {
    /// Author or translator identifier.
    pub author: Box<str>,
    /// Localized text content.
    pub text: Box<str>,
}

/// Strongly-typed `config_multi_language*.bin` table keyed by `(table_key, row_field_key)`.
pub type MultiLanguageTable = BinTable<(Box<str>, Box<str>), MultiLanguageRow>;

/// Single affix visual/gameplay effect record in `config_affix_effect.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AffixEffectRow {
    /// Serialized trigger JSON payload (`""` when unused).
    pub trigger_json: Box<str>,
    /// Serialized primary condition JSON payload (`""` when unused).
    pub condition1_json: Box<str>,
    /// Serialized secondary condition JSON payload (`""` when unused).
    pub condition2_json: Box<str>,
    /// Serialized pre-effect JSON payload (`""` when unused).
    pub pre_effect_json: Box<str>,
    /// Serialized effect JSON payload (`""` when unused).
    pub effect_json: Box<str>,
    /// Outer affix ID.
    pub id: i32,
    /// Outer sub-ID.
    pub sub_id: i32,
    /// Inner sub-ID.
    pub sub_id_inner: i32,
    /// Inner affix ID.
    pub id_inner: i32,
    /// Leading effect parameter 0 (`AffixData::ReadRow` offset `+0x08`).
    pub param0: i32,
    /// Trailing effect parameter 1 after `effect_json` (`AffixData::ReadRow` offset `+0xb0`).
    pub param1: i32,
}

/// Strongly-typed `config_affix_effect.bin` table keyed by `(id, sub_id)`.
pub type AffixEffectTable = BinTable<(i32, i32), AffixEffectRow>;

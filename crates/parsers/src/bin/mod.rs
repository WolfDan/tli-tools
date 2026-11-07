//! Strongly-typed TigerStyle parsers for Zstandard-compressed `*_ChunkA0.bin` tables,
//! uncompressed `Maps/<Map>/config_*.bin` tables, `config_multi_language*.bin`, and
//! `config_affix_effect.bin`.

pub mod affix;
pub mod container;
pub mod error;
#[rustfmt::skip]
pub mod tables;
pub mod types;
pub mod wire;

#[cfg(test)]
mod tests;

pub use affix::{AffixLookupEntry, AffixOperand, AffixSubOpKind, AffixTreeNode, FConfigAffix};
pub use container::{
    BinKey, BinRow, BinScratch, normalize_chunk_bin_stem, parse_affix_effect_table,
    parse_chunk_bin_table, parse_map_bin_table, parse_multi_language_table,
};
pub use error::{BinError, BinLimitKind, WireContext};
pub use types::{
    AFFIX_STACK_DEPTH_MAX, AffixEffectRow, AffixEffectTable, BLOCK_UNCOMPRESSED_BYTES_MAX,
    BinLimits, BinTable, CHUNK_BLOCKS_COUNT_MAX, COLLECTION_COUNT_MAX, KEY_DEPTH_COUNT_MAX,
    KeySchema, MultiLanguageRow, MultiLanguageTable, STRING_BYTES_MAX, TABLE_ROWS_COUNT_MAX,
    TREE_NODES_COUNT_MAX, VecDim,
};
pub use wire::WireCursor;

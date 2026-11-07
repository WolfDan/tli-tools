//! Concrete error types for `.bin` binary config and localization parsers.

use core::str::Utf8Error;
use thiserror::Error;

/// Explicit resource or wire limit category exceeded during `.bin` parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinLimitKind {
    /// Block count in a `*_ChunkA0.bin` header exceeded `block_count_max`.
    ChunkBlocksCount,
    /// Uncompressed byte size of a single block exceeded `block_uncompressed_bytes_max`.
    BlockUncompressedBytes,
    /// Total rows or records in a table exceeded `rows_count_max`.
    RowsCount,
    /// Byte length of a wire string or byte slice exceeded `string_bytes_max`.
    StringBytes,
    /// Element count of an array or map column exceeded `collection_count_max`.
    CollectionCount,
    /// Expression tree work-stack depth exceeded `AFFIX_STACK_DEPTH_MAX`.
    AffixStackDepth,
    /// Expression tree node count exceeded `tree_nodes_count_max`.
    TreeNodesCount,
}

/// Wire structure or column context where a layout, EOF, or UTF-8 failure occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireContext {
    /// `*_ChunkA0.bin` 8-byte header (`block_count`, `hash_buckets`).
    ChunkHeader,
    /// `*_ChunkA0.bin` block descriptor table (`RawBlockDesc[block_count]`).
    BlockDescriptorTable,
    /// Decompressed `*_ChunkA0.bin` block (`top_count` or trailing bytes).
    DecompressedBlock,
    /// Uncompressed map `.bin` header (`buckets`, `top_count`).
    MapContainerHeader,
    /// Uncompressed map `.bin` trailing bytes after reading all rows.
    MapContainerEof,
    /// Outer or leaf hierarchy key slot.
    KeyHierarchy,
    /// Primitive scalar or fixed-width column (`bool`, `u8`, `i32`, `u32`, `f64`, tuple, color).
    ScalarColumn,
    /// Length-prefixed string (`Box<str>`) or raw byte slice (`Box<[u8]>`).
    StringColumn,
    /// Dimensional vector column (`VecDim`: `dim` tag + `f64[2]` or `f64[3]`).
    VecDimColumn,
    /// Variable-length array column (`<u32 count>` + elements).
    ArrayColumn,
    /// Variable-length map column (`<u32 buckets><u32 count>` + entries).
    MapColumn,
    /// Compound `FConfig*` sub-struct column.
    SubStructColumn,
    /// `FConfigAffix` expression tree column.
    AffixExpression,
    /// `ActionData` tagged union column.
    ActionDataPayload,
    /// `config_multi_language*.bin` header or record.
    MultiLanguageRecord,
    /// `config_affix_effect.bin` header or record.
    AffixEffectRecord,
}

/// Structured error returned by `parsers::bin` decoders.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BinError {
    /// Invalid `BinLimits` configuration (a limit was `0` or exceeded a hard ceiling).
    #[error("invalid BinLimits for {kind:?}: value {value} (max allowed {max})")]
    InvalidLimits {
        /// Limit category that failed validation.
        kind: BinLimitKind,
        /// Provided value.
        value: u64,
        /// Maximum permitted ceiling.
        max: u64,
    },
    /// Unexpected end of buffer or invalid wire layout in `context`.
    #[error("corrupt .bin wire layout or unexpected EOF in {context:?}")]
    CorruptWireLayout {
        /// Wire region being decoded when the buffer ended early.
        context: WireContext,
    },
    /// Wire count or byte length exceeded the configured `BinLimits`.
    #[error("limit exceeded for {kind:?}: actual {actual} > max {max}")]
    LimitExceeded {
        /// Resource limit category that was exceeded.
        kind: BinLimitKind,
        /// Actual wire value encountered.
        actual: u64,
        /// Configured maximum limit.
        max: u64,
    },
    /// Compressed frame offsets in `RawBlockDesc` are non-monotonic or out of bounds.
    #[error(
        "invalid block {block_index} frame range [{start_offset}..{end_offset}] in payload of {payload_bytes} bytes"
    )]
    InvalidBlockRange {
        /// Zero-based block index (`0..block_count`).
        block_index: u16,
        /// Start byte offset in `payload`.
        start_offset: u32,
        /// End byte offset in `payload`.
        end_offset: u32,
        /// Total length of `payload` in bytes.
        payload_bytes: u64,
    },
    /// A block with `size_uncompressed == 0` had a non-empty compressed slice.
    #[error(
        "block {block_index} has size_uncompressed == 0 but compressed frame is {compressed_bytes} bytes"
    )]
    NonEmptyFrameForEmptyBlock {
        /// Zero-based block index.
        block_index: u16,
        /// Non-zero compressed byte count.
        compressed_bytes: u32,
    },
    /// Zstandard frame decompression failed for `block_index`.
    #[error("zstd decompression failed at block {block_index} (error code {code_raw})")]
    ZstdDecompressFailed {
        /// Zero-based block index.
        block_index: u16,
        /// Raw `zstd_safe` error code.
        code_raw: u64,
    },
    /// Decompressed block byte length did not match `RawBlockDesc::size_uncompressed`.
    #[error(
        "block {block_index} decompressed to {actual_bytes} bytes, expected {expected_bytes} bytes"
    )]
    DecompressedSizeMismatch {
        /// Zero-based block index.
        block_index: u16,
        /// Expected uncompressed byte count from descriptor.
        expected_bytes: u32,
        /// Actual decompressed byte count written by Zstandard.
        actual_bytes: u64,
    },
    /// Unconsumed trailing bytes remained at the end of a block or file.
    #[error("trailing unconsumed bytes in {context:?}: {remaining_bytes} bytes remaining")]
    TrailingBytes {
        /// Container context where EOF was expected.
        context: WireContext,
        /// Number of unconsumed bytes remaining.
        remaining_bytes: u64,
    },
    /// Invalid `VecDim` dimension tag (expected `2` or `3`, or `0` for zeroed 2D vector).
    #[error("invalid VecDim dimension tag: {raw_dim}")]
    InvalidVecDim {
        /// Raw dimension byte read from wire.
        raw_dim: u8,
    },
    /// Invalid `TreeNode` tag in an `FConfigAffix` expression tree (expected `0..=8`).
    #[error("invalid Affix TreeNode tag: {raw_tag} (expected 0..=8)")]
    InvalidTreeNodeTag {
        /// Raw tag byte read from wire.
        raw_tag: u8,
    },
    /// Invalid `TreeNode` tag `3` sub-operation opcode (`op > AFFIX_OP3_INDEX_MAX`).
    #[error("invalid Affix TreeNode tag-3 opcode: {raw_op}")]
    InvalidAffixOp3 {
        /// Raw opcode byte read from wire.
        raw_op: u8,
    },
    /// Invalid `ActionData` tag (`tag != 0` and outside `1000..=1000 + ACTION_DATA_INDEX_MAX`).
    #[error("invalid ActionData tag: {raw_tag}")]
    InvalidActionDataTag {
        /// Raw 32-bit signed tag read from wire.
        raw_tag: i32,
    },
    /// Duplicate primary key or map column key encountered.
    #[error("duplicate key in {context:?}")]
    DuplicateKey {
        /// Table or map column context where the duplicate key occurred.
        context: WireContext,
    },
    /// Invalid UTF-8 bytes inside a `Box<str>` string column or key.
    #[error("invalid UTF-8 in .bin string")]
    InvalidUtf8 {
        /// Underlying UTF-8 validation error.
        #[from]
        source: Utf8Error,
    },
    /// Unrecognized `.bin` table file stem in stem-based corpus dispatch.
    #[error("unknown .bin table stem `{stem}`")]
    UnknownTableStem {
        /// Unrecognized normalized stem.
        stem: Box<str>,
    },
}

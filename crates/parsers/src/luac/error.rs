//! Structured error hierarchy for `.out` / `\x01XDI` Lua 5.3 bytecode parsing, decompilation, and `DataCfg` evaluation.

use crate::luac::types::LuacLimitKind;
use thiserror::Error;

/// Errors occurring during `DataCfg` static table evaluation or typed row materialization.
#[derive(Debug, Error)]
pub enum DataCfgError {
    /// Encountered a VM instruction outside the permitted static data-construction subset.
    #[error("unsupported DataCfg instruction opcode {opcode} at pc {pc}")]
    UnsupportedInstruction { pc: u32, opcode: u8 },

    /// `LOADBOOL` attempted to perform a conditional jump skip (`C != 0`) inside a static table constructor.
    #[error("LOADBOOL with non-zero skip C={c} at pc {pc}")]
    LoadBoolSkipNotAllowed { pc: u32, c: u16 },

    /// `LOADKX` or `SETLIST C=0` was not immediately followed by an `EXTRAARG` instruction.
    #[error("missing EXTRAARG instruction after pc {pc}")]
    MissingExtraArg { pc: u32 },

    /// Register or constant index resolved to an out-of-bounds constant slot.
    #[error("constant index {index} out of bounds (count {count}) at pc {pc}")]
    ConstantIndexOutOfBounds { pc: u32, index: u32, count: u32 },

    /// Expected a table in register `reg`, but found a non-table value.
    #[error("register R[{reg}] does not hold a table at pc {pc}")]
    RegisterNotTable { pc: u32, reg: u8 },

    /// Expected a table value when decoding a row or nested record/list.
    #[error("expected table value for {context}")]
    ExpectedTable { context: &'static str },

    /// A single table used both `SETLIST` and `SETTABLE`.
    #[error("table {table_id} mixed list and map operations at pc {pc}")]
    MixedTableKind { pc: u32, table_id: u32 },

    /// Multi-batch `SETLIST` batch offset did not match the current list length.
    #[error(
        "SETLIST batch offset {expected_len} does not match current list length {actual_len} at pc {pc}"
    )]
    SetListBatchMismatch {
        pc: u32,
        expected_len: u32,
        actual_len: u32,
    },

    /// `SETLIST` count `B` was zero (multret) or `A + B` exceeded the 256-register file.
    #[error("invalid SETLIST register span A={a}, B={b} at pc {pc}")]
    InvalidSetListCount { pc: u32, a: u8, b: u16 },

    /// Root `Proto` did not contain the expected `mode_readonly` epilogue boundary before `CLOSURE`.
    #[error("missing DataCfg mode_readonly epilogue boundary")]
    MissingModeReadonlyBoundary,

    /// Encountered an unrecognized column name while decoding a typed `DataCfg` row struct.
    #[error("unknown column `{field}` in table `{table}`")]
    UnknownField {
        table: &'static str,
        field: Box<str>,
    },

    /// Column value type did not match the expected Rust field type.
    #[error("type mismatch on `{table}.{field}`: expected {expected}")]
    TypeMismatch {
        table: &'static str,
        field: &'static str,
        expected: &'static str,
    },

    /// Integer value in `DataCfg` overflowed the narrowed Rust integer field type.
    #[error("integer overflow on `{table}.{field}`: value {value} does not fit in {target_type}")]
    IntegerOverflow {
        table: &'static str,
        field: &'static str,
        value: i64,
        target_type: &'static str,
    },
}

/// Top-level error returned by `parsers::luac` operations.
#[derive(Debug, Error)]
pub enum LuacError {
    /// Input buffer ended prematurely before `needed` bytes could be read at `offset`.
    #[error(
        "unexpected end of buffer at byte offset {offset}: needed {needed} bytes, {remaining} remaining"
    )]
    UnexpectedEof {
        offset: u32,
        needed: u32,
        remaining: u32,
    },

    /// Decrypted buffer did not start with `\x01XDI\x11\r\n\x1a\n` or `\x1bLua`.
    #[error("invalid Lua bytecode magic prefix")]
    InvalidMagic,

    /// Standard 64-bit Lua 5.3 header bytes did not match the canonical 33-byte prefix.
    #[error("invalid canonical 64-bit Lua 5.3 header")]
    InvalidLua53Header,

    /// `\x01XDI` short string tag was in the invalid range `1..=9`.
    #[error("invalid XDI string tag 0x{tag:02X} at byte offset {offset} (must be 0x00 or >= 0x0A)")]
    InvalidXdiStringTag { offset: u32, tag: u8 },

    /// `\x01XDI` long string (`0xFF`) had a raw `u32` length `< 10`.
    #[error("invalid XDI long string raw length {raw_len} at byte offset {offset} (must be >= 10)")]
    InvalidXdiLongStringLength { offset: u32, raw_len: u32 },

    /// Canonical Lua 5.3 long string (`0xFF`) had a raw `u64` length `0` or exceeding `u32::MAX`.
    #[error("invalid Lua 5.3 long string raw length {raw_len} at byte offset {offset}")]
    InvalidLua53StringLength { offset: u32, raw_len: u64 },

    /// Constant pool string entry (`0x04` or `0x14`) carried a null string tag (`0x00`).
    #[error(
        "constant {const_index} in proto {proto_index} has string tag 0x{tag:02X} with null string payload"
    )]
    NullConstantString {
        proto_index: u32,
        const_index: u32,
        tag: u8,
    },

    /// Constant pool entry started with an unrecognized tag byte.
    #[error(
        "invalid constant tag 0x{tag:02X} at byte offset {offset} (proto {proto_index}, const {const_index})"
    )]
    InvalidConstantTag {
        offset: u32,
        proto_index: u32,
        const_index: u32,
        tag: u8,
    },

    /// Float constant (`0x03`) was `NaN` or infinite.
    #[error(
        "non-finite float64 constant (bits 0x{bits:016X}) in proto {proto_index}, const {const_index}"
    )]
    NonFiniteFloatConstant {
        proto_index: u32,
        const_index: u32,
        bits: u64,
    },

    /// `\x01XDI` instruction word had a top-6-bit opcode `>= 47`.
    #[error("invalid XDI opcode {raw_op} at proto {proto_index}, pc {pc}")]
    InvalidXdiOpcode {
        proto_index: u32,
        pc: u32,
        raw_op: u8,
    },

    /// Standard Lua 5.3 instruction word had a bottom-6-bit opcode `>= 47`.
    #[error("invalid Lua 5.3 opcode {raw_op} at proto {proto_index}, pc {pc}")]
    InvalidLua53Opcode {
        proto_index: u32,
        pc: u32,
        raw_op: u8,
    },

    /// Unconsumed trailing bytes remained after the root `Proto` ended.
    #[error("trailing bytes after root proto: consumed {consumed} of {total} bytes")]
    TrailingBytes { consumed: u32, total: u32 },

    /// Configured resource limit was exceeded.
    #[error("resource limit {kind:?} exceeded: actual {actual} > max {max}")]
    LimitExceeded {
        kind: LuacLimitKind,
        actual: u64,
        max: u64,
    },

    /// String bytes were not valid UTF-8.
    #[error("invalid UTF-8 string in Lua bytecode")]
    InvalidUtf8 {
        #[source]
        source: core::str::Utf8Error,
    },

    /// Static `DataCfg` table evaluation or struct decoding failed.
    #[error("DataCfg evaluation failed")]
    DataCfg {
        #[from]
        source: DataCfgError,
    },

    /// `unluac` decompilation pipeline failed.
    #[error("unluac decompilation failed")]
    Decompile {
        #[source]
        source: Box<unluac::decompile::DecompileError>,
    },

    /// `unluac` finished without populating the generated source chunk.
    #[error("unluac pipeline finished without generating source")]
    MissingGeneratedChunk,
}

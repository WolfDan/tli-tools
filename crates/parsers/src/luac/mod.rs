//! High-performance parser, `unluac` decompiler, and typed `DataCfg` table evaluator for `.out` / `\x01XDI` Lua 5.3 bytecode.

pub(crate) mod crypto;
mod datacfg;
mod decompile;
mod error;
pub(crate) mod opcode;
pub mod tables;
pub(crate) mod transpile;
mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use datacfg::{DataCfgRow, EvalContext, EvalVal, parse_luatable_datacfg};
pub use datacfg::{DataCfgScratch, DataTable, PreloadTable};
pub use decompile::{
    DecompileConfig, DecompileScratch, DecompiledChunkKind, DecompiledLua, decompile_luac,
};
pub use error::{DataCfgError, LuacError};
pub use types::{
    CONSTANTS_COUNT_MAX_DEFAULT, DATACFG_LIST_ITEMS_MAX_DEFAULT, DATACFG_MAP_ENTRIES_MAX_DEFAULT,
    DATACFG_TABLES_COUNT_MAX_DEFAULT, DataCfgLimits, INSTRUCTIONS_COUNT_MAX_DEFAULT, IntOrIntList,
    LOC_VARS_COUNT_MAX_DEFAULT, LuacLimitKind, LuacLimits, PROTO_DEPTH_HARD_MAX,
    PROTO_DEPTH_MAX_DEFAULT, PROTOS_COUNT_MAX_DEFAULT, TRANSPILE_BYTES_MAX_DEFAULT,
    UPVALUES_COUNT_MAX_DEFAULT,
};

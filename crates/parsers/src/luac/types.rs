//! Domain types, resource limits, and composite field containers for `parsers::luac`.

use crate::luac::error::LuacError;
use crate::u32_to_usize;

/// 9-byte magic header prefix of decrypted Xindong `\x01XDI` Lua 5.3 chunks.
pub(crate) const XDI_MAGIC: [u8; 9] = *b"\x01XDI\x11\r\n\x1a\n";

/// 33-byte header prefix of canonical 64-bit PUC-Lua 5.3 chunks (excluding `main_upvalue_count`).
pub(crate) const LUA53_HEADER: [u8; 33] = [
    0x1B, 0x4C, 0x75, 0x61, // "\x1bLua"
    0x53, 0x00, // version 0x53, format 0x00
    0x19, 0x93, 0x0D, 0x0A, 0x1A, 0x0A, // LUAC_DATA
    0x04, 0x08, 0x04, 0x08,
    0x08, // sizes: int=4, size_t=8, Instruction=4, lua_Integer=8, lua_Number=8
    0x78, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // LUAC_INT (0x5678i64)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x28, 0x77, 0x40, // LUAC_NUM (370.5f64)
];

/// Hard upper bound on nested `Proto` depth for inline `ArrayVec` work stacks.
pub const PROTO_DEPTH_HARD_MAX: u32 = 64;
pub(crate) const PROTO_DEPTH_HARD_MAX_USIZE: usize = u32_to_usize(PROTO_DEPTH_HARD_MAX);

/// Maximum control-flow block nesting depth within a single `Proto` during decompilation.
pub const DECOMPILE_BLOCK_DEPTH_MAX: u32 = 4096;

/// Default maximum `Proto` nesting depth.
pub const PROTO_DEPTH_MAX_DEFAULT: u32 = 32;
/// Default maximum number of `Proto`s in a single chunk.
pub const PROTOS_COUNT_MAX_DEFAULT: u32 = 16_384;
/// Default maximum number of instructions across a single chunk.
pub const INSTRUCTIONS_COUNT_MAX_DEFAULT: u32 = 524_288;
/// Default maximum number of constants across a single chunk.
pub const CONSTANTS_COUNT_MAX_DEFAULT: u32 = 262_144;
/// Default maximum number of upvalues in a single `Proto`.
pub const UPVALUES_COUNT_MAX_DEFAULT: u32 = 255;
/// Default maximum number of local variables across a single chunk.
pub const LOC_VARS_COUNT_MAX_DEFAULT: u32 = 65_536;
/// Default maximum transpiled Lua 5.3 buffer size in bytes (`32 MiB`).
pub const TRANSPILE_BYTES_MAX_DEFAULT: u32 = 32 * 1024 * 1024;

/// Default maximum number of tables constructed in a single `DataCfg` file.
pub const DATACFG_TABLES_COUNT_MAX_DEFAULT: u32 = 262_144;
/// Default maximum number of list elements across a single `DataCfg` file.
pub const DATACFG_LIST_ITEMS_MAX_DEFAULT: u32 = 1_048_576;
/// Default maximum number of map key-value entries across a single `DataCfg` file.
pub const DATACFG_MAP_ENTRIES_MAX_DEFAULT: u32 = 1_048_576;

/// Identifies which resource limit was exceeded during parsing, transpilation, or evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuacLimitKind {
    /// Input buffer length exceeds `u32::MAX`.
    InputBytes,
    /// `Proto` nesting depth exceeds `proto_depth_max`.
    ProtoDepth,
    /// Total `Proto` count exceeds `protos_count_max`.
    ProtosCount,
    /// Total instruction count exceeds `instructions_count_max`.
    InstructionsCount,
    /// Total constant count exceeds `constants_count_max`.
    ConstantsCount,
    /// Upvalue count in a `Proto` exceeds `upvalues_count_max`.
    UpvaluesCount,
    /// Local variable count exceeds `loc_vars_count_max`.
    LocVarsCount,
    /// Transpiled Lua 5.3 output size exceeds `transpile_bytes_max`.
    TranspileBytes,
    /// Total `DataCfg` tables count exceeds `tables_count_max`.
    DataCfgTablesCount,
    /// Total `DataCfg` list elements exceed `list_items_max`.
    DataCfgListItems,
    /// Total `DataCfg` map entries exceed `map_entries_max`.
    DataCfgMapEntries,
}

/// Resource bounds for `\x01XDI` / Lua 5.3 bytecode parsing and transpilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LuacLimits {
    proto_depth_max: u32,
    protos_count_max: u32,
    instructions_count_max: u32,
    constants_count_max: u32,
    upvalues_count_max: u32,
    loc_vars_count_max: u32,
    transpile_bytes_max: u32,
}

impl LuacLimits {
    /// Default limits sized above the maximums observed across `TLI_GAME_DIR`.
    pub const DEFAULT: Self = Self {
        proto_depth_max: PROTO_DEPTH_MAX_DEFAULT,
        protos_count_max: PROTOS_COUNT_MAX_DEFAULT,
        instructions_count_max: INSTRUCTIONS_COUNT_MAX_DEFAULT,
        constants_count_max: CONSTANTS_COUNT_MAX_DEFAULT,
        upvalues_count_max: UPVALUES_COUNT_MAX_DEFAULT,
        loc_vars_count_max: LOC_VARS_COUNT_MAX_DEFAULT,
        transpile_bytes_max: TRANSPILE_BYTES_MAX_DEFAULT,
    };

    /// Returns a copy with an updated `proto_depth_max` (`1..=PROTO_DEPTH_HARD_MAX`).
    pub fn try_with_proto_depth_max(mut self, max: u32) -> Result<Self, LuacError> {
        if max == 0 || max > PROTO_DEPTH_HARD_MAX {
            return Err(LuacError::LimitExceeded {
                kind: LuacLimitKind::ProtoDepth,
                actual: u64::from(max),
                max: u64::from(PROTO_DEPTH_HARD_MAX),
            });
        }
        self.proto_depth_max = max;
        Ok(self)
    }

    /// Returns a copy with an updated `protos_count_max`.
    #[must_use]
    pub const fn with_protos_count_max(mut self, max: u32) -> Self {
        self.protos_count_max = max;
        self
    }

    /// Returns a copy with an updated `instructions_count_max`.
    #[must_use]
    pub const fn with_instructions_count_max(mut self, max: u32) -> Self {
        self.instructions_count_max = max;
        self
    }

    /// Returns a copy with an updated `constants_count_max`.
    #[must_use]
    pub const fn with_constants_count_max(mut self, max: u32) -> Self {
        self.constants_count_max = max;
        self
    }

    /// Returns a copy with an updated `upvalues_count_max` (`<= 255`).
    pub fn try_with_upvalues_count_max(mut self, max: u32) -> Result<Self, LuacError> {
        if max > 255 {
            return Err(LuacError::LimitExceeded {
                kind: LuacLimitKind::UpvaluesCount,
                actual: u64::from(max),
                max: 255,
            });
        }
        self.upvalues_count_max = max;
        Ok(self)
    }

    /// Returns a copy with an updated `loc_vars_count_max`.
    #[must_use]
    pub const fn with_loc_vars_count_max(mut self, max: u32) -> Self {
        self.loc_vars_count_max = max;
        self
    }

    /// Returns a copy with an updated `transpile_bytes_max`.
    #[must_use]
    pub const fn with_transpile_bytes_max(mut self, max: u32) -> Self {
        self.transpile_bytes_max = max;
        self
    }

    #[must_use]
    pub const fn proto_depth_max(&self) -> u32 {
        self.proto_depth_max
    }

    #[must_use]
    pub const fn protos_count_max(&self) -> u32 {
        self.protos_count_max
    }

    #[must_use]
    pub const fn instructions_count_max(&self) -> u32 {
        self.instructions_count_max
    }

    #[must_use]
    pub const fn constants_count_max(&self) -> u32 {
        self.constants_count_max
    }

    #[must_use]
    pub const fn upvalues_count_max(&self) -> u32 {
        self.upvalues_count_max
    }

    #[must_use]
    pub const fn loc_vars_count_max(&self) -> u32 {
        self.loc_vars_count_max
    }

    #[must_use]
    pub const fn transpile_bytes_max(&self) -> u32 {
        self.transpile_bytes_max
    }
}

/// Resource bounds for `DataCfg` static table evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataCfgLimits {
    luac: LuacLimits,
    tables_count_max: u32,
    list_items_max: u32,
    map_entries_max: u32,
}

impl DataCfgLimits {
    /// Default limits sized above the largest `DataCfg` files in `TLI_GAME_DIR`.
    pub const DEFAULT: Self = Self {
        luac: LuacLimits::DEFAULT,
        tables_count_max: DATACFG_TABLES_COUNT_MAX_DEFAULT,
        list_items_max: DATACFG_LIST_ITEMS_MAX_DEFAULT,
        map_entries_max: DATACFG_MAP_ENTRIES_MAX_DEFAULT,
    };

    #[must_use]
    pub const fn with_luac_limits(mut self, luac: LuacLimits) -> Self {
        self.luac = luac;
        self
    }

    #[must_use]
    pub const fn with_tables_count_max(mut self, max: u32) -> Self {
        self.tables_count_max = max;
        self
    }

    #[must_use]
    pub const fn with_list_items_max(mut self, max: u32) -> Self {
        self.list_items_max = max;
        self
    }

    #[must_use]
    pub const fn with_map_entries_max(mut self, max: u32) -> Self {
        self.map_entries_max = max;
        self
    }

    #[must_use]
    pub const fn luac(&self) -> LuacLimits {
        self.luac
    }

    #[must_use]
    pub const fn tables_count_max(&self) -> u32 {
        self.tables_count_max
    }

    #[must_use]
    pub const fn list_items_max(&self) -> u32 {
        self.list_items_max
    }

    #[must_use]
    pub const fn map_entries_max(&self) -> u32 {
        self.map_entries_max
    }
}

/// Value that is either a single integer or a list of integers (`config_preload_sp_map_v2_level_id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntOrIntList<T> {
    Int(T),
    List(Box<[T]>),
}

impl<T: Default> Default for IntOrIntList<T> {
    fn default() -> Self {
        Self::Int(T::default())
    }
}

/// Internal byte span referencing a slice inside the input bytecode buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ByteSpan {
    pub(crate) offset: u32,
    pub(crate) len: u32,
}
const _: () = assert!(core::mem::size_of::<ByteSpan>() == 8);

/// Internal cipher state of a `.out` / `.luac` buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CipherState {
    PlainXdi,
    PlainLua53,
    DecryptedFromCipher,
}

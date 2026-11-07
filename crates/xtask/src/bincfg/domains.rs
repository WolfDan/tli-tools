//! Domain module classification, inductive wire-type AST, and alignment-tier ordering for `.bin` tables.

use crate::datacfg::domains::{strip_config_prefix, to_snake_case};
use proc_macro2::TokenStream;
use quote::quote;

/// Key scalar kind for hash-bucketed `.bin` map columns (`<buckets: u32><count: u32>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MapKeyWire {
    /// 32-bit signed integer key (`i32`).
    I32,
    /// Length-prefixed UTF-8 string key (`Box<str>`).
    Str,
}

impl MapKeyWire {
    /// Returns the Rust key type token stream for this map key kind.
    #[inline]
    #[must_use]
    pub fn rust_type_tokens(self) -> TokenStream {
        match self {
            Self::I32 => quote! { i32 },
            Self::Str => quote! { Box<str> },
        }
    }

    /// Returns the compact PascalCase token for this map key kind.
    #[inline]
    #[must_use]
    pub const fn type_token(self) -> &'static str {
        match self {
            Self::I32 => "I32",
            Self::Str => "Str",
        }
    }
}

/// Inductive wire-level column or sub-struct field type extracted from `ReadBin` disassembly.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WireType {
    /// 1-byte boolean (`bool`).
    Bool,
    /// 4-byte little-endian signed integer (`i32`).
    I32,
    /// 8-byte little-endian IEEE-754 float (`f64`).
    F64,
    /// Length-prefixed UTF-8 string (`Box<str>`).
    Str,
    /// Length-prefixed raw byte slice (`Box<[u8]>`), promoted when non-UTF-8 bytes occur in corpus.
    Bytes,
    /// 1-byte dimension tag (`0 | 2 | 3`) followed by `f64[2]` or `f64[3]` (`VecDim`).
    VecDim,
    /// `FConfigAffix` expression tree (`FConfigAffix`).
    AffixEntry,
    /// Tagged polymorphic gameplay action payload (`ActionData`).
    ActionData,
    /// Length-prefixed homogeneous array (`<count: u32><T[count]>`).
    Array(Box<Self>),
    /// Hash-bucketed key-value map (`<buckets: u32><count: u32><(K, V)[count]>`).
    Map(MapKeyWire, Box<Self>),
    /// Compound sequence of fields decoded in order (`struct`).
    Struct(Vec<Self>),
}

impl WireType {
    /// Returns the alignment tier (`0` = 8-byte aligned, `1` = 4-byte aligned, `2` = 1-byte aligned)
    /// used to order generated struct fields so padding is `< 4` bytes per struct.
    #[must_use]
    pub fn align_tier(&self) -> u8 {
        match self {
            Self::Bool => 2,
            Self::I32 => 1,
            Self::Struct(fields) => fields.iter().fold(2u8, |acc, f| acc.min(f.align_tier())),
            Self::F64
            | Self::Str
            | Self::Bytes
            | Self::VecDim
            | Self::AffixEntry
            | Self::ActionData
            | Self::Array(_)
            | Self::Map(_, _) => 0,
        }
    }

    /// Returns `true` if this wire type contains `f64` (preventing `#[derive(Eq)]`).
    #[must_use]
    pub fn has_float(&self) -> bool {
        match self {
            Self::F64 | Self::VecDim | Self::AffixEntry | Self::ActionData => true,
            Self::Bool | Self::I32 | Self::Str | Self::Bytes => false,
            Self::Array(elem) | Self::Map(_, elem) => elem.has_float(),
            Self::Struct(fields) => fields.iter().any(Self::has_float),
        }
    }

    /// Returns `true` if this wire type directly (outside a named `Struct`) uses `FxHashMap`.
    #[must_use]
    pub fn uses_direct_fx_hash_map(&self) -> bool {
        match self {
            Self::Map(_, _) => true,
            Self::Array(elem) => elem.uses_direct_fx_hash_map(),
            Self::Bool
            | Self::I32
            | Self::F64
            | Self::Str
            | Self::Bytes
            | Self::VecDim
            | Self::AffixEntry
            | Self::ActionData
            | Self::Struct(_) => false,
        }
    }

    /// Returns `true` if this wire type directly (outside a named `Struct`) uses `VecDim`.
    #[must_use]
    pub fn uses_direct_vec_dim(&self) -> bool {
        match self {
            Self::VecDim => true,
            Self::Array(elem) | Self::Map(_, elem) => elem.uses_direct_vec_dim(),
            Self::Bool
            | Self::I32
            | Self::F64
            | Self::Str
            | Self::Bytes
            | Self::AffixEntry
            | Self::ActionData
            | Self::Struct(_) => false,
        }
    }

    /// Returns `true` if this wire type directly (outside a named `Struct`) uses `FConfigAffix`.
    #[must_use]
    pub fn uses_direct_affix(&self) -> bool {
        match self {
            Self::AffixEntry => true,
            Self::Array(elem) | Self::Map(_, elem) => elem.uses_direct_affix(),
            Self::Bool
            | Self::I32
            | Self::F64
            | Self::Str
            | Self::Bytes
            | Self::VecDim
            | Self::ActionData
            | Self::Struct(_) => false,
        }
    }

    /// Returns `true` if this wire type directly (outside a named `Struct`) uses `ActionData`.
    #[must_use]
    pub fn uses_direct_action_data(&self) -> bool {
        match self {
            Self::ActionData => true,
            Self::Array(elem) | Self::Map(_, elem) => elem.uses_direct_action_data(),
            Self::Bool
            | Self::I32
            | Self::F64
            | Self::Str
            | Self::Bytes
            | Self::VecDim
            | Self::AffixEntry
            | Self::Struct(_) => false,
        }
    }

    /// Builds a deterministic compact `PascalCase` token describing this `WireType`.
    #[must_use]
    pub fn type_token(&self) -> String {
        match self {
            Self::Bool => "Bool".to_owned(),
            Self::I32 => "I32".to_owned(),
            Self::F64 => "F64".to_owned(),
            Self::Str => "Str".to_owned(),
            Self::Bytes => "Bytes".to_owned(),
            Self::VecDim => "VecDim".to_owned(),
            Self::AffixEntry => "Affix".to_owned(),
            Self::ActionData => "ActionData".to_owned(),
            Self::Array(elem) => {
                if let Self::Array(inner) = &**elem {
                    format!("Array2d{}", inner.type_token())
                } else {
                    format!("Array{}", elem.type_token())
                }
            }
            Self::Map(k, v) => format!("Map{}{}", k.type_token(), v.type_token()),
            Self::Struct(fields) => {
                let mut out = String::new();
                let mut i = 0;
                while i < fields.len() {
                    let tok = fields[i].type_token();
                    let mut run = 1usize;
                    while i + run < fields.len() && fields[i + run] == fields[i] {
                        run += 1;
                    }
                    if run >= 3 {
                        out.push_str(&format!("{tok}x{run}"));
                    } else {
                        for _ in 0..run {
                            out.push_str(&tok);
                        }
                    }
                    i += run;
                }
                out
            }
        }
    }
}

/// Expression tree operand kind inside `TreeNode::Op3`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubOp {
    FormulaOrF64,
    FormulaOrI32,
    FormulaOrBool,
    FormulaOrU8,
    ArrayFormulaOrI32,
}

const BIN_DOMAIN_RULES: &[(&str, &[&str])] = &[
    ("affix", &["affix", "random_affix", "season_affix"]),
    (
        "attribute",
        &[
            "attribute",
            "attributes",
            "damage",
            "resist",
            "shield",
            "state",
            "tag",
            "terminology",
            "word",
        ],
    ),
    (
        "audio",
        &["audio", "bgm", "cv", "music", "sound", "subtitle", "voice"],
    ),
    ("buff", &["aura", "buff", "char_buff", "debuff"]),
    (
        "char",
        &[
            "action",
            "anim",
            "avatar",
            "camera",
            "career",
            "char",
            "emote",
            "player",
            "portrait",
            "profession",
            "role",
        ],
    ),
    (
        "drop",
        &["award", "chest", "drop", "loot", "reward", "treasure"],
    ),
    (
        "equip",
        &[
            "armor",
            "ashes",
            "candle",
            "corrosion",
            "enchant",
            "equip",
            "forge",
            "gem",
            "inlay",
            "memory",
            "weapon",
        ],
    ),
    ("hero", &["hero", "relic", "trait"]),
    (
        "item",
        &[
            "auction", "bag", "card", "consum", "currency", "fashion", "gacha", "gold", "item",
            "money", "pack", "prop", "shop", "tattoo", "trade",
        ],
    ),
    (
        "level",
        &[
            "achieve",
            "chapter",
            "cinematic",
            "cutscene",
            "dialog",
            "guide",
            "level",
            "mission",
            "plot",
            "quest",
            "stage",
            "story",
            "task",
            "tutorial",
        ],
    ),
    (
        "map",
        &[
            "area",
            "climb_tower",
            "dungeon",
            "environment",
            "map",
            "portal",
            "tower",
            "waypoint",
            "weather",
            "world",
        ],
    ),
    (
        "mercenary",
        &["companion", "follower", "mercenary", "partner"],
    ),
    (
        "modifier",
        &[
            "bullet", "calc", "effect", "formula", "hit", "modifier", "summon", "trap", "trigger",
        ],
    ),
    (
        "monster",
        &["ai", "boss", "camp", "faction", "monster", "spawn", "wave"],
    ),
    (
        "npc",
        &[
            "device",
            "gadget",
            "interact",
            "mechanism",
            "npc",
            "obstacle",
        ],
    ),
    (
        "resource",
        &[
            "asset", "decal", "fx", "icon", "light", "material", "mesh", "model", "particle",
            "preload", "resource", "sfx", "spine", "texture", "vfx",
        ],
    ),
    (
        "season",
        &[
            "activity",
            "chess",
            "clockwork",
            "cube",
            "doll",
            "eternal",
            "event",
            "fishing",
            "gameplay",
            "god",
            "hunting",
            "labyrinth",
            "mist",
            "moba",
            "nightmare",
            "pirate",
            "plague",
            "roguelike",
            "sandtable",
            "season",
            "td",
            "train",
        ],
    ),
    ("servant", &["pet", "servant"]),
    ("skill", &["passive", "skill", "spell", "support_skill"]),
    ("sp", &["chaos", "constellation", "sp_", "star", "void"]),
    (
        "talent",
        &["devil", "divinity", "pedigree", "prism", "slate", "talent"],
    ),
    (
        "ui",
        &[
            "chat", "color", "font", "friend", "gamepad", "guild", "hotkey", "hud", "input",
            "language", "mail", "notice", "panel", "rank", "style", "team", "text", "tip", "ui",
            "window",
        ],
    ),
];

/// Classifies a `.bin` table stem (`config_*`) into its target domain module.
#[must_use]
pub fn classify_bin_domain(stem: &str, is_map_table: bool) -> &'static str {
    if is_map_table {
        return "map_instance";
    }
    let s = to_snake_case(strip_config_prefix(stem));
    if s.starts_with('s') && s.as_bytes().get(1).is_some_and(u8::is_ascii_digit) {
        return "season";
    }
    for &(domain, prefixes) in BIN_DOMAIN_RULES {
        if prefixes.iter().any(|p| s.starts_with(p)) {
            return domain;
        }
    }
    "system"
}

/// Converts a C++ `F*Info` or `LuaBridge` property name into a valid, unique Rust `snake_case` field name.
#[must_use]
pub fn normalize_field_name(raw_name: &str, fallback_idx: u32) -> String {
    let trimmed = raw_name.trim_matches('_');
    if trimmed.is_empty() {
        return format!("col_{fallback_idx}");
    }
    let snake = to_snake_case(trimmed);
    if snake.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        format!("f_{snake}")
    } else {
        snake
    }
}

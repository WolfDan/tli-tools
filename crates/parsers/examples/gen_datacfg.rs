//! Automated code generator for `crates/parsers/src/luac/tables/`.

use pak::{AesKey, ReadScratch, ScratchLimits};
use pak_vfs::{FileId, PakVfs};
use parsers::u32_to_usize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Write as _;
use std::path::Path;
use thiserror::Error;

const XDI_TO_LUA53_OPCODE: [u8; 47] = [
    4, 15, 10, 33, 11, 2, 1, 17, 7, 35, 32, 31, 0, 45, 5, 3, 19, 18, 6, 8, 21, 38, 13, 29, 36, 42,
    44, 40, 43, 37, 39, 20, 9, 22, 30, 16, 27, 12, 26, 25, 24, 41, 34, 28, 14, 23, 46,
];

#[derive(Debug, Error)]
enum GenError {
    #[error("failed to read environment variable")]
    Env(#[from] std::env::VarError),
    #[error("pak archive error")]
    Pak(#[from] pak::PakError),
    #[error("pak vfs error")]
    Vfs(#[from] pak_vfs::VfsError),
    #[error("filesystem I/O error")]
    Io(#[from] std::io::Error),
    #[error("invalid UTF-8 in bytecode or path")]
    Utf8(#[from] core::str::Utf8Error),
    #[error("unexpected eof or malformed bytecode in {context}")]
    MalformedBytecode { context: &'static str },
    #[error("incompatible inferred types: {left} vs {right}")]
    IncompatibleTypes { left: String, right: String },
}

const fn low_u8(x: u64) -> u8 {
    (x & 0xFF) as u8
}

fn decrypt_in_place(buf: &mut [u8]) {
    if buf.starts_with(b"\x01XDI") || buf.starts_with(b"\x1bLua") {
        return;
    }
    for (i, byte) in buf.iter_mut().enumerate() {
        if i <= 1 {
            continue;
        }
        let i64_val = i as u64;
        let v = i64_val.wrapping_mul(0x2021_0507);
        let mask = match i % 3 {
            0 => low_u8(i64_val.wrapping_add(v & 1).wrapping_add(v >> 28)),
            1 => low_u8(((v >> 16) & 0xFF).wrapping_sub(i64_val)),
            _ => low_u8(i64_val | (v >> 21)),
        };
        *byte ^= mask;
    }
}

#[derive(Clone, Debug)]
enum Val {
    Nil,
    Bool,
    Int(i64),
    Float,
    Str(String),
    Table(u32),
}

#[derive(Clone, Debug, Default)]
struct TableObj {
    list: Vec<Val>,
    map: Vec<(Val, Val)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IntBounds {
    min: i64,
    max: i64,
}

impl IntBounds {
    const fn single(v: i64) -> Self {
        Self { min: v, max: v }
    }

    fn merge(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    const fn rust_type(self) -> &'static str {
        if self.min >= 0 {
            if self.max <= u8::MAX as i64 {
                "u8"
            } else if self.max <= u16::MAX as i64 {
                "u16"
            } else if self.max <= u32::MAX as i64 {
                "u32"
            } else {
                "u64"
            }
        } else if self.min >= i8::MIN as i64 && self.max <= i8::MAX as i64 {
            "i8"
        } else if self.min >= i16::MIN as i64 && self.max <= i16::MAX as i64 {
            "i16"
        } else if self.min >= i32::MIN as i64 && self.max <= i32::MAX as i64 {
            "i32"
        } else {
            "i64"
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InferredType {
    EmptyTable,
    Bool,
    Int(IntBounds),
    Float,
    Str,
    List {
        elem: Box<InferredType>,
        has_empty: bool,
    },
    KvRecord {
        key: Box<InferredType>,
        val: Box<InferredType>,
        tuple_elems: Option<Vec<InferredType>>,
    },
    StringRecord(BTreeMap<String, InferredType>),
    MapTable(Box<InferredType>, Box<InferredType>),
    IntOrList(IntBounds),
}

fn merge_kv_records(
    ka: InferredType,
    va: InferredType,
    ta: Option<Vec<InferredType>>,
    kb: InferredType,
    vb: InferredType,
    tb: Option<Vec<InferredType>>,
) -> Result<InferredType, GenError> {
    let merged_tuple = match (ta, tb) {
        (Some(va_list), Some(vb_list)) if va_list.len() == vb_list.len() => {
            let mut elems = Vec::with_capacity(va_list.len());
            for (x, y) in va_list.into_iter().zip(vb_list) {
                elems.push(InferredType::merge(x, y)?);
            }
            Some(elems)
        }
        _ => None,
    };
    Ok(InferredType::KvRecord {
        key: Box::new(InferredType::merge(ka, kb)?),
        val: Box::new(InferredType::merge(va, vb)?),
        tuple_elems: merged_tuple,
    })
}

fn merge_string_records(
    mut ma: BTreeMap<String, InferredType>,
    mb: BTreeMap<String, InferredType>,
) -> Result<InferredType, GenError> {
    if ma.keys().eq(mb.keys()) {
        for (k, vb) in mb {
            if let Some(va) = ma.remove(&k) {
                ma.insert(k, InferredType::merge(va, vb)?);
            }
        }
        Ok(InferredType::StringRecord(ma))
    } else {
        let mut val_ty = InferredType::EmptyTable;
        for (_, v) in ma.into_iter().chain(mb) {
            val_ty = InferredType::merge(val_ty, v)?;
        }
        Ok(InferredType::MapTable(
            Box::new(InferredType::Str),
            Box::new(val_ty),
        ))
    }
}

fn merge_int_and_elem(ba: IntBounds, eb: InferredType) -> Result<InferredType, GenError> {
    match eb {
        InferredType::Int(bb) => Ok(InferredType::IntOrList(ba.merge(bb))),
        other @ (InferredType::EmptyTable
        | InferredType::Bool
        | InferredType::Float
        | InferredType::Str
        | InferredType::List { .. }
        | InferredType::KvRecord { .. }
        | InferredType::StringRecord(_)
        | InferredType::MapTable(_, _)
        | InferredType::IntOrList(_)) => Err(GenError::IncompatibleTypes {
            left: format!("Int({ba:?})"),
            right: format!("List({other:?})"),
        }),
    }
}

fn strip_config_prefix(s: &str) -> &str {
    match s.strip_prefix("config_") {
        Some(rest) => rest,
        None => s,
    }
}

fn merge_map_and_string_record(
    k: Box<InferredType>,
    mut val_ty: InferredType,
    m: BTreeMap<String, InferredType>,
) -> Result<InferredType, GenError> {
    for (_, mv) in m {
        val_ty = InferredType::merge(val_ty, mv)?;
    }
    Ok(InferredType::MapTable(k, Box::new(val_ty)))
}

fn merge_empty_with(other: InferredType) -> InferredType {
    match other {
        InferredType::List { elem, .. } => InferredType::List {
            elem,
            has_empty: true,
        },
        InferredType::KvRecord { key, val, .. } => InferredType::KvRecord {
            key,
            val,
            tuple_elems: None,
        },
        InferredType::EmptyTable
        | InferredType::Bool
        | InferredType::Int(_)
        | InferredType::Float
        | InferredType::Str
        | InferredType::StringRecord(_)
        | InferredType::MapTable(_, _)
        | InferredType::IntOrList(_) => other,
    }
}

impl InferredType {
    fn merge(a: InferredType, b: InferredType) -> Result<InferredType, GenError> {
        match (a, b) {
            (Self::EmptyTable, other) | (other, Self::EmptyTable) => Ok(merge_empty_with(other)),
            (Self::Bool, Self::Bool) => Ok(Self::Bool),
            (Self::Int(ba), Self::Int(bb)) => Ok(Self::Int(ba.merge(bb))),
            (Self::Float, Self::Float)
            | (Self::Int(_), Self::Float)
            | (Self::Float, Self::Int(_)) => Ok(Self::Float),
            (Self::Str, Self::Str) => Ok(Self::Str),
            (
                Self::List {
                    elem: ea,
                    has_empty: ha,
                },
                Self::List {
                    elem: eb,
                    has_empty: hb,
                },
            ) => Ok(Self::List {
                elem: Box::new(Self::merge(*ea, *eb)?),
                has_empty: ha || hb,
            }),
            (
                Self::KvRecord {
                    key: ka,
                    val: va,
                    tuple_elems: ta,
                },
                Self::KvRecord {
                    key: kb,
                    val: vb,
                    tuple_elems: tb,
                },
            ) => merge_kv_records(*ka, *va, ta, *kb, *vb, tb),
            (Self::StringRecord(ma), Self::StringRecord(mb)) => merge_string_records(ma, mb),
            (Self::StringRecord(m), Self::MapTable(k, v))
            | (Self::MapTable(k, v), Self::StringRecord(m)) => {
                merge_map_and_string_record(k, *v, m)
            }
            (Self::MapTable(ka, va), Self::MapTable(kb, vb)) => Ok(Self::MapTable(
                Box::new(Self::merge(*ka, *kb)?),
                Box::new(Self::merge(*va, *vb)?),
            )),
            (Self::Int(ba) | Self::IntOrList(ba), Self::List { elem: eb, .. })
            | (Self::List { elem: eb, .. }, Self::Int(ba) | Self::IntOrList(ba)) => {
                merge_int_and_elem(ba, *eb)
            }
            (Self::IntOrList(ba), Self::Int(bb) | Self::IntOrList(bb))
            | (Self::Int(ba), Self::IntOrList(bb)) => Ok(Self::IntOrList(ba.merge(bb))),
            (
                left @ (Self::Bool
                | Self::Int(_)
                | Self::Float
                | Self::Str
                | Self::List { .. }
                | Self::KvRecord { .. }
                | Self::StringRecord(_)
                | Self::MapTable(_, _)
                | Self::IntOrList(_)),
                right,
            ) => Err(GenError::IncompatibleTypes {
                left: format!("{left:?}"),
                right: format!("{right:?}"),
            }),
        }
    }

    fn is_copy(&self) -> bool {
        match self {
            Self::EmptyTable | Self::Bool | Self::Int(_) | Self::Float => true,
            Self::Str | Self::List { .. } | Self::MapTable(_, _) | Self::IntOrList(_) => false,
            Self::KvRecord {
                key,
                val,
                tuple_elems,
            } => {
                key.is_copy()
                    && match tuple_elems {
                        Some(elems) => elems.iter().all(Self::is_copy),
                        None => val.is_copy(),
                    }
            }
            Self::StringRecord(m) => m.values().all(Self::is_copy),
        }
    }

    fn has_float(&self) -> bool {
        match self {
            Self::Float => true,
            Self::EmptyTable | Self::Bool | Self::Int(_) | Self::Str | Self::IntOrList(_) => false,
            Self::List { elem, .. } => elem.has_float(),
            Self::KvRecord {
                key,
                val,
                tuple_elems,
            } => {
                key.has_float()
                    || match tuple_elems {
                        Some(elems) => elems.iter().any(Self::has_float),
                        None => val.has_float(),
                    }
            }
            Self::StringRecord(m) => m.values().any(Self::has_float),
            Self::MapTable(k, v) => k.has_float() || v.has_float(),
        }
    }

    fn uses_fx_hash_map(&self) -> bool {
        match self {
            Self::MapTable(_, _) => true,
            Self::List { elem, .. } => elem.uses_fx_hash_map(),
            Self::KvRecord { key, val, .. } => key.uses_fx_hash_map() || val.uses_fx_hash_map(),
            Self::StringRecord(m) => m.values().any(Self::uses_fx_hash_map),
            Self::EmptyTable
            | Self::Bool
            | Self::Int(_)
            | Self::Float
            | Self::Str
            | Self::IntOrList(_) => false,
        }
    }

    fn uses_int_or_list(&self) -> bool {
        match self {
            Self::IntOrList(_) => true,
            Self::List { elem, .. } => elem.uses_int_or_list(),
            Self::KvRecord { key, val, .. } => key.uses_int_or_list() || val.uses_int_or_list(),
            Self::StringRecord(m) => m.values().any(Self::uses_int_or_list),
            Self::MapTable(k, v) => k.uses_int_or_list() || v.uses_int_or_list(),
            Self::EmptyTable | Self::Bool | Self::Int(_) | Self::Float | Self::Str => false,
        }
    }
}

fn find_nested_record(ty: &InferredType) -> Option<(&InferredType, bool)> {
    match ty {
        InferredType::KvRecord { .. } | InferredType::StringRecord(_) => Some((ty, false)),
        InferredType::List { elem, .. } => find_nested_record(elem).map(|(r, _)| (r, true)),
        InferredType::MapTable(_, v) => find_nested_record(v).map(|(r, _)| (r, true)),
        InferredType::EmptyTable
        | InferredType::Bool
        | InferredType::Int(_)
        | InferredType::Float
        | InferredType::Str
        | InferredType::IntOrList(_) => None,
    }
}

fn is_valid_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().enumerate().all(|(i, c)| {
            if i == 0 {
                c.is_ascii_alphabetic() || c == '_'
            } else {
                c.is_ascii_alphanumeric() || c == '_'
            }
        })
}

fn infer_map_table(t: &TableObj, tables: &[TableObj]) -> Result<InferredType, GenError> {
    let all_str_ident_keys = t
        .map
        .iter()
        .all(|(k, _)| matches!(k, Val::Str(s) if is_valid_ident(s)));
    if all_str_ident_keys {
        let mut field_map = BTreeMap::new();
        for (k, v) in &t.map {
            if let Val::Str(s) = k {
                field_map.insert(s.clone(), infer_val(v, tables)?);
            }
        }
        if field_map.len() == 2
            && let Some(k_ty) = field_map.remove("Key")
            && let Some(v_ty) = field_map.remove("Value")
        {
            let mut tuple_elems = None;
            for (k, v) in &t.map {
                if let (Val::Str(s), Val::Table(vtid)) = (k, v)
                    && s == "Value"
                {
                    let vt = &tables[u32_to_usize(*vtid)];
                    if (2..=4).contains(&vt.list.len())
                        && vt.list.iter().all(|x| !matches!(x, Val::Table(_)))
                    {
                        let mut elems = Vec::with_capacity(vt.list.len());
                        for item in &vt.list {
                            elems.push(infer_val(item, tables)?);
                        }
                        tuple_elems = Some(elems);
                    }
                }
            }
            return Ok(InferredType::KvRecord {
                key: Box::new(k_ty),
                val: Box::new(v_ty),
                tuple_elems,
            });
        }
        return Ok(InferredType::StringRecord(field_map));
    }
    let mut k_ty = infer_val(&t.map[0].0, tables)?;
    let mut v_ty = infer_val(&t.map[0].1, tables)?;
    for (k, v) in &t.map[1..] {
        k_ty = InferredType::merge(k_ty, infer_val(k, tables)?)?;
        v_ty = InferredType::merge(v_ty, infer_val(v, tables)?)?;
    }
    Ok(InferredType::MapTable(Box::new(k_ty), Box::new(v_ty)))
}

fn infer_val(val: &Val, tables: &[TableObj]) -> Result<InferredType, GenError> {
    match val {
        Val::Nil => Err(GenError::MalformedBytecode {
            context: "unexpected nil value",
        }),
        Val::Bool => Ok(InferredType::Bool),
        Val::Int(i) => Ok(InferredType::Int(IntBounds::single(*i))),
        Val::Float => Ok(InferredType::Float),
        Val::Str(_) => Ok(InferredType::Str),
        Val::Table(tid) => {
            let t = &tables[u32_to_usize(*tid)];
            if t.list.is_empty() && t.map.is_empty() {
                return Ok(InferredType::EmptyTable);
            }
            if !t.list.is_empty() {
                let mut elem = infer_val(&t.list[0], tables)?;
                for item in &t.list[1..] {
                    elem = InferredType::merge(elem, infer_val(item, tables)?)?;
                }
                return Ok(InferredType::List {
                    elem: Box::new(elem),
                    has_empty: false,
                });
            }
            infer_map_table(t, tables)
        }
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut cap = true;
    for ch in s.chars() {
        if ch == '_' {
            cap = true;
        } else if cap {
            out.push(ch.to_ascii_uppercase());
            cap = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                let next_is_lower = chars.get(i + 1).is_some_and(char::is_ascii_lowercase);
                if prev.is_ascii_lowercase()
                    || prev.is_ascii_digit()
                    || (prev.is_ascii_uppercase() && next_is_lower)
                {
                    out.push('_');
                }
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    match out.as_str() {
        "as" | "async" | "await" | "box" | "break" | "const" | "continue" | "crate" | "dyn"
        | "else" | "enum" | "extern" | "false" | "final" | "fn" | "for" | "gen" | "if" | "impl"
        | "in" | "let" | "loop" | "macro" | "match" | "mod" | "move" | "mut" | "override"
        | "priv" | "pub" | "ref" | "return" | "static" | "struct" | "trait" | "true" | "try"
        | "type" | "typeof" | "unsafe" | "unsized" | "use" | "virtual" | "where" | "while"
        | "yield" => format!("r#{out}"),
        _ => out,
    }
}

const DOMAIN_PREFIX_RULES: &[(&str, &[&str], &[&str])] = &[
    ("preload", &[], &["preload_"]),
    (
        "achievement",
        &["fight_title", "handbook"],
        &["achievement", "collection", "manual_", "strategy_"],
    ),
    (
        "activity",
        &[
            "accrual_recharge",
            "appstorecard",
            "boss_challenge_detail",
            "levelupvoucher",
            "month_card",
        ],
        &[
            "activity_",
            "functionpreview",
            "growth_fund",
            "newbie_",
            "world_boss_",
        ],
    ),
    ("auction", &[], &["auction"]),
    (
        "chat",
        &["chat_moment", "mail_content", "p2p_buttonlist"],
        &["friend_gift_", "guild_", "system_chat", "team_"],
    ),
    ("contract", &[], &["contract_"]),
    ("drop", &[], &["drop_", "dropfilter_"]),
    (
        "equip",
        &[
            "ashesconfig",
            "candle_combine_cost",
            "comburent",
            "cost_forgel",
            "install_gold",
            "mercenary_equip_transform",
            "rarity_equip",
            "sort_ashesconfig",
        ],
        &[
            "corrosion_",
            "enchantment",
            "equip_",
            "memory_awaken_",
            "sift_",
        ],
    ),
    ("fashion", &[], &["fashion_"]),
    ("forge", &["cardrecipe"], &["forge_", "recipe_"]),
    ("gacha", &[], &["gacha"]),
    ("grow_up", &["gradenotice", "guide"], &["grow_up_"]),
    ("hero", &["bd_reference_tag"], &["hero_", "heroranking"]),
    (
        "item",
        &[
            "confirm_buy_item_text",
            "consum",
            "convert",
            "experience",
            "money",
            "reward",
            "specified_item",
        ],
        &["item_"],
    ),
    ("map", &[], &["map_"]),
    (
        "modifier",
        &["attribute_category", "attributes_show_cal", "function_des"],
        &["modifier_"],
    ),
    (
        "season",
        &["enum_gameplay_season_plague", "enum_season_tycoon"],
        &["season"],
    ),
    ("servant", &[], &["servant_"]),
    ("shop", &[], &["shop", "vendor"]),
    ("skill", &[], &["skill_"]),
    ("sp", &[], &["sp_"]),
    ("talent", &[], &["devil_block_", "prism_", "talent_"]),
];

fn classify_domain(stem: &str) -> &'static str {
    let s = strip_config_prefix(stem);
    if s.starts_with('s') && s.as_bytes().get(1).is_some_and(u8::is_ascii_digit) {
        return "season";
    }
    for &(domain, exact_list, prefix_list) in DOMAIN_PREFIX_RULES {
        if exact_list.contains(&s) || prefix_list.iter().any(|p| s.starts_with(p)) {
            return domain;
        }
    }
    "system"
}

fn infer_tuple_kv_names(table_short: &str, col_name: &str, arity: usize) -> (String, Vec<String>) {
    match (table_short, col_name, arity) {
        (_, "TargetAffixTier", 2) => (
            "tier".to_owned(),
            vec!["prob_type".to_owned(), "prob_per_myriad".to_owned()],
        ),
        (_, "Shape" | "Shape0", 2) => (
            "part_index".to_owned(),
            vec!["offset_x".to_owned(), "offset_y".to_owned()],
        ),
        (_, "ProfessionSpecifiedCost", 2) => (
            "profession_id".to_owned(),
            vec!["item_id".to_owned(), "count".to_owned()],
        ),
        (_, "DealSearchDefaultModifier", 2) => (
            "match_mode".to_owned(),
            vec!["affix_type".to_owned(), "min_value".to_owned()],
        ),
        ("cardrecipe", "RecipeUnique", 2) => (
            "recipe_id".to_owned(),
            vec!["target_id".to_owned(), "weight".to_owned()],
        ),
        (_, "AffixSlotNum", 3) => (
            "group".to_owned(),
            vec![
                "min_slots".to_owned(),
                "max_slots".to_owned(),
                "weight".to_owned(),
            ],
        ),
        (_, "ItemTypeList", 4) => (
            "mode".to_owned(),
            vec![
                "type1".to_owned(),
                "type2".to_owned(),
                "type3".to_owned(),
                "type4".to_owned(),
            ],
        ),
        (_, "ReshapeConsume" | "RotationConsume", 4) => (
            "step".to_owned(),
            vec![
                "min_count".to_owned(),
                "max_count".to_owned(),
                "item_id".to_owned(),
                "count".to_owned(),
            ],
        ),
        (_, _, n) => {
            let mut v_names = Vec::with_capacity(n);
            for idx in 1..=n {
                v_names.push(format!("param{idx}"));
            }
            ("key".to_owned(), v_names)
        }
    }
}

const SPECIFIC_KV_FIELD_NAMES: &[(&str, &str, &str, &str)] = &[
    (
        "dropfilter_sys_condition",
        "Placeholder",
        "index",
        "select_data_type",
    ),
    (
        "dropfilter_sys_condition",
        "SelectionValue",
        "index",
        "values",
    ),
    (
        "dropfilter_var_condition",
        "BaseModifierListArgs",
        "mode",
        "args",
    ),
    ("dropfilter_var_condition", "ModListArgs", "mode", "args"),
    (
        "dropfilter_var_condition",
        "TalentType",
        "talent_type",
        "sub_types",
    ),
    ("gacha", "PoolRarityWeight", "rarity", "weight"),
    ("gacha", "UpBanner", "reward_kind", "id"),
    ("gacha", "UpPossibilityInformation", "reward_kind", "id"),
    ("gacha_label", "ItemPreview", "reward_kind", "id"),
    ("gamepad_tips", "Content", "button_id", "text_key"),
    (
        "gameplay_duration",
        "DifficultySpRankMap",
        "difficulty",
        "sp_rank_ids",
    ),
    (
        "gameplay_duration",
        "PageTimeoutSecMap",
        "page_name",
        "timeout_sec",
    ),
    (
        "gradenotice",
        "NoticeCondition01",
        "condition_kind",
        "task_id",
    ),
    (
        "gradenotice",
        "NoticeCondition02",
        "condition_kind",
        "task_id",
    ),
    ("grow_up_label", "Unlock", "unlock_kind", "required_level"),
    (
        "item_target_function",
        "TargetFunction",
        "function_id",
        "enabled",
    ),
    ("item_tattoo", "UpModifier", "slot", "modifier_id"),
    (
        "map_gameplay_season_lost_talent_tree",
        "PreReq",
        "prev_node_id",
        "prev_node_level",
    ),
    (
        "memory_awaken_recipe_s14",
        "AwakenPart2",
        "boost_id",
        "probability",
    ),
    ("newbie_builds", "Content", "group_id", "content_ids"),
    ("p2p_buttonlist", "Priority", "button_id", "priority"),
    ("season", "AffiliatedSeasonId", "season_type", "season_id"),
    (
        "servant_subscription_group",
        "Servant",
        "servant_id",
        "star_level",
    ),
    ("shop", "RefreshRule", "rule_type", "args"),
    ("shop_form", "RefreshRule", "rule_type", "args"),
    ("shop", "Tab", "first_type", "second_type"),
    ("sift_equip", "Affix", "affix_id", "weight"),
    ("sift_equip", "AshesAffixType", "affix_id", "affix_type"),
    ("sift_equip", "Enchantment", "enchantment_id", "weight"),
    (
        "sp_area_v2",
        "TalentpointLimit",
        "area_level",
        "talent_point_limit",
    ),
    ("sp_card_v2", "Keyword", "keyword_id", "value"),
    ("sp_card_v2", "Modifiers", "modifier_id", "keywords"),
    (
        "sp_gameplayboss_combine",
        "Combine",
        "combine_key",
        "boss_id",
    ),
    ("sp_tree_node", "FunctionModifier", "modifier_id", "params"),
    ("sp_tree_node", "MapModifier", "modifier_id", "params"),
    (
        "team_td_challenge",
        "FirstAward",
        "round_id",
        "reward_group_id",
    ),
    ("cardrecipe", "ItemGoldId", "item_gold_id", "weight"),
    ("cardrecipe", "RewardGroupId", "reward_group_id", "weight"),
    ("cardrecipe", "RecipeRandomWeight", "group_id", "weight"),
    ("cardrecipe", "SkillType", "skill_type", "weight"),
];

fn infer_kv_field_names(
    table_stem: &str,
    col_name: &str,
    key_ty: &InferredType,
    val_ty: &InferredType,
    tuple_elems: Option<&[InferredType]>,
) -> (String, Vec<String>) {
    let table_short = strip_config_prefix(table_stem);
    if let Some(elems) = tuple_elems {
        return infer_tuple_kv_names(table_short, col_name, elems.len());
    }
    for &(t, c, k, v) in SPECIFIC_KV_FIELD_NAMES {
        if t == table_short && c == col_name {
            return (k.to_owned(), vec![v.to_owned()]);
        }
    }
    let is_list_val = matches!(val_ty, InferredType::List { .. });
    let pair: (&str, &str) = match col_name {
        col if col.ends_with("Position") || col.ends_with("Pos") => ("x", "y"),
        "DefaultLanguage" | "LineNum" => ("min", "max"),
        col if col.ends_with("Range") => ("min", "max"),
        "EnableCost" => ("cost_key", "count"),
        "UninstallCost" => ("item_id", "counts"),
        col if col.ends_with("Consume") || col.ends_with("Cost") || col.starts_with("CostItem") => {
            if matches!(key_ty, InferredType::Str) {
                ("cost_key", "count")
            } else {
                ("item_id", "count")
            }
        }
        "UnlockReq" => ("area_id", "area_level"),
        "CostWeight" => ("level_range", "weight"),
        col if col.ends_with("Tier") => ("tier", "weight"),
        col if col.ends_with("Num") => ("count", "weight"),
        "InfluenceShape" => ("shape_id", "weight"),
        "TalentRule" => ("rule_id", "weight"),
        "SlotRecipe" => ("recipe_id", "weight"),
        col if col.ends_with("Modifier") => {
            if is_list_val {
                ("modifier_id", "params")
            } else {
                ("modifier_id", "value")
            }
        }
        "Function" | "Event" | "Param" | "Parameter" | "Effect" | "Group" => {
            if is_list_val {
                ("kind", "params")
            } else {
                ("kind", "value")
            }
        }
        _ => {
            let col_snake = to_snake_case(col_name);
            return (
                format!("{col_snake}_key"),
                vec![format!("{col_snake}_value")],
            );
        }
    };
    (pair.0.to_owned(), vec![pair.1.to_owned()])
}

struct Cursor<'a> {
    buf: &'a [u8],
    pos: u32,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, len: u32) -> Result<&'a [u8], GenError> {
        let start = u32_to_usize(self.pos);
        let end = start
            .checked_add(u32_to_usize(len))
            .ok_or(GenError::MalformedBytecode {
                context: "cursor overflow",
            })?;
        let slice = self
            .buf
            .get(start..end)
            .ok_or(GenError::MalformedBytecode {
                context: "cursor eof",
            })?;
        self.pos += len;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8, GenError> {
        Ok(self.take(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32, GenError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_i64(&mut self) -> Result<i64, GenError> {
        let b = self.take(8)?;
        Ok(i64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn read_xdi_string(&mut self) -> Result<Option<String>, GenError> {
        let tag = self.read_u8()?;
        if tag == 0 {
            return Ok(None);
        }
        let byte_len = if tag < 0xFF {
            u32::from(tag).checked_sub(10)
        } else {
            self.read_u32()?.checked_sub(10)
        }
        .ok_or(GenError::MalformedBytecode {
            context: "xdi string length < 10",
        })?;
        let bytes = self.take(byte_len)?;
        Ok(Some(core::str::from_utf8(bytes)?.to_owned()))
    }
}

fn step_gen_instruction(
    code: &[u32],
    consts: &[Val],
    regs: &mut [Val; 256],
    tables: &mut Vec<TableObj>,
    pc: &mut u32,
) -> Result<bool, GenError> {
    let w = code[u32_to_usize(*pc)];
    let std_op = XDI_TO_LUA53_OPCODE[u32_to_usize(w >> 26)];
    let a = u32_to_usize(w & 0xFF);
    let c = u32_to_usize((w >> 8) & 0x1FF);
    let b = u32_to_usize((w >> 17) & 0x1FF);
    let bx = u32_to_usize((w >> 8) & 0x3FFFF);
    match std_op {
        11 => {
            let tid = u32::try_from(tables.len()).map_err(|_| GenError::MalformedBytecode {
                context: "table count overflow",
            })?;
            tables.push(TableObj::default());
            regs[a] = Val::Table(tid);
        }
        1 => regs[a] = consts[bx].clone(),
        2 => {
            let next_w = code[u32_to_usize(*pc + 1)];
            let ax = u32_to_usize(next_w & 0x03FF_FFFF);
            regs[a] = consts[ax].clone();
            *pc += 1;
        }
        3 => {
            regs[a] = Val::Bool;
            if c != 0 {
                *pc += 1;
            }
        }
        0 => regs[a] = regs[b].clone(),
        43 => {
            let count = if b == 0 { 50 } else { b };
            let Val::Table(tid) = regs[a] else {
                return Err(GenError::MalformedBytecode {
                    context: "setlist on non-table",
                });
            };
            for i in 1..=count {
                let v = regs[a + i].clone();
                tables[u32_to_usize(tid)].list.push(v);
            }
            if c == 0 {
                *pc += 1;
            }
        }
        10 => {
            let k = if (b & 0x100) != 0 {
                consts[b & 0xFF].clone()
            } else {
                regs[b].clone()
            };
            let v = if (c & 0x100) != 0 {
                consts[c & 0xFF].clone()
            } else {
                regs[c].clone()
            };
            let Val::Table(tid) = regs[a] else {
                return Err(GenError::MalformedBytecode {
                    context: "settable on non-table",
                });
            };
            tables[u32_to_usize(tid)].map.push((k, v));
        }
        36 | 38 => return Ok(false),
        _ => {}
    }
    *pc += 1;
    Ok(true)
}

fn eval_root_slice(
    code: &[u32],
    consts: &[Val],
    regs: &mut [Val; 256],
    tables: &mut Vec<TableObj>,
) -> Result<(), GenError> {
    let code_len = u32::try_from(code.len()).map_err(|_| GenError::MalformedBytecode {
        context: "code length overflow",
    })?;
    let mut pc = 0_u32;
    for _ in 0..code_len {
        if pc >= code_len || !step_gen_instruction(code, consts, regs, tables, &mut pc)? {
            break;
        }
    }
    Ok(())
}

fn parse_root_proto(cur: &mut Cursor<'_>) -> Result<(Vec<u32>, Vec<Val>), GenError> {
    let _src = cur.read_xdi_string()?;
    let _meta = cur.take(11)?;
    let size_code = cur.read_u32()?;
    let mut code = Vec::with_capacity(u32_to_usize(size_code));
    for _ in 0..size_code {
        code.push(cur.read_u32()?);
    }
    let size_k = cur.read_u32()?;
    let mut consts = Vec::with_capacity(u32_to_usize(size_k));
    for _ in 0..size_k {
        let tag = cur.read_u8()?;
        let v = match tag {
            0x00 => Val::Nil,
            0x01 => {
                let _b = cur.read_u8()?;
                Val::Bool
            }
            0x03 => {
                let _f = cur.take(8)?;
                Val::Float
            }
            0x13 => Val::Int(cur.read_i64()?),
            0x04 | 0x14 => Val::Str(cur.read_xdi_string()?.ok_or(GenError::MalformedBytecode {
                context: "null string constant",
            })?),
            _ => {
                return Err(GenError::MalformedBytecode {
                    context: "invalid constant tag",
                });
            }
        };
        consts.push(v);
    }
    Ok((code, consts))
}

fn scalar_or_compound_rust_type(
    ty: &InferredType,
    helper_struct_name: Option<&str>,
) -> Result<String, GenError> {
    match ty {
        InferredType::EmptyTable => Ok("[u8; 0]".to_owned()),
        InferredType::Bool => Ok("bool".to_owned()),
        InferredType::Int(b) => Ok(b.rust_type().to_owned()),
        InferredType::Float => Ok("f64".to_owned()),
        InferredType::Str => Ok("Box<str>".to_owned()),
        InferredType::List { elem, .. } => Ok(format!(
            "Box<[{}]>",
            scalar_or_compound_rust_type(elem, helper_struct_name)?
        )),
        InferredType::KvRecord { .. } | InferredType::StringRecord(_) => helper_struct_name
            .map(str::to_owned)
            .ok_or(GenError::MalformedBytecode {
                context: "missing helper struct name",
            }),
        InferredType::MapTable(k, v) => Ok(format!(
            "FxHashMap<{}, {}>",
            scalar_or_compound_rust_type(k, None)?,
            scalar_or_compound_rust_type(v, helper_struct_name)?
        )),
        InferredType::IntOrList(b) => Ok(format!("IntOrIntList<{}>", b.rust_type())),
    }
}

struct LuaTableSpec {
    stem: String,
    pascal: String,
    key_rust: String,
    is_3d: bool,
    cols: BTreeMap<String, InferredType>,
}

enum PreloadKind {
    List(String),
    Map1D(String, String),
    Map2D(String, String, String),
}

struct PreloadSpec {
    stem: String,
    pascal: String,
    kind: PreloadKind,
    uses_int_or_list: bool,
}

struct HelperHeader<'a> {
    stem: &'a str,
    col: &'a str,
    struct_name: &'a str,
    derive_line: &'a str,
}

fn emit_kv_helper_struct(
    code: &mut String,
    hdr: &HelperHeader<'_>,
    key: &InferredType,
    val: &InferredType,
    tuple_elems: Option<&[InferredType]>,
) -> Result<(), GenError> {
    let (k_name, v_names) = infer_kv_field_names(hdr.stem, hdr.col, key, val, tuple_elems);
    let k_rust = scalar_or_compound_rust_type(key, None)?;
    code.push_str(&format!(
        "/// Entry for `{}.{}`.\n{}\npub struct {} {{\n    pub {k_name}: {k_rust},\n",
        hdr.stem, hdr.col, hdr.derive_line, hdr.struct_name
    ));
    if let Some(elems) = tuple_elems {
        assert_eq!(elems.len(), v_names.len());
        for (ety, vname) in elems.iter().zip(&v_names) {
            let ety_rust = scalar_or_compound_rust_type(ety, None)?;
            code.push_str(&format!("    pub {vname}: {ety_rust},\n"));
        }
    } else {
        let v_rust = scalar_or_compound_rust_type(val, None)?;
        code.push_str(&format!("    pub {}: {v_rust},\n", v_names[0]));
    }
    code.push_str("}\n\n");
    code.push_str(&format!(
        "impl FromEvalVal for {} {{\n    #[inline]\n    fn from_eval_val(\n        ctx: &EvalContext<'_, '_>,\n        val: EvalVal,\n        table: &'static str,\n        field: &'static str,\n    ) -> Result<Self, LuacError> {{\n",
        hdr.struct_name
    ));
    if tuple_elems.is_some() {
        let tuple_pat = v_names.join(", ");
        code.push_str(&format!(
            "        let ({k_name}, ({tuple_pat})) = ctx.decode_kv_pair(val, table, field)?;\n        Ok(Self {{\n            {k_name},\n"
        ));
        for vname in &v_names {
            code.push_str(&format!("            {vname},\n"));
        }
        code.push_str("        })\n");
    } else {
        let vname = &v_names[0];
        code.push_str(&format!(
            "        let ({k_name}, {vname}) = ctx.decode_kv_pair(val, table, field)?;\n        Ok(Self {{ {k_name}, {vname} }})\n"
        ));
    }
    code.push_str("    }\n}\n\n");
    Ok(())
}

fn emit_string_helper_struct(
    code: &mut String,
    hdr: &HelperHeader<'_>,
    m: &BTreeMap<String, InferredType>,
) -> Result<(), GenError> {
    code.push_str(&format!(
        "/// Sub-record for `{}.{}`.\n{}\npub struct {} {{\n",
        hdr.stem, hdr.col, hdr.derive_line, hdr.struct_name
    ));
    for (k, v) in m {
        let f_snake = to_snake_case(k);
        let v_rust = scalar_or_compound_rust_type(v, None)?;
        code.push_str(&format!("    pub {f_snake}: {v_rust},\n"));
    }
    code.push_str("}\n\n");
    code.push_str(&format!(
        "impl FromEvalVal for {} {{\n    fn from_eval_val(\n        ctx: &EvalContext<'_, '_>,\n        val: EvalVal,\n        table: &'static str,\n        field: &'static str,\n    ) -> Result<Self, LuacError> {{\n        let mut out = Self::default();\n        for &(k, v) in ctx.table_map(val, table, field)? {{\n            match ctx.decode_str_key(k, table, field)? {{\n",
        hdr.struct_name
    ));
    for k in m.keys() {
        let f_snake = to_snake_case(k);
        code.push_str(&format!(
            "                b{k:?} => out.{f_snake} = ctx.decode_field(v, table, field)?,\n"
        ));
    }
    code.push_str(
        "                other => return Err(ctx.unknown_field(table, other)),\n            }\n        }\n        Ok(out)\n    }\n}\n\n",
    );
    Ok(())
}

fn emit_helper_struct(
    code: &mut String,
    stem: &str,
    col: &str,
    struct_name: &str,
    rec_ty: &InferredType,
) -> Result<(), GenError> {
    let mut traits = vec!["Debug", "Clone"];
    if rec_ty.is_copy() {
        traits.push("Copy");
    }
    traits.push("PartialEq");
    if !rec_ty.has_float() {
        traits.push("Eq");
    }
    traits.push("Default");
    let derive_line = format!("#[derive({})]", traits.join(", "));
    let hdr = HelperHeader {
        stem,
        col,
        struct_name,
        derive_line: &derive_line,
    };
    match rec_ty {
        InferredType::KvRecord {
            key,
            val,
            tuple_elems,
        } => emit_kv_helper_struct(code, &hdr, key, val, tuple_elems.as_deref()),
        InferredType::StringRecord(m) => emit_string_helper_struct(code, &hdr, m),
        InferredType::EmptyTable
        | InferredType::Bool
        | InferredType::Int(_)
        | InferredType::Float
        | InferredType::Str
        | InferredType::List { .. }
        | InferredType::MapTable(_, _)
        | InferredType::IntOrList(_) => Err(GenError::MalformedBytecode {
            context: "expected record type",
        }),
    }
}

fn merge_opt_type(slot: &mut Option<InferredType>, next: InferredType) -> Result<(), GenError> {
    *slot = Some(match slot.take() {
        Some(prev) => InferredType::merge(prev, next)?,
        None => next,
    });
    Ok(())
}

fn collect_3d_keys_and_rows(
    main_tid: u32,
    tables: &[TableObj],
    row_tids: &mut Vec<u32>,
) -> Result<String, GenError> {
    let mut k1_ty = None;
    let mut k2_ty = None;
    let mut k3_ty = None;
    for (k1, v1) in &tables[u32_to_usize(main_tid)].map {
        merge_opt_type(&mut k1_ty, infer_val(k1, tables)?)?;
        let Val::Table(tid1) = v1 else {
            return Err(GenError::MalformedBytecode { context: "3d l1" });
        };
        for (k2, v2) in &tables[u32_to_usize(*tid1)].map {
            merge_opt_type(&mut k2_ty, infer_val(k2, tables)?)?;
            let Val::Table(tid2) = v2 else {
                return Err(GenError::MalformedBytecode { context: "3d l2" });
            };
            for (k3, v3) in &tables[u32_to_usize(*tid2)].map {
                merge_opt_type(&mut k3_ty, infer_val(k3, tables)?)?;
                if let Val::Table(rtid) = v3 {
                    row_tids.push(*rtid);
                }
            }
        }
    }
    let t1 = k1_ty.ok_or(GenError::MalformedBytecode { context: "k1" })?;
    let t2 = k2_ty.ok_or(GenError::MalformedBytecode { context: "k2" })?;
    let t3 = k3_ty.ok_or(GenError::MalformedBytecode { context: "k3" })?;
    Ok(format!(
        "({}, {}, {})",
        scalar_or_compound_rust_type(&t1, None)?,
        scalar_or_compound_rust_type(&t2, None)?,
        scalar_or_compound_rust_type(&t3, None)?,
    ))
}

fn analyze_luatable(stem: String, buf: &mut [u8]) -> Result<LuaTableSpec, GenError> {
    let pascal = to_pascal_case(strip_config_prefix(&stem));
    decrypt_in_place(buf);
    let mut cur = Cursor { buf, pos: 10 };
    let (code, consts) = parse_root_proto(&mut cur)?;
    let mut regs: [Val; 256] = std::array::from_fn(|_| Val::Nil);
    let mut tables = Vec::new();
    eval_root_slice(&code, &consts, &mut regs, &mut tables)?;

    let (Val::Table(main_tid), Val::Table(most_tid), Val::Table(def_tid)) =
        (&regs[0], &regs[1], &regs[2])
    else {
        return Err(GenError::MalformedBytecode {
            context: "missing R[0..=2] tables",
        });
    };
    let (main_tid, most_tid, def_tid) = (*main_tid, *most_tid, *def_tid);

    let is_3d = tables[u32_to_usize(main_tid)]
        .map
        .first()
        .is_some_and(|(_, v1)| {
            matches!(v1, Val::Table(t1) if tables[u32_to_usize(*t1)].map.first().is_some_and(|(k2, v2)| {
                matches!(k2, Val::Int(_)) && matches!(v2, Val::Table(t2) if tables[u32_to_usize(*t2)].map.first().is_some_and(|(k3, _)| matches!(k3, Val::Int(_))))
            }))
        });

    let mut row_tids = vec![most_tid, def_tid];
    let key_rust = if is_3d {
        collect_3d_keys_and_rows(main_tid, &tables, &mut row_tids)?
    } else {
        let mut k_ty = None;
        for (k, v) in &tables[u32_to_usize(main_tid)].map {
            merge_opt_type(&mut k_ty, infer_val(k, &tables)?)?;
            if let Val::Table(rtid) = v {
                row_tids.push(*rtid);
            }
        }
        match k_ty {
            Some(kt) => scalar_or_compound_rust_type(&kt, None)?,
            None => "u32".to_owned(),
        }
    };

    let mut cols: BTreeMap<String, InferredType> = BTreeMap::new();
    for rtid in row_tids {
        for (k, v) in &tables[u32_to_usize(rtid)].map {
            let Val::Str(col_name) = k else { continue };
            let inferred = infer_val(v, &tables)?;
            if let Some(existing) = cols.remove(col_name) {
                cols.insert(col_name.clone(), InferredType::merge(existing, inferred)?);
            } else {
                cols.insert(col_name.clone(), inferred);
            }
        }
    }
    Ok(LuaTableSpec {
        stem,
        pascal,
        key_rust,
        is_3d,
        cols,
    })
}

fn infer_preload_2d(
    root_tbl: &TableObj,
    tables: &[TableObj],
) -> Result<(PreloadKind, bool), GenError> {
    let mut k1_ty = None;
    let mut k2_ty = None;
    let mut v_ty = None;
    for (k1, v1) in &root_tbl.map {
        merge_opt_type(&mut k1_ty, infer_val(k1, tables)?)?;
        let Val::Table(sub_tid) = v1 else {
            return Err(GenError::MalformedBytecode { context: "2d sub" });
        };
        for (k2, v2) in &tables[u32_to_usize(*sub_tid)].map {
            merge_opt_type(&mut k2_ty, infer_val(k2, tables)?)?;
            merge_opt_type(&mut v_ty, infer_val(v2, tables)?)?;
        }
    }
    let vt = v_ty.ok_or(GenError::MalformedBytecode { context: "2d val" })?;
    let uses_int_or_list = vt.uses_int_or_list();
    let kind = PreloadKind::Map2D(
        scalar_or_compound_rust_type(
            &k1_ty.ok_or(GenError::MalformedBytecode { context: "2d k1" })?,
            None,
        )?,
        scalar_or_compound_rust_type(
            &k2_ty.ok_or(GenError::MalformedBytecode { context: "2d k2" })?,
            None,
        )?,
        scalar_or_compound_rust_type(&vt, None)?,
    );
    Ok((kind, uses_int_or_list))
}

fn analyze_preload(stem: String, buf: &mut [u8]) -> Result<PreloadSpec, GenError> {
    let pascal = to_pascal_case(&stem);
    decrypt_in_place(buf);
    let mut cur = Cursor { buf, pos: 10 };
    let (code, consts) = parse_root_proto(&mut cur)?;
    let mut regs: [Val; 256] = std::array::from_fn(|_| Val::Nil);
    let mut tables = Vec::new();
    eval_root_slice(&code, &consts, &mut regs, &mut tables)?;
    let Val::Table(main_tid) = regs[0] else {
        return Err(GenError::MalformedBytecode {
            context: "missing preload R[0]",
        });
    };
    let root_tbl = &tables[u32_to_usize(main_tid)];
    let (kind, uses_int_or_list) = if !root_tbl.list.is_empty() {
        let mut elem = infer_val(&root_tbl.list[0], &tables)?;
        for item in &root_tbl.list[1..] {
            elem = InferredType::merge(elem, infer_val(item, &tables)?)?;
        }
        (
            PreloadKind::List(scalar_or_compound_rust_type(&elem, None)?),
            false,
        )
    } else if root_tbl.map.first().is_some_and(
        |(_, v)| matches!(v, Val::Table(tid) if !tables[u32_to_usize(*tid)].map.is_empty()),
    ) {
        infer_preload_2d(root_tbl, &tables)?
    } else {
        let mut k_ty = None;
        let mut v_ty = None;
        for (k, v) in &root_tbl.map {
            merge_opt_type(&mut k_ty, infer_val(k, &tables)?)?;
            merge_opt_type(&mut v_ty, infer_val(v, &tables)?)?;
        }
        let vt = v_ty.ok_or(GenError::MalformedBytecode { context: "1d val" })?;
        let uses = vt.uses_int_or_list();
        (
            PreloadKind::Map1D(
                scalar_or_compound_rust_type(
                    &k_ty.ok_or(GenError::MalformedBytecode { context: "1d key" })?,
                    None,
                )?,
                scalar_or_compound_rust_type(&vt, None)?,
            ),
            uses,
        )
    };
    Ok(PreloadSpec {
        stem,
        pascal,
        kind,
        uses_int_or_list,
    })
}

fn write_file_atomic(path: &Path, content: &str) -> Result<(), GenError> {
    let tmp_path = path.with_extension("rs.tmp");
    let mut file = File::create(&tmp_path)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn emit_row_impl(code: &mut String, row_name: &str, spec: &LuaTableSpec) {
    let col_names: Vec<&String> = spec.cols.keys().collect();
    if col_names.len() > 12 {
        code.push_str(&format!("impl {row_name} {{\n"));
        for (idx, chunk) in col_names.chunks(12).enumerate() {
            code.push_str(&format!(
                "    fn set_field_{idx}(\n        &mut self,\n        ctx: &EvalContext<'_, '_>,\n        field: &[u8],\n        val: EvalVal,\n    ) -> Result<bool, LuacError> {{\n        match field {{\n"
            ));
            for col in chunk {
                let field_snake = to_snake_case(col);
                code.push_str(&format!(
                    "            b{col:?} => self.{field_snake} = ctx.decode_field(val, Self::TABLE_NAME, {col:?})?,\n"
                ));
            }
            code.push_str(
                "            _ => return Ok(false),\n        }\n        Ok(true)\n    }\n\n",
            );
        }
        code.push_str("}\n\n");
    }

    code.push_str(&format!(
        "impl DataCfgRow for {row_name} {{\n    const TABLE_NAME: &'static str = {:?};\n\n    fn set_field(\n        &mut self,\n        ctx: &EvalContext<'_, '_>,\n        field: &[u8],\n        val: EvalVal,\n    ) -> Result<(), LuacError> {{\n",
        spec.stem
    ));
    if col_names.len() > 12 {
        let count = col_names.chunks(12).len();
        let checks: Vec<String> = (0..count)
            .map(|idx| format!("self.set_field_{idx}(ctx, field, val)?"))
            .collect();
        code.push_str(&format!(
            "        if {} {{\n            return Ok(());\n        }}\n        Err(ctx.unknown_field(Self::TABLE_NAME, field))\n    }}\n}}\n\n",
            checks.join("\n            || ")
        ));
    } else {
        code.push_str("        match field {\n");
        for col in &col_names {
            let field_snake = to_snake_case(col);
            code.push_str(&format!(
                "            b{col:?} => self.{field_snake} = ctx.decode_field(val, Self::TABLE_NAME, {col:?})?,\n"
            ));
        }
        code.push_str(
            "            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),\n        }\n        Ok(())\n    }\n}\n\n",
        );
    }
}

fn emit_table_spec(code: &mut String, spec: &LuaTableSpec) -> Result<(), GenError> {
    let mut col_helper_names: BTreeMap<String, String> = BTreeMap::new();
    for (col, ty) in &spec.cols {
        if let Some((rec_ty, is_collection)) = find_nested_record(ty) {
            let col_pascal = to_pascal_case(col);
            let struct_name = if is_collection {
                format!("{}{col_pascal}Entry", spec.pascal)
            } else {
                format!("{}{col_pascal}", spec.pascal)
            };
            emit_helper_struct(code, &spec.stem, col, &struct_name, rec_ty)?;
            col_helper_names.insert(col.clone(), struct_name);
        }
    }

    let row_name = format!("Config{}Row", spec.pascal);
    let table_alias = format!("Config{}Table", spec.pascal);
    let row_derive = if spec.cols.values().any(InferredType::has_float) {
        "#[derive(Debug, Clone, PartialEq, Default)]"
    } else {
        "#[derive(Debug, Clone, PartialEq, Eq, Default)]"
    };
    code.push_str(&format!(
        "/// Row record for `{}.out`.\n{row_derive}\npub struct {row_name} {{\n",
        spec.stem
    ));
    for (col, ty) in &spec.cols {
        let field_snake = to_snake_case(col);
        let helper_name = col_helper_names.get(col).map(String::as_str);
        let field_rust = scalar_or_compound_rust_type(ty, helper_name)?;
        code.push_str(&format!("    pub {field_snake}: {field_rust},\n"));
    }
    code.push_str("}\n\n");

    emit_row_impl(code, &row_name, spec);

    let parse_fn = if spec.is_3d {
        "parse_luatable_datacfg_3d"
    } else {
        "parse_luatable_datacfg"
    };
    code.push_str(&format!(
        "/// Typed `DataTable` for `{0}.out`.\npub type {table_alias} = DataTable<{1}, {row_name}>;\n\n/// Parses `{0}.out` into a [`{table_alias}`].\npub fn parse_{0}(\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Result<{table_alias}, LuacError> {{\n    {parse_fn}(buf, scratch)\n}}\n\n",
        spec.stem, spec.key_rust
    ));
    Ok(())
}

fn emit_domain_verifier(code: &mut String, domain: &str, specs: &[LuaTableSpec]) {
    if specs.len() > 14 {
        for (idx, chunk) in specs.chunks(14).enumerate() {
            code.push_str(&format!(
                "fn verify_{domain}_chunk_{idx}(\n    stem: &str,\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Option<Result<u32, LuacError>> {{\n    match stem {{\n"
            ));
            for spec in chunk {
                code.push_str(&format!(
                    "        {0:?} => Some(parse_{0}(buf, scratch).map(|t| t.len())),\n",
                    spec.stem
                ));
            }
            code.push_str("        _ => None,\n    }\n}\n\n");
        }
        code.push_str(&format!(
            "pub(crate) fn verify_{domain}_datacfg_by_stem(\n    stem: &str,\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Option<Result<u32, LuacError>> {{\n"
        ));
        let count = specs.chunks(14).len();
        for idx in 0..count {
            code.push_str(&format!(
                "    if let Some(res) = verify_{domain}_chunk_{idx}(stem, buf, scratch) {{\n        return Some(res);\n    }}\n"
            ));
        }
        code.push_str("    None\n}\n");
    } else {
        code.push_str(&format!(
            "pub(crate) fn verify_{domain}_datacfg_by_stem(\n    stem: &str,\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Option<Result<u32, LuacError>> {{\n    match stem {{\n"
        ));
        for spec in specs {
            code.push_str(&format!(
                "        {0:?} => Some(parse_{0}(buf, scratch).map(|t| t.len())),\n",
                spec.stem
            ));
        }
        code.push_str("        _ => None,\n    }\n}\n");
    }
}

fn emit_domain_module(
    out_dir: &Path,
    domain: &str,
    specs: &[LuaTableSpec],
) -> Result<(), GenError> {
    let mut code = format!(
        "//! Auto-generated `LuaTable/DataCfg` types and parsers for the `{domain}` domain.\n\n"
    );
    let any_3d = specs.iter().any(|s| s.is_3d);
    let any_helper = specs
        .iter()
        .any(|s| s.cols.values().any(|t| find_nested_record(t).is_some()));
    let any_fx_map = specs
        .iter()
        .any(|s| s.cols.values().any(InferredType::uses_fx_hash_map));
    let any_int_or_list = specs
        .iter()
        .any(|s| s.cols.values().any(InferredType::uses_int_or_list));

    let mut datacfg_imports = vec![
        "DataCfgRow",
        "DataCfgScratch",
        "DataTable",
        "EvalContext",
        "EvalVal",
    ];
    if any_helper {
        datacfg_imports.push("FromEvalVal");
    }
    datacfg_imports.push("parse_luatable_datacfg");
    if any_3d {
        datacfg_imports.push("parse_luatable_datacfg_3d");
    }
    code.push_str(&format!(
        "use crate::luac::datacfg::{{{}}};\nuse crate::luac::error::LuacError;\n",
        datacfg_imports.join(", ")
    ));
    if any_int_or_list {
        code.push_str("use crate::luac::types::IntOrIntList;\n");
    }
    if any_fx_map {
        code.push_str("use rustc_hash::FxHashMap;\n");
    }
    code.push('\n');

    for spec in specs {
        emit_table_spec(&mut code, spec)?;
    }

    emit_domain_verifier(&mut code, domain, specs);
    write_file_atomic(&out_dir.join(format!("{domain}.rs")), &code)
}

fn emit_preload_module(out_dir: &Path, preload_specs: &[PreloadSpec]) -> Result<(), GenError> {
    let mut code = String::from(
        "//! Auto-generated `LuaPreLoadTable/DataCfg` preload table types and parsers.\n\nuse crate::bounded_len_u32;\nuse crate::luac::datacfg::{\n    DataCfgScratch, PreloadTable, parse_preload_list, parse_preload_map, parse_preload_map_2d,\n};\nuse crate::luac::error::LuacError;\n",
    );
    if preload_specs.iter().any(|s| s.uses_int_or_list) {
        code.push_str("use crate::luac::types::IntOrIntList;\n");
    }
    code.push('\n');

    for p in preload_specs {
        let alias = format!("{}Table", p.pascal);
        match &p.kind {
            PreloadKind::List(elem_ty) => {
                code.push_str(&format!(
                    "/// Typed preload list for `{0}.out`.\npub type {alias} = Box<[{elem_ty}]>;\n\n/// Parses `{0}.out` into a [`{alias}`].\npub fn parse_{0}(\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Result<{alias}, LuacError> {{\n    parse_preload_list(buf, scratch, {0:?})\n}}\n\n",
                    p.stem
                ));
            }
            PreloadKind::Map1D(k_ty, v_ty) => {
                code.push_str(&format!(
                    "/// Typed preload table for `{0}.out`.\npub type {alias} = PreloadTable<{k_ty}, {v_ty}>;\n\n/// Parses `{0}.out` into a [`{alias}`].\npub fn parse_{0}(\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Result<{alias}, LuacError> {{\n    parse_preload_map(buf, scratch, {0:?})\n}}\n\n",
                    p.stem
                ));
            }
            PreloadKind::Map2D(k1_ty, k2_ty, v_ty) => {
                code.push_str(&format!(
                    "/// Typed 2D preload table for `{0}.out`.\npub type {alias} = PreloadTable<({k1_ty}, {k2_ty}), {v_ty}>;\n\n/// Parses `{0}.out` into a [`{alias}`].\npub fn parse_{0}(\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Result<{alias}, LuacError> {{\n    parse_preload_map_2d(buf, scratch, {0:?})\n}}\n\n",
                    p.stem
                ));
            }
        }
    }

    code.push_str(
        "pub(crate) fn verify_preload_datacfg_by_stem(\n    stem: &str,\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Option<Result<u32, LuacError>> {\n    match stem {\n",
    );
    for p in preload_specs {
        if matches!(p.kind, PreloadKind::List(_)) {
            code.push_str(&format!(
                "        {0:?} => Some(parse_{0}(buf, scratch).map(|t| bounded_len_u32(t.len()))),\n",
                p.stem
            ));
        } else {
            code.push_str(&format!(
                "        {0:?} => Some(parse_{0}(buf, scratch).map(|t| t.len())),\n",
                p.stem
            ));
        }
    }
    code.push_str("        _ => None,\n    }\n}\n");
    write_file_atomic(&out_dir.join("preload.rs"), &code)
}

fn emit_mod_rs(out_dir: &Path, all_modules: &BTreeSet<&'static str>) -> Result<(), GenError> {
    let mut mod_code = String::from(
        "//! Strongly-typed `DataCfg` and `LuaPreLoadTable` parsers grouped into 23 domain modules.\n\n",
    );
    for m in all_modules {
        mod_code.push_str(&format!("pub mod {m};\n"));
    }
    mod_code.push('\n');
    for m in all_modules {
        mod_code.push_str(&format!("pub use {m}::*;\n"));
    }
    mod_code.push_str(&format!(
        "\nuse crate::luac::datacfg::DataCfgScratch;\nuse crate::luac::error::{{DataCfgError, LuacError}};\n\ntype DomainVerifier = fn(&str, &mut [u8], &mut DataCfgScratch) -> Option<Result<u32, LuacError>>;\n\nconst DOMAIN_VERIFIERS: [DomainVerifier; {}] = [\n",
        all_modules.len()
    ));
    for m in all_modules {
        mod_code.push_str(&format!("    {m}::verify_{m}_datacfg_by_stem,\n"));
    }
    mod_code.push_str(
        "];\n\n/// Evaluates a `DataCfg` `.out` buffer by file stem (`config_*`) and returns its row/entry count.\npub fn verify_all_datacfg_by_stem(\n    stem: &str,\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Result<u32, LuacError> {\n    for verifier in DOMAIN_VERIFIERS {\n        if let Some(res) = verifier(stem, buf, scratch) {\n            return res;\n        }\n    }\n    Err(DataCfgError::UnknownField {\n        table: \"<datacfg_corpus>\",\n        field: stem.into(),\n    }\n    .into())\n}\n",
    );
    write_file_atomic(&out_dir.join("mod.rs"), &mod_code)
}

type FileEntryList = Vec<(String, FileId)>;

fn collect_datacfg_files(vfs: &PakVfs) -> Result<(FileEntryList, FileEntryList), GenError> {
    let mut path_buf = Vec::new();
    let mut luatable_files = Vec::new();
    let mut preload_files = Vec::new();
    for fid in vfs.iter_files() {
        path_buf.clear();
        vfs.write_file_path(fid, &mut path_buf);
        let s = core::str::from_utf8(&path_buf)?.to_owned();
        if s.starts_with("UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_")
            && s.ends_with(".out")
        {
            luatable_files.push((s, fid));
        } else if s.starts_with(
            "UE_game/Content/Script/Luac/Config/LuaPreLoadTable/DataCfg/config_preload_",
        ) && s.ends_with(".out")
        {
            preload_files.push((s, fid));
        }
    }
    luatable_files.sort_by(|a, b| a.0.cmp(&b.0));
    preload_files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok((luatable_files, preload_files))
}

fn read_vfs_entry(
    vfs: &PakVfs,
    path: &str,
    fid: FileId,
    scratch: &mut ReadScratch,
    buf: &mut Vec<u8>,
) -> Result<String, GenError> {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or(GenError::MalformedBytecode { context: "stem" })?
        .to_owned();
    let len = usize::try_from(vfs.file_meta(fid).uncompressed_bytes()).map_err(|_| {
        GenError::MalformedBytecode {
            context: "file size",
        }
    })?;
    buf.resize(len, 0);
    vfs.read_into(fid, scratch, buf)?;
    Ok(stem)
}

fn run_codegen() -> Result<(), GenError> {
    let game_dir = std::env::var("TLI_GAME_DIR")?;
    let key_hex = std::env::var("TLI_AES_KEY")?;
    let key = AesKey::try_from_hex(key_hex.trim())?;
    let vfs = PakVfs::open_tli_game_dir(Path::new(&game_dir), Some(key))?;
    let mut scratch = ReadScratch::with_limits(ScratchLimits::DEFAULT);
    let mut buf = Vec::new();
    let (luatable_files, preload_files) = collect_datacfg_files(&vfs)?;

    let mut domains: BTreeMap<&'static str, Vec<LuaTableSpec>> = BTreeMap::new();
    for &(ref path, fid) in &luatable_files {
        let stem = read_vfs_entry(&vfs, path, fid, &mut scratch, &mut buf)?;
        let domain = classify_domain(&stem);
        let spec = analyze_luatable(stem, &mut buf)?;
        domains.entry(domain).or_default().push(spec);
    }

    let mut preload_specs = Vec::with_capacity(preload_files.len());
    for &(ref path, fid) in &preload_files {
        let stem = read_vfs_entry(&vfs, path, fid, &mut scratch, &mut buf)?;
        preload_specs.push(analyze_preload(stem, &mut buf)?);
    }

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/luac/tables");
    fs::create_dir_all(&out_dir)?;

    let mut all_modules = BTreeSet::new();
    for (domain, specs) in &domains {
        all_modules.insert(*domain);
        emit_domain_module(&out_dir, domain, specs)?;
    }
    all_modules.insert("preload");
    emit_preload_module(&out_dir, &preload_specs)?;
    emit_mod_rs(&out_dir, &all_modules)?;

    println!(
        "Successfully generated {} domain modules ({} total tables) into {}",
        all_modules.len(),
        luatable_files.len() + preload_files.len(),
        out_dir.display()
    );
    Ok(())
}

fn main() -> Result<(), GenError> {
    run_codegen()
}

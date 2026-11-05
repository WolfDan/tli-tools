//! Domain classification and identifier naming rules for `DataCfg` code generation.

use crate::u32_to_usize;

/// Field names generated for a `{ Key, Value }` helper struct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KvFieldNames {
    pub key_name: String,
    pub val_names: Vec<String>,
}

/// Strips the leading `"config_"` prefix from a `DataCfg` file stem if present.
#[must_use]
pub fn strip_config_prefix(s: &str) -> &str {
    match s.strip_prefix("config_") {
        Some(rest) => rest,
        None => s,
    }
}

/// Converts a `snake_case` identifier into `PascalCase`.
#[must_use]
pub fn to_pascal_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
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

/// Converts a `PascalCase` or `camelCase` identifier into `snake_case`, escaping Rust keywords.
#[must_use]
pub fn to_snake_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
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

/// Classifies a `config_*` table stem into one of the 23 domain modules.
#[must_use]
pub fn classify_domain(stem: &str) -> &'static str {
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

fn infer_tuple_kv_names(table_short: &str, col_name: &str, arity: u32) -> KvFieldNames {
    let (key_str, vals): (&str, Vec<String>) = match (table_short, col_name, arity) {
        (_, "TargetAffixTier", 2) => (
            "tier",
            vec!["prob_type".to_owned(), "prob_per_myriad".to_owned()],
        ),
        (_, "Shape" | "Shape0", 2) => (
            "part_index",
            vec!["offset_x".to_owned(), "offset_y".to_owned()],
        ),
        (_, "ProfessionSpecifiedCost", 2) => (
            "profession_id",
            vec!["item_id".to_owned(), "count".to_owned()],
        ),
        (_, "DealSearchDefaultModifier", 2) => (
            "match_mode",
            vec!["affix_type".to_owned(), "min_value".to_owned()],
        ),
        ("cardrecipe", "RecipeUnique", 2) => (
            "recipe_id",
            vec!["target_id".to_owned(), "weight".to_owned()],
        ),
        (_, "AffixSlotNum", 3) => (
            "group",
            vec![
                "min_slots".to_owned(),
                "max_slots".to_owned(),
                "weight".to_owned(),
            ],
        ),
        (_, "ItemTypeList", 4) => (
            "mode",
            vec![
                "type1".to_owned(),
                "type2".to_owned(),
                "type3".to_owned(),
                "type4".to_owned(),
            ],
        ),
        (_, "ReshapeConsume" | "RotationConsume", 4) => (
            "step",
            vec![
                "min_count".to_owned(),
                "max_count".to_owned(),
                "item_id".to_owned(),
                "count".to_owned(),
            ],
        ),
        (_, _, n) => {
            let mut v_names = Vec::with_capacity(u32_to_usize(n));
            for idx in 1..=n {
                v_names.push(format!("param{idx}"));
            }
            ("key", v_names)
        }
    };
    KvFieldNames {
        key_name: key_str.to_owned(),
        val_names: vals,
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

/// Infers semantic Rust field names for a `{ Key, Value }` helper struct.
#[must_use]
pub fn infer_kv_field_names(
    table_stem: &str,
    col_name: &str,
    key_is_str: bool,
    val_is_list: bool,
    tuple_arity: Option<u32>,
) -> KvFieldNames {
    let table_short = strip_config_prefix(table_stem);
    if let Some(arity) = tuple_arity {
        return infer_tuple_kv_names(table_short, col_name, arity);
    }
    for &(t, c, k, v) in SPECIFIC_KV_FIELD_NAMES {
        if t == table_short && c == col_name {
            return KvFieldNames {
                key_name: k.to_owned(),
                val_names: vec![v.to_owned()],
            };
        }
    }
    let pair: (&str, &str) = match col_name {
        col if col.ends_with("Position") || col.ends_with("Pos") => ("x", "y"),
        "DefaultLanguage" | "LineNum" => ("min", "max"),
        col if col.ends_with("Range") => ("min", "max"),
        "EnableCost" => ("cost_key", "count"),
        "UninstallCost" => ("item_id", "counts"),
        col if col.ends_with("Consume") || col.ends_with("Cost") || col.starts_with("CostItem") => {
            if key_is_str {
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
            if val_is_list {
                ("modifier_id", "params")
            } else {
                ("modifier_id", "value")
            }
        }
        "Function" | "Event" | "Param" | "Parameter" | "Effect" | "Group" => {
            if val_is_list {
                ("kind", "params")
            } else {
                ("kind", "value")
            }
        }
        _ => {
            let col_snake = to_snake_case(col_name);
            return KvFieldNames {
                key_name: format!("{col_snake}_key"),
                val_names: vec![format!("{col_snake}_value")],
            };
        }
    };
    KvFieldNames {
        key_name: pair.0.to_owned(),
        val_names: vec![pair.1.to_owned()],
    }
}

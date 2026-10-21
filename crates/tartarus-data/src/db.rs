//! S16 Tartarus game database schema, ID mappers, and `.pak` Lua/localization joiner.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tartarus_extract::{LuaValue, PakVfs, strip_rich_text};

use crate::error::DataError;

const _: () = {
    assert!(std::mem::size_of::<S16RelicEntry>() >= 64);
    assert!(std::mem::size_of::<S16GameDatabase>() >= 64);
};

/// Authoritative mapping of all 31 active S16 Relics (`101..148`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct S16RelicEntry {
    pub id: i64,
    pub js_sim: String,
    pub icon_asset: String,
    pub sp_icon_asset: String,
    pub name_key: String,
    pub name: String,
    pub trigger_key: String,
    pub trigger_raw: String,
    pub trigger_text: String,
    pub effect_key: String,
    pub effect_raw: String,
    pub effect_text: String,
    pub relic_type: i64,
    pub priority: i64,
    pub weight: i64,
    pub effect_id: i64,
    pub unlock_type: i64,
    pub unlock_param: i64,
    pub one_turn_limit: i64,
    pub limit_total: i64,
}

/// Authoritative mapping of the 4 Warden's Gifts (`1..4`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct S16GiftEntry {
    pub id: i64,
    pub js_sim: String,
    pub icon_asset: String,
    pub name_key: String,
    pub name: String,
    pub text_key: String,
    pub text_raw: String,
    pub text_clean: String,
    pub effect_id: i64,
    pub reward_type: i64,
}

/// Authoritative mapping of the 17 S16 Prison Events (`1..103`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct S16EventEntry {
    pub id: i64,
    pub js_sim: String,
    pub name_key: String,
    pub name: String,
    pub desc_key: String,
    pub desc_raw: String,
    pub desc_text: String,
    pub effect_id: i64,
    pub weight: i64,
}

/// Authoritative mapping of the 5 S16 Trial Objectives (`1..5`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct S16TaskEntry {
    pub id: i64,
    pub js_sim: String,
    pub name_key: String,
    pub name: String,
    pub desc_key: String,
    pub desc_raw: String,
    pub desc_text: String,
    pub task_type: i64,
    pub progress_param: i64,
    pub weight: i64,
}

/// Authoritative mapping of the 4 Beast portrait pairs (`normal` and `jailed`)
/// and special cell mark icons.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct S16BeastEntry {
    pub beast: String,
    pub display_name: String,
    pub normal_portrait_asset: String,
    pub jailed_portrait_asset: String,
    pub special_mark_asset: Option<String>,
    pub special_mark_desc: Option<String>,
}

/// Complete S16 Tartarus game database extracted from `.pak` archives.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct S16GameDatabase {
    pub language: String,
    pub relics: Vec<S16RelicEntry>,
    pub start_themes: Vec<S16RelicEntry>,
    pub colored_rewards: Vec<S16GiftEntry>,
    pub beasts: Vec<S16BeastEntry>,
    pub events: Vec<S16EventEntry>,
    pub tasks: Vec<S16TaskEntry>,
}

/// Maps an official S16 Relic ID (`101..148`, `301..304`) to its canonical
/// `ToolId::as_str()` / `StartEventId` identifier.
pub fn map_relic_id_to_sim(id: i64) -> &'static str {
    debug_assert!((-1_000_000..=1_000_000).contains(&id));
    let out = match id {
        101 => "muzzle",
        102 => "ravenous",
        103 => "collar",
        104 => "revenge",
        105 => "cage",
        106 => "whip",
        107 => "saw",
        111 => "blacklist",
        112 => "keys",
        113 => "coin",
        114 => "fire",
        115 => "dogtag",
        116 => "forged",
        117 => "hook",
        121 => "scapegoat",
        122 => "axe",
        123 => "maiden",
        124 => "scales",
        125 => "rope",
        126 => "stake",
        127 => "pestle",
        131 => "shortchain",
        132 => "shackles",
        141 => "tongs",
        142 => "maul",
        143 => "trap",
        144 => "crown",
        145 => "claws",
        146 => "guillotine",
        147 => "ironbox",
        148 => "skewer",
        301 => "tyranny",
        302 => "opulence",
        303 => "ironlaw",
        304 => "foodchain",
        _ => "unknown",
    };
    debug_assert!(!out.is_empty());
    out
}

/// Maps an official S16 Colored Reward ID (`1..4`) to its `GiftId::as_str()` identifier.
pub fn map_gift_id_to_sim(id: i64) -> &'static str {
    debug_assert!((-1_000_000..=1_000_000).contains(&id));
    let out = match id {
        1 => "ring",
        2 => "horn",
        3 => "wine",
        4 => "brand",
        _ => "unknown",
    };
    debug_assert!(!out.is_empty());
    out
}

/// Maps an official S16 Event ID (`201..255`) to its `EventKind::as_str()` identifier.
pub fn map_event_id_to_sim(id: i64) -> &'static str {
    debug_assert!((-1_000_000..=1_000_000).contains(&id));
    let out = match id {
        201 => "convict",
        202 => "freedom",
        203 => "closed",
        211 => "appetite",
        212 => "portion",
        213 => "fortune",
        221 => "archenemy",
        222 => "efficiency",
        231 => "whispers",
        232 => "treasure",
        241 => "ratway",
        242 => "chain",
        251 => "transfer",
        252 => "usurp1",
        253 => "usurp2",
        254 => "usurp3",
        255 => "usurp4",
        _ => "unknown",
    };
    debug_assert!(!out.is_empty());
    out
}

/// Maps an official S16 Task ID (`1..5`) to its `ObjectiveId` identifier.
pub fn map_task_id_to_sim(id: i64) -> &'static str {
    debug_assert!((-1_000_000..=1_000_000).contains(&id));
    let out = match id {
        1 => "king",
        2 => "devour",
        3 => "open",
        4 => "incident",
        5 => "walk",
        _ => "other",
    };
    debug_assert!(!out.is_empty());
    out
}

/// Canonical beast portrait and cell mark definitions confirmed against the Pak VFS.
pub fn canonical_beasts() -> Vec<S16BeastEntry> {
    let list = vec![
        S16BeastEntry {
            beast: "mammoth".to_string(),
            display_name: "Mammoth".to_string(),
            normal_portrait_asset: "UI_T_V2_S16BoardCell_101".to_string(),
            jailed_portrait_asset: "UI_T_V2_S16BoardCell_045".to_string(),
            special_mark_asset: Some("UI_S16Gameplay_RelicSP_03".to_string()),
            special_mark_desc: Some(
                "Ivory / Tusk Mark (#141 Iron Tongs of Lust: Mammoth)".to_string(),
            ),
        },
        S16BeastEntry {
            beast: "lion".to_string(),
            display_name: "Berserk Lion".to_string(),
            normal_portrait_asset: "UI_T_V2_S16BoardCell_102".to_string(),
            jailed_portrait_asset: "UI_T_V2_S16BoardCell_047".to_string(),
            special_mark_asset: Some("UI_S16Gameplay_RelicSP_04".to_string()),
            special_mark_desc: Some(
                "Prey Mark (#143 Blood-Stained Trap: Berserk Lion)".to_string(),
            ),
        },
        S16BeastEntry {
            beast: "wolf".to_string(),
            display_name: "Alpha Wolf".to_string(),
            normal_portrait_asset: "UI_T_V2_S16BoardCell_103".to_string(),
            jailed_portrait_asset: "UI_T_V2_S16BoardCell_050".to_string(),
            special_mark_asset: Some("UI_S16Gameplay_RelicSP_01".to_string()),
            special_mark_desc: Some("Scratch Marks (#145 Filthy Claws: Alpha Wolf)".to_string()),
        },
        S16BeastEntry {
            beast: "rat".to_string(),
            display_name: "Colossal Rat".to_string(),
            normal_portrait_asset: "UI_T_V2_S16BoardCell_104".to_string(),
            jailed_portrait_asset: "UI_T_V2_S16BoardCell_048".to_string(),
            special_mark_asset: Some("UI_S16Gameplay_RelicSP_02".to_string()),
            special_mark_desc: Some(
                "Rat Nest Mark (#147 Squirming Iron Box: Colossal Rat)".to_string(),
            ),
        },
    ];
    assert_eq!(list.len(), 4);
    assert!(list.iter().all(|b| !b.beast.is_empty()));
    list
}

fn row_str(row: &BTreeMap<String, LuaValue>, key: &str) -> String {
    debug_assert!(!key.is_empty());
    let val = row
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    debug_assert!(val.len() <= 65536);
    val
}

fn row_i64(row: &BTreeMap<String, LuaValue>, key: &str) -> i64 {
    debug_assert!(!key.is_empty());
    let val = row.get(key).and_then(|v| v.as_i64()).unwrap_or(0);
    debug_assert!(val >= i64::MIN / 2);
    val
}

fn extract_s16_relics_and_themes(
    vfs: &PakVfs,
    lang_map: &BTreeMap<String, String>,
) -> Result<(Vec<S16RelicEntry>, Vec<S16RelicEntry>), DataError> {
    let relic_tbl = vfs.read_lua_config_table(
        "UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_s16_gameplay_relic.out",
    )?;
    assert!(!relic_tbl.rows.is_empty());
    let mut relics = Vec::new();
    let mut start_themes = Vec::new();

    for (&id, row) in &relic_tbl.rows {
        let name_key = row_str(row, "_Name");
        let trigger_key = row_str(row, "_DescStory");
        let effect_key = row_str(row, "_Desc");

        let name = lang_map.get(&name_key).cloned().unwrap_or_default();
        let trigger_raw = lang_map.get(&trigger_key).cloned().unwrap_or_default();
        let effect_raw = lang_map.get(&effect_key).cloned().unwrap_or_default();

        let entry = S16RelicEntry {
            id,
            js_sim: map_relic_id_to_sim(id).to_string(),
            icon_asset: row_str(row, "Pic"),
            sp_icon_asset: row_str(row, "PicIconSP"),
            name_key,
            name: strip_rich_text(&name),
            trigger_key,
            trigger_text: strip_rich_text(&trigger_raw),
            trigger_raw,
            effect_key,
            effect_text: strip_rich_text(&effect_raw),
            effect_raw,
            relic_type: row_i64(row, "RelicType"),
            priority: row_i64(row, "Priority"),
            weight: row_i64(row, "Weight"),
            effect_id: row_i64(row, "EffectId"),
            unlock_type: row_i64(row, "UnlockType"),
            unlock_param: row_i64(row, "UnlockParam"),
            one_turn_limit: row_i64(row, "OneTurnTriggerLimit"),
            limit_total: row_i64(row, "TotalTriggerLimit"),
        };

        if id >= 300 {
            start_themes.push(entry);
        } else {
            relics.push(entry);
        }
    }
    assert_eq!(relics.len() + start_themes.len(), relic_tbl.rows.len());
    Ok((relics, start_themes))
}

fn extract_s16_gifts(
    vfs: &PakVfs,
    lang_map: &BTreeMap<String, String>,
) -> Result<Vec<S16GiftEntry>, DataError> {
    let gift_path = concat!(
        "UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/",
        "config_s16_gameplay_colored_reward.out"
    );
    let gift_tbl = vfs.read_lua_config_table(gift_path)?;
    assert!(!gift_tbl.rows.is_empty());
    let mut colored_rewards = Vec::new();
    for (&id, row) in &gift_tbl.rows {
        let name_key = row_str(row, "_Name");
        let text_key = row_str(row, "_Text");
        let name = lang_map.get(&name_key).cloned().unwrap_or_default();
        let text_raw = lang_map.get(&text_key).cloned().unwrap_or_default();

        colored_rewards.push(S16GiftEntry {
            id,
            js_sim: map_gift_id_to_sim(id).to_string(),
            icon_asset: row_str(row, "Pic"),
            name_key,
            name: strip_rich_text(&name),
            text_key,
            text_clean: strip_rich_text(&text_raw),
            text_raw,
            effect_id: row_i64(row, "EffectId"),
            reward_type: row_i64(row, "RewardType"),
        });
    }
    assert_eq!(colored_rewards.len(), gift_tbl.rows.len());
    Ok(colored_rewards)
}

fn extract_s16_events(
    vfs: &PakVfs,
    lang_map: &BTreeMap<String, String>,
) -> Result<Vec<S16EventEntry>, DataError> {
    let event_tbl = vfs.read_lua_config_table(
        "UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_s16_gameplay_event.out",
    )?;
    assert!(!event_tbl.rows.is_empty());
    let mut events = Vec::new();
    for (&id, row) in &event_tbl.rows {
        let name_key = row_str(row, "_Name");
        let desc_key = row_str(row, "_Desc");
        let name = lang_map.get(&name_key).cloned().unwrap_or_default();
        let desc_raw = lang_map.get(&desc_key).cloned().unwrap_or_default();

        events.push(S16EventEntry {
            id,
            js_sim: map_event_id_to_sim(id).to_string(),
            name_key,
            name: strip_rich_text(&name),
            desc_key,
            desc_text: strip_rich_text(&desc_raw),
            desc_raw,
            effect_id: row_i64(row, "EffectId"),
            weight: row_i64(row, "Weight"),
        });
    }
    assert_eq!(events.len(), event_tbl.rows.len());
    Ok(events)
}

fn extract_s16_tasks(
    vfs: &PakVfs,
    lang_map: &BTreeMap<String, String>,
) -> Result<Vec<S16TaskEntry>, DataError> {
    let task_tbl = vfs.read_lua_config_table(
        "UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_s16_gameplay_task.out",
    )?;
    assert!(!task_tbl.rows.is_empty());
    let mut tasks = Vec::new();
    for (&id, row) in &task_tbl.rows {
        let name_key = row_str(row, "_Name");
        let desc_key = row_str(row, "_Desc");
        let name = lang_map.get(&name_key).cloned().unwrap_or_default();
        let desc_raw = lang_map.get(&desc_key).cloned().unwrap_or_default();

        tasks.push(S16TaskEntry {
            id,
            js_sim: map_task_id_to_sim(id).to_string(),
            name_key,
            name: strip_rich_text(&name),
            desc_key,
            desc_text: strip_rich_text(&desc_raw),
            desc_raw,
            task_type: row_i64(row, "TaskType"),
            progress_param: row_i64(row, "ProgressParam"),
            weight: row_i64(row, "Weight"),
        });
    }
    assert_eq!(tasks.len(), task_tbl.rows.len());
    Ok(tasks)
}

/// Extracts and joins all S16 Lua config tables (`config_s16_gameplay_relic.out`,
/// `config_s16_gameplay_colored_reward.out`, `config_s16_gameplay_event.out`,
/// `config_s16_gameplay_task.out`) with `config_multi_language_<lang>.bin`.
pub fn extract_s16_game_database(vfs: &PakVfs, lang: &str) -> Result<S16GameDatabase, DataError> {
    assert!(!lang.is_empty());
    let lang_map = vfs.read_multi_language(lang, Some("s16_gameplay_"))?;
    let (relics, start_themes) = extract_s16_relics_and_themes(vfs, &lang_map)?;
    let colored_rewards = extract_s16_gifts(vfs, &lang_map)?;
    let events = extract_s16_events(vfs, &lang_map)?;
    let tasks = extract_s16_tasks(vfs, &lang_map)?;
    let beasts = canonical_beasts();
    assert_eq!(beasts.len(), 4);

    Ok(S16GameDatabase {
        language: lang.to_string(),
        relics,
        start_themes,
        colored_rewards,
        beasts,
        events,
        tasks,
    })
}

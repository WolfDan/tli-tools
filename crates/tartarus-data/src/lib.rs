//! Authoritative S16 Tartarus game database, visual asset catalog renderer,
//! and static Rust CV table generator (`crates/tartarus-scan/src/cv_data.rs`).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tartarus_extract::{PakVfs, crop_rgba8, encode_png_rgba8, strip_rich_text};

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
    pub total_limit: i64,
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

/// Authoritative mapping of the 4 Beast portrait pairs (`normal` and `jailed`) and special cell mark icons.
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

/// Decoded RGBA8 texture or sliced PaperSprite from the Pak VFS.
#[derive(Clone, Debug)]
pub struct DecodedAssetImage {
    pub virtual_path: String,
    pub short_name: String,
    pub width: usize,
    pub height: usize,
    pub rgba8: Vec<u8>,
}

/// Maps an official S16 Relic ID (`101..148`, `301..304`) to its canonical `ToolId::as_str()` / `StartEventId` identifier.
pub fn map_relic_id_to_sim(id: i64) -> &'static str {
    match id {
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
    }
}

/// Maps an official S16 Colored Reward ID (`1..4`) to its `GiftId::as_str()` identifier.
pub fn map_gift_id_to_sim(id: i64) -> &'static str {
    match id {
        1 => "ring",
        2 => "horn",
        3 => "wine",
        4 => "brand",
        _ => "unknown",
    }
}

/// Maps an official S16 Event ID (`201..255`) to its `EventKind::as_str()` identifier.
pub fn map_event_id_to_sim(id: i64) -> &'static str {
    match id {
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
    }
}

/// Maps an official S16 Task ID (`1..5`) to its `ObjectiveId` identifier.
pub fn map_task_id_to_sim(id: i64) -> &'static str {
    match id {
        1 => "king",
        2 => "devour",
        3 => "open",
        4 => "incident",
        5 => "walk",
        _ => "other",
    }
}

/// Canonical beast portrait and cell mark definitions confirmed against the Pak VFS.
pub fn canonical_beasts() -> Vec<S16BeastEntry> {
    vec![
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
    ]
}

/// Extracts and joins all S16 Lua config tables (`config_s16_gameplay_relic.out`,
/// `config_s16_gameplay_colored_reward.out`, `config_s16_gameplay_event.out`,
/// `config_s16_gameplay_task.out`) with `config_multi_language_<lang>.bin`.
pub fn extract_s16_game_database(vfs: &PakVfs, lang: &str) -> Result<S16GameDatabase, String> {
    let lang_map = vfs.read_multi_language(lang, Some("s16_gameplay_"))?;

    let relic_tbl = vfs.read_lua_config_table(
        "UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_s16_gameplay_relic.out",
    )?;
    let gift_tbl = vfs.read_lua_config_table(
        "UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_s16_gameplay_colored_reward.out",
    )?;
    let event_tbl = vfs.read_lua_config_table(
        "UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_s16_gameplay_event.out",
    )?;
    let task_tbl = vfs.read_lua_config_table(
        "UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_s16_gameplay_task.out",
    )?;

    let mut relics = Vec::new();
    let mut start_themes = Vec::new();

    for (&id, row) in &relic_tbl.rows {
        let get_str = |k: &str| {
            row.get(k)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        };
        let get_i64 = |k: &str| row.get(k).and_then(|v| v.as_i64()).unwrap_or(0);

        let name_key = get_str("_Name");
        let trigger_key = get_str("_DescStory");
        let effect_key = get_str("_Desc");

        let name = lang_map.get(&name_key).cloned().unwrap_or_default();
        let trigger_raw = lang_map.get(&trigger_key).cloned().unwrap_or_default();
        let effect_raw = lang_map.get(&effect_key).cloned().unwrap_or_default();

        let entry = S16RelicEntry {
            id,
            js_sim: map_relic_id_to_sim(id).to_string(),
            icon_asset: get_str("Pic"),
            sp_icon_asset: get_str("PicIconSP"),
            name_key,
            name: strip_rich_text(&name),
            trigger_key,
            trigger_text: strip_rich_text(&trigger_raw),
            trigger_raw,
            effect_key,
            effect_text: strip_rich_text(&effect_raw),
            effect_raw,
            relic_type: get_i64("RelicType"),
            priority: get_i64("Priority"),
            weight: get_i64("Weight"),
            effect_id: get_i64("EffectId"),
            unlock_type: get_i64("UnlockType"),
            unlock_param: get_i64("UnlockParam"),
            one_turn_limit: get_i64("OneTurnTriggerLimit"),
            total_limit: get_i64("TotalTriggerLimit"),
        };

        if id >= 300 {
            start_themes.push(entry);
        } else {
            relics.push(entry);
        }
    }

    let mut colored_rewards = Vec::new();
    for (&id, row) in &gift_tbl.rows {
        let get_str = |k: &str| {
            row.get(k)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        };
        let get_i64 = |k: &str| row.get(k).and_then(|v| v.as_i64()).unwrap_or(0);

        let name_key = get_str("_Name");
        let text_key = get_str("_Text");
        let name = lang_map.get(&name_key).cloned().unwrap_or_default();
        let text_raw = lang_map.get(&text_key).cloned().unwrap_or_default();

        colored_rewards.push(S16GiftEntry {
            id,
            js_sim: map_gift_id_to_sim(id).to_string(),
            icon_asset: get_str("Pic"),
            name_key,
            name: strip_rich_text(&name),
            text_key,
            text_clean: strip_rich_text(&text_raw),
            text_raw,
            effect_id: get_i64("EffectId"),
            reward_type: get_i64("RewardType"),
        });
    }

    let mut events = Vec::new();
    for (&id, row) in &event_tbl.rows {
        let get_str = |k: &str| {
            row.get(k)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        };
        let get_i64 = |k: &str| row.get(k).and_then(|v| v.as_i64()).unwrap_or(0);

        let name_key = get_str("_Name");
        let desc_key = get_str("_Desc");
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
            effect_id: get_i64("EffectId"),
            weight: get_i64("Weight"),
        });
    }

    let mut tasks = Vec::new();
    for (&id, row) in &task_tbl.rows {
        let get_str = |k: &str| {
            row.get(k)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        };
        let get_i64 = |k: &str| row.get(k).and_then(|v| v.as_i64()).unwrap_or(0);

        let name_key = get_str("_Name");
        let desc_key = get_str("_Desc");
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
            task_type: get_i64("TaskType"),
            progress_param: get_i64("ProgressParam"),
            weight: get_i64("Weight"),
        });
    }

    Ok(S16GameDatabase {
        language: lang.to_string(),
        relics,
        start_themes,
        colored_rewards,
        beasts: canonical_beasts(),
        events,
        tasks,
    })
}

/// Decodes all S16 textures and PaperSprites needed for the catalog and CV feature synthesis,
/// keyed by short asset name (e.g. `"UI_S16Gameplay_RelicIcon_01"`, `"UI_Img_S16Gameplay_Jail_Bg_Normal"`).
pub fn load_s16_textures_by_name(vfs: &PakVfs) -> BTreeMap<String, DecodedAssetImage> {
    let mut map: BTreeMap<String, DecodedAssetImage> = BTreeMap::new();
    let filters = [
        "s16gameplay",
        "mercenary_style",
        "s16mercenary",
        "s16boardcell",
    ];

    for meta in vfs.entries.values() {
        let vpath = &meta.virtual_path;
        if !vpath.ends_with(".uasset")
            || vpath.ends_with("_Sprite.uasset")
            || vpath.ends_with("_Atlas.uasset")
        {
            continue;
        }
        let lower = vpath.to_lowercase();
        if !filters.iter().any(|f| lower.contains(f)) {
            continue;
        }
        if let Ok(tex) = vfs.decode_texture(vpath)
            && tex.width >= 16
            && tex.height >= 16
            && tex.width <= 4096
            && tex.height <= 4096
        {
            let short_name = Path::new(vpath)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if map.get(&short_name).is_some_and(|prev| {
                prev.virtual_path.contains("/NoAtlas/") && !vpath.contains("/NoAtlas/")
            }) {
                continue;
            }
            map.insert(
                short_name.clone(),
                DecodedAssetImage {
                    virtual_path: vpath.clone(),
                    short_name,
                    width: tex.width,
                    height: tex.height,
                    rgba8: tex.rgba8,
                },
            );
        }
    }

    // Also load any UPaperSprite assets if not already decoded as standalone textures
    let mut by_atlas: BTreeMap<String, Vec<(&str, tartarus_extract::PaperSpriteInfo)>> =
        BTreeMap::new();
    for meta in vfs.entries.values() {
        let vpath = &meta.virtual_path;
        if !vpath.ends_with("_Sprite.uasset") {
            continue;
        }
        let lower = vpath.to_lowercase();
        if !filters.iter().any(|f| lower.contains(f)) {
            continue;
        }
        if let Ok(info) = vfs.read_paper_sprite(vpath) {
            by_atlas
                .entry(info.atlas_virtual_path.clone())
                .or_default()
                .push((vpath, info));
        }
    }

    for (atlas_path, list) in by_atlas {
        if let Ok(atlas_tex) = vfs.decode_texture(&atlas_path) {
            for (vpath, info) in list {
                let cw =
                    (info.width as usize).min(atlas_tex.width.saturating_sub(info.uv_x as usize));
                let ch =
                    (info.height as usize).min(atlas_tex.height.saturating_sub(info.uv_y as usize));
                if cw < 12 || ch < 12 {
                    continue;
                }
                let short_name = Path::new(vpath)
                    .file_stem()
                    .map(|s| s.to_string_lossy().trim_end_matches("_Sprite").to_string())
                    .unwrap_or_default();
                if map.contains_key(&short_name) {
                    continue;
                }
                let rgba8 = crop_rgba8(
                    &atlas_tex.rgba8,
                    atlas_tex.width,
                    atlas_tex.height,
                    info.uv_x as usize,
                    info.uv_y as usize,
                    cw,
                    ch,
                );
                map.insert(
                    short_name.clone(),
                    DecodedAssetImage {
                        virtual_path: vpath.to_string(),
                        short_name,
                        width: cw,
                        height: ch,
                        rgba8,
                    },
                );
            }
        }
    }

    map
}

#[inline]
pub fn hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    use palette::{FromColor, Hsv, Srgb};
    let c = Hsv::from_color(Srgb::new(r / 255.0, g / 255.0, b / 255.0));
    (c.hue.into_positive_degrees(), c.saturation, c.value)
}

pub fn alpha_bbox_thresh(
    rgba: &[u8],
    w: usize,
    h: usize,
    thresh: u8,
) -> (usize, usize, usize, usize) {
    let mut x0 = w;
    let mut y0 = h;
    let mut x1 = 0usize;
    let mut y1 = 0usize;
    for y in 0..h {
        for x in 0..w {
            if rgba[(y * w + x) * 4 + 3] > thresh {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + 1);
                y1 = y1.max(y + 1);
            }
        }
    }
    if x0 < x1 && y0 < y1 {
        (x0, y0, x1, y1)
    } else {
        (0, 0, w, h)
    }
}

pub fn alpha_bbox(rgba: &[u8], w: usize, h: usize) -> (usize, usize, usize, usize) {
    alpha_bbox_thresh(rgba, w, h, 16)
}

/// Computes the exact 1%–99% percentile bounding box used by `tartarus_scan::pick_art_box`
/// on an RGBA buffer within `[x0..x1, y0..y1]`.
pub fn pick_art_bbox_on_rgba(
    rgba: &[u8],
    (img_w, img_h): (usize, usize),
    (x0, y0, x1, y1): (usize, usize, usize, usize),
    mx_hi: u8,
    v_thr: f32,
) -> Option<(usize, usize, usize, usize)> {
    let x1 = x1.min(img_w);
    let y1 = y1.min(img_h);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let w = x1 - x0;
    let h = y1 - y0;
    let mut x_hist = vec![0u32; w];
    let mut y_hist = vec![0u32; h];
    let mut count = 0usize;
    for (dy, y_slot) in y_hist.iter_mut().enumerate() {
        let row_base = (y0 + dy) * img_w * 4;
        let mut row_hits = 0u32;
        for (dx, x_slot) in x_hist.iter_mut().enumerate() {
            let o = row_base + (x0 + dx) * 4;
            let r = rgba[o];
            let g = rgba[o + 1];
            let b = rgba[o + 2];
            let mx = r.max(g).max(b);
            let matched = if mx >= mx_hi {
                true
            } else if mx <= 45 {
                false
            } else {
                let mn = r.min(g).min(b);
                if ((mx - mn) as u32) * 20 <= 9 * (mx as u32) {
                    false
                } else {
                    let (hue, sat, v) = hsv(r as f32, g as f32, b as f32);
                    v > v_thr || (sat > 0.45 && v > 0.18 && !(15.0..=50.0).contains(&hue))
                }
            };
            if matched {
                *x_slot += 1;
                row_hits += 1;
            }
        }
        *y_slot = row_hits;
        count += row_hits as usize;
    }
    if count < 64 {
        return None;
    }
    let rank_lo = (0.01 * ((count - 1) as f64)).round() as u32;
    let rank_hi = (0.99 * ((count - 1) as f64)).round() as u32;
    let percentile = |hist: &[u32], base: usize, rank: u32| -> usize {
        let mut acc = 0u32;
        for (idx, &c) in hist.iter().enumerate() {
            acc += c;
            if acc > rank {
                return base + idx;
            }
        }
        base + hist.len().saturating_sub(1)
    };
    let bx0 = percentile(&x_hist, x0, rank_lo);
    let by0 = percentile(&y_hist, y0, rank_lo);
    let bx1 = (percentile(&x_hist, x0, rank_hi) + 1).min(img_w);
    let by1 = (percentile(&y_hist, y0, rank_hi) + 1).min(img_h);
    if bx1 > bx0 && by1 > by0 {
        Some((bx0, by0, bx1, by1))
    } else {
        None
    }
}

/// Computes the `4x6` RGB grid (`[f32; 72]`) and 13-bin HSV histogram (`[f32; 13]`) matching `tartarus-scan::art_feature`.
pub fn compute_art_feature_region(
    rgba: &[u8],
    (w, h): (usize, usize),
    (x0, y0, x1, y1): (usize, usize, usize, usize),
    bg_rgb: (f32, f32, f32),
) -> ([f32; 72], [f32; 13]) {
    let gx = 4usize;
    let gy = 6usize;
    let mut grid = [0.0f32; 72];
    let mut hist = [0.0f32; 13];
    let rw = x1.saturating_sub(x0).max(1);
    let rh = y1.saturating_sub(y0).max(1);

    for j in 0..gy {
        let py0 = y0 + (j * rh) / gy;
        let py1 = (y0 + ((j + 1) * rh) / gy).max(py0 + 1).min(h);
        for i in 0..gx {
            let px0 = x0 + (i * rw) / gx;
            let px1 = (x0 + ((i + 1) * rw) / gx).max(px0 + 1).min(w);
            let mut r_sum = 0.0f32;
            let mut g_sum = 0.0f32;
            let mut b_sum = 0.0f32;
            let mut n = 0usize;
            for y in py0..py1 {
                for x in px0..px1 {
                    let idx = (y * w + x) * 4;
                    let a = (rgba[idx + 3] as f32) / 255.0;
                    let rf = (rgba[idx] as f32) * a + bg_rgb.0 * (1.0 - a);
                    let gf = (rgba[idx + 1] as f32) * a + bg_rgb.1 * (1.0 - a);
                    let bf = (rgba[idx + 2] as f32) * a + bg_rgb.2 * (1.0 - a);
                    r_sum += rf;
                    g_sum += gf;
                    b_sum += bf;
                    n += 1;
                    let (hh, ss, vv) = hsv(rf, gf, bf);
                    if ss > 0.25 && vv > 0.25 {
                        hist[((hh / 30.0).floor() as usize) % 12] += ss * vv;
                    } else {
                        hist[12] += vv * 0.3;
                    }
                }
            }
            if n > 0 {
                let o = (j * gx + i) * 3;
                let nf = n as f32;
                grid[o] = r_sum / nf;
                grid[o + 1] = g_sum / nf;
                grid[o + 2] = b_sum / nf;
            }
        }
    }
    (grid, hist)
}

/// Synthesizes a `([f32; 72], [f32; 13])` feature vector from a decoded game icon texture
/// using its alpha bounding box and the target UI backdrop color.
pub fn synthesize_icon_feature(
    img: &DecodedAssetImage,
    bg_rgb: (f32, f32, f32),
) -> ([f32; 72], [f32; 13]) {
    let (x0, y0, x1, y1) = alpha_bbox(&img.rgba8, img.width, img.height);
    compute_art_feature_region(
        &img.rgba8,
        (img.width, img.height),
        (x0, y0, x1, y1),
        bg_rgb,
    )
}

// ============================================================================
// Visual Catalog Rendering (3 clean sheets with font8x8 bitmap font)
// ============================================================================

fn draw_bitmap_label(
    buf: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    text: &str,
    color: [u8; 4],
) {
    use font8x8::UnicodeFonts;
    let mut cx = x;
    for ch in text.chars() {
        let ch_ascii = match ch {
            '\u{2018}' | '\u{2019}' => '\'',
            '\u{201c}' | '\u{201d}' => '"',
            '\u{2013}' | '\u{2014}' => '-',
            other => other.to_ascii_uppercase(),
        };
        if let Some(rows) = font8x8::BASIC_FONTS.get(ch_ascii) {
            for (gy, &row_bits) in rows.iter().enumerate() {
                for gx in 0..8 {
                    if (row_bits >> gx) & 1 != 0 {
                        let px = cx + gx;
                        let py = y + gy;
                        if px < w && py < h {
                            let idx = (py * w + px) * 4;
                            buf[idx..idx + 4].copy_from_slice(&color);
                        }
                    }
                }
            }
        }
        cx += 6;
    }
}

fn wrap_text_lines(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    textwrap::wrap(text, max_chars)
        .into_iter()
        .take(max_lines)
        .map(|line| line.into_owned())
        .collect()
}

fn fill_rect(
    buf: &mut [u8],
    (w, h): (usize, usize),
    (x0, y0, rw, rh): (usize, usize, usize, usize),
    color: [u8; 4],
) {
    for y in y0..(y0 + rh).min(h) {
        for x in x0..(x0 + rw).min(w) {
            let idx = (y * w + x) * 4;
            buf[idx..idx + 4].copy_from_slice(&color);
        }
    }
}

fn blit_scaled(
    dst: &mut [u8],
    (dw, dh): (usize, usize),
    (dx0, dy0, tw, th): (usize, usize, usize, usize),
    src_img: &DecodedAssetImage,
) {
    if src_img.width == 0 || src_img.height == 0 || tw == 0 || th == 0 {
        return;
    }
    let Some(src) = image::RgbaImage::from_raw(
        src_img.width as u32,
        src_img.height as u32,
        src_img.rgba8.clone(),
    ) else {
        return;
    };
    let resized = image::imageops::resize(
        &src,
        tw as u32,
        th as u32,
        image::imageops::FilterType::Nearest,
    );
    for (x, y, px) in resized.enumerate_pixels() {
        let tx = dx0 + x as usize;
        let ty = dy0 + y as usize;
        if tx < dw && ty < dh {
            let a = (px[3] as u32) as f32 / 255.0;
            let r = (px[0] as f32 * a + 32.0 * (1.0 - a)) as u8;
            let g = (px[1] as f32 * a + 28.0 * (1.0 - a)) as u8;
            let b = (px[2] as f32 * a + 36.0 * (1.0 - a)) as u8;
            let d_idx = (ty * dw + tx) * 4;
            dst[d_idx..d_idx + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }
}

/// Renders `catalog_relics.png`: all 31 active S16 Torture Instruments (`RelicIcon_01..31`)
/// with their official ID, simulator key, English name, trigger text, and effect text.
pub fn render_catalog_relics_png(
    out_path: &Path,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), String> {
    let cols = 3usize;
    let cell_w = 420usize;
    let cell_h = 112usize;
    let rows = db.relics.len().div_ceil(cols);
    let width = 24 + cols * cell_w;
    let height = 52 + rows * cell_h;
    let mut buf = vec![0u8; width * height * 4];
    for px in buf.as_chunks_mut::<4>().0 {
        px.copy_from_slice(&[18, 16, 22, 255]);
    }

    draw_bitmap_label(
        &mut buf,
        width,
        height,
        16,
        16,
        "S16 TARTARUS OFFICIAL RELIC CATALOG (31 ACTIVE TORTURE INSTRUMENTS)",
        [250, 210, 120, 255],
    );

    for (idx, relic) in db.relics.iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;
        let x0 = 16 + col * cell_w;
        let y0 = 44 + row * cell_h;

        fill_rect(
            &mut buf,
            (width, height),
            (x0, y0, cell_w - 12, cell_h - 8),
            [28, 25, 34, 255],
        );

        if let Some(img) = textures.get(&relic.icon_asset) {
            blit_scaled(&mut buf, (width, height), (x0 + 8, y0 + 10, 76, 76), img);
        }

        let compact_asset = relic.icon_asset.trim_start_matches("UI_S16Gameplay_");
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            x0 + 8,
            y0 + 90,
            compact_asset,
            [150, 145, 165, 255],
        );

        let tx = x0 + 92;
        let hdr = format!(
            "#{} [{}] {}",
            relic.id,
            relic.js_sim.to_uppercase(),
            relic.name
        );
        let hdr_trunc: String = hdr.chars().take(51).collect();
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            tx,
            y0 + 8,
            &hdr_trunc,
            [255, 220, 130, 255],
        );

        let trig_str = format!("WHEN: {}", relic.trigger_text);
        let trig_lines = wrap_text_lines(&trig_str, 51, 2);
        let mut cy = y0 + 22;
        for line in trig_lines {
            draw_bitmap_label(&mut buf, width, height, tx, cy, &line, [170, 220, 255, 255]);
            cy += 11;
        }

        let eff_str = format!("DOES: {}", relic.effect_text);
        let eff_lines = wrap_text_lines(&eff_str, 51, 4);
        cy += 2;
        for line in eff_lines {
            if cy + 8 <= y0 + cell_h - 8 {
                draw_bitmap_label(&mut buf, width, height, tx, cy, &line, [215, 235, 205, 255]);
                cy += 11;
            }
        }
    }

    let png = encode_png_rgba8(width as u32, height as u32, &buf)?;
    fs::write(out_path, png).map_err(|e| e.to_string())
}

/// Renders `catalog_beasts_and_cells.png`: the 4 Beasts (`Mammoth`, `Lion`, `Wolf`, `Rat`)
/// with both their normal portrait (`101..104`) and behind-bars portrait (`045/047/050/048`),
/// plus their special cell marks (`RelicSP_01..04`).
pub fn render_catalog_beasts_and_cells_png(
    out_path: &Path,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), String> {
    let width = 980usize;
    let row_h = 128usize;
    let height = 56 + db.beasts.len() * row_h;
    let mut buf = vec![0u8; width * height * 4];
    for px in buf.as_chunks_mut::<4>().0 {
        px.copy_from_slice(&[18, 16, 22, 255]);
    }

    draw_bitmap_label(
        &mut buf,
        width,
        height,
        16,
        16,
        "S16 TARTARUS BEAST PORTRAITS (NORMAL + BEHIND BARS) & SPECIAL CELL MARKS",
        [250, 210, 120, 255],
    );

    for (idx, b) in db.beasts.iter().enumerate() {
        let y0 = 46 + idx * row_h;
        fill_rect(
            &mut buf,
            (width, height),
            (16, y0, width - 32, row_h - 10),
            [28, 25, 34, 255],
        );

        let title = format!(
            "{} ({})",
            b.display_name.to_uppercase(),
            b.beast.to_uppercase()
        );
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            28,
            y0 + 12,
            &title,
            [255, 220, 130, 255],
        );

        if let Some(norm) = textures.get(&b.normal_portrait_asset) {
            blit_scaled(&mut buf, (width, height), (260, y0 + 10, 84, 84), norm);
            draw_bitmap_label(
                &mut buf,
                width,
                height,
                260,
                y0 + 98,
                &format!(
                    "NORMAL: {}",
                    b.normal_portrait_asset.trim_start_matches("UI_T_V2_")
                ),
                [190, 235, 190, 255],
            );
        }

        if let Some(jailed) = textures.get(&b.jailed_portrait_asset) {
            blit_scaled(&mut buf, (width, height), (470, y0 + 10, 84, 84), jailed);
            draw_bitmap_label(
                &mut buf,
                width,
                height,
                470,
                y0 + 98,
                &format!(
                    "JAILED: {}",
                    b.jailed_portrait_asset.trim_start_matches("UI_T_V2_")
                ),
                [170, 220, 255, 255],
            );
        }

        if let Some(sp_name) = &b.special_mark_asset {
            if let Some(sp_img) = textures.get(sp_name) {
                blit_scaled(&mut buf, (width, height), (690, y0 + 18, 64, 64), sp_img);
            }
            draw_bitmap_label(
                &mut buf,
                width,
                height,
                690,
                y0 + 98,
                sp_name.trim_start_matches("UI_S16Gameplay_"),
                [245, 200, 140, 255],
            );
        }

        if let Some(desc) = &b.special_mark_desc {
            let lines = wrap_text_lines(desc, 34, 3);
            for (li, line) in lines.iter().enumerate() {
                draw_bitmap_label(
                    &mut buf,
                    width,
                    height,
                    28,
                    y0 + 36 + li * 13,
                    line,
                    [215, 210, 225, 255],
                );
            }
        }
    }

    let png = encode_png_rgba8(width as u32, height as u32, &buf)?;
    fs::write(out_path, png).map_err(|e| e.to_string())
}

/// Renders `catalog_gifts_themes_events.png`: the 4 Warden's Gifts (`ColorRewards_01..04`),
/// 4 Start Themes (`SpecialTheme_01..04`), 5 Trial Objectives (`Tasks 1..5`), and 17 Prison Events (`Events 201..255`).
pub fn render_catalog_gifts_themes_events_png(
    out_path: &Path,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), String> {
    let width = 1280usize;
    let height = 980usize;
    let mut buf = vec![0u8; width * height * 4];
    for px in buf.as_chunks_mut::<4>().0 {
        px.copy_from_slice(&[18, 16, 22, 255]);
    }

    // Section 1: 4 Warden's Gifts + 4 Start Themes
    draw_bitmap_label(
        &mut buf,
        width,
        height,
        16,
        14,
        "WARDEN'S GIFTS (COLORED REWARDS 1..4) & STARTING THEMES (301..304)",
        [250, 210, 120, 255],
    );

    for (idx, gift) in db.colored_rewards.iter().enumerate() {
        let x0 = 16 + idx * 312;
        let y0 = 34;
        fill_rect(
            &mut buf,
            (width, height),
            (x0, y0, 300, 100),
            [28, 25, 34, 255],
        );
        if let Some(img) = textures.get(&gift.icon_asset) {
            blit_scaled(&mut buf, (width, height), (x0 + 8, y0 + 12, 68, 68), img);
        }
        let hdr = format!(
            "GIFT #{} [{}] {}",
            gift.id,
            gift.js_sim.to_uppercase(),
            gift.name
        );
        let hdr_t: String = hdr.chars().take(34).collect();
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            x0 + 84,
            y0 + 10,
            &hdr_t,
            [255, 220, 130, 255],
        );
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            x0 + 84,
            y0 + 23,
            gift.icon_asset.trim_start_matches("UI_S16Gameplay_"),
            [150, 200, 240, 255],
        );
        for (li, line) in wrap_text_lines(&gift.text_clean, 34, 4).iter().enumerate() {
            draw_bitmap_label(
                &mut buf,
                width,
                height,
                x0 + 84,
                y0 + 38 + li * 12,
                line,
                [215, 235, 205, 255],
            );
        }
    }

    for (idx, theme) in db.start_themes.iter().enumerate() {
        let x0 = 16 + idx * 312;
        let y0 = 144;
        fill_rect(
            &mut buf,
            (width, height),
            (x0, y0, 300, 108),
            [28, 25, 34, 255],
        );
        if let Some(img) = textures.get(&theme.icon_asset) {
            blit_scaled(&mut buf, (width, height), (x0 + 8, y0 + 14, 68, 68), img);
        }
        let hdr = format!(
            "THEME #{} [{}] {}",
            theme.id,
            theme.js_sim.to_uppercase(),
            theme.name
        );
        let hdr_t: String = hdr.chars().take(34).collect();
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            x0 + 84,
            y0 + 10,
            &hdr_t,
            [255, 220, 130, 255],
        );
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            x0 + 84,
            y0 + 23,
            theme.icon_asset.trim_start_matches("UI_S16Gameplay_"),
            [150, 200, 240, 255],
        );
        for (li, line) in wrap_text_lines(&theme.effect_text, 34, 5)
            .iter()
            .enumerate()
        {
            draw_bitmap_label(
                &mut buf,
                width,
                height,
                x0 + 84,
                y0 + 38 + li * 12,
                line,
                [215, 235, 205, 255],
            );
        }
    }

    // Section 2: 5 Trial Objectives (Tasks)
    draw_bitmap_label(
        &mut buf,
        width,
        height,
        16,
        266,
        "TRIAL OBJECTIVES (CONFIG_S16_GAMEPLAY_TASK 1..5)",
        [250, 210, 120, 255],
    );
    for (idx, task) in db.tasks.iter().enumerate() {
        let x0 = 16 + idx * 250;
        let y0 = 284;
        fill_rect(
            &mut buf,
            (width, height),
            (x0, y0, 240, 74),
            [28, 25, 34, 255],
        );
        let hdr = format!(
            "TASK #{} [{}] {}",
            task.id,
            task.js_sim.to_uppercase(),
            task.name
        );
        let hdr_t: String = hdr.chars().take(38).collect();
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            x0 + 8,
            y0 + 8,
            &hdr_t,
            [255, 220, 130, 255],
        );
        let desc = format!("{} (Need: {})", task.desc_text, task.progress_param);
        for (li, line) in wrap_text_lines(&desc, 38, 3).iter().enumerate() {
            draw_bitmap_label(
                &mut buf,
                width,
                height,
                x0 + 8,
                y0 + 24 + li * 12,
                line,
                [210, 230, 215, 255],
            );
        }
    }

    // Section 3: 17 Special Events
    draw_bitmap_label(
        &mut buf,
        width,
        height,
        16,
        374,
        "PRISON SPECIAL EVENTS (CONFIG_S16_GAMEPLAY_EVENT 201..255)",
        [250, 210, 120, 255],
    );
    let ev_cols = 3usize;
    let ev_w = 416usize;
    let ev_h = 94usize;
    for (idx, ev) in db.events.iter().enumerate() {
        let col = idx % ev_cols;
        let row = idx / ev_cols;
        let x0 = 16 + col * ev_w;
        let y0 = 394 + row * ev_h;
        fill_rect(
            &mut buf,
            (width, height),
            (x0, y0, ev_w - 10, ev_h - 8),
            [28, 25, 34, 255],
        );
        let hdr = format!(
            "EVENT #{} [{}] {} (W:{})",
            ev.id,
            ev.js_sim.to_uppercase(),
            ev.name,
            ev.weight
        );
        let hdr_t: String = hdr.chars().take(64).collect();
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            x0 + 10,
            y0 + 8,
            &hdr_t,
            [255, 220, 130, 255],
        );
        for (li, line) in wrap_text_lines(&ev.desc_text, 64, 5).iter().enumerate() {
            draw_bitmap_label(
                &mut buf,
                width,
                height,
                x0 + 10,
                y0 + 23 + li * 12,
                line,
                [215, 225, 235, 255],
            );
        }
    }

    let png = encode_png_rgba8(width as u32, height as u32, &buf)?;
    fs::write(out_path, png).map_err(|e| e.to_string())
}

// ============================================================================
// TrueType (.ufont) Parser & Rasterizer via `ab_glyph`
// ============================================================================

pub struct TtfFont {
    pub font: ab_glyph::FontVec,
    pub units_per_em: u16,
}

pub struct RasterizedGlyph {
    pub cw: usize,
    pub ch_px: usize,
    pub blur: Vec<f32>,
    pub bx0: f64,
    pub by0: f64,
    pub bw: f64,
    pub bh: f64,
}

pub type BoardPoint = (i32, i32, f32);
pub type RoomBox = (u16, u16, u16, u16);
pub type DigitTemplates = [Vec<[f32; 140]>; 10];
pub type PrefixTemplates = Vec<[f32; 400]>;

fn box_blur_3x3(raw: &[f32], cw: usize, ch_px: usize) -> Vec<f32> {
    let mut blur = vec![30.0f32; cw * ch_px];
    for y in 0..ch_px {
        for x in 0..cw {
            let mut s = 0.0f32;
            for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    let yy = (y as isize + dy).clamp(0, ch_px as isize - 1) as usize;
                    let xx = (x as isize + dx).clamp(0, cw as isize - 1) as usize;
                    s += raw[yy * cw + xx];
                }
            }
            blur[y * cw + x] = s / 9.0;
        }
    }
    blur
}

impl TtfFont {
    pub fn parse(data: Vec<u8>) -> Result<Self, String> {
        use ab_glyph::Font;
        let font = ab_glyph::FontVec::try_from_vec(data)
            .map_err(|e| format!("Invalid TrueType font: {e}"))?;
        let units_per_em = font.units_per_em().unwrap_or(1000.0).round() as u16;
        Ok(Self { font, units_per_em })
    }

    /// Rasterizes a single glyph into a padded anti-aliased + `3x3`-box-blurred grayscale buffer,
    /// returning `RasterizedGlyph`.
    pub fn rasterize_char(&self, ch: char, target_h: usize, pad: usize) -> Option<RasterizedGlyph> {
        use ab_glyph::Font;
        let gid = self.font.glyph_id(ch);
        if gid.0 == 0 {
            return None;
        }
        let ref_outlined = self.font.outline_glyph(gid.with_scale(100.0))?;
        let ref_bounds = ref_outlined.px_bounds();
        let ref_h = ref_bounds.height().max(1.0);
        let px_scale = 100.0 * (target_h as f32) / ref_h;
        let outlined = self.font.outline_glyph(gid.with_scale(px_scale))?;
        let bounds = outlined.px_bounds();
        let gw = (bounds.width().ceil() as usize).max(1);
        let gh = target_h;
        let cw = gw + 2 * pad;
        let ch_px = gh + 2 * pad;
        let mut raw = vec![30.0f32; cw * ch_px];

        outlined.draw(|px, py, cov| {
            let x = px as usize + pad;
            let y = py as usize + pad;
            if x < cw && y < ch_px {
                raw[y * cw + x] = 30.0 + 215.0 * cov;
            }
        });
        let blur = box_blur_3x3(&raw, cw, ch_px);
        Some(RasterizedGlyph {
            cw,
            ch_px,
            blur,
            bx0: pad as f64,
            by0: pad as f64,
            bw: bounds.width() as f64,
            bh: gh as f64,
        })
    }

    /// Rasterizes a multi-character text string (such as `"Go to Cell"`) into a padded blurred canvas.
    pub fn rasterize_text(&self, text: &str, scale: f32, pad: usize) -> Option<RasterizedGlyph> {
        use ab_glyph::{Font, ScaleFont, point};
        let px_scale = scale * self.font.height_unscaled();
        let scaled = self.font.as_scaled(px_scale);
        let mut pen_x = 0.0f32;
        let mut outlined_glyphs = Vec::new();
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for ch in text.chars() {
            let gid = scaled.glyph_id(ch);
            let glyph = gid.with_scale_and_position(px_scale, point(pen_x, 0.0));
            if let Some(outlined) = self.font.outline_glyph(glyph) {
                let b = outlined.px_bounds();
                min_x = min_x.min(b.min.x);
                max_x = max_x.max(b.max.x);
                min_y = min_y.min(b.min.y);
                max_y = max_y.max(b.max.y);
                outlined_glyphs.push(outlined);
            }
            pen_x += scaled.h_advance(gid);
        }
        if outlined_glyphs.is_empty() {
            return None;
        }
        let pw = (max_x - min_x).ceil().max(1.0) as usize;
        let ph = (max_y - min_y).ceil().max(1.0) as usize;
        let cw = pw + 2 * pad;
        let ch_px = ph + 2 * pad;
        let mut raw = vec![30.0f32; cw * ch_px];

        for outlined in &outlined_glyphs {
            let b = outlined.px_bounds();
            let ox = (b.min.x - min_x).round() as isize + pad as isize;
            let oy = (b.min.y - min_y).round() as isize + pad as isize;
            outlined.draw(|px, py, cov| {
                let x = ox + px as isize;
                let y = oy + py as isize;
                if x >= 0 && (x as usize) < cw && y >= 0 && (y as usize) < ch_px {
                    let idx = (y as usize) * cw + (x as usize);
                    raw[idx] = raw[idx].max(30.0 + 215.0 * cov);
                }
            });
        }
        let blur = box_blur_3x3(&raw, cw, ch_px);
        Some(RasterizedGlyph {
            cw,
            ch_px,
            blur,
            bx0: pad as f64,
            by0: pad as f64,
            bw: (max_x - min_x) as f64,
            bh: (max_y - min_y) as f64,
        })
    }
}

// ============================================================================
// Pure-.pak CV Feature & Template Synthesis (`crates/tartarus-scan/src/cv_data.rs`)
// ============================================================================

const H0: [f64; 9] = [
    119.44981,
    -9.9567366,
    358.07993,
    -0.26960321,
    90.497559,
    136.75498,
    -0.00012281694,
    -0.015743494,
    1.0,
];

const RING: [(u8, u8); 16] = [
    (0, 0),
    (1, 0),
    (2, 0),
    (3, 0),
    (4, 0),
    (5, 0),
    (5, 1),
    (5, 2),
    (5, 3),
    (4, 3),
    (3, 3),
    (2, 3),
    (1, 3),
    (0, 3),
    (0, 2),
    (0, 1),
];

const S_1344_TO_BG: f64 = 1.9390;
const OX_1344_TO_BG: f64 = 324.0;
const OY_1344_TO_BG: f64 = 240.0;

#[inline]
fn sample_gray_bilinear(g: &[f32], w: usize, h: usize, mut x: f64, mut y: f64) -> f32 {
    x = x.clamp(0.0, (w as f64) - 1.001);
    y = y.clamp(0.0, (h as f64) - 1.001);
    let x0 = x as usize;
    let y0 = y as usize;
    let fx = (x - (x0 as f64)) as f32;
    let fy = (y - (y0 as f64)) as f32;
    let i = y0 * w + x0;
    (g[i] * (1.0 - fx) + g[i + 1] * fx) * (1.0 - fy)
        + (g[i + w] * (1.0 - fx) + g[i + w + 1] * fx) * fy
}

#[inline]
fn normalize_zero_mean(v: &mut [f32]) {
    let n = v.len() as f32;
    let mean = v.iter().sum::<f32>() / n;
    let mut norm = 0.0f32;
    for x in v.iter_mut() {
        *x -= mean;
        norm += *x * *x;
    }
    let inv = 1.0 / norm.sqrt().max(1.0);
    for x in v.iter_mut() {
        *x *= inv;
    }
}

fn composite_rgba_scaled(
    bg: &DecodedAssetImage,
    fg: &DecodedAssetImage,
    cx: isize,
    cy: isize,
    fg_w: usize,
    fg_h: usize,
    dark_bg: (f32, f32, f32),
) -> Vec<u8> {
    let mut out = vec![0u8; bg.width * bg.height * 4];
    for i in 0..(bg.width * bg.height) {
        let o = i * 4;
        let ba = (bg.rgba8[o + 3] as f32) / 255.0;
        out[o] = ((bg.rgba8[o] as f32) * ba + dark_bg.0 * (1.0 - ba)).round() as u8;
        out[o + 1] = ((bg.rgba8[o + 1] as f32) * ba + dark_bg.1 * (1.0 - ba)).round() as u8;
        out[o + 2] = ((bg.rgba8[o + 2] as f32) * ba + dark_bg.2 * (1.0 - ba)).round() as u8;
        out[o + 3] = 255;
    }
    let x0 = cx - (fg_w as isize) / 2;
    let y0 = cy - (fg_h as isize) / 2;
    for dy in 0..fg_h {
        let py = y0 + (dy as isize);
        if py < 0 || py >= bg.height as isize {
            continue;
        }
        let sy = (dy * fg.height) / fg_h;
        for dx in 0..fg_w {
            let px = x0 + (dx as isize);
            if px < 0 || px >= bg.width as isize {
                continue;
            }
            let sx = (dx * fg.width) / fg_w;
            let fi = (sy * fg.width + sx) * 4;
            let fa = (fg.rgba8[fi + 3] as f32) / 255.0;
            if fa > 0.01 {
                let oi = (py as usize * bg.width + px as usize) * 4;
                out[oi] =
                    ((fg.rgba8[fi] as f32) * fa + (out[oi] as f32) * (1.0 - fa)).round() as u8;
                out[oi + 1] = ((fg.rgba8[fi + 1] as f32) * fa + (out[oi + 1] as f32) * (1.0 - fa))
                    .round() as u8;
                out[oi + 2] = ((fg.rgba8[fi + 2] as f32) * fa + (out[oi + 2] as f32) * (1.0 - fa))
                    .round() as u8;
            }
        }
    }
    out
}

fn composite_portrait_on_shield(
    shield: &DecodedAssetImage,
    portrait: &DecodedAssetImage,
    ox: isize,
    oy: isize,
    bg_rgb: (f32, f32, f32),
) -> Vec<f32> {
    let w = shield.width;
    let h = shield.height;
    let mut gray = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let s_idx = (y * w + x) * 4;
            let sa = (shield.rgba8[s_idx + 3] as f32) / 255.0;
            let mut r = (shield.rgba8[s_idx] as f32) * sa + bg_rgb.0 * (1.0 - sa);
            let mut g = (shield.rgba8[s_idx + 1] as f32) * sa + bg_rgb.1 * (1.0 - sa);
            let mut b = (shield.rgba8[s_idx + 2] as f32) * sa + bg_rgb.2 * (1.0 - sa);

            let px = (x as isize) - ox;
            let py = (y as isize) - oy;
            if px >= 0 && px < portrait.width as isize && py >= 0 && py < portrait.height as isize {
                let p_idx = (py as usize * portrait.width + px as usize) * 4;
                let pa = (portrait.rgba8[p_idx + 3] as f32) / 255.0;
                r = (portrait.rgba8[p_idx] as f32) * pa + r * (1.0 - pa);
                g = (portrait.rgba8[p_idx + 1] as f32) * pa + g * (1.0 - pa);
                b = (portrait.rgba8[p_idx + 2] as f32) * pa + b * (1.0 - pa);
            }
            gray[y * w + x] = 0.299 * r + 0.587 * g + 0.114 * b;
        }
    }
    let mut blur = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0f32;
            for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    let yy = (y as isize + dy).clamp(0, h as isize - 1) as usize;
                    let xx = (x as isize + dx).clamp(0, w as isize - 1) as usize;
                    s += gray[yy * w + xx];
                }
            }
            blur[y * w + x] = s / 9.0;
        }
    }
    blur
}

/// Synthesizes the 1002 high-gradient structural board reference points (`REF_BOARD_POINTS`)
/// in `1344x756` `H0` space and the 16 room highlight boxes (`ROOM_BG_BOXES`) in `3200x1920`
/// `UI_Img_S16Gameplay_Jail_Bg_Normal` space directly from `.pak` textures.
pub fn synthesize_board_points_and_rooms(
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(Vec<BoardPoint>, [RoomBox; 16]), String> {
    let bg = textures
        .get("UI_Img_S16Gameplay_Jail_Bg_Normal")
        .ok_or_else(|| "Missing UI_Img_S16Gameplay_Jail_Bg_Normal".to_string())?;
    let mut bg_gray = vec![0.0f32; bg.width * bg.height];
    for (i, g) in bg_gray.iter_mut().enumerate() {
        let o = i * 4;
        *g = 0.299 * (bg.rgba8[o] as f32)
            + 0.587 * (bg.rgba8[o + 1] as f32)
            + 0.114 * (bg.rgba8[o + 2] as f32);
    }

    let mut cell_centers_1344 = Vec::with_capacity(16);
    let mut room_boxes = [(0u16, 0u16, 0u16, 0u16); 16];
    for (idx, &(u, v)) in RING.iter().enumerate() {
        let uf = u as f64;
        let vf = v as f64;
        let w = H0[6] * uf + H0[7] * vf + H0[8];
        let x_1344 = (H0[0] * uf + H0[1] * vf + H0[2]) / w;
        let y_1344 = (H0[3] * uf + H0[4] * vf + H0[5]) / w;
        cell_centers_1344.push((x_1344, y_1344));

        let pass_name = format!("UI_Img_S16Gameplay_Jail_Pass_White_Light{:02}", idx + 1);
        let (pw, ph) = textures
            .get(&pass_name)
            .map(|t| (t.width as f64, t.height as f64))
            .unwrap_or((220.0, 168.0));
        let bx = x_1344 * S_1344_TO_BG + OX_1344_TO_BG;
        let by = y_1344 * S_1344_TO_BG + OY_1344_TO_BG;
        let x0 = (bx - pw * 0.5).round().clamp(0.0, bg.width as f64) as u16;
        let y0 = (by - ph * 0.5).round().clamp(0.0, bg.height as f64) as u16;
        room_boxes[idx] = (x0, y0, pw as u16, ph as u16);
    }

    // Render bg_gray (3200x1920) into 1344x756 reference frame with area averaging,
    // then apply the 3x3 box filter matching `ImageView::sharp_px` / `sample_sharp`.
    let mut ref_1344 = vec![0.0f32; 1344 * 756];
    for ry in 0..756 {
        for rx in 0..1344 {
            let mut acc = 0.0f32;
            for sy in [-0.5f64, 0.0, 0.5] {
                for sx in [-0.5f64, 0.0, 0.5] {
                    let bx = ((rx as f64) + sx) * S_1344_TO_BG + OX_1344_TO_BG;
                    let by = ((ry as f64) + sy) * S_1344_TO_BG + OY_1344_TO_BG;
                    acc += sample_gray_bilinear(&bg_gray, bg.width, bg.height, bx, by);
                }
            }
            ref_1344[ry * 1344 + rx] = acc * (1.0 / 9.0);
        }
    }
    let mut sharp_1344 = vec![0.0f32; 1344 * 756];
    for ry in 1..755 {
        for rx in 1..1343 {
            let mut acc = 0.0f32;
            for dy in 0..3 {
                let row = (ry - 1 + dy) * 1344 + (rx - 1);
                acc += ref_1344[row] + ref_1344[row + 1] + ref_1344[row + 2];
            }
            sharp_1344[ry * 1344 + rx] = acc * (1.0 / 9.0);
        }
    }

    let mut cands: Vec<(i32, i32, f32, f32)> = Vec::new();
    for ry in (226..=284).step_by(2) {
        for rx in (475..=825).step_by(2) {
            let u = rx as usize;
            let v = ry as usize;
            let val = sharp_1344[v * 1344 + u];
            let gx = sharp_1344[v * 1344 + (u + 2)] - sharp_1344[v * 1344 + (u - 2)];
            let gy = sharp_1344[(v + 2) * 1344 + u] - sharp_1344[(v - 2) * 1344 + u];
            let grad = gx.hypot(gy);
            cands.push((rx, ry, (val * 10.0).round() / 10.0, grad));
        }
    }
    cands.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
    let top_1002: Vec<(i32, i32, f32)> = cands
        .into_iter()
        .take(1002)
        .map(|(x, y, v, _)| (x, y, v))
        .collect();
    Ok((top_1002, room_boxes))
}

/// Synthesizes the `18x22 = 396` beast face templates for a beast from its normal (`112x120`)
/// and jailed (`120x144`) portraits composited onto the `120x144` cell shield (`UI_T_V2_S16BoardCell_046`).
pub fn synthesize_beast_face_templates(
    shield: &DecodedAssetImage,
    norm_tex: &DecodedAssetImage,
    jail_tex: &DecodedAssetImage,
) -> Vec<[f32; 396]> {
    let mut out = Vec::with_capacity(54);
    for (tex, ox, oy) in [(norm_tex, 4isize, 0isize), (jail_tex, 0isize, 0isize)] {
        let blur = composite_portrait_on_shield(shield, tex, ox, oy, (28.0, 26.0, 32.0));
        for &(cw, ch) in &[(74.0f64, 88.0f64), (82.0, 96.0), (90.0, 104.0)] {
            for &cy in &[55.0f64, 59.0, 63.0] {
                for &cx in &[58.0f64, 60.0, 62.0] {
                    let x0 = cx - cw * 0.5;
                    let y0 = cy - ch * 0.5;
                    let mut tv = [0.0f32; 396];
                    for j in 0..22 {
                        for i in 0..18 {
                            let sx = x0 + cw * ((i as f64) + 0.5) / 18.0;
                            let sy = y0 + ch * ((j as f64) + 0.5) / 22.0;
                            tv[j * 18 + i] = sample_gray_bilinear(&blur, 120, 144, sx, sy);
                        }
                    }
                    normalize_zero_mean(&mut tv);
                    out.push(tv);
                }
            }
        }
    }
    out
}

/// Synthesizes `DIGITS` (`0..9`, `10x14 = 140`) and `PREFIX_GLYPHS` (`"Go to Cell"`, `40x10 = 400`)
/// directly from `UI_Font_Light_EN_AZ.ufont` and `tl_main.ufont` in the `.pak` archives.
pub fn synthesize_digits_and_prefix(
    vfs: &PakVfs,
) -> Result<(DigitTemplates, PrefixTemplates), String> {
    let f1_bytes = vfs.read_path("UE_game/Content/Art/UI/Fonts/UI_Font_Light_EN_AZ.ufont")?;
    let f2_bytes = vfs.read_path("UE_game/Content/HotUpdate/Fonts/tl_main.ufont")?;
    let fonts = [TtfFont::parse(f1_bytes)?, TtfFont::parse(f2_bytes)?];

    let mut digits: DigitTemplates = Default::default();
    for (d, digit_vecs) in digits.iter_mut().enumerate() {
        let ch = char::from_digit(d as u32, 10).unwrap();
        for font in &fonts {
            let Some(RasterizedGlyph {
                cw,
                ch_px,
                blur,
                bx0,
                by0,
                bw: gw,
                bh: gh,
            }) = font.rasterize_char(ch, 24, 16)
            else {
                continue;
            };
            let gcx = bx0 + gw * 0.5;
            let gcy = by0 + gh * 0.5;
            for &sy_f in &[0.96f64, 1.00, 1.04] {
                let sy_span = gh * sy_f;
                let sx_span = sy_span * 10.0 / 14.0;
                for &dy_f in &[-0.03f64, 0.0, 0.03] {
                    for &dx_f in &[-0.03f64, 0.0, 0.03] {
                        let cx = gcx + dx_f * sx_span;
                        let cy = gcy + dy_f * sy_span;
                        let mut tv = [0.0f32; 140];
                        for j in 0..14 {
                            for i in 0..10 {
                                let sx = cx + (((i as f64) + 0.5) / 10.0 - 0.5) * sx_span;
                                let sy = cy + (((j as f64) + 0.5) / 14.0 - 0.5) * sy_span;
                                tv[j * 10 + i] = sample_gray_bilinear(&blur, cw, ch_px, sx, sy);
                            }
                        }
                        normalize_zero_mean(&mut tv);
                        digit_vecs.push(tv);
                    }
                }
            }
        }
    }

    let mut prefixes = Vec::new();
    for font in &fonts {
        let scale = 22.0 / (font.units_per_em as f32);
        let Some(RasterizedGlyph {
            cw,
            ch_px,
            blur,
            bx0,
            by0,
            bw: pw,
            bh: ph,
        }) = font.rasterize_text("Go to Cell", scale, 16)
        else {
            continue;
        };
        let pcx = bx0 + pw * 0.5;
        let pcy = by0 + ph * 0.5;
        for &sx_f in &[0.96f64, 1.00, 1.04] {
            for &sy_f in &[0.96f64, 1.02, 1.08] {
                for &dy_f in &[-0.04f64, 0.0, 0.04] {
                    let sx_span = pw * sx_f;
                    let sy_span = ph * sy_f;
                    let cy = pcy + dy_f * sy_span;
                    let mut tv = [0.0f32; 400];
                    for j in 0..10 {
                        for i in 0..40 {
                            let sx = pcx + (((i as f64) + 0.5) / 40.0 - 0.5) * sx_span;
                            let sy = cy + (((j as f64) + 0.5) / 10.0 - 0.5) * sy_span;
                            tv[j * 40 + i] = sample_gray_bilinear(&blur, cw, ch_px, sx, sy);
                        }
                    }
                    normalize_zero_mean(&mut tv);
                    prefixes.push(tv);
                }
            }
        }
    }

    Ok((digits, prefixes))
}

/// Synthesizes `24x24 = 576` zero-mean unit-norm grayscale templates for the 8 route/action
/// card medallions (`UI_S16Gameplay_Route_01..02` and `UI_S16Gameplay_Action_01..06`).
pub fn synthesize_card_medallions(
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Vec<(&'static str, &'static str, [f32; 576])> {
    let specs: [(&str, &str); 8] = [
        ("cell_door", "UI_S16Gameplay_Route_01"),
        ("warden_office", "UI_S16Gameplay_Route_02"),
        ("devour", "UI_S16Gameplay_Action_01"),
        ("swap", "UI_S16Gameplay_Action_02"),
        ("hunt", "UI_S16Gameplay_Action_03"),
        ("end_turn", "UI_S16Gameplay_Action_04"),
        ("incident", "UI_S16Gameplay_Action_05"),
        ("bestow", "UI_S16Gameplay_Action_06"),
    ];
    let mut out = Vec::with_capacity(8);
    for (id, asset) in specs {
        let mut grid = [0.0f32; 576];
        if let Some(tex) = textures.get(asset) {
            let mut gray = vec![0.0f32; tex.width * tex.height];
            for (i, g_val) in gray.iter_mut().enumerate() {
                let o = i * 4;
                let a = (tex.rgba8[o + 3] as f32) / 255.0;
                let r = (tex.rgba8[o] as f32) * a + 28.0 * (1.0 - a);
                let g = (tex.rgba8[o + 1] as f32) * a + 26.0 * (1.0 - a);
                let b = (tex.rgba8[o + 2] as f32) * a + 30.0 * (1.0 - a);
                *g_val = 0.299 * r + 0.587 * g + 0.114 * b;
            }
            let (x0, y0, x1, y1) = alpha_bbox(&tex.rgba8, tex.width, tex.height);
            let bw = (x1 - x0).max(1) as f64;
            let bh = (y1 - y0).max(1) as f64;
            for j in 0..24 {
                for i in 0..24 {
                    let sx = (x0 as f64) + bw * ((i as f64) + 0.5) / 24.0;
                    let sy = (y0 as f64) + bh * ((j as f64) + 0.5) / 24.0;
                    grid[j * 24 + i] = sample_gray_bilinear(&gray, tex.width, tex.height, sx, sy);
                }
            }
            normalize_zero_mean(&mut grid);
        }
        out.push((id, asset, grid));
    }
    out
}

/// Renders `catalog_board_layout.png`: visual verification of the `.pak`-synthesized board
/// points, 16 room highlight polygons, TrueType font digits/prefix, and route/action medallions.
pub fn render_catalog_board_layout_png(
    out_path: &Path,
    vfs: &PakVfs,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), String> {
    let width = 1600usize;
    let height = 1120usize;
    let mut buf = vec![0u8; width * height * 4];
    for px in buf.as_chunks_mut::<4>().0 {
        px.copy_from_slice(&[18, 16, 22, 255]);
    }

    draw_bitmap_label(
        &mut buf,
        width,
        height,
        16,
        14,
        "S16 PAK-SYNTHESIZED CV ASSETS: BOARD ARCHITECTURE (3200X1920), ROOMS (01..16), TRUETYPE GLYPHS & MEDALLIONS",
        [250, 210, 120, 255],
    );

    if let Some(bg) = textures.get("UI_Img_S16Gameplay_Jail_Bg_Normal") {
        blit_scaled(&mut buf, (width, height), (16, 36, 1280, 768), bg);
    }

    let (pts, room_boxes) = synthesize_board_points_and_rooms(textures)?;
    // Draw the 16 room boxes in cyan outline and room numbers
    for (idx, &(rx, ry, rw, rh)) in room_boxes.iter().enumerate() {
        let x0 = 16 + (rx as usize * 1280) / 3200;
        let y0 = 36 + (ry as usize * 768) / 1920;
        let w = (rw as usize * 1280) / 3200;
        let h = (rh as usize * 768) / 1920;
        fill_rect(
            &mut buf,
            (width, height),
            (x0, y0, w, 2),
            [80, 220, 255, 255],
        );
        fill_rect(
            &mut buf,
            (width, height),
            (x0, y0 + h, w, 2),
            [80, 220, 255, 255],
        );
        fill_rect(
            &mut buf,
            (width, height),
            (x0, y0, 2, h),
            [80, 220, 255, 255],
        );
        fill_rect(
            &mut buf,
            (width, height),
            (x0 + w, y0, 2, h),
            [80, 220, 255, 255],
        );
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            x0 + 6,
            y0 + 6,
            &format!("CELL {:02}", idx + 1),
            [255, 235, 120, 255],
        );
    }

    // Draw the 1002 high-gradient structural board points in bright gold/green dots
    for &(rx, ry, _) in &pts {
        let bx = (rx as f64) * S_1344_TO_BG + OX_1344_TO_BG;
        let by = (ry as f64) * S_1344_TO_BG + OY_1344_TO_BG;
        let px = 16 + ((bx * 1280.0 / 3200.0).round() as usize);
        let py = 36 + ((by * 768.0 / 1920.0).round() as usize);
        fill_rect(
            &mut buf,
            (width, height),
            (px.saturating_sub(1), py.saturating_sub(1), 3, 3),
            [90, 255, 130, 255],
        );
    }

    // Right sidebar: 8 Route & Action Medallions
    draw_bitmap_label(
        &mut buf,
        width,
        height,
        1316,
        40,
        "ROUTE & ACTION SPRITES",
        [250, 210, 120, 255],
    );
    let specs: [(&str, &str); 8] = [
        ("CELL DOOR", "UI_S16Gameplay_Route_01"),
        ("WARDEN OFFICE", "UI_S16Gameplay_Route_02"),
        ("CONSUME", "UI_S16Gameplay_Action_01"),
        ("REPLACE", "UI_S16Gameplay_Action_02"),
        ("HUNT", "UI_S16Gameplay_Action_03"),
        ("END TURN", "UI_S16Gameplay_Action_04"),
        ("INCIDENT", "UI_S16Gameplay_Action_05"),
        ("BESTOW", "UI_S16Gameplay_Action_06"),
    ];
    for (i, (label, asset)) in specs.iter().enumerate() {
        let y0 = 64 + i * 92;
        fill_rect(
            &mut buf,
            (width, height),
            (1316, y0, 268, 84),
            [28, 25, 34, 255],
        );
        if let Some(tex) = textures.get(*asset) {
            blit_scaled(&mut buf, (width, height), (1324, y0 + 10, 64, 64), tex);
        }
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            1398,
            y0 + 22,
            label,
            [255, 220, 130, 255],
        );
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            1398,
            y0 + 40,
            asset.trim_start_matches("UI_S16Gameplay_"),
            [170, 215, 250, 255],
        );
    }

    // Bottom bar: TrueType Font Rasterization Preview (UI_Font_Light_EN_AZ.ufont)
    draw_bitmap_label(
        &mut buf,
        width,
        height,
        16,
        824,
        "TRUETYPE FONT RASTERIZATION (UI_FONT_LIGHT_EN_AZ.UFONT): DIGITS 0..9 (10X14) & 'GO TO CELL' (40X10)",
        [250, 210, 120, 255],
    );
    if let Ok(f1_bytes) = vfs.read_path("UE_game/Content/Art/UI/Fonts/UI_Font_Light_EN_AZ.ufont")
        && let Ok(font) = TtfFont::parse(f1_bytes)
    {
        for d in 0..=9 {
            let ch = char::from_digit(d as u32, 10).unwrap();
            let x0 = 16 + d * 96;
            let y0 = 848;
            fill_rect(
                &mut buf,
                (width, height),
                (x0, y0, 86, 120),
                [28, 25, 34, 255],
            );
            draw_bitmap_label(
                &mut buf,
                width,
                height,
                x0 + 10,
                y0 + 8,
                &format!("DIGIT {d}"),
                [255, 220, 130, 255],
            );
            if let Some(RasterizedGlyph {
                cw, ch_px, blur, ..
            }) = font.rasterize_char(ch, 48, 8)
            {
                for py in 0..ch_px.min(84) {
                    for px in 0..cw.min(72) {
                        let v = blur[py * cw + px].clamp(0.0, 255.0) as u8;
                        let idx = ((y0 + 26 + py) * width + (x0 + 12 + px)) * 4;
                        buf[idx..idx + 4].copy_from_slice(&[v, v, v, 255]);
                    }
                }
            }
        }
        let x0 = 990;
        let y0 = 848;
        fill_rect(
            &mut buf,
            (width, height),
            (x0, y0, 594, 120),
            [28, 25, 34, 255],
        );
        draw_bitmap_label(
            &mut buf,
            width,
            height,
            x0 + 12,
            y0 + 8,
            "PREFIX: 'GO TO CELL'",
            [255, 220, 130, 255],
        );
        let scale = 42.0 / (font.units_per_em as f32);
        if let Some(RasterizedGlyph {
            cw, ch_px, blur, ..
        }) = font.rasterize_text("Go to Cell", scale, 10)
        {
            for py in 0..ch_px.min(84) {
                for px in 0..cw.min(560) {
                    let v = blur[py * cw + px].clamp(0.0, 255.0) as u8;
                    let idx = ((y0 + 26 + py) * width + (x0 + 16 + px)) * 4;
                    buf[idx..idx + 4].copy_from_slice(&[v, v, v, 255]);
                }
            }
        }
    }

    let png = encode_png_rgba8(width as u32, height as u32, &buf)?;
    fs::write(out_path, png).map_err(|e| e.to_string())
}

fn format_f32_slice(arr: &[f32]) -> String {
    let mut s = String::from("[");
    for (i, &v) in arr.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&format!("{:.6}_f32", v));
    }
    s.push(']');
    s
}

fn sim_str_to_tool_id_expr(sim: &str) -> &'static str {
    match sim {
        "muzzle" => "ToolId::Muzzle",
        "ravenous" => "ToolId::Ravenous",
        "collar" => "ToolId::Collar",
        "revenge" => "ToolId::Revenge",
        "cage" => "ToolId::Cage",
        "whip" => "ToolId::Whip",
        "saw" => "ToolId::Saw",
        "blacklist" => "ToolId::Blacklist",
        "keys" => "ToolId::Keys",
        "coin" => "ToolId::Coin",
        "fire" => "ToolId::Fire",
        "dogtag" => "ToolId::Dogtag",
        "forged" => "ToolId::Forged",
        "hook" => "ToolId::Hook",
        "scapegoat" => "ToolId::Scapegoat",
        "axe" => "ToolId::Axe",
        "maiden" => "ToolId::Maiden",
        "scales" => "ToolId::Scales",
        "rope" => "ToolId::Rope",
        "stake" => "ToolId::Stake",
        "pestle" => "ToolId::Pestle",
        "shortchain" => "ToolId::Shortchain",
        "shackles" => "ToolId::Shackles",
        "tongs" => "ToolId::Tongs",
        "maul" => "ToolId::Maul",
        "trap" => "ToolId::Trap",
        "crown" => "ToolId::Crown",
        "claws" => "ToolId::Claws",
        "guillotine" => "ToolId::Guillotine",
        "ironbox" => "ToolId::Ironbox",
        "skewer" => "ToolId::Skewer",
        _ => unreachable!("unknown active relic sim key"),
    }
}

/// Generates the self-contained Rust source file `crates/tartarus-scan/src/cv_data.rs`
/// 100% from the `.pak` archives (`PakVfs`), with zero `.js` dependencies.
pub fn generate_cv_data_rs(
    out_rs_path: &Path,
    vfs: &PakVfs,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), String> {
    let (board_pts, room_boxes) = synthesize_board_points_and_rooms(textures)?;
    let (digits, prefixes) = synthesize_digits_and_prefix(vfs)?;
    let medallions = synthesize_card_medallions(textures);

    let shield = textures
        .get("UI_T_V2_S16BoardCell_046")
        .ok_or_else(|| "Missing UI_T_V2_S16BoardCell_046".to_string())?;

    let mut out = String::with_capacity(640 * 1024);
    out.push_str("// @generated by `cargo run -p tartarus-data -- --emit-rust crates/tartarus-scan/src/cv_data.rs`.\n");
    out.push_str("// Authoritative S16 Tartarus CV templates and official game metadata synthesized 100% from .pak assets.\n\n");

    // 1. Official S16 Relic Metadata table (all 31 active relics)
    out.push_str("#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct S16RelicMeta {\n");
    out.push_str("    pub game_id: u16,\n");
    out.push_str("    pub sim: ToolId,\n");
    out.push_str("    pub icon_asset: &'static str,\n");
    out.push_str("    pub name: &'static str,\n");
    out.push_str("    pub trigger: &'static str,\n");
    out.push_str("    pub effect: &'static str,\n");
    out.push_str("}\n\n");

    out.push_str(&format!(
        "pub static S16_RELIC_META: [S16RelicMeta; {}] = [\n",
        db.relics.len()
    ));
    for r in &db.relics {
        out.push_str(&format!(
            "    S16RelicMeta {{ game_id: {}, sim: {}, icon_asset: {:?}, name: {:?}, trigger: {:?}, effect: {:?} }},\n",
            r.id,
            sim_str_to_tool_id_expr(&r.js_sim),
            r.icon_asset,
            r.name,
            r.trigger_text,
            r.effect_text
        ));
    }
    out.push_str("];\n\n");

    // 2. Board points & 16 Room highlight boxes from UI_Img_S16Gameplay_Jail_Bg_Normal + Pass_White_Light01..16
    out.push_str("pub const REF_BOARD_WIDTH: f32 = 755.0;\n");
    out.push_str("pub const REF_BOARD_HEIGHT: f32 = 424.0;\n");
    out.push_str("pub static ROOM_BG_BOXES: [(u16, u16, u16, u16); 16] = [\n");
    for &(x, y, w, h) in &room_boxes {
        out.push_str(&format!("    ({x}, {y}, {w}, {h}),\n"));
    }
    out.push_str("];\n\n");

    out.push_str(&format!(
        "pub static REF_BOARD_POINTS: [(i32, i32, f32); {}] = [\n",
        board_pts.len()
    ));
    for &(rx, ry, val) in &board_pts {
        out.push_str(&format!("    ({rx}, {ry}, {val:.1}_f32),\n"));
    }
    out.push_str("];\n\n");

    // 3. Beast Faces (18x22 = 396) synthesized from UI_T_V2_S16BoardCell_101..104 & 045/047/050/048 on 046
    for (const_name, norm_name, jail_name) in [
        (
            "FACES_LION",
            "UI_T_V2_S16BoardCell_102",
            "UI_T_V2_S16BoardCell_047",
        ),
        (
            "FACES_MAMMOTH",
            "UI_T_V2_S16BoardCell_101",
            "UI_T_V2_S16BoardCell_045",
        ),
        (
            "FACES_RAT",
            "UI_T_V2_S16BoardCell_104",
            "UI_T_V2_S16BoardCell_048",
        ),
        (
            "FACES_WOLF",
            "UI_T_V2_S16BoardCell_103",
            "UI_T_V2_S16BoardCell_050",
        ),
    ] {
        let norm_tex = textures
            .get(norm_name)
            .ok_or_else(|| format!("Missing {norm_name}"))?;
        let jail_tex = textures
            .get(jail_name)
            .ok_or_else(|| format!("Missing {jail_name}"))?;
        let vecs = synthesize_beast_face_templates(shield, norm_tex, jail_tex);
        out.push_str(&format!(
            "pub static {}: [[f32; 396]; {}] = [\n",
            const_name,
            vecs.len()
        ));
        for v in &vecs {
            out.push_str("    ");
            out.push_str(&format_f32_slice(v));
            out.push_str(",\n");
        }
        out.push_str("];\n\n");
    }

    // 4. Digits 0..9 (10x14 = 140) from UI_Font_Light_EN_AZ.ufont & tl_main.ufont
    for (d, vecs) in digits.iter().enumerate() {
        out.push_str(&format!(
            "pub static DIGIT_{}: [[f32; 140]; {}] = [\n",
            d,
            vecs.len()
        ));
        for v in vecs {
            out.push_str("    ");
            out.push_str(&format_f32_slice(v));
            out.push_str(",\n");
        }
        out.push_str("];\n");
    }
    out.push_str("pub static DIGITS: [&[[f32; 140]]; 10] = [\n");
    for d in 0..=9 {
        out.push_str(&format!("    &DIGIT_{},\n", d));
    }
    out.push_str("];\n\n");

    // 5. Prefix glyphs ("Go to Cell", 40x10 = 400)
    out.push_str("pub const PREFIX_DIM: usize = 400;\n");
    out.push_str(&format!(
        "pub static PREFIX_GLYPHS: [[f32; 400]; {}] = [\n",
        prefixes.len()
    ));
    for v in &prefixes {
        out.push_str("    ");
        out.push_str(&format_f32_slice(v));
        out.push_str(",\n");
    }
    out.push_str("];\n\n");

    // 6. Route & Action Card Medallions (24x24 = 576)
    out.push_str("#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct CardMedallionTemplate {\n");
    out.push_str("    pub id: &'static str,\n");
    out.push_str("    pub asset: &'static str,\n");
    out.push_str("    pub grid: [f32; 576],\n");
    out.push_str("}\n\n");
    out.push_str(&format!(
        "pub static CARD_MEDALLIONS: [CardMedallionTemplate; {}] = [\n",
        medallions.len()
    ));
    for (id, asset, grid) in &medallions {
        out.push_str(&format!(
            "    CardMedallionTemplate {{ id: {:?}, asset: {:?}, grid: {} }},\n",
            id,
            asset,
            format_f32_slice(grid)
        ));
    }
    out.push_str("];\n\n");

    // 7. Synthesized REF_TOOLS and REF_PICK_TOOLS for all 31 active S16 Relics
    let item_bg = textures.get("UI_Img_S16Gameplay_Jail_Item_Bg");
    let big_pics = textures.get("UI_Btn_S16Gameplay_Jail_Device_Big_Pics");

    struct ToolFeatureRow {
        id: String,
        sim: String,
        name: String,
        is_panel: bool,
        grid: [f32; 72],
        hist: [f32; 13],
    }

    let mut ref_tools_rows: Vec<ToolFeatureRow> = Vec::new();
    let mut ref_pick_rows: Vec<(String, [f32; 72], [f32; 13])> = Vec::new();

    for r in &db.relics {
        let Some(icon) = textures.get(&r.icon_asset) else {
            continue;
        };
        let (x0, y0, x1, y1) = alpha_bbox(&icon.rgba8, icon.width, icon.height);
        let bw = (x1 - x0) as isize;
        let bh = (y1 - y0) as isize;

        // Panel features: alpha-bbox variants + composited on UI_Img_S16Gameplay_Jail_Item_Bg (104x172)
        for &pad_f in &[-0.06f32, 0.0, 0.08] {
            let px = ((bw as f32) * pad_f).round() as isize;
            let py = ((bh as f32) * pad_f).round() as isize;
            let cx0 = (x0 as isize - px).clamp(0, icon.width as isize - 1) as usize;
            let cy0 = (y0 as isize - py).clamp(0, icon.height as isize - 1) as usize;
            let cx1 = (x1 as isize + px).clamp((cx0 + 1) as isize, icon.width as isize) as usize;
            let cy1 = (y1 as isize + py).clamp((cy0 + 1) as isize, icon.height as isize) as usize;
            let (grid, hist) = compute_art_feature_region(
                &icon.rgba8,
                (icon.width, icon.height),
                (cx0, cy0, cx1, cy1),
                (18.0, 16.0, 20.0),
            );
            ref_tools_rows.push(ToolFeatureRow {
                id: r.js_sim.clone(),
                sim: r.js_sim.clone(),
                name: r.name.clone(),
                is_panel: true,
                grid,
                hist,
            });
        }
        if let Some(bg) = item_bg {
            for &s in &[82usize, 92, 100] {
                let comp = composite_rgba_scaled(bg, icon, 52, 86, s, s, (16.0, 14.0, 18.0));
                let (grid, hist) = compute_art_feature_region(
                    &comp,
                    (bg.width, bg.height),
                    (8, 36, bg.width - 8, bg.height - 36),
                    (16.0, 14.0, 18.0),
                );
                ref_tools_rows.push(ToolFeatureRow {
                    id: r.js_sim.clone(),
                    sim: r.js_sim.clone(),
                    name: r.name.clone(),
                    is_panel: true,
                    grid,
                    hist,
                });
            }
        }

        // Non-panel entry in REF_TOOLS + entries in REF_PICK_TOOLS using exact pick_art_box 1%-99% percentile crop
        let (p_grid, p_hist) = synthesize_icon_feature(icon, (28.0, 22.0, 20.0));
        ref_tools_rows.push(ToolFeatureRow {
            id: r.js_sim.clone(),
            sim: r.js_sim.clone(),
            name: r.name.clone(),
            is_panel: false,
            grid: p_grid,
            hist: p_hist,
        });
        for &alpha_thr in &[16u8, 48, 84] {
            let (ax0, ay0, ax1, ay1) =
                alpha_bbox_thresh(&icon.rgba8, icon.width, icon.height, alpha_thr);
            let (grid, hist) = compute_art_feature_region(
                &icon.rgba8,
                (icon.width, icon.height),
                (ax0, ay0, ax1, ay1),
                (26.0, 21.0, 19.0),
            );
            ref_pick_rows.push((r.js_sim.clone(), grid, hist));
        }

        if let Some(bg) = big_pics {
            for &s in &[250usize, 285, 315] {
                let comp = composite_rgba_scaled(bg, icon, 250, 260, s, s, (22.0, 20.0, 24.0));
                for &(mx_hi, v_thr) in &[(72u8, 0.28f32), (82, 0.32), (94, 0.36)] {
                    if let Some((bx0, by0, bx1, by1)) = pick_art_bbox_on_rgba(
                        &comp,
                        (bg.width, bg.height),
                        (250 - 110, 130, 250 + 110, 395),
                        mx_hi,
                        v_thr,
                    ) {
                        let (grid, hist) = compute_art_feature_region(
                            &comp,
                            (bg.width, bg.height),
                            (bx0, by0, bx1, by1),
                            (22.0, 20.0, 24.0),
                        );
                        ref_pick_rows.push((r.js_sim.clone(), grid, hist));
                    }
                }
            }
        }
    }

    out.push_str("#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct RefToolEntry {\n");
    out.push_str("    pub id: &'static str,\n");
    out.push_str("    pub sim: ToolId,\n");
    out.push_str("    pub name: &'static str,\n");
    out.push_str("    pub is_panel: bool,\n");
    out.push_str("    pub grid: [f32; 72],\n");
    out.push_str("    pub hist: [f32; 13],\n");
    out.push_str("}\n\n");

    out.push_str(&format!(
        "pub static REF_TOOLS: [RefToolEntry; {}] = [\n",
        ref_tools_rows.len()
    ));
    for row in &ref_tools_rows {
        out.push_str(&format!(
            "    RefToolEntry {{ id: {:?}, sim: {}, name: {:?}, is_panel: {}, grid: {}, hist: {} }},\n",
            row.id,
            sim_str_to_tool_id_expr(&row.sim),
            row.name,
            row.is_panel,
            format_f32_slice(&row.grid),
            format_f32_slice(&row.hist)
        ));
    }
    out.push_str("];\n\n");

    out.push_str("#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct RefPickToolEntry {\n");
    out.push_str("    pub sim: ToolId,\n");
    out.push_str("    pub grid: [f32; 72],\n");
    out.push_str("    pub hist: [f32; 13],\n");
    out.push_str("}\n\n");

    out.push_str(&format!(
        "pub static REF_PICK_TOOLS: [RefPickToolEntry; {}] = [\n",
        ref_pick_rows.len()
    ));
    for (sim, grid, hist) in &ref_pick_rows {
        out.push_str(&format!(
            "    RefPickToolEntry {{ sim: {}, grid: {}, hist: {} }},\n",
            sim_str_to_tool_id_expr(sim),
            format_f32_slice(grid),
            format_f32_slice(hist)
        ));
    }
    out.push_str("];\n");

    fs::write(out_rs_path, out)
        .map_err(|e| format!("Failed to write {}: {e}", out_rs_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_rich_text_and_glyphs() {
        let raw = "Requires <Red>1</> fewer beast; grants +50% 'Rainbow' [145]!";
        let clean = strip_rich_text(raw);
        assert_eq!(
            clean,
            "Requires 1 fewer beast; grants +50% 'Rainbow' [145]!"
        );
        for ch in ['#', '\'', '+', '%', ';', '!', '?', '"', '[', ']'] {
            let mut buf = [0u8; 8 * 8 * 4];
            draw_bitmap_label(&mut buf, 8, 8, 0, 0, &ch.to_string(), [255, 255, 255, 255]);
            assert!(
                buf.iter().any(|&b| b != 0),
                "Glyph for {ch:?} should not be blank"
            );
        }
    }

    #[test]
    fn test_all_31_active_relics_have_unique_sim_keys() {
        let mut sims = std::collections::BTreeSet::new();
        for id in [
            101, 102, 103, 104, 105, 106, 107, 111, 112, 113, 114, 115, 116, 117, 121, 122, 123,
            124, 125, 126, 127, 131, 132, 141, 142, 143, 144, 145, 146, 147, 148,
        ] {
            let sim = map_relic_id_to_sim(id);
            assert_ne!(
                sim, "unknown",
                "Active relic {id} must map to a known sim key"
            );
            assert!(sims.insert(sim), "Duplicate sim key {sim} for relic {id}");
        }
        assert_eq!(sims.len(), 31);
    }

    #[test]
    fn test_all_tasks_and_events_have_unique_sim_keys() {
        let mut tasks = std::collections::BTreeSet::new();
        for id in 1..=5 {
            let sim = map_task_id_to_sim(id);
            assert_ne!(sim, "other", "Task {id} must not map to 'other'");
            assert!(
                tasks.insert(sim),
                "Duplicate task sim key {sim} for task {id}"
            );
        }
        assert_eq!(tasks.len(), 5);
        assert_eq!(map_task_id_to_sim(4), "incident");

        let mut events = std::collections::BTreeSet::new();
        for id in [
            201, 202, 203, 211, 212, 213, 221, 222, 231, 232, 241, 242, 251, 252, 253, 254, 255,
        ] {
            let sim = map_event_id_to_sim(id);
            assert_ne!(sim, "unknown", "Event {id} must map to a known sim key");
            assert!(
                events.insert(sim),
                "Duplicate event sim key {sim} for event {id}"
            );
        }
        assert_eq!(events.len(), 17);
    }

    #[test]
    fn test_cv_data_uses_tool_id_enum() {
        let cv_data_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../tartarus-scan/src/cv_data.rs");
        let content = fs::read_to_string(&cv_data_path).expect("read cv_data.rs");
        assert!(!content.contains("pub sim: &'static str,"));
        assert!(content.contains("pub sim: ToolId,"));
    }
}

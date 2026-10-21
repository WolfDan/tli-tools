//! Pure-`.pak` UMG layout & SIMD template synthesis (`crates/tartarus-scan/src/ui_data.rs`).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tartarus_extract::{PakVfs, UmgAnchorData};

use crate::assets::{
    DecodedAssetImage, alpha_bbox, alpha_bbox_thresh, compute_art_feature_region,
    pick_art_bbox_on_rgba, synthesize_icon_feature,
};
use crate::db::{S16GameDatabase, S16GiftEntry, S16RelicEntry};
use crate::error::DataError;
use crate::font::{BoardPoint, DigitTemplates, PrefixTemplates, RasterizedGlyph, RoomBox, TtfFont};

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

pub(crate) const S_1344_TO_BG: f64 = 1.9390;
pub(crate) const OX_1344_TO_BG: f64 = 324.0;
pub(crate) const OY_1344_TO_BG: f64 = 240.0;

const _: () = {
    assert!(H0.len() == 9);
    assert!(RING.len() == 16);
};

#[inline]
fn sample_gray_bilinear(g: &[f32], w: usize, h: usize, mut x: f64, mut y: f64) -> f32 {
    debug_assert!(w >= 2 && h >= 2);
    debug_assert!(g.len() >= w * h);
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
    assert!(!v.is_empty());
    let n = v.len() as f32;
    let mean = v.iter().sum::<f32>() / n;
    let mut norm = 0.0f32;
    for x in v.iter_mut() {
        *x -= mean;
        norm += *x * *x;
    }
    assert!(norm >= 0.0);
    let inv = 1.0 / norm.sqrt().max(1.0);
    for x in v.iter_mut() {
        *x *= inv;
    }
}

fn composite_rgba_scaled(
    bg: &DecodedAssetImage,
    fg: &DecodedAssetImage,
    (cx, cy): (isize, isize),
    (fg_w, fg_h): (usize, usize),
    dark_bg: (f32, f32, f32),
) -> Vec<u8> {
    assert!(bg.width > 0 && bg.height > 0);
    assert!(fg_w > 0 && fg_h > 0);
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
    assert!(w > 0 && h > 0);
    assert!(portrait.width > 0 && portrait.height > 0);
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

fn compute_sharp_1344_from_bg(bg: &DecodedAssetImage) -> Vec<f32> {
    assert!(bg.width >= 2 && bg.height >= 2);
    let mut bg_gray = vec![0.0f32; bg.width * bg.height];
    for (i, g) in bg_gray.iter_mut().enumerate() {
        let o = i * 4;
        *g = 0.299 * (bg.rgba8[o] as f32)
            + 0.587 * (bg.rgba8[o + 1] as f32)
            + 0.114 * (bg.rgba8[o + 2] as f32);
    }

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
    assert_eq!(sharp_1344.len(), 1344 * 756);
    sharp_1344
}

/// Synthesizes the 1008 high-gradient structural board reference points (`REF_BOARD_POINTS`,
/// multiple of 16 for `f32x16` SIMD!) in `1344x756` `H0` space and the 16 room highlight boxes
/// (`ROOM_BG_BOXES`) in `3200x1920` `UI_Img_S16Gameplay_Jail_Bg_Normal` space directly from `.pak`.
pub fn synthesize_board_points_and_rooms(
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(Vec<BoardPoint>, [RoomBox; 16]), DataError> {
    assert!(!textures.is_empty());
    let bg = textures
        .get("UI_Img_S16Gameplay_Jail_Bg_Normal")
        .ok_or_else(|| DataError::MissingAsset("UI_Img_S16Gameplay_Jail_Bg_Normal".to_string()))?;

    let mut room_boxes = [(0u16, 0u16, 0u16, 0u16); 16];
    for (idx, &(u, v)) in RING.iter().enumerate() {
        let uf = u as f64;
        let vf = v as f64;
        let w = H0[6] * uf + H0[7] * vf + H0[8];
        let x_1344 = (H0[0] * uf + H0[1] * vf + H0[2]) / w;
        let y_1344 = (H0[3] * uf + H0[4] * vf + H0[5]) / w;

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

    let sharp_1344 = compute_sharp_1344_from_bg(bg);
    let mut cands: Vec<(i32, i32, f32, f32)> = Vec::new();
    for ry in (226..=284).step_by(2) {
        for rx in (475..=825).step_by(2) {
            let u = rx as usize;
            let v = ry as usize;
            let val = sharp_1344[v * 1344 + u];
            let gx = sharp_1344[v * 1344 + (u + 2)] - sharp_1344[v * 1344 + (u - 2)];
            let gy = sharp_1344[(v + 2) * 1344 + u] - sharp_1344[(v - 2) * 1344 + u];
            cands.push((rx, ry, (val * 10.0).round() / 10.0, gx.hypot(gy)));
        }
    }
    cands.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
    let top_pts: Vec<(i32, i32, f32)> = cands
        .into_iter()
        .take(1008)
        .map(|(x, y, v, _)| (x, y, v))
        .collect();
    assert_eq!(top_pts.len(), 1008);
    Ok((top_pts, room_boxes))
}

/// Synthesizes `16x16 = 256` zero-mean unit-norm beast face templates (`f32x16` SIMD aligned!)
/// for a beast from its normal (`112x120`) and jailed (`120x144`) portraits composited onto
/// the `120x144` cell shield (`UI_T_V2_S16BoardCell_046`).
pub fn synthesize_beast_face_templates(
    shield: &DecodedAssetImage,
    norm_tex: &DecodedAssetImage,
    jail_tex: &DecodedAssetImage,
) -> Vec<[f32; 256]> {
    assert!(shield.width > 0 && shield.height > 0);
    let mut out = Vec::with_capacity(54);
    for (tex, ox, oy) in [(norm_tex, 4isize, 0isize), (jail_tex, 0isize, 0isize)] {
        let blur = composite_portrait_on_shield(shield, tex, ox, oy, (28.0, 26.0, 32.0));
        for &(cw, ch) in &[(74.0f64, 88.0f64), (82.0, 96.0), (90.0, 104.0)] {
            for &cy in &[55.0f64, 59.0, 63.0] {
                for &cx in &[58.0f64, 60.0, 62.0] {
                    let x0 = cx - cw * 0.5;
                    let y0 = cy - ch * 0.5;
                    let mut tv = [0.0f32; 256];
                    for j in 0..16 {
                        for i in 0..16 {
                            let sx = x0 + cw * ((i as f64) + 0.5) / 16.0;
                            let sy = y0 + ch * ((j as f64) + 0.5) / 16.0;
                            tv[j * 16 + i] = sample_gray_bilinear(&blur, 120, 144, sx, sy);
                        }
                    }
                    normalize_zero_mean(&mut tv);
                    out.push(tv);
                }
            }
        }
    }
    assert_eq!(out.len(), 54);
    out
}

fn synthesize_digits_from_fonts(fonts: &[TtfFont; 2]) -> DigitTemplates {
    assert_eq!(fonts.len(), 2);
    let mut digits: DigitTemplates = Default::default();
    for (d, digit_vecs) in digits.iter_mut().enumerate() {
        let ch = char::from_digit(d as u32, 10).unwrap();
        for font in fonts {
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
                let sx_span = sy_span * 10.0 / 16.0;
                for &dy_f in &[-0.03f64, 0.0, 0.03] {
                    for &dx_f in &[-0.03f64, 0.0, 0.03] {
                        let cx = gcx + dx_f * sx_span;
                        let cy = gcy + dy_f * sy_span;
                        let mut tv = [0.0f32; 160];
                        for j in 0..16 {
                            for i in 0..10 {
                                let sx = cx + (((i as f64) + 0.5) / 10.0 - 0.5) * sx_span;
                                let sy = cy + (((j as f64) + 0.5) / 16.0 - 0.5) * sy_span;
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
    assert_eq!(digits.len(), 10);
    digits
}

fn synthesize_prefixes_from_fonts(fonts: &[TtfFont; 2]) -> PrefixTemplates {
    assert_eq!(fonts.len(), 2);
    let mut prefixes = Vec::new();
    for font in fonts {
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
    assert!(!prefixes.is_empty());
    prefixes
}

/// Synthesizes `DIGITS` (`0..9`, `10x16 = 160`, multiple of 16 for SIMD!) and
/// `PREFIX_GLYPHS` (`"Go to Cell"`, `40x10 = 400`, multiple of 16 for SIMD!)
/// directly from `UI_Font_Light_EN_AZ.ufont` and `tl_main.ufont` in the `.pak` archives.
pub fn synthesize_digits_and_prefix(
    vfs: &PakVfs,
) -> Result<(DigitTemplates, PrefixTemplates), DataError> {
    let f1_bytes = vfs.read_path("UE_game/Content/Art/UI/Fonts/UI_Font_Light_EN_AZ.ufont")?;
    let f2_bytes = vfs.read_path("UE_game/Content/HotUpdate/Fonts/tl_main.ufont")?;
    assert!(!f1_bytes.is_empty() && !f2_bytes.is_empty());
    let fonts = [TtfFont::parse(f1_bytes)?, TtfFont::parse(f2_bytes)?];
    let digits = synthesize_digits_from_fonts(&fonts);
    let prefixes = synthesize_prefixes_from_fonts(&fonts);
    assert_eq!(digits.len(), 10);
    Ok((digits, prefixes))
}

/// Synthesizes `24x24 = 576` (`36` `f32x16` SIMD vectors) zero-mean unit-norm grayscale templates
/// for the 8 route/action card medallions (`UI_S16Gameplay_Route_01..02` & `Action_01..06`).
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
    assert_eq!(specs.len(), 8);
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
    assert_eq!(out.len(), 8);
    out
}

fn format_f32_slice(arr: &[f32]) -> String {
    assert!(!arr.is_empty());
    let mut s = String::from("[\n        ");
    for (i, &v) in arr.iter().enumerate() {
        if i > 0 {
            if i % 5 == 0 {
                s.push_str(",\n        ");
            } else {
                s.push_str(", ");
            }
        }
        let rounded: f32 = format!("{v:.5}").parse().unwrap_or(v);
        s.push_str(&format!("{rounded:?}_f32"));
    }
    s.push_str(",\n    ]");
    assert!(s.len() > 2);
    s
}

fn sim_str_to_tool_id_expr(sim: &str) -> &'static str {
    assert!(!sim.is_empty());
    let out = match sim {
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
    };
    assert!(out.starts_with("ToolId::"));
    out
}

fn sim_str_to_gift_id_expr(sim: &str) -> &'static str {
    assert!(!sim.is_empty());
    let out = match sim {
        "ring" => "GiftId::Ring",
        "horn" => "GiftId::Horn",
        "wine" => "GiftId::Wine",
        "brand" => "GiftId::Brand",
        _ => unreachable!("unknown gift sim key"),
    };
    assert!(out.starts_with("GiftId::"));
    out
}

fn extract_umg_board_cell_slots(vfs: &PakVfs) -> [UmgAnchorData; 16] {
    let mut out = [UmgAnchorData::default(); 16];
    if let Ok(bp) =
        vfs.read_umg_blueprint("UE_game/Content/Art/UIV2/S16Gameplay/BluePrint/S16BoardCentralView")
    {
        for (i, slot) in bp.anchor_slots.iter().copied().take(16).enumerate() {
            out[i] = slot;
        }
    }
    assert_eq!(out.len(), 16);
    assert!(out[0].offset_left.is_finite());
    out
}

fn emit_header_and_umg_slots(
    out: &mut String,
    db: &S16GameDatabase,
    board_pts: &[BoardPoint],
    room_boxes: &[RoomBox; 16],
    umg_slots: &[UmgAnchorData; 16],
) {
    assert_eq!(room_boxes.len(), 16);
    assert_eq!(board_pts.len(), 1008);
    out.push_str("// @generated by `cargo run -p tartarus-data`.\n");
    out.push_str(
        "// Authoritative S16 UMG layouts & SIMD templates synthesized 100% from .pak assets.\n\n",
    );
    out.push_str("#[repr(C, align(64))]\n#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct SimdVec160(pub [f32; 160]);\n\n");
    out.push_str("#[repr(C, align(64))]\n#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct SimdVec256(pub [f32; 256]);\n\n");
    out.push_str("#[repr(C, align(64))]\n#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct SimdVec400(pub [f32; 400]);\n\n");
    out.push_str("#[repr(C, align(64))]\n#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct SimdVec576(pub [f32; 576]);\n\n");

    out.push_str("#[derive(Clone, Copy, Debug)]\npub struct UmgSlotSpec {\n");
    out.push_str("    pub offset_left: f32,\n    pub offset_top: f32,\n");
    out.push_str("    pub offset_right: f32,\n    pub offset_bottom: f32,\n");
    out.push_str("    pub anchor_min_x: f32,\n    pub anchor_min_y: f32,\n");
    out.push_str("    pub anchor_max_x: f32,\n    pub anchor_max_y: f32,\n");
    out.push_str("    pub align_x: f32,\n    pub align_y: f32,\n}\n\n");

    out.push_str("pub static UMG_BOARD_CELL_SLOTS: [UmgSlotSpec; 16] = [\n");
    for s in umg_slots {
        out.push_str("    UmgSlotSpec {\n");
        out.push_str(&format!(
            "        offset_left: {:.2}_f32, offset_top: {:.2}_f32,\n",
            s.offset_left, s.offset_top
        ));
        out.push_str(&format!(
            "        offset_right: {:.2}_f32, offset_bottom: {:.2}_f32,\n",
            s.offset_right, s.offset_bottom
        ));
        out.push_str(&format!(
            "        anchor_min_x: {:.4}_f32, anchor_min_y: {:.4}_f32,\n",
            s.anchor_min_x, s.anchor_min_y
        ));
        out.push_str(&format!(
            "        anchor_max_x: {:.4}_f32, anchor_max_y: {:.4}_f32,\n",
            s.anchor_max_x, s.anchor_max_y
        ));
        out.push_str(&format!(
            "        align_x: {:.4}_f32, align_y: {:.4}_f32,\n",
            s.align_x, s.align_y
        ));
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");
    emit_relic_meta_and_board(out, db, board_pts, room_boxes);
}

fn format_rust_str_100(val: &str) -> String {
    assert!(val.len() <= 4096);
    if val.len() <= 54 {
        let out = format!("{val:?}");
        assert!(!out.is_empty());
        return out;
    }
    let mut parts = Vec::new();
    let mut cur = String::new();
    for ch in val.chars() {
        cur.push(ch);
        if cur.len() >= 54 {
            parts.push(format!("{cur:?}"));
            cur.clear();
        }
    }
    if !cur.is_empty() {
        parts.push(format!("{cur:?}"));
    }
    assert!(!parts.is_empty());
    format!(
        "concat!(\n            {}\n        )",
        parts.join(",\n            ")
    )
}

fn emit_relic_meta_and_board(
    out: &mut String,
    db: &S16GameDatabase,
    board_pts: &[BoardPoint],
    room_boxes: &[RoomBox; 16],
) {
    assert_eq!(room_boxes.len(), 16);
    assert!(!board_pts.is_empty());
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
        out.push_str("    S16RelicMeta {\n");
        out.push_str(&format!("        game_id: {},\n", r.id));
        out.push_str(&format!(
            "        sim: {},\n",
            sim_str_to_tool_id_expr(&r.js_sim)
        ));
        out.push_str(&format!("        icon_asset: {:?},\n", r.icon_asset));
        out.push_str(&format!(
            "        name: {},\n",
            format_rust_str_100(&r.name)
        ));
        out.push_str(&format!(
            "        trigger: {},\n",
            format_rust_str_100(&r.trigger_text)
        ));
        out.push_str(&format!(
            "        effect: {},\n",
            format_rust_str_100(&r.effect_text)
        ));
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");

    out.push_str("pub const REF_BOARD_WIDTH: f32 = 755.0;\n");
    out.push_str("pub const REF_BOARD_HEIGHT: f32 = 424.0;\n");
    out.push_str("pub static ROOM_BG_BOXES: [(u16, u16, u16, u16); 16] = [\n");
    for &(x, y, w, h) in room_boxes {
        out.push_str(&format!("    ({x}, {y}, {w}, {h}),\n"));
    }
    out.push_str("];\n\n");

    out.push_str(&format!(
        "pub static REF_BOARD_POINTS: [(i32, i32, f32); {}] = [\n",
        board_pts.len()
    ));
    for &(rx, ry, val) in board_pts {
        out.push_str(&format!("    ({rx}, {ry}, {val:.1}_f32),\n"));
    }
    out.push_str("];\n\n");
}

fn emit_beast_faces(
    out: &mut String,
    shield: &DecodedAssetImage,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), DataError> {
    assert!(shield.width > 0 && shield.height > 0);
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
            .ok_or_else(|| DataError::MissingAsset(norm_name.to_string()))?;
        let jail_tex = textures
            .get(jail_name)
            .ok_or_else(|| DataError::MissingAsset(jail_name.to_string()))?;
        let vecs = synthesize_beast_face_templates(shield, norm_tex, jail_tex);
        out.push_str(&format!(
            "pub static {}: [SimdVec256; {}] = [\n",
            const_name,
            vecs.len()
        ));
        for v in &vecs {
            out.push_str("    SimdVec256(");
            out.push_str(&format_f32_slice(v));
            out.push_str("),\n");
        }
        out.push_str("];\n\n");
    }
    assert!(!out.is_empty());
    Ok(())
}

fn emit_digits_prefixes_and_medallions(
    out: &mut String,
    digits: &DigitTemplates,
    prefixes: &PrefixTemplates,
    medallions: &[(&str, &str, [f32; 576])],
) {
    assert_eq!(digits.len(), 10);
    assert_eq!(medallions.len(), 8);
    for (d, vecs) in digits.iter().enumerate() {
        out.push_str(&format!(
            "pub static DIGIT_{}: [SimdVec160; {}] = [\n",
            d,
            vecs.len()
        ));
        for v in vecs {
            out.push_str("    SimdVec160(");
            out.push_str(&format_f32_slice(v));
            out.push_str("),\n");
        }
        out.push_str("];\n");
    }
    out.push_str("pub static DIGITS: [&[SimdVec160]; 10] = [\n");
    for d in 0..=9 {
        out.push_str(&format!("    &DIGIT_{d},\n"));
    }
    out.push_str("];\n\n");

    out.push_str("pub const PREFIX_DIM: usize = 400;\n");
    out.push_str(&format!(
        "pub static PREFIX_GLYPHS: [SimdVec400; {}] = [\n",
        prefixes.len()
    ));
    for v in prefixes {
        out.push_str("    SimdVec400(");
        out.push_str(&format_f32_slice(v));
        out.push_str("),\n");
    }
    out.push_str("];\n\n");

    out.push_str("#[repr(C, align(64))]\n#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct CardMedallionTemplate {\n");
    out.push_str("    pub grid: SimdVec576,\n");
    out.push_str("    pub id: &'static str,\n");
    out.push_str("    pub asset: &'static str,\n");
    out.push_str("}\n\n");
    out.push_str(&format!(
        "pub static CARD_MEDALLIONS: [CardMedallionTemplate; {}] = [\n",
        medallions.len()
    ));
    for (id, asset, grid) in medallions {
        out.push_str("    CardMedallionTemplate {\n");
        out.push_str(&format!(
            "        grid: SimdVec576({}),\n",
            format_f32_slice(grid)
        ));
        out.push_str(&format!("        id: {:?},\n", id));
        out.push_str(&format!("        asset: {:?},\n", asset));
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");
}

struct ToolFeatureRow {
    id: String,
    sim: String,
    name: String,
    is_panel: bool,
    grid: [f32; 72],
    hist: [f32; 13],
}

struct ToolRowsAccum {
    ref_tools: Vec<ToolFeatureRow>,
    ref_picks: Vec<(String, [f32; 72], [f32; 13])>,
    ref_gifts: Vec<(String, [f32; 72], [f32; 13])>,
}

fn synthesize_relic_panel_rows(
    r: &S16RelicEntry,
    icon: &DecodedAssetImage,
    item_bg: Option<&DecodedAssetImage>,
    accum: &mut ToolRowsAccum,
) {
    assert!(icon.width > 0 && icon.height > 0);
    let (x0, y0, x1, y1) = alpha_bbox(&icon.rgba8, icon.width, icon.height);
    let bw = (x1 - x0) as isize;
    let bh = (y1 - y0) as isize;

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
        accum.ref_tools.push(ToolFeatureRow {
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
            let comp = composite_rgba_scaled(bg, icon, (52, 86), (s, s), (16.0, 14.0, 18.0));
            let (grid, hist) = compute_art_feature_region(
                &comp,
                (bg.width, bg.height),
                (8, 36, bg.width - 8, bg.height - 36),
                (16.0, 14.0, 18.0),
            );
            accum.ref_tools.push(ToolFeatureRow {
                id: r.js_sim.clone(),
                sim: r.js_sim.clone(),
                name: r.name.clone(),
                is_panel: true,
                grid,
                hist,
            });
        }
    }
    assert!(!accum.ref_tools.is_empty());
}

fn synthesize_relic_pick_rows(
    r: &S16RelicEntry,
    icon: &DecodedAssetImage,
    big_pics: Option<&DecodedAssetImage>,
    accum: &mut ToolRowsAccum,
) {
    assert!(icon.width > 0 && icon.height > 0);
    let (p_grid, p_hist) = synthesize_icon_feature(icon, (28.0, 22.0, 20.0));
    accum.ref_tools.push(ToolFeatureRow {
        id: r.js_sim.clone(),
        sim: r.js_sim.clone(),
        name: r.name.clone(),
        is_panel: false,
        grid: p_grid,
        hist: p_hist,
    });
    for &alpha_thr in &[16u8, 48, 84] {
        let bbox = alpha_bbox_thresh(&icon.rgba8, icon.width, icon.height, alpha_thr);
        let (grid, hist) = compute_art_feature_region(
            &icon.rgba8,
            (icon.width, icon.height),
            bbox,
            (26.0, 21.0, 19.0),
        );
        accum.ref_picks.push((r.js_sim.clone(), grid, hist));
    }

    if let Some(bg) = big_pics {
        for &s in &[250usize, 285, 315] {
            let comp = composite_rgba_scaled(bg, icon, (250, 260), (s, s), (22.0, 20.0, 24.0));
            for &(mx_hi, v_thr) in &[(72u8, 0.28f32), (82, 0.32), (94, 0.36)] {
                if let Some(bbox) = pick_art_bbox_on_rgba(
                    &comp,
                    (bg.width, bg.height),
                    (250 - 110, 130, 250 + 110, 395),
                    mx_hi,
                    v_thr,
                ) {
                    let (grid, hist) = compute_art_feature_region(
                        &comp,
                        (bg.width, bg.height),
                        bbox,
                        (22.0, 20.0, 24.0),
                    );
                    accum.ref_picks.push((r.js_sim.clone(), grid, hist));
                }
            }
        }
    }
    assert!(!accum.ref_picks.is_empty());
}

fn synthesize_gift_rows(g: &S16GiftEntry, icon: &DecodedAssetImage, accum: &mut ToolRowsAccum) {
    assert!(icon.width > 0 && icon.height > 0);
    for &alpha_thr in &[16u8, 48, 84] {
        let bbox = alpha_bbox_thresh(&icon.rgba8, icon.width, icon.height, alpha_thr);
        let (grid, hist) = compute_art_feature_region(
            &icon.rgba8,
            (icon.width, icon.height),
            bbox,
            (24.0, 20.0, 22.0),
        );
        accum.ref_gifts.push((g.js_sim.clone(), grid, hist));
    }
    assert!(!accum.ref_gifts.is_empty());
}

fn emit_ref_tools_and_gifts(
    out: &mut String,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) {
    assert!(!db.relics.is_empty());
    let item_bg = textures.get("UI_Img_S16Gameplay_Jail_Item_Bg");
    let big_pics = textures.get("UI_Btn_S16Gameplay_Jail_Device_Big_Pics");
    let mut accum = ToolRowsAccum {
        ref_tools: Vec::new(),
        ref_picks: Vec::new(),
        ref_gifts: Vec::new(),
    };

    for r in &db.relics {
        if let Some(icon) = textures.get(&r.icon_asset) {
            synthesize_relic_panel_rows(r, icon, item_bg, &mut accum);
            synthesize_relic_pick_rows(r, icon, big_pics, &mut accum);
        }
    }
    for g in &db.colored_rewards {
        if let Some(icon) = textures.get(&g.icon_asset) {
            synthesize_gift_rows(g, icon, &mut accum);
        }
    }
    assert!(!accum.ref_tools.is_empty());
    emit_tool_and_gift_statics(out, &accum);
}

fn emit_tool_and_gift_statics(out: &mut String, accum: &ToolRowsAccum) {
    assert!(!accum.ref_tools.is_empty());
    assert!(!accum.ref_picks.is_empty());
    out.push_str("#[repr(C, align(64))]\n#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct RefToolEntry {\n");
    out.push_str("    pub grid: [f32; 72],\n    pub hist: [f32; 13],\n");
    out.push_str("    pub id: &'static str,\n    pub sim: ToolId,\n");
    out.push_str("    pub name: &'static str,\n    pub is_panel: bool,\n}\n\n");

    out.push_str(&format!(
        "pub static REF_TOOLS: [RefToolEntry; {}] = [\n",
        accum.ref_tools.len()
    ));
    for row in &accum.ref_tools {
        out.push_str("    RefToolEntry {\n");
        out.push_str(&format!("        grid: {},\n", format_f32_slice(&row.grid)));
        out.push_str(&format!("        hist: {},\n", format_f32_slice(&row.hist)));
        out.push_str(&format!("        id: {:?},\n", row.id));
        out.push_str(&format!(
            "        sim: {},\n",
            sim_str_to_tool_id_expr(&row.sim)
        ));
        out.push_str(&format!(
            "        name: {},\n",
            format_rust_str_100(&row.name)
        ));
        out.push_str(&format!("        is_panel: {},\n", row.is_panel));
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");

    out.push_str("#[repr(C, align(64))]\n#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct RefPickToolEntry {\n");
    out.push_str("    pub grid: [f32; 72],\n    pub hist: [f32; 13],\n");
    out.push_str("    pub sim: ToolId,\n}\n\n");

    out.push_str(&format!(
        "pub static REF_PICK_TOOLS: [RefPickToolEntry; {}] = [\n",
        accum.ref_picks.len()
    ));
    for (sim, grid, hist) in &accum.ref_picks {
        out.push_str("    RefPickToolEntry {\n");
        out.push_str(&format!("        grid: {},\n", format_f32_slice(grid)));
        out.push_str(&format!("        hist: {},\n", format_f32_slice(hist)));
        out.push_str(&format!("        sim: {},\n", sim_str_to_tool_id_expr(sim)));
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");

    out.push_str("#[repr(C, align(64))]\n#[derive(Clone, Copy, Debug)]\n");
    out.push_str("pub struct RefGiftEntry {\n");
    out.push_str("    pub grid: [f32; 72],\n    pub hist: [f32; 13],\n");
    out.push_str("    pub sim: GiftId,\n}\n\n");

    out.push_str(&format!(
        "pub static REF_GIFTS: [RefGiftEntry; {}] = [\n",
        accum.ref_gifts.len()
    ));
    for (sim, grid, hist) in &accum.ref_gifts {
        out.push_str("    RefGiftEntry {\n");
        out.push_str(&format!("        grid: {},\n", format_f32_slice(grid)));
        out.push_str(&format!("        hist: {},\n", format_f32_slice(hist)));
        out.push_str(&format!("        sim: {},\n", sim_str_to_gift_id_expr(sim)));
        out.push_str("    },\n");
    }
    out.push_str("];\n");
}

/// Generates the self-contained Rust source file `crates/tartarus-scan/src/ui_data.rs`
/// 100% from the `.pak` archives (`PakVfs`).
pub fn generate_ui_data_rs(
    out_rs_path: &Path,
    vfs: &PakVfs,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), DataError> {
    assert!(!out_rs_path.as_os_str().is_empty());
    let (board_pts, room_boxes) = synthesize_board_points_and_rooms(textures)?;
    let (digits, prefixes) = synthesize_digits_and_prefix(vfs)?;
    let medallions = synthesize_card_medallions(textures);
    let umg_slots = extract_umg_board_cell_slots(vfs);

    let shield = textures
        .get("UI_T_V2_S16BoardCell_046")
        .ok_or_else(|| DataError::MissingAsset("UI_T_V2_S16BoardCell_046".to_string()))?;

    let mut out = String::with_capacity(640 * 1024);
    emit_header_and_umg_slots(&mut out, db, &board_pts, &room_boxes, &umg_slots);
    emit_beast_faces(&mut out, shield, textures)?;
    emit_digits_prefixes_and_medallions(&mut out, &digits, &prefixes, &medallions);
    emit_ref_tools_and_gifts(&mut out, db, textures);
    assert!(!out.is_empty());

    fs::write(out_rs_path, out)?;
    Ok(())
}

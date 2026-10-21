//! Visual catalog sheet renderer (`catalog_relics.png`, `catalog_beasts_and_cells.png`,
//! `catalog_gifts_themes_events.png`, `catalog_board_layout.png`).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tartarus_extract::{PakVfs, encode_png_rgba8};

use crate::assets::DecodedAssetImage;
use crate::codegen::{
    OX_1344_TO_BG, OY_1344_TO_BG, S_1344_TO_BG, synthesize_board_points_and_rooms,
};
use crate::db::S16GameDatabase;
use crate::error::DataError;
use crate::font::{RasterizedGlyph, TtfFont};

pub(crate) fn draw_bitmap_label(
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
) -> Result<(), DataError> {
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
    fs::write(out_path, png)?;
    Ok(())
}

/// Renders `catalog_beasts_and_cells.png`: the 4 Beasts (`Mammoth`, `Lion`, `Wolf`, `Rat`)
/// with both their normal portrait (`101..104`) and behind-bars portrait (`045/047/050/048`),
/// plus their special cell marks (`RelicSP_01..04`).
pub fn render_catalog_beasts_and_cells_png(
    out_path: &Path,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), DataError> {
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
    fs::write(out_path, png)?;
    Ok(())
}

/// Renders `catalog_gifts_themes_events.png`: the 4 Warden's Gifts (`ColorRewards_01..04`),
/// 4 Start Themes (`SpecialTheme_01..04`), 5 Trial Objectives (`Tasks 1..5`), and 17 Prison Events (`Events 201..255`).
pub fn render_catalog_gifts_themes_events_png(
    out_path: &Path,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), DataError> {
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
    fs::write(out_path, png)?;
    Ok(())
}

/// Renders `catalog_board_layout.png`: visual verification of the `.pak`-synthesized board
/// points, 16 room highlight polygons, TrueType font digits/prefix, and route/action medallions.
pub fn render_catalog_board_layout_png(
    out_path: &Path,
    vfs: &PakVfs,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), DataError> {
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
    fs::write(out_path, png)?;
    Ok(())
}

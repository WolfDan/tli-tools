//! Visual catalog sheet renderer (`catalog_relics.png`, `catalog_beasts_and_cells.png`,
//! `catalog_gifts_themes_events.png`, `catalog_board_layout.png`).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tartarus_extract::{PakVfs, encode_png_rgba8};

use crate::assets::DecodedAssetImage;
use crate::db::{S16BeastEntry, S16GameDatabase, S16RelicEntry};
use crate::error::DataError;
use crate::font::{RasterizedGlyph, TtfFont};
use crate::ui_codegen::{
    OX_1344_TO_BG, OY_1344_TO_BG, S_1344_TO_BG, synthesize_board_points_and_rooms,
};

struct CatalogCanvas {
    buf: Vec<u8>,
    width: usize,
    height: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CatalogCanvas>() >= 24);
};

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
    assert!(w > 0 && h > 0);
    assert!(buf.len() >= w * h * 4);
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

fn wrap_text_lines(text: &str, chars_max: usize, lines_max: usize) -> Vec<String> {
    assert!(chars_max > 0);
    assert!(lines_max > 0);
    textwrap::wrap(text, chars_max)
        .into_iter()
        .take(lines_max)
        .map(|line| line.into_owned())
        .collect()
}

fn fill_rect(
    buf: &mut [u8],
    (w, h): (usize, usize),
    (x0, y0, rw, rh): (usize, usize, usize, usize),
    color: [u8; 4],
) {
    assert!(w > 0 && h > 0);
    assert!(buf.len() >= w * h * 4);
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
    assert!(dw > 0 && dh > 0);
    assert!(dst.len() >= dw * dh * 4);
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

impl CatalogCanvas {
    fn new(width: usize, height: usize, title: &str) -> Self {
        assert!(width > 0 && height > 0);
        let mut buf = vec![0u8; width * height * 4];
        for px in buf.as_chunks_mut::<4>().0 {
            px.copy_from_slice(&[18, 16, 22, 255]);
        }
        let mut canvas = Self { buf, width, height };
        canvas.label(16, 16, title, [250, 210, 120, 255]);
        assert_eq!(canvas.buf.len(), width * height * 4);
        canvas
    }

    fn label(&mut self, x: usize, y: usize, text: &str, color: [u8; 4]) {
        assert!(self.width > 0 && self.height > 0);
        assert_eq!(self.buf.len(), self.width * self.height * 4);
        draw_bitmap_label(&mut self.buf, self.width, self.height, x, y, text, color);
    }

    fn rect(&mut self, rect: (usize, usize, usize, usize), color: [u8; 4]) {
        assert!(self.width > 0 && self.height > 0);
        assert_eq!(self.buf.len(), self.width * self.height * 4);
        fill_rect(&mut self.buf, (self.width, self.height), rect, color);
    }

    fn blit(&mut self, rect: (usize, usize, usize, usize), img: &DecodedAssetImage) {
        assert!(self.width > 0 && self.height > 0);
        assert_eq!(self.buf.len(), self.width * self.height * 4);
        blit_scaled(&mut self.buf, (self.width, self.height), rect, img);
    }

    fn save(&self, out_path: &Path) -> Result<(), DataError> {
        assert!(!out_path.as_os_str().is_empty());
        assert_eq!(self.buf.len(), self.width * self.height * 4);
        let png = encode_png_rgba8(self.width as u32, self.height as u32, &self.buf)?;
        fs::write(out_path, png)?;
        Ok(())
    }
}

fn draw_relic_card(
    canvas: &mut CatalogCanvas,
    idx: usize,
    relic: &S16RelicEntry,
    textures: &BTreeMap<String, DecodedAssetImage>,
) {
    let cols = 3usize;
    let cell_w = 420usize;
    let cell_h = 112usize;
    let x0 = 16 + (idx % cols) * cell_w;
    let y0 = 44 + (idx / cols) * cell_h;
    assert!(x0 < canvas.width && y0 < canvas.height);

    canvas.rect((x0, y0, cell_w - 12, cell_h - 8), [28, 25, 34, 255]);
    if let Some(img) = textures.get(&relic.icon_asset) {
        canvas.blit((x0 + 8, y0 + 10, 76, 76), img);
    }

    let compact_asset = relic.icon_asset.trim_start_matches("UI_S16Gameplay_");
    canvas.label(x0 + 8, y0 + 90, compact_asset, [150, 145, 165, 255]);

    let tx = x0 + 92;
    let hdr = format!(
        "#{} [{}] {}",
        relic.id,
        relic.js_sim.to_uppercase(),
        relic.name
    );
    let hdr_trunc: String = hdr.chars().take(51).collect();
    canvas.label(tx, y0 + 8, &hdr_trunc, [255, 220, 130, 255]);

    let trig_str = format!("WHEN: {}", relic.trigger_text);
    let mut cy = y0 + 22;
    for line in wrap_text_lines(&trig_str, 51, 2) {
        canvas.label(tx, cy, &line, [170, 220, 255, 255]);
        cy += 11;
    }

    let eff_str = format!("DOES: {}", relic.effect_text);
    cy += 2;
    for line in wrap_text_lines(&eff_str, 51, 4) {
        if cy + 8 <= y0 + cell_h - 8 {
            canvas.label(tx, cy, &line, [215, 235, 205, 255]);
            cy += 11;
        }
    }
    assert!(cy <= y0 + cell_h + 16);
}

/// Renders `catalog_relics.png`: all 31 active S16 Torture Instruments (`RelicIcon_01..31`)
/// with their official ID, simulator key, English name, trigger text, and effect text.
pub fn render_catalog_relics_png(
    out_path: &Path,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), DataError> {
    assert!(!out_path.as_os_str().is_empty());
    let cols = 3usize;
    let rows = db.relics.len().div_ceil(cols).max(1);
    let mut canvas = CatalogCanvas::new(
        24 + cols * 420,
        52 + rows * 112,
        "S16 TARTARUS OFFICIAL RELIC CATALOG (31 ACTIVE TORTURE INSTRUMENTS)",
    );
    for (idx, relic) in db.relics.iter().enumerate() {
        draw_relic_card(&mut canvas, idx, relic, textures);
    }
    assert_eq!(canvas.width, 24 + cols * 420);
    canvas.save(out_path)
}

fn draw_beast_row(
    canvas: &mut CatalogCanvas,
    idx: usize,
    b: &S16BeastEntry,
    textures: &BTreeMap<String, DecodedAssetImage>,
) {
    let row_h = 128usize;
    let y0 = 46 + idx * row_h;
    assert!(y0 < canvas.height);
    canvas.rect((16, y0, canvas.width - 32, row_h - 10), [28, 25, 34, 255]);

    let title = format!(
        "{} ({})",
        b.display_name.to_uppercase(),
        b.beast.to_uppercase()
    );
    canvas.label(28, y0 + 12, &title, [255, 220, 130, 255]);

    if let Some(norm) = textures.get(&b.normal_portrait_asset) {
        canvas.blit((260, y0 + 10, 84, 84), norm);
        let lbl = format!(
            "NORMAL: {}",
            b.normal_portrait_asset.trim_start_matches("UI_T_V2_")
        );
        canvas.label(260, y0 + 98, &lbl, [190, 235, 190, 255]);
    }

    if let Some(jailed) = textures.get(&b.jailed_portrait_asset) {
        canvas.blit((470, y0 + 10, 84, 84), jailed);
        let lbl = format!(
            "JAILED: {}",
            b.jailed_portrait_asset.trim_start_matches("UI_T_V2_")
        );
        canvas.label(470, y0 + 98, &lbl, [170, 220, 255, 255]);
    }

    if let Some(sp_name) = &b.special_mark_asset {
        if let Some(sp_img) = textures.get(sp_name) {
            canvas.blit((690, y0 + 18, 64, 64), sp_img);
        }
        let lbl = sp_name.trim_start_matches("UI_S16Gameplay_");
        canvas.label(690, y0 + 98, lbl, [245, 200, 140, 255]);
    }

    if let Some(desc) = &b.special_mark_desc {
        for (li, line) in wrap_text_lines(desc, 34, 3).iter().enumerate() {
            canvas.label(28, y0 + 36 + li * 13, line, [215, 210, 225, 255]);
        }
    }
    assert!(y0 + row_h <= canvas.height);
}

/// Renders `catalog_beasts_and_cells.png`: the 4 Beasts (`Mammoth`, `Lion`, `Wolf`, `Rat`)
/// with both their normal portrait (`101..104`) and behind-bars portrait (`045/047/050/048`),
/// plus their special cell marks (`RelicSP_01..04`).
pub fn render_catalog_beasts_and_cells_png(
    out_path: &Path,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), DataError> {
    assert!(!out_path.as_os_str().is_empty());
    let rows = db.beasts.len().max(1);
    let mut canvas = CatalogCanvas::new(
        980,
        56 + rows * 128,
        "S16 TARTARUS BEAST PORTRAITS (NORMAL + BEHIND BARS) & SPECIAL CELL MARKS",
    );
    for (idx, b) in db.beasts.iter().enumerate() {
        draw_beast_row(&mut canvas, idx, b, textures);
    }
    assert_eq!(canvas.width, 980);
    canvas.save(out_path)
}

fn draw_gifts_and_themes_section(
    canvas: &mut CatalogCanvas,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) {
    assert!(canvas.width >= 1280);
    for (idx, gift) in db.colored_rewards.iter().enumerate() {
        let x0 = 16 + idx * 312;
        let y0 = 34;
        canvas.rect((x0, y0, 300, 100), [28, 25, 34, 255]);
        if let Some(img) = textures.get(&gift.icon_asset) {
            canvas.blit((x0 + 8, y0 + 12, 68, 68), img);
        }
        let hdr = format!(
            "GIFT #{} [{}] {}",
            gift.id,
            gift.js_sim.to_uppercase(),
            gift.name
        );
        let hdr_t: String = hdr.chars().take(34).collect();
        canvas.label(x0 + 84, y0 + 10, &hdr_t, [255, 220, 130, 255]);
        let short_icon = gift.icon_asset.trim_start_matches("UI_S16Gameplay_");
        canvas.label(x0 + 84, y0 + 23, short_icon, [150, 200, 240, 255]);
        for (li, line) in wrap_text_lines(&gift.text_clean, 34, 4).iter().enumerate() {
            canvas.label(x0 + 84, y0 + 38 + li * 12, line, [215, 235, 205, 255]);
        }
    }

    for (idx, theme) in db.start_themes.iter().enumerate() {
        let x0 = 16 + idx * 312;
        let y0 = 144;
        canvas.rect((x0, y0, 300, 108), [28, 25, 34, 255]);
        if let Some(img) = textures.get(&theme.icon_asset) {
            canvas.blit((x0 + 8, y0 + 14, 68, 68), img);
        }
        let hdr = format!(
            "THEME #{} [{}] {}",
            theme.id,
            theme.js_sim.to_uppercase(),
            theme.name
        );
        let hdr_t: String = hdr.chars().take(34).collect();
        canvas.label(x0 + 84, y0 + 10, &hdr_t, [255, 220, 130, 255]);
        let short_icon = theme.icon_asset.trim_start_matches("UI_S16Gameplay_");
        canvas.label(x0 + 84, y0 + 23, short_icon, [150, 200, 240, 255]);
        for (li, line) in wrap_text_lines(&theme.effect_text, 34, 5)
            .iter()
            .enumerate()
        {
            canvas.label(x0 + 84, y0 + 38 + li * 12, line, [215, 235, 205, 255]);
        }
    }
    assert!(canvas.height >= 260);
}

fn draw_tasks_and_events_section(canvas: &mut CatalogCanvas, db: &S16GameDatabase) {
    assert!(canvas.width >= 1280);
    canvas.label(
        16,
        266,
        "TRIAL OBJECTIVES (CONFIG_S16_GAMEPLAY_TASK 1..5)",
        [250, 210, 120, 255],
    );
    for (idx, task) in db.tasks.iter().enumerate() {
        let x0 = 16 + idx * 250;
        let y0 = 284;
        canvas.rect((x0, y0, 240, 74), [28, 25, 34, 255]);
        let hdr = format!(
            "TASK #{} [{}] {}",
            task.id,
            task.js_sim.to_uppercase(),
            task.name
        );
        let hdr_t: String = hdr.chars().take(38).collect();
        canvas.label(x0 + 8, y0 + 8, &hdr_t, [255, 220, 130, 255]);
        let desc = format!("{} (Need: {})", task.desc_text, task.progress_param);
        for (li, line) in wrap_text_lines(&desc, 38, 3).iter().enumerate() {
            canvas.label(x0 + 8, y0 + 24 + li * 12, line, [210, 230, 215, 255]);
        }
    }

    canvas.label(
        16,
        374,
        "PRISON SPECIAL EVENTS (CONFIG_S16_GAMEPLAY_EVENT 201..255)",
        [250, 210, 120, 255],
    );
    let ev_cols = 3usize;
    let ev_w = 416usize;
    let ev_h = 94usize;
    for (idx, ev) in db.events.iter().enumerate() {
        let x0 = 16 + (idx % ev_cols) * ev_w;
        let y0 = 394 + (idx / ev_cols) * ev_h;
        canvas.rect((x0, y0, ev_w - 10, ev_h - 8), [28, 25, 34, 255]);
        let hdr = format!(
            "EVENT #{} [{}] {} (W:{})",
            ev.id,
            ev.js_sim.to_uppercase(),
            ev.name,
            ev.weight
        );
        let hdr_t: String = hdr.chars().take(64).collect();
        canvas.label(x0 + 10, y0 + 8, &hdr_t, [255, 220, 130, 255]);
        for (li, line) in wrap_text_lines(&ev.desc_text, 64, 5).iter().enumerate() {
            canvas.label(x0 + 10, y0 + 23 + li * 12, line, [215, 225, 235, 255]);
        }
    }
    assert!(canvas.height >= 900);
}

/// Renders `catalog_gifts_themes_events.png`: the 4 Warden's Gifts (`ColorRewards_01..04`),
/// 4 Start Themes (`SpecialTheme_01..04`), 5 Trial Objectives (`Tasks 1..5`), and
/// 17 Prison Events (`Events 201..255`).
pub fn render_catalog_gifts_themes_events_png(
    out_path: &Path,
    db: &S16GameDatabase,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), DataError> {
    assert!(!out_path.as_os_str().is_empty());
    let mut canvas = CatalogCanvas::new(
        1280,
        980,
        "WARDEN'S GIFTS (COLORED REWARDS 1..4) & STARTING THEMES (301..304)",
    );
    draw_gifts_and_themes_section(&mut canvas, db, textures);
    draw_tasks_and_events_section(&mut canvas, db);
    assert_eq!(canvas.width, 1280);
    canvas.save(out_path)
}

fn draw_board_rooms_and_points(
    canvas: &mut CatalogCanvas,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), DataError> {
    assert!(canvas.width >= 1280 && canvas.height >= 800);
    if let Some(bg) = textures.get("UI_Img_S16Gameplay_Jail_Bg_Normal") {
        canvas.blit((16, 36, 1280, 768), bg);
    }

    let (pts, room_boxes) = synthesize_board_points_and_rooms(textures)?;
    assert_eq!(room_boxes.len(), 16);
    for (idx, &(rx, ry, rw, rh)) in room_boxes.iter().enumerate() {
        let x0 = 16 + (rx as usize * 1280) / 3200;
        let y0 = 36 + (ry as usize * 768) / 1920;
        let w = (rw as usize * 1280) / 3200;
        let h = (rh as usize * 768) / 1920;
        let cyan = [80, 220, 255, 255];
        canvas.rect((x0, y0, w, 2), cyan);
        canvas.rect((x0, y0 + h, w, 2), cyan);
        canvas.rect((x0, y0, 2, h), cyan);
        canvas.rect((x0 + w, y0, 2, h), cyan);
        canvas.label(
            x0 + 6,
            y0 + 6,
            &format!("CELL {:02}", idx + 1),
            [255, 235, 120, 255],
        );
    }

    for &(rx, ry, _) in &pts {
        let bx = (rx as f64) * S_1344_TO_BG + OX_1344_TO_BG;
        let by = (ry as f64) * S_1344_TO_BG + OY_1344_TO_BG;
        let px = 16 + ((bx * 1280.0 / 3200.0).round() as usize);
        let py = 36 + ((by * 768.0 / 1920.0).round() as usize);
        canvas.rect(
            (px.saturating_sub(1), py.saturating_sub(1), 3, 3),
            [90, 255, 130, 255],
        );
    }
    Ok(())
}

fn draw_medallions_sidebar(
    canvas: &mut CatalogCanvas,
    textures: &BTreeMap<String, DecodedAssetImage>,
) {
    assert!(canvas.width >= 1600);
    canvas.label(1316, 40, "ROUTE & ACTION SPRITES", [250, 210, 120, 255]);
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
    assert_eq!(specs.len(), 8);
    for (i, (label, asset)) in specs.iter().enumerate() {
        let y0 = 64 + i * 92;
        canvas.rect((1316, y0, 268, 84), [28, 25, 34, 255]);
        if let Some(tex) = textures.get(*asset) {
            canvas.blit((1324, y0 + 10, 64, 64), tex);
        }
        canvas.label(1398, y0 + 22, label, [255, 220, 130, 255]);
        let short_asset = asset.trim_start_matches("UI_S16Gameplay_");
        canvas.label(1398, y0 + 40, short_asset, [170, 215, 250, 255]);
    }
}

fn draw_font_preview_bar(canvas: &mut CatalogCanvas, vfs: &PakVfs) {
    assert!(canvas.width >= 1600 && canvas.height >= 1000);
    canvas.label(
        16,
        824,
        "TRUETYPE FONT RASTERIZATION (UI_FONT_LIGHT_EN_AZ.UFONT): DIGITS 0..9 & 'GO TO CELL'",
        [250, 210, 120, 255],
    );
    let font_path = "UE_game/Content/Art/UI/Fonts/UI_Font_Light_EN_AZ.ufont";
    let Ok(f1_bytes) = vfs.read_path(font_path) else {
        return;
    };
    let Ok(font) = TtfFont::parse(f1_bytes) else {
        return;
    };
    assert!(font.units_per_em > 0);
    for d in 0..=9 {
        let ch = char::from_digit(d as u32, 10).unwrap();
        let x0 = 16 + d * 96;
        let y0 = 848;
        canvas.rect((x0, y0, 86, 120), [28, 25, 34, 255]);
        canvas.label(x0 + 10, y0 + 8, &format!("DIGIT {d}"), [255, 220, 130, 255]);
        if let Some(RasterizedGlyph {
            cw, ch_px, blur, ..
        }) = font.rasterize_char(ch, 48, 8)
        {
            for py in 0..ch_px.min(84) {
                for px in 0..cw.min(72) {
                    let v = blur[py * cw + px].clamp(0.0, 255.0) as u8;
                    let idx = ((y0 + 26 + py) * canvas.width + (x0 + 12 + px)) * 4;
                    canvas.buf[idx..idx + 4].copy_from_slice(&[v, v, v, 255]);
                }
            }
        }
    }

    let x0 = 990;
    let y0 = 848;
    canvas.rect((x0, y0, 594, 120), [28, 25, 34, 255]);
    canvas.label(
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
                let idx = ((y0 + 26 + py) * canvas.width + (x0 + 16 + px)) * 4;
                canvas.buf[idx..idx + 4].copy_from_slice(&[v, v, v, 255]);
            }
        }
    }
}

/// Renders `catalog_board_layout.png`: visual verification of the `.pak`-synthesized board
/// points, 16 room highlight polygons, TrueType font digits/prefix, and route/action medallions.
pub fn render_catalog_board_layout_png(
    out_path: &Path,
    vfs: &PakVfs,
    textures: &BTreeMap<String, DecodedAssetImage>,
) -> Result<(), DataError> {
    assert!(!out_path.as_os_str().is_empty());
    let mut canvas = CatalogCanvas::new(
        1600,
        1120,
        "S16 PAK-SYNTHESIZED CV ASSETS: BOARD (3200X1920), ROOMS (01..16), GLYPHS & MEDALLIONS",
    );
    draw_board_rooms_and_points(&mut canvas, textures)?;
    draw_medallions_sidebar(&mut canvas, textures);
    draw_font_preview_bar(&mut canvas, vfs);
    assert_eq!(canvas.width, 1600);
    canvas.save(out_path)
}

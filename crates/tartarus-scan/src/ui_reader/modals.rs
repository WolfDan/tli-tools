//! Modal screen readers: 3-card Instrument Pick (`RelicModalRead` / `S16BoardRelicPop`),
//! 3-card Colored Reward (`GiftModalRead` / `S16BoardColoredPop`),
//! Objective & Theme scroll popups (`TitleModalRead` / `S16BoardTaskPop`, `S16BoardThemePop`),
//! and Consume target banner (`ConvoyChainRead`).

use tartarus_core::{FixedVec, GiftId, ObjectiveId, StartEventId, ToolId};

use super::board::{CellRead, WardenRead};
use super::data::{REF_GIFTS, REF_PICK_TOOLS, REF_TOOLS};
use super::glyphs::{
    Piece, UiScratchSlot, prepare_sat_region, scan_radii_from_built_sat,
    text_lines_radii_with_slot, text_radii,
};
use super::hud::{art_feature, art_similarity};
use super::simd::{FrameView, color_share_rgb, hsv, is_banner_red_rgb};
use super::viewport::UmgViewport;

const _: () = {
    assert!(REF_PICK_TOOLS.len() == 372);
    assert!(REF_GIFTS.len() == 12);
};

#[derive(Clone, Copy, Debug)]
pub struct ConvoyChainRead {
    pub banner: f32,
    pub targets: FixedVec<u8, 16>,
}

#[inline]
pub(crate) fn chain_banner_score(frame: &FrameView<'_>, viewport: &UmgViewport) -> f32 {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(viewport.s > 0.0);
    let step = ((2.0 * viewport.s).round() as usize).clamp(1, 4);
    let mut banner = 0.0_f32;
    let mut v = 3.9_f64;
    let mut iterations = 0usize;
    while v <= 5.01 {
        iterations += 1;
        assert!(iterations <= 16);
        let (x0, y0) = viewport.at(1.0, v);
        let (x1, y1) = viewport.at(4.0, v + 0.15);
        let sh = color_share_rgb(
            frame,
            x0.min(x1),
            y0.min(y1),
            x0.max(x1),
            y0.max(y1),
            step,
            is_banner_red_rgb,
        );
        if sh > banner {
            banner = sh;
        }
        v += 0.15;
    }
    banner
}

pub fn read_convoy_chain(
    frame: &FrameView<'_>,
    viewport: &UmgViewport,
    cells: &[CellRead; 16],
    warden: Option<&WardenRead>,
) -> Option<ConvoyChainRead> {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert_eq!(cells.len(), 16);
    read_convoy_chain_with_banner(
        frame,
        viewport,
        cells,
        warden,
        chain_banner_score(frame, viewport),
    )
}

pub(crate) fn read_convoy_chain_with_banner(
    frame: &FrameView<'_>,
    _viewport: &UmgViewport,
    cells: &[CellRead; 16],
    warden: Option<&WardenRead>,
    banner: f32,
) -> Option<ConvoyChainRead> {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(banner.is_finite());
    if banner < 0.20 {
        return None;
    }
    let mut targets = FixedVec::new();
    for c in cells.iter().skip(1) {
        if warden.is_some_and(|w| c.n == w.cell) {
            continue;
        }
        let k = c.k;
        let step = (k.round() as usize).clamp(1, 2);
        if color_share_rgb(
            frame,
            c.x - 40.0 * k,
            c.y - 80.0 * k,
            c.x + 40.0 * k,
            c.y - 50.0 * k,
            step,
            is_banner_red_rgb,
        ) > 0.15
        {
            targets.push(c.n);
        }
    }
    if targets.is_empty() {
        None
    } else {
        Some(ConvoyChainRead { banner, targets })
    }
}

pub(crate) fn has_devour_banner(frame: &FrameView<'_>) -> bool {
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    let x0 = (frame.width() * 3) / 10;
    let x1 = (frame.width() * 7) / 10;
    let y0 = (frame.height() * 45) / 100;
    let y1 = (frame.height() * 90) / 100;
    if x1 <= x0 || y1 <= y0 {
        return false;
    }
    let step_x = ((x1 - x0) / 32).max(2);
    let step_y = ((y1 - y0) / 24).max(2);
    for y in (y0..y1).step_by(step_y) {
        let mut hits = 0usize;
        let mut samples_total = 0usize;
        for x in (x0..x1).step_by(step_x) {
            let (r, g, b) = frame.rgb_at(x as isize, y as isize);
            samples_total += 1;
            if r >= 102 && r > g && r > b {
                let (h, s, v) = hsv(r as f32, g as f32, b as f32);
                if (h < 16.0 || h > 340.0) && s > 0.45 && v > 0.40 {
                    hits += 1;
                }
            }
        }
        if samples_total > 0 && hits * 6 >= samples_total {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy, Debug)]
pub struct RelicCardRead {
    pub sim: ToolId,
    pub score: f32,
    pub conf: f32,
}

#[derive(Clone, Debug)]
pub struct RelicModalRead {
    pub cards: [RelicCardRead; 3],
    pub reroll: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct GiftCardRead {
    pub gift: GiftId,
    pub score: f32,
    pub conf: f32,
}

#[derive(Clone, Debug)]
pub struct GiftModalRead {
    pub cards: [GiftCardRead; 3],
}

#[inline]
fn pick_card_body_dark_ok(frame: &FrameView<'_>, k: i32, s: f64, step: usize) -> bool {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(step >= 1);
    let c = (frame.width() as f64) * 0.5 + (k as f64) * 480.0 * s;
    let x0 = ((c - 150.0 * s).round() as isize).max(0) as usize;
    let x1 = ((c + 150.0 * s).round() as isize).min(frame.width() as isize - 1) as usize;
    let y0 = ((560.0 * s).round() as isize).max(0) as usize;
    let y1 = ((760.0 * s).round() as isize).min(frame.height() as isize - 1) as usize;
    if x1 <= x0 || y1 <= y0 {
        return false;
    }
    let mut px_total = 0usize;
    let mut dark_slate = 0usize;
    let mut sum_v = 0u32;
    for y in (y0..=y1).step_by(step) {
        let row = frame.row_offset(y);
        for x in (x0..=x1).step_by(step) {
            let o = row + x * 4;
            let (r, g, b) = (
                frame.pixels_rgba8[o],
                frame.pixels_rgba8[o + 1],
                frame.pixels_rgba8[o + 2],
            );
            let mx = r.max(g).max(b);
            let mn = r.min(g).min(b);
            px_total += 1;
            sum_v += mx as u32;
            if mx <= 105 && (mx - mn) <= 48 {
                dark_slate += 1;
            }
        }
    }
    px_total > 0 && dark_slate * 100 >= px_total * 48 && (sum_v / (px_total as u32)) <= 96
}

#[inline]
fn pick_modal_backdrop_ok(frame: &FrameView<'_>) -> bool {
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    let s = (frame.height() as f64) / 1080.0;
    let step = (4.0 * s).round().max(2.0) as usize;

    for k in [-1i32, 0, 1] {
        if !pick_card_body_dark_ok(frame, k, s, step) {
            return false;
        }
    }

    for sign in [-1.0_f64, 1.0_f64] {
        let gx = (frame.width() as f64) * 0.5 + sign * 240.0 * s;
        let x0 = ((gx - 18.0 * s).round() as isize).max(0) as usize;
        let x1 = ((gx + 18.0 * s).round() as isize).min(frame.width() as isize - 1) as usize;
        let y0 = ((280.0 * s).round() as isize).max(0) as usize;
        let y1 = ((740.0 * s).round() as isize).min(frame.height() as isize - 1) as usize;
        if x1 <= x0 || y1 <= y0 {
            return false;
        }
        let mut px_total = 0usize;
        let mut sum_v = 0u32;
        for y in (y0..=y1).step_by(step) {
            let row = frame.row_offset(y);
            for x in (x0..=x1).step_by(step) {
                let o = row + x * 4;
                let mx = frame.pixels_rgba8[o]
                    .max(frame.pixels_rgba8[o + 1])
                    .max(frame.pixels_rgba8[o + 2]);
                px_total += 1;
                sum_v += mx as u32;
            }
        }
        if px_total == 0 || (sum_v / (px_total as u32)) > 110 {
            return false;
        }
    }

    true
}

#[inline]
fn is_pick_art_pixel(r: u8, g: u8, b: u8) -> bool {
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    debug_assert!(mx >= mn);
    debug_assert!(mn <= r);
    if mx >= 82 {
        return true;
    }
    let delta = (mx - mn) as u32;
    if mx <= 45 || delta * 20 <= 9 * (mx as u32) {
        return false;
    }
    let warm_brown =
        r == mx && g >= b && ((g - b) as u32) * 4 >= delta && ((g - b) as u32) * 6 <= delta * 5;
    !warm_brown
}

#[inline]
fn percentile_from_hist(hist: &[u32], base: i32, rank: u32) -> i32 {
    debug_assert!(!hist.is_empty());
    debug_assert!(base >= 0);
    let mut acc = 0u32;
    for (idx, &c) in hist.iter().enumerate() {
        acc += c;
        if acc > rank {
            return base + (idx as i32);
        }
    }
    base + (hist.len().saturating_sub(1) as i32)
}

pub fn pick_art_box(frame: &FrameView<'_>, k: i32) -> Option<[i32; 4]> {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!((-1..=1).contains(&k));
    let s = (frame.height() as f64) / 1080.0;
    let c = (frame.width() as f64) * 0.5 + (k as f64) * 480.0 * s;
    let x0 = ((c - 85.0 * s).round() as i32).max(0);
    let x1 = ((c + 85.0 * s).round() as i32).min(frame.width() as i32 - 1);
    let y0 = ((270.0 * s).round() as i32).max(0);
    let y1 = ((458.0 * s).round() as i32).min(frame.height() as i32 - 1);
    if x1 < x0 || y1 < y0 {
        return None;
    }
    let w = (x1 - x0 + 1) as usize;
    let h = (y1 - y0 + 1) as usize;
    if w > 512 || h > 512 {
        return None;
    }
    let mut x_hist = [0u32; 512];
    let mut y_hist = [0u32; 512];
    let mut count = 0usize;
    let (ux0, uy0) = (x0 as usize, y0 as usize);
    for (dy, y_slot) in y_hist[..h].iter_mut().enumerate() {
        let row_base = frame.row_offset(uy0 + dy);
        let mut row_hits = 0u32;
        for (dx, x_slot) in x_hist[..w].iter_mut().enumerate() {
            let o = row_base + (ux0 + dx) * 4;
            if is_pick_art_pixel(
                frame.pixels_rgba8[o],
                frame.pixels_rgba8[o + 1],
                frame.pixels_rgba8[o + 2],
            ) {
                *x_slot += 1;
                row_hits += 1;
            }
        }
        *y_slot = row_hits;
        count += row_hits as usize;
    }
    let area_total = w * h;
    if (count as f64) < 400.0 * s * s || count * 100 > area_total * 92 {
        return None;
    }
    let rank_lo = (0.01 * ((count - 1) as f64)).round() as u32;
    let rank_hi = (0.99 * ((count - 1) as f64)).round() as u32;
    Some([
        percentile_from_hist(&x_hist[..w], x0, rank_lo),
        percentile_from_hist(&y_hist[..h], y0, rank_lo),
        percentile_from_hist(&x_hist[..w], x0, rank_hi),
        percentile_from_hist(&y_hist[..h], y0, rank_hi),
    ])
}

#[inline]
fn count_neutral_light_thresh(
    frame: &FrameView<'_>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    mx_min: u8,
) -> usize {
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    let ux0 = x0.max(0) as usize;
    let uy0 = y0.max(0) as usize;
    let ux1 = x1.min(frame.width() as i32 - 1);
    let uy1 = y1.min(frame.height() as i32 - 1);
    if ux1 < ux0 as i32 || uy1 < uy0 as i32 {
        return 0;
    }
    let ux1 = ux1 as usize;
    let uy1 = uy1 as usize;
    let mut light = 0usize;
    for y in (uy0..=uy1).step_by(2) {
        let row_base = frame.row_offset(y);
        for x in (ux0..=ux1).step_by(2) {
            let o = row_base + x * 4;
            let r = frame.pixels_rgba8[o];
            let g = frame.pixels_rgba8[o + 1];
            let b = frame.pixels_rgba8[o + 2];
            let mx = r.max(g).max(b);
            if mx > mx_min {
                let mn = r.min(g).min(b);
                if ((mx - mn) as u32) * 4 < (mx as u32) {
                    light += 1;
                }
            }
        }
    }
    light
}

#[inline]
fn count_neutral_light(frame: &FrameView<'_>, x0: i32, y0: i32, x1: i32, y1: i32) -> usize {
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    count_neutral_light_thresh(frame, x0, y0, x1, y1, 140)
}

pub fn pick_reroll_shown(frame: &FrameView<'_>) -> bool {
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    let s = (frame.height() as f64) / 1080.0;
    let x0 = ((frame.width() as f64) * 0.5 - 330.0 * s).round() as i32;
    let x1 = ((frame.width() as f64) * 0.5 - 175.0 * s).round() as i32;
    let y0 = (940.0 * s).round() as i32;
    let y1 = (985.0 * s).round() as i32;
    (count_neutral_light(frame, x0, y0, x1, y1) as f64) > 15.0 * s * s
}

pub fn pick_confirm_shown(frame: &FrameView<'_>) -> bool {
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    let s = (frame.height() as f64) / 1080.0;
    let x0 = ((frame.width() as f64) * 0.5 - 120.0 * s).round() as i32;
    let x1 = ((frame.width() as f64) * 0.5 + 330.0 * s).round() as i32;
    let y0 = (940.0 * s).round() as i32;
    let y1 = (985.0 * s).round() as i32;
    (count_neutral_light_thresh(frame, x0, y0, x1, y1, 115) as f64) > 15.0 * s * s
}

fn count_pick_title_glyphs(
    frame: &FrameView<'_>,
    k: i32,
    s: f64,
    slot: &mut UiScratchSlot,
) -> (usize, usize) {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!((-1..=1).contains(&k));
    let radii = text_radii(frame);
    let title_radius = [radii[1]];
    let title_h_min = (13.0 * s).round().max(7.0) as i32;
    let c = (frame.width() as f64) * 0.5 + (k as f64) * 480.0 * s;
    let title_lines = text_lines_radii_with_slot(
        frame,
        [c - 175.0 * s, 456.0 * s, c + 175.0 * s, 542.0 * s],
        ((7.0 * s).max(5.0), (45.0 * s).max(16.0)),
        &title_radius,
        0.35,
        slot,
    );
    let mut observed_title_glyphs = 0usize;
    let mut observed_title_lines = 0usize;
    if let Some(lines) = title_lines.first() {
        for line in lines {
            let ly = (line.y0 + line.y1) as f64 * 0.5;
            if ly < 458.0 * s || ly > 540.0 * s || line.h < title_h_min {
                continue;
            }
            let mut line_glyphs = 0usize;
            for piece in &line.pieces {
                let px = (piece.x0 + piece.x1) as f64 * 0.5;
                if (px - c).abs() <= 155.0 * s && piece.glyphs.len() >= 2 {
                    line_glyphs += piece.glyphs.len();
                }
            }
            if line_glyphs >= 2 {
                observed_title_glyphs += line_glyphs;
                observed_title_lines += 1;
            }
        }
        if observed_title_glyphs < 4 {
            for line in lines {
                if line.glyphs.len() >= 4 {
                    observed_title_glyphs = observed_title_glyphs.max(line.glyphs.len());
                    observed_title_lines = observed_title_lines.max(1);
                }
            }
        }
    }
    (observed_title_glyphs, observed_title_lines)
}

fn score_single_pick_feat(fg: &[f32; 72], fh: &[f32; 13], best_by_sim: &mut [f32; 31]) {
    assert_eq!(best_by_sim.len(), 31);
    assert_eq!(fg.len(), 72);
    for r in &REF_PICK_TOOLS {
        let sc = art_similarity(fg, fh, &r.grid, &r.hist);
        let slot_sc = &mut best_by_sim[r.sim.idx()];
        if sc > *slot_sc {
            *slot_sc = sc;
        }
    }
    for r in &REF_TOOLS {
        if r.is_panel {
            continue;
        }
        let sc = art_similarity(fg, fh, &r.grid, &r.hist);
        let slot_sc = &mut best_by_sim[r.sim.idx()];
        if sc > *slot_sc {
            *slot_sc = sc;
        }
    }
}

fn score_pick_card_templates(frame: &FrameView<'_>, bx: [i32; 4]) -> [f32; 31] {
    assert!(bx[2] >= bx[0]);
    assert!(bx[3] >= bx[1]);
    let mut best_by_sim = [f32::NEG_INFINITY; 31];
    let (fg0, fh0) = art_feature(frame, bx[0], bx[1], bx[2], bx[3]);
    score_single_pick_feat(&fg0, &fh0, &mut best_by_sim);
    let mut top = f32::NEG_INFINITY;
    let mut second = f32::NEG_INFINITY;
    for &sc in &best_by_sim {
        if sc > top {
            second = top;
            top = sc;
        } else if sc > second {
            second = sc;
        }
    }
    if top < 0.94 || (top - second) < 0.05 {
        for [dx, dy] in [[-2, 0], [2, 0], [0, -2], [0, 2]] {
            let (fg, fh) = art_feature(frame, bx[0] + dx, bx[1] + dy, bx[2] + dx, bx[3] + dy);
            score_single_pick_feat(&fg, &fh, &mut best_by_sim);
        }
    }
    best_by_sim
}

fn classify_pick_card(
    frame: &FrameView<'_>,
    bx: [i32; 4],
    (observed_title_glyphs, observed_title_lines): (usize, usize),
) -> Option<RelicCardRead> {
    assert!(observed_title_glyphs >= 4);
    assert!(bx[2] >= bx[0] && bx[3] >= bx[1]);
    let best_by_sim = score_pick_card_templates(frame, bx);
    let mut top_sim = ToolId::Revenge;
    let mut top_raw_sc = f32::NEG_INFINITY;
    let mut top_sc = f32::NEG_INFINITY;
    let mut second_sc = f32::NEG_INFINITY;
    for (&sim_id, &raw_sc) in tartarus_core::ALL_TOOLS.iter().zip(best_by_sim.iter()) {
        if !raw_sc.is_finite() {
            continue;
        }
        let expected_chars = sim_id.name_alnum_len();
        let diff = observed_title_glyphs.abs_diff(expected_chars);
        let char_adj = if diff == 0 {
            0.035
        } else if diff <= 1 {
            0.025
        } else if diff <= 2 {
            0.012
        } else if diff <= 4 {
            0.0
        } else {
            -((diff as f32 - 4.0) * 0.008).min(0.08)
        };
        let line_adj = if (observed_title_lines >= 2 && expected_chars <= 21)
            || (observed_title_lines == 1 && expected_chars >= 26)
        {
            -0.08
        } else {
            0.0
        };
        let sc = raw_sc + char_adj + line_adj;
        if sc > top_sc {
            second_sc = top_sc;
            top_sc = sc;
            top_raw_sc = raw_sc;
            top_sim = sim_id;
        } else if sc > second_sc {
            second_sc = sc;
        }
    }
    if !top_raw_sc.is_finite() || top_raw_sc < 0.80 {
        return None;
    }
    let second = if second_sc.is_finite() {
        second_sc
    } else {
        0.0
    };
    let margin = top_sc - second;
    let eff_sc = top_sc.max(top_raw_sc);
    let conf = if eff_sc < 0.82 {
        0.0
    } else {
        ((eff_sc - 0.85) * 5.0 + margin * 8.0).clamp(0.0, 1.0)
    };
    Some(RelicCardRead {
        sim: top_sim,
        score: eff_sc,
        conf,
    })
}

pub(crate) fn read_relic_modal_with_slot(
    frame: &FrameView<'_>,
    slot: &mut UiScratchSlot,
) -> Option<RelicModalRead> {
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    if !pick_confirm_shown(frame) || !pick_modal_backdrop_ok(frame) {
        return None;
    }
    let s = (frame.height() as f64) / 1080.0;
    for k in [-1, 0, 1] {
        let c = (frame.width() as f64) * 0.5 + (k as f64) * 480.0 * s;
        let tx0 = (c - 180.0 * s).round() as i32;
        let tx1 = (c + 180.0 * s).round() as i32;
        let ty0 = (420.0 * s).round() as i32;
        let ty1 = (550.0 * s).round() as i32;
        if (count_neutral_light(frame, tx0, ty0, tx1, ty1) as f64) < 10.0 * s * s {
            return None;
        }
    }
    let art_boxes = [
        pick_art_box(frame, -1)?,
        pick_art_box(frame, 0)?,
        pick_art_box(frame, 1)?,
    ];
    for bx in &art_boxes {
        if ((bx[2] - bx[0]) as f64) < 60.0 * s || ((bx[3] - bx[1]) as f64) < 60.0 * s {
            return None;
        }
    }
    let mut cards = [RelicCardRead {
        sim: ToolId::Revenge,
        score: 0.0,
        conf: 0.0,
    }; 3];
    for (idx, k) in [-1, 0, 1].into_iter().enumerate() {
        let title_info = count_pick_title_glyphs(frame, k, s, slot);
        if title_info.0 < 4 {
            return None;
        }
        cards[idx] = classify_pick_card(frame, art_boxes[idx], title_info)?;
    }
    if cards[0].sim == cards[1].sim || cards[1].sim == cards[2].sim || cards[0].sim == cards[2].sim
    {
        return None;
    }
    if cards.iter().all(|c| c.conf < 0.20) {
        return None;
    }
    Some(RelicModalRead {
        cards,
        reroll: pick_reroll_shown(frame),
    })
}

fn classify_gift_card(frame: &FrameView<'_>, bx: [i32; 4]) -> Option<GiftCardRead> {
    assert!(bx[2] >= bx[0]);
    assert!(bx[3] >= bx[1]);
    let (fg0, fh0) = art_feature(frame, bx[0], bx[1], bx[2], bx[3]);
    let mut top_gift = GiftId::Ring;
    let mut top_sc = f32::NEG_INFINITY;
    let mut second_sc = f32::NEG_INFINITY;
    for r in &REF_GIFTS {
        let sc = art_similarity(&fg0, &fh0, &r.grid, &r.hist);
        if sc > top_sc {
            second_sc = top_sc;
            top_sc = sc;
            top_gift = r.sim;
        } else if sc > second_sc {
            second_sc = sc;
        }
    }
    if !top_sc.is_finite() || top_sc < 0.78 {
        return None;
    }
    let margin = top_sc - second_sc.max(0.0);
    let conf = ((top_sc - 0.80) * 5.0 + margin * 8.0).clamp(0.0, 1.0);
    Some(GiftCardRead {
        gift: top_gift,
        score: top_sc,
        conf,
    })
}

pub fn read_gift_modal(frame: &FrameView<'_>) -> Option<GiftModalRead> {
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    if !pick_confirm_shown(frame) || !pick_modal_backdrop_ok(frame) {
        return None;
    }
    let art_boxes = [
        pick_art_box(frame, -1)?,
        pick_art_box(frame, 0)?,
        pick_art_box(frame, 1)?,
    ];
    let cards = [
        classify_gift_card(frame, art_boxes[0])?,
        classify_gift_card(frame, art_boxes[1])?,
        classify_gift_card(frame, art_boxes[2])?,
    ];
    if cards[0].gift == cards[1].gift
        || cards[1].gift == cards[2].gift
        || cards[0].gift == cards[2].gift
    {
        return None;
    }
    Some(GiftModalRead { cards })
}

#[derive(Clone, Copy, Debug)]
pub enum TitleModalRead {
    Objective(ObjectiveId),
    StartEvent(StartEventId),
}

fn is_opulence_popup(frame: &FrameView<'_>) -> bool {
    let w = frame.width();
    let h = frame.height();
    if w == 0 || h == 0 {
        return false;
    }
    assert!(w > 0);
    assert!(h > 0);
    let mut crimson = 0usize;
    let mut purple = 0usize;
    let ex0 = ((w as f64) * 0.46) as usize;
    let ex1 = (((w as f64) * 0.54) as usize).min(w);
    let ey0 = ((h as f64) * 0.16) as usize;
    let ey1 = (((h as f64) * 0.27) as usize).min(h);
    for y in ey0..ey1 {
        let row = frame.row_offset(y);
        for x in ex0..ex1 {
            let idx = row + x * 4;
            let r = frame.pixels_rgba8[idx] as i32;
            let g = frame.pixels_rgba8[idx + 1] as i32;
            let b = frame.pixels_rgba8[idx + 2] as i32;
            if r > 110 && r > b + 45 && r > g + 35 {
                crimson += 1;
            } else if b > 70 && r > 50 && b > g + 15 && r > g + 10 && r <= b + 30 {
                purple += 1;
            }
        }
    }

    let mut cyan_line1 = 0usize;
    let lx0 = ((w as f64) * 0.45) as usize;
    let lx1 = (((w as f64) * 0.65) as usize).min(w);
    let ly0 = ((h as f64) * 0.365) as usize;
    let ly1 = (((h as f64) * 0.402) as usize).min(h);
    for y in ly0..ly1 {
        let row = frame.row_offset(y);
        for x in lx0..lx1 {
            let idx = row + x * 4;
            let r = frame.pixels_rgba8[idx] as i32;
            let g = frame.pixels_rgba8[idx + 1] as i32;
            let b = frame.pixels_rgba8[idx + 2] as i32;
            if b > 130 && g > 120 && b > r + 40 && g > r + 30 {
                cyan_line1 += 1;
            }
        }
    }

    crimson > purple.saturating_mul(2).max(10) && cyan_line1 < 5
}

fn match_title_piece(
    p: &Piece,
    line_h: i32,
    img_w: usize,
    patterns: &[(TitleModalRead, &[isize])],
    best_match: &mut Option<(TitleModalRead, i32, isize)>,
) {
    assert!(line_h > 0);
    assert!(img_w > 0);
    let words: Vec<isize> = p.words.iter().map(|w| w.glyphs.len() as isize).collect();
    if words.is_empty() {
        return;
    }
    let has_obj_prefix = words[0] >= 8 && words[0] <= 11 && words.len() > 1;
    let name = if has_obj_prefix {
        &words[1..]
    } else {
        &words[..]
    };
    let mid_x = ((p.x0 + p.x1) as f64) * 0.5;
    let centered = (mid_x - (img_w as f64) * 0.5).abs() <= (img_w as f64) * 0.12;
    for &(id, want) in patterns {
        if has_obj_prefix && matches!(id, TitleModalRead::StartEvent(_)) {
            continue;
        }
        if want.len() == 1 && matches!(id, TitleModalRead::StartEvent(_)) && !centered {
            continue;
        }
        if want.len() != name.len() {
            continue;
        }
        if want
            .iter()
            .zip(name.iter())
            .any(|(&a, &b)| (a - b).abs() > if a >= 8 { 2 } else { 1 })
        {
            continue;
        }
        let off: isize = want
            .iter()
            .zip(name.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        let want_total: isize = want.iter().sum();
        if (off as f64) <= 1.0 + (want_total as f64) * 0.15 {
            let better = best_match.is_none_or(|(_, bh, boff)| {
                let same_tier = (line_h - bh).abs() <= bh.max(line_h) / 5;
                if same_tier {
                    off < boff || (off == boff && line_h > bh)
                } else {
                    line_h > bh
                }
            });
            if better {
                *best_match = Some((id, line_h, off));
            }
        }
    }
}

pub(crate) fn read_title_modal_with_slot(
    frame: &FrameView<'_>,
    slot: &mut UiScratchSlot,
) -> Option<TitleModalRead> {
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    let patterns: &[(TitleModalRead, &[isize])] = &[
        (
            TitleModalRead::Objective(ObjectiveId::Consume),
            &[8, 2, 3, 7],
        ),
        (TitleModalRead::Objective(ObjectiveId::Consume), &[8, 5, 7]),
        (TitleModalRead::Objective(ObjectiveId::Doors), &[6, 4, 5]),
        (TitleModalRead::Objective(ObjectiveId::Incident), &[13]),
        (TitleModalRead::Objective(ObjectiveId::King), &[4, 2, 3, 5]),
        (TitleModalRead::Objective(ObjectiveId::King), &[4, 5, 5]),
        (TitleModalRead::Objective(ObjectiveId::Walk), &[6, 4]),
        (TitleModalRead::StartEvent(StartEventId::CageFight), &[4, 5]),
        (TitleModalRead::StartEvent(StartEventId::CageFight), &[4, 4]),
        (TitleModalRead::StartEvent(StartEventId::IronLaw), &[4, 3]),
        (TitleModalRead::StartEvent(StartEventId::Tyranny), &[7]),
        (TitleModalRead::StartEvent(StartEventId::Tyranny), &[8]),
    ];

    let mut best_match: Option<(TitleModalRead, i32, isize)> = None;
    let radii = text_radii(frame);
    let w = frame.width() as f64;
    let h = frame.height() as f64;
    let h_bounds = (6.0, 48.0_f64.max(h * 0.07));
    if let Some(dims) = prepare_sat_region(
        frame,
        [w * 0.28, h * 0.21, w * 0.72, h * 0.33],
        radii[2].max(1),
        0.34,
        slot,
    ) {
        for lines in scan_radii_from_built_sat(dims, h_bounds, &radii[1..3], slot) {
            for line in lines {
                for p in &line.pieces {
                    match_title_piece(p, line.h, frame.width(), patterns, &mut best_match);
                }
            }
        }
    }
    if best_match.is_none() {
        for lines in text_lines_radii_with_slot(
            frame,
            [w * 0.25, h * 0.12, w * 0.75, h * 0.40],
            h_bounds,
            &radii,
            0.34,
            slot,
        ) {
            for line in lines {
                for p in &line.pieces {
                    match_title_piece(p, line.h, frame.width(), patterns, &mut best_match);
                }
            }
        }
    }
    best_match.map(|(id, _, _)| match id {
        TitleModalRead::StartEvent(StartEventId::Tyranny) if is_opulence_popup(frame) => {
            TitleModalRead::StartEvent(StartEventId::Opulence)
        }
        other => other,
    })
}

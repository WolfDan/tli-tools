//! HUD element readers: bottom door cards, decision cards, days remaining, torture instruments panel, chromatic vault, and HUD objective.

use tartarus_core::{FixedVec, ObjectiveId, ToolId};

use super::arena::{Blob, ScanScratchSlot};
use super::data::{CARD_FRAME, CARD_LIFT, PREFIX_NUM, REF_TOOLS};
use super::image::{ImageView, hsv};
use super::math::{BoardLoc, cosine};
use super::ocr::{
    ALLOWED_CELLS, ALLOWED_DAYS, LabelBox, Line, Piece, prefix_like, read_number, split_glyph,
    text_lines_radii_with_slot, text_radii, text_radii_for_scale,
};

#[derive(Clone, Debug)]
pub struct DoorsRead {
    pub cells: FixedVec<u8, 4>,
    pub conf: f32,
    pub candidates: Vec<Vec<(u8, f32)>>,
    pub office_card: bool,
}

fn solve_door_assignment(
    candidates: &[Vec<(u8, f32)>],
    office_card: bool,
    warden_cell: Option<u8>,
) -> Option<(FixedVec<u8, 4>, f32)> {
    let k = candidates.len();
    if k == 0 {
        if office_card {
            let mut cells = FixedVec::new();
            cells.push(0);
            return Some((cells, 0.85));
        }
        return None;
    }

    struct DoorSearch<'a> {
        k: usize,
        candidates: &'a [Vec<(u8, f32)>],
        warden_cell: Option<u8>,
        cur: [u8; 3],
        best_adj: f32,
        best_raw: f32,
        best_cells: FixedVec<u8, 4>,
    }

    impl DoorSearch<'_> {
        fn step(&mut self, idx: usize, adj_sum: f32, raw_sum: f32) {
            if idx == self.k {
                if adj_sum > self.best_adj {
                    self.best_adj = adj_sum;
                    self.best_raw = raw_sum;
                    self.best_cells.clear();
                    self.best_cells.extend_from_slice(&self.cur[..self.k]);
                }
                return;
            }
            for &(cell_idx, sc) in self.candidates[idx].iter().take(4) {
                if self.cur[..idx].contains(&cell_idx) {
                    continue;
                }
                let penalty = if self.warden_cell == Some(cell_idx) {
                    0.16
                } else {
                    0.0
                };
                self.cur[idx] = cell_idx;
                self.step(idx + 1, adj_sum + sc - penalty, raw_sum + sc);
            }
        }
    }

    let mut searcher = DoorSearch {
        k: k.min(3),
        candidates,
        warden_cell,
        cur: [0u8; 3],
        best_adj: f32::NEG_INFINITY,
        best_raw: 0.0,
        best_cells: FixedVec::new(),
    };
    searcher.step(0, 0.0, 0.0);
    let mut best_cells = searcher.best_cells;
    let mut best_raw_score = searcher.best_raw;

    if best_cells.is_empty() {
        for list in candidates {
            if let Some(&(c, sc)) = list.first() {
                if !best_cells.contains(&c) {
                    let _ = best_cells.try_push(c);
                }
                best_raw_score += sc;
            }
        }
    }

    if office_card && !best_cells.contains(&0) {
        let _ = best_cells.try_push(0);
    }
    let avg_score = best_raw_score / (k.max(1) as f32);
    let conf = ((avg_score - 0.30) * 1.6).clamp(0.1, 1.0);
    Some((best_cells, conf))
}

pub fn refine_doors_with_warden(doors: &mut Option<DoorsRead>, warden_cell: Option<u8>) {
    let Some(d) = doors else {
        return;
    };
    if d.candidates.is_empty() {
        return;
    }
    if let Some((cells, conf)) = solve_door_assignment(&d.candidates, d.office_card, warden_cell) {
        d.cells = cells;
        d.conf = conf;
    }
}

pub(crate) fn read_doors_from_lines(
    img: &ImageView<'_>,
    lines_by_r: &[Vec<Line>],
) -> Option<DoorsRead> {
    let mut best_group: Vec<(Piece, i32, f64, Vec<Blob>, f32)> = Vec::new();
    let mut best_fit = f32::INFINITY;
    let mut all_pieces: Vec<Piece> = Vec::new();

    for lines in lines_by_r {
        let mut cards = Vec::new();
        for line in lines {
            for p in &line.pieces {
                all_pieces.push(p.clone());
                let cap = p.glyphs[0].h as f64;
                if cap < 6.0 || p.glyphs.len() < 4 {
                    continue;
                }
                let span = ((p.x1 - p.x0) as f64) / cap;
                if !(5.9..=7.8).contains(&span) {
                    continue;
                }
                let mut start = (p.x0 as f64) + PREFIX_NUM * cap;
                let mut gap_best = 0i32;
                for i in 0..(p.glyphs.len() - 1) {
                    let end = ((p.glyphs[i].x1 - p.x0) as f64) / cap;
                    let gap = p.glyphs[i + 1].x0 - p.glyphs[i].x1;
                    if (4.9..=6.1).contains(&end) && gap > gap_best {
                        gap_best = gap;
                        start = p.glyphs[i + 1].x0 as f64;
                    }
                }
                let num: Vec<Blob> = p
                    .glyphs
                    .iter()
                    .copied()
                    .filter(|g| (g.x0 as f64) >= start && (g.h as f64) >= 0.45 * cap)
                    .collect();
                if num.is_empty() || num.len() > 2 {
                    continue;
                }
                let like = prefix_like(img, p);
                if like < 0.50 {
                    continue;
                }
                let mid = (p.y0 + p.y1) as f64 * 0.5;
                cards.push((p.clone(), cap as i32, mid, num, 1.0 - like));
            }
        }
        for c in &cards {
            let group: Vec<_> = cards
                .iter()
                .filter(|d| {
                    (d.2 - c.2).abs() < 2.5 * (c.1 as f64)
                        && ((d.1 - c.1).abs() as f64) < 0.3 * (c.1 as f64)
                })
                .cloned()
                .collect();
            if group.len() > 3 {
                continue;
            }
            let fit: f32 = group.iter().map(|d| d.4).sum();
            if group.len() > best_group.len() || (group.len() == best_group.len() && fit < best_fit)
            {
                best_group = group;
                best_fit = fit;
            }
        }
    }

    let office_card = if !best_group.is_empty() {
        let h = best_group[0].1 as f64;
        all_pieces.iter().any(|p| {
            let cap = p.glyphs[0].h as f64;
            let mid = (p.y0 + p.y1) as f64 * 0.5;
            if (cap - h).abs() > 0.35 * h || !best_group.iter().any(|c| (c.2 - mid).abs() < 2.2 * h)
            {
                return false;
            }
            if best_group.iter().any(|c| c.0.x0 < p.x1 && p.x0 < c.0.x1) {
                return false;
            }
            let span = ((p.x1 - p.x0) as f64) / cap;
            let glyph_fill: f64 = p.glyphs.iter().map(|g| g.w.max(1) as f64).sum::<f64>() / cap;
            (9.5..=20.0).contains(&span) && glyph_fill >= 5.5 && p.glyphs.len() >= 10
        })
    } else {
        false
    };

    if best_group.len() < 2 && !(best_group.len() == 1 && office_card) {
        return None;
    }
    best_group.sort_by_key(|c| c.0.x0);

    let mut candidates: Vec<Vec<(u8, f32)>> = Vec::with_capacity(best_group.len());
    for c in &best_group {
        let mut whole = read_number(img, &c.3, c.1, &ALLOWED_CELLS);
        if c.3.len() == 1
            && (c.3[0].w as f64) > 0.72 * (c.1 as f64)
            && let Some(parts) = split_glyph(img, &c.3[0])
        {
            whole.extend(read_number(img, &parts, c.1, &ALLOWED_CELLS));
            whole.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        }
        if whole.is_empty() {
            return None;
        }
        let mut card_cands: Vec<(u8, f32)> = Vec::with_capacity(whole.len());
        for (val, sc) in whole {
            let idx = (val - 1) as u8;
            if !card_cands.iter().any(|&(existing, _)| existing == idx) {
                card_cands.push((idx, sc));
            }
        }
        candidates.push(card_cands);
    }

    let (cells, conf) = solve_door_assignment(&candidates, office_card, None)?;
    Some(DoorsRead {
        cells,
        conf,
        candidates,
        office_card,
    })
}

fn extract_days_from_lines(img: &ImageView<'_>, lines: &[Line]) -> Option<(u8, f32)> {
    let tallest = lines
        .iter()
        .flat_map(|l| {
            l.glyphs
                .iter()
                .filter(|g| (g.w as f64) >= 0.25 * (g.h as f64))
                .map(|g| g.h)
        })
        .max()
        .unwrap_or(1)
        .max(1) as f64;

    let pair = |a: &Blob, b: &Blob| -> bool {
        ((b.h - a.h).abs() as f64) < 0.2 * (a.h as f64)
            && ((b.x0 - a.x1) as f64) < 0.7 * (a.h as f64)
            && (b.w as f64) < (b.h as f64) * 1.1
            && (a.w as f64) < (a.h as f64) * 1.1
    };

    let mut best: Option<(f64, Vec<Blob>, i32)> = None;
    for line in lines {
        let glyphs: Vec<Blob> = line
            .glyphs
            .iter()
            .copied()
            .filter(|g| {
                (g.w as f64) >= 0.25 * (g.h as f64) && (g.h as f64) >= (img.height as f64) * 0.016
            })
            .collect();
        for i in 0..glyphs.len() {
            let a = glyphs[i];
            if a.h < 8
                || (a.h as f64) > (img.height as f64) * 0.09
                || (a.w as f64) > (a.h as f64) * 1.1
                || (i > 0 && pair(&glyphs[i - 1], &a))
            {
                continue;
            }
            let mut word = vec![a];
            if i + 1 < glyphs.len() && pair(&a, &glyphs[i + 1]) {
                word.push(glyphs[i + 1]);
            }
            let end = word.last().unwrap().x1;
            let after: Vec<&Blob> = glyphs
                .iter()
                .filter(|g| g.x0 > end && (g.x0 - end) < 6 * a.h)
                .collect();
            let smaller = after
                .iter()
                .filter(|g| (g.h as f64) < 0.85 * (a.h as f64))
                .count();
            if !after.is_empty() && smaller * 2 < after.len() {
                continue;
            }
            let cx = (a.x0 + end) as f64 * 0.5;
            let half_w = (img.width as f64) * 0.5;
            let sc = (smaller.min(8) as f64)
                + 6.0 * (1.0 - (cx - half_w).abs() / half_w)
                + 6.0 * ((a.h as f64) / tallest);
            if best.as_ref().is_none_or(|b| sc > b.0) {
                best = Some((sc, word, a.h));
            }
        }
    }

    let (_, word, h_val) = best?;
    let res = read_number(img, &word, h_val, &ALLOWED_DAYS);
    let &(days, score) = res.first()?;
    Some((days as u8, ((score - 0.30) * 1.6).clamp(0.1, 1.0)))
}

pub fn read_days_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    slot: &mut ScanScratchSlot,
) -> Option<(u8, f32)> {
    if loc.by_labels {
        return None;
    }
    let top = loc.at(0.0, 0.0).1.min(loc.at(5.0, 0.0).1) - 95.0 * loc.s;
    if top < 8.0 {
        return None;
    }
    let radii = text_radii_for_scale(loc.s);
    let max_h = (72.0 * loc.s).max(24.0);
    let y1 = top.min(115.0 * loc.s);
    let fast_lines: Vec<Line> = text_lines_radii_with_slot(
        img,
        [
            (img.width as f64) * 0.35,
            0.0,
            (img.width as f64) * 0.65,
            y1,
        ],
        (8.0, max_h),
        &radii[1..4],
        1.01,
        slot,
    )
    .into_iter()
    .flatten()
    .collect();
    let fast_res = extract_days_from_lines(img, &fast_lines);
    if let Some((days, conf)) = fast_res
        && conf >= 0.45
    {
        return Some((days, conf));
    }
    if fast_lines.is_empty() {
        return None;
    }

    let rem_radii = [radii[0]];
    let lines: Vec<Line> = text_lines_radii_with_slot(
        img,
        [
            (img.width as f64) * 0.32,
            0.0,
            (img.width as f64) * 0.68,
            top,
        ],
        (8.0, max_h),
        &rem_radii,
        1.01,
        slot,
    )
    .into_iter()
    .flatten()
    .collect();
    match (fast_res, extract_days_from_lines(img, &lines)) {
        (Some(a), Some(b)) => Some(if b.1 > a.1 { b } else { a }),
        (Some(a), None) => Some(a),
        (None, b) => b,
    }
}

pub fn art_feature(
    img: &ImageView<'_>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) -> ([f32; 72], [f32; 13]) {
    let gx = 4usize;
    let gy = 6usize;
    let mut grid = [0.0_f32; 72];
    let mut hist = [0.0_f32; 13];
    let cx0 = x0.clamp(0, img.width as i32 - 1);
    let cy0 = y0.clamp(0, img.height as i32 - 1);
    let cx1 = x1.clamp(cx0, img.width as i32 - 1);
    let cy1 = y1.clamp(cy0, img.height as i32 - 1);
    let w = (cx1 - cx0 + 1).max(1) as usize;
    let h = (cy1 - cy0 + 1).max(1) as usize;

    for j in 0..gy {
        let py0 = (cy0 as usize) + (j * h) / gy;
        let py1 = (cy0 as usize) + ((j + 1) * h) / gy;
        for i in 0..gx {
            let px0 = (cx0 as usize) + (i * w) / gx;
            let px1 = (cx0 as usize) + ((i + 1) * w) / gx;
            let mut r = 0.0_f32;
            let mut g = 0.0_f32;
            let mut b = 0.0_f32;
            let mut n = 0usize;
            for y in py0..py1 {
                let row_base = y * img.width * 4;
                for x in px0..px1 {
                    let o = row_base + x * 4;
                    let rr = img.data[o];
                    let gg = img.data[o + 1];
                    let bb = img.data[o + 2];
                    let rf = rr as f32;
                    let gf = gg as f32;
                    let bf = bb as f32;
                    r += rf;
                    g += gf;
                    b += bf;
                    n += 1;
                    let mx = rr.max(gg).max(bb);
                    let mn = rr.min(gg).min(bb);
                    if mx > 63 && ((mx - mn) as u32) * 4 > (mx as u32) {
                        let (hh, ss, vv) = hsv(rf, gf, bf);
                        hist[((hh / 30.0).floor() as usize) % 12] += ss * vv;
                    } else {
                        hist[12] += (mx as f32) * (0.3 / 255.0);
                    }
                }
            }
            let o = (j * gx + i) * 3;
            if n > 0 {
                let nf = n as f32;
                grid[o] = r / nf;
                grid[o + 1] = g / nf;
                grid[o + 2] = b / nf;
            }
        }
    }
    (grid, hist)
}

#[inline]
pub fn art_similarity(
    a_grid: &[f32; 72],
    a_hist: &[f32; 13],
    b_grid: &[f32; 72],
    b_hist: &[f32; 13],
) -> f32 {
    let am = a_grid.iter().sum::<f32>() / 72.0;
    let bm = b_grid.iter().sum::<f32>() / 72.0;
    let mut ac = [0.0_f32; 72];
    let mut bc = [0.0_f32; 72];
    for i in 0..72 {
        ac[i] = a_grid[i] - am;
        bc[i] = b_grid[i] - bm;
    }
    0.5 * cosine(a_hist, b_hist) + 0.5 * (0.5 + 0.5 * cosine(&ac, &bc))
}

#[derive(Clone, Copy, Debug)]
pub struct ScannedTool {
    pub id: &'static str,
    pub sim: ToolId,
    pub name: &'static str,
    pub score: f32,
    pub conf: f32,
}

impl Default for ScannedTool {
    fn default() -> Self {
        Self {
            id: "",
            sim: ToolId::Revenge,
            name: "",
            score: 0.0,
            conf: 0.0,
        }
    }
}

pub fn is_overlay_occluded(img: &ImageView<'_>, x0: f64, y0: f64, x1: f64, y1: f64) -> bool {
    let ix0 = (x0.round() as isize).clamp(0, img.width as isize - 1) as usize;
    let ix1 = (x1.round() as isize).clamp(ix0 as isize, img.width as isize - 1) as usize;
    let iy0 = (y0.round() as isize).clamp(0, img.height as isize - 1) as usize;
    let iy1 = (y1.round() as isize).clamp(iy0 as isize, img.height as isize - 1) as usize;
    let mut cool = 0usize;
    let mut total = 0usize;
    let mut y = iy0;
    while y <= iy1 {
        let mut x = ix0;
        while x <= ix1 {
            let (r, g, b) = img.rgb_at(x as isize, y as isize);
            total += 1;
            if b > r.saturating_add(3) && g >= r {
                cool += 1;
            }
            x += 2;
        }
        y += 2;
    }
    total > 0 && cool * 3 >= total
}

pub fn read_tools_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    _slot: &mut ScanScratchSlot,
) -> (FixedVec<ScannedTool, 6>, Option<LabelBox>) {
    let left = loc.at(0.0, 0.0).0.min(loc.at(0.0, 3.0).0) - 70.0 * loc.s;
    if left < 30.0 {
        return (FixedVec::new(), None);
    }
    let h = 11.17 * loc.s;
    let mean_v = |x0: f64, y0: f64, x1: f64, y1: f64| -> f64 {
        let mut sum = 0.0_f64;
        let mut n = 0usize;
        let iy0 = (y0.round() as isize).max(0) as usize;
        let iy1 = (y1 as isize).min(img.height as isize - 1);
        let ix0 = (x0.round() as isize).max(0) as usize;
        let ix1 = (x1 as isize).min(img.width as isize - 1);
        if iy1 < iy0 as isize || ix1 < ix0 as isize {
            return 0.0;
        }
        for y in iy0..=(iy1 as usize) {
            for x in ix0..=(ix1 as usize) {
                let (r, g, b) = img.rgb_at(x as isize, y as isize);
                sum += (r.max(g).max(b) as f64) / 255.0;
                n += 1;
            }
        }
        if n > 0 { sum / (n as f64) } else { 0.0 }
    };

    let card_frame_edge = |x0: f64, y0: f64, x1: f64, y1: f64| -> f64 {
        let w = (0.1 * h).round().max(2.0);
        let reach = (0.15 * h).round().max(2.0) as isize;
        let ya = y0 + 0.8 * h;
        let yb = y1 - 0.8 * h;
        let xa = x0 + 0.8 * h;
        let xb = x1 - 0.8 * h;
        let step = |f: &dyn Fn(f64) -> f64| -> f64 {
            let mut best = 0.0_f64;
            for o in -reach..=reach {
                best = best.max(f(o as f64).abs());
            }
            best
        };
        let mut edges = [
            step(&|o| {
                mean_v(x0 + o - w, ya, x0 + o - 1.0, yb) - mean_v(x0 + o + 1.0, ya, x0 + o + w, yb)
            }),
            step(&|o| {
                mean_v(x1 + o - w, ya, x1 + o - 1.0, yb) - mean_v(x1 + o + 1.0, ya, x1 + o + w, yb)
            }),
            step(&|o| {
                mean_v(xa, y0 + o - w, xb, y0 + o - 1.0) - mean_v(xa, y0 + o + 1.0, xb, y0 + o + w)
            }),
            step(&|o| {
                mean_v(xa, y1 + o - w, xb, y1 + o - 1.0) - mean_v(xa, y1 + o + 1.0, xb, y1 + o + w)
            }),
        ];
        edges.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (edges[1] + edges[2]) * 0.5
    };

    // Direct UMG left-panel anchor with sub-millisecond Slot-0 frame micro-alignment (zero OCR)
    let base_x0 = (54.0 * loc.s + if loc.tx < -10.0 * loc.s { loc.tx } else { 0.0 }).max(0.0);
    let base_y1 = loc.ty + 210.4 * loc.s;
    let mut best_dx = 0.0_f64;
    let mut best_dy = 0.0_f64;
    let mut best_edge = -1.0_f64;
    for dx_u in [-1.5, 0.0, 1.5] {
        let dx = dx_u * loc.s;
        for dy_u in [-2.0, 0.0, 2.0] {
            let dy = dy_u * loc.s;
            let sx0 = base_x0 + dx + 0.15 * h;
            let sx1 = sx0 + 4.45 * h;
            let sy0 = base_y1 + dy + 1.55 * h;
            let sy1 = sy0 + 7.45 * h;
            if sx1 <= left {
                let edge = card_frame_edge(sx0, sy0, sx1, sy1);
                if edge > best_edge {
                    best_edge = edge;
                    best_dx = dx;
                    best_dy = dy;
                }
            }
        }
    }
    let label = LabelBox {
        x0: (base_x0 + best_dx).round() as i32,
        x1: (base_x0 + best_dx + 12.2 * h).round() as i32,
        y0: (base_y1 + best_dy - 10.6 * loc.s).round() as i32,
        y1: (base_y1 + best_dy).round() as i32,
        h: h.round().max(1.0) as i32,
    };

    let mut out = FixedVec::new();
    for i in 0..6 {
        let x0 = (label.x0 as f64) + (0.15 + 6.05 * (i as f64)) * h;
        let x1 = x0 + 4.45 * h;
        let y0 = (label.y1 as f64) + 1.55 * h;
        let y1 = y0 + 7.45 * h;
        if x1 > left {
            break;
        }
        let mut sy0 = (y0 + 0.4 * h).round().clamp(0.0, img.height as f64 - 1.0) as i32;
        let sy1 = (y1 - 0.4 * h).floor().clamp(0.0, img.height as f64 - 1.0) as i32;
        let sx0 = (x0 + 0.4 * h).round().clamp(0.0, img.width as f64 - 1.0) as i32;
        let sx1 = (x1 - 0.4 * h).floor().clamp(0.0, img.width as f64 - 1.0) as i32;
        while sy0 < sy1 {
            let mut cool_px = 0usize;
            let mut tot_px = 0usize;
            for x in sx0..=sx1 {
                let (r, g, b) = img.rgb_at(x as isize, sy0 as isize);
                tot_px += 1;
                if b > r.saturating_add(2) && g >= r {
                    cool_px += 1;
                }
            }
            if tot_px > 0 && cool_px * 3 >= tot_px {
                sy0 += 1;
            } else {
                break;
            }
        }
        // If we skipped overlay rows at the top, skip 1 extra border row
        if sy0 > (y0 + 0.4 * h).round() as i32 && sy0 < sy1 {
            sy0 += 1;
        }
        let inside = mean_v(
            x0 + 0.5 * h,
            (sy0 as f64).max(y0 + 0.5 * h),
            x1 - 0.5 * h,
            y1 - 0.5 * h,
        );
        let wall = mean_v(x1 + 0.35 * h, (sy0 as f64).max(y0), x1 + 1.3 * h, y1);
        let thr = 0.16_f32.max((inside * 1.9) as f32);
        let mut ax0 = i32::MAX;
        let mut ay0 = i32::MAX;
        let mut ax1 = -1i32;
        let mut ay1 = -1i32;
        let mut art = 0usize;
        for y in sy0..=sy1 {
            let row_base = (y as usize) * img.width * 4;
            for x in sx0..=sx1 {
                let o = row_base + (x as usize) * 4;
                let r = img.data[o];
                let g = img.data[o + 1];
                let b = img.data[o + 2];
                let mx = r.max(g).max(b);
                let v = (mx as f32) * (1.0 / 255.0);
                if v > thr {
                    let mn = r.min(g).min(b);
                    if ((mx - mn) as u32) * 5 > (mx as u32) || v > thr * 1.3 {
                        art += 1;
                        ax0 = ax0.min(x);
                        ax1 = ax1.max(x);
                        ay0 = ay0.min(y);
                        ay1 = ay1.max(y);
                    }
                }
            }
        }
        if (art as f64) < 1.0 * h * h && !((art as f64) > 0.5 * h * h && inside > wall + 0.012) {
            break;
        }
        let bw = (ax1 - ax0 + 1) as f64;
        let bh = (ay1 - ay0 + 1) as f64;
        if (art as f64) / (bw * bh).max(1.0) < 0.065 || bw > 2.0 * bh {
            break;
        }
        if card_frame_edge(x0, y0, x1, y1) < CARD_FRAME && inside - wall < CARD_LIFT {
            break;
        }

        let (fg0, fh0) = art_feature(img, ax0, ay0, ax1, ay1);
        let mut best_entry = &REF_TOOLS[0];
        let mut best_sc = f32::NEG_INFINITY;
        let mut second_sc = f32::NEG_INFINITY;
        for t in &REF_TOOLS {
            let sc = art_similarity(&fg0, &fh0, &t.grid, &t.hist);
            if sc > best_sc {
                if t.id != best_entry.id {
                    second_sc = best_sc;
                }
                best_sc = sc;
                best_entry = t;
            } else if t.id != best_entry.id && sc > second_sc {
                second_sc = sc;
            }
        }
        if best_sc < 0.86 || (best_sc - second_sc) < 0.04 {
            let extra_feats: [([f32; 72], [f32; 13]); 4] = [
                art_feature(img, ax0 - 1, ay0, ax1 - 1, ay1),
                art_feature(img, ax0 + 1, ay0, ax1 + 1, ay1),
                art_feature(img, ax0, ay0 - 1, ax1, ay1 - 1),
                art_feature(img, ax0, ay0 + 1, ax1, ay1 + 1),
            ];
            best_entry = &REF_TOOLS[0];
            best_sc = f32::NEG_INFINITY;
            second_sc = f32::NEG_INFINITY;
            for t in &REF_TOOLS {
                let sc0 = art_similarity(&fg0, &fh0, &t.grid, &t.hist);
                let sc = extra_feats
                    .iter()
                    .map(|(fg, fh)| art_similarity(fg, fh, &t.grid, &t.hist))
                    .fold(sc0, f32::max);
                if sc > best_sc {
                    if t.id != best_entry.id {
                        second_sc = best_sc;
                    }
                    best_sc = sc;
                    best_entry = t;
                } else if t.id != best_entry.id && sc > second_sc {
                    second_sc = sc;
                }
            }
        }
        let second = if second_sc.is_finite() {
            second_sc
        } else {
            0.0
        };
        let margin = best_sc - second;
        let conf = ((best_sc - 0.60) * 2.5 + margin * 5.0).clamp(0.1, 1.0);
        out.push(ScannedTool {
            id: best_entry.id,
            sim: best_entry.sim,
            name: best_entry.name,
            score: best_sc,
            conf,
        });
    }
    (out, Some(label))
}

pub fn read_chromatic_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    slot: &mut ScanScratchSlot,
) -> Option<usize> {
    let right = loc.at(5.0, 0.0).0.max(loc.at(5.0, 3.0).0) + 60.0 * loc.s;
    if right > (img.width as f64) - 30.0 {
        return None;
    }
    let u = loc.s;
    // Direct UMG right-panel anchor (integral image 2x5 grid search handles ±20*u alignment directly in O(1))
    let cx = (loc.tx + 1259.0 * u).min((img.width as f64) - 21.0 * u);
    let cy = loc.ty + 96.65 * u;
    let x0 = ((cx - 110.0 * u).floor() as isize).max(0) as usize;
    let x1 = ((cx + 40.0 * u).ceil() as isize)
        .min(img.width as isize - 1)
        .max(0) as usize;
    let y0 = ((cy + 10.0 * u).floor() as isize).max(0) as usize;
    let y1 = ((cy + 410.0 * u).ceil() as isize)
        .min(img.height as isize - 1)
        .max(0) as usize;
    if x1 < x0 || y1 < y0 {
        return None;
    }
    let sw = x1 - x0 + 2;
    let sh = y1 - y0 + 2;
    if slot.sat.len() < sw * sh {
        slot.sat.resize(sw * sh, 0);
    }
    let sum = &mut slot.sat[..sw * sh];
    sum[..sw].fill(0);
    for y in y0..=y1 {
        let row_off = (y - y0 + 1) * sw;
        let prev_off = (y - y0) * sw;
        sum[row_off] = 0;
        let mut row = 0u32;
        for x in x0..=x1 {
            let (r, g, b) = img.rgb_at(x as isize, y as isize);
            if r.max(g).max(b) > 51 {
                row += 1;
            }
            sum[row_off + (x - x0 + 1)] = sum[prev_off + (x - x0 + 1)] + row;
        }
    }
    let light_share = |x: f64, y: f64| -> f64 {
        let a = ((x - 18.0 * u).round() as isize).max(x0 as isize) as usize;
        let b = ((x + 18.0 * u).round() as isize).min(x1 as isize).max(0) as usize;
        let c = ((y - 18.0 * u).round() as isize).max(y0 as isize) as usize;
        let d = ((y + 18.0 * u).round() as isize).min(y1 as isize).max(0) as usize;
        if b < a || d < c {
            return 0.0;
        }
        let at2 = |px: usize, py: usize| sum[(py - y0) * sw + (px - x0)] as f64;
        (at2(b + 1, d + 1) - at2(a, d + 1) - at2(b + 1, c) + at2(a, c))
            / (((b - a + 1) * (d - c + 1)) as f64)
    };
    let grid_total = |ox: f64, oy: f64| -> f64 {
        let mut total = 0.0_f64;
        for r in 0..5 {
            for dx in [-66.0, -6.0] {
                let x = cx + (dx + ox) * u;
                let y = cy + (54.0 + 68.5 * (r as f64) + oy) * u;
                total += light_share(x, y);
            }
        }
        total
    };
    let mut best_ox = 0.0_f64;
    let mut best_oy = 0.0_f64;
    let mut best_total = grid_total(0.0, 0.0);
    for oy_i in (-20..=20).step_by(2) {
        let oy = oy_i as f64;
        for ox_i in (-20..=20).step_by(2) {
            let ox = ox_i as f64;
            let total = grid_total(ox, oy);
            if total > best_total {
                best_total = total;
                best_ox = ox;
                best_oy = oy;
            }
        }
    }
    let mut count = 0usize;
    'outer: for r in 0..5 {
        for dx in [-66.0, -6.0] {
            let x = cx + (dx + best_ox) * u;
            let y = cy + (54.0 + 68.5 * (r as f64) + best_oy) * u;
            if light_share(x, y) > 0.35 {
                count += 1;
            } else {
                break 'outer;
            }
        }
    }
    Some(count)
}

pub fn read_hud_objective_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    slot: &mut ScanScratchSlot,
) -> Option<ObjectiveId> {
    let patterns: &[(ObjectiveId, &[isize], bool)] = &[
        // Full single-line titles (with or without "Objective:")
        (ObjectiveId::Consume, &[8, 2, 3, 7], false),
        (ObjectiveId::Consume, &[8, 5, 7], false),
        (ObjectiveId::Doors, &[6, 4, 5], false),
        (ObjectiveId::Incident, &[13], true),
        (ObjectiveId::King, &[4, 2, 3, 5], false),
        (ObjectiveId::King, &[4, 5, 5], false),
        (ObjectiveId::Walk, &[6, 4], true),
        // Line-wrapped HUD titles ("Objective: Survival of the", "Objective: King of the")
        (ObjectiveId::Consume, &[8, 2, 3], true),
        (ObjectiveId::King, &[4, 2, 3], true),
        // Distinct HUD requirement lines below title
        (ObjectiveId::King, &[2, 5, 1, 7, 6], false),
        (ObjectiveId::King, &[7, 8], false),
        (ObjectiveId::Consume, &[7, 1, 5, 3], false),
        (ObjectiveId::Incident, &[7, 6, 8, 1, 5, 3], false),
        (ObjectiveId::Incident, &[7, 6, 8], false),
    ];
    let x1 =
        (loc.at(0.0, 0.0).0.min(loc.at(0.0, 3.0).0) - 40.0 * loc.s).min((img.width as f64) * 0.30);
    let mut y0 = (loc.ty + 15.0 * loc.s).max(0.0);
    let y1 = (loc.ty + 185.0 * loc.s).min((img.height as f64) * 0.30);
    let probe_x0 = (20.0 * loc.s).max(0.0);
    let probe_x1 = x1.min(160.0 * loc.s);
    while y0 + 8.0 < y1 && is_overlay_occluded(img, probe_x0, y0, probe_x1, y0 + 8.0) {
        y0 += 6.0;
    }
    if x1 < 40.0 || y1 <= y0 + 14.0 {
        return None;
    }
    let radii = text_radii(img);
    let mut best_match: Option<(ObjectiveId, isize)> = None;
    for lines in text_lines_radii_with_slot(
        img,
        [0.0, y0, x1, y1],
        (5.0, 36.0_f64.max((img.height as f64) * 0.05)),
        &radii[..3],
        0.32,
        slot,
    ) {
        for line in lines {
            for p in &line.pieces {
                let words: Vec<isize> = p.words.iter().map(|w| w.glyphs.len() as isize).collect();
                if words.is_empty() {
                    continue;
                }
                let has_obj_prefix = words[0] >= 8 && words[0] <= 11 && words.len() > 1;
                let name = if has_obj_prefix {
                    &words[1..]
                } else {
                    &words[..]
                };
                for &(id, want, need_prefix) in patterns {
                    if need_prefix && !has_obj_prefix {
                        continue;
                    }
                    if want.len() != name.len() {
                        continue;
                    }
                    if want
                        .iter()
                        .zip(name.iter())
                        .any(|(&a, &b)| (a - b).abs() > 1)
                    {
                        continue;
                    }
                    let off: isize = want
                        .iter()
                        .zip(name.iter())
                        .map(|(a, b)| (a - b).abs())
                        .sum();
                    if off <= 1 && best_match.is_none_or(|(_, boff)| off < boff) {
                        best_match = Some((id, off));
                    }
                }
                // Disambiguate HUD requirement lines ("Move 18 tiles ...", "Open 10 cells ...",
                // "Consume 4 times ...", "Trigger Prison Incident ...") when title is covered
                if !has_obj_prefix && words.len() >= 3 {
                    if (3..=5).contains(&words[0])
                        && (1..=3).contains(&words[1])
                        && (4..=6).contains(&words[2])
                    {
                        let nums = read_number(img, &p.words[1].glyphs, line.h, &[10, 18]);
                        if let Some(&(n, score)) = nums.first()
                            && score >= 0.30
                        {
                            let id = if n == 18 {
                                ObjectiveId::Walk
                            } else {
                                ObjectiveId::Doors
                            };
                            if best_match.is_none_or(|(_, boff)| boff > 0) {
                                best_match = Some((id, 0));
                            }
                        }
                    } else if (6..=8).contains(&words[0])
                        && (1..=2).contains(&words[1])
                        && (4..=6).contains(&words[2])
                    {
                        if best_match.is_none() {
                            best_match = Some((ObjectiveId::Consume, 1));
                        }
                    } else if (6..=8).contains(&words[0])
                        && (5..=7).contains(&words[1])
                        && (7..=9).contains(&words[2])
                        && best_match.is_none()
                    {
                        best_match = Some((ObjectiveId::Incident, 1));
                    }
                }
            }
        }
    }
    best_match.map(|(id, _)| id)
}

#[derive(Clone, Copy, Debug)]
pub struct DecisionRead {
    pub ratway: bool,
    pub office: bool,
    pub replace: bool,
    pub consume: bool,
    pub hunt: bool,
    pub end: bool,
    pub title: bool,
}

pub(crate) fn read_decision_from_lines(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    lines_by_r: &[Vec<Line>],
) -> Option<DecisionRead> {
    let bottom = loc.at(0.0, 3.0).1.max(loc.at(5.0, 3.0).1) + 30.0 * loc.s;
    if bottom >= (img.height as f64) - 20.0 {
        return None;
    }
    let x0 = (loc.at(0.0, 0.0).0.min(loc.at(0.0, 3.0).0) - 40.0 * loc.s).max(0.0);
    let x1 =
        (loc.at(5.0, 0.0).0.max(loc.at(5.0, 3.0).0) + 40.0 * loc.s).min((img.width - 1) as f64);

    let mut title = false;
    let mut pick_cell = false;
    let mut replace = false;
    let mut hunt = false;
    let mut end = false;
    let mut labels: Vec<(i32, i32, f64, i32)> = Vec::new();
    let mut hunts: Vec<(i32, i32, f64, i32)> = Vec::new();
    let mut single: Vec<(isize, f64, i32, i32, f64, i32)> = Vec::new();
    let mut ends: Vec<(i32, f64, i32)> = Vec::new();
    let mut lefts: Vec<(i32, f64)> = Vec::new();

    let near = |w: &[isize], want: &[isize]| -> bool {
        if w.len() < want.len() {
            return false;
        }
        let tail = &w[w.len() - want.len()..];
        tail.iter()
            .zip(want.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<isize>()
            <= 2
    };

    for lines in lines_by_r {
        for line in lines {
            for p in &line.pieces {
                let n: isize = p.words.iter().map(|w| w.glyphs.len() as isize).sum();
                let span = ((p.x1 - p.x0) as f64) / (line.h.max(1) as f64);
                if p.words.len() <= 2 && (5..=11).contains(&n) {
                    single.push((n, span, p.x0, p.x1, line.mid, line.h));
                }
                if (14..=20).contains(&n) && span > 7.0 && span < 15.0 {
                    let words: Vec<isize> =
                        p.words.iter().map(|w| w.glyphs.len() as isize).collect();
                    if near(&words, &[6, 2, 9]) && !near(&words, &[4, 4, 8]) {
                        pick_cell = true;
                    } else if near(&words, &[4, 4, 8]) {
                        title = true;
                    }
                } else if p.words.len() <= 2 && (6..=8).contains(&n) && span > 2.8 && span < 6.8 {
                    labels.push((p.x0, p.x1, line.mid, line.h));
                } else if p.words.len() == 1 && (4..=5).contains(&n) && span > 1.8 && span < 4.2 {
                    hunts.push((p.x0, p.x1, line.mid, line.h));
                } else if n == 3 && span > 1.5 && span < 3.2 {
                    end = true;
                    if p.words.len() == 1 {
                        ends.push((p.x0, line.mid, line.h));
                    }
                }
                if (6..=13).contains(&n) && span > 3.0 && span < 8.0 {
                    lefts.push((p.x1, line.mid));
                }
            }
        }
    }

    let red_ratio = |rx0: usize, ry0: usize, rx1: usize, ry1: usize| -> f32 {
        let mut n = 0usize;
        let mut hit = 0usize;
        let mut y = ry0;
        while y < ry1 {
            let mut x = rx0;
            while x < rx1 {
                let (r, g, b) = img.rgb_at(x as isize, y as isize);
                n += 1;
                if r >= 76 && r > g && r > b {
                    let (h, s, v) = hsv(r as f32, g as f32, b as f32);
                    if (h < 12.0 || h > 348.0) && s > 0.55 && v > 0.30 {
                        hit += 1;
                    }
                }
                x += 2;
            }
            y += 2;
        }
        if n > 0 {
            (hit as f32) / (n as f32)
        } else {
            0.0
        }
    };

    let strip = ((100.0 * loc.s).round() as usize).max(8);
    let mut red = 0.0_f32;
    let mut sx = x0.round() as usize;
    let sx_end = x1 as usize;
    let by_start = bottom.round() as usize;
    let by_end = img.height.min(by_start + (260.0 * loc.s) as usize);
    while sx + strip <= sx_end {
        red = red.max(red_ratio(sx, by_start, sx + strip, by_end));
        sx += strip;
    }
    let consume = red > 0.01;

    let red_by = |lx0: i32, lx1: i32, lmid: f64, lh: i32| -> f32 {
        let w = lx1 - lx0;
        let y0 = ((lmid - 5.0 * (lh as f64)).round() as isize).max(0) as usize;
        let y1 = ((lmid + (lh as f64)) as usize).min(img.height);
        let rx0 = (lx0 - w / 2).max(0) as usize;
        let rx1 = ((lx1 + w / 2) as usize).min(img.width);
        red_ratio(rx0, y0, rx1, y1)
    };

    for &(lx0, lx1, lmid, lh) in &labels {
        if red_by(lx0, lx1, lmid, lh) < 0.01 {
            replace = true;
        }
    }
    for &(hx0, hx1, hmid, hh) in &hunts {
        let same_row = ends
            .iter()
            .any(|&(ex0, emid, eh)| hx1 < ex0 && (hmid - emid).abs() < (eh as f64) * 0.6)
            || labels.iter().any(|&(lx0, _, lmid, lh)| {
                (hx0 - lx0).abs() > 2 * lh && (hmid - lmid).abs() < (lh as f64) * 0.6
            });
        if same_row && red_by(hx0, hx1, hmid, hh) < 0.01 {
            hunt = true;
        }
    }

    let office = single.iter().any(|&(an, aspan, ax0, ax1, amid, ah)| {
        (7..=11).contains(&an)
            && aspan > 3.8
            && aspan < 7.0
            && red_by(ax0, ax1, amid, ah) < 0.01
            && single.iter().any(|&(bn, bspan, bx0, bx1, bmid, bh)| {
                (5..=7).contains(&bn)
                    && bspan > 3.0
                    && bspan < 5.8
                    && bx0 > ax1
                    && (bmid - amid).abs() < (ah as f64) * 0.6
                    && red_by(bx0, bx1, bmid, bh) < 0.01
            })
    });

    if pick_cell && !title {
        return Some(DecisionRead {
            ratway: true,
            office: false,
            replace: false,
            consume: false,
            hunt: false,
            end: false,
            title: false,
        });
    }
    let row = ends.iter().any(|&(ex0, emid, eh)| {
        lefts.iter().any(|&(lx1, lmid)| {
            (lmid - emid).abs() < (eh as f64) * 0.5 && lx1 < ex0 && (ex0 - lx1) < 30 * eh
        })
    });
    if !title && !(row && (consume || replace)) {
        return None;
    }
    if office {
        return Some(DecisionRead {
            ratway: false,
            office: true,
            replace: false,
            consume: false,
            hunt: false,
            end: false,
            title,
        });
    }
    if !replace && !consume && !end {
        return None;
    }
    Some(DecisionRead {
        ratway: false,
        office: false,
        replace,
        consume,
        hunt,
        end,
        title,
    })
}

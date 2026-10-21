//! Integral-image multi-radius text line extraction, glyph vectorization, digit template matching, and prefix matching.

use super::arena::{Blob, ScanScratchSlot, blobs_from_mask_hits};
use super::data::{DIGITS, GLYPH_H, GLYPH_W, PREFIX_GLYPHS, PREFIX_H, PREFIX_W};
use super::image::ImageView;
use super::math::{dot_f32, normalize_zero_mean_in_place};

#[derive(Clone, Debug)]
pub struct Word {
    pub glyphs: Vec<Blob>,
    pub x0: i32,
    pub x1: i32,
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub words: Vec<Word>,
    pub glyphs: Vec<Blob>,
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub mid: f64,
    pub h: i32,
    pub glyphs: Vec<Blob>,
    pub pieces: Vec<Piece>,
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
}

fn group_blobs_into_lines(mut found: Vec<Blob>, min_h: f64, max_h: f64) -> Vec<Line> {
    found.retain(|b| {
        (b.h as f64) >= min_h * 0.45
            && (b.h as f64) <= max_h
            && (b.w as f64) <= max_h * 1.2
            && b.n >= 4
    });

    found.sort_by(|a, b| {
        let ma = (a.y0 + a.y1) as f64 * 0.5;
        let mb = (b.y0 + b.y1) as f64 * 0.5;
        ma.partial_cmp(&mb).unwrap()
    });

    let mut lines: Vec<Line> = Vec::new();
    for g in found {
        let mid = (g.y0 + g.y1) as f64 * 0.5;
        if let Some(line) = lines
            .iter_mut()
            .find(|l| (l.mid - mid).abs() < 4.0_f64.max((l.h as f64) * 0.55))
        {
            line.glyphs.push(g);
            line.h = line.h.max(g.h);
        } else {
            lines.push(Line {
                mid,
                h: g.h,
                glyphs: vec![g],
                pieces: Vec::new(),
                x0: g.x0,
                x1: g.x1,
                y0: g.y0,
                y1: g.y1,
            });
        }
    }

    let mut out = Vec::new();
    for mut line in lines {
        line.glyphs.sort_by_key(|g| g.x0);
        let mut tall: Vec<i32> = line.glyphs.iter().map(|g| g.h).collect();
        tall.sort_unstable();
        if let Some(&h75) = tall.get((tall.len() * 3) / 4) {
            line.h = h75;
        }
        let lh = line.h as f64;
        line.glyphs
            .retain(|g| !((g.h as f64) < 0.32 * lh && (g.n as f64) < 0.06 * lh * lh));
        if line.glyphs.is_empty() {
            continue;
        }

        let mut merged: Vec<Blob> = Vec::new();
        for g in line.glyphs {
            if let Some(prev) = merged.last_mut() {
                let overlap = prev.x1.min(g.x1) - prev.x0.max(g.x0) + 1;
                if (overlap as f64) > 0.5 * (prev.w.min(g.w) as f64) {
                    prev.x0 = prev.x0.min(g.x0);
                    prev.x1 = prev.x1.max(g.x1);
                    prev.y0 = prev.y0.min(g.y0);
                    prev.y1 = prev.y1.max(g.y1);
                    prev.w = prev.x1 - prev.x0 + 1;
                    prev.h = prev.y1 - prev.y0 + 1;
                    prev.n += g.n;
                    continue;
                }
            }
            merged.push(g);
        }
        line.glyphs = merged;

        let mut pieces: Vec<Piece> = Vec::new();
        let mut last_x1: Option<i32> = None;
        for g in &line.glyphs {
            let gap = last_x1.map_or(f64::INFINITY, |lx| (g.x0 - lx) as f64);
            if pieces.is_empty() || gap > 2.0 * lh {
                pieces.push(Piece {
                    words: Vec::new(),
                    glyphs: Vec::new(),
                    x0: g.x0,
                    x1: g.x1,
                    y0: g.y0,
                    y1: g.y1,
                });
            }
            let piece = pieces.last_mut().unwrap();
            if piece.words.is_empty() || gap > lh * 0.36 {
                piece.words.push(Word {
                    glyphs: Vec::new(),
                    x0: g.x0,
                    x1: g.x1,
                });
            }
            let word = piece.words.last_mut().unwrap();
            word.glyphs.push(*g);
            word.x1 = word.x1.max(g.x1);
            piece.glyphs.push(*g);
            piece.x1 = piece.x1.max(g.x1);
            piece.y0 = piece.y0.min(g.y0);
            piece.y1 = piece.y1.max(g.y1);
            last_x1 = Some(g.x1);
        }
        line.x0 = line.glyphs[0].x0;
        line.x1 = line.glyphs.iter().map(|g| g.x1).max().unwrap();
        line.y0 = line.glyphs.iter().map(|g| g.y0).min().unwrap();
        line.y1 = line.glyphs.iter().map(|g| g.y1).max().unwrap();
        line.pieces = pieces;
        out.push(line);
    }
    out
}

pub fn text_lines_radii_with_slot(
    img: &ImageView<'_>,
    [x0, y0, x1, y1]: [f64; 4],
    (min_h, max_h): (f64, f64),
    radii: &[usize],
    sat_limit: f32,
    slot: &mut ScanScratchSlot,
) -> Vec<Vec<Line>> {
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1.ceil() as isize).min(img.width as isize - 1);
    let by1 = (y1.ceil() as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return vec![Vec::new(); radii.len()];
    }
    let bx1 = bx1 as usize;
    let by1 = by1 as usize;
    let w = bx1 - bx0 + 1;
    let h = by1 - by0 + 1;
    if w < 4 || h < 4 || radii.is_empty() {
        return vec![Vec::new(); radii.len()];
    }

    let ScanScratchSlot {
        sat,
        elig_coords,
        hit_coords,
        mask,
        stack,
    } = slot;

    let r_max = radii.iter().copied().max().unwrap_or(1).max(1);
    let sw = w + 2 * r_max;
    let sh = h + 2 * r_max;
    let sat_stride = sw + 1;
    let sat_len = (sh + 1) * sat_stride;
    if sat.len() < sat_len {
        sat.resize(sat_len, 0);
    }
    sat[..sat_stride].fill(0);

    elig_coords.clear();
    let sat_mul = (sat_limit * 256.0).round() as u32;

    let fill_border_row = |sat: &mut [u32], sy: usize, py: usize| {
        let row_base = (py * img.width + bx0) * 4;
        let data = &img.data[row_base..row_base + w * 4];
        let prev_row = sy * sat_stride;
        let cur_row = (sy + 1) * sat_stride;
        sat[cur_row] = 0;
        let mut row_sum = 0u32;
        let l_q0 = (77 * (data[0] as u32) + 150 * (data[1] as u32) + 29 * (data[2] as u32)) >> 6;
        for sx in 0..r_max {
            row_sum += l_q0;
            sat[cur_row + sx + 1] = sat[prev_row + sx + 1] + row_sum;
        }
        for x in 0..w {
            let o = x * 4;
            let l_q =
                (77 * (data[o] as u32) + 150 * (data[o + 1] as u32) + 29 * (data[o + 2] as u32))
                    >> 6;
            row_sum += l_q;
            sat[cur_row + r_max + x + 1] = sat[prev_row + r_max + x + 1] + row_sum;
        }
        let o_last = (w - 1) * 4;
        let l_qn = (77 * (data[o_last] as u32)
            + 150 * (data[o_last + 1] as u32)
            + 29 * (data[o_last + 2] as u32))
            >> 6;
        for sx in (r_max + w)..sw {
            row_sum += l_qn;
            sat[cur_row + sx + 1] = sat[prev_row + sx + 1] + row_sum;
        }
    };

    for sy in 0..r_max {
        fill_border_row(sat, sy, by0);
    }

    for y in 0..h {
        let sy = y + r_max;
        let py = by0 + y;
        let row_base = (py * img.width + bx0) * 4;
        let data = &img.data[row_base..row_base + w * 4];
        let prev_row = sy * sat_stride;
        let cur_row = (sy + 1) * sat_stride;
        sat[cur_row] = 0;
        let mut row_sum = 0u32;
        let l_q0 = (77 * (data[0] as u32) + 150 * (data[1] as u32) + 29 * (data[2] as u32)) >> 6;
        for sx in 0..r_max {
            row_sum += l_q0;
            sat[cur_row + sx + 1] = sat[prev_row + sx + 1] + row_sum;
        }
        for x in 0..w {
            let o = x * 4;
            let rr = data[o];
            let gg = data[o + 1];
            let bb = data[o + 2];
            let l_256 = 77 * (rr as u32) + 150 * (gg as u32) + 29 * (bb as u32);
            row_sum += l_256 >> 6;
            sat[cur_row + r_max + x + 1] = sat[prev_row + r_max + x + 1] + row_sum;
            let mx = rr.max(gg).max(bb);
            if mx >= 45 && l_256 >= 45 * 256 {
                let mn = rr.min(gg).min(bb);
                if ((mx - mn) as u32) * 256 < sat_mul * (mx as u32) {
                    elig_coords.push((x as u16, y as u16, l_256 as u16));
                }
            }
        }
        let o_last = (w - 1) * 4;
        let l_qn = (77 * (data[o_last] as u32)
            + 150 * (data[o_last + 1] as u32)
            + 29 * (data[o_last + 2] as u32))
            >> 6;
        for sx in (r_max + w)..sw {
            row_sum += l_qn;
            sat[cur_row + sx + 1] = sat[prev_row + sx + 1] + row_sum;
        }
    }

    if elig_coords.is_empty() {
        return vec![Vec::new(); radii.len()];
    }

    for sy in (r_max + h)..sh {
        fill_border_row(sat, sy, by1);
    }

    let n = w * h;
    if mask.len() < n {
        mask.resize(n, 0);
    }

    let mut results = Vec::with_capacity(radii.len());
    for &rad in radii {
        let r = rad.max(1);
        let span = 2 * r + 1;
        let inv_area_scaled = 0.25_f32 / ((span * span) as f32);
        hit_coords.clear();
        for &(ex_u16, ey_u16, l_256) in elig_coords.iter() {
            let ex = ex_u16 as usize;
            let ey = ey_u16 as usize;
            let cx = ex + r_max;
            let cy = ey + r_max;
            let y0s = (cy - r) * sat_stride;
            let y1s = (cy + r + 1) * sat_stride;
            let x0s = cx - r;
            let x1s = cx + r + 1;
            let box_sum =
                (sat[y1s + x1s] + sat[y0s + x0s]).saturating_sub(sat[y0s + x1s] + sat[y1s + x0s]);
            let around_val = (box_sum as f32) * inv_area_scaled;
            let l = (l_256 as f32) * (1.0 / 256.0);
            if l - around_val >= 16.0_f32.max(0.3 * around_val) {
                mask[ey * w + ex] = 1;
                hit_coords.push((ex_u16, ey_u16));
            }
        }
        let found = if hit_coords.is_empty() {
            Vec::new()
        } else {
            blobs_from_mask_hits(&mut mask[..n], hit_coords, stack, w, h, bx0, by0)
        };
        results.push(group_blobs_into_lines(found, min_h, max_h));
    }
    results
}

pub fn glyph_vector(img: &ImageView<'_>, g: &Blob, line_h: i32) -> [f32; 140] {
    let mut v = [0.0_f32; 140];
    let scale = (g.h as f64).max((line_h as f64) * 0.7) / (GLYPH_H as f64);
    let cx = (g.x0 + g.x1) as f64 * 0.5;
    let cy = (g.y0 + g.y1) as f64 * 0.5;
    let sub = (scale.round() as usize).clamp(1, 4);
    let mut mean = 0.0_f32;
    for j in 0..GLYPH_H {
        for i in 0..GLYPH_W {
            let mut val = 0.0_f32;
            for b in 0..sub {
                for a in 0..sub {
                    let sx = cx
                        + ((i as f64) + ((a as f64) + 0.5) / (sub as f64) - (GLYPH_W as f64) * 0.5)
                            * scale;
                    let sy = cy
                        + ((j as f64) + ((b as f64) + 0.5) / (sub as f64) - (GLYPH_H as f64) * 0.5)
                            * scale;
                    val += img.sample_sharp(sx, sy);
                }
            }
            val /= (sub * sub) as f32;
            v[j * GLYPH_W + i] = val;
            mean += val;
        }
    }
    mean /= 140.0;
    normalize_zero_mean_in_place(&mut v, mean);
    v
}

pub fn digit_scores(img: &ImageView<'_>, g: &Blob, line_h: i32) -> [f32; 10] {
    let v = glyph_vector(img, g, line_h);
    let mut out = [-1.0_f32; 10];
    for (d, refs) in DIGITS.iter().enumerate() {
        for r in *refs {
            let dot = dot_f32(r, &v);
            if dot > out[d] {
                out[d] = dot;
            }
        }
    }
    out
}

pub(crate) const ALLOWED_CELLS: [u32; 15] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
pub(crate) const ALLOWED_DAYS: [u32; 41] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
];

#[inline]
const fn u32_digits(v: u32) -> ([usize; 2], usize) {
    if v >= 10 {
        ([(v / 10) as usize, (v % 10) as usize], 2)
    } else {
        ([v as usize, 0], 1)
    }
}

pub fn read_number(
    img: &ImageView<'_>,
    glyphs: &[Blob],
    line_h: i32,
    allowed: &[u32],
) -> Vec<(u32, f32)> {
    let n_glyphs = glyphs.len();
    if n_glyphs == 0 || n_glyphs > 2 {
        return Vec::new();
    }
    let mut per = [[-1.0_f32; 10]; 2];
    for (i, g) in glyphs.iter().enumerate() {
        per[i] = digit_scores(img, g, line_h);
    }
    let mut out = Vec::with_capacity(allowed.len());
    for &value in allowed {
        let (digits, dlen) = u32_digits(value);
        if dlen != n_glyphs {
            continue;
        }
        let mut sc = 0.0_f32;
        for i in 0..dlen {
            sc += per[i][digits[i]];
        }
        out.push((value, sc / (dlen as f32)));
    }
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    out
}

pub fn prefix_vector(
    img: &ImageView<'_>,
    piece: &Piece,
    end: f64,
    dy0: i32,
    dy1: i32,
) -> [f32; 400] {
    let g0 = &piece.glyphs[0];
    let x0 = g0.x0 as f64;
    let x1 = x0 + end * (g0.h as f64);
    let y0 = (g0.y0 + dy0) as f64;
    let y1 = (g0.y1 + dy1) as f64;
    let mut v = [0.0_f32; 400];
    let mut mean = 0.0_f32;
    for j in 0..PREFIX_H {
        for i in 0..PREFIX_W {
            let sx = x0 + ((x1 - x0) * ((i as f64) + 0.5)) / (PREFIX_W as f64);
            let sy = y0 + ((y1 - y0) * ((j as f64) + 0.5)) / (PREFIX_H as f64);
            let val = img.sample_sharp(sx, sy);
            v[j * PREFIX_W + i] = val;
            mean += val;
        }
    }
    mean /= 400.0;
    normalize_zero_mean_in_place(&mut v, mean);
    v
}

pub fn prefix_like(img: &ImageView<'_>, piece: &Piece) -> f32 {
    let pv0 = prefix_vector(img, piece, 5.46, 0, 0);
    let mut best = -1.0_f32;
    for r in &PREFIX_GLYPHS {
        let dot = dot_f32(r, &pv0);
        if dot > best {
            best = dot;
        }
    }
    if best < 0.22 || best >= 0.56 {
        return best;
    }
    const CENTER_PREFIX: [usize; 2] = [13, 40];
    for end in [5.3, 5.46, 5.62] {
        for dy0 in [-1, 0, 1] {
            for dy1 in [-1, 0, 1] {
                if end == 5.46 && dy0 == 0 && dy1 == 0 {
                    continue;
                }
                let pv = prefix_vector(img, piece, end, dy0, dy1);
                let mut shift_best = -1.0_f32;
                for &idx in &CENTER_PREFIX {
                    let dot = dot_f32(&PREFIX_GLYPHS[idx], &pv);
                    if dot > shift_best {
                        shift_best = dot;
                    }
                }
                if shift_best > best {
                    best = shift_best;
                }
                if shift_best >= 0.44 {
                    for r in &PREFIX_GLYPHS {
                        let dot = dot_f32(r, &pv);
                        if dot > best {
                            best = dot;
                        }
                    }
                }
            }
        }
    }
    best
}

#[inline]
pub fn text_radii(img: &ImageView<'_>) -> [usize; 4] {
    [0.006, 0.01, 0.016, 0.026].map(|f| ((img.height as f64 * f).round() as usize).max(3))
}

#[inline]
pub fn text_radii_for_scale(s: f64) -> [usize; 4] {
    let vh = 720.0 * s.max(0.5);
    [0.006, 0.01, 0.016, 0.026].map(|f| ((vh * f).round() as usize).max(3))
}

pub fn split_glyph(img: &ImageView<'_>, g: &Blob) -> Option<[Blob; 2]> {
    let mut cut = -1i32;
    let mut least = f32::INFINITY;
    let start_x = ((g.x0 as f64) + (g.w as f64) * 0.3).round() as i32;
    let end_x = ((g.x1 as f64) - (g.w as f64) * 0.3).floor() as i32;
    for x in start_x..=end_x {
        let mut ink = 0.0_f32;
        for y in g.y0..=g.y1 {
            ink += img.sharp_px(x as isize, y as isize);
        }
        if ink < least {
            least = ink;
            cut = x;
        }
    }
    if cut < 0 {
        return None;
    }
    let mut left = *g;
    left.x1 = cut - 1;
    left.w = cut - g.x0;
    let mut right = *g;
    right.x0 = cut + 1;
    right.w = g.x1 - cut;
    Some([left, right])
}

#[derive(Clone, Copy, Debug)]
pub struct LabelBox {
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
    pub h: i32,
}

pub fn find_label_with_slot(
    img: &ImageView<'_>,
    bx: [f64; 4],
    letters: isize,
    shape: f64,
    slot: &mut ScanScratchSlot,
) -> Option<LabelBox> {
    let mut best: Option<(f64, LabelBox)> = None;
    let radii = text_radii(img);
    let max_h = 40.0_f64.max((img.height as f64) * 0.06);
    let ordered_radii = [radii[1], radii[2], radii[0]];
    for lines in text_lines_radii_with_slot(img, bx, (6.0, max_h), &ordered_radii, 0.50, slot) {
        for line in lines {
            for p in &line.pieces {
                let n: isize = p.words.iter().map(|w| w.glyphs.len() as isize).sum();
                let ratio = ((p.x1 - p.x0) as f64) / (line.h.max(1) as f64);
                if (n - letters).abs() > 6 || (ratio - shape).abs() > 2.5 {
                    continue;
                }
                let off = ((n - letters).abs() as f64) / 3.0 + (ratio - shape).abs();
                if best.as_ref().is_none_or(|b| off < b.0) {
                    best = Some((
                        off,
                        LabelBox {
                            x0: p.x0,
                            x1: p.x1,
                            y0: p.y0,
                            y1: p.y1,
                            h: line.h,
                        },
                    ));
                }
            }
        }
        if best.as_ref().is_some_and(|(off, _)| *off <= 1.5) {
            break;
        }
    }
    best.map(|b| b.1)
}

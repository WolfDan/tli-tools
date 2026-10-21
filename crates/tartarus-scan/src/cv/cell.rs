//! Cell state classification, face template matching, frame rarity, warden arrow, and event exclamation detection.

use tartarus_core::Beast;

use super::arena::{
    Blob, CellScratchSlot, ScanScratchSlot, blobs_from_mask_hits, blobs_rgb_strided_with_slot,
    blobs_with_slot,
};
use super::data::{
    ARROW, FACE_GX, FACE_GY, FACE_X0, FACE_X1, FACE_Y0, FACE_Y1, FACES_LION, FACES_MAMMOTH,
    FACES_RAT, FACES_WOLF, RING,
};
use super::image::{
    ImageView, color_share, color_share_rgb, is_gold, is_gold_rgb, is_jaws_rgb, is_nest_rgb,
    is_orange_rgb, sample,
};
use super::math::{BoardLoc, dot_f32, normalize_zero_mean_in_place};

pub fn padlock_at(img: &ImageView<'_>, x: f64, y: f64, k: f64) -> f32 {
    let x0 = x - 13.0 * k;
    let y0 = y - 11.0 * k;
    let x1 = x + 13.0 * k;
    let y1 = y + 11.0 * k;
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1.ceil() as isize).min(img.width as isize - 1);
    let by1 = (y1.ceil() as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return 0.0;
    }
    let bx1 = bx1 as usize;
    let by1 = by1 as usize;
    let w = bx1 - bx0 + 1;
    let h = by1 - by0 + 1;
    let n_px = w * h;

    let test = |px: usize, py: usize| -> bool {
        let (r, g, b) = img.rgb_at(px as isize, py as isize);
        let r = r as i32;
        let g = g as i32;
        let b = b as i32;
        let l = (r + g + b) / 3;
        l > 112
            && r >= g
            && g >= b - 4
            && r - b >= 6
            && (r - b <= 64 || (r - b <= 96 && g >= 95 && b >= 75))
    };

    let found = if n_px <= 4096 {
        let mut mask = [0u8; 4096];
        let mut hit_coords = [(0u16, 0u16); 4096];
        let mut hit_len = 0usize;
        for yy in 0..h {
            let row_off = yy * w;
            let py = by0 + yy;
            for xx in 0..w {
                if test(bx0 + xx, py) {
                    mask[row_off + xx] = 1;
                    hit_coords[hit_len] = (xx as u16, yy as u16);
                    hit_len += 1;
                }
            }
        }
        if hit_len == 0 {
            Vec::new()
        } else {
            let mut stack = Vec::with_capacity(64);
            blobs_from_mask_hits(
                &mut mask[..n_px],
                &hit_coords[..hit_len],
                &mut stack,
                w,
                h,
                bx0,
                by0,
            )
        }
    } else {
        let mut slot = ScanScratchSlot::default();
        blobs_with_slot(img, x0, y0, x1, y1, &mut slot, test)
    };

    let mut best_fit: Option<f64> = None;
    for b in found {
        let area = (b.n as f64) / (k * k);
        let bw = (b.w as f64) / k;
        let bh = (b.h as f64) / k;
        let off = (b.cx - x).hypot(b.cy - y) / k;
        if !(35.0..=150.0).contains(&area)
            || !(7.0..=18.0).contains(&bw)
            || !(5.0..=15.0).contains(&bh)
            || off > 7.0
        {
            continue;
        }
        let fit = 1.0 - (area - 78.0).abs() / 78.0 - off / 14.0;
        if best_fit.is_none_or(|bf| fit > bf) {
            best_fit = Some(fit);
        }
    }
    best_fit.map_or(0.0, |fit| (0.55 + fit / 2.0).clamp(0.05, 1.0) as f32)
}

pub fn face_vector(gray: &[f32], w: usize, h: usize, x: f64, y: f64, k: f64) -> ([f32; 396], f32) {
    let mut v = [0.0_f32; 396];
    let mut mean = 0.0_f32;
    for j in 0..FACE_GY {
        for i in 0..FACE_GX {
            let px =
                x + k * (FACE_X0 + ((FACE_X1 - FACE_X0) * ((i as f64) + 0.5)) / (FACE_GX as f64));
            let py =
                y + k * (FACE_Y0 + ((FACE_Y1 - FACE_Y0) * ((j as f64) + 0.5)) / (FACE_GY as f64));
            let val = sample(gray, w, h, px, py);
            v[j * FACE_GX + i] = val;
            mean += val;
        }
    }
    mean /= 396.0;
    normalize_zero_mean_in_place(&mut v, mean);
    (v, mean)
}

pub struct FaceMatch {
    pub beast: Beast,
    pub score: f32,
    pub second: f32,
    pub brightness: f32,
}

pub fn face_match(gray: &[f32], w: usize, h: usize, x: f64, y: f64, k: f64) -> FaceMatch {
    const CENTER_INDICES: [usize; 6] = [4, 13, 22, 31, 40, 49];
    let mut scores = [f32::NEG_INFINITY; 4];
    let (fv0, brightness) = face_vector(gray, w, h, x, y, k);
    for (idx, refs) in [
        (0usize, &FACES_MAMMOTH[..]),
        (1usize, &FACES_LION[..]),
        (2usize, &FACES_WOLF[..]),
        (3usize, &FACES_RAT[..]),
    ] {
        for r in refs {
            let dot = dot_f32(r, &fv0);
            if dot > scores[idx] {
                scores[idx] = dot;
            }
        }
    }
    let max0 = scores[0].max(scores[1]).max(scores[2]).max(scores[3]);
    if max0 >= 0.25 {
        let mut dy = -3.0_f64;
        while dy <= 3.001 {
            let mut dx = -3.0_f64;
            while dx <= 3.001 {
                if dx.abs() > 1e-5 || dy.abs() > 1e-5 {
                    let (fv, _) = face_vector(gray, w, h, x + dx * k, y + dy * k, k);
                    for (idx, refs) in [
                        (0usize, &FACES_MAMMOTH[..]),
                        (1usize, &FACES_LION[..]),
                        (2usize, &FACES_WOLF[..]),
                        (3usize, &FACES_RAT[..]),
                    ] {
                        for &ci in &CENTER_INDICES {
                            let dot = dot_f32(&refs[ci], &fv);
                            if dot > scores[idx] {
                                scores[idx] = dot;
                            }
                        }
                    }
                }
                dx += 1.5;
            }
            dy += 1.5;
        }
    }

    let mut ranked = [
        (Beast::Mammoth, scores[0]),
        (Beast::Lion, scores[1]),
        (Beast::Wolf, scores[2]),
        (Beast::Rat, scores[3]),
    ];
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    FaceMatch {
        beast: ranked[0].0,
        score: ranked[0].1,
        second: ranked[1].1,
        brightness,
    }
}

pub struct FrameRarity {
    pub tier: u8,
    pub conf: f32,
    pub colored: f32,
}

pub fn frame_rarity(img: &ImageView<'_>, x: f64, y: f64, k: f64) -> FrameRarity {
    let tier_by_hue: [(f32, f32, usize); 5] = [
        (0.0, 14.0, 4),
        (14.0, 62.0, 3),
        (175.0, 242.0, 1),
        (242.0, 305.0, 2),
        (330.0, 360.0, 4),
    ];
    let mut bins = [0.0_f32; 5];
    let mut samples = 0usize;
    let mut bright = 0usize;
    let mut visit = |px: f64, py: f64| {
        let (hh, ss, vv) = img.hsv_at(px, py);
        samples += 1;
        if vv > 0.62 && ss < 0.25 {
            bright += 1;
        }
        if vv < 0.45 || ss < 0.32 {
            return;
        }
        let weight = ((ss - 0.28) * 1.6).clamp(0.5, 1.25);
        for &(lo, hi, t) in &tier_by_hue {
            if hh >= lo && hh < hi {
                bins[t] += weight;
            } else if (hh - lo).abs() < 3.5 || (hh - hi).abs() < 3.5 {
                bins[t] += 0.35 * weight;
            }
        }
    };
    let step_px = k.max(1.0);
    let mut py = y - 60.0 * k;
    while py <= y - 12.0 * k + 1e-5 {
        let mut px = x - 29.0 * k;
        while px <= x - 17.0 * k + 1e-5 {
            visit(px, py);
            px += step_px;
        }
        let mut px = x + 17.0 * k;
        while px <= x + 29.0 * k + 1e-5 {
            visit(px, py);
            px += step_px;
        }
        py += step_px;
    }
    let mut py = y - 14.0 * k;
    while py <= y - 4.0 * k + 1e-5 {
        let mut px = x - 9.0 * k;
        while px <= x + 9.0 * k + 1e-5 {
            visit(px, py);
            px += step_px;
        }
        py += step_px;
    }
    let cf: f32 = bins.iter().sum();
    let sf = samples.max(1) as f32;
    if cf < 0.025 * sf {
        let conf = if cf < 0.008 * sf {
            0.85
        } else if (bright as f32) > 0.04 * sf {
            0.80
        } else {
            0.55
        };
        return FrameRarity {
            tier: 0,
            conf,
            colored: cf / sf,
        };
    }
    let mut tier = 1usize;
    for t in 2..5 {
        if bins[t] > bins[tier] {
            tier = t;
        }
    }
    FrameRarity {
        tier: tier as u8,
        conf: (0.5 + bins[tier] / cf.max(1e-5) / 2.0).min(1.0),
        colored: cf / sf,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScannedCellKind {
    Office,
    Locked,
    Inmate,
    #[default]
    Empty,
}

#[derive(Clone, Copy, Debug)]
pub struct ScannedCell {
    pub n: u8,
    pub kind: ScannedCellKind,
    pub beast: Beast,
    pub tier: u8,
    pub conf: f32,
    pub x: f64,
    pub y: f64,
    pub k: f64,
    pub nest: bool,
    pub jaws: bool,
    pub ghost: bool,
    pub event: bool,
}

impl Default for ScannedCell {
    fn default() -> Self {
        Self {
            n: 0,
            kind: ScannedCellKind::Empty,
            beast: Beast::Mammoth,
            tier: 0,
            conf: 0.0,
            x: 0.0,
            y: 0.0,
            k: 0.0,
            nest: false,
            jaws: false,
            ghost: false,
            event: false,
        }
    }
}

pub fn read_cell(img: &ImageView<'_>, loc: &BoardLoc, n: u8) -> ScannedCell {
    let mut slot = CellScratchSlot::with_capacity();
    read_cell_with_slot(img, loc, n, &mut slot)
}

pub fn read_cell_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    n: u8,
    slot: &mut CellScratchSlot,
) -> ScannedCell {
    if n == 0 {
        let (ox, oy) = loc.at(0.0, 0.0);
        return ScannedCell {
            n: 0,
            kind: ScannedCellKind::Office,
            beast: Beast::Mammoth,
            tier: 0,
            conf: 1.0,
            x: ox,
            y: oy,
            k: loc.scale_at(0.0, 0.0),
            nest: false,
            jaws: false,
            ghost: false,
            event: false,
        };
    }
    let (u, v) = RING[n as usize];
    let uf = u as f64;
    let vf = v as f64;
    let (x, y) = loc.at(uf, vf);
    let k = loc.scale_at(uf, vf);
    let lock = padlock_at(img, x, y, k);
    let nest = color_share_rgb(
        img,
        x - 15.0 * k,
        y - 28.0 * k,
        x + 15.0 * k,
        y - 6.0 * k,
        1,
        is_nest_rgb,
    ) > 0.07;

    // Fast path only for high-confidence padlocks; borderline padlocks are cross-checked
    // against face template score and card frame rarity below.
    if lock > 0.68 {
        return ScannedCell {
            n,
            kind: ScannedCellKind::Locked,
            beast: Beast::Mammoth,
            tier: 0,
            conf: lock,
            x,
            y,
            k,
            nest,
            jaws: false,
            ghost: false,
            event: false,
        };
    }

    let wi = img.width as isize;
    let hi = img.height as isize;
    let bx0 = (((x + k * (-3.0 + FACE_X0)).floor() as isize) - 1).clamp(0, wi - 1) as usize;
    let bx1 = (((x + k * (3.0 + FACE_X1)).ceil() as isize) + 2).clamp(0, wi - 1) as usize;
    let by0 = (((y + k * (-3.0 + FACE_Y0)).floor() as isize) - 1).clamp(0, hi - 1) as usize;
    let by1 = (((y + k * (3.0 + FACE_Y1)).ceil() as isize) + 2).clamp(0, hi - 1) as usize;
    let pw = (bx1 - bx0 + 1).max(2);
    let ph = (by1 - by0 + 1).max(2);
    let row_buf_len = (ph + 2) * pw;
    let patch_len = pw * ph;

    let CellScratchSlot { row_buf, patch } = slot;
    if row_buf.len() < row_buf_len {
        row_buf.resize(row_buf_len, 0.0);
    }
    if patch.len() < patch_len {
        patch.resize(patch_len, 0.0);
    }
    let row_buf = &mut row_buf[..row_buf_len];
    let patch = &mut patch[..patch_len];

    let w = img.width;
    for ry in 0..(ph + 2) {
        let cy = ((by0 as isize) + (ry as isize) - 1).clamp(0, hi - 1) as usize;
        let row_base = cy * w * 4;
        let dst = &mut row_buf[ry * pw..(ry + 1) * pw];
        let x_m1 = ((bx0 as isize) - 1).max(0) as usize;
        let o_m1 = row_base + x_m1 * 4;
        let mut l0 = 0.299 * (img.data[o_m1] as f32)
            + 0.587 * (img.data[o_m1 + 1] as f32)
            + 0.114 * (img.data[o_m1 + 2] as f32);
        let o_0 = row_base + bx0 * 4;
        let mut l1 = 0.299 * (img.data[o_0] as f32)
            + 0.587 * (img.data[o_0 + 1] as f32)
            + 0.114 * (img.data[o_0 + 2] as f32);
        for (px, dst_slot) in dst.iter_mut().enumerate() {
            let cx2 = (bx0 + px + 1).min(w - 1);
            let o2 = row_base + cx2 * 4;
            let l2 = 0.299 * (img.data[o2] as f32)
                + 0.587 * (img.data[o2 + 1] as f32)
                + 0.114 * (img.data[o2 + 2] as f32);
            *dst_slot = l0 + l1 + l2;
            l0 = l1;
            l1 = l2;
        }
    }
    const INV9: f32 = 1.0 / 9.0;
    for py in 0..ph {
        let r0 = &row_buf[py * pw..(py + 1) * pw];
        let r1 = &row_buf[(py + 1) * pw..(py + 2) * pw];
        let r2 = &row_buf[(py + 2) * pw..(py + 3) * pw];
        let out = &mut patch[py * pw..(py + 1) * pw];
        for px in 0..pw {
            out[px] = (r0[px] + r1[px] + r2[px]) * INV9;
        }
    }
    let face = face_match(patch, pw, ph, x - (bx0 as f64), y - (by0 as f64), k);
    let frame = frame_rarity(img, x, y, k);

    // If a borderline padlock was found, only override it if the face template & frame clearly show an inmate
    if lock > 0.40
        && !(face.score > 0.62 && (frame.colored > 0.08 || (face.score - face.second) > 0.12))
    {
        return ScannedCell {
            n,
            kind: ScannedCellKind::Locked,
            beast: Beast::Mammoth,
            tier: 0,
            conf: lock,
            x,
            y,
            k,
            nest,
            jaws: false,
            ghost: false,
            event: false,
        };
    }

    let jaws = color_share_rgb(
        img,
        x - 12.0 * k,
        y - 70.0 * k,
        x + 12.0 * k,
        y - 50.0 * k,
        1,
        is_jaws_rgb,
    ) > 0.14
        && color_share_rgb(
            img,
            x - 10.0 * k,
            y - 55.0 * k,
            x + 10.0 * k,
            y - 48.0 * k,
            1,
            is_jaws_rgb,
        ) > 0.18;
    let ghost = face.brightness > 175.0 && face.score < 0.55;

    if face.score > 0.42 || ghost || frame.colored > 0.12 {
        let margin = face.score - face.second;
        let c1 = (face.score - 0.30) * 1.6;
        let c2 = 0.40 + margin * 3.0;
        let raw_conf = c1.min(c2).min(frame.conf).clamp(0.05, 1.0);
        let conf = raw_conf * if ghost { 0.5 } else { 1.0 };
        return ScannedCell {
            n,
            kind: ScannedCellKind::Inmate,
            beast: face.beast,
            tier: frame.tier,
            conf,
            x,
            y,
            k,
            nest,
            jaws,
            ghost,
            event: false,
        };
    }

    ScannedCell {
        n,
        kind: ScannedCellKind::Empty,
        beast: Beast::Mammoth,
        tier: 0,
        conf: 0.40,
        x,
        y,
        k,
        nest,
        jaws,
        ghost,
        event: false,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WardenRead {
    pub cell: u8,
    pub conf: f32,
    pub box_coords: [i32; 4],
}

pub fn find_warden_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    slot: &mut ScanScratchSlot,
) -> Option<WardenRead> {
    let corners =
        [(0.15, -0.15), (4.85, -0.15), (4.85, 2.3), (0.15, 2.3)].map(|(u, v)| loc.at(u, v));
    let min_x = corners.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
    let max_x = corners
        .iter()
        .map(|c| c.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = corners.iter().map(|c| c.1).fold(f64::INFINITY, f64::min);
    let max_y = corners
        .iter()
        .map(|c| c.1)
        .fold(f64::NEG_INFINITY, f64::max);

    // Central courtyard (u in 1.25..3.75, v in 0.72..1.40) is > 0.55 grid units from all ARROW points
    let (ix0, iy0) = loc.at(1.25, 0.72);
    let (ix1, iy1) = loc.at(3.75, 1.40);
    let skip_rect = if ix1 > ix0 + 8.0 && iy1 > iy0 + 8.0 {
        Some([
            ix0.ceil().max(0.0) as usize,
            iy0.ceil().max(0.0) as usize,
            ix1.floor().max(0.0) as usize,
            iy1.floor().max(0.0) as usize,
        ])
    } else {
        None
    };

    let k = loc.s;
    let step_y = ((3.0 * k) as usize).clamp(1, 4);
    let found = blobs_rgb_strided_with_slot(
        img,
        [min_x, min_y, max_x, max_y],
        skip_rect,
        step_y,
        slot,
        is_gold_rgb,
    );
    let mut groups: Vec<(f64, f64, usize, [i32; 4])> = Vec::new();
    for b in found {
        if (b.n as f64) < 6.0 * k * k {
            continue;
        }
        if let Some(g) = groups
            .iter_mut()
            .find(|q| (q.0 - b.cx).hypot(q.1 - b.cy) < 22.0 * k)
        {
            let tot = (g.2 + b.n) as f64;
            g.0 = (g.0 * (g.2 as f64) + b.cx * (b.n as f64)) / tot;
            g.1 = (g.1 * (g.2 as f64) + b.cy * (b.n as f64)) / tot;
            g.2 += b.n;
            g.3 = [
                g.3[0].min(b.x0),
                g.3[1].min(b.y0),
                g.3[2].max(b.x1),
                g.3[3].max(b.y1),
            ];
        } else {
            groups.push((b.cx, b.cy, b.n, [b.x0, b.y0, b.x1, b.y1]));
        }
    }

    let mut best: Option<(u8, f64, f64, [i32; 4])> = None;
    for (cx, cy, n, bx) in groups {
        let area = (n as f64) / (k * k);
        if !(70.0..=1600.0).contains(&area) {
            continue;
        }
        let (gu, gv) = loc.grid_of(cx, cy);
        let mut cell = 0u8;
        let mut dist = f64::INFINITY;
        for (idx, &(au, av)) in ARROW.iter().enumerate() {
            let d = (au - gu).hypot(av - gv);
            if d < dist {
                dist = d;
                cell = idx as u8;
            }
        }
        if dist > 0.55 {
            continue;
        }
        let fit = area / 500.0 - dist;
        if best.is_none_or(|b| fit > b.1) {
            best = Some((cell, fit, dist, bx));
        }
    }

    let blob_res = best.map(|(cell, _, dist, box_coords)| WardenRead {
        cell,
        conf: (1.0 - dist).clamp(0.2, 1.0) as f32,
        box_coords,
    });

    if blob_res.as_ref().is_some_and(|w| w.conf >= 0.58) {
        return blob_res;
    }

    // Fallback: direct anchor evaluation across all 16 ARROW positions
    let mut best_anchor: Option<(u8, f32, [i32; 4])> = None;
    for (idx, &(au, av)) in ARROW.iter().enumerate() {
        let (ax, ay) = loc.at(au, av);
        let rx = 18.0 * k;
        let ry = 20.0 * k;
        let share = color_share(img, ax - rx, ay - ry, ax + rx, ay + ry, is_gold);
        if share > 0.08 && best_anchor.is_none_or(|b| share > b.1) {
            best_anchor = Some((
                idx as u8,
                share,
                [
                    (ax - rx).round() as i32,
                    (ay - ry).round() as i32,
                    (ax + rx).round() as i32,
                    (ay + ry).round() as i32,
                ],
            ));
        }
    }

    if let Some((cell, share, box_coords)) = best_anchor {
        let anchor_conf = (share * 3.2).clamp(0.25, 0.85);
        if blob_res.as_ref().is_none_or(|w| anchor_conf > w.conf) {
            return Some(WardenRead {
                cell,
                conf: anchor_conf,
                box_coords,
            });
        }
    }

    blob_res
}

fn event_band_rects(loc: &BoardLoc) -> [[f64; 4]; 4] {
    let c1 = loc.at(1.0, 0.0);
    let c5 = loc.at(5.0, 0.0);
    let k_top = loc.scale_at(1.0, 0.0).max(loc.scale_at(5.0, 0.0));
    let top = [
        c1.0.min(c5.0) - 115.0 * k_top,
        c1.1.min(c5.1) - 155.0 * k_top,
        c1.0.max(c5.0) + 115.0 * k_top,
        c1.1.max(c5.1) + 80.0 * k_top,
    ];

    let c13 = loc.at(0.0, 3.0);
    let c8 = loc.at(5.0, 3.0);
    let k_bot = loc.scale_at(0.0, 3.0).max(loc.scale_at(5.0, 3.0));
    let bot = [
        c13.0.min(c8.0) - 115.0 * k_bot,
        c13.1.min(c8.1) - 155.0 * k_bot,
        c13.0.max(c8.0) + 115.0 * k_bot,
        c13.1.max(c8.1) + 80.0 * k_bot,
    ];

    let c15 = loc.at(0.0, 1.0);
    let c14 = loc.at(0.0, 2.0);
    let k_left = loc.scale_at(0.0, 1.0).max(loc.scale_at(0.0, 2.0));
    let left = [
        c14.0.min(c15.0) - 115.0 * k_left,
        c15.1.min(c14.1) - 155.0 * k_left,
        c14.0.max(c15.0) + 115.0 * k_left,
        c15.1.max(c14.1) + 80.0 * k_left,
    ];

    let c6 = loc.at(5.0, 1.0);
    let c7 = loc.at(5.0, 2.0);
    let k_right = loc.scale_at(5.0, 1.0).max(loc.scale_at(5.0, 2.0));
    let right = [
        c6.0.min(c7.0) - 115.0 * k_right,
        c6.1.min(c7.1) - 155.0 * k_right,
        c6.0.max(c7.0) + 115.0 * k_right,
        c6.1.max(c7.1) + 80.0 * k_right,
    ];

    [top, bot, left, right]
}

pub(crate) fn find_event_blobs_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    slot: &mut ScanScratchSlot,
) -> Vec<Blob> {
    let bands = event_band_rects(loc);
    let step_y = ((6.0 * loc.s) as usize).clamp(1, 8);
    let mut out = Vec::with_capacity(32);
    for rect in bands {
        out.extend(blobs_rgb_strided_with_slot(
            img,
            rect,
            None,
            step_y,
            slot,
            is_orange_rgb,
        ));
    }
    out
}

pub(crate) fn match_events_from_blobs(
    found: &[Blob],
    loc: &BoardLoc,
    cells: &[ScannedCell; 16],
    warden: Option<&WardenRead>,
) -> u16 {
    let k = loc.s;
    let mut out = 0u16;
    for b in found {
        let bh = b.h as f64;
        let bw = b.w as f64;
        if bh < 12.0 * k || bh > 36.0 * k || bh < 1.8 * bw {
            continue;
        }
        if let Some(w) = warden {
            let ab = w.box_coords;
            if b.cx > (ab[0] as f64) - 4.0 * k
                && b.cx < (ab[2] as f64) + 4.0 * k
                && b.cy > (ab[1] as f64) - 4.0 * k
                && b.cy < (ab[3] as f64) + 4.0 * k
            {
                continue;
            }
        }
        let mut nearest = 0u8;
        let mut dist = f64::INFINITY;
        for c in cells.iter().skip(1) {
            let d = (b.cx - c.x).hypot(b.cy - (c.y - 38.0 * c.k)) / c.k;
            if d < dist {
                dist = d;
                nearest = c.n;
            }
        }
        if dist > 35.0 && dist < 95.0 {
            out |= 1u16 << nearest;
        }
    }
    out
}

pub fn find_events_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    cells: &[ScannedCell; 16],
    warden: Option<&WardenRead>,
    slot: &mut ScanScratchSlot,
) -> u16 {
    let found = find_event_blobs_with_slot(img, loc, slot);
    match_events_from_blobs(&found, loc, cells, warden)
}

//! Multi-scale board template localization (`locate_board`) and cell-number
//! RANSAC (`locate_by_labels_with_slot`).

use super::arena::ScanScratchSlot;
use super::data::{REF_BOARD_POINTS, RING};
use super::image::{ImageView, half, half_gray_of, sample};
use super::math::{BoardLoc, ncc_score_slice};
use super::ocr::{ALLOWED_CELLS, read_number, text_lines_radii_with_slot};

struct PyrLevel {
    g: Vec<f32>,
    w: usize,
    h: usize,
    f: f64,
}

pub(crate) struct BoardTemplateRefs {
    pub tv: Vec<f32>,
    pub c4_pts: Vec<(i32, i32, f32)>,
    pub c4_tv: Vec<f32>,
    pub c8_pts: Vec<(i32, i32, f32)>,
    pub c8_tv: Vec<f32>,
    pub c32_pts: Vec<(i32, i32, f32)>,
    pub c32_tv: Vec<f32>,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

fn build_subsampled_template(
    pts: &[(i32, i32, f32)],
    step: usize,
) -> (Vec<(i32, i32, f32)>, Vec<f32>) {
    assert!(step >= 1);
    assert!(!pts.is_empty());
    let sub_pts: Vec<(i32, i32, f32)> = pts.iter().copied().step_by(step).collect();
    let cn = sub_pts.len();
    let mut cmean = 0.0_f32;
    for p in &sub_pts {
        cmean += p.2;
    }
    cmean /= cn as f32;
    let mut ctv = vec![0.0_f32; cn];
    let mut cnorm = 0.0_f32;
    for (j, p) in sub_pts.iter().enumerate() {
        ctv[j] = p.2 - cmean;
        cnorm += ctv[j] * ctv[j];
    }
    cnorm = cnorm.sqrt().max(1e-6);
    for v in &mut ctv {
        *v /= cnorm;
    }
    (sub_pts, ctv)
}

pub(crate) fn board_template_refs() -> &'static BoardTemplateRefs {
    static REFS: std::sync::OnceLock<BoardTemplateRefs> = std::sync::OnceLock::new();
    let refs = REFS.get_or_init(|| {
        let pts = &REF_BOARD_POINTS;
        let (pts_all, tv) = build_subsampled_template(pts, 1);
        let _ = pts_all;
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for p in pts {
            x_min = x_min.min(p.0 as f64);
            x_max = x_max.max(p.0 as f64);
            y_min = y_min.min(p.1 as f64);
            y_max = y_max.max(p.1 as f64);
        }
        let (c4_pts, c4_tv) = build_subsampled_template(pts, 4);
        let (c8_pts, c8_tv) = build_subsampled_template(pts, 8);
        let (c32_pts, c32_tv) = build_subsampled_template(pts, 32);
        BoardTemplateRefs {
            tv,
            c4_pts,
            c4_tv,
            c8_pts,
            c8_tv,
            c32_pts,
            c32_tv,
            x_min,
            x_max,
            y_min,
            y_max,
        }
    });
    assert!(!refs.tv.is_empty());
    assert!(refs.x_max > refs.x_min);
    refs
}

fn score_direct_gray(
    img: &ImageView<'_>,
    sub_pts: &[(i32, i32, f32)],
    sub_tv: &[f32],
    s: f64,
    ox: f64,
    oy: f64,
) -> f64 {
    let m = sub_pts.len();
    assert_eq!(m, sub_tv.len());
    assert!(m <= 450);
    let wi = img.width as isize;
    let hi = img.height as isize;
    let mut buf = [0.0_f32; 450];
    for i in 0..m {
        let xi = (ox + (sub_pts[i].0 as f64) * s) as isize;
        let yi = (oy + (sub_pts[i].1 as f64) * s) as isize;
        buf[i] = img.gray_px(xi.clamp(0, wi - 1), yi.clamp(0, hi - 1));
    }
    ncc_score_slice(&buf[..m], sub_tv)
}

fn score_direct_sharp_sub(
    img: &ImageView<'_>,
    sub_pts: &[(i32, i32, f32)],
    sub_tv: &[f32],
    s: f64,
    ox: f64,
    oy: f64,
) -> f64 {
    let m = sub_pts.len();
    assert_eq!(m, sub_tv.len());
    assert!(m <= 1800);
    let mut buf = [0.0_f32; 1800];
    for i in 0..m {
        buf[i] = img.sample_sharp(
            ox + (sub_pts[i].0 as f64) * s,
            oy + (sub_pts[i].1 as f64) * s,
        );
    }
    ncc_score_slice(&buf[..m], sub_tv)
}

fn make_loc(s: f64, tx: f64, ty: f64, score: f64, w: usize, h: usize) -> BoardLoc {
    assert!(s > 0.0);
    assert!(w > 0 && h > 0);
    BoardLoc {
        s,
        tx,
        ty,
        score,
        w,
        h,
        by_labels: false,
        cells_matched: 0,
    }
}

// Zero-allocation refinement around a candidate (s, tx, ty):
// 1. 225-point gate via img.gray_px (~0.4us)
// 2. Fast exact-center check when stationary (~4us)
// 3. 3-scale 1.0px integer grid search with 450 points + 0.5px peak refinement on winning scale
fn refine_near_direct(
    img: &ImageView<'_>,
    t_refs: &BoardTemplateRefs,
    cand_s: f64,
    cand_tx: f64,
    cand_ty: f64,
    accept_min: f64,
) -> Option<BoardLoc> {
    assert!(cand_s > 0.0);
    assert!(img.width > 0 && img.height > 0);
    let pts = &REF_BOARD_POINTS;
    let cx = (t_refs.x_min + t_refs.x_max) * 0.5;
    let cy = (t_refs.y_min + t_refs.y_max) * 0.5;
    let coarse_check =
        score_direct_gray(img, &t_refs.c8_pts, &t_refs.c8_tv, cand_s, cand_tx, cand_ty);
    if coarse_check < (accept_min - 0.16).max(0.46) {
        return None;
    }
    if coarse_check >= (accept_min - 0.08).max(0.68) {
        let exact = score_direct_sharp_sub(img, pts, &t_refs.tv, cand_s, cand_tx, cand_ty);
        if exact >= accept_min.max(0.78) {
            return Some(make_loc(
                cand_s, cand_tx, cand_ty, exact, img.width, img.height,
            ));
        }
    }
    let mut best_1px = (cand_s, cand_tx, cand_ty, -2.0_f64);
    for i in -1isize..=1 {
        let sc = cand_s * (1.0 + (i as f64) * 0.0025);
        let bx = cand_tx - (sc - cand_s) * cx;
        let by = cand_ty - (sc - cand_s) * cy;
        for dy in -2isize..=2 {
            for dx in -2isize..=2 {
                let tx = bx + (dx as f64);
                let ty = by + (dy as f64);
                let v = score_direct_gray(img, &t_refs.c4_pts, &t_refs.c4_tv, sc, tx, ty);
                if v > best_1px.3 {
                    best_1px = (sc, tx, ty, v);
                }
            }
        }
    }
    if best_1px.3 < (accept_min - 0.08).max(0.50) {
        return None;
    }
    let (sc, cx_tx, cy_ty, _) = best_1px;
    let mut best_fine = (sc, cx_tx, cy_ty, -2.0_f64);
    for hdy in -1isize..=1 {
        let ty = cy_ty + (hdy as f64) * 0.5;
        for hdx in -1isize..=1 {
            let tx = cx_tx + (hdx as f64) * 0.5;
            let v = score_direct_sharp_sub(img, pts, &t_refs.tv, sc, tx, ty);
            if v > best_fine.3 {
                best_fine = (sc, tx, ty, v);
            }
        }
    }
    (best_fine.3 >= accept_min).then(|| {
        make_loc(
            best_fine.0,
            best_fine.1,
            best_fine.2,
            best_fine.3,
            img.width,
            img.height,
        )
    })
}

fn scan_coarse_32_grid(
    img: &ImageView<'_>,
    t_refs: &BoardTemplateRefs,
    s_win: f64,
    tx_win: f64,
    ty_win: f64,
) -> Option<(f64, isize, isize, f64)> {
    let cn32 = t_refs.c32_pts.len();
    assert!(s_win > 0.0);
    assert!(cn32 <= 64);
    let wi = img.width as isize;
    let hi = img.height as isize;
    let cx = (t_refs.x_min + t_refs.x_max) * 0.5;
    let cy = (t_refs.y_min + t_refs.y_max) * 0.5;
    let mut best_coarse: Option<(f64, isize, isize, f64)> = None;
    let mut dxs32 = [0isize; 64];
    let mut dys32 = [0isize; 64];
    let mut row_bases32 = [0usize; 64];
    let mut vals32 = [0.0_f32; 64];
    let dt = 3.5 * s_win;

    for i in -4isize..=4 {
        let s = s_win * (1.0 + (i as f64) * 0.015);
        let bx = tx_win - (s - s_win) * cx;
        let by = ty_win - (s - s_win) * cy;
        for j in 0..cn32 {
            dxs32[j] = ((t_refs.c32_pts[j].0 as f64) * s).round() as isize;
            dys32[j] = ((t_refs.c32_pts[j].1 as f64) * s).round() as isize;
        }
        for dy_i in (-6isize..=12).step_by(2) {
            let oy = (by + (dy_i as f64) * dt).round() as isize;
            for j in 0..cn32 {
                let yi = (oy + dys32[j]).clamp(0, hi - 1) as usize;
                row_bases32[j] = yi * img.width * 4;
            }
            for dx_i in (-6isize..=6).step_by(2) {
                let ox = (bx + (dx_i as f64) * dt).round() as isize;
                for j in 0..cn32 {
                    let xi = (ox + dxs32[j]).clamp(0, wi - 1) as usize;
                    let o = row_bases32[j] + xi * 4;
                    vals32[j] = 0.299 * (img.data[o] as f32)
                        + 0.587 * (img.data[o + 1] as f32)
                        + 0.114 * (img.data[o + 2] as f32);
                }
                let sc32 = ncc_score_slice(&vals32[..cn32], &t_refs.c32_tv);
                if best_coarse.is_none_or(|b| sc32 > b.3) {
                    best_coarse = Some((s, dx_i, dy_i, sc32));
                }
            }
        }
    }
    best_coarse
}

fn refine_window_candidate(
    img: &ImageView<'_>,
    t_refs: &BoardTemplateRefs,
    s_win: f64,
    tx_win: f64,
    ty_win: f64,
    coarse: (f64, isize, isize, f64),
) -> Option<BoardLoc> {
    use rayon::prelude::*;
    assert!(s_win > 0.0);
    assert!(coarse.0 > 0.0);
    let (cs, cdx, cdy, csc32) = coarse;
    let cx = (t_refs.x_min + t_refs.x_max) * 0.5;
    let cy = (t_refs.y_min + t_refs.y_max) * 0.5;
    let dt = 3.5 * s_win;
    let bx0 = tx_win - (cs - s_win) * cx;
    let by0 = ty_win - (cs - s_win) * cy;
    let mut best_win = (cs, bx0 + (cdx as f64) * dt, by0 + (cdy as f64) * dt, csc32);
    if csc32 >= 0.36 {
        best_win.3 = -2.0;
        for fdy in (cdy - 1).max(-8)..=(cdy + 1).min(14) {
            for fdx in (cdx - 1).max(-8)..=(cdx + 1).min(8) {
                let ftx = bx0 + (fdx as f64) * dt;
                let fty = by0 + (fdy as f64) * dt;
                let sc8 = score_direct_gray(img, &t_refs.c8_pts, &t_refs.c8_tv, cs, ftx, fty);
                if sc8 > best_win.3 {
                    best_win = (cs, ftx, fty, sc8);
                }
            }
        }
    }
    let (ws, wtx, wty, wsc) = best_win;
    if wsc >= 0.48 {
        let dt_mid = 1.2 * s_win;
        if let Some((ms, mtx, mty, msc)) = (-3isize..=3)
            .into_par_iter()
            .map(|i| {
                let s = ws * (1.0 + (i as f64) * 0.005);
                let bx = wtx - (s - ws) * cx;
                let by = wty - (s - ws) * cy;
                let mut best = (s, bx, by, -2.0_f64);
                for dy in -3isize..=3 {
                    for dx in -3isize..=3 {
                        let tx = bx + (dx as f64) * dt_mid;
                        let ty = by + (dy as f64) * dt_mid;
                        let sc = score_direct_gray(img, &t_refs.c4_pts, &t_refs.c4_tv, s, tx, ty);
                        if sc > best.3 {
                            best = (s, tx, ty, sc);
                        }
                    }
                }
                best
            })
            .reduce_with(|a, b| if b.3 > a.3 { b } else { a })
            && msc >= 0.52
            && let Some(near) = refine_near_direct(img, t_refs, ms, mtx, mty, 0.65)
        {
            return Some(near);
        }
    }
    let aspect = (img.width as f64) / (img.height as f64);
    (1.45..=2.5)
        .contains(&aspect)
        .then(|| make_loc(ws, wtx, wty, wsc.min(0.60), img.width, img.height))
}

fn score_slice(lv: &PyrLevel, pxs: &[f64], pys: &[f64], tv: &[f32], ox: f64, oy: f64) -> f64 {
    let n = tv.len();
    assert!(lv.w > 0 && lv.h > 0);
    assert_eq!(pxs.len(), n);
    let mut sx = 0.0_f32;
    let mut sxx = 0.0_f32;
    let mut sxt = 0.0_f32;
    if lv.f > 1.0 {
        let lw = lv.w as isize;
        let lh = lv.h as isize;
        for i in 0..n {
            let xi = ((ox + pxs[i]) as isize).clamp(0, lw - 1) as usize;
            let yi = ((oy + pys[i]) as isize).clamp(0, lh - 1) as usize;
            let v = lv.g[yi * lv.w + xi];
            sx += v;
            sxx += v * v;
            sxt += v * tv[i];
        }
    } else {
        for i in 0..n {
            let v = sample(&lv.g, lv.w, lv.h, ox + pxs[i], oy + pys[i]);
            sx += v;
            sxx += v * v;
            sxt += v * tv[i];
        }
    }
    let sx64 = sx as f64;
    let varx = (sxx as f64) - (sx64 * sx64) / (n as f64);
    if varx > 1e-6 {
        (sxt as f64) / varx.sqrt()
    } else {
        -1.0
    }
}

fn build_gray_pyramid(img: &ImageView<'_>, bw: f64, bh: f64) -> (Vec<PyrLevel>, Vec<f64>) {
    assert!(img.width >= 64 && img.height >= 64);
    assert!(bw > 0.0 && bh > 0.0);
    let (g2, w2, h2) = half_gray_of(img);
    let mut levels = vec![PyrLevel {
        g: g2,
        w: w2,
        h: h2,
        f: 2.0,
    }];
    const PYR_LEVELS_MAX: usize = 16;
    while levels.len() < PYR_LEVELS_MAX
        && levels.last().unwrap().w > 120
        && levels.last().unwrap().h > 60
    {
        let last = levels.last().unwrap();
        let (dg, dw, dh) = half(&last.g, last.w, last.h);
        let nf = last.f * 2.0;
        levels.push(PyrLevel {
            g: dg,
            w: dw,
            h: dh,
            f: nf,
        });
    }
    let s_min = 0.12_f64.max((0.3 * (img.width as f64)) / bw);
    let s_max = 8.0_f64.min(1.1 * ((img.width as f64) / bw).max((img.height as f64) / bh));
    let mut scales = Vec::new();
    let mut s = s_min;
    const SCALES_MAX: usize = 256;
    while s <= s_max && scales.len() < SCALES_MAX {
        scales.push(s);
        s *= 1.05;
    }
    (levels, scales)
}

fn search_single_pyramid_scale(
    lv: &PyrLevel,
    s: f64,
    t_refs: &BoardTemplateRefs,
) -> Option<(f64, f64, f64, f64, f64)> {
    let cn = t_refs.c4_pts.len();
    assert!(cn > 0);
    assert!(lv.f > 0.0);
    let bw = t_refs.x_max - t_refs.x_min;
    let bh = t_refs.y_max - t_refs.y_min;
    let k = s / lv.f;
    let ox0 = (-t_refs.x_min * k - 0.06 * bw * k).ceil() as isize;
    let ox1 = ((lv.w as f64) - 1.0 - t_refs.x_max * k + 0.06 * bw * k).floor() as isize;
    let oy0 = (-t_refs.y_min * k - 0.08 * bh * k).ceil() as isize;
    let oy1 = ((lv.h as f64) - 1.0 - t_refs.y_max * k + 0.08 * bh * k).floor() as isize;
    if ox1 < ox0 || oy1 < oy0 {
        return None;
    }
    let step = if (ox1 - ox0 + 1) * (oy1 - oy0 + 1) > 12_000 {
        2usize
    } else {
        1usize
    };
    let lw = lv.w as isize;
    let lh = lv.h as isize;
    let dxs: Vec<isize> = t_refs
        .c4_pts
        .iter()
        .map(|p| ((p.0 as f64) * k) as isize)
        .collect();
    let dys: Vec<isize> = t_refs
        .c4_pts
        .iter()
        .map(|p| ((p.1 as f64) * k) as isize)
        .collect();
    let mut row_bases = vec![0usize; cn];
    let mut best: Option<(isize, isize, f64)> = None;
    for oy in (oy0..=oy1).step_by(step) {
        for j in 0..cn {
            let yi = (oy + dys[j]).clamp(0, lh - 1) as usize;
            row_bases[j] = yi * lv.w;
        }
        for ox in (ox0..=ox1).step_by(step) {
            let mut sx = 0.0_f32;
            let mut sxx = 0.0_f32;
            let mut sxt = 0.0_f32;
            for j in 0..cn {
                let xi = (ox + dxs[j]).clamp(0, lw - 1) as usize;
                let v = lv.g[row_bases[j] + xi];
                sx += v;
                sxx += v * v;
                sxt += v * t_refs.c4_tv[j];
            }
            let sx64 = sx as f64;
            let varx = (sxx as f64) - (sx64 * sx64) / (cn as f64);
            let sc = if varx > 1e-6 {
                (sxt as f64) / varx.sqrt()
            } else {
                -1.0
            };
            if best.is_none_or(|b| sc > b.2) {
                best = Some((ox, oy, sc));
            }
        }
    }
    best.map(|(bx, by, bsc)| (s, (bx as f64) * lv.f, (by as f64) * lv.f, lv.f, bsc))
}

fn search_pyramid_scales(
    levels: &[PyrLevel],
    scales: &[f64],
    t_refs: &BoardTemplateRefs,
) -> Vec<(f64, f64, f64, f64, f64)> {
    use rayon::prelude::*;
    assert!(!levels.is_empty());
    assert_eq!(t_refs.c4_pts.len(), t_refs.c4_tv.len());
    scales
        .par_iter()
        .filter_map(|&s| {
            let mut l_idx = 0;
            while l_idx + 1 < levels.len() && s / levels[l_idx + 1].f >= 0.14 {
                l_idx += 1;
            }
            search_single_pyramid_scale(&levels[l_idx], s, t_refs)
        })
        .collect()
}

fn grid_lv(
    cand: (f64, f64, f64, f64),
    lv: &PyrLevel,
    t_refs: &BoardTemplateRefs,
    ds: f64,
    steps: isize,
    dt: f64,
    reach: isize,
) -> (f64, f64, f64, f64) {
    let (c_s, c_tx, c_ty, c_score) = cand;
    assert!(c_s > 0.0);
    assert!(lv.f > 0.0);
    let pts = &REF_BOARD_POINTS;
    let n = pts.len();
    let cx = (t_refs.x_min + t_refs.x_max) * 0.5;
    let cy = (t_refs.y_min + t_refs.y_max) * 0.5;
    let mut best = (c_s, c_tx, c_ty, c_score);
    let mut pxs = vec![0.0_f64; n];
    let mut pys = vec![0.0_f64; n];
    for i in -steps..=steps {
        let s = c_s * (1.0 + (i as f64) * ds);
        let k = s / lv.f;
        for (idx, p) in pts.iter().enumerate() {
            pxs[idx] = (p.0 as f64) * k;
            pys[idx] = (p.1 as f64) * k;
        }
        let bx = c_tx - (s - c_s) * cx;
        let by = c_ty - (s - c_s) * cy;
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let tx = bx + (dx as f64) * dt;
                let ty = by + (dy as f64) * dt;
                let sc = score_slice(lv, &pxs, &pys, &t_refs.tv, tx / lv.f, ty / lv.f);
                if sc > best.3 {
                    best = (s, tx, ty, sc);
                }
            }
        }
    }
    best
}

fn fine_refine_candidate(
    img: &ImageView<'_>,
    t_refs: &BoardTemplateRefs,
    cand: (f64, f64, f64, f64),
) -> (f64, f64, f64, f64) {
    let (c_s, c_tx, c_ty, _) = cand;
    assert!(c_s > 0.0);
    assert!(!t_refs.tv.is_empty());
    let pts = &REF_BOARD_POINTS;
    let cx = (t_refs.x_min + t_refs.x_max) * 0.5;
    let cy = (t_refs.y_min + t_refs.y_max) * 0.5;
    let mut best = (c_s, c_tx, c_ty, -2.0_f64);
    for i in -3..=3 {
        let s = c_s * (1.0 + (i as f64) * 0.0025);
        let bx = c_tx - (s - c_s) * cx;
        let by = c_ty - (s - c_s) * cy;
        for dy in -3..=3 {
            for dx in -3..=3 {
                let tx = bx + (dx as f64) * 0.5;
                let ty = by + (dy as f64) * 0.5;
                let sc = score_direct_sharp_sub(img, pts, &t_refs.tv, s, tx, ty);
                if sc > best.3 {
                    best = (s, tx, ty, sc);
                }
            }
        }
    }
    best
}

fn refine_pyramid_candidates(
    img: &ImageView<'_>,
    levels: &[PyrLevel],
    t_refs: &BoardTemplateRefs,
    top: &[(f64, f64, f64, f64, f64)],
) -> Option<BoardLoc> {
    use rayon::prelude::*;
    assert!(!top.is_empty());
    assert!(!levels.is_empty());
    if top[0].4 < 0.45 {
        return Some(make_loc(
            top[0].0, top[0].1, top[0].2, top[0].4, img.width, img.height,
        ));
    }
    let mid_candidates: Vec<(f64, f64, f64, f64)> = top
        .par_iter()
        .take(5)
        .filter(|c| c.4 >= 0.45)
        .map(|&(cs, ctx, cty, cf, _)| {
            let mut l_idx = 0;
            while l_idx + 1 < levels.len() && cs / levels[l_idx + 1].f >= 0.5 {
                l_idx += 1;
            }
            grid_lv(
                (cs, ctx, cty, -2.0),
                &levels[l_idx],
                t_refs,
                0.01,
                4,
                1.0_f64.max(cf / 2.0),
                3,
            )
        })
        .collect();
    let best_mid = mid_candidates
        .iter()
        .copied()
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap())?;
    if best_mid.3 < 0.52 {
        return Some(make_loc(
            best_mid.0, best_mid.1, best_mid.2, best_mid.3, img.width, img.height,
        ));
    }
    let best_final = mid_candidates
        .par_iter()
        .filter(|c| c.3 >= 0.52)
        .map(|&cand| fine_refine_candidate(img, t_refs, cand))
        .reduce_with(|a, b| if b.3 > a.3 { b } else { a })?;
    Some(make_loc(
        best_final.0,
        best_final.1,
        best_final.2,
        best_final.3,
        img.width,
        img.height,
    ))
}

pub fn locate_board(img: &ImageView<'_>, hint: Option<BoardLoc>) -> Option<BoardLoc> {
    assert!(img.data.len() >= img.width * img.height * 4);
    assert!(!REF_BOARD_POINTS.is_empty());
    let (w, h) = (img.width, img.height);
    if w < 64 || h < 64 {
        return None;
    }
    let t_refs = board_template_refs();
    if let Some(hloc) = hint
        && hloc.w == w
        && hloc.h == h
        && let Some(near) = refine_near_direct(
            img,
            t_refs,
            hloc.s,
            hloc.tx,
            hloc.ty,
            0.65_f64.max(hloc.score - 0.08),
        )
    {
        return Some(near);
    }

    // Standard 16:9 game window board position (reference frame is 1344x756)
    let (wf, hf) = (w as f64, h as f64);
    let s_win = (wf / 1344.0).min(hf / 756.0);
    let tx_win = (wf - 1344.0 * s_win) * 0.5;
    let ty_win = (hf - 756.0 * s_win) * 0.5;
    if let Some(near) = refine_near_direct(img, t_refs, s_win, tx_win, ty_win, 0.75) {
        return Some(near);
    }
    let s_w = wf / 1344.0;
    for &(cand_s, cand_tx, cand_ty) in &[
        (s_w, wf * 0.5 - 658.0 * s_w, hf * 0.5 - 370.5 * s_w),
        (s_w, wf * 0.5 - 658.0 * s_w, 0.0),
        (s_w, (wf - 1344.0 * s_w) * 0.5, (hf - 756.0 * s_w) * 0.5),
        (s_win, wf * 0.5 - 658.0 * s_win, hf * 0.5 - 370.5 * s_win),
    ] {
        if let Some(near) = refine_near_direct(img, t_refs, cand_s, cand_tx, cand_ty, 0.75) {
            return Some(near);
        }
    }

    if let Some(coarse) = scan_coarse_32_grid(img, t_refs, s_win, tx_win, ty_win)
        && let Some(loc) = refine_window_candidate(img, t_refs, s_win, tx_win, ty_win, coarse)
    {
        return Some(loc);
    }

    let bw = t_refs.x_max - t_refs.x_min;
    let bh = t_refs.y_max - t_refs.y_min;
    let (levels, scales) = build_gray_pyramid(img, bw, bh);
    let mut top = search_pyramid_scales(&levels, &scales, t_refs);
    if top.is_empty() {
        return None;
    }
    top.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap());
    refine_pyramid_candidates(img, &levels, t_refs, &top)
}

struct LabelRansacBest {
    s: f64,
    cells: usize,
    inl: Vec<(usize, f64, f64)>,
}

fn unit_label_spots(w: usize, h: usize) -> ([(f64, f64); 16], f64) {
    assert!(w > 0 && h > 0);
    assert_eq!(RING.len(), 16);
    let unit = BoardLoc {
        s: 1.0,
        tx: 0.0,
        ty: 0.0,
        score: 0.0,
        w,
        h,
        by_labels: true,
        cells_matched: 0,
    };
    let spot: [(f64, f64); 16] = std::array::from_fn(|n| {
        if n == 0 {
            return (0.0, 0.0);
        }
        let (u, v) = RING[n];
        let (x, y) = unit.at(u as f64, v as f64);
        let k = unit.scale_at(u as f64, v as f64);
        let (ox, oy) = if v == 0 {
            (0.0, -88.0)
        } else if v == 3 {
            (0.0, 22.0)
        } else if u == 5 {
            (63.0, -38.0)
        } else {
            (-62.0, -39.0)
        };
        (x + ox * k, y + oy * k)
    });
    let cell_w = unit.at(1.0, 0.0).0 - unit.at(0.0, 0.0).0;
    (spot, cell_w)
}

fn collect_seen_cell_labels(
    img: &ImageView<'_>,
    slot: &mut ScanScratchSlot,
) -> Vec<(usize, f64, f64)> {
    assert!(img.width > 0 && img.height > 0);
    assert!(!ALLOWED_CELLS.is_empty());
    let mut seen: Vec<(usize, f64, f64)> = Vec::new();
    let rad = ((40.0_f64 / 3.0).round() as usize).max(3);

    for line in text_lines_radii_with_slot(
        img,
        [0.0, 0.0, (img.width - 1) as f64, (img.height - 1) as f64],
        (6.0, 40.0),
        &[rad],
        0.34,
        slot,
    )
    .into_iter()
    .next()
    .unwrap_or_default()
    {
        for p in &line.pieces {
            if p.words.len() != 1 || p.words[0].glyphs.is_empty() || p.words[0].glyphs.len() > 2 {
                continue;
            }
            let r = read_number(img, &p.words[0].glyphs, line.h, &ALLOWED_CELLS);
            if let Some(&(val, score)) = r.first()
                && score > 0.70
            {
                seen.push((
                    (val - 1) as usize,
                    (p.x0 + p.x1) as f64 * 0.5,
                    (p.y0 + p.y1) as f64 * 0.5,
                ));
            }
        }
    }
    seen
}

fn ransac_label_transform(
    seen: &[(usize, f64, f64)],
    spot: &[(f64, f64); 16],
    cell_w: f64,
) -> Option<LabelRansacBest> {
    assert!(cell_w > 0.0);
    assert_eq!(spot.len(), 16);
    let mut best: Option<LabelRansacBest> = None;
    for i in 0..seen.len() {
        for j in (i + 1)..seen.len() {
            let a = seen[i];
            let b = seen[j];
            if a.0 == b.0 {
                continue;
            }
            let dr = (spot[b.0].0 - spot[a.0].0).hypot(spot[b.0].1 - spot[a.0].1);
            if dr < cell_w * 0.9 {
                continue;
            }
            let s = (b.1 - a.1).hypot(b.2 - a.2) / dr;
            if !(0.2..=6.0).contains(&s) {
                continue;
            }
            let tx = a.1 - s * spot[a.0].0;
            let ty = a.2 - s * spot[a.0].1;
            let inl: Vec<(usize, f64, f64)> = seen
                .iter()
                .copied()
                .filter(|c| {
                    (s * spot[c.0].0 + tx - c.1).hypot(s * spot[c.0].1 + ty - c.2)
                        < 0.18 * cell_w * s
                })
                .collect();
            let mut uniq = 0u16;
            for c in &inl {
                uniq |= 1u16 << c.0;
            }
            let cells = uniq.count_ones() as usize;
            if best.as_ref().is_none_or(|b| cells > b.cells) {
                best = Some(LabelRansacBest { s, cells, inl });
            }
        }
    }
    best
}

pub fn locate_by_labels_with_slot(
    img: &ImageView<'_>,
    slot: &mut ScanScratchSlot,
) -> Option<BoardLoc> {
    assert!(img.width > 0 && img.height > 0);
    assert!(img.data.len() >= img.width * img.height * 4);
    let (spot, cell_w) = unit_label_spots(img.width, img.height);
    let seen = collect_seen_cell_labels(img, slot);
    let LabelRansacBest {
        s: best_s,
        cells: cells_matched,
        inl,
    } = ransac_label_transform(&seen, &spot, cell_w)?;
    if cells_matched < 5 {
        return None;
    }
    let m = inl.len() as f64;
    let rx = inl.iter().map(|c| spot[c.0].0).sum::<f64>() / m;
    let ry = inl.iter().map(|c| spot[c.0].1).sum::<f64>() / m;
    let px = inl.iter().map(|c| c.1).sum::<f64>() / m;
    let py = inl.iter().map(|c| c.2).sum::<f64>() / m;
    let mut num = 0.0_f64;
    let mut den = 0.0_f64;
    for c in &inl {
        num += (spot[c.0].0 - rx) * (c.1 - px) + (spot[c.0].1 - ry) * (c.2 - py);
        den += (spot[c.0].0 - rx).powi(2) + (spot[c.0].1 - ry).powi(2);
    }
    let s = if den > 0.0 { num / den } else { best_s };
    Some(BoardLoc {
        s,
        tx: px - s * rx,
        ty: py - s * ry,
        score: ((cells_matched as f64) / 15.0).min(1.0),
        w: img.width,
        h: img.height,
        by_labels: true,
        cells_matched,
    })
}

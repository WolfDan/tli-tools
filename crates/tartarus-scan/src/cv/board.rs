//! Multi-scale board template localization (`locate_board`) and cell-number RANSAC (`locate_by_labels_with_slot`).

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
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

pub(crate) fn board_template_refs() -> &'static BoardTemplateRefs {
    static REFS: std::sync::OnceLock<BoardTemplateRefs> = std::sync::OnceLock::new();
    REFS.get_or_init(|| {
        let pts = &REF_BOARD_POINTS;
        let n = pts.len();
        let mut mean = 0.0_f32;
        for p in pts {
            mean += p.2;
        }
        mean /= n as f32;
        let mut tv = vec![0.0_f32; n];
        let mut norm = 0.0_f32;
        for (i, p) in pts.iter().enumerate() {
            tv[i] = p.2 - mean;
            norm += tv[i] * tv[i];
        }
        norm = norm.sqrt().max(1e-6);
        for v in &mut tv {
            *v /= norm;
        }

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for p in pts {
            min_x = min_x.min(p.0 as f64);
            max_x = max_x.max(p.0 as f64);
            min_y = min_y.min(p.1 as f64);
            max_y = max_y.max(p.1 as f64);
        }

        let build_sub = |step: usize| -> (Vec<(i32, i32, f32)>, Vec<f32>) {
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
        };

        let (c4_pts, c4_tv) = build_sub(4);
        let (c8_pts, c8_tv) = build_sub(8);
        let (c32_pts, c32_tv) = build_sub(32);

        BoardTemplateRefs {
            tv,
            c4_pts,
            c4_tv,
            c8_pts,
            c8_tv,
            c32_pts,
            c32_tv,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    })
}

pub fn locate_board(img: &ImageView<'_>, hint: Option<BoardLoc>) -> Option<BoardLoc> {
    use rayon::prelude::*;

    let w = img.width;
    let h = img.height;
    if w < 64 || h < 64 {
        return None;
    }
    let pts = &REF_BOARD_POINTS;
    let n = pts.len();
    let t_refs = board_template_refs();
    let tv = &t_refs.tv;
    let c4_pts = &t_refs.c4_pts;
    let c4_tv = &t_refs.c4_tv;
    let c8_pts = &t_refs.c8_pts;
    let c8_tv = &t_refs.c8_tv;
    let c32_pts = &t_refs.c32_pts;
    let c32_tv = &t_refs.c32_tv;
    let min_x = t_refs.min_x;
    let max_x = t_refs.max_x;
    let min_y = t_refs.min_y;
    let max_y = t_refs.max_y;
    let bw = max_x - min_x;
    let bh = max_y - min_y;
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;

    let score_direct_gray =
        |sub_pts: &[(i32, i32, f32)], sub_tv: &[f32], s: f64, ox: f64, oy: f64| -> f64 {
            let m = sub_pts.len();
            let wi = w as isize;
            let hi = h as isize;
            let mut buf = [0.0_f32; 450];
            for i in 0..m {
                let xi = (ox + (sub_pts[i].0 as f64) * s) as isize;
                let yi = (oy + (sub_pts[i].1 as f64) * s) as isize;
                buf[i] = img.gray_px(xi.clamp(0, wi - 1), yi.clamp(0, hi - 1));
            }
            ncc_score_slice(&buf[..m], sub_tv)
        };

    let score_direct_sharp_sub =
        |sub_pts: &[(i32, i32, f32)], sub_tv: &[f32], s: f64, ox: f64, oy: f64| -> f64 {
            let m = sub_pts.len();
            let mut buf = [0.0_f32; 1800];
            for i in 0..m {
                buf[i] = img.sample_sharp(
                    ox + (sub_pts[i].0 as f64) * s,
                    oy + (sub_pts[i].1 as f64) * s,
                );
            }
            ncc_score_slice(&buf[..m], sub_tv)
        };

    // Zero-allocation refinement around a candidate (s, tx, ty):
    // 1. 225-point gate via img.gray_px (~0.4us)
    // 2. Fast exact-center check when stationary (~4us)
    // 3. 3-scale 1.0px integer grid search with 450 points + 0.5px peak refinement on winning scale
    let refine_near_direct =
        |cand_s: f64, cand_tx: f64, cand_ty: f64, min_accept: f64| -> Option<BoardLoc> {
            let coarse_check = score_direct_gray(c8_pts, c8_tv, cand_s, cand_tx, cand_ty);
            if coarse_check < (min_accept - 0.16).max(0.46) {
                return None;
            }
            if coarse_check >= (min_accept - 0.08).max(0.68) {
                let exact = score_direct_sharp_sub(pts, tv, cand_s, cand_tx, cand_ty);
                if exact >= min_accept.max(0.78) {
                    return Some(BoardLoc {
                        s: cand_s,
                        tx: cand_tx,
                        ty: cand_ty,
                        score: exact,
                        w,
                        h,
                        by_labels: false,
                        cells_matched: 0,
                    });
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
                        let v = score_direct_gray(c4_pts, c4_tv, sc, tx, ty);
                        if v > best_1px.3 {
                            best_1px = (sc, tx, ty, v);
                        }
                    }
                }
            }
            if best_1px.3 < (min_accept - 0.08).max(0.50) {
                return None;
            }
            let (sc, cx_tx, cy_ty, _) = best_1px;
            let mut best_fine = (sc, cx_tx, cy_ty, -2.0_f64);
            for hdy in -1isize..=1 {
                let ty = cy_ty + (hdy as f64) * 0.5;
                for hdx in -1isize..=1 {
                    let tx = cx_tx + (hdx as f64) * 0.5;
                    let v = score_direct_sharp_sub(pts, tv, sc, tx, ty);
                    if v > best_fine.3 {
                        best_fine = (sc, tx, ty, v);
                    }
                }
            }

            if best_fine.3 >= min_accept {
                Some(BoardLoc {
                    s: best_fine.0,
                    tx: best_fine.1,
                    ty: best_fine.2,
                    score: best_fine.3,
                    w,
                    h,
                    by_labels: false,
                    cells_matched: 0,
                })
            } else {
                None
            }
        };

    if let Some(hloc) = hint
        && hloc.w == w
        && hloc.h == h
        && let Some(near) =
            refine_near_direct(hloc.s, hloc.tx, hloc.ty, 0.65_f64.max(hloc.score - 0.08))
    {
        return Some(near);
    }

    // Standard 16:9 game window board position (reference frame is 1344x756)
    let s_win = ((w as f64) / 1344.0).min((h as f64) / 756.0);
    let tx_win = ((w as f64) - 1344.0 * s_win) * 0.5;
    let ty_win = ((h as f64) - 756.0 * s_win) * 0.5;
    if let Some(near) = refine_near_direct(s_win, tx_win, ty_win, 0.75) {
        return Some(near);
    }
    let s_w = (w as f64) / 1344.0;
    for &(cand_s, cand_tx, cand_ty) in &[
        (
            s_w,
            (w as f64) * 0.5 - 658.0 * s_w,
            (h as f64) * 0.5 - 370.5 * s_w,
        ),
        (s_w, (w as f64) * 0.5 - 658.0 * s_w, 0.0),
        (
            s_w,
            ((w as f64) - 1344.0 * s_w) * 0.5,
            ((h as f64) - 756.0 * s_w) * 0.5,
        ),
        (
            s_win,
            (w as f64) * 0.5 - 658.0 * s_win,
            (h as f64) * 0.5 - 370.5 * s_win,
        ),
    ] {
        if let Some(near) = refine_near_direct(cand_s, cand_tx, cand_ty, 0.75) {
            return Some(near);
        }
    }

    // Fast 9-scale window neighborhood pre-check using 56 points (~0.025ms when no board is present).
    let cn32 = c32_pts.len();
    let wi = w as isize;
    let hi = h as isize;
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
            dxs32[j] = ((c32_pts[j].0 as f64) * s).round() as isize;
            dys32[j] = ((c32_pts[j].1 as f64) * s).round() as isize;
        }
        for dy_i in (-6isize..=12).step_by(2) {
            let ty = by + (dy_i as f64) * dt;
            let oy = ty.round() as isize;
            for j in 0..cn32 {
                let yi = (oy + dys32[j]).clamp(0, hi - 1) as usize;
                row_bases32[j] = yi * w * 4;
            }
            for dx_i in (-6isize..=6).step_by(2) {
                let tx = bx + (dx_i as f64) * dt;
                let ox = tx.round() as isize;
                for j in 0..cn32 {
                    let xi = (ox + dxs32[j]).clamp(0, wi - 1) as usize;
                    let o = row_bases32[j] + xi * 4;
                    vals32[j] = 0.299 * (img.data[o] as f32)
                        + 0.587 * (img.data[o + 1] as f32)
                        + 0.114 * (img.data[o + 2] as f32);
                }
                let sc32 = ncc_score_slice(&vals32[..cn32], c32_tv);
                if best_coarse.is_none_or(|b| sc32 > b.3) {
                    best_coarse = Some((s, dx_i, dy_i, sc32));
                }
            }
        }
    }

    if let Some((cs, cdx, cdy, csc32)) = best_coarse {
        let mut best_win = (
            cs,
            tx_win - (cs - s_win) * cx + (cdx as f64) * dt,
            ty_win - (cs - s_win) * cy + (cdy as f64) * dt,
            csc32,
        );
        if csc32 >= 0.36 {
            let bx = tx_win - (cs - s_win) * cx;
            let by = ty_win - (cs - s_win) * cy;
            best_win.3 = -2.0;
            for fdy in (cdy - 1).max(-8)..=(cdy + 1).min(14) {
                for fdx in (cdx - 1).max(-8)..=(cdx + 1).min(8) {
                    let ftx = bx + (fdx as f64) * dt;
                    let fty = by + (fdy as f64) * dt;
                    let sc8 = score_direct_gray(c8_pts, c8_tv, cs, ftx, fty);
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
                            let sc = score_direct_gray(c4_pts, c4_tv, s, tx, ty);
                            if sc > best.3 {
                                best = (s, tx, ty, sc);
                            }
                        }
                    }
                    best
                })
                .reduce_with(|a, b| if b.3 > a.3 { b } else { a })
                && msc >= 0.52
                && let Some(near) = refine_near_direct(ms, mtx, mty, 0.65)
            {
                return Some(near);
            }
        }

        let aspect = (w as f64) / (h as f64);
        if (1.45..=2.5).contains(&aspect) {
            return Some(BoardLoc {
                s: ws,
                tx: wtx,
                ty: wty,
                score: wsc.min(0.60),
                w,
                h,
                by_labels: false,
                cells_matched: 0,
            });
        }
    }

    let score_slice = |g: &[f32],
                       lw_u: usize,
                       lh_u: usize,
                       pxs: &[f64],
                       pys: &[f64],
                       ox: f64,
                       oy: f64,
                       nearest: bool|
     -> f64 {
        let mut sx = 0.0_f32;
        let mut sxx = 0.0_f32;
        let mut sxt = 0.0_f32;
        if nearest {
            let lw = lw_u as isize;
            let lh = lh_u as isize;
            for i in 0..n {
                let xi = ((ox + pxs[i]) as isize).clamp(0, lw - 1) as usize;
                let yi = ((oy + pys[i]) as isize).clamp(0, lh - 1) as usize;
                let v = g[yi * lw_u + xi];
                sx += v;
                sxx += v * v;
                sxt += v * tv[i];
            }
        } else {
            for i in 0..n {
                let v = sample(g, lw_u, lh_u, ox + pxs[i], oy + pys[i]);
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
    };

    let score_pre =
        |lv: &PyrLevel, pxs: &[f64], pys: &[f64], ox: f64, oy: f64, nearest: bool| -> f64 {
            score_slice(&lv.g, lv.w, lv.h, pxs, pys, ox, oy, nearest)
        };

    // Fallback multi-scale pyramid for non-standard/cropped images
    let (g2, w2, h2) = half_gray_of(img);
    let mut levels = vec![PyrLevel {
        g: g2,
        w: w2,
        h: h2,
        f: 2.0,
    }];

    while levels.last().unwrap().w > 120 && levels.last().unwrap().h > 60 {
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

    let cn = c4_pts.len();
    let s_min = 0.12_f64.max((0.3 * (w as f64)) / bw);
    let s_max = 8.0_f64.min(1.1 * ((w as f64) / bw).max((h as f64) / bh));
    let mut scales = Vec::new();
    let mut s = s_min;
    while s <= s_max {
        scales.push(s);
        s *= 1.05;
    }

    let mut top: Vec<(f64, f64, f64, f64, f64)> = scales
        .par_iter()
        .filter_map(|&s| {
            let mut l_idx = 0;
            while l_idx + 1 < levels.len() && s / levels[l_idx + 1].f >= 0.14 {
                l_idx += 1;
            }
            let lv = &levels[l_idx];
            let k = s / lv.f;
            let ox0 = (-min_x * k - 0.06 * bw * k).ceil() as isize;
            let ox1 = ((lv.w as f64) - 1.0 - max_x * k + 0.06 * bw * k).floor() as isize;
            let oy0 = (-min_y * k - 0.08 * bh * k).ceil() as isize;
            let oy1 = ((lv.h as f64) - 1.0 - max_y * k + 0.08 * bh * k).floor() as isize;
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
            let dxs: Vec<isize> = c4_pts.iter().map(|p| ((p.0 as f64) * k) as isize).collect();
            let dys: Vec<isize> = c4_pts.iter().map(|p| ((p.1 as f64) * k) as isize).collect();
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
                        sxt += v * c4_tv[j];
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
        })
        .collect();

    if top.is_empty() {
        return None;
    }
    top.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap());

    if top[0].4 < 0.45 {
        return Some(BoardLoc {
            s: top[0].0,
            tx: top[0].1,
            ty: top[0].2,
            score: top[0].4,
            w,
            h,
            by_labels: false,
            cells_matched: 0,
        });
    }

    let grid_lv = |c_s: f64,
                   c_tx: f64,
                   c_ty: f64,
                   c_score: f64,
                   lv: &PyrLevel,
                   ds: f64,
                   steps: isize,
                   dt: f64,
                   reach: isize|
     -> (f64, f64, f64, f64) {
        let mut best = (c_s, c_tx, c_ty, c_score);
        let mut pxs = vec![0.0_f64; n];
        let mut pys = vec![0.0_f64; n];
        let nearest = lv.f > 1.0;
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
                    let sc = score_pre(lv, &pxs, &pys, tx / lv.f, ty / lv.f, nearest);
                    if sc > best.3 {
                        best = (s, tx, ty, sc);
                    }
                }
            }
        }
        best
    };

    let mid_candidates: Vec<(f64, f64, f64, f64)> = top
        .par_iter()
        .take(5)
        .filter(|c| c.4 >= 0.45)
        .map(|&(cs, ctx, cty, cf, _)| {
            let mut l_idx = 0;
            while l_idx + 1 < levels.len() && cs / levels[l_idx + 1].f >= 0.5 {
                l_idx += 1;
            }
            let mid = &levels[l_idx];
            grid_lv(cs, ctx, cty, -2.0, mid, 0.01, 4, 1.0_f64.max(cf / 2.0), 3)
        })
        .collect();

    let best_mid = mid_candidates
        .iter()
        .copied()
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap())?;

    if best_mid.3 < 0.52 {
        return Some(BoardLoc {
            s: best_mid.0,
            tx: best_mid.1,
            ty: best_mid.2,
            score: best_mid.3,
            w,
            h,
            by_labels: false,
            cells_matched: 0,
        });
    }

    let best_final = mid_candidates
        .par_iter()
        .filter(|c| c.3 >= 0.52)
        .map(|&(c_s, c_tx, c_ty, _)| {
            let mut best = (c_s, c_tx, c_ty, -2.0_f64);
            for i in -3..=3 {
                let s = c_s * (1.0 + (i as f64) * 0.0025);
                let bx = c_tx - (s - c_s) * cx;
                let by = c_ty - (s - c_s) * cy;
                for dy in -3..=3 {
                    for dx in -3..=3 {
                        let tx = bx + (dx as f64) * 0.5;
                        let ty = by + (dy as f64) * 0.5;
                        let sc = score_direct_sharp_sub(pts, tv, s, tx, ty);
                        if sc > best.3 {
                            best = (s, tx, ty, sc);
                        }
                    }
                }
            }
            best
        })
        .reduce_with(|a, b| if b.3 > a.3 { b } else { a })?;

    Some(BoardLoc {
        s: best_final.0,
        tx: best_final.1,
        ty: best_final.2,
        score: best_final.3,
        w,
        h,
        by_labels: false,
        cells_matched: 0,
    })
}

pub fn locate_by_labels_with_slot(
    img: &ImageView<'_>,
    slot: &mut ScanScratchSlot,
) -> Option<BoardLoc> {
    let unit = BoardLoc {
        s: 1.0,
        tx: 0.0,
        ty: 0.0,
        score: 0.0,
        w: img.width,
        h: img.height,
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

    struct LabelRansacBest {
        s: f64,
        cells: usize,
        inl: Vec<(usize, f64, f64)>,
    }

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

    let LabelRansacBest {
        s: best_s,
        cells: cells_matched,
        inl,
    } = best?;
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

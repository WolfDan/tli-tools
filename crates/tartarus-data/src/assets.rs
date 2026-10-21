//! S16 texture loading, alpha bounding-box detection, and CV art feature synthesis.

use std::collections::BTreeMap;
use std::path::Path;

use tartarus_extract::{PakVfs, crop_rgba8};

/// Decoded RGBA8 texture or sliced PaperSprite from the Pak VFS.
#[derive(Clone, Debug)]
pub struct DecodedAssetImage {
    pub virtual_path: String,
    pub short_name: String,
    pub width: usize,
    pub height: usize,
    pub rgba8: Vec<u8>,
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

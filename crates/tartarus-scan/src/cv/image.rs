//! RGBA `ImageView`, grayscale pyramid helpers, bilinear sampling, and RGB/HSV color predicates.

pub struct ImageView<'a> {
    pub width: usize,
    pub height: usize,
    pub data: &'a [u8], // RGBA
}

impl<'a> ImageView<'a> {
    #[inline]
    pub fn rgb_at(&self, x: isize, y: isize) -> (u8, u8, u8) {
        debug_assert!(self.width > 0);
        debug_assert!(self.height > 0);
        let xi = x.clamp(0, self.width as isize - 1) as usize;
        let yi = y.clamp(0, self.height as isize - 1) as usize;
        let o = (yi * self.width + xi) * 4;
        (self.data[o], self.data[o + 1], self.data[o + 2])
    }

    #[inline]
    pub fn hsv_at(&self, x: f64, y: f64) -> (f32, f32, f32) {
        debug_assert!(x.is_finite());
        debug_assert!(y.is_finite());
        let (r, g, b) = self.rgb_at(x.round() as isize, y.round() as isize);
        hsv(r as f32, g as f32, b as f32)
    }

    #[inline]
    pub fn gray_px(&self, x: isize, y: isize) -> f32 {
        debug_assert!(self.width > 0);
        debug_assert!(self.height > 0);
        let (r, g, b) = self.rgb_at(x, y);
        0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32)
    }

    #[inline]
    pub fn sharp_px(&self, x: isize, y: isize) -> f32 {
        debug_assert!(self.width > 0);
        debug_assert!(self.height > 0);
        let wi = self.width as isize;
        let hi = self.height as isize;
        if x > 0 && x + 1 < wi && y > 0 && y + 1 < hi {
            let w = self.width;
            let ux = (x as usize) - 1;
            let uy = (y as usize) - 1;
            let mut sr = 0u32;
            let mut sg = 0u32;
            let mut sb = 0u32;
            for dy in 0..3 {
                let base = ((uy + dy) * w + ux) * 4;
                let row = &self.data[base..base + 12];
                sr += (row[0] as u32) + (row[4] as u32) + (row[8] as u32);
                sg += (row[1] as u32) + (row[5] as u32) + (row[9] as u32);
                sb += (row[2] as u32) + (row[6] as u32) + (row[10] as u32);
            }
            (0.299 * (sr as f32) + 0.587 * (sg as f32) + 0.114 * (sb as f32)) * (1.0 / 9.0)
        } else {
            let mut acc = 0.0_f32;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    acc += self.gray_px(x + dx, y + dy);
                }
            }
            acc * (1.0 / 9.0)
        }
    }

    #[inline]
    pub fn sample_sharp(&self, mut x: f64, mut y: f64) -> f32 {
        debug_assert!(self.width > 0);
        debug_assert!(self.height > 0);
        let w_max = (self.width as f64) - 1.001;
        let h_max = (self.height as f64) - 1.001;
        x = x.clamp(0.0, w_max);
        y = y.clamp(0.0, h_max);
        let x0 = x as usize;
        let y0 = y as usize;
        let fx = (x - (x0 as f64)) as f32;
        let fy = (y - (y0 as f64)) as f32;
        if x0 >= 1 && x0 + 2 < self.width && y0 >= 1 && y0 + 2 < self.height {
            let w = self.width;
            let ux = x0 - 1;
            let uy = y0 - 1;
            let wx0 = 1.0 - fx;
            let wx3 = fx;
            let wy0 = 1.0 - fy;
            let wy3 = fy;
            let mut r_tot = 0.0_f32;
            let mut g_tot = 0.0_f32;
            let mut b_tot = 0.0_f32;
            for (dy, wy) in [(0usize, wy0), (1, 1.0), (2, 1.0), (3, wy3)] {
                let base = ((uy + dy) * w + ux) * 4;
                let row = &self.data[base..base + 16];
                let rr = wx0 * (row[0] as f32)
                    + ((row[4] as u32 + row[8] as u32) as f32)
                    + wx3 * (row[12] as f32);
                let gg = wx0 * (row[1] as f32)
                    + ((row[5] as u32 + row[9] as u32) as f32)
                    + wx3 * (row[13] as f32);
                let bb = wx0 * (row[2] as f32)
                    + ((row[6] as u32 + row[10] as u32) as f32)
                    + wx3 * (row[14] as f32);
                r_tot += wy * rr;
                g_tot += wy * gg;
                b_tot += wy * bb;
            }
            (0.299 * r_tot + 0.587 * g_tot + 0.114 * b_tot) * (1.0 / 9.0)
        } else {
            let ix0 = x0 as isize;
            let iy0 = y0 as isize;
            let s00 = self.sharp_px(ix0, iy0);
            let s10 = self.sharp_px(ix0 + 1, iy0);
            let s01 = self.sharp_px(ix0, iy0 + 1);
            let s11 = self.sharp_px(ix0 + 1, iy0 + 1);
            (s00 * (1.0 - fx) + s10 * fx) * (1.0 - fy) + (s01 * (1.0 - fx) + s11 * fx) * fy
        }
    }
}

#[inline]
pub fn hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    use palette::{FromColor, Hsv, Srgb};
    debug_assert!(r.is_finite() && g.is_finite() && b.is_finite());
    let c = Hsv::from_color(Srgb::new(r / 255.0, g / 255.0, b / 255.0));
    let out = (c.hue.into_positive_degrees(), c.saturation, c.value);
    debug_assert!(out.0.is_finite());
    out
}

#[inline]
fn quad_gray_px(data: &[u8], c0: usize, c2: usize) -> f32 {
    debug_assert!(c0 + 8 <= data.len());
    debug_assert!(c2 + 8 <= data.len());
    let c1 = c0 + 4;
    let c3 = c2 + 4;
    let r = (data[c0] as u32 + data[c1] as u32 + data[c2] as u32 + data[c3] as u32) as f32;
    let g = (data[c0 + 1] as u32 + data[c1 + 1] as u32 + data[c2 + 1] as u32 + data[c3 + 1] as u32)
        as f32;
    let b = (data[c0 + 2] as u32 + data[c1 + 2] as u32 + data[c2 + 2] as u32 + data[c3 + 2] as u32)
        as f32;
    0.25 * (0.299 * r + 0.587 * g + 0.114 * b)
}

pub fn half_gray_of(img: &ImageView<'_>) -> (Vec<f32>, usize, usize) {
    use rayon::prelude::*;
    assert!(img.width >= 2);
    assert!(img.height >= 2);
    let w = img.width;
    let w2 = w >> 1;
    let h2 = img.height >> 1;
    let mut out = vec![0.0_f32; w2 * h2];
    if w2 * h2 >= 25_000 {
        out.par_chunks_mut(w2)
            .enumerate()
            .for_each(|(y2, row_out)| {
                let r0 = (2 * y2) * w * 4;
                let r1 = (2 * y2 + 1) * w * 4;
                for (x2, out_px) in row_out.iter_mut().enumerate() {
                    let off = (2 * x2) * 4;
                    *out_px = quad_gray_px(img.data, r0 + off, r1 + off);
                }
            });
    } else {
        for y2 in 0..h2 {
            let r0 = (2 * y2) * w * 4;
            let r1 = (2 * y2 + 1) * w * 4;
            let row_out = &mut out[y2 * w2..(y2 + 1) * w2];
            for (x2, out_px) in row_out.iter_mut().enumerate() {
                let off = (2 * x2) * 4;
                *out_px = quad_gray_px(img.data, r0 + off, r1 + off);
            }
        }
    }
    (out, w2, h2)
}

pub fn half(g: &[f32], w: usize, h: usize) -> (Vec<f32>, usize, usize) {
    assert!(w >= 2);
    assert!(h >= 2);
    let w2 = w >> 1;
    let h2 = h >> 1;
    let mut out = vec![0.0_f32; w2 * h2];
    for y in 0..h2 {
        for x in 0..w2 {
            let i = 2 * y * w + 2 * x;
            out[y * w2 + x] = 0.25 * (g[i] + g[i + 1] + g[i + w] + g[i + w + 1]);
        }
    }
    (out, w2, h2)
}

#[inline]
pub fn sample(g: &[f32], w: usize, h: usize, mut x: f64, mut y: f64) -> f32 {
    debug_assert!(w >= 2);
    debug_assert!(h >= 2);
    if x < 0.0 {
        x = 0.0;
    } else if x > (w as f64) - 1.001 {
        x = (w as f64) - 1.001;
    }
    if y < 0.0 {
        y = 0.0;
    } else if y > (h as f64) - 1.001 {
        y = (h as f64) - 1.001;
    }
    let x0 = x as usize;
    let y0 = y as usize;
    let fx = (x - (x0 as f64)) as f32;
    let fy = (y - (y0 as f64)) as f32;
    let i = y0 * w + x0;
    (g[i] * (1.0 - fx) + g[i + 1] * fx) * (1.0 - fy)
        + (g[i + w] * (1.0 - fx) + g[i + w + 1] * fx) * fy
}

pub fn color_share<F>(img: &ImageView<'_>, x0: f64, y0: f64, x1: f64, y1: f64, mut test: F) -> f32
where
    F: FnMut(f32, f32, f32) -> bool,
{
    assert!(img.width > 0);
    assert!(img.height > 0);
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1 as isize).min(img.width as isize - 1);
    let by1 = (y1 as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return 0.0;
    }
    let mut n = 0usize;
    let mut hit = 0usize;
    for y in by0..=(by1 as usize) {
        for x in bx0..=(bx1 as usize) {
            n += 1;
            let (r, g, b) = img.rgb_at(x as isize, y as isize);
            let (hh, ss, vv) = hsv(r as f32, g as f32, b as f32);
            if test(hh, ss, vv) {
                hit += 1;
            }
        }
    }
    if n > 0 {
        (hit as f32) / (n as f32)
    } else {
        0.0
    }
}

#[inline]
pub fn color_share_rgb<F>(
    img: &ImageView<'_>,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    step: usize,
    test: F,
) -> f32
where
    F: Fn(u8, u8, u8) -> bool,
{
    assert!(img.width > 0);
    assert!(img.height > 0);
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1 as isize).min(img.width as isize - 1);
    let by1 = (y1 as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return 0.0;
    }
    let bx1 = bx1 as usize;
    let by1 = by1 as usize;
    let step = step.max(1);
    let mut n = 0usize;
    let mut hit = 0usize;
    for y in (by0..=by1).step_by(step) {
        let row_base = y * img.width * 4;
        for x in (bx0..=bx1).step_by(step) {
            n += 1;
            let o = row_base + x * 4;
            if test(img.data[o], img.data[o + 1], img.data[o + 2]) {
                hit += 1;
            }
        }
    }
    if n > 0 {
        (hit as f32) / (n as f32)
    } else {
        0.0
    }
}

#[inline]
pub fn is_jaws_rgb(r: u8, g: u8, b: u8) -> bool {
    if r < 141 || r < g || r < b {
        return false;
    }
    debug_assert!(r >= g);
    debug_assert!(r >= b);
    let ru = r as u32;
    let mn = g.min(b) as u32;
    let d = ru - mn;
    if 5 * d <= 2 * ru {
        return false;
    }
    if g >= b {
        15 * ((g - b) as u32) < 4 * d
    } else {
        3 * ((b - g) as u32) < d
    }
}

#[inline]
pub fn is_nest_rgb(r: u8, g: u8, b: u8) -> bool {
    if r < 115 || r <= g || g < b {
        return false;
    }
    debug_assert!(r > g);
    debug_assert!(g >= b);
    let ru = r as u32;
    let d = (r - b) as u32;
    let dg = (g - b) as u32;
    2 * d > ru && 3 * d <= 10 * dg && 30 * dg <= 23 * d
}

#[inline]
pub fn is_banner_red_rgb(r: u8, g: u8, b: u8) -> bool {
    if r < 103 || r < g || r < b {
        return false;
    }
    debug_assert!(r >= g);
    debug_assert!(r >= b);
    let ru = r as u32;
    let mn = g.min(b) as u32;
    let d = ru - mn;
    if 20 * d <= 9 * ru {
        return false;
    }
    if g >= b {
        15 * ((g - b) as u32) < 4 * d
    } else {
        3 * ((b - g) as u32) < d
    }
}

#[inline]
pub fn is_jaws(h: f32, s: f32, v: f32) -> bool {
    debug_assert!(h.is_finite());
    debug_assert!(s.is_finite() && v.is_finite());
    (h < 16.0 || h > 340.0) && s > 0.40 && v > 0.55
}

#[inline]
pub fn is_orange(h: f32, s: f32, v: f32) -> bool {
    debug_assert!(h.is_finite());
    debug_assert!(s.is_finite() && v.is_finite());
    (18.0..=46.0).contains(&h) && s > 0.60 && v > 0.55
}

#[inline]
pub fn is_gold(h: f32, s: f32, v: f32) -> bool {
    debug_assert!(h.is_finite());
    debug_assert!(s.is_finite() && v.is_finite());
    (32.0..=56.0).contains(&h) && s > 0.38 && s < 0.90 && v > 0.50
}

#[inline]
pub fn is_gold_rgb(r: u8, g: u8, b: u8) -> bool {
    if r < 128 || r <= g || g < b {
        return false;
    }
    debug_assert!(r > g);
    debug_assert!(g >= b);
    let ru = r as u32;
    let d = (r - b) as u32;
    let dg = (g - b) as u32;
    d * 50 > 19 * ru && d * 10 < 9 * ru && 8 * d <= 15 * dg && 15 * dg <= 14 * d
}

#[inline]
pub fn is_orange_rgb(r: u8, g: u8, b: u8) -> bool {
    if r < 141 || r <= g || g < b {
        return false;
    }
    debug_assert!(r > g);
    debug_assert!(g >= b);
    let ru = r as u32;
    let d = (r - b) as u32;
    let dg = (g - b) as u32;
    5 * d > 3 * ru && 3 * d <= 10 * dg && 30 * dg <= 23 * d
}

//! `FrameView` zero-copy pixel view and `std::simd::f32x16` correlation kernels.

use std::simd::f32x16;
use std::simd::num::SimdFloat;

use palette::{FromColor, Hsv, Srgb};

const FRAME_DIM_MAX: u32 = 16_384;

const _: () = {
    assert!(FRAME_DIM_MAX == 16_384);
    assert!(std::mem::size_of::<f32x16>() == 64);
};

/// Zero-copy RGBA8 frame view with explicit pixel dimensions (`_px`) and row stride (`_bytes`).
#[derive(Clone, Copy, Debug)]
pub struct FrameView<'a> {
    pub pixels_rgba8: &'a [u8],
    pub width_px: u32,
    pub height_px: u32,
    pub stride_bytes: u32,
}

impl<'a> FrameView<'a> {
    #[inline]
    pub fn new(pixels_rgba8: &'a [u8], width_px: u32, height_px: u32) -> Self {
        assert!(width_px > 0 && width_px <= FRAME_DIM_MAX);
        assert!(height_px > 0 && height_px <= FRAME_DIM_MAX);
        let stride_bytes = width_px * 4;
        assert!(pixels_rgba8.len() >= (stride_bytes as usize) * (height_px as usize));
        Self {
            pixels_rgba8,
            width_px,
            height_px,
            stride_bytes,
        }
    }

    #[inline]
    pub fn with_stride(
        pixels_rgba8: &'a [u8],
        width_px: u32,
        height_px: u32,
        stride_bytes: u32,
    ) -> Self {
        assert!(width_px > 0 && height_px > 0);
        assert!(stride_bytes >= width_px * 4);
        assert!(pixels_rgba8.len() >= (stride_bytes as usize) * (height_px as usize));
        Self {
            pixels_rgba8,
            width_px,
            height_px,
            stride_bytes,
        }
    }

    #[inline]
    pub const fn width(&self) -> usize {
        self.width_px as usize
    }

    #[inline]
    pub const fn height(&self) -> usize {
        self.height_px as usize
    }

    #[inline]
    pub fn row_offset(&self, y: usize) -> usize {
        debug_assert!(y < self.height());
        debug_assert!(self.stride_bytes >= self.width_px * 4);
        y * (self.stride_bytes as usize)
    }

    #[inline]
    pub fn rgb_at(&self, mut x: isize, mut y: isize) -> (u8, u8, u8) {
        debug_assert!(self.width_px > 0 && self.height_px > 0);
        x = x.clamp(0, (self.width_px as isize) - 1);
        y = y.clamp(0, (self.height_px as isize) - 1);
        let o = self.row_offset(y as usize) + (x as usize) * 4;
        debug_assert!(o + 2 < self.pixels_rgba8.len());
        (
            self.pixels_rgba8[o],
            self.pixels_rgba8[o + 1],
            self.pixels_rgba8[o + 2],
        )
    }

    #[inline]
    pub fn gray_px(&self, x: isize, y: isize) -> f32 {
        let (r, g, b) = self.rgb_at(x, y);
        let luma = 0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32);
        debug_assert!(luma.is_finite());
        debug_assert!(luma >= 0.0);
        luma
    }

    #[inline]
    pub fn gray_at(&self, mut x: f64, mut y: f64) -> f32 {
        debug_assert!(self.width_px >= 2);
        debug_assert!(self.height_px >= 2);
        x = x.clamp(0.0, (self.width_px as f64) - 1.001);
        y = y.clamp(0.0, (self.height_px as f64) - 1.001);
        let x0 = x as isize;
        let y0 = y as isize;
        let fx = (x - (x0 as f64)) as f32;
        let fy = (y - (y0 as f64)) as f32;
        (self.gray_px(x0, y0) * (1.0 - fx) + self.gray_px(x0 + 1, y0) * fx) * (1.0 - fy)
            + (self.gray_px(x0, y0 + 1) * (1.0 - fx) + self.gray_px(x0 + 1, y0 + 1) * fx) * fy
    }

    #[inline]
    pub fn sample_sharp(&self, x: f64, y: f64) -> f32 {
        debug_assert!(x.is_finite());
        debug_assert!(y.is_finite());
        let mut acc = 0.0_f32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                acc += self.gray_at(x + (dx as f64), y + (dy as f64));
            }
        }
        acc * (1.0 / 9.0)
    }

    #[inline]
    pub fn hsv_at(&self, x: f64, y: f64) -> (f32, f32, f32) {
        debug_assert!(x.is_finite());
        debug_assert!(y.is_finite());
        let (r, g, b) = self.rgb_at(x.round() as isize, y.round() as isize);
        hsv(r as f32, g as f32, b as f32)
    }
}

/// 4-way unrolled `f32x16` SIMD dot product over 256-float (`16x16`) aligned templates.
#[inline]
pub fn dot_vec256_simd(a: &[f32; 256], b: &[f32; 256]) -> f32 {
    debug_assert_eq!(a.len(), 256);
    debug_assert_eq!(b.len(), 256);
    let mut s0 = f32x16::splat(0.0);
    let mut s1 = f32x16::splat(0.0);
    let mut s2 = f32x16::splat(0.0);
    let mut s3 = f32x16::splat(0.0);
    let mut i = 0usize;
    while i < 256 {
        s0 += f32x16::from_slice(&a[i..i + 16]) * f32x16::from_slice(&b[i..i + 16]);
        s1 += f32x16::from_slice(&a[i + 16..i + 32]) * f32x16::from_slice(&b[i + 16..i + 32]);
        s2 += f32x16::from_slice(&a[i + 32..i + 48]) * f32x16::from_slice(&b[i + 32..i + 48]);
        s3 += f32x16::from_slice(&a[i + 48..i + 64]) * f32x16::from_slice(&b[i + 48..i + 64]);
        i += 64;
    }
    ((s0 + s1) + (s2 + s3)).reduce_sum()
}

/// 2-way unrolled `f32x16` SIMD dot product over 160-float (`10x16`) aligned digit templates.
#[inline]
pub fn dot_vec160_simd(a: &[f32; 160], b: &[f32; 160]) -> f32 {
    debug_assert_eq!(a.len(), 160);
    debug_assert_eq!(b.len(), 160);
    let mut s0 = f32x16::splat(0.0);
    let mut s1 = f32x16::splat(0.0);
    let mut i = 0usize;
    while i < 160 {
        s0 += f32x16::from_slice(&a[i..i + 16]) * f32x16::from_slice(&b[i..i + 16]);
        s1 += f32x16::from_slice(&a[i + 16..i + 32]) * f32x16::from_slice(&b[i + 16..i + 32]);
        i += 32;
    }
    (s0 + s1).reduce_sum()
}

/// `f32x16` SIMD dot product over 400-float (`40x10`) aligned prefix templates.
#[inline]
pub fn dot_vec400_simd(a: &[f32; 400], b: &[f32; 400]) -> f32 {
    debug_assert_eq!(a.len(), 400);
    debug_assert_eq!(b.len(), 400);
    let mut s0 = f32x16::splat(0.0);
    let mut i = 0usize;
    while i < 400 {
        s0 += f32x16::from_slice(&a[i..i + 16]) * f32x16::from_slice(&b[i..i + 16]);
        i += 16;
    }
    s0.reduce_sum()
}

/// General `f32x16` SIMD dot product for arbitrary-length slices.
#[inline]
pub fn dot_f32_simd(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    debug_assert_eq!(a.len(), b.len());
    let (a_chunks, a_rem) = a[..n].as_chunks::<16>();
    let (b_chunks, b_rem) = b[..n].as_chunks::<16>();
    let mut acc = f32x16::splat(0.0);
    for (ca, cb) in a_chunks.iter().zip(b_chunks.iter()) {
        acc += f32x16::from_array(*ca) * f32x16::from_array(*cb);
    }
    let mut sum = acc.reduce_sum();
    for (&xa, &xb) in a_rem.iter().zip(b_rem.iter()) {
        sum += xa * xb;
    }
    debug_assert!(sum.is_finite());
    sum
}

/// `f32x16` SIMD cosine similarity between two equal-length feature slices.
#[inline]
pub fn cosine_simd(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    debug_assert!(!a.is_empty());
    debug_assert_eq!(a.len(), b.len());
    let (a_chunks, a_rem) = a[..n].as_chunks::<16>();
    let (b_chunks, b_rem) = b[..n].as_chunks::<16>();
    let mut v_ab = f32x16::splat(0.0);
    let mut v_aa = f32x16::splat(0.0);
    let mut v_bb = f32x16::splat(0.0);
    for (ca, cb) in a_chunks.iter().zip(b_chunks.iter()) {
        let va = f32x16::from_array(*ca);
        let vb = f32x16::from_array(*cb);
        v_ab += va * vb;
        v_aa += va * va;
        v_bb += vb * vb;
    }
    let mut ab = v_ab.reduce_sum();
    let mut aa = v_aa.reduce_sum();
    let mut bb = v_bb.reduce_sum();
    for (&xa, &xb) in a_rem.iter().zip(b_rem.iter()) {
        ab += xa * xb;
        aa += xa * xa;
        bb += xb * xb;
    }
    if aa > 0.0 && bb > 0.0 {
        ab / (aa * bb).sqrt()
    } else {
        0.0
    }
}

/// Single-pass `f32x16` SIMD normalized cross-correlation against a zero-mean unit-norm template.
#[inline]
pub fn ncc_score_simd(vals: &[f32], tv_unit: &[f32]) -> f64 {
    let m = vals.len().min(tv_unit.len());
    debug_assert!(m > 0);
    debug_assert_eq!(vals.len(), tv_unit.len());
    let (v_chunks, v_rem) = vals[..m].as_chunks::<16>();
    let (t_chunks, t_rem) = tv_unit[..m].as_chunks::<16>();
    let mut v_sx = f32x16::splat(0.0);
    let mut v_sxx = f32x16::splat(0.0);
    let mut v_sxt = f32x16::splat(0.0);
    for (cv, ct) in v_chunks.iter().zip(t_chunks.iter()) {
        let v = f32x16::from_array(*cv);
        let t = f32x16::from_array(*ct);
        v_sx += v;
        v_sxx += v * v;
        v_sxt += v * t;
    }
    let mut sx = v_sx.reduce_sum();
    let mut sxx = v_sxx.reduce_sum();
    let mut sxt = v_sxt.reduce_sum();
    for (&xv, &xt) in v_rem.iter().zip(t_rem.iter()) {
        sx += xv;
        sxx += xv * xv;
        sxt += xv * xt;
    }
    let sx64 = sx as f64;
    let varx = (sxx as f64) - (sx64 * sx64) / (m as f64);
    if varx > 1e-6 {
        (sxt as f64) / varx.sqrt()
    } else {
        -1.0
    }
}

/// `f32x16` SIMD in-place zero-mean unit-L2-norm normalizer.
#[inline]
pub fn normalize_zero_mean_in_place_simd(v: &mut [f32], mean: f32) {
    assert!(!v.is_empty());
    let (chunks, rem) = v.as_chunks_mut::<16>();
    let mean_v = f32x16::splat(mean);
    let mut norm_v = f32x16::splat(0.0);
    for c in chunks.iter_mut() {
        let d = f32x16::from_array(*c) - mean_v;
        *c = d.to_array();
        norm_v += d * d;
    }
    let mut norm = norm_v.reduce_sum();
    for x in rem.iter_mut() {
        *x -= mean;
        norm += *x * *x;
    }
    assert!(norm >= 0.0);
    let inv = 1.0 / norm.sqrt().max(1.0);
    let inv_v = f32x16::splat(inv);
    for c in chunks.iter_mut() {
        *c = (f32x16::from_array(*c) * inv_v).to_array();
    }
    for x in rem.iter_mut() {
        *x *= inv;
    }
}

#[inline]
pub fn sample_bilinear(g: &[f32], w: usize, h: usize, mut x: f64, mut y: f64) -> f32 {
    debug_assert!(w >= 2 && h >= 2);
    debug_assert!(g.len() >= w * h);
    x = x.clamp(0.0, (w as f64) - 1.001);
    y = y.clamp(0.0, (h as f64) - 1.001);
    let x0 = x as usize;
    let y0 = y as usize;
    let fx = (x - (x0 as f64)) as f32;
    let fy = (y - (y0 as f64)) as f32;
    let i = y0 * w + x0;
    (g[i] * (1.0 - fx) + g[i + 1] * fx) * (1.0 - fy)
        + (g[i + w] * (1.0 - fx) + g[i + w + 1] * fx) * fy
}

pub fn half_gray_from_frame(frame: &FrameView<'_>) -> (Vec<f32>, usize, usize) {
    assert!(frame.width() >= 2);
    assert!(frame.height() >= 2);
    let w2 = frame.width() / 2;
    let h2 = frame.height() / 2;
    let mut out = vec![0.0_f32; w2 * h2];
    for y in 0..h2 {
        let r0 = frame.row_offset(y * 2);
        let r1 = frame.row_offset(y * 2 + 1);
        for x in 0..w2 {
            let x_off = x * 8;
            let o00 = r0 + x_off;
            let o10 = o00 + 4;
            let o01 = r1 + x_off;
            let o11 = o01 + 4;
            let r = (frame.pixels_rgba8[o00] as u32)
                + (frame.pixels_rgba8[o10] as u32)
                + (frame.pixels_rgba8[o01] as u32)
                + (frame.pixels_rgba8[o11] as u32);
            let g = (frame.pixels_rgba8[o00 + 1] as u32)
                + (frame.pixels_rgba8[o10 + 1] as u32)
                + (frame.pixels_rgba8[o01 + 1] as u32)
                + (frame.pixels_rgba8[o11 + 1] as u32);
            let b = (frame.pixels_rgba8[o00 + 2] as u32)
                + (frame.pixels_rgba8[o10 + 2] as u32)
                + (frame.pixels_rgba8[o01 + 2] as u32)
                + (frame.pixels_rgba8[o11 + 2] as u32);
            out[y * w2 + x] = 0.07475 * (r as f32) + 0.14675 * (g as f32) + 0.0285 * (b as f32);
        }
    }
    (out, w2, h2)
}

pub fn half_gray_level(g: &[f32], w: usize, h: usize) -> (Vec<f32>, usize, usize) {
    assert!(w >= 2 && h >= 2);
    assert!(g.len() >= w * h);
    let w2 = w / 2;
    let h2 = h / 2;
    let mut out = vec![0.0_f32; w2 * h2];
    for y in 0..h2 {
        for x in 0..w2 {
            let i = (y * 2) * w + (x * 2);
            out[y * w2 + x] = (g[i] + g[i + 1] + g[i + w] + g[i + w + 1]) * 0.25;
        }
    }
    (out, w2, h2)
}

#[inline]
pub fn hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    debug_assert!(r.is_finite() && g.is_finite() && b.is_finite());
    let c = Hsv::from_color(Srgb::new(r / 255.0, g / 255.0, b / 255.0));
    let out = (c.hue.into_positive_degrees(), c.saturation, c.value);
    debug_assert!(out.0 >= 0.0 && out.1 >= 0.0 && out.2 >= 0.0);
    out
}

#[inline]
pub fn is_gold(h: f32, s: f32, v: f32) -> bool {
    debug_assert!(h.is_finite());
    debug_assert!(s.is_finite());
    (25.0..=58.0).contains(&h) && s > 0.42 && v > 0.50
}

#[inline]
pub fn is_gold_rgb(r: u8, g: u8, b: u8) -> bool {
    debug_assert!((r as u32) + (g as u32) + (b as u32) <= 765);
    if r <= 127 || r < g || g <= b || ((r - b) as u32) * 50 <= 21 * (r as u32) {
        return false;
    }
    let (h, s, v) = hsv(r as f32, g as f32, b as f32);
    debug_assert!(h.is_finite());
    is_gold(h, s, v)
}

#[inline]
pub fn is_jaws_rgb(r: u8, g: u8, b: u8) -> bool {
    debug_assert!((r as u32) + (g as u32) + (b as u32) <= 765);
    let mn = g.min(b);
    if r <= 107 || r < g || r < b || ((r - mn) as u32) * 20 <= 9 * (r as u32) {
        return false;
    }
    let (h, s, v) = hsv(r as f32, g as f32, b as f32);
    debug_assert!(h.is_finite());
    (h < 18.0 || h > 340.0) && s > 0.45 && v > 0.42
}

#[inline]
pub fn is_nest_rgb(r: u8, g: u8, b: u8) -> bool {
    debug_assert!((r as u32) + (g as u32) + (b as u32) <= 765);
    let mn = r.min(b);
    if g <= 89 || g < r || g < b || ((g - mn) as u32) * 10 <= 3 * (g as u32) {
        return false;
    }
    let (h, s, v) = hsv(r as f32, g as f32, b as f32);
    debug_assert!(h.is_finite());
    (80.0..=165.0).contains(&h) && s > 0.30 && v > 0.35
}

#[inline]
pub fn is_orange_rgb(r: u8, g: u8, b: u8) -> bool {
    debug_assert!((r as u32) + (g as u32) + (b as u32) <= 765);
    if r <= 140 || r < g || g < b || ((r - b) as u32) * 2 <= (r as u32) {
        return false;
    }
    let (h, s, v) = hsv(r as f32, g as f32, b as f32);
    debug_assert!(h.is_finite());
    (12.0..=45.0).contains(&h) && s > 0.50 && v > 0.55
}

#[inline]
pub fn is_banner_red_rgb(r: u8, g: u8, b: u8) -> bool {
    debug_assert!((r as u32) + (g as u32) + (b as u32) <= 765);
    let mn = g.min(b);
    if r <= 89 || r < g || r < b || ((r - mn) as u32) * 5 <= 2 * (r as u32) {
        return false;
    }
    let (h, s, v) = hsv(r as f32, g as f32, b as f32);
    debug_assert!(h.is_finite());
    (h < 20.0 || h > 335.0) && s > 0.40 && v > 0.35
}

#[inline]
pub fn color_share_hsv<F>(frame: &FrameView<'_>, x0: f64, y0: f64, x1: f64, y1: f64, test: F) -> f32
where
    F: Fn(f32, f32, f32) -> bool,
{
    assert!(frame.width_px > 0);
    assert!(frame.height_px > 0);
    let mut hit = 0usize;
    let mut count = 0usize;
    let mut y = y0.round() as isize;
    let y_end = y1.round() as isize;
    let x_start = x0.round() as isize;
    let x_end = x1.round() as isize;
    while y <= y_end {
        let mut x = x_start;
        while x <= x_end {
            let (r, g, b) = frame.rgb_at(x, y);
            let (hh, ss, vv) = hsv(r as f32, g as f32, b as f32);
            if test(hh, ss, vv) {
                hit += 1;
            }
            count += 1;
            x += 1;
        }
        y += 1;
    }
    if count > 0 {
        (hit as f32) / (count as f32)
    } else {
        0.0
    }
}

#[inline]
pub fn color_share_rgb<F>(
    frame: &FrameView<'_>,
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
    assert!(frame.width_px > 0 && frame.height_px > 0);
    assert!(step >= 1);
    let step = step as isize;
    let mut hit = 0usize;
    let mut count = 0usize;
    let mut y = y0.round() as isize;
    let y_end = y1.round() as isize;
    let x_start = x0.round() as isize;
    let x_end = x1.round() as isize;
    while y <= y_end {
        let mut x = x_start;
        while x <= x_end {
            let (r, g, b) = frame.rgb_at(x, y);
            if test(r, g, b) {
                hit += 1;
            }
            count += 1;
            x += step;
        }
        y += step;
    }
    if count > 0 {
        (hit as f32) / (count as f32)
    } else {
        0.0
    }
}

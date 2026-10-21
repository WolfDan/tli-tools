//! SIMD vector math, normalized cross-correlation, and planar homography projection.

use super::data::{H0, PITCH0};

#[inline]
pub fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    use std::simd::StdFloat;
    use std::simd::prelude::*;
    debug_assert!(!a.is_empty());
    debug_assert_eq!(a.len(), b.len());
    let n = a.len().min(b.len());
    let mut acc0 = f32x8::splat(0.0);
    let mut acc1 = f32x8::splat(0.0);
    let mut i = 0usize;
    while i + 16 <= n {
        let va0 = f32x8::from_slice(&a[i..i + 8]);
        let vb0 = f32x8::from_slice(&b[i..i + 8]);
        let va1 = f32x8::from_slice(&a[i + 8..i + 16]);
        let vb1 = f32x8::from_slice(&b[i + 8..i + 16]);
        acc0 = va0.mul_add(vb0, acc0);
        acc1 = va1.mul_add(vb1, acc1);
        i += 16;
    }
    if i + 8 <= n {
        let va = f32x8::from_slice(&a[i..i + 8]);
        let vb = f32x8::from_slice(&b[i..i + 8]);
        acc0 = va.mul_add(vb, acc0);
        i += 8;
    }
    let mut sum = (acc0 + acc1).reduce_sum();
    while i < n {
        sum += a[i] * b[i];
        i += 1;
    }
    sum
}

#[inline]
pub(crate) fn normalize_zero_mean_in_place(v: &mut [f32], mean: f32) {
    use std::simd::StdFloat;
    use std::simd::prelude::*;
    debug_assert!(!v.is_empty());
    debug_assert!(mean.is_finite());
    let n = v.len();
    let vmean = f32x8::splat(mean);
    let mut nacc0 = f32x8::splat(0.0);
    let mut nacc1 = f32x8::splat(0.0);
    let mut i = 0usize;
    while i + 16 <= n {
        let x0 = f32x8::from_slice(&v[i..i + 8]) - vmean;
        let x1 = f32x8::from_slice(&v[i + 8..i + 16]) - vmean;
        x0.copy_to_slice(&mut v[i..i + 8]);
        x1.copy_to_slice(&mut v[i + 8..i + 16]);
        nacc0 = x0.mul_add(x0, nacc0);
        nacc1 = x1.mul_add(x1, nacc1);
        i += 16;
    }
    let mut norm = (nacc0 + nacc1).reduce_sum();
    while i < n {
        let x = v[i] - mean;
        v[i] = x;
        norm += x * x;
        i += 1;
    }
    let inv_norm = 1.0_f32 / norm.sqrt().max(1.0);
    let vinv = f32x8::splat(inv_norm);
    let mut j = 0usize;
    while j + 16 <= n {
        let x0 = f32x8::from_slice(&v[j..j + 8]) * vinv;
        let x1 = f32x8::from_slice(&v[j + 8..j + 16]) * vinv;
        x0.copy_to_slice(&mut v[j..j + 8]);
        x1.copy_to_slice(&mut v[j + 8..j + 16]);
        j += 16;
    }
    while j < n {
        v[j] *= inv_norm;
        j += 1;
    }
}

#[inline]
pub(crate) fn cosine_simd_impl(a: &[f32], b: &[f32]) -> f32 {
    use std::simd::StdFloat;
    use std::simd::prelude::*;
    debug_assert!(!a.is_empty());
    debug_assert_eq!(a.len(), b.len());
    let n = a.len().min(b.len());
    let mut v_ab = f32x8::splat(0.0);
    let mut v_aa = f32x8::splat(0.0);
    let mut v_bb = f32x8::splat(0.0);
    let mut i = 0usize;
    while i + 8 <= n {
        let va = f32x8::from_slice(&a[i..i + 8]);
        let vb = f32x8::from_slice(&b[i..i + 8]);
        v_ab = va.mul_add(vb, v_ab);
        v_aa = va.mul_add(va, v_aa);
        v_bb = vb.mul_add(vb, v_bb);
        i += 8;
    }
    let mut ab = v_ab.reduce_sum();
    let mut aa = v_aa.reduce_sum();
    let mut bb = v_bb.reduce_sum();
    while i < n {
        let x = a[i];
        let y = b[i];
        ab += x * y;
        aa += x * x;
        bb += y * y;
        i += 1;
    }
    if aa > 0.0 && bb > 0.0 {
        ab / (aa * bb).sqrt()
    } else {
        0.0
    }
}

#[inline]
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    debug_assert!(!a.is_empty());
    debug_assert_eq!(a.len(), b.len());
    cosine_simd_impl(a, b)
}

#[inline(always)]
pub(crate) fn ncc_accum_simd_impl(vals: &[f32], tv: &[f32]) -> (f32, f32, f32) {
    use std::simd::StdFloat;
    use std::simd::prelude::*;
    debug_assert!(!vals.is_empty());
    debug_assert_eq!(vals.len(), tv.len());
    let n = vals.len().min(tv.len());
    let mut v_sx = f32x8::splat(0.0);
    let mut v_sxx = f32x8::splat(0.0);
    let mut v_sxt = f32x8::splat(0.0);
    let mut i = 0usize;
    while i + 8 <= n {
        let v = f32x8::from_slice(&vals[i..i + 8]);
        let t = f32x8::from_slice(&tv[i..i + 8]);
        v_sx += v;
        v_sxx = v.mul_add(v, v_sxx);
        v_sxt = v.mul_add(t, v_sxt);
        i += 8;
    }
    let mut sx = v_sx.reduce_sum();
    let mut sxx = v_sxx.reduce_sum();
    let mut sxt = v_sxt.reduce_sum();
    while i < n {
        let v = vals[i];
        sx += v;
        sxx += v * v;
        sxt += v * tv[i];
        i += 1;
    }
    (sx, sxx, sxt)
}

#[inline]
pub(crate) fn ncc_score_slice(vals: &[f32], tv: &[f32]) -> f64 {
    debug_assert!(!vals.is_empty());
    debug_assert_eq!(vals.len(), tv.len());
    let m = vals.len().min(tv.len());
    let (sx, sxx, sxt) = ncc_accum_simd_impl(vals, tv);
    let sx64 = sx as f64;
    let varx = (sxx as f64) - (sx64 * sx64) / (m as f64);
    if varx > 1e-6 {
        (sxt as f64) / varx.sqrt()
    } else {
        -1.0
    }
}

#[inline]
pub fn project(h: &[f64; 9], u: f64, v: f64) -> (f64, f64) {
    debug_assert!(u.is_finite());
    debug_assert!(v.is_finite());
    let p = glam::DMat3::from_cols_array(h)
        .transpose()
        .mul_vec3(glam::DVec3::new(u, v, 1.0));
    (p.x / p.z, p.y / p.z)
}

pub fn invert(m: &[f64; 9]) -> [f64; 9] {
    debug_assert!(m[0].is_finite());
    debug_assert!(m[8].is_finite());
    glam::DMat3::from_cols_array(m)
        .transpose()
        .inverse()
        .transpose()
        .to_cols_array()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardLoc {
    pub s: f64,
    pub tx: f64,
    pub ty: f64,
    pub score: f64,
    pub w: usize,
    pub h: usize,
    pub by_labels: bool,
    pub cells_matched: usize,
}

impl BoardLoc {
    #[inline]
    pub fn at(&self, u: f64, v: f64) -> (f64, f64) {
        debug_assert!(self.s > 0.0);
        debug_assert!(u.is_finite() && v.is_finite());
        let (x, y) = project(&H0, u, v);
        (self.s * x + self.tx, self.s * y + self.ty)
    }

    #[inline]
    pub fn grid_of(&self, x: f64, y: f64) -> (f64, f64) {
        debug_assert!(self.s > 0.0);
        debug_assert!(x.is_finite() && y.is_finite());
        let h_inv = invert(&H0);
        project(&h_inv, (x - self.tx) / self.s, (y - self.ty) / self.s)
    }

    #[inline]
    pub fn scale_at(&self, u: f64, v: f64) -> f64 {
        debug_assert!(self.s > 0.0);
        debug_assert!(u.is_finite() && v.is_finite());
        let (x0, y0) = project(&H0, u - 0.5, v);
        let (x1, y1) = project(&H0, u + 0.5, v);
        (self.s * (x1 - x0).hypot(y1 - y0)) / PITCH0
    }
}

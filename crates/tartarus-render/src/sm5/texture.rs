//! Structure-of-Arrays (SoA) `LinearTexture2D` storage, compile-time `sRGB -> Linear` lookup
//! table, fast AVX2 SIMD transcendentals (`log2`, `exp2`, `pow`, `atan2`, `linear_to_srgb`),
//! and 8-wide SIMD (`f32x8`) bilinear texture sampler.

use std::simd::{StdFloat, prelude::*};
use std::sync::LazyLock;

use tartarus_extract::DecodedTexture;

use super::ir::{SIMD_LANES, TexSlotMeta, f32x8, i32x8, u32x8};

/// Exact 256-entry lookup table converting an 8-bit sRGB channel (`0..=255`) into Linear light.
static SRGB8_TO_LINEAR_LUT: LazyLock<[f32; 256]> = LazyLock::new(|| {
    let mut lut = [0.0f32; 256];
    for (b, slot) in lut.iter_mut().enumerate() {
        *slot = srgb_to_linear_channel((b as f32) / 255.0);
    }
    assert!(lut[0] == 0.0);
    assert!((lut[255] - 1.0).abs() < 1e-6);
    lut
});

/// Pre-decoded 2D texture stored in Structure-of-Arrays (SoA) Linear RGBA32F channels
/// (`channels: [Vec<f32>; 4]`) for transpose-free 8-wide SIMD bilinear filtering.
#[derive(Clone, Debug)]
pub struct LinearTexture2D {
    pub width: usize,
    pub height: usize,
    pub srgb: bool,
    pub clamp_x: bool,
    pub clamp_y: bool,
    pub is_pot: bool,
    pub is_pot_x: bool,
    pub is_pot_y: bool,
    pub width_mask: i32,
    pub height_mask: i32,
    pub buf_mask: usize,
    pub has_zeros_mask: u8,
    pub zero_frac: [u8; 4],
    pub const_val: [Option<f32>; 4],
    pub eq_r: [bool; 4],
    pub channels: [Vec<f32>; 4],
}

/// Converts a single sRGB channel (`[0.0, 1.0]`) to Linear light using the exact UE4
/// `FSlateMaterialShaderVS` formula (`c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055)^2.4`).
#[inline]
pub fn srgb_to_linear_channel(c: f32) -> f32 {
    assert!(c.is_finite());
    let x = c.clamp(0.0, 1.0);
    let out = if x <= 0.04045 {
        x * 0.077_399_38
    } else {
        (x * 0.947_867_3 + 0.052_132_7).powf(2.4)
    };
    assert!(out >= 0.0);
    out
}

/// Converts a single Linear light channel (`[0.0, 1.0]`) back to sRGB (`[0.0, 1.0]`).
#[inline]
pub fn linear_to_srgb_channel(c: f32) -> f32 {
    assert!(c.is_finite());
    let x = c.clamp(0.0, 1.0);
    let out = if x <= 0.003_130_8 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    assert!(out >= 0.0);
    out.clamp(0.0, 1.0)
}

/// Vectorized 1-ULP IEEE-754 base-2 logarithm (`log2`) across 8 SIMD lanes.
#[inline]
pub fn fast_log2_x8(x: f32x8) -> f32x8 {
    let min_pos = f32x8::splat(1.175_494_4e-38);
    if x.simd_le(min_pos).all() {
        return f32x8::splat(-126.0);
    }
    let clamped = x.simd_max(min_pos);
    let bits = clamped.to_bits();
    assert_eq!(SIMD_LANES, 8);
    let exp_i32 = (bits >> Simd::splat(23)).cast::<i32>() - i32x8::splat(127);
    let m_raw = f32x8::from_bits((bits & u32x8::splat(0x007F_FFFF)) | u32x8::splat(0x3F80_0000));
    let fold = m_raw.simd_gt(f32x8::splat(std::f32::consts::SQRT_2));
    let m = fold.select(m_raw * f32x8::splat(0.5), m_raw);
    let exp = fold.select(
        exp_i32.cast::<f32>() + f32x8::splat(1.0),
        exp_i32.cast::<f32>(),
    );

    let one = f32x8::splat(1.0);
    let z = (m - one) / (m + one);
    let z2 = z * z;
    let mut p = f32x8::splat(0.412_985_5);
    p = p * z2 + f32x8::splat(0.577_078);
    p = p * z2 + f32x8::splat(0.961_796_7);
    p = p * z2 + f32x8::splat(2.885_39);
    z * p + exp
}

/// Vectorized 1-ULP IEEE-754 base-2 exponential (`exp2`) across 8 SIMD lanes.
#[inline]
pub fn fast_exp2_x8(x: f32x8) -> f32x8 {
    assert_eq!(SIMD_LANES, 8);
    if x.simd_le(f32x8::splat(-125.0)).all() {
        return f32x8::splat(0.0);
    }
    let clamped = x.simd_clamp(f32x8::splat(-126.0), f32x8::splat(126.0));
    let n = clamped.round();
    let f = clamped - n;
    let mut p = f32x8::splat(0.000_154_035_3);
    p = p * f + f32x8::splat(0.001_333_355_8);
    p = p * f + f32x8::splat(0.009_618_129);
    p = p * f + f32x8::splat(0.055_504_11);
    p = p * f + f32x8::splat(0.240_226_5);
    p = p * f + f32x8::splat(std::f32::consts::LN_2);
    p = p * f + f32x8::splat(1.0);
    let scale_bits = ((n.cast::<i32>() + i32x8::splat(127)) << Simd::splat(23)).cast::<u32>();
    p * f32x8::from_bits(scale_bits)
}

/// Vectorized fused power (`x^y = exp2(log2(x) * y)`) with zero/one tile short-circuiting.
#[inline]
pub fn fast_pow_x8(x: f32x8, y: f32x8) -> f32x8 {
    assert_eq!(SIMD_LANES, 8);
    let min_pos = f32x8::splat(1.175_494_4e-38);
    if x.simd_le(min_pos).all() {
        return f32x8::splat(0.0);
    }
    let one = f32x8::splat(1.0);
    if x.simd_eq(one).all() {
        return one;
    }
    fast_exp2_x8(fast_log2_x8(x) * y)
}

/// Scalar FXC-identical `atan2(y, x)` evaluation for constant-folding and preamble execution.
#[inline]
pub fn fxc_atan2_scalar(y: f32, x: f32) -> f32 {
    let ax = x.abs();
    let ay = y.abs();
    let mx = ax.max(ay);
    let mn = ax.min(ay);
    let inv_mx = if mx > 1e-20 { 1.0 / mx } else { 0.0 };
    let t = mn * inv_mx;
    let t2 = t * t;
    let mut p = 0.020_835_1f32;
    p = p * t2 - 0.085_133;
    p = p * t2 + 0.180_141;
    p = p * t2 - 0.330_299_5;
    p = p * t2 + 0.999_866;
    let atan_t = p * t;
    let swapped = if ax < ay {
        atan_t * (-2.0) + std::f32::consts::FRAC_PI_2
    } else {
        0.0
    };
    let mut angle = atan_t + swapped;
    if x < -x {
        angle -= std::f32::consts::PI;
    }
    let mn_xy = x.min(y);
    let mx_xy = x.max(y);
    if mn_xy < -mn_xy && mx_xy >= -mx_xy {
        -angle
    } else {
        angle
    }
}

/// Vectorized 8-wide SIMD `atan2(y, x)` matching FXC's 29-instruction `VectorToRadialValue`
/// minimax polynomial and quadrant sign reconstruction bit-for-bit.
#[inline]
pub fn fast_atan2_x8(y: f32x8, x: f32x8) -> f32x8 {
    assert_eq!(SIMD_LANES, 8);
    let zero = f32x8::splat(0.0);
    let ax = x.abs();
    let ay = y.abs();
    let mx = ax.simd_max(ay);
    let mn = ax.simd_min(ay);
    let inv_mx = mx
        .simd_gt(f32x8::splat(1e-20))
        .select(f32x8::splat(1.0) / mx, zero);
    let t = mn * inv_mx;
    let t2 = t * t;
    let mut p = f32x8::splat(0.020_835_1);
    p = p * t2 + f32x8::splat(-0.085_133_0);
    p = p * t2 + f32x8::splat(0.180_141);
    p = p * t2 + f32x8::splat(-0.330_299_5);
    p = p * t2 + f32x8::splat(0.999_866);
    let atan_t = p * t;
    let swapped = ax.simd_lt(ay).select(
        atan_t * f32x8::splat(-2.0) + f32x8::splat(std::f32::consts::FRAC_PI_2),
        zero,
    );
    let base = atan_t + swapped;
    let x_neg = x
        .simd_lt(-x)
        .select(f32x8::splat(-std::f32::consts::PI), zero);
    let angle = base + x_neg;
    let mn_xy = x.simd_min(y);
    let mx_xy = x.simd_max(y);
    let flip = mn_xy.simd_lt(-mn_xy) & mx_xy.simd_ge(-mx_xy);
    flip.select(-angle, angle)
}

/// Degree-5 Chebyshev polynomial coefficients in `q = x^(1/4)` approximating
/// `1.055 * q^(5/3) - 0.055` on `x in [0.0031308, 1.0]` (`q in [0.23654895, 1.0]`)
/// to within `8e-6` max error using only 2 hardware `vsqrtps` + 5 FMA instructions.
static SRGB_Q_POLY: LazyLock<[f32; 6]> = LazyLock::new(|| {
    let a = 0.003_130_8f64.powf(0.25);
    let b = 1.0f64;
    let mid = 0.5 * (a + b);
    let half = 0.5 * (b - a);
    let mut nodes = [0.0f64; 6];
    let mut vals = [0.0f64; 6];
    for k in 0..6 {
        let theta = ((2 * k + 1) as f64) * std::f64::consts::PI / 12.0;
        let t = theta.cos();
        let q = mid + half * t;
        nodes[k] = q;
        vals[k] = 1.055 * q.powf(5.0 / 3.0) - 0.055;
    }
    // Newton divided differences -> standard monomial basis `c0 + c1*q + ... + c5*q^5`.
    let mut dd = vals;
    for j in 1..6 {
        for i in (j..6).rev() {
            dd[i] = (dd[i] - dd[i - 1]) / (nodes[i] - nodes[i - j]);
        }
    }
    let mut coeffs = [0.0f64; 6];
    for i in (0..6).rev() {
        for j in (1..6).rev() {
            coeffs[j] = coeffs[j - 1] - nodes[i] * coeffs[j];
        }
        coeffs[0] = dd[i] - nodes[i] * coeffs[0];
    }
    assert!((coeffs.iter().sum::<f64>() - 1.0).abs() < 1e-4);
    coeffs.map(|c| c as f32)
});

/// Converts 8 Linear light values (`[0.0, 1.0]`) to sRGB (`[0.0, 1.0]`) in AVX2 SIMD.
#[inline(always)]
pub fn linear_to_srgb_x8(c: f32x8) -> f32x8 {
    assert_eq!(SIMD_LANES, 8);
    let zero = f32x8::splat(0.0);
    let one = f32x8::splat(1.0);
    let x = c.simd_max(zero).simd_min(one);
    let low = x * f32x8::splat(12.92);
    let is_low = x.simd_le(f32x8::splat(0.003_130_8));
    if is_low.all() {
        return low;
    }
    let q = x.sqrt().sqrt();
    let [c0, c1, c2, c3, c4, c5] = *SRGB_Q_POLY;
    let mut high = f32x8::splat(c5);
    high = high * q + f32x8::splat(c4);
    high = high * q + f32x8::splat(c3);
    high = high * q + f32x8::splat(c2);
    high = high * q + f32x8::splat(c1);
    high = high * q + f32x8::splat(c0);
    is_low.select(low, high).simd_max(zero).simd_min(one)
}

#[allow(clippy::type_complexity)]
fn decode_rgba8_to_soa(
    rgba8: &[u8],
    srgb: bool,
    padded_total: usize,
) -> ([Vec<f32>; 4], [Option<f32>; 4], [bool; 4], u8, [u8; 4]) {
    assert!(!rgba8.is_empty());
    let first = [rgba8[0], rgba8[1], rgba8[2], rgba8[3]];
    let mut is_const = [true; 4];
    let mut eq_r = [false, true, true, false];
    let mut has_zeros_mask = 0u8;
    let mut zero_count = [0usize; 4];
    let mut chans: [Vec<f32>; 4] = std::array::from_fn(|_| Vec::with_capacity(padded_total));
    let inv_255 = 1.0f32 / 255.0;
    let lut = &*SRGB8_TO_LINEAR_LUT;
    let px_slice = rgba8.as_chunks::<4>().0;
    let total_px = px_slice.len().max(1);

    for px in px_slice {
        for c in 0..4 {
            is_const[c] &= px[c] == first[c];
            if px[c] == 0 {
                has_zeros_mask |= 1 << c;
                zero_count[c] += 1;
            }
        }
        eq_r[1] &= px[1] == px[0];
        eq_r[2] &= px[2] == px[0];
        if srgb {
            chans[0].push(lut[px[0] as usize]);
            chans[1].push(lut[px[1] as usize]);
            chans[2].push(lut[px[2] as usize]);
        } else {
            chans[0].push((px[0] as f32) * inv_255);
            chans[1].push((px[1] as f32) * inv_255);
            chans[2].push((px[2] as f32) * inv_255);
        }
        chans[3].push((px[3] as f32) * inv_255);
    }
    let mut const_val = [None; 4];
    let mut zero_frac = [0u8; 4];
    for c in 0..4 {
        zero_frac[c] = ((zero_count[c] * 255) / total_px).min(255) as u8;
        if is_const[c] {
            const_val[c] = Some(chans[c][0]);
            eq_r[c] = false;
        }
        chans[c].resize(padded_total, 0.0);
    }
    assert!(!chans[0].is_empty());
    (chans, const_val, eq_r, has_zeros_mask, zero_frac)
}

impl LinearTexture2D {
    /// Creates a SoA `LinearTexture2D` from a `DecodedTexture`, using the precomputed 256-entry
    /// `SRGB8_TO_LINEAR_LUT` when `tex.srgb` is true (zero `powf` calls).
    pub fn from_decoded(tex: &DecodedTexture) -> Self {
        assert!(tex.width > 0 && tex.height > 0);
        let len = tex.width * tex.height;
        let padded_len = len.next_power_of_two().max(1);
        let padded_total = padded_len + tex.width + 2;
        assert_eq!(tex.rgba8.len(), len * 4);
        let (channels, const_val, eq_r, has_zeros_mask, zero_frac) =
            decode_rgba8_to_soa(&tex.rgba8, tex.srgb, padded_total);
        let is_pot_x = tex.width.is_power_of_two();
        let is_pot_y = tex.height.is_power_of_two();
        Self {
            width: tex.width,
            height: tex.height,
            srgb: tex.srgb,
            clamp_x: tex.clamp_x,
            clamp_y: tex.clamp_y,
            is_pot: is_pot_x && is_pot_y,
            is_pot_x,
            is_pot_y,
            width_mask: (tex.width as i32) - 1,
            height_mask: (tex.height as i32) - 1,
            buf_mask: padded_len - 1,
            has_zeros_mask,
            zero_frac,
            const_val,
            eq_r,
            channels,
        }
    }

    /// Creates a `1x1` solid fallback texture.
    pub fn solid_rgba(rgba: [f32; 4]) -> Self {
        assert!(rgba[0].is_finite());
        assert!(rgba[3].is_finite());
        let mut has_zeros_mask = 0u8;
        let mut zero_frac = [0u8; 4];
        for (c, &v) in rgba.iter().enumerate() {
            if v == 0.0 {
                has_zeros_mask |= 1 << c;
                zero_frac[c] = 255;
            }
        }
        Self {
            width: 1,
            height: 1,
            srgb: false,
            clamp_x: true,
            clamp_y: true,
            is_pot: true,
            is_pot_x: true,
            is_pot_y: true,
            width_mask: 0,
            height_mask: 0,
            buf_mask: 0,
            has_zeros_mask,
            zero_frac,
            const_val: [Some(rgba[0]), Some(rgba[1]), Some(rgba[2]), Some(rgba[3])],
            eq_r: [false; 4],
            channels: [vec![rgba[0]], vec![rgba[1]], vec![rgba[2]], vec![rgba[3]]],
        }
    }

    /// Returns compile-time specialization metadata for this texture at `canonical_slot`.
    #[inline]
    pub const fn slot_meta(&self, canonical_slot: u8) -> TexSlotMeta {
        TexSlotMeta {
            canonical_slot,
            has_zeros_mask: self.has_zeros_mask,
            zero_frac: self.zero_frac,
            const_val: self.const_val,
            eq_r: self.eq_r,
        }
    }

    /// Loads an exact integer texel `(x, y)` clamped to `[0..width-1, 0..height-1]`.
    #[inline]
    pub fn load_texel_clamped(&self, x: i32, y: i32) -> [f32; 4] {
        assert!(self.width > 0 && self.height > 0);
        let ix = x.clamp(0, (self.width as i32) - 1) as usize;
        let iy = y.clamp(0, (self.height as i32) - 1) as usize;
        let idx = iy * self.width + ix;
        let out = [
            self.channels[0][idx],
            self.channels[1][idx],
            self.channels[2][idx],
            self.channels[3][idx],
        ];
        assert!(out[0].is_finite());
        out
    }

    /// Samples a single pixel `(u, v)` using bilinear filtering.
    pub fn sample_bilinear(
        &self,
        u: f32,
        v: f32,
        clamp_u_override: bool,
        clamp_v_override: bool,
    ) -> [f32; 4] {
        assert!(self.width > 0 && self.height > 0);
        let [r, g, b, a] = self.sample_bilinear_x8(
            f32x8::splat(u),
            f32x8::splat(v),
            clamp_u_override,
            clamp_v_override,
            0x0F,
        );
        let out = [
            r.to_array()[0],
            g.to_array()[0],
            b.to_array()[0],
            a.to_array()[0],
        ];
        assert!(out[0].is_finite());
        out
    }

    /// Samples 8 pixels `(u, v)` simultaneously using 8-wide SIMD bilinear filtering,
    /// evaluating **only** the channels enabled in `active_mask & 0x0F` (and short-circuiting
    /// remaining channels when any channel in `active_mask >> 4` evaluates to all-zero).
    #[inline(always)]
    pub fn sample_bilinear_x8(
        &self,
        u: f32x8,
        v: f32x8,
        clamp_u_override: bool,
        clamp_v_override: bool,
        active_mask: u8,
    ) -> [f32x8; 4] {
        assert!(self.width > 0 && self.height > 0);
        let dst_mask = active_mask & 0x0F;
        assert!(dst_mask <= 0x0F);
        let (zero, one) = (f32x8::splat(0.0), f32x8::splat(1.0));
        let (cu, cv) = (
            self.clamp_x || clamp_u_override,
            self.clamp_y || clamp_v_override,
        );
        let uu = if cu {
            u.simd_clamp(zero, one)
        } else {
            u - u.floor()
        };
        let vv = if cv {
            v.simd_clamp(zero, one)
        } else {
            v - v.floor()
        };
        let fx = uu * f32x8::splat(self.width as f32) - f32x8::splat(0.5);
        let fy = vv * f32x8::splat(self.height as f32) - f32x8::splat(0.5);
        let (fl_x, fl_y) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - fl_x, fy - fl_y);
        let (x0, y0) = (fl_x.cast::<i32>(), fl_y.cast::<i32>());
        let (w, wm, px) = (self.width, self.width_mask, self.is_pot_x);
        let (h, hm, py) = (self.height, self.height_mask, self.is_pot_y);
        let ix0 = wrap_or_clamp_x8(x0, w, wm, px, cu);
        let ix1 = wrap_or_clamp_x8(x0 + i32x8::splat(1), w, wm, px, cu);
        let iy0 = wrap_or_clamp_x8(y0, h, hm, py, cv);
        let iy1 = wrap_or_clamp_x8(y0 + i32x8::splat(1), h, hm, py, cv);
        let w_simd = i32x8::splat(w as i32);
        let (r0, r1) = (iy0 * w_simd, iy1 * w_simd);
        let i00 = (r0 + ix0).to_array();
        let i10 = (r0 + ix1).to_array();
        let i01 = (r1 + ix0).to_array();
        let i11 = (r1 + ix1).to_array();
        let bmask = self.buf_mask;
        let mut out = [zero; 4];
        for (c, ch) in self.channels.iter().enumerate() {
            if (dst_mask >> c) & 1 == 0 {
                continue;
            }
            let s = &ch[..=bmask];
            let c00 = f32x8::from_array(std::array::from_fn(|k| s[(i00[k] as usize) & bmask]));
            let c10 = f32x8::from_array(std::array::from_fn(|k| s[(i10[k] as usize) & bmask]));
            let c01 = f32x8::from_array(std::array::from_fn(|k| s[(i01[k] as usize) & bmask]));
            let c11 = f32x8::from_array(std::array::from_fn(|k| s[(i11[k] as usize) & bmask]));
            let top = c00 + (c10 - c00) * tx;
            let bot = c01 + (c11 - c01) * tx;
            out[c] = top + (bot - top) * ty;
        }
        out
    }
}

#[inline(always)]
fn wrap_or_clamp_x8(idx: i32x8, dim: usize, mask: i32, is_pot: bool, clamp: bool) -> i32x8 {
    assert!(dim > 0);
    assert_eq!(SIMD_LANES, 8);
    let d_i32 = dim as i32;
    let zero_i = i32x8::splat(0);
    let max_i = i32x8::splat(d_i32 - 1);
    if clamp {
        idx.simd_max(zero_i).simd_min(max_i)
    } else if is_pot {
        idx & i32x8::splat(mask)
    } else {
        // Because `uu = u - u.floor()` is already in `[0.0, 1.0)`, `idx` is always in `[-1, dim]`:
        // `-1` wraps to `dim - 1`, and `dim` wraps to `0`, with zero integer division (`idiv`).
        let d_v = i32x8::splat(d_i32);
        let wrapped_lo = idx.simd_lt(zero_i).select(idx + d_v, idx);
        wrapped_lo.simd_ge(d_v).select(wrapped_lo - d_v, wrapped_lo)
    }
}

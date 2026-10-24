//! Structure-of-Arrays (SoA) `LinearTexture2D` storage, compile-time `sRGB -> Linear` lookup
//! table, fast AVX2 SIMD transcendentals (`log2`, `exp2`, `pow`, `atan2`, `linear_to_srgb`),
//! and 8-wide SIMD (`f32x8`) bilinear texture sampler.

use std::simd::{StdFloat, prelude::*};

use tartarus_extract::DecodedTexture;

use super::ir::{SIMD_LANES, TexSlotMeta, f32x8, i32x8, u32x8};

const fn const_ln64(x: f64) -> f64 {
    assert!(x > 0.0);
    let bits = x.to_bits();
    let mut exp = ((bits >> 52) & 0x7ff) as i64 - 1023;
    let mut m = f64::from_bits((bits & ((1u64 << 52) - 1)) | (1023u64 << 52));
    if m > std::f64::consts::SQRT_2 {
        m *= 0.5;
        exp += 1;
    }
    let u = (m - 1.0) / (m + 1.0);
    let u2 = u * u;
    let mut term = u;
    let mut sum = u;
    let mut k = 1usize;
    while k <= 14 {
        term *= u2;
        sum += term / ((2 * k + 1) as f64);
        k += 1;
    }
    assert!(sum.is_finite());
    2.0 * sum + (exp as f64) * std::f64::consts::LN_2
}

const fn const_exp64(x: f64) -> f64 {
    assert!(x > -700.0 && x < 700.0);
    let inv_ln2 = std::f64::consts::LOG2_E;
    let k_f = x * inv_ln2;
    let k = if k_f >= 0.0 {
        (k_f + 0.5) as i64
    } else {
        (k_f - 0.5) as i64
    };
    let r = x - (k as f64) * std::f64::consts::LN_2;
    let mut term = 1.0f64;
    let mut sum = 1.0f64;
    let mut n = 1usize;
    while n <= 18 {
        term *= r / (n as f64);
        sum += term;
        n += 1;
    }
    let scale = f64::from_bits(((1023 + k) as u64) << 52);
    assert!(sum > 0.0);
    sum * scale
}

const fn const_powf64(base: f64, exp: f64) -> f64 {
    assert!(base >= 0.0);
    assert!(exp.is_finite());
    if base == 0.0 {
        0.0
    } else {
        const_exp64(exp * const_ln64(base))
    }
}

const fn const_cos64(theta: f64) -> f64 {
    assert!(theta >= 0.0 && theta <= std::f64::consts::PI);
    let t2 = theta * theta;
    let mut term = 1.0f64;
    let mut sum = 1.0f64;
    let mut n = 1usize;
    while n <= 16 {
        let d = ((2 * n - 1) * (2 * n)) as f64;
        term = -term * t2 / d;
        sum += term;
        n += 1;
    }
    assert!(sum >= -1.000_001 && sum <= 1.000_001);
    sum
}

const fn build_srgb8_to_linear_lut() -> [f32; 256] {
    let mut lut = [0.0f32; 256];
    let mut b = 0usize;
    while b < 256 {
        let c = (b as f64) / 255.0;
        let lin = if c <= 0.04045 {
            c / 12.92
        } else {
            const_powf64((c + 0.055) / 1.055, 2.4)
        };
        lut[b] = lin as f32;
        b += 1;
    }
    assert!(lut[0] == 0.0);
    assert!((lut[255] - 1.0).abs() < 1e-6);
    lut
}

/// Exact 256-entry compile-time lookup table converting an 8-bit sRGB channel (`0..=255`) into Linear light.
const SRGB8_TO_LINEAR_LUT: [f32; 256] = build_srgb8_to_linear_lut();

/// Pre-decoded 2D texture stored in Structure-of-Arrays (SoA) Linear RGBA32F channels
/// (`channels: [Vec<f32>; 4]`) for transpose-free 8-wide SIMD bilinear filtering.
#[derive(Clone, Debug)]
pub struct LinearTexture2D {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub width_f32: f32,
    pub height_f32: f32,
    pub width_m1_f32: f32,
    pub height_m1_f32: f32,
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
    pub nonzero_uv_bounds: [[f32; 4]; 4],
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

const fn build_srgb_q_poly() -> [f32; 6] {
    let a = const_powf64(0.003_130_8, 0.25);
    let b = 1.0f64;
    let mid = 0.5 * (a + b);
    let half = 0.5 * (b - a);
    let mut nodes = [0.0f64; 6];
    let mut vals = [0.0f64; 6];
    let mut k = 0usize;
    while k < 6 {
        let theta = ((2 * k + 1) as f64) * std::f64::consts::PI / 12.0;
        let t = const_cos64(theta);
        let q = mid + half * t;
        nodes[k] = q;
        vals[k] = 1.055 * const_powf64(q, 5.0 / 3.0) - 0.055;
        k += 1;
    }
    // Newton divided differences -> standard monomial basis `c0 + c1*q + ... + c5*q^5`.
    let mut dd = vals;
    let mut j = 1usize;
    while j < 6 {
        let mut i = 5usize;
        while i >= j {
            dd[i] = (dd[i] - dd[i - 1]) / (nodes[i] - nodes[i - j]);
            i -= 1;
        }
        j += 1;
    }
    let mut coeffs = [0.0f64; 6];
    let mut i = 6usize;
    while i > 0 {
        i -= 1;
        let mut m = 5usize;
        while m >= 1 {
            coeffs[m] = coeffs[m - 1] - nodes[i] * coeffs[m];
            m -= 1;
        }
        coeffs[0] = dd[i] - nodes[i] * coeffs[0];
    }
    let sum = coeffs[0] + coeffs[1] + coeffs[2] + coeffs[3] + coeffs[4] + coeffs[5];
    assert!((sum - 1.0).abs() < 1e-4);
    assert!(coeffs[5].is_finite());
    [
        coeffs[0] as f32,
        coeffs[1] as f32,
        coeffs[2] as f32,
        coeffs[3] as f32,
        coeffs[4] as f32,
        coeffs[5] as f32,
    ]
}

/// Degree-5 Chebyshev polynomial coefficients in `q = x^(1/4)` approximating
/// `1.055 * q^(5/3) - 0.055` on `x in [0.0031308, 1.0]` (`q in [0.23654895, 1.0]`)
/// to within `8e-6` max error using only 2 hardware `vsqrtps` + 5 FMA instructions.
const SRGB_Q_POLY: [f32; 6] = build_srgb_q_poly();

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
    let [c0, c1, c2, c3, c4, c5] = SRGB_Q_POLY;
    let mut high = f32x8::splat(c5);
    high = high * q + f32x8::splat(c4);
    high = high * q + f32x8::splat(c3);
    high = high * q + f32x8::splat(c2);
    high = high * q + f32x8::splat(c1);
    high = high * q + f32x8::splat(c0);
    is_low.select(low, high).simd_max(zero).simd_min(one)
}

type DecodedSoaData = (
    [Vec<f32>; 4],
    [Option<f32>; 4],
    [bool; 4],
    u8,
    [u8; 4],
    [[f32; 4]; 4],
);

fn compute_nonzero_uv_rect(
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
    width: usize,
    height: usize,
) -> [f32; 4] {
    assert!(width > 0 && height > 0);
    if min_x > max_x {
        return [1.0, 1.0, 0.0, 0.0];
    }
    let u0 = if min_x > 1 && max_x + 1 < width {
        ((min_x as f32) - 0.5) / (width as f32)
    } else {
        0.0
    };
    let u1 = if min_x > 0 && max_x + 2 < width {
        ((max_x as f32) + 1.5) / (width as f32)
    } else {
        1.0
    };
    let v0 = if min_y > 1 && max_y + 1 < height {
        ((min_y as f32) - 0.5) / (height as f32)
    } else {
        0.0
    };
    let v1 = if min_y > 0 && max_y + 2 < height {
        ((max_y as f32) + 1.5) / (height as f32)
    } else {
        1.0
    };
    [u0, v0, u1, v1]
}

fn decode_rgba8_to_soa(
    rgba8: &[u8],
    width: usize,
    height: usize,
    srgb: bool,
    padded_total: usize,
) -> DecodedSoaData {
    assert!(!rgba8.is_empty());
    assert_eq!(rgba8.len(), width * height * 4);
    let stride = width + 1;
    let first = [rgba8[0], rgba8[1], rgba8[2], rgba8[3]];
    let mut is_const = [true; 4];
    let mut eq_r = [false, true, true, false];
    let mut has_zeros_mask = 0u8;
    let mut zero_count = [0usize; 4];
    let (mut min_x, mut max_x) = ([width; 4], [0usize; 4]);
    let (mut min_y, mut max_y) = ([height; 4], [0usize; 4]);
    let mut chans: [Vec<f32>; 4] = std::array::from_fn(|_| Vec::with_capacity(padded_total));
    let inv_255 = 1.0f32 / 255.0;
    let lut = &SRGB8_TO_LINEAR_LUT;
    let px_slice = rgba8.as_chunks::<4>().0;
    let total_px = px_slice.len().max(1);

    for y in 0..height {
        let row_px = &px_slice[y * width..(y + 1) * width];
        for (x, px) in row_px.iter().enumerate() {
            for c in 0..4 {
                is_const[c] &= px[c] == first[c];
                if px[c] == 0 {
                    has_zeros_mask |= 1 << c;
                    zero_count[c] += 1;
                } else {
                    min_x[c] = min_x[c].min(x);
                    max_x[c] = max_x[c].max(x);
                    min_y[c] = min_y[c].min(y);
                    max_y[c] = max_y[c].max(y);
                }
            }
            eq_r[1] &= px[1] == px[0];
            eq_r[2] &= px[2] == px[0];
            for c in 0..3 {
                let v = if srgb {
                    lut[px[c] as usize]
                } else {
                    (px[c] as f32) * inv_255
                };
                chans[c].push(v);
            }
            chans[3].push((px[3] as f32) * inv_255);
        }
        // Right wrap-guard column: `row[y][width] = row[y][0]`.
        for ch in &mut chans {
            let row0_val = ch[y * stride];
            ch.push(row0_val);
        }
    }
    let mut const_val = [None; 4];
    let mut zero_frac = [0u8; 4];
    let mut nz_bounds = [[0.0f32, 0.0, 1.0, 1.0]; 4];
    for c in 0..4 {
        // Bottom wrap-guard row: `row[height][0..=width] = row[0][0..=width]`.
        chans[c].extend_from_within(0..stride);
        zero_frac[c] = ((zero_count[c] * 255) / total_px).min(255) as u8;
        if is_const[c] {
            const_val[c] = Some(chans[c][0]);
            eq_r[c] = false;
        }
        nz_bounds[c] =
            compute_nonzero_uv_rect(min_x[c], max_x[c], min_y[c], max_y[c], width, height);
        chans[c].resize(padded_total, 0.0);
    }
    (chans, const_val, eq_r, has_zeros_mask, zero_frac, nz_bounds)
}

impl LinearTexture2D {
    /// Creates a SoA `LinearTexture2D` from a `DecodedTexture`, using the precomputed 256-entry
    /// `SRGB8_TO_LINEAR_LUT` when `tex.srgb` is true (zero `powf` calls).
    pub fn from_decoded(tex: &DecodedTexture) -> Self {
        assert!(tex.width > 0 && tex.height > 0);
        let stride = tex.width + 1;
        let guarded_len = stride * (tex.height + 1);
        let padded_len = guarded_len.next_power_of_two().max(1);
        let padded_total = padded_len + stride + 2;
        assert_eq!(tex.rgba8.len(), tex.width * tex.height * 4);
        let (channels, const_val, eq_r, has_zeros_mask, zero_frac, nonzero_uv_bounds) =
            decode_rgba8_to_soa(&tex.rgba8, tex.width, tex.height, tex.srgb, padded_total);
        let is_pot_x = tex.width.is_power_of_two();
        let is_pot_y = tex.height.is_power_of_two();
        Self {
            width: tex.width,
            height: tex.height,
            stride,
            width_f32: tex.width as f32,
            height_f32: tex.height as f32,
            width_m1_f32: (tex.width.saturating_sub(1)) as f32,
            height_m1_f32: (tex.height.saturating_sub(1)) as f32,
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
            nonzero_uv_bounds,
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
            stride: 2,
            width_f32: 1.0,
            height_f32: 1.0,
            width_m1_f32: 0.0,
            height_m1_f32: 0.0,
            srgb: false,
            clamp_x: true,
            clamp_y: true,
            is_pot: true,
            is_pot_x: true,
            is_pot_y: true,
            width_mask: 0,
            height_mask: 0,
            buf_mask: 3,
            has_zeros_mask,
            zero_frac,
            const_val: [Some(rgba[0]), Some(rgba[1]), Some(rgba[2]), Some(rgba[3])],
            eq_r: [false; 4],
            nonzero_uv_bounds: [[0.0, 0.0, 1.0, 1.0]; 4],
            channels: [
                vec![rgba[0]; 8],
                vec![rgba[1]; 8],
                vec![rgba[2]; 8],
                vec![rgba[3]; 8],
            ],
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
        let idx = iy * self.stride + ix;
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

    /// Samples 8 pixels `(u, v)` simultaneously using 8-wide SIMD bilinear filtering over the
    /// 2D wrap-guard padded SoA layout (`stride = width + 1`), evaluating only `active_mask & 0x0F`.
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
        let cu = self.clamp_x || clamp_u_override;
        let cv = self.clamp_y || clamp_v_override;
        let (ix0, tx) = coord_and_frac_x8(
            u,
            self.width_f32,
            self.width_m1_f32,
            self.width as i32,
            self.width_mask,
            self.is_pot_x,
            cu,
        );
        let (iy0, ty) = coord_and_frac_x8(
            v,
            self.height_f32,
            self.height_m1_f32,
            self.height as i32,
            self.height_mask,
            self.is_pot_y,
            cv,
        );
        let stride = self.stride;
        let bmask = self.buf_mask;
        let i00 = (iy0 * i32x8::splat(stride as i32) + ix0).to_array();
        let mut out = [f32x8::splat(0.0); 4];
        if active_mask >= 16 {
            let kc = ((active_mask >> 4).trailing_zeros() as usize) & 3;
            let kv = sample_ch_x8(&self.channels[kc], &i00, tx, ty, stride, bmask);
            if kv.to_bits() == u32x8::splat(0) {
                return out;
            }
            out[kc] = kv;
        }
        let rem_mask = dst_mask & !(active_mask >> 4);
        for (c, ch) in self.channels.iter().enumerate() {
            if (rem_mask >> c) & 1 != 0 {
                out[c] = sample_ch_x8(ch, &i00, tx, ty, stride, bmask);
            }
        }
        out
    }
}

#[inline(always)]
fn sample_ch_x8(
    ch: &[f32],
    i00: &[i32; SIMD_LANES],
    tx: f32x8,
    ty: f32x8,
    stride: usize,
    bmask: usize,
) -> f32x8 {
    assert!(ch.len() > bmask);
    assert_eq!(SIMD_LANES, 8);
    let s = &ch[..=bmask];
    let c00 = f32x8::from_array(std::array::from_fn(|k| s[(i00[k] as usize) & bmask]));
    let c10 = f32x8::from_array(std::array::from_fn(|k| {
        s[(i00[k] as usize).wrapping_add(1) & bmask]
    }));
    let c01 = f32x8::from_array(std::array::from_fn(|k| {
        s[(i00[k] as usize).wrapping_add(stride) & bmask]
    }));
    let c11 = f32x8::from_array(std::array::from_fn(|k| {
        s[(i00[k] as usize).wrapping_add(stride + 1) & bmask]
    }));
    let top = c00 + (c10 - c00) * tx;
    let bot = c01 + (c11 - c01) * tx;
    top + (bot - top) * ty
}

#[inline(always)]
fn coord_and_frac_x8(
    coord: f32x8,
    dim_f32: f32,
    dim_m1_f32: f32,
    dim_i32: i32,
    mask: i32,
    is_pot: bool,
    clamp: bool,
) -> (i32x8, f32x8) {
    assert!(dim_i32 > 0);
    assert_eq!(SIMD_LANES, 8);
    let zero = f32x8::splat(0.0);
    let half = f32x8::splat(0.5);
    if clamp {
        let uu = coord.simd_clamp(zero, f32x8::splat(1.0));
        let fx = (uu * f32x8::splat(dim_f32) - half).simd_clamp(zero, f32x8::splat(dim_m1_f32));
        let fl = fx.floor();
        (fl.cast::<i32>(), fx - fl)
    } else {
        // Because `uu = coord - coord.floor()` is in `[0.0, 1.0)`, `x0` is in `[-1, dim - 1]`:
        // `-1` wraps to `dim - 1`, and `ix0 + 1` reads the wrap-guard texel at `row[dim]`.
        let uu = coord - coord.floor();
        let fx = uu * f32x8::splat(dim_f32) - half;
        let fl = fx.floor();
        let x0 = fl.cast::<i32>();
        let ix0 = if is_pot {
            x0 & i32x8::splat(mask)
        } else {
            x0.simd_lt(i32x8::splat(0))
                .select(x0 + i32x8::splat(dim_i32), x0)
        };
        (ix0, fx - fl)
    }
}

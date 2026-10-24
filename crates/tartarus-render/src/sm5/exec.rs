//! Stage 1 Scalar Uniform Preamble Executor, Stage 2 8-Wide SIMD (`f32x8` / `u32x8`)
//! Wavefront Kernel Executor (with a dedicated branchless straight-line fast path),
//! and `4x2` Dual-Quad Tile Rasterizer.

use std::simd::{StdFloat, prelude::*, simd_swizzle};

use super::ir::{
    CompiledSm5Shader, MAX_CF_DEPTH, MAX_EXEC_STEPS, MAX_INDEXABLE_TEMP_LEN, REG_CB0_1_W,
    REG_CB0_1_X, REG_FILE_SIZE, REG_ONE, REG_OUT_A, REG_OUT_B, REG_OUT_G, REG_OUT_R, REG_REAL_TIME,
    REG_TRUE, REG_V0_W, REG_V0_X, REG_V0_Y, REG_V1_A, REG_V1_B, REG_V1_G, REG_V1_R, REG_V2_X,
    REG_V2_Y, REG_V3_U, REG_V3_V, REG_V4_W, REG_V4_X, REG_V4_Y, REG_V4_Z, SIMD_LANES,
    ShaderSpecialization, VmInst, VmOp, f32x8, i32x8, u32x8,
};
use super::texture::{
    LinearTexture2D, fast_atan2_x8, fast_exp2_x8, fast_log2_x8, fast_pow_x8, fxc_atan2_scalar,
    linear_to_srgb_channel, linear_to_srgb_rgb_x8, linear_to_srgb_x8,
};

/// Lane X offsets within a `4x2` pixel tile (two adjacent `2x2` pixel quads).
const TILE_DX: [f32; SIMD_LANES] = [0.0, 1.0, 0.0, 1.0, 2.0, 3.0, 2.0, 3.0];
/// Lane Y offsets within a `4x2` pixel tile (two adjacent `2x2` pixel quads).
const TILE_DY: [f32; SIMD_LANES] = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
/// Integer X offsets within a `4x2` pixel tile.
const TILE_IX: [u32; SIMD_LANES] = [0, 1, 0, 1, 2, 3, 2, 3];
/// Integer Y offsets within a `4x2` pixel tile.
const TILE_IY: [u32; SIMD_LANES] = [0, 0, 1, 1, 0, 0, 1, 1];

#[inline]
fn fast_sin_x8(x: f32x8) -> f32x8 {
    assert_eq!(SIMD_LANES, 8);
    let inv_two_pi = f32x8::splat(std::f32::consts::FRAC_1_PI * 0.5);
    let two_pi = f32x8::splat(std::f32::consts::TAU);
    let pi_half = f32x8::splat(std::f32::consts::FRAC_PI_2);
    let k = (x * inv_two_pi).round();
    let t = x - k * two_pi;
    let abs_t = t.abs();
    let folded = pi_half - (abs_t - pi_half).abs();
    let u = t.is_sign_negative().select(-folded, folded);
    let u2 = u * u;
    let mut p = f32x8::splat(-0.000_198_074);
    p = p * u2 + f32x8::splat(0.008_333_025);
    p = p * u2 + f32x8::splat(-0.166_666_57);
    (u * u2) * p + u
}

#[inline]
fn fast_cos_x8(x: f32x8) -> f32x8 {
    assert_eq!(SIMD_LANES, 8);
    fast_sin_x8(x + f32x8::splat(std::f32::consts::FRAC_PI_2))
}

#[inline]
fn ddx_quad_x8(v: f32x8) -> f32x8 {
    assert_eq!(SIMD_LANES, 8);
    let right = simd_swizzle!(v, [1, 1, 3, 3, 5, 5, 7, 7]);
    let left = simd_swizzle!(v, [0, 0, 2, 2, 4, 4, 6, 6]);
    right - left
}

#[inline]
fn ddy_quad_x8(v: f32x8) -> f32x8 {
    assert_eq!(SIMD_LANES, 8);
    let bot = simd_swizzle!(v, [2, 3, 2, 3, 6, 7, 6, 7]);
    let top = simd_swizzle!(v, [0, 1, 0, 1, 4, 5, 4, 5]);
    bot - top
}

#[inline]
fn sanitize_f32_scalar(val: f32, saturate: bool) -> u32 {
    let mut v = if val.is_nan() { 0.0 } else { val };
    if saturate {
        v = v.clamp(0.0, 1.0);
    }
    assert!(!v.is_nan());
    v.to_bits()
}

#[inline(always)]
fn write_u32x8(regs: &mut [u32x8; REG_FILE_SIZE], dst: u8, val: u32x8, exec_mask: u8) {
    assert_eq!(SIMD_LANES, 8);
    let d = dst as usize;
    if exec_mask == 0xFF {
        regs[d] = val;
    } else if exec_mask != 0 {
        let m = Mask::<i32, SIMD_LANES>::from_bitmask(exec_mask as u64);
        regs[d] = m.select(val, regs[d]);
    }
}

#[inline]
fn f32_to_f16_bits(val: f32) -> u32 {
    let bits = val.to_bits();
    let sign = (bits >> 16) & 0x8000;
    let exp = ((bits >> 23) & 0xFF) as i32 - 127 + 15;
    let mant = bits & 0x007F_FFFF;
    if exp <= 0 {
        sign
    } else if exp >= 31 {
        sign | 0x7C00
    } else {
        sign | ((exp as u32) << 10) | (mant >> 13)
    }
}

#[inline]
fn f16_to_f32_bits(h: u32) -> u32 {
    let sign = (h & 0x8000) << 16;
    let exp = (h >> 10) & 0x1F;
    let mant = h & 0x03FF;
    if exp == 0 {
        if mant == 0 {
            sign
        } else {
            let v = (mant as f32) * (1.0 / 16_777_216.0);
            if sign != 0 {
                (-v).to_bits()
            } else {
                v.to_bits()
            }
        }
    } else if exp == 31 {
        sign | 0x7F80_0000 | (mant << 13)
    } else {
        sign | ((exp + 127 - 15) << 23) | (mant << 13)
    }
}

fn eval_scalar_float_op(op: VmOp, a: f32, b: f32, c: f32) -> Option<(f32, bool)> {
    let (res, sat) = match op {
        VmOp::MovSat => (a, true),
        VmOp::FAdd => (a + b, false),
        VmOp::FSub => (a - b, false),
        VmOp::FMul => (a * b, false),
        VmOp::FMul3 => ((a * b) * c, false),
        VmOp::FMad => (a * b + c, false),
        VmOp::FNmad => (c - a * b, false),
        VmOp::FDiv => (if b.abs() > 1e-20 { a / b } else { 0.0 }, false),
        VmOp::FMin => (a.min(fb_clean(b)), false),
        VmOp::FMax => (a.max(fb_clean(b)), false),
        VmOp::FFrc => (a - a.floor(), false),
        VmOp::FRoundNe => (a.round_ties_even(), false),
        VmOp::FFloor => (a.floor(), false),
        VmOp::FCeil => (a.ceil(), false),
        VmOp::FTrunc => (a.trunc(), false),
        VmOp::FSqrt => (a.max(0.0).sqrt(), false),
        VmOp::FRsq => (if a > 1e-20 { 1.0 / a.sqrt() } else { 0.0 }, false),
        VmOp::FRcp => (if a.abs() > 1e-20 { 1.0 / a } else { 0.0 }, false),
        VmOp::FExp2 => (a.exp2(), false),
        VmOp::FLog2 => (a.max(1.175_494_4e-38).log2(), false),
        VmOp::FPow => (a.max(1.175_494_4e-38).powf(b), false),
        VmOp::FAtan2 => (fxc_atan2_scalar(a, b), false),
        VmOp::FHermite => ((a * a) * (3.0 - (a + a)), false),
        VmOp::LinearToSrgb => (linear_to_srgb_channel((a * b) * c), false),
        VmOp::FSin => (a.sin(), false),
        VmOp::FCos => (a.cos(), false),
        _ => return None,
    };
    Some((res, sat))
}

#[inline]
fn fb_clean(v: f32) -> f32 {
    if v.is_nan() { 0.0 } else { v }
}

fn exec_preamble_inst(
    regs: &mut [u32; REG_FILE_SIZE],
    inst: &VmInst,
    cb2: &[[f32; 4]],
    icb: &[[u32; 4]],
) {
    assert_eq!(regs.len(), REG_FILE_SIZE);
    let ua = regs[inst.src0 as usize];
    let ub = regs[inst.src1 as usize];
    let uc = regs[inst.src2 as usize];
    let fa = f32::from_bits(ua);
    let fb = f32::from_bits(ub);
    let fc = f32::from_bits(uc);

    if let Some((f_out, sat)) = eval_scalar_float_op(inst.op, fa, fb, fc) {
        regs[inst.dst as usize] = sanitize_f32_scalar(f_out, sat);
        return;
    }

    let out: u32 = match inst.op {
        VmOp::Mov => ua,
        VmOp::FNeg => ua ^ 0x8000_0000,
        VmOp::FAbs => ua & 0x7FFF_FFFF,
        VmOp::FNegAbs => (ua & 0x7FFF_FFFF) | 0x8000_0000,
        VmOp::Movc => {
            if ua != 0 {
                ub
            } else {
                uc
            }
        }
        VmOp::FCmpEq => {
            if fa == fb {
                u32::MAX
            } else {
                0
            }
        }
        VmOp::FCmpNe => {
            if fa != fb {
                u32::MAX
            } else {
                0
            }
        }
        VmOp::FCmpLt => {
            if fa < fb {
                u32::MAX
            } else {
                0
            }
        }
        VmOp::FCmpGe => {
            if fa >= fb {
                u32::MAX
            } else {
                0
            }
        }
        VmOp::ICmpEq => {
            if ua == ub {
                u32::MAX
            } else {
                0
            }
        }
        VmOp::ICmpNe => {
            if ua != ub {
                u32::MAX
            } else {
                0
            }
        }
        VmOp::ICmpLt => {
            if (ua as i32) < (ub as i32) {
                u32::MAX
            } else {
                0
            }
        }
        VmOp::ICmpGe => {
            if (ua as i32) >= (ub as i32) {
                u32::MAX
            } else {
                0
            }
        }
        VmOp::UCmpLt => {
            if ua < ub {
                u32::MAX
            } else {
                0
            }
        }
        VmOp::UCmpGe => {
            if ua >= ub {
                u32::MAX
            } else {
                0
            }
        }
        VmOp::BitAnd => ua & ub,
        VmOp::BitOr => ua | ub,
        VmOp::BitXor => ua ^ ub,
        VmOp::BitNot => !ua,
        VmOp::IAdd => (ua as i32).wrapping_add(ub as i32) as u32,
        VmOp::INeg => (ua as i32).wrapping_neg() as u32,
        VmOp::IAbs => (ua as i32).wrapping_abs() as u32,
        VmOp::IMulLo => ua.wrapping_mul(ub),
        VmOp::IMad => ua.wrapping_mul(ub).wrapping_add(uc),
        VmOp::IMin => (ua as i32).min(ub as i32) as u32,
        VmOp::IMax => (ua as i32).max(ub as i32) as u32,
        VmOp::UMin => ua.min(ub),
        VmOp::UMax => ua.max(ub),
        VmOp::IShl => ua.wrapping_shl(ub & 31),
        VmOp::IShr => ((ua as i32).wrapping_shr(ub & 31)) as u32,
        VmOp::UShr => ua.wrapping_shr(ub & 31),
        VmOp::FToI => (fa as i32) as u32,
        VmOp::FToU => fa.max(0.0) as u32,
        VmOp::IToF => ((ua as i32) as f32).to_bits(),
        VmOp::UToF => (ua as f32).to_bits(),
        VmOp::F32ToF16 => f32_to_f16_bits(fa),
        VmOp::F16ToF32 => f16_to_f32_bits(ua),
        VmOp::LoadCb2Dyn => {
            let idx = (ua as usize).wrapping_add(inst.imm16 as usize);
            cb2.get(idx)
                .map_or(0, |v| v[(inst.src1 & 3) as usize].to_bits())
        }
        VmOp::LoadIcbDyn => {
            let idx = (ua as usize).wrapping_add(inst.imm16 as usize);
            icb.get(idx).map_or(0, |v| v[(inst.src1 & 3) as usize])
        }
        _ => 0,
    };
    regs[inst.dst as usize] = out;
}

pub(super) fn populate_static_uniform_regs(
    shader: &mut CompiledSm5Shader,
    spec: ShaderSpecialization<'_>,
) {
    assert!(shader.num_uniform_regs <= REG_FILE_SIZE);
    let mut scalar_regs = [0u32; REG_FILE_SIZE];
    let one_bits = 1.0f32.to_bits();
    scalar_regs[REG_V0_W as usize] = one_bits;
    scalar_regs[REG_CB0_1_X as usize] = one_bits;
    scalar_regs[REG_CB0_1_W as usize] = one_bits;
    scalar_regs[REG_ONE as usize] = one_bits;
    scalar_regs[REG_TRUE as usize] = u32::MAX;
    if let Some(v1) = spec.static_v1 {
        scalar_regs[REG_V1_R as usize] = v1[0].to_bits();
        scalar_regs[REG_V1_G as usize] = v1[1].to_bits();
        scalar_regs[REG_V1_B as usize] = v1[2].to_bits();
        scalar_regs[REG_V1_A as usize] = v1[3].to_bits();
    }
    for b in &shader.const_bindings {
        scalar_regs[b.dst_reg as usize] = b.bits;
    }
    if let Some(cb2) = spec.static_cb2 {
        for b in &shader.cb2_bindings {
            let v = cb2
                .get(b.vec_idx as usize)
                .map_or(0.0, |vec4| vec4[(b.comp & 3) as usize]);
            scalar_regs[b.dst_reg as usize] = v.to_bits();
        }
        let v1_read_in_pre = shader.preamble.iter().any(|inst| {
            matches!(inst.src0, REG_V1_R..=REG_V1_A)
                || matches!(inst.src1, REG_V1_R..=REG_V1_A)
                || matches!(inst.src2, REG_V1_R..=REG_V1_A)
        });
        if spec.static_v1.is_some() || !v1_read_in_pre {
            for inst in &shader.preamble {
                exec_preamble_inst(&mut scalar_regs, inst, cb2, &shader.icb_vec4s);
            }
            shader.has_static_preamble = true;
        }
    }
    let splat_limit = shader.num_uniform_regs.clamp(41, REG_FILE_SIZE);
    assert!(splat_limit <= REG_FILE_SIZE);
    shader.static_uniform_regs = scalar_regs[..splat_limit].to_vec();
}

#[inline(always)]
fn init_tile_uniform_regs(
    shader: &CompiledSm5Shader,
    tile_regs: &mut [u32x8; REG_FILE_SIZE],
    vertex_color_linear: [f32; 4],
    cb2: &[[f32; 4]],
    real_time: f32,
) {
    assert!(shader.num_regs_used <= REG_FILE_SIZE);
    assert!(vertex_color_linear[0].is_finite());
    if shader.has_static_preamble && real_time == 0.0 {
        for (dst, &bits) in tile_regs.iter_mut().zip(&shader.static_uniform_regs) {
            *dst = u32x8::splat(bits);
        }
        tile_regs[REG_V1_R as usize] = f32x8::splat(vertex_color_linear[0]).to_bits();
        tile_regs[REG_V1_G as usize] = f32x8::splat(vertex_color_linear[1]).to_bits();
        tile_regs[REG_V1_B as usize] = f32x8::splat(vertex_color_linear[2]).to_bits();
        tile_regs[REG_V1_A as usize] = f32x8::splat(vertex_color_linear[3]).to_bits();
        return;
    }
    eval_shader_preamble_into(shader, tile_regs, vertex_color_linear, cb2, real_time);
}

fn eval_shader_preamble_into(
    shader: &CompiledSm5Shader,
    out_simd: &mut [u32x8; REG_FILE_SIZE],
    vertex_color_linear: [f32; 4],
    cb2: &[[f32; 4]],
    real_time: f32,
) {
    assert!(shader.num_regs_used <= REG_FILE_SIZE);
    assert!(vertex_color_linear[0].is_finite());
    let mut scalar_regs = [0u32; REG_FILE_SIZE];
    let one_bits = 1.0f32.to_bits();

    scalar_regs[REG_V0_W as usize] = one_bits;
    scalar_regs[REG_V1_R as usize] = vertex_color_linear[0].to_bits();
    scalar_regs[REG_V1_G as usize] = vertex_color_linear[1].to_bits();
    scalar_regs[REG_V1_B as usize] = vertex_color_linear[2].to_bits();
    scalar_regs[REG_V1_A as usize] = vertex_color_linear[3].to_bits();
    scalar_regs[REG_CB0_1_X as usize] = one_bits;
    scalar_regs[REG_CB0_1_W as usize] = one_bits;
    scalar_regs[REG_REAL_TIME as usize] = real_time.to_bits();
    scalar_regs[REG_ONE as usize] = one_bits;
    scalar_regs[REG_TRUE as usize] = u32::MAX;

    for b in &shader.const_bindings {
        scalar_regs[b.dst_reg as usize] = b.bits;
    }
    for b in &shader.cb2_bindings {
        let v = cb2
            .get(b.vec_idx as usize)
            .map_or(0.0, |vec4| vec4[(b.comp & 3) as usize]);
        scalar_regs[b.dst_reg as usize] = v.to_bits();
    }
    for inst in &shader.preamble {
        exec_preamble_inst(&mut scalar_regs, inst, cb2, &shader.icb_vec4s);
    }

    let splat_limit = shader.num_uniform_regs.clamp(41, REG_FILE_SIZE);
    for (dst, &bits) in out_simd[..splat_limit]
        .iter_mut()
        .zip(&scalar_regs[..splat_limit])
    {
        *dst = u32x8::splat(bits);
    }
}

/// Evaluates Stage 1 (`shader.preamble`) once per quad on scalar registers and splats the
/// initialized state into an 8-wide SIMD register template (`[u32x8; 256]`).
pub fn eval_shader_preamble(
    shader: &CompiledSm5Shader,
    vertex_color_linear: [f32; 4],
    cb2: &[[f32; 4]],
    real_time: f32,
) -> [u32x8; REG_FILE_SIZE] {
    assert!(shader.num_regs_used <= REG_FILE_SIZE);
    assert!(vertex_color_linear[0].is_finite());
    let mut base_simd = [u32x8::splat(0); REG_FILE_SIZE];
    eval_shader_preamble_into(shader, &mut base_simd, vertex_color_linear, cb2, real_time);
    base_simd
}

#[inline(always)]
fn eval_simd_alu_value(regs: &[u32x8; REG_FILE_SIZE], inst: &VmInst) -> u32x8 {
    assert_eq!(SIMD_LANES, 8);
    let ua = regs[inst.src0 as usize];
    let ub = regs[inst.src1 as usize];
    let uc = regs[inst.src2 as usize];
    let a = f32x8::from_bits(ua);
    let b = f32x8::from_bits(ub);
    let c = f32x8::from_bits(uc);
    let zero = f32x8::splat(0.0);
    let one = f32x8::splat(1.0);

    match inst.op {
        VmOp::FAdd => (a + b).to_bits(),
        VmOp::FSub => (a - b).to_bits(),
        VmOp::FMul => (a * b).to_bits(),
        VmOp::FMul3 => ((a * b) * c).to_bits(),
        VmOp::FMad => (a * b + c).to_bits(),
        VmOp::FNmad => (c - a * b).to_bits(),
        VmOp::Mov => ua,
        VmOp::MovSat => a.simd_max(zero).simd_min(one).to_bits(),
        VmOp::FNeg => ua ^ u32x8::splat(0x8000_0000),
        VmOp::FAbs => ua & u32x8::splat(0x7FFF_FFFF),
        VmOp::FMin => a.simd_min(b).to_bits(),
        VmOp::FMax => a.simd_max(b).to_bits(),
        VmOp::FFrc => (a - a.floor()).to_bits(),
        VmOp::FSqrt => a.simd_max(zero).sqrt().to_bits(),
        VmOp::FPow => fast_pow_x8(a, b).to_bits(),
        VmOp::FAtan2 => fast_atan2_x8(a, b).to_bits(),
        VmOp::FHermite => ((a * a) * (f32x8::splat(3.0) - (a + a))).to_bits(),
        VmOp::LinearToSrgb => linear_to_srgb_x8((a * b) * c).to_bits(),
        _ => eval_simd_cold_float_value(regs, inst, ua, ub, uc),
    }
}

#[cold]
#[inline(never)]
fn eval_simd_cold_float_value(
    regs: &[u32x8; REG_FILE_SIZE],
    inst: &VmInst,
    ua: u32x8,
    ub: u32x8,
    uc: u32x8,
) -> u32x8 {
    assert_eq!(SIMD_LANES, 8);
    let a = f32x8::from_bits(ua);
    let b = f32x8::from_bits(ub);
    let zero = f32x8::splat(0.0);
    let one = f32x8::splat(1.0);
    let eps = f32x8::splat(1e-20);

    match inst.op {
        VmOp::FNegAbs => (ua & u32x8::splat(0x7FFF_FFFF)) | u32x8::splat(0x8000_0000),
        VmOp::Movc => ua.simd_ne(u32x8::splat(0)).select(ub, uc),
        VmOp::FDiv => b.abs().simd_gt(eps).select(a / b, zero).to_bits(),
        VmOp::FRoundNe => a.round().to_bits(),
        VmOp::FFloor => a.floor().to_bits(),
        VmOp::FCeil => a.ceil().to_bits(),
        VmOp::FTrunc => a.trunc().to_bits(),
        VmOp::FRsq => a
            .simd_gt(eps)
            .select(one / a.simd_max(eps).sqrt(), zero)
            .to_bits(),
        VmOp::FRcp => a.abs().simd_gt(eps).select(one / a, zero).to_bits(),
        VmOp::FExp2 => fast_exp2_x8(a).to_bits(),
        VmOp::FLog2 => fast_log2_x8(a).to_bits(),
        VmOp::FSin => fast_sin_x8(a).to_bits(),
        VmOp::FCos => fast_cos_x8(a).to_bits(),
        VmOp::FDdx => ddx_quad_x8(a).to_bits(),
        VmOp::FDdy => ddy_quad_x8(a).to_bits(),
        VmOp::FCmpEq => a.simd_eq(b).select(u32x8::splat(u32::MAX), u32x8::splat(0)),
        VmOp::FCmpNe => a.simd_ne(b).select(u32x8::splat(u32::MAX), u32x8::splat(0)),
        VmOp::FCmpLt => a.simd_lt(b).select(u32x8::splat(u32::MAX), u32x8::splat(0)),
        VmOp::FCmpGe => a.simd_ge(b).select(u32x8::splat(u32::MAX), u32x8::splat(0)),
        VmOp::BitAnd => ua & ub,
        VmOp::BitOr => ua | ub,
        VmOp::BitXor => ua ^ ub,
        VmOp::BitNot => !ua,
        _ => eval_simd_int_value(regs, inst, ua, ub, uc),
    }
}

#[cold]
#[inline(never)]
fn eval_simd_int_value(
    regs: &[u32x8; REG_FILE_SIZE],
    inst: &VmInst,
    ua: u32x8,
    ub: u32x8,
    uc: u32x8,
) -> u32x8 {
    assert_eq!(SIMD_LANES, 8);
    let ia = ua.cast::<i32>();
    let ib = ub.cast::<i32>();
    let fa = f32x8::from_bits(ua);

    match inst.op {
        VmOp::ICmpEq => ua
            .simd_eq(ub)
            .select(u32x8::splat(u32::MAX), u32x8::splat(0)),
        VmOp::ICmpNe => ua
            .simd_ne(ub)
            .select(u32x8::splat(u32::MAX), u32x8::splat(0)),
        VmOp::ICmpLt => ia
            .simd_lt(ib)
            .select(u32x8::splat(u32::MAX), u32x8::splat(0)),
        VmOp::ICmpGe => ia
            .simd_ge(ib)
            .select(u32x8::splat(u32::MAX), u32x8::splat(0)),
        VmOp::UCmpLt => ua
            .simd_lt(ub)
            .select(u32x8::splat(u32::MAX), u32x8::splat(0)),
        VmOp::UCmpGe => ua
            .simd_ge(ub)
            .select(u32x8::splat(u32::MAX), u32x8::splat(0)),
        VmOp::IAdd => (ia + ib).cast::<u32>(),
        VmOp::INeg => (-ia).cast::<u32>(),
        VmOp::IAbs => ia.abs().cast::<u32>(),
        VmOp::IMulLo => ua * ub,
        VmOp::IMad => ua * ub + uc,
        VmOp::IMin => ia.simd_min(ib).cast::<u32>(),
        VmOp::IMax => ia.simd_max(ib).cast::<u32>(),
        VmOp::UMin => ua.simd_min(ub),
        VmOp::UMax => ua.simd_max(ub),
        VmOp::IShl => ua << (ub & u32x8::splat(31)),
        VmOp::IShr => (ia >> (ib & i32x8::splat(31))).cast::<u32>(),
        VmOp::UShr => ua >> (ub & u32x8::splat(31)),
        VmOp::FToI => fa.cast::<i32>().cast::<u32>(),
        VmOp::FToU => fa.simd_max(f32x8::splat(0.0)).cast::<u32>(),
        VmOp::IToF => ia.cast::<f32>().to_bits(),
        VmOp::UToF => ua.cast::<f32>().to_bits(),
        _ => eval_simd_special_int(regs, inst, ua, ub, uc),
    }
}

#[cold]
fn eval_simd_special_int(
    regs: &[u32x8; REG_FILE_SIZE],
    inst: &VmInst,
    ua: u32x8,
    ub: u32x8,
    uc: u32x8,
) -> u32x8 {
    assert_eq!(SIMD_LANES, 8);
    let a_arr = ua.to_array();
    let b_arr = ub.to_array();
    let c_arr = uc.to_array();
    let mut out = [0u32; SIMD_LANES];

    for i in 0..SIMD_LANES {
        let a = a_arr[i];
        let b = b_arr[i];
        let c = c_arr[i];
        out[i] = match inst.op {
            VmOp::IMulHi => (((a as i32 as i64) * (b as i32 as i64)) >> 32) as u32,
            VmOp::UMulHi => (((a as u64) * (b as u64)) >> 32) as u32,
            VmOp::UDivQuot => a.checked_div(b).unwrap_or(u32::MAX),
            VmOp::UDivRem => a.checked_rem(b).unwrap_or(u32::MAX),
            VmOp::UAddCarry => u32::from(a.overflowing_add(b).1),
            VmOp::USubBorrow => u32::from(a >= b),
            VmOp::F32ToF16 => f32_to_f16_bits(f32::from_bits(a)),
            VmOp::F16ToF32 => f16_to_f32_bits(a),
            VmOp::CountBits => a.count_ones(),
            VmOp::FirstBitHi => {
                if a == 0 {
                    u32::MAX
                } else {
                    31 - a.leading_zeros()
                }
            }
            VmOp::FirstBitLo => {
                if a == 0 {
                    u32::MAX
                } else {
                    a.trailing_zeros()
                }
            }
            VmOp::FirstBitShi => {
                let inv = if (a as i32) < 0 { !a } else { a };
                if inv == 0 {
                    u32::MAX
                } else {
                    31 - inv.leading_zeros()
                }
            }
            VmOp::Bfrev => a.reverse_bits(),
            VmOp::Ubfe => {
                let width = a & 31;
                let offset = b & 31;
                if width == 0 {
                    0
                } else if width + offset < 32 {
                    (c << (32 - width - offset)) >> (32 - width)
                } else {
                    c >> offset
                }
            }
            VmOp::Ibfe => {
                let width = a & 31;
                let offset = b & 31;
                if width == 0 {
                    0
                } else if width + offset < 32 {
                    (((c as i32) << (32 - width - offset)) >> (32 - width)) as u32
                } else {
                    ((c as i32) >> offset) as u32
                }
            }
            VmOp::Bfi => {
                let width = a & 31;
                let offset = b & 31;
                let bitmask = (((1u64 << width) - 1) << offset) as u32;
                let insert = regs[inst.aux as usize].to_array()[i];
                ((c << offset) & bitmask) | (insert & !bitmask)
            }
            _ => 0,
        };
    }
    u32x8::from_array(out)
}

#[inline(always)]
fn exec_sample2d_unmasked(
    regs: &mut [u32x8; REG_FILE_SIZE],
    inst: &VmInst,
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    row_y_cache: &[(i32x8, f32x8); 4],
) {
    assert_eq!(SIMD_LANES, 8);
    let u = f32x8::from_bits(regs[inst.src0 as usize]);
    let v = f32x8::from_bits(regs[inst.src1 as usize]);
    let t_idx = inst.src2 as usize;
    let (cu, cv) = tex_clamps.get(t_idx).copied().unwrap_or((false, false));
    let swz_bits = inst.imm16 & 0x01FF;
    let dst_mask = inst.aux & 0x0F;
    let zero = f32x8::splat(0.0);
    let one = f32x8::splat(1.0);

    // Fast path for canonical identity swizzle `(0, 1, 2, 3)` without saturate (`228`).
    if swz_bits == 228 {
        let ry_slot = (inst.imm16 >> 9) as usize;
        let sampled = textures.get(t_idx).map_or([zero, zero, zero, one], |tex| {
            if ry_slot > 0 {
                let (iy0_s, ty) = row_y_cache[(ry_slot - 1) & 3];
                tex.sample_bilinear_with_y_x8(u, iy0_s, ty, cu, inst.aux)
            } else {
                tex.sample_bilinear_x8(u, v, cu, cv, inst.aux)
            }
        });
        let d = inst.dst as usize;
        if dst_mask & 1 != 0 {
            regs[d] = sampled[0].to_bits();
        }
        if dst_mask & 2 != 0 {
            regs[d + 1] = sampled[1].to_bits();
        }
        if dst_mask & 4 != 0 {
            regs[d + 2] = sampled[2].to_bits();
        }
        if dst_mask & 8 != 0 {
            regs[d + 3] = sampled[3].to_bits();
        }
        return;
    }

    let sat = (swz_bits & (1 << 8)) != 0;
    let mut raw_needed = 0u8;
    for c in 0..4u8 {
        if (dst_mask >> c) & 1 != 0 {
            let src_c = ((swz_bits >> (c * 2)) & 3) as u8;
            raw_needed |= 1 << src_c;
        }
    }
    let sampled = textures.get(t_idx).map_or([zero, zero, zero, one], |tex| {
        tex.sample_bilinear_x8(u, v, cu, cv, raw_needed)
    });
    for c in 0..4u8 {
        if (dst_mask >> c) & 1 != 0 {
            let src_c = ((swz_bits >> (c * 2)) & 3) as usize;
            let val = if sat {
                sampled[src_c].simd_max(zero).simd_min(one)
            } else {
                sampled[src_c]
            };
            regs[inst.dst.wrapping_add(c) as usize] = val.to_bits();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn exec_simd_tex_or_mem(
    regs: &mut [u32x8; REG_FILE_SIZE],
    inst: &VmInst,
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    cb2: &[[f32; 4]],
    icb: &[[u32; 4]],
    idx_temps: &mut [[[u32; 4]; MAX_INDEXABLE_TEMP_LEN]; SIMD_LANES],
    exec_mask: u8,
) {
    assert_eq!(SIMD_LANES, 8);
    let zero = f32x8::splat(0.0);
    let one = f32x8::splat(1.0);
    match inst.op {
        VmOp::Sample2D | VmOp::Gather4 if exec_mask == 0xFF => {
            let mut no_row_y = *inst;
            no_row_y.imm16 &= 0x01FF;
            let dummy_y = [(i32x8::splat(0), zero); 4];
            exec_sample2d_unmasked(regs, &no_row_y, textures, tex_clamps, &dummy_y);
        }
        VmOp::Sample2D | VmOp::Gather4 => {
            let u = f32x8::from_bits(regs[inst.src0 as usize]);
            let v = f32x8::from_bits(regs[inst.src1 as usize]);
            let t_idx = inst.src2 as usize;
            let (cu, cv) = tex_clamps.get(t_idx).copied().unwrap_or((false, false));
            let swz_bits = inst.imm16 & 0x01FF;
            let sat = (swz_bits & (1 << 8)) != 0;
            let dst_mask = inst.aux & 0x0F;
            let mut raw_needed = 0u8;
            for c in 0..4u8 {
                if (dst_mask >> c) & 1 != 0 {
                    raw_needed |= 1 << (((swz_bits >> (c * 2)) & 3) as u8);
                }
            }
            let sampled = textures.get(t_idx).map_or([zero, zero, zero, one], |tex| {
                tex.sample_bilinear_x8(u, v, cu, cv, raw_needed)
            });
            for c in 0..4u8 {
                if (dst_mask >> c) & 1 != 0 {
                    let src_c = ((swz_bits >> (c * 2)) & 3) as usize;
                    let val = if sat {
                        sampled[src_c].simd_max(zero).simd_min(one)
                    } else {
                        sampled[src_c]
                    };
                    write_u32x8(regs, inst.dst.wrapping_add(c), val.to_bits(), exec_mask);
                }
            }
        }
        VmOp::TexLoad2D => {
            let x_arr = regs[inst.src0 as usize].cast::<i32>().to_array();
            let y_arr = regs[inst.src1 as usize].cast::<i32>().to_array();
            let t_idx = inst.src2 as usize;
            let swz_bits = inst.imm16 & 0x01FF;
            let sat = (swz_bits & (1 << 8)) != 0;
            let mut ch = [[0.0f32; SIMD_LANES]; 4];
            for lane in 0..SIMD_LANES {
                let px = textures.get(t_idx).map_or([0.0, 0.0, 0.0, 1.0], |tex| {
                    tex.load_texel_clamped(x_arr[lane], y_arr[lane])
                });
                for c in 0..4 {
                    ch[c][lane] = px[c];
                }
            }
            for c in 0..4u8 {
                if (inst.aux >> c) & 1 != 0 {
                    let src_c = ((swz_bits >> (c * 2)) & 3) as usize;
                    let v_f = f32x8::from_array(ch[src_c]);
                    let val = if sat {
                        v_f.simd_max(zero).simd_min(one)
                    } else {
                        v_f
                    };
                    write_u32x8(regs, inst.dst.wrapping_add(c), val.to_bits(), exec_mask);
                }
            }
        }
        VmOp::ResInfo2D => {
            let t_idx = inst.src2 as usize;
            let (w, h) = textures
                .get(t_idx)
                .map_or((1.0f32, 1.0f32), |t| (t.width as f32, t.height as f32));
            let vals = [w, h, 1.0, 1.0];
            for c in 0..4u8 {
                if (inst.aux >> c) & 1 != 0 {
                    let src_c = ((inst.imm16 >> (c * 2)) & 3) as usize;
                    write_u32x8(
                        regs,
                        inst.dst.wrapping_add(c),
                        f32x8::splat(vals[src_c]).to_bits(),
                        exec_mask,
                    );
                }
            }
        }
        VmOp::LoadCb2Dyn => {
            let rel = regs[inst.src0 as usize].to_array();
            let comp = (inst.src1 & 3) as usize;
            let mut out = [0u32; SIMD_LANES];
            for lane in 0..SIMD_LANES {
                let idx = (rel[lane] as usize).wrapping_add(inst.imm16 as usize);
                out[lane] = cb2.get(idx).map_or(0, |v| v[comp].to_bits());
            }
            write_u32x8(regs, inst.dst, u32x8::from_array(out), exec_mask);
        }
        VmOp::LoadIcbDyn => {
            let rel = regs[inst.src0 as usize].to_array();
            let comp = (inst.src1 & 3) as usize;
            let mut out = [0u32; SIMD_LANES];
            for lane in 0..SIMD_LANES {
                let idx = (rel[lane] as usize).wrapping_add(inst.imm16 as usize);
                out[lane] = icb.get(idx).map_or(0, |v| v[comp]);
            }
            write_u32x8(regs, inst.dst, u32x8::from_array(out), exec_mask);
        }
        VmOp::LoadIdxTemp => {
            let rel = regs[inst.src0 as usize].to_array();
            let comp = (inst.src2 & 3) as usize;
            let mut out = [0u32; SIMD_LANES];
            for lane in 0..SIMD_LANES {
                let idx = (rel[lane] as usize)
                    .wrapping_add(inst.imm16 as usize)
                    .min(MAX_INDEXABLE_TEMP_LEN - 1);
                out[lane] = idx_temps[lane][idx][comp];
            }
            write_u32x8(regs, inst.dst, u32x8::from_array(out), exec_mask);
        }
        VmOp::StoreIdxTemp => {
            let rel = regs[inst.src0 as usize].to_array();
            let val = regs[inst.src1 as usize].to_array();
            let comp = (inst.src2 & 3) as usize;
            for lane in 0..SIMD_LANES {
                if (exec_mask >> lane) & 1 != 0 {
                    let idx = (rel[lane] as usize)
                        .wrapping_add(inst.imm16 as usize)
                        .min(MAX_INDEXABLE_TEMP_LEN - 1);
                    idx_temps[lane][idx][comp] = val[lane];
                }
            }
        }
        _ => {}
    }
}

#[inline(always)]
fn set_tile_y_regs(
    shader: &CompiledSm5Shader,
    regs: &mut [u32x8; REG_FILE_SIZE],
    fy: f32x8,
    v: f32x8,
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    row_y_cache: &mut [(i32x8, f32x8); 4],
) {
    assert_eq!(SIMD_LANES, 8);
    assert!(shader.num_row_y_samples <= 4);
    let fy_b = fy.to_bits();
    let v_b = v.to_bits();
    regs[REG_V0_Y as usize] = fy_b;
    regs[REG_V2_Y as usize] = fy_b;
    regs[REG_V3_V as usize] = v_b;
    regs[REG_V4_Y as usize] = v_b;
    regs[REG_V4_W as usize] = v_b;

    for inst in &shader.row_preamble {
        regs[inst.dst as usize] = eval_simd_alu_value(regs, inst);
    }
    for (slot_out, &(t_slot, v_reg)) in row_y_cache
        .iter_mut()
        .zip(&shader.row_y_samples[..(shader.num_row_y_samples as usize)])
    {
        if let Some(tex) = textures.get(t_slot as usize) {
            let (_, cv) = tex_clamps
                .get(t_slot as usize)
                .copied()
                .unwrap_or((false, false));
            let v_vec = f32x8::from_bits(regs[v_reg as usize]);
            *slot_out = tex.precompute_row_y_x8(v_vec, cv);
        }
    }
}

#[inline(always)]
fn apply_srgb_output_mask(rgba: &mut [f32x8; 4], mask: u8) {
    assert!(mask < 8);
    assert_eq!(SIMD_LANES, 8);
    if mask == 0 {
        return;
    }
    let srgb = linear_to_srgb_rgb_x8([rgba[0], rgba[1], rgba[2]]);
    if mask & 1 != 0 {
        rgba[0] = srgb[0];
    }
    if mask & 2 != 0 {
        rgba[1] = srgb[1];
    }
    if mask & 4 != 0 {
        rgba[2] = srgb[2];
    }
}

/// Branchless straight-line fast path for `shader.kernel` when `!shader.has_control_flow` and
/// `!shader.uses_indexable_temps`. Uniform registers (`0..shader.num_uniform_regs`) and row-varying
/// Y registers (`0..shader.num_row_regs`) are never modified by `kernel`, so `regs` needs
/// **zero `memcpy` per tile** and only 5 X register writes.
/// Returns `None` when an `EarlyOutZero` instruction short-circuits the entire 8-pixel tile.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn exec_tile_branchless(
    shader: &CompiledSm5Shader,
    regs: &mut [u32x8; REG_FILE_SIZE],
    fx: f32x8,
    u: f32x8,
    cb2: &[[f32; 4]],
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    row_y_cache: &[(i32x8, f32x8); 4],
) -> Option<[f32x8; 4]> {
    assert!(!shader.has_control_flow);
    assert!(!shader.uses_indexable_temps);

    let fx_b = fx.to_bits();
    let u_b = u.to_bits();
    regs[REG_V0_X as usize] = fx_b;
    regs[REG_V2_X as usize] = fx_b;
    regs[REG_V3_U as usize] = u_b;
    regs[REG_V4_X as usize] = u_b;
    regs[REG_V4_Z as usize] = u_b;

    for inst in &shader.kernel {
        match inst.op {
            VmOp::Ret => break,
            VmOp::EarlyOutZero => {
                if regs[inst.src0 as usize] == u32x8::splat(0) {
                    return None;
                }
            }
            VmOp::Sample2D | VmOp::Gather4 => {
                exec_sample2d_unmasked(regs, inst, textures, tex_clamps, row_y_cache);
            }
            VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::LoadCb2Dyn | VmOp::LoadIcbDyn => {
                exec_tex_or_mem_cold(regs, inst, textures, tex_clamps, cb2, &shader.icb_vec4s);
            }
            _ => {
                regs[inst.dst as usize] = eval_simd_alu_value(regs, inst);
            }
        }
    }

    Some([
        f32x8::from_bits(regs[REG_OUT_R as usize]),
        f32x8::from_bits(regs[REG_OUT_G as usize]),
        f32x8::from_bits(regs[REG_OUT_B as usize]),
        f32x8::from_bits(regs[REG_OUT_A as usize]),
    ])
}

#[cold]
fn exec_tex_or_mem_cold(
    regs: &mut [u32x8; REG_FILE_SIZE],
    inst: &VmInst,
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    cb2: &[[f32; 4]],
    icb: &[[u32; 4]],
) {
    let mut dummy_idx = [[[0u32; 4]; MAX_INDEXABLE_TEMP_LEN]; SIMD_LANES];
    exec_simd_tex_or_mem(
        regs,
        inst,
        textures,
        tex_clamps,
        cb2,
        icb,
        &mut dummy_idx,
        0xFF,
    );
}

#[inline]
fn eval_cond_mask(regs: &[u32x8; REG_FILE_SIZE], reg: u8, test_nonzero: bool, active: u8) -> u8 {
    assert_eq!(SIMD_LANES, 8);
    let v = regs[reg as usize];
    let raw_mask = if test_nonzero {
        v.simd_ne(u32x8::splat(0)).to_bitmask() as u8
    } else {
        v.simd_eq(u32x8::splat(0)).to_bitmask() as u8
    };
    raw_mask & active
}

/// Executes Stage 2 (`shader.kernel`) for an 8-pixel (`4x2` dual-quad) tile.
/// Returns `(alive_mask, [out_r, out_g, out_b, out_a])`.
#[allow(clippy::too_many_arguments)]
pub fn execute_tile_x8(
    shader: &CompiledSm5Shader,
    base_simd: &[u32x8; REG_FILE_SIZE],
    fx: f32x8,
    fy: f32x8,
    u: f32x8,
    v: f32x8,
    valid_lanes: u8,
    cb2: &[[f32; 4]],
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
) -> (u8, [f32x8; 4]) {
    let mut regs = *base_simd;
    if !shader.has_control_flow && !shader.uses_indexable_temps {
        let mut row_y_cache = [(i32x8::splat(0), f32x8::splat(0.0)); 4];
        set_tile_y_regs(
            shader,
            &mut regs,
            fy,
            v,
            textures,
            tex_clamps,
            &mut row_y_cache,
        );
        let mut rgba = exec_tile_branchless(
            shader,
            &mut regs,
            fx,
            u,
            cb2,
            textures,
            tex_clamps,
            &row_y_cache,
        )
        .unwrap_or([f32x8::splat(0.0); 4]);
        apply_srgb_output_mask(&mut rgba, shader.srgb_encode_mask);
        return (valid_lanes, rgba);
    }
    let mut idx_temps = [[[0u32; 4]; MAX_INDEXABLE_TEMP_LEN]; SIMD_LANES];
    execute_tile_x8_with_scratch(
        shader,
        base_simd,
        &mut regs,
        &mut idx_temps,
        fx,
        fy,
        u,
        v,
        valid_lanes,
        cb2,
        textures,
        tex_clamps,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_tile_x8_with_scratch(
    shader: &CompiledSm5Shader,
    base_simd: &[u32x8; REG_FILE_SIZE],
    regs: &mut [u32x8; REG_FILE_SIZE],
    idx_temps: &mut [[[u32; 4]; MAX_INDEXABLE_TEMP_LEN]; SIMD_LANES],
    fx: f32x8,
    fy: f32x8,
    u: f32x8,
    v: f32x8,
    valid_lanes: u8,
    cb2: &[[f32; 4]],
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
) -> (u8, [f32x8; 4]) {
    assert!(shader.num_regs_used <= REG_FILE_SIZE);
    if !shader.has_control_flow && !shader.uses_indexable_temps {
        let mut row_y_cache = [(i32x8::splat(0), f32x8::splat(0.0)); 4];
        set_tile_y_regs(shader, regs, fy, v, textures, tex_clamps, &mut row_y_cache);
        let mut rgba =
            exec_tile_branchless(shader, regs, fx, u, cb2, textures, tex_clamps, &row_y_cache)
                .unwrap_or([f32x8::splat(0.0); 4]);
        apply_srgb_output_mask(&mut rgba, shader.srgb_encode_mask);
        return (valid_lanes, rgba);
    }

    regs[..shader.num_regs_used].copy_from_slice(&base_simd[..shader.num_regs_used]);
    let fx_b = fx.to_bits();
    let fy_b = fy.to_bits();
    let u_b = u.to_bits();
    let v_b = v.to_bits();
    regs[REG_V0_X as usize] = fx_b;
    regs[REG_V0_Y as usize] = fy_b;
    regs[REG_V2_X as usize] = fx_b;
    regs[REG_V2_Y as usize] = fy_b;
    regs[REG_V3_U as usize] = u_b;
    regs[REG_V3_V as usize] = v_b;
    regs[REG_V4_X as usize] = u_b;
    regs[REG_V4_Y as usize] = v_b;
    regs[REG_V4_Z as usize] = u_b;
    regs[REG_V4_W as usize] = v_b;
    for inst in &shader.row_preamble {
        let val = eval_simd_alu_value(regs, inst);
        regs[inst.dst as usize] = val;
    }

    if shader.uses_indexable_temps {
        idx_temps.fill([[0u32; 4]; MAX_INDEXABLE_TEMP_LEN]);
    }
    let mut alive_mask = valid_lanes;
    let mut active_mask: u8 = 0xFF;
    let mut exec_mask: u8 = 0xFF;
    let mut cf_stack = [(0u8, 0u8); MAX_CF_DEPTH];
    let mut cf_sp = 0usize;

    let insts = &shader.kernel;
    let mut pc = 0usize;

    for _ in 0..MAX_EXEC_STEPS {
        if pc >= insts.len() || alive_mask == 0 || active_mask == 0 {
            break;
        }
        let inst = &insts[pc];
        pc += 1;

        match inst.op {
            VmOp::IfZ | VmOp::IfNz => {
                let cond = eval_cond_mask(regs, inst.src0, inst.op == VmOp::IfNz, exec_mask);
                if cf_sp < MAX_CF_DEPTH {
                    cf_stack[cf_sp] = (exec_mask, cond);
                    cf_sp += 1;
                }
                if cond == 0 {
                    exec_mask = 0;
                    pc = inst.imm16 as usize;
                } else {
                    exec_mask = cond;
                }
            }
            VmOp::Else => {
                if cf_sp > 0 {
                    let (parent_mask, then_mask) = cf_stack[cf_sp - 1];
                    let else_mask = parent_mask & !then_mask & active_mask;
                    if else_mask == 0 {
                        exec_mask = parent_mask & active_mask;
                        cf_sp -= 1;
                        pc = inst.imm16 as usize;
                    } else {
                        exec_mask = else_mask;
                    }
                }
            }
            VmOp::EndIf => {
                if cf_sp > 0 {
                    cf_sp -= 1;
                    exec_mask = cf_stack[cf_sp].0 & active_mask;
                }
            }
            VmOp::Loop => {
                if cf_sp < MAX_CF_DEPTH {
                    cf_stack[cf_sp] = (exec_mask, exec_mask);
                    cf_sp += 1;
                }
            }
            VmOp::EndLoop => {
                if cf_sp > 0 {
                    let loop_active = cf_stack[cf_sp - 1].1 & active_mask;
                    if loop_active != 0 {
                        exec_mask = loop_active;
                        pc = inst.imm16 as usize;
                    } else {
                        cf_sp -= 1;
                        exec_mask = cf_stack[cf_sp].0 & active_mask;
                    }
                }
            }
            VmOp::Break | VmOp::BreakZ | VmOp::BreakNz => {
                let brk = if inst.op == VmOp::Break {
                    exec_mask
                } else {
                    eval_cond_mask(regs, inst.src0, inst.op == VmOp::BreakNz, exec_mask)
                };
                let loop_slot = (inst.aux as usize).min(cf_sp.saturating_sub(1));
                if brk != 0 && cf_sp > 0 {
                    cf_stack[loop_slot].1 &= !brk;
                    for entry in &mut cf_stack[(loop_slot + 1)..cf_sp] {
                        entry.0 &= !brk;
                        entry.1 &= !brk;
                    }
                    exec_mask &= !brk;
                    if cf_stack[loop_slot].1 == 0 {
                        cf_sp = loop_slot;
                        exec_mask = cf_stack[loop_slot].0 & active_mask;
                        pc = inst.imm16 as usize;
                    }
                }
            }
            VmOp::Continue | VmOp::ContinueZ | VmOp::ContinueNz => {
                let cnt = if inst.op == VmOp::Continue {
                    exec_mask
                } else {
                    eval_cond_mask(regs, inst.src0, inst.op == VmOp::ContinueNz, exec_mask)
                };
                let loop_slot = (inst.aux as usize).min(cf_sp.saturating_sub(1));
                if cnt != 0 && cf_sp > 0 {
                    for entry in &mut cf_stack[(loop_slot + 1)..cf_sp] {
                        entry.0 &= !cnt;
                        entry.1 &= !cnt;
                    }
                    exec_mask &= !cnt;
                    if exec_mask == 0 && cf_sp == loop_slot + 1 {
                        pc = inst.imm16 as usize;
                    }
                }
            }
            VmOp::DiscardZ | VmOp::DiscardNz => {
                let disc = eval_cond_mask(regs, inst.src0, inst.op == VmOp::DiscardNz, exec_mask);
                alive_mask &= !disc;
                active_mask &= !disc;
                exec_mask &= !disc;
            }
            VmOp::Ret => break,
            VmOp::RetZ | VmOp::RetNz => {
                let ret = eval_cond_mask(regs, inst.src0, inst.op == VmOp::RetNz, exec_mask);
                active_mask &= !ret;
                exec_mask &= !ret;
            }
            VmOp::Sample2D
            | VmOp::Gather4
            | VmOp::TexLoad2D
            | VmOp::ResInfo2D
            | VmOp::LoadCb2Dyn
            | VmOp::LoadIcbDyn
            | VmOp::LoadIdxTemp
            | VmOp::StoreIdxTemp => {
                if exec_mask != 0 {
                    exec_simd_tex_or_mem(
                        regs,
                        inst,
                        textures,
                        tex_clamps,
                        cb2,
                        &shader.icb_vec4s,
                        idx_temps,
                        exec_mask,
                    );
                }
            }
            _ => {
                if exec_mask != 0 {
                    let val = eval_simd_alu_value(regs, inst);
                    write_u32x8(regs, inst.dst, val, exec_mask);
                }
            }
        }
    }

    let mut out_rgba = [
        f32x8::from_bits(regs[REG_OUT_R as usize]),
        f32x8::from_bits(regs[REG_OUT_G as usize]),
        f32x8::from_bits(regs[REG_OUT_B as usize]),
        f32x8::from_bits(regs[REG_OUT_A as usize]),
    ];
    apply_srgb_output_mask(&mut out_rgba, shader.srgb_encode_mask);
    (alive_mask, out_rgba)
}

#[inline(always)]
fn blend_pixel_into(dst: &mut [f32; 4], src: [f32; 4], is_additive: bool) {
    if is_additive {
        let r = src[0].max(0.0);
        let g = src[1].max(0.0);
        let b = src[2].max(0.0);
        dst[0] = (dst[0] + r).min(1.0);
        dst[1] = (dst[1] + g).min(1.0);
        dst[2] = (dst[2] + b).min(1.0);
        let lum_a = r.max(g).max(b).clamp(0.0, 1.0);
        dst[3] = (dst[3] + lum_a * (1.0 - dst[3])).clamp(0.0, 1.0);
    } else {
        let a = src[3].clamp(0.0, 1.0);
        let inv_a = 1.0 - a;
        dst[0] = (src[0].clamp(0.0, 1.0) * a + dst[0] * inv_a).clamp(0.0, 1.0);
        dst[1] = (src[1].clamp(0.0, 1.0) * a + dst[1] * inv_a).clamp(0.0, 1.0);
        dst[2] = (src[2].clamp(0.0, 1.0) * a + dst[2] * inv_a).clamp(0.0, 1.0);
        dst[3] = (a + dst[3] * inv_a).clamp(0.0, 1.0);
    }
}

/// Vectorized 8-wide SIMD (`f32x8`) `4x2` tile blender.
///
/// For interior tiles (`px + 4 <= w_usize && py + 2 <= h_usize`), loads the two 4-pixel canvas
/// rows (`[0, 1, 4, 5]` and `[2, 3, 6, 7]`), blends all 8 pixels simultaneously in `f32x8`, and
/// writes the blended result back branchlessly.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn blend_tile_x8(
    canvas: &mut [[f32; 4]],
    w_usize: usize,
    h_usize: usize,
    px: u32,
    py: u32,
    write_mask: u8,
    out_rgba: [f32x8; 4],
    is_additive: bool,
) {
    assert_eq!(SIMD_LANES, 8);
    assert!(w_usize > 0 && h_usize > 0);
    let px_u = px as usize;
    let py_u = py as usize;
    if px_u + 4 <= w_usize && py_u + 2 <= h_usize {
        let row0 = py_u * w_usize + px_u;
        let row1 = (py_u + 1) * w_usize + px_u;
        let p = [
            canvas[row0],
            canvas[row0 + 1],
            canvas[row1],
            canvas[row1 + 1],
            canvas[row0 + 2],
            canvas[row0 + 3],
            canvas[row1 + 2],
            canvas[row1 + 3],
        ];
        let dr = f32x8::from_array(p.map(|v| v[0]));
        let dg = f32x8::from_array(p.map(|v| v[1]));
        let db = f32x8::from_array(p.map(|v| v[2]));
        let da = f32x8::from_array(p.map(|v| v[3]));
        let zero = f32x8::splat(0.0);
        let one = f32x8::splat(1.0);

        let (mut nr, mut ng, mut nb, mut na) = if is_additive {
            let r = out_rgba[0].simd_max(zero);
            let g = out_rgba[1].simd_max(zero);
            let b = out_rgba[2].simd_max(zero);
            let lum_a = r.simd_max(g).simd_max(b).simd_min(one);
            (
                (dr + r).simd_min(one),
                (dg + g).simd_min(one),
                (db + b).simd_min(one),
                (da + lum_a * (one - da)).simd_max(zero).simd_min(one),
            )
        } else {
            let a = out_rgba[3].simd_max(zero).simd_min(one);
            let inv_a = one - a;
            let sr = out_rgba[0].simd_max(zero).simd_min(one) * a;
            let sg = out_rgba[1].simd_max(zero).simd_min(one) * a;
            let sb = out_rgba[2].simd_max(zero).simd_min(one) * a;
            (
                (sr + dr * inv_a).simd_max(zero).simd_min(one),
                (sg + dg * inv_a).simd_max(zero).simd_min(one),
                (sb + db * inv_a).simd_max(zero).simd_min(one),
                (a + da * inv_a).simd_max(zero).simd_min(one),
            )
        };
        if write_mask != 0xFF {
            let m = Mask::<i32, SIMD_LANES>::from_bitmask(write_mask as u64);
            nr = m.select(nr, dr);
            ng = m.select(ng, dg);
            nb = m.select(nb, db);
            na = m.select(na, da);
        }
        let ra = nr.to_array();
        let ga = ng.to_array();
        let ba = nb.to_array();
        let aa = na.to_array();
        canvas[row0] = [ra[0], ga[0], ba[0], aa[0]];
        canvas[row0 + 1] = [ra[1], ga[1], ba[1], aa[1]];
        canvas[row1] = [ra[2], ga[2], ba[2], aa[2]];
        canvas[row1 + 1] = [ra[3], ga[3], ba[3], aa[3]];
        canvas[row0 + 2] = [ra[4], ga[4], ba[4], aa[4]];
        canvas[row0 + 3] = [ra[5], ga[5], ba[5], aa[5]];
        canvas[row1 + 2] = [ra[6], ga[6], ba[6], aa[6]];
        canvas[row1 + 3] = [ra[7], ga[7], ba[7], aa[7]];
        return;
    }

    let r_a = out_rgba[0].to_array();
    let g_a = out_rgba[1].to_array();
    let b_a = out_rgba[2].to_array();
    let a_a = out_rgba[3].to_array();
    for lane in 0..SIMD_LANES {
        if (write_mask >> lane) & 1 != 0 {
            let lx = (px + TILE_IX[lane]) as usize;
            let ly = (py + TILE_IY[lane]) as usize;
            blend_pixel_into(
                &mut canvas[ly * w_usize + lx],
                [r_a[lane], g_a[lane], b_a[lane], a_a[lane]],
                is_additive,
            );
        }
    }
}

#[inline]
fn narrow_quad_pixel_bounds(
    rect_bounds: [f32; 4],
    clip_bounds: [f32; 4],
    uv_bounds: Option<[f32; 4]>,
    width: u32,
    height: u32,
) -> Option<(u32, u32, u32, u32)> {
    assert!(width > 0 && height > 0);
    let [x0, y0, x1, y1] = rect_bounds;
    let [cx0, cy0, cx1, cy1] = clip_bounds;
    let qw = x1 - x0;
    let qh = y1 - y0;
    if qw <= 1e-3 || qh <= 1e-3 {
        return None;
    }
    let (mut rx0, mut ry0, mut rx1, mut ry1) = (x0.max(cx0), y0.max(cy0), x1.min(cx1), y1.min(cy1));
    if let Some([u0, v0, u1, v1]) = uv_bounds {
        if u0 > u1 || v0 > v1 {
            return None;
        }
        rx0 = rx0.max(x0 + u0 * qw);
        ry0 = ry0.max(y0 + v0 * qh);
        rx1 = rx1.min(x0 + u1 * qw);
        ry1 = ry1.min(y0 + v1 * qh);
    }
    let min_x = rx0.floor().max(0.0) as u32;
    let max_x = rx1.ceil().min(width as f32) as u32;
    let min_y = ry0.floor().max(0.0) as u32;
    let max_y = ry1.ceil().min(height as f32) as u32;
    assert!(max_x <= width && max_y <= height);
    (min_x < max_x && min_y < max_y).then_some((min_x, max_x, min_y, max_y))
}

#[inline(always)]
fn row_coarse_occ(
    kill_tex_ch: Option<(&LinearTexture2D, usize)>,
    y0: f32,
    inv_qh_s: f32,
    py: u32,
) -> u16 {
    assert!(inv_qh_s > 0.0);
    let Some((tex, ch)) = kill_tex_ch else {
        return 0xFFFF;
    };
    assert!(ch < 4);
    let v0 = ((py as f32) + 0.5 - y0) * inv_qh_s;
    let v1 = ((py as f32) + 1.5 - y0) * inv_qh_s;
    tex.row_occupancy_mask(ch, v0, v1)
}

#[inline(always)]
fn coarse_u_span_mask(x0: f32, inv_qw_s: f32, px: u32) -> u16 {
    assert!(inv_qw_s > 0.0);
    let u0 = ((px as f32) + 0.5 - x0) * inv_qw_s;
    let u1 = ((px as f32) + 3.5 - x0) * inv_qw_s;
    assert!(u0 <= u1);
    let gx0 = ((u0 * 16.0).floor() as i32).clamp(0, 15) as u32;
    let gx1 = ((u1 * 16.0).floor() as i32).clamp(0, 15) as u32;
    ((2u32 << gx1) - (1u32 << gx0)) as u16
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn exec_raster_tile(
    canvas: &mut [[f32; 4]],
    w_usize: usize,
    h_usize: usize,
    px: u32,
    py: u32,
    valid_mask: u8,
    is_branchless: bool,
    is_additive: bool,
    shader: &CompiledSm5Shader,
    base_simd: &[u32x8; REG_FILE_SIZE],
    tile_regs: &mut [u32x8; REG_FILE_SIZE],
    fx: f32x8,
    fy: f32x8,
    u: f32x8,
    v: f32x8,
    cb2: &[[f32; 4]],
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    row_y_cache: &[(i32x8, f32x8); 4],
) {
    assert_eq!(SIMD_LANES, 8);
    assert!(valid_mask != 0);
    let (alive, out_opt) = if is_branchless {
        (
            valid_mask,
            exec_tile_branchless(
                shader,
                tile_regs,
                fx,
                u,
                cb2,
                textures,
                tex_clamps,
                row_y_cache,
            ),
        )
    } else {
        let mut tile_idx_temps = [[[0u32; 4]; MAX_INDEXABLE_TEMP_LEN]; SIMD_LANES];
        let (a, rgba) = execute_tile_x8_with_scratch(
            shader,
            base_simd,
            tile_regs,
            &mut tile_idx_temps,
            fx,
            fy,
            u,
            v,
            valid_mask,
            cb2,
            textures,
            tex_clamps,
        );
        (a, (a != 0).then_some(rgba))
    };
    if let Some(mut out_rgba) = out_opt {
        let zero_v = f32x8::splat(0.0);
        let active_px = if is_additive {
            (out_rgba[0].simd_gt(zero_v)
                | out_rgba[1].simd_gt(zero_v)
                | out_rgba[2].simd_gt(zero_v))
            .to_bitmask() as u8
        } else {
            out_rgba[3].simd_gt(zero_v).to_bitmask() as u8
        };
        let write_mask = alive & active_px;
        if write_mask != 0 {
            if is_branchless {
                apply_srgb_output_mask(&mut out_rgba, shader.srgb_encode_mask);
            }
            blend_tile_x8(
                canvas,
                w_usize,
                h_usize,
                px,
                py,
                write_mask,
                out_rgba,
                is_additive,
            );
        }
    }
}

/// Rasterizes a compiled SM5 material quad across `canvas` using Stage 1 scalar uniform preamble
/// evaluation + Stage 2 `4x2` dual-quad `f32x8` SIMD tiles.
#[allow(clippy::too_many_arguments)]
pub fn rasterize_compiled_quad(
    canvas: &mut [[f32; 4]],
    width: u32,
    height: u32,
    rect_bounds: [f32; 4],
    clip_bounds: [f32; 4],
    shader: &CompiledSm5Shader,
    vertex_color_linear: [f32; 4],
    cb2: &[[f32; 4]],
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    real_time: f32,
    is_additive: bool,
) {
    assert_eq!(canvas.len(), (width * height) as usize);
    let kill_tex_ch = shader
        .uv_kill_slot_ch
        .and_then(|(slot, ch)| textures.get(slot as usize).map(|&t| (t, (ch & 3) as usize)));
    let uv_bounds = kill_tex_ch.map(|(t, c)| t.nonzero_uv_bounds[c]);
    let Some((min_x, max_x, min_y, max_y)) =
        narrow_quad_pixel_bounds(rect_bounds, clip_bounds, uv_bounds, width, height)
    else {
        return;
    };
    let [x0, y0, x1, y1] = rect_bounds;
    let inv_qw_s = 1.0 / (x1 - x0);
    let inv_qh_s = 1.0 / (y1 - y0);
    assert!(max_x <= width && max_y <= height);

    let is_branchless = !shader.has_control_flow && !shader.uses_indexable_temps;
    let mut tile_regs = [u32x8::splat(0); REG_FILE_SIZE];
    init_tile_uniform_regs(shader, &mut tile_regs, vertex_color_linear, cb2, real_time);
    let base_simd = if is_branchless {
        [u32x8::splat(0); REG_FILE_SIZE]
    } else {
        tile_regs
    };
    let (inv_qw, inv_qh) = (f32x8::splat(inv_qw_s), f32x8::splat(inv_qh_s));
    let (x0_v, y0_v) = (f32x8::splat(x0), f32x8::splat(y0));
    let (dx_v, dy_v) = (f32x8::from_array(TILE_DX), f32x8::from_array(TILE_DY));
    let (ix_v, iy_v) = (u32x8::from_array(TILE_IX), u32x8::from_array(TILE_IY));
    let (max_x_v, max_y_v) = (u32x8::splat(max_x), u32x8::splat(max_y));
    let (zero_v, one_v) = (f32x8::splat(0.0), f32x8::splat(1.0));
    let mut row_y_cache = [(i32x8::splat(0), f32x8::splat(0.0)); 4];

    let mut py = min_y;
    while py < max_y {
        let row_occ = row_coarse_occ(kill_tex_ch, y0, inv_qh_s, py);
        if row_occ == 0 {
            py += 2;
            continue;
        }
        let fy = f32x8::splat((py as f32) + 0.5) + dy_v;
        let v = (fy - y0_v) * inv_qh;
        let py_lanes = u32x8::splat(py) + iy_v;
        let y_valid_mask =
            (py_lanes.simd_lt(max_y_v) & v.simd_ge(zero_v) & v.simd_le(one_v)).to_bitmask() as u8;
        if y_valid_mask == 0 {
            py += 2;
            continue;
        }
        if is_branchless {
            set_tile_y_regs(
                shader,
                &mut tile_regs,
                fy,
                v,
                textures,
                tex_clamps,
                &mut row_y_cache,
            );
        }

        let mut px = min_x;
        while px < max_x {
            if row_occ != 0xFFFF && (row_occ & coarse_u_span_mask(x0, inv_qw_s, px)) == 0 {
                px += 4;
                continue;
            }
            let fx = f32x8::splat((px as f32) + 0.5) + dx_v;
            let u = (fx - x0_v) * inv_qw;
            let px_lanes = u32x8::splat(px) + ix_v;
            let x_valid = px_lanes.simd_lt(max_x_v) & u.simd_ge(zero_v) & u.simd_le(one_v);
            let valid_mask = (x_valid.to_bitmask() as u8) & y_valid_mask;
            if valid_mask != 0 {
                exec_raster_tile(
                    canvas,
                    width as usize,
                    height as usize,
                    px,
                    py,
                    valid_mask,
                    is_branchless,
                    is_additive,
                    shader,
                    &base_simd,
                    &mut tile_regs,
                    fx,
                    fy,
                    u,
                    v,
                    cb2,
                    textures,
                    tex_clamps,
                    &row_y_cache,
                );
            }
            px += 4;
        }
        py += 2;
    }
}

/// Rasterizes a plain unshaded `LinearTexture2D` sprite quad across `canvas` using 8-wide
/// SIMD (`f32x8`) bilinear filtering and vectorized `Linear -> sRGB` conversion.
pub fn rasterize_sprite_quad_simd(
    canvas: &mut [[f32; 4]],
    width: u32,
    height: u32,
    rect_bounds: [f32; 4],
    clip_bounds: [f32; 4],
    tex: &LinearTexture2D,
    vertex_color_linear: [f32; 4],
) {
    assert_eq!(canvas.len(), (width * height) as usize);
    let Some((min_x, max_x, min_y, max_y)) = narrow_quad_pixel_bounds(
        rect_bounds,
        clip_bounds,
        Some(tex.nonzero_uv_bounds[3]),
        width,
        height,
    ) else {
        return;
    };
    let [x0, y0, x1, y1] = rect_bounds;
    let inv_qw_s = 1.0 / (x1 - x0);
    let inv_qh_s = 1.0 / (y1 - y0);
    assert!(max_x <= width && max_y <= height);

    let (inv_qw, inv_qh) = (f32x8::splat(inv_qw_s), f32x8::splat(inv_qh_s));
    let (x0_v, y0_v) = (f32x8::splat(x0), f32x8::splat(y0));
    let (dx_v, dy_v) = (f32x8::from_array(TILE_DX), f32x8::from_array(TILE_DY));
    let (ix_v, iy_v) = (u32x8::from_array(TILE_IX), u32x8::from_array(TILE_IY));
    let (max_x_v, max_y_v) = (u32x8::splat(max_x), u32x8::splat(max_y));
    let (zero_v, one_v) = (f32x8::splat(0.0), f32x8::splat(1.0));
    let vcol_r = f32x8::splat(vertex_color_linear[0]);
    let vcol_g = f32x8::splat(vertex_color_linear[1]);
    let vcol_b = f32x8::splat(vertex_color_linear[2]);
    let vcol_a = f32x8::splat(vertex_color_linear[3]);

    let mut py = min_y;
    while py < max_y {
        let row_occ = row_coarse_occ(Some((tex, 3)), y0, inv_qh_s, py);
        if row_occ == 0 {
            py += 2;
            continue;
        }
        let fy = f32x8::splat((py as f32) + 0.5) + dy_v;
        let v = (fy - y0_v) * inv_qh;
        let py_lanes = u32x8::splat(py) + iy_v;
        let y_valid_mask =
            (py_lanes.simd_lt(max_y_v) & v.simd_ge(zero_v) & v.simd_le(one_v)).to_bitmask() as u8;
        if y_valid_mask == 0 {
            py += 2;
            continue;
        }
        let (iy0_s, ty) = tex.precompute_row_y_x8(v, true);

        let mut px = min_x;
        while px < max_x {
            if row_occ != 0xFFFF && (row_occ & coarse_u_span_mask(x0, inv_qw_s, px)) == 0 {
                px += 4;
                continue;
            }
            let fx = f32x8::splat((px as f32) + 0.5) + dx_v;
            let u = (fx - x0_v) * inv_qw;
            let px_lanes = u32x8::splat(px) + ix_v;
            let x_valid = px_lanes.simd_lt(max_x_v) & u.simd_ge(zero_v) & u.simd_le(one_v);
            let valid_mask = (x_valid.to_bitmask() as u8) & y_valid_mask;
            if valid_mask != 0 {
                let t = tex.sample_bilinear_with_y_x8(u, iy0_s, ty, true, 0x8F);
                let sa = t[3] * vcol_a;
                let write_mask = valid_mask & (sa.simd_gt(zero_v).to_bitmask() as u8);
                if write_mask != 0 {
                    let [sr, sg, sb] =
                        linear_to_srgb_rgb_x8([t[0] * vcol_r, t[1] * vcol_g, t[2] * vcol_b]);
                    blend_tile_x8(
                        canvas,
                        width as usize,
                        height as usize,
                        px,
                        py,
                        write_mask,
                        [sr, sg, sb, sa],
                        false,
                    );
                }
            }
            px += 4;
        }
        py += 2;
    }
}

#[allow(clippy::too_many_arguments)]
fn exec_tile_transformed_branchless(
    shader: &CompiledSm5Shader,
    regs: &mut [u32x8; REG_FILE_SIZE],
    fx: f32x8,
    fy: f32x8,
    u0: f32x8,
    v0: f32x8,
    u1: f32x8,
    v1: f32x8,
    cb2: &[[f32; 4]],
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
) -> Option<[f32x8; 4]> {
    assert!(!shader.has_control_flow);
    assert!(!shader.uses_indexable_temps);
    let fx_b = fx.to_bits();
    let fy_b = fy.to_bits();
    let u0_b = u0.to_bits();
    let v0_b = v0.to_bits();
    regs[REG_V0_X as usize] = fx_b;
    regs[REG_V0_Y as usize] = fy_b;
    regs[REG_V2_X as usize] = fx_b;
    regs[REG_V2_Y as usize] = fy_b;
    regs[REG_V3_U as usize] = u0_b;
    regs[REG_V3_V as usize] = v0_b;
    regs[REG_V4_X as usize] = u0_b;
    regs[REG_V4_Y as usize] = v0_b;
    regs[REG_V4_Z as usize] = u1.to_bits();
    regs[REG_V4_W as usize] = v1.to_bits();

    for inst in &shader.row_preamble {
        regs[inst.dst as usize] = eval_simd_alu_value(regs, inst);
    }
    let dummy_y = [(i32x8::splat(0), f32x8::splat(0.0)); 4];
    for inst in &shader.kernel {
        match inst.op {
            VmOp::Ret => break,
            VmOp::EarlyOutZero => {
                if regs[inst.src0 as usize] == u32x8::splat(0) {
                    return None;
                }
            }
            VmOp::Sample2D | VmOp::Gather4 => {
                let mut no_row_y = *inst;
                no_row_y.imm16 &= 0x01FF;
                exec_sample2d_unmasked(regs, &no_row_y, textures, tex_clamps, &dummy_y);
            }
            VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::LoadCb2Dyn | VmOp::LoadIcbDyn => {
                exec_tex_or_mem_cold(regs, inst, textures, tex_clamps, cb2, &shader.icb_vec4s);
            }
            _ => {
                regs[inst.dst as usize] = eval_simd_alu_value(regs, inst);
            }
        }
    }
    Some([
        f32x8::from_bits(regs[REG_OUT_R as usize]),
        f32x8::from_bits(regs[REG_OUT_G as usize]),
        f32x8::from_bits(regs[REG_OUT_B as usize]),
        f32x8::from_bits(regs[REG_OUT_A as usize]),
    ])
}

/// Rasterizes a compiled SM5 quad with arbitrary 2D affine UV transforms (`uv0_affine` for
/// `v3.xy` / `v4.xy` and `uv1_affine` for `v4.zw`, where `[a, b, c, d, e, f]` maps screen
/// `(fx, fy)` to `(a*fx + b*fy + c, d*fx + e*fy + f)`), supporting rotated/sheared widgets,
/// `3_Slate_Tiled_UV` (`v4.zw = (Utiling, Vtiling)`), and `4_Slate_FontAtlas` (`FSlateMaterialShaderPS<true>`).
#[allow(clippy::too_many_arguments)]
pub fn rasterize_compiled_quad_transformed(
    canvas: &mut [[f32; 4]],
    width: u32,
    height: u32,
    clip_bounds: [f32; 4],
    uv0_affine: [f32; 6],
    uv1_affine: [f32; 6],
    shader: &CompiledSm5Shader,
    vertex_color_linear: [f32; 4],
    cb2: &[[f32; 4]],
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    real_time: f32,
    is_additive: bool,
) {
    assert_eq!(canvas.len(), (width * height) as usize);
    let Some((min_x, max_x, min_y, max_y)) =
        narrow_quad_pixel_bounds(clip_bounds, clip_bounds, None, width, height)
    else {
        return;
    };
    assert!(max_x <= width && max_y <= height);

    let mut tile_regs = [u32x8::splat(0); REG_FILE_SIZE];
    init_tile_uniform_regs(shader, &mut tile_regs, vertex_color_linear, cb2, real_time);
    let base_simd = tile_regs;
    let is_branchless = !shader.has_control_flow && !shader.uses_indexable_temps;
    let (dx_v, dy_v) = (f32x8::from_array(TILE_DX), f32x8::from_array(TILE_DY));
    let (ix_v, iy_v) = (u32x8::from_array(TILE_IX), u32x8::from_array(TILE_IY));
    let (max_x_v, max_y_v) = (u32x8::splat(max_x), u32x8::splat(max_y));
    let (zero_v, one_v) = (f32x8::splat(0.0), f32x8::splat(1.0));
    let [a0, b0, c0, d0, e0, f0] = uv0_affine.map(f32x8::splat);
    let [a1, b1, c1, d1, e1, f1] = uv1_affine.map(f32x8::splat);

    let mut py = min_y;
    while py < max_y {
        let fy = f32x8::splat((py as f32) + 0.5) + dy_v;
        let py_valid = (u32x8::splat(py) + iy_v).simd_lt(max_y_v);
        let mut px = min_x;
        while px < max_x {
            let fx = f32x8::splat((px as f32) + 0.5) + dx_v;
            let u0 = a0 * fx + b0 * fy + c0;
            let v0 = d0 * fx + e0 * fy + f0;
            let in_quad = u0.simd_ge(zero_v)
                & u0.simd_le(one_v)
                & v0.simd_ge(zero_v)
                & v0.simd_le(one_v)
                & (u32x8::splat(px) + ix_v).simd_lt(max_x_v)
                & py_valid;
            let valid_mask = in_quad.to_bitmask() as u8;
            if valid_mask != 0 {
                let u1 = a1 * fx + b1 * fy + c1;
                let v1 = d1 * fx + e1 * fy + f1;
                if is_branchless {
                    if let Some(mut rgba) = exec_tile_transformed_branchless(
                        shader,
                        &mut tile_regs,
                        fx,
                        fy,
                        u0,
                        v0,
                        u1,
                        v1,
                        cb2,
                        textures,
                        tex_clamps,
                    ) {
                        apply_srgb_output_mask(&mut rgba, shader.srgb_encode_mask);
                        let write_mask = valid_mask & (rgba[3].simd_gt(zero_v).to_bitmask() as u8);
                        if write_mask != 0 {
                            blend_tile_x8(
                                canvas,
                                width as usize,
                                height as usize,
                                px,
                                py,
                                write_mask,
                                rgba,
                                is_additive,
                            );
                        }
                    }
                } else {
                    let (alive, rgba) = execute_tile_x8(
                        shader, &base_simd, fx, fy, u0, v0, valid_mask, cb2, textures, tex_clamps,
                    );
                    if alive != 0 {
                        blend_tile_x8(
                            canvas,
                            width as usize,
                            height as usize,
                            px,
                            py,
                            alive,
                            rgba,
                            is_additive,
                        );
                    }
                }
            }
            px += 4;
        }
        py += 2;
    }
}

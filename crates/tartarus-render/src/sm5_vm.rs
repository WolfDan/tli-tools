//! Pure-Rust CPU interpreter for DirectX Shader Model 5 (`ps_5_0`) pixel shaders
//! extracted from Unreal Engine 4.27 `FSlateMaterialShaderPS`.

use tartarus_extract::{DecodedTexture, DxbcShader, Sm5Instruction, Sm5Operand};

const MAX_TEMPS: usize = 64;
const MAX_INPUTS: usize = 8;
const MAX_INSTRUCTIONS_EXEC: usize = 4096;

const _: () = {
    assert!(MAX_TEMPS == 64);
    assert!(MAX_INPUTS == 8);
};

/// Pre-decoded 2D texture in Linear RGBA32F space for fast bilinear sampling in SM5 shaders.
#[derive(Clone, Debug)]
pub struct LinearTexture2D {
    pub width: usize,
    pub height: usize,
    pub srgb: bool,
    pub clamp_x: bool,
    pub clamp_y: bool,
    pub texels: Vec<[f32; 4]>,
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

impl LinearTexture2D {
    /// Creates a `LinearTexture2D` from a `DecodedTexture`, converting RGB from sRGB to Linear
    /// if `tex.srgb` is true.
    pub fn from_decoded(tex: &DecodedTexture) -> Self {
        assert!(tex.width > 0 && tex.height > 0);
        assert_eq!(tex.rgba8.len(), tex.width * tex.height * 4);
        let mut texels = Vec::with_capacity(tex.width * tex.height);
        for px in tex.rgba8.as_chunks::<4>().0 {
            let r = px[0] as f32 / 255.0;
            let g = px[1] as f32 / 255.0;
            let b = px[2] as f32 / 255.0;
            let a = px[3] as f32 / 255.0;
            if tex.srgb {
                texels.push([
                    srgb_to_linear_channel(r),
                    srgb_to_linear_channel(g),
                    srgb_to_linear_channel(b),
                    a,
                ]);
            } else {
                texels.push([r, g, b, a]);
            }
        }
        assert_eq!(texels.len(), tex.width * tex.height);
        Self {
            width: tex.width,
            height: tex.height,
            srgb: tex.srgb,
            clamp_x: tex.clamp_x,
            clamp_y: tex.clamp_y,
            texels,
        }
    }

    /// Creates a `1x1` solid fallback texture.
    pub fn solid_rgba(rgba: [f32; 4]) -> Self {
        assert!(rgba[0].is_finite());
        assert!(rgba[3].is_finite());
        Self {
            width: 1,
            height: 1,
            srgb: false,
            clamp_x: true,
            clamp_y: true,
            texels: vec![rgba],
        }
    }

    /// Samples the texture at `(u, v)` using bilinear filtering, honoring both the texture's
    /// cooked `AddressX`/`AddressY` mode and per-slot material `UClamp`/`VClamp` overrides.
    pub fn sample_bilinear(
        &self,
        u: f32,
        v: f32,
        clamp_u_override: bool,
        clamp_v_override: bool,
    ) -> [f32; 4] {
        assert!(self.width > 0 && self.height > 0);
        assert_eq!(self.texels.len(), self.width * self.height);
        let clamp_u = self.clamp_x || clamp_u_override;
        let clamp_v = self.clamp_y || clamp_v_override;
        let uu = if clamp_u {
            u.clamp(0.0, 1.0)
        } else {
            u.rem_euclid(1.0)
        };
        let vv = if clamp_v {
            v.clamp(0.0, 1.0)
        } else {
            v.rem_euclid(1.0)
        };

        let fx = uu * (self.width as f32) - 0.5;
        let fy = vv * (self.height as f32) - 0.5;
        let x0 = fx.floor() as isize;
        let y0 = fy.floor() as isize;
        let tx = fx - (x0 as f32);
        let ty = fy - (y0 as f32);

        let coord = |idx: isize, dim: usize, clamp: bool| -> usize {
            if clamp {
                idx.clamp(0, dim as isize - 1) as usize
            } else {
                idx.rem_euclid(dim as isize) as usize
            }
        };
        let ix0 = coord(x0, self.width, clamp_u);
        let ix1 = coord(x0 + 1, self.width, clamp_u);
        let iy0 = coord(y0, self.height, clamp_v);
        let iy1 = coord(y0 + 1, self.height, clamp_v);

        let c00 = self.texels[iy0 * self.width + ix0];
        let c10 = self.texels[iy0 * self.width + ix1];
        let c01 = self.texels[iy1 * self.width + ix0];
        let c11 = self.texels[iy1 * self.width + ix1];

        let mut out = [0.0f32; 4];
        for i in 0..4 {
            let top = c00[i] + (c10[i] - c00[i]) * tx;
            let bot = c01[i] + (c11[i] - c01[i]) * tx;
            out[i] = top + (bot - top) * ty;
        }
        out
    }
}

fn is_candidate_slate_quad_ps(shader: &DxbcShader, expected_num_textures: usize) -> bool {
    assert!(shader.instructions.len() <= 65_536);
    if shader.program_type != 0
        || shader.input_signature.len() != 5
        || shader.output_signature.len() != 1
    {
        return false;
    }
    let mut max_tex_reg: Option<u32> = None;
    for inst in &shader.instructions {
        // Reject 9-slice box shaders that read `cb0[2]`.
        for op in &inst.operands {
            if op.op_type == 8 && op.indices.first() == Some(&0) && op.indices.get(1) == Some(&2) {
                return false;
            }
        }
        if matches!(inst.opcode, 69..=74)
            && let Some(tex_op) = inst.operands.get(2)
            && tex_op.op_type == 7
            && let Some(&t_idx) = tex_op.indices.first()
        {
            max_tex_reg = Some(max_tex_reg.map_or(t_idx, |m| m.max(t_idx)));
        }
    }
    // Reject the `bUseTextureAlpha = true` font permutation which binds an extra `t{N}` slot.
    let actual_tex_count = max_tex_reg.map_or(0, |m| (m + 1) as usize);
    actual_tex_count <= expected_num_textures
}

fn uses_v4_border_or_box_tiling(shader: &DxbcShader) -> bool {
    assert!(shader.instructions.len() <= 65_536);
    for inst in &shader.instructions {
        // `frc` (`26`) directly on `v4` or multiplying `v4.xy * v4.zw` (`56`) is used by
        // the Slate border/box tiling vertex-to-pixel permutations.
        if inst.opcode == 26
            && let Some(src) = inst.operands.get(1)
            && src.op_type == 1
            && src.indices.first() == Some(&4)
        {
            return true;
        }
        if inst.opcode == 56
            && let (Some(a), Some(b)) = (inst.operands.get(1), inst.operands.get(2))
            && a.op_type == 1
            && b.op_type == 1
            && a.indices.first() == Some(&4)
            && b.indices.first() == Some(&4)
            && a.swizzle != b.swizzle
        {
            return true;
        }
    }
    false
}

/// Selects the standard Slate 2D quad pixel shader (`FSlateMaterialShaderPS<false>`) from a
/// material's `ShaderMap` shader list.
pub fn select_slate_default_ps(
    shaders: &[DxbcShader],
    expected_num_textures: usize,
) -> Option<usize> {
    assert!(shaders.len() <= 1024);
    let mut best_idx = None;
    let mut best_score = (true, usize::MAX);
    for (idx, sh) in shaders.iter().enumerate() {
        if !is_candidate_slate_quad_ps(sh, expected_num_textures) {
            continue;
        }
        let is_tiling = uses_v4_border_or_box_tiling(sh);
        let score = (is_tiling, sh.instructions.len());
        if score < best_score {
            best_score = score;
            best_idx = Some(idx);
        }
    }
    best_idx
}

struct Sm5VmState<'a> {
    r: [[u32; 4]; MAX_TEMPS],
    v: [[f32; 4]; MAX_INPUTS],
    o0: [f32; 4],
    discarded: bool,
    cb0: [[f32; 4]; 8],
    cb1_142: [f32; 4],
    cb2: &'a [[f32; 4]],
    textures: &'a [&'a LinearTexture2D],
    tex_clamps: &'a [(bool, bool)],
    icb: &'a [[u32; 4]],
}

impl<'a> Sm5VmState<'a> {
    fn read_raw_u32_vec4(&self, op: &Sm5Operand) -> [u32; 4] {
        assert!(op.num_comps <= 3);
        let idx0 = op.indices.first().copied().unwrap_or(0) as usize;
        let idx1 = op.indices.get(1).copied().unwrap_or(0) as usize;
        match op.op_type {
            0 => self.r.get(idx0).copied().unwrap_or([0; 4]),
            1 => self
                .v
                .get(idx0)
                .copied()
                .unwrap_or([0.0; 4])
                .map(f32::to_bits),
            4 | 5 => {
                if op.num_comps == 2 {
                    op.imm_u32
                } else {
                    [op.imm_u32[0]; 4]
                }
            }
            8 => {
                let f = match idx0 {
                    0 => self.cb0.get(idx1).copied().unwrap_or([0.0; 4]),
                    1 => {
                        if idx1 == 142 {
                            self.cb1_142
                        } else {
                            [0.0; 4]
                        }
                    }
                    2 => self.cb2.get(idx1).copied().unwrap_or([0.0; 4]),
                    _ => [0.0; 4],
                };
                f.map(f32::to_bits)
            }
            9 => self.icb.get(idx0).copied().unwrap_or([0; 4]),
            _ => [0; 4],
        }
    }

    fn read_u32_vec4(&self, op: &Sm5Operand) -> [u32; 4] {
        assert!(op.num_comps <= 3);
        let raw = self.read_raw_u32_vec4(op);
        let swz = if op.op_type == 4 || op.op_type == 5 {
            raw
        } else if op.num_comps == 2 {
            [
                raw[(op.swizzle[0] as usize) & 3],
                raw[(op.swizzle[1] as usize) & 3],
                raw[(op.swizzle[2] as usize) & 3],
                raw[(op.swizzle[3] as usize) & 3],
            ]
        } else {
            [raw[0]; 4]
        };
        if !op.abs && !op.neg {
            return swz;
        }
        swz.map(|bits| {
            let mut f = f32::from_bits(bits);
            if op.abs {
                f = f.abs();
            }
            if op.neg {
                f = -f;
            }
            f.to_bits()
        })
    }

    fn read_f32_vec4(&self, op: &Sm5Operand) -> [f32; 4] {
        self.read_u32_vec4(op).map(f32::from_bits)
    }

    fn write_f32_vec4(&mut self, dst: &Sm5Operand, val: [f32; 4], saturate: bool) {
        assert!(dst.num_comps <= 3);
        let mut bits = [0u32; 4];
        for i in 0..4 {
            let mut v = if val[i].is_nan() { 0.0 } else { val[i] };
            if saturate {
                v = v.clamp(0.0, 1.0);
            }
            bits[i] = v.to_bits();
        }
        self.write_u32_vec4(dst, bits);
    }

    fn write_u32_vec4(&mut self, dst: &Sm5Operand, val: [u32; 4]) {
        assert!(dst.num_comps <= 3);
        let idx0 = dst.indices.first().copied().unwrap_or(0) as usize;
        let mask = dst.mask;
        match dst.op_type {
            0 if idx0 < MAX_TEMPS => {
                for c in 0..4 {
                    if (mask >> c) & 1 != 0 {
                        self.r[idx0][c] = val[c];
                    }
                }
            }
            2 if idx0 == 0 => {
                for c in 0..4 {
                    if (mask >> c) & 1 != 0 {
                        self.o0[c] = f32::from_bits(val[c]);
                    }
                }
            }
            _ => {}
        }
    }
}

fn find_matching_branch(
    instructions: &[Sm5Instruction],
    start_pc: usize,
    stop_on_else: bool,
) -> usize {
    assert!(start_pc <= instructions.len());
    let mut depth = 0usize;
    for (pc, inst) in instructions.iter().enumerate().skip(start_pc) {
        match inst.opcode {
            31 | 48 | 76 => depth += 1,
            18 if depth == 0 && stop_on_else => return pc + 1,
            21..=23 => {
                if depth == 0 {
                    return pc + 1;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    instructions.len()
}

fn exec_float_alu(state: &mut Sm5VmState<'_>, inst: &Sm5Instruction) -> bool {
    assert!(inst.operands.len() >= 2);
    let dst = &inst.operands[0];
    let a = state.read_f32_vec4(&inst.operands[1]);
    let b = inst
        .operands
        .get(2)
        .map_or([0.0; 4], |op| state.read_f32_vec4(op));
    let c = inst
        .operands
        .get(3)
        .map_or([0.0; 4], |op| state.read_f32_vec4(op));

    let mut out = [0.0f32; 4];
    match inst.opcode {
        0 => out = [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]],
        56 => out = [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]],
        50 => {
            for i in 0..4 {
                out[i] = a[i] * b[i] + c[i];
            }
        }
        14 => {
            for i in 0..4 {
                out[i] = if b[i].abs() > 1e-12 { a[i] / b[i] } else { 0.0 };
            }
        }
        15 => out = [a[0] * b[0] + a[1] * b[1]; 4],
        16 => out = [a[0] * b[0] + a[1] * b[1] + a[2] * b[2]; 4],
        17 => out = [a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]; 4],
        51 => {
            out = [
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].min(b[2]),
                a[3].min(b[3]),
            ]
        }
        52 => {
            out = [
                a[0].max(b[0]),
                a[1].max(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ]
        }
        25 => out = a.map(f32::exp2),
        47 => out = a.map(|x| x.max(1e-12).log2()),
        26 => out = a.map(|x| x - x.floor()),
        64 => out = a.map(f32::round),
        65 => out = a.map(f32::floor),
        66 => out = a.map(f32::ceil),
        67 => out = a.map(f32::trunc),
        68 => out = a.map(|x| if x > 1e-12 { 1.0 / x.sqrt() } else { 0.0 }),
        75 => out = a.map(|x| x.max(0.0).sqrt()),
        129 => out = a.map(|x| if x.abs() > 1e-12 { 1.0 / x } else { 0.0 }),
        11 | 12 | 122..=125 => out = [0.0; 4],
        _ => return false,
    }
    state.write_f32_vec4(dst, out, inst.saturate);
    true
}

fn exec_int_or_cmp_alu(state: &mut Sm5VmState<'_>, inst: &Sm5Instruction) -> bool {
    assert!(inst.operands.len() >= 2);
    let dst = &inst.operands[0];
    let ua = state.read_u32_vec4(&inst.operands[1]);
    let ub = inst
        .operands
        .get(2)
        .map_or([0; 4], |op| state.read_u32_vec4(op));
    let uc = inst
        .operands
        .get(3)
        .map_or([0; 4], |op| state.read_u32_vec4(op));
    let fa = ua.map(f32::from_bits);
    let fb = ub.map(f32::from_bits);

    let mut out = [0u32; 4];
    for i in 0..4 {
        out[i] = match inst.opcode {
            54 => {
                if inst.saturate {
                    fa[i].clamp(0.0, 1.0).to_bits()
                } else {
                    ua[i]
                }
            }
            55 => {
                let chosen = if ua[i] != 0 { ub[i] } else { uc[i] };
                if inst.saturate {
                    f32::from_bits(chosen).clamp(0.0, 1.0).to_bits()
                } else {
                    chosen
                }
            }
            24 => {
                if fa[i] == fb[i] {
                    0xFFFF_FFFF
                } else {
                    0
                }
            }
            57 => {
                if fa[i] != fb[i] {
                    0xFFFF_FFFF
                } else {
                    0
                }
            }
            49 => {
                if fa[i] < fb[i] {
                    0xFFFF_FFFF
                } else {
                    0
                }
            }
            29 => {
                if fa[i] >= fb[i] {
                    0xFFFF_FFFF
                } else {
                    0
                }
            }
            32 => {
                if ua[i] == ub[i] {
                    0xFFFF_FFFF
                } else {
                    0
                }
            }
            39 => {
                if ua[i] != ub[i] {
                    0xFFFF_FFFF
                } else {
                    0
                }
            }
            34 => {
                if (ua[i] as i32) < (ub[i] as i32) {
                    0xFFFF_FFFF
                } else {
                    0
                }
            }
            33 => {
                if (ua[i] as i32) >= (ub[i] as i32) {
                    0xFFFF_FFFF
                } else {
                    0
                }
            }
            79 => {
                if ua[i] < ub[i] {
                    0xFFFF_FFFF
                } else {
                    0
                }
            }
            80 => {
                if ua[i] >= ub[i] {
                    0xFFFF_FFFF
                } else {
                    0
                }
            }
            1 => ua[i] & ub[i],
            60 => ua[i] | ub[i],
            87 => ua[i] ^ ub[i],
            59 => !ua[i],
            30 => (ua[i] as i32).wrapping_add(ub[i] as i32) as u32,
            38 | 81 => ua[i].wrapping_mul(ub[i]),
            35 | 82 => ua[i].wrapping_mul(ub[i]).wrapping_add(uc[i]),
            40 => (ua[i] as i32).wrapping_neg() as u32,
            41 => ua[i].wrapping_shl(ub[i] & 31),
            42 => ((ua[i] as i32).wrapping_shr(ub[i] & 31)) as u32,
            85 => ua[i].wrapping_shr(ub[i] & 31),
            27 => (fa[i] as i32) as u32,
            28 => fa[i].max(0.0) as u32,
            43 => ((ua[i] as i32) as f32).to_bits(),
            86 => (ua[i] as f32).to_bits(),
            _ => return false,
        };
    }
    state.write_u32_vec4(dst, out);
    true
}

fn exec_sample_or_sincos(state: &mut Sm5VmState<'_>, inst: &Sm5Instruction) {
    assert!(!inst.operands.is_empty());
    if inst.opcode == 77 && inst.operands.len() >= 3 {
        let src = state.read_f32_vec4(&inst.operands[2]);
        if inst.operands[0].op_type != 13 {
            state.write_f32_vec4(&inst.operands[0], src.map(f32::sin), inst.saturate);
        }
        if inst.operands[1].op_type != 13 {
            state.write_f32_vec4(&inst.operands[1], src.map(f32::cos), inst.saturate);
        }
        return;
    }
    if matches!(inst.opcode, 69..=74) && inst.operands.len() >= 3 {
        let uv = state.read_f32_vec4(&inst.operands[1]);
        let tex_op = &inst.operands[2];
        let t_idx = tex_op.indices.first().copied().unwrap_or(0) as usize;
        let (cu, cv) = state
            .tex_clamps
            .get(t_idx)
            .copied()
            .unwrap_or((false, false));
        let raw_rgba = state
            .textures
            .get(t_idx)
            .map_or([0.0, 0.0, 0.0, 1.0], |tex| {
                tex.sample_bilinear(uv[0], uv[1], cu, cv)
            });
        let swz = tex_op.swizzle;
        let sampled = [
            raw_rgba[(swz[0] as usize) & 3],
            raw_rgba[(swz[1] as usize) & 3],
            raw_rgba[(swz[2] as usize) & 3],
            raw_rgba[(swz[3] as usize) & 3],
        ];
        state.write_f32_vec4(&inst.operands[0], sampled, inst.saturate);
    }
}

/// Executes a parsed SM5 pixel shader (`DxbcShader`) for a single pixel at normalized quad
/// coordinates `uv` with vertex color `vertex_color_linear` (`v1 : COLOR0` in Linear RGB)
/// and material uniform buffer `cb2`. Returns `None` if `discard` was executed.
///
/// `cb0[1].y` (`ShaderParams.y = b GammaCorrect`) is set to `0.0` so `FSlateMaterialShaderPS`
/// executes its built-in 15-instruction sRGB epilogue (`r1.xyz`) and outputs gamma-corrected
/// sRGB into `o0.xyz`, matching UE4 Slate's gamma-space render target.
#[allow(clippy::too_many_arguments)]
pub fn execute_sm5_ps(
    shader: &DxbcShader,
    uv: [f32; 2],
    screen_pos: [f32; 2],
    vertex_color_linear: [f32; 4],
    cb2: &[[f32; 4]],
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    real_time: f32,
) -> Option<[f32; 4]> {
    let [u, v] = uv;
    assert!(shader.instructions.len() <= 65_536);
    assert!(u.is_finite() && v.is_finite());

    let mut cb0 = [[0.0f32; 4]; 8];
    cb0[1] = [1.0, 0.0, 0.0, 1.0];

    let mut v_regs = [[0.0f32; 4]; MAX_INPUTS];
    v_regs[0] = [screen_pos[0], screen_pos[1], 0.0, 1.0];
    v_regs[1] = vertex_color_linear;
    v_regs[2] = [screen_pos[0], screen_pos[1], 0.0, 0.0];
    v_regs[3] = [u, v, 0.0, 0.0];
    v_regs[4] = [u, v, u, v];

    let mut state = Sm5VmState {
        r: [[0u32; 4]; MAX_TEMPS],
        v: v_regs,
        o0: [0.0; 4],
        discarded: false,
        cb0,
        cb1_142: [0.0, 0.0, real_time, 0.0],
        cb2,
        textures,
        tex_clamps,
        icb: &shader.icb_vec4s,
    };

    let insts = &shader.instructions;
    let mut pc = 0usize;
    for _ in 0..MAX_INSTRUCTIONS_EXEC {
        if pc >= insts.len() {
            break;
        }
        let inst = &insts[pc];
        pc += 1;
        let eval_cond = |s: &Sm5VmState<'_>, op: &Sm5Operand| {
            let cond = s.read_u32_vec4(op)[0];
            if inst.test_nonzero {
                cond != 0
            } else {
                cond == 0
            }
        };
        match inst.opcode {
            88..=106 | 147..=162 | 21..=23 | 58 => continue,
            31 if !inst.operands.is_empty() && !eval_cond(&state, &inst.operands[0]) => {
                pc = find_matching_branch(insts, pc, true);
            }
            18 => pc = find_matching_branch(insts, pc, false),
            13 if !inst.operands.is_empty() && eval_cond(&state, &inst.operands[0]) => {
                return None;
            }
            62 => break,
            63 if !inst.operands.is_empty() && eval_cond(&state, &inst.operands[0]) => break,
            69..=74 | 77 => exec_sample_or_sincos(&mut state, inst),
            _ if inst.operands.len() >= 2 => {
                if !exec_float_alu(&mut state, inst) {
                    let _ = exec_int_or_cmp_alu(&mut state, inst);
                }
            }
            _ => {}
        }
    }
    (!state.discarded).then_some(state.o0)
}

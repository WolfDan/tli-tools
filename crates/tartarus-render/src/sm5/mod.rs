//! High-performance Data-Oriented, Scalarized SIMT Virtual Machine (`f32x8` / `u32x8` dual-quad
//! wavefronts) for DirectX Shader Model 5 (`ps_5_0`) shaders.

pub mod compiler;
pub mod exec;
pub mod ir;
pub mod texture;

use tartarus_extract::DxbcShader;

pub use compiler::{
    compile_sm5_shader, compile_sm5_shader_specialized, compile_sm5_shader_with_cb2,
};
pub use exec::{
    eval_shader_preamble, execute_tile_x8, rasterize_compiled_quad,
    rasterize_compiled_quad_transformed, rasterize_sprite_quad_simd,
};
pub use ir::{
    CompiledSm5Shader, SIMD_LANES, ShaderSpecialization, TexSlotMeta, VmInst, VmOp, f32x8, i32x8,
    u32x8,
};
pub use texture::{
    LinearTexture2D, fast_atan2_x8, fast_exp2_x8, fast_log2_x8, fast_pow_x8,
    linear_to_srgb_channel, linear_to_srgb_x8, srgb_to_linear_channel,
};

fn count_sampled_textures(shader: &DxbcShader) -> usize {
    assert!(shader.instructions.len() <= 65_536);
    let mut max_tex_reg: Option<u32> = None;
    for inst in &shader.instructions {
        if matches!(inst.opcode, 69..=74)
            && let Some(tex_op) = inst.operands.get(2)
            && tex_op.op_type == 7
            && let Some(&t_idx) = tex_op.indices.first()
        {
            max_tex_reg = Some(max_tex_reg.map_or(t_idx, |m| m.max(t_idx)));
        }
    }
    max_tex_reg.map_or(0, |m| (m + 1) as usize)
}

fn is_candidate_slate_quad_ps(shader: &DxbcShader, expected_num_textures: usize) -> bool {
    assert!(shader.instructions.len() <= 65_536);
    if shader.program_type != 0
        || shader.input_signature.len() != 5
        || shader.output_signature.len() != 1
    {
        return false;
    }
    for inst in &shader.instructions {
        // Reject 9-slice box shaders that read `cb0[2]`.
        for op in &inst.operands {
            if op.op_type == 8 && op.indices.first() == Some(&0) && op.indices.get(1) == Some(&2) {
                return false;
            }
        }
    }
    count_sampled_textures(shader) <= expected_num_textures
}

fn uses_v4_9slice_border(shader: &DxbcShader) -> bool {
    assert!(shader.instructions.len() <= 65_536);
    shader.instructions.iter().any(|inst| {
        inst.opcode == 26
            && inst
                .operands
                .get(1)
                .is_some_and(|s| s.op_type == 1 && s.indices.first() == Some(&4))
    })
}

fn uses_v4_border_or_box_tiling(shader: &DxbcShader) -> bool {
    assert!(shader.instructions.len() <= 65_536);
    if shader
        .input_signature
        .iter()
        .any(|s| s.register == 4 && (s.rw_mask & 0x0C) != 0)
    {
        return true;
    }
    for inst in &shader.instructions {
        if inst.opcode == 95
            && let Some(op) = inst.operands.first()
            && op.op_type == 1
            && op.indices.first() == Some(&4)
            && (op.mask & 0x0C) != 0
        {
            return true;
        }
        for op in inst.operands.iter().skip(1) {
            if op.op_type == 1
                && op.indices.first() == Some(&4)
                && op.swizzle.iter().any(|&c| c >= 2)
            {
                return true;
            }
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

/// Selects the Slate tiled brush UV pixel shader (`v4.xy * v4.zw`) from a material's `ShaderMap`.
pub fn select_slate_tiled_ps(
    shaders: &[DxbcShader],
    expected_num_textures: usize,
) -> Option<usize> {
    assert!(shaders.len() <= 1024);
    let mut best_idx = None;
    let mut best_len = usize::MAX;
    for (idx, sh) in shaders.iter().enumerate() {
        if !is_candidate_slate_quad_ps(sh, expected_num_textures) {
            continue;
        }
        if uses_v4_border_or_box_tiling(sh)
            && !uses_v4_9slice_border(sh)
            && sh.instructions.len() < best_len
        {
            best_len = sh.instructions.len();
            best_idx = Some(idx);
        }
    }
    best_idx
}

/// Selects the Slate FontAtlas pixel shader (`FSlateMaterialShaderPS<true>`) from a material's
/// `ShaderMap`, where `t_{N}` is the font glyph atlas sampled at `v4.xy` and `t0..t_{N-1}` are
/// the material textures sampled at `v4.zw`.
pub fn select_slate_font_ps(
    shaders: &[DxbcShader],
    expected_num_textures: usize,
    font_rgba: bool,
) -> Option<usize> {
    assert!(shaders.len() <= 1024);
    let mut candidates = Vec::new();
    for (idx, sh) in shaders.iter().enumerate() {
        if !is_candidate_slate_quad_ps(sh, expected_num_textures + 1) {
            continue;
        }
        if count_sampled_textures(sh) == expected_num_textures + 1
            && uses_v4_border_or_box_tiling(sh)
            && !uses_v4_9slice_border(sh)
        {
            candidates.push((sh.instructions.len(), idx));
        }
    }
    candidates.sort_by_key(|&(len, idx)| (len, idx));
    if font_rgba {
        candidates.last().map(|&(_, idx)| idx)
    } else {
        candidates.first().map(|&(_, idx)| idx)
    }
}

/// Executes a pre-compiled `CompiledSm5Shader` for a single pixel `(u, v)`.
#[allow(clippy::too_many_arguments)]
pub fn execute_compiled_sm5_ps(
    compiled: &CompiledSm5Shader,
    uv: [f32; 2],
    screen_pos: [f32; 2],
    vertex_color_linear: [f32; 4],
    cb2: &[[f32; 4]],
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    real_time: f32,
) -> Option<[f32; 4]> {
    assert!(uv[0].is_finite() && uv[1].is_finite());
    assert!(screen_pos[0].is_finite() && screen_pos[1].is_finite());
    let base_simd = eval_shader_preamble(compiled, vertex_color_linear, cb2, real_time);
    let (alive, out_rgba) = execute_tile_x8(
        compiled,
        &base_simd,
        f32x8::splat(screen_pos[0]),
        f32x8::splat(screen_pos[1]),
        f32x8::splat(uv[0]),
        f32x8::splat(uv[1]),
        0x01,
        cb2,
        textures,
        tex_clamps,
    );
    (alive & 1 != 0).then(|| {
        [
            out_rgba[0].to_array()[0],
            out_rgba[1].to_array()[0],
            out_rgba[2].to_array()[0],
            out_rgba[3].to_array()[0],
        ]
    })
}

/// Compiles and executes a `DxbcShader` for a single pixel `(u, v)`.
/// For full-quad rendering, prefer pre-compiling with [`compile_sm5_shader`] and calling
/// [`rasterize_compiled_quad`].
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
    assert!(shader.instructions.len() <= 65_536);
    assert!(uv[0].is_finite() && uv[1].is_finite());
    let compiled = compile_sm5_shader(shader);
    execute_compiled_sm5_ps(
        &compiled,
        uv,
        screen_pos,
        vertex_color_linear,
        cb2,
        textures,
        tex_clamps,
        real_time,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tartarus_extract::{Sm5Instruction, Sm5Operand};

    fn make_reg_op(op_type: u32, idx: u32, mask: u8, swizzle: [u8; 4]) -> Sm5Operand {
        Sm5Operand {
            op_type,
            num_comps: 2,
            sel_mode: 1,
            mask,
            swizzle,
            neg: false,
            abs: false,
            indices: vec![idx],
            dynamic_index: None,
            imm_u32: [0; 4],
        }
    }

    fn make_imm_f32(vals: [f32; 4]) -> Sm5Operand {
        Sm5Operand {
            op_type: 4,
            num_comps: 2,
            sel_mode: 1,
            mask: 0x0F,
            swizzle: [0, 1, 2, 3],
            neg: false,
            abs: false,
            indices: Vec::new(),
            dynamic_index: None,
            imm_u32: vals.map(f32::to_bits),
        }
    }

    fn make_inst(opcode: u32, saturate: bool, operands: Vec<Sm5Operand>) -> Sm5Instruction {
        Sm5Instruction {
            opcode,
            opcode_token0: opcode,
            saturate,
            test_nonzero: true,
            res_dim: 0,
            operands,
            raw_dwords: Vec::new(),
        }
    }

    #[test]
    fn test_simd_transcendentals_and_srgb_accuracy() {
        let inputs = [0.001f32, 0.01, 0.05, 0.1, 0.25, 0.5, 0.75, 1.0];
        let v = f32x8::from_array(inputs);
        let l2 = fast_log2_x8(v).to_array();
        let e2 = fast_exp2_x8(v).to_array();
        let srgb = linear_to_srgb_x8(v).to_array();

        for i in 0..8 {
            assert!((l2[i] - inputs[i].log2()).abs() < 1e-4);
            assert!((e2[i] - inputs[i].exp2()).abs() < 1e-4);
            assert!((srgb[i] - linear_to_srgb_channel(inputs[i])).abs() < 1e-4);
        }
    }

    #[test]
    fn test_self_aliasing_swizzle_and_derivatives() {
        // r0 = v3.xyzw; r0.xy = r0.yx * [2.0, 3.0, 1.0, 1.0]; o0.xy = ddx(v0.x), ddy(v0.y), r0.xy
        let r0_all = make_reg_op(0, 0, 0x0F, [0, 1, 2, 3]);
        let r0_xy = make_reg_op(0, 0, 0x03, [0, 1, 2, 3]);
        let r0_yx = make_reg_op(0, 0, 0x0F, [1, 0, 2, 3]);
        let v3_all = make_reg_op(1, 3, 0x0F, [0, 1, 2, 3]);
        let v0_all = make_reg_op(1, 0, 0x0F, [0, 1, 2, 3]);
        let o0_zw = make_reg_op(2, 0, 0x0C, [0, 1, 0, 1]);
        let o0_x = make_reg_op(2, 0, 0x01, [0, 0, 0, 0]);
        let o0_y = make_reg_op(2, 0, 0x02, [1, 1, 1, 1]);

        let shader = DxbcShader {
            program_type: 0,
            major_version: 5,
            minor_version: 0,
            total_size: 0,
            chunk_fourccs: Vec::new(),
            input_signature: Vec::new(),
            output_signature: Vec::new(),
            instructions: vec![
                make_inst(54, false, vec![r0_all, v3_all]),
                make_inst(
                    56,
                    false,
                    vec![r0_xy, r0_yx, make_imm_f32([2.0, 3.0, 1.0, 1.0])],
                ),
                make_inst(
                    54,
                    false,
                    vec![o0_zw, make_reg_op(0, 0, 0x0F, [0, 1, 0, 1])],
                ),
                make_inst(11, false, vec![o0_x, v0_all.clone()]),
                make_inst(12, false, vec![o0_y, v0_all]),
                make_inst(62, false, Vec::new()),
            ],
            icb_vec4s: Vec::new(),
        };

        let compiled = compile_sm5_shader(&shader);
        let base_simd = eval_shader_preamble(&compiled, [1.0; 4], &[], 0.0);
        let fx = f32x8::from_array([0.5, 1.5, 0.5, 1.5, 2.5, 3.5, 2.5, 3.5]);
        let fy = f32x8::from_array([0.5, 0.5, 1.5, 1.5, 0.5, 0.5, 1.5, 1.5]);
        let u = f32x8::splat(0.25);
        let v = f32x8::splat(0.5);
        let (alive, out) =
            execute_tile_x8(&compiled, &base_simd, fx, fy, u, v, 0xFF, &[], &[], &[]);
        assert_eq!(alive, 0xFF);
        // ddx(fx) = 1.0, ddy(fy) = 1.0, r0.x = v * 2.0 = 1.0, r0.y = u * 3.0 = 0.75
        assert!((out[0].to_array()[0] - 1.0).abs() < 1e-6);
        assert!((out[1].to_array()[0] - 1.0).abs() < 1e-6);
        assert!((out[2].to_array()[0] - 1.0).abs() < 1e-6);
        assert!((out[3].to_array()[0] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_uniform_preamble_hoisting_and_dce() {
        // r0.xyz = dp3(cb2[0].xyz, -cb2[1].xyz) -> uniform, must hoist to preamble!
        // r1.xyzw = dead calculation -> must be eliminated by DCE!
        // o0.xyzw = r0.xxxx + v3.xyzw -> varying, stays in kernel!
        let mut cb2_1_neg = Sm5Operand {
            op_type: 8,
            num_comps: 2,
            sel_mode: 1,
            mask: 0x0F,
            swizzle: [0, 1, 2, 3],
            neg: true,
            abs: false,
            indices: vec![2, 1],
            dynamic_index: None,
            imm_u32: [0; 4],
        };
        let mut cb2_0 = cb2_1_neg.clone();
        cb2_0.neg = false;
        cb2_0.indices = vec![2, 0];
        cb2_1_neg.abs = true;

        let r0_x = make_reg_op(0, 0, 0x01, [0, 0, 0, 0]);
        let r1_all = make_reg_op(0, 1, 0x0F, [0, 1, 2, 3]);
        let o0_all = make_reg_op(2, 0, 0x0F, [0, 1, 2, 3]);
        let v3_all = make_reg_op(1, 3, 0x0F, [0, 1, 2, 3]);

        let shader = DxbcShader {
            program_type: 0,
            major_version: 5,
            minor_version: 0,
            total_size: 0,
            chunk_fourccs: Vec::new(),
            input_signature: Vec::new(),
            output_signature: Vec::new(),
            instructions: vec![
                make_inst(16, false, vec![r0_x, cb2_0, cb2_1_neg]),
                make_inst(
                    0,
                    false,
                    vec![r1_all, v3_all.clone(), make_imm_f32([9.0; 4])],
                ),
                make_inst(
                    0,
                    false,
                    vec![o0_all, make_reg_op(0, 0, 0x0F, [0, 0, 0, 0]), v3_all],
                ),
                make_inst(62, false, Vec::new()),
            ],
            icb_vec4s: Vec::new(),
        };

        let compiled = compile_sm5_shader(&shader);
        assert!(!compiled.preamble.is_empty());
        // Kernel only contains the 2 varying FAdd instructions for o0.xy (o0.zw are uniform and hoisted to preamble).
        assert_eq!(compiled.kernel.len(), 2);

        let cb2 = [[1.0f32, 2.0, 3.0, 0.0], [2.0, -1.0, 4.0, 0.0]];
        // dp3([1,2,3], -|[2,-1,4]|) = 1*(-2) + 2*(-1) + 3*(-4) = -16.0
        let px = execute_compiled_sm5_ps(
            &compiled,
            [0.5, 0.25],
            [0.0, 0.0],
            [1.0; 4],
            &cb2,
            &[],
            &[],
            0.0,
        )
        .unwrap();
        assert!((px[0] - (-15.5)).abs() < 1e-5);
        assert!((px[1] - (-15.75)).abs() < 1e-5);
        assert!((px[2] - (-16.0)).abs() < 1e-5);
        assert!((px[3] - (-16.0)).abs() < 1e-5);
    }

    #[test]
    fn test_divergent_control_flow_loop_and_multi_dst_int() {
        // Test divergent if/else + loop/breakc + discard + udiv across 8 lanes:
        // r0.x = ftou(v0.x); (lane 0..8 -> 0, 1, 0, 1, 2, 3, 2, 3)
        // udiv r1.x (quot), r2.x (rem), r0.x, imm(2)
        // if_nz r2.x (odd X lanes: 1, 3, 5, 7) -> o0.x = 10.0 else -> o0.x = 2.0
        // discard_nz (v0.x >= 3.0) -> discards lanes 5 & 7 (where fx = 3.5)
        let r0_x = make_reg_op(0, 0, 0x01, [0; 4]);
        let r1_x = make_reg_op(0, 1, 0x01, [0; 4]);
        let r2_x = make_reg_op(0, 2, 0x01, [0; 4]);
        let r3_x = make_reg_op(0, 3, 0x01, [0; 4]);
        let o0_all = make_reg_op(2, 0, 0x0F, [0, 0, 0, 0]);
        let v0_x = make_reg_op(1, 0, 0x01, [0; 4]);
        let imm_u2 = Sm5Operand {
            op_type: 4,
            num_comps: 2,
            sel_mode: 1,
            mask: 0x0F,
            swizzle: [0, 1, 2, 3],
            neg: false,
            abs: false,
            indices: Vec::new(),
            dynamic_index: None,
            imm_u32: [2; 4],
        };

        let shader = DxbcShader {
            program_type: 0,
            major_version: 5,
            minor_version: 0,
            total_size: 0,
            chunk_fourccs: Vec::new(),
            input_signature: Vec::new(),
            output_signature: Vec::new(),
            instructions: vec![
                make_inst(28, false, vec![r0_x.clone(), v0_x.clone()]),
                make_inst(78, false, vec![r1_x, r2_x.clone(), r0_x, imm_u2]),
                make_inst(31, false, vec![r2_x]),
                make_inst(54, false, vec![o0_all.clone(), make_imm_f32([10.0; 4])]),
                make_inst(18, false, Vec::new()),
                make_inst(54, false, vec![o0_all, make_imm_f32([2.0; 4])]),
                make_inst(21, false, Vec::new()),
                make_inst(29, false, vec![r3_x.clone(), v0_x, make_imm_f32([3.0; 4])]),
                make_inst(13, false, vec![r3_x]),
                make_inst(62, false, Vec::new()),
            ],
            icb_vec4s: Vec::new(),
        };

        let compiled = compile_sm5_shader(&shader);
        let base_simd = eval_shader_preamble(&compiled, [1.0; 4], &[], 0.0);
        let fx = f32x8::from_array([0.5, 1.5, 0.5, 1.5, 2.5, 3.5, 2.5, 3.5]);
        let fy = f32x8::splat(0.5);
        let (alive, out) = execute_tile_x8(
            &compiled,
            &base_simd,
            fx,
            fy,
            f32x8::splat(0.0),
            f32x8::splat(0.0),
            0xFF,
            &[],
            &[],
            &[],
        );
        // Lanes 5 and 7 have fx = 3.5 >= 3.0, so they are discarded (0xFF & !(1<<5 | 1<<7) = 0x5F).
        assert_eq!(alive, 0b0101_1111);
        let out_r = out[0].to_array();
        assert_eq!(out_r[0], 2.0);
        assert_eq!(out_r[1], 10.0);
        assert_eq!(out_r[4], 2.0);
    }

    #[test]
    fn test_fast_atan2_and_pow_accuracy() {
        let ys = [-1.0f32, -0.5, -0.1, 0.0, 0.1, 0.5, 1.0, 2.0];
        let xs = [1.0f32, -1.0, 0.5, 1.0, -0.5, 0.25, -2.0, 0.5];
        let atan_v = fast_atan2_x8(f32x8::from_array(ys), f32x8::from_array(xs)).to_array();
        for i in 0..8 {
            let expected = ys[i].atan2(xs[i]);
            assert!(
                (atan_v[i] - expected).abs() < 1e-4,
                "atan2 mismatch at lane {i}: got {}, expected {expected}",
                atan_v[i]
            );
        }

        let bases = [0.05f32, 0.1, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0];
        let exps = [2.2f32, 0.4545, 3.0, 1.5, 0.5, 4.0, 2.0, 3.0];
        let pow_v = fast_pow_x8(f32x8::from_array(bases), f32x8::from_array(exps)).to_array();
        for i in 0..8 {
            let expected = bases[i].powf(exps[i]);
            assert!(
                (pow_v[i] - expected).abs() < 1e-3,
                "pow mismatch at lane {i}: got {}, expected {expected}",
                pow_v[i]
            );
        }
    }

    #[test]
    fn test_npot_wrap_guard_and_uv_bounds() {
        let mut rgba = vec![0u8; 6 * 5 * 4];
        // Non-zero texel at (2, 2) inside a 6x5 NPOT texture -> interior non-zero UV bounds.
        let idx = (2 * 6 + 2) * 4;
        rgba[idx] = 255;
        rgba[idx + 3] = 255;
        let dec = tartarus_extract::DecodedTexture {
            width: 6,
            height: 5,
            format: tartarus_extract::PixelFormat::B8G8R8A8,
            num_mips: 1,
            srgb: false,
            clamp_x: false,
            clamp_y: false,
            names: Vec::new(),
            rgba8: rgba,
        };
        let tex = LinearTexture2D::from_decoded(&dec);
        assert_eq!(tex.stride, 7);
        assert!(tex.nonzero_uv_bounds[0][0] > 0.0);
        assert!(tex.nonzero_uv_bounds[0][2] < 1.0);
        // Verify wrap-around at seam u = 0.0 and u = 1.0 matches across wrap guard.
        let s0 = tex.sample_bilinear(0.0, 0.0, false, false);
        let s1 = tex.sample_bilinear(1.0, 1.0, false, false);
        assert!((s0[0] - s1[0]).abs() < 1e-6);
    }

    #[test]
    fn test_compile_and_execute_all_s16_mechanic_ui_shaders_if_game_installed() {
        let Some(game_dir) = tartarus_extract::discover_game_dir() else {
            return;
        };
        let Ok(vfs) =
            tartarus_extract::PakVfs::open_game_dir(&game_dir, tartarus_extract::DEFAULT_AES_KEY)
        else {
            return;
        };
        let Ok(archives) = vfs.load_shader_archive_indices() else {
            return;
        };

        let dummy_tex = LinearTexture2D::solid_rgba([0.5, 0.5, 0.5, 0.5]);
        let tex_refs: [&LinearTexture2D; 12] = [&dummy_tex; 12];
        let tex_clamps = [(false, false); 12];

        let mut mat_paths: Vec<String> = vfs
            .collect_entries(
                |dir_lower| {
                    dir_lower.contains("s16gameplay")
                        || dir_lower.contains("uiv2")
                        || dir_lower.contains("art/ui/")
                },
                |dir_lower, file_lower| {
                    file_lower.ends_with(".uasset")
                        && ((dir_lower.contains("s16gameplay")
                            && (dir_lower.contains("/material")
                                || file_lower.contains("_mi_")
                                || file_lower.contains("_m_")))
                            || dir_lower.contains("/fontmats")
                            || file_lower.starts_with("ui_mi_v2_autoreward")
                            || matches!(
                                file_lower,
                                "ui_m_v2_base.uasset"
                                    | "ui_m_v2_highdissolve.uasset"
                                    | "ui_m_v2_pageflip.uasset"
                                    | "ui_m_v2_shieldprogress.uasset"
                                    | "ui_m_v2_breatheuvbase.uasset"
                                    | "ui_v2_mat_ptc01_lf.uasset"
                                    | "ui_v2_mat_ptc02_lf.uasset"
                                    | "ui_mat_basenew.uasset"
                            ))
                },
            )
            .into_iter()
            .map(|(p, _)| p.trim_end_matches(".uasset").to_string())
            .collect();
        mat_paths.sort();
        mat_paths.dedup();

        let mut valid_mats = 0usize;
        let mut seen_shadermaps = std::collections::BTreeSet::new();
        let mut total_ps_tested = 0usize;
        let mut saw_hermite = false;

        for mat_path in &mat_paths {
            let Ok(info) = vfs.read_material_info(mat_path) else {
                continue;
            };
            let Some(sm_hash) = info.shader_map_hash else {
                continue;
            };
            valid_mats += 1;
            let cb2 = info.evaluate_uniform_buffer(&[], &[]);
            let expected_tex = info.resolve_texture_slots(&[]).len();

            if !seen_shadermaps.insert((sm_hash, expected_tex, cb2.len())) {
                continue;
            }
            let mut shaders = Vec::new();
            for (rec, arch) in &archives {
                let Some(&sm_idx) = arch.map_hash_to_index.get(&sm_hash) else {
                    continue;
                };
                let sm = &arch.shader_map_entries[sm_idx];
                for k in 0..(sm.num_shaders as usize) {
                    let s_idx =
                        arch.shader_indices[(sm.shader_indices_offset as usize) + k] as usize;
                    if arch.shader_entries[s_idx].frequency == tartarus_extract::ShaderStage::Pixel
                    {
                        let blob = vfs.extract_shader_entry(rec, arch, s_idx).unwrap();
                        shaders.push(tartarus_extract::parse_dxbc(&blob.dxbc_bytes).unwrap());
                    }
                }
                if !shaders.is_empty() {
                    break;
                }
            }
            assert!(!shaders.is_empty(), "No pixel shaders for {mat_path}");

            let def_idx = select_slate_default_ps(&shaders, expected_tex).expect(mat_path);
            assert!(
                !uses_v4_border_or_box_tiling(&shaders[def_idx]),
                "Expected non-tiling FSlateMaterialShaderPS<false> for {mat_path}"
            );
            assert!(select_slate_tiled_ps(&shaders, expected_tex).is_some());
            assert!(select_slate_font_ps(&shaders, expected_tex, false).is_some());

            for sh in &shaders {
                let comp_dyn = compile_sm5_shader(sh);
                let comp_spec = compile_sm5_shader_with_cb2(sh, Some(&cb2));
                assert!(comp_dyn.num_regs_used <= super::ir::REG_FILE_SIZE);
                assert!(comp_spec.num_regs_used <= super::ir::REG_FILE_SIZE);
                if comp_dyn.kernel.iter().any(|i| i.op == VmOp::FHermite) {
                    saw_hermite = true;
                }
                let base_simd = eval_shader_preamble(&comp_spec, [1.0; 4], &cb2, 0.5);
                let (_, rgba) = execute_tile_x8(
                    &comp_spec,
                    &base_simd,
                    f32x8::splat(16.5),
                    f32x8::splat(16.5),
                    f32x8::splat(0.5),
                    f32x8::splat(0.5),
                    0xFF,
                    &cb2,
                    &tex_refs,
                    &tex_clamps,
                );
                for ch in &rgba {
                    assert!(ch.to_array().iter().all(|v| v.is_finite()));
                }
                total_ps_tested += 1;
            }
        }
        assert!(
            valid_mats >= 312,
            "Expected >= 312 S16 materials, got {valid_mats}"
        );
        assert!(
            total_ps_tested >= 579,
            "Expected >= 579 PS tested, got {total_ps_tested}"
        );
        assert!(
            saw_hermite,
            "Expected VmOp::FHermite fusion in UI_M_V2_HIghDissolve"
        );
    }
}

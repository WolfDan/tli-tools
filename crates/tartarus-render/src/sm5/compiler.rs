//! DXBC-to-Scalar-SIMT SSA Compiler, Constant/Peephole Folder, Common Subexpression Eliminator,
//! Dead-Code Eliminator, Uniform Preamble Splitter, and Linear-Scan Physical Register Allocator.

use tartarus_extract::{DxbcShader, Sm5Instruction, Sm5Operand};

use super::ir::{
    Cb2RegBinding, CompiledSm5Shader, ConstRegBinding, MAX_STAGE_INSTS, REG_ALIAS_TMP0,
    REG_DOT_ACC, REG_DYNAMIC_BASE, REG_FILE_SIZE, REG_MOD0, REG_MOD1, REG_MOD2, REG_NULL, REG_ONE,
    REG_OUT_A, REG_OUT_B, REG_OUT_G, REG_OUT_R, REG_REAL_TIME, REG_TRUE, REG_V0_W, REG_V0_X,
    REG_V1_R, REG_V2_X, REG_V3_U, REG_V4_X, REG_ZERO, VmInst, VmOp,
};

/// Internal SSA virtual-register instruction (`u16` register indices allow unlimited SSA renaming
/// before Dead-Code Elimination and Linear-Scan compaction into `u8` physical registers).
#[derive(Clone, Copy, Debug)]
struct RawInst {
    op: VmOp,
    dst: u16,
    src0: u16,
    src1: u16,
    src2: u16,
    aux: u8,
    imm16: u16,
}

impl RawInst {
    #[inline]
    const fn new(op: VmOp, dst: u16, src0: u16, src1: u16, src2: u16) -> Self {
        Self {
            op,
            dst,
            src0,
            src1,
            src2,
            aux: 0,
            imm16: 0,
        }
    }

    #[inline]
    const fn with_aux_imm(
        op: VmOp,
        dst: u16,
        src0: u16,
        src1: u16,
        src2: u16,
        aux: u8,
        imm16: u16,
    ) -> Self {
        Self {
            op,
            dst,
            src0,
            src1,
            src2,
            aux,
            imm16,
        }
    }
}

struct VregCb2Binding {
    vreg: u16,
    comp: u8,
    vec_idx: u16,
}

struct VregConstBinding {
    vreg: u16,
    bits: u32,
}

struct CompilerBuilder<'a> {
    shader: &'a DxbcShader,
    static_cb2: Option<&'a [[f32; 4]]>,
    insts: Vec<RawInst>,
    cb2_bindings: Vec<VregCb2Binding>,
    const_bindings: Vec<VregConstBinding>,
    ssa_map: [[u16; 4]; 64],
    next_vreg: u16,
    use_ssa: bool,
    has_control_flow: bool,
    uses_indexable_temps: bool,
}

fn scan_shader_hazards(shader: &DxbcShader) -> (bool, bool) {
    assert!(shader.instructions.len() <= 65_536);
    let mut has_cf = false;
    let mut has_idx = false;
    for inst in &shader.instructions {
        if matches!(
            inst.opcode,
            31 | 18 | 21 | 48 | 22 | 2 | 3 | 7 | 8 | 13 | 63
        ) {
            has_cf = true;
        }
        for op in &inst.operands {
            if op.op_type == 3 {
                has_idx = true;
            }
        }
    }
    assert!(!has_idx || shader.instructions.len() <= 65_536);
    (has_cf, has_idx)
}

impl<'a> CompilerBuilder<'a> {
    fn new(shader: &'a DxbcShader, static_cb2: Option<&'a [[f32; 4]]>) -> Self {
        assert!(shader.instructions.len() <= 65_536);
        let (has_control_flow, uses_indexable_temps) = scan_shader_hazards(shader);
        let use_ssa = !has_control_flow && !uses_indexable_temps;
        let mut ssa_map = [[u16::from(REG_ZERO); 4]; 64];
        let mut next_vreg = u16::from(REG_DYNAMIC_BASE);
        if !use_ssa {
            for row in ssa_map.iter_mut().take(16) {
                for slot in row.iter_mut() {
                    *slot = next_vreg;
                    next_vreg += 1;
                }
            }
        }
        assert!(next_vreg >= u16::from(REG_DYNAMIC_BASE));
        Self {
            shader,
            static_cb2,
            insts: Vec::with_capacity(shader.instructions.len() * 3),
            cb2_bindings: Vec::new(),
            const_bindings: Vec::new(),
            ssa_map,
            next_vreg,
            use_ssa,
            has_control_flow,
            uses_indexable_temps,
        }
    }

    #[inline]
    fn alloc_vreg(&mut self, count: u16) -> u16 {
        assert!((1..=4).contains(&count));
        assert!(self.next_vreg <= 32_768);
        let base = self.next_vreg;
        self.next_vreg += count;
        base
    }

    #[inline]
    fn alloc_scratch_vreg(&mut self, fallback_mod: u8) -> u16 {
        assert!(fallback_mod < REG_DYNAMIC_BASE);
        if self.use_ssa {
            self.alloc_vreg(1)
        } else {
            u16::from(fallback_mod)
        }
    }

    fn alloc_const(&mut self, bits: u32) -> u16 {
        assert!(self.const_bindings.len() <= 4096);
        if bits == 0 || bits == 0x8000_0000 {
            return u16::from(REG_ZERO);
        }
        if bits == 0x3F80_0000 {
            return u16::from(REG_ONE);
        }
        if bits == 0xFFFF_FFFF {
            return u16::from(REG_TRUE);
        }
        if let Some(b) = self.const_bindings.iter().find(|b| b.bits == bits) {
            return b.vreg;
        }
        let vreg = self.alloc_vreg(1);
        self.const_bindings.push(VregConstBinding { vreg, bits });
        assert!(!self.const_bindings.is_empty());
        vreg
    }

    fn alloc_cb2(&mut self, vec_idx: u16, comp: u8) -> u16 {
        assert!(comp < 4);
        if let Some(cb2) = self.static_cb2 {
            let val = cb2
                .get(vec_idx as usize)
                .map_or(0.0, |v| v[(comp & 3) as usize]);
            let clean = if val.is_nan() { 0.0 } else { val };
            return self.alloc_const(clean.to_bits());
        }
        if let Some(b) = self
            .cb2_bindings
            .iter()
            .find(|b| b.vec_idx == vec_idx && b.comp == comp)
        {
            return b.vreg;
        }
        let vreg = self.alloc_vreg(1);
        self.cb2_bindings.push(VregCb2Binding {
            vreg,
            comp,
            vec_idx,
        });
        assert!(!self.cb2_bindings.is_empty());
        vreg
    }

    fn parse_rel_reg(&mut self, dyn_str: &str) -> u16 {
        assert!(!dyn_str.is_empty());
        let token = dyn_str.split('+').next().unwrap_or(dyn_str).trim();
        let (reg_part, comp_part) = token.split_once('.').unwrap_or((token, "x"));
        let comp = match comp_part {
            "y" => 1usize,
            "z" => 2usize,
            "w" => 3usize,
            _ => 0usize,
        };
        let idx: usize = reg_part.trim_start_matches(['r', 'v']).parse().unwrap_or(0);
        assert!(comp < 4);
        if reg_part.starts_with('r') {
            self.ssa_map[idx.min(63)][comp]
        } else {
            u16::from(REG_ZERO)
        }
    }

    fn resolve_raw_src(&mut self, op: &Sm5Operand, comp: usize, mod_slot: u8) -> u16 {
        assert!(comp < 4);
        assert!(op.num_comps <= 3);
        let idx0 = op.indices.first().copied().unwrap_or(0) as usize;
        let idx1 = op.indices.get(1).copied().unwrap_or(0) as usize;
        let swz_c = if op.num_comps == 2 {
            op.swizzle[comp] & 3
        } else {
            0u8
        };

        match op.op_type {
            0 => self.ssa_map[idx0.min(63)][swz_c as usize],
            1 => u16::from(match idx0 {
                0 => REG_V0_X + swz_c,
                1 => REG_V1_R + swz_c,
                2 => REG_V2_X + swz_c,
                3 => REG_V3_U + swz_c,
                4 => REG_V4_X + swz_c,
                _ => REG_ZERO,
            }),
            3 => {
                let rel_reg = op
                    .dynamic_index
                    .as_deref()
                    .map_or(u16::from(REG_ZERO), |s| self.parse_rel_reg(s));
                let dst = self.alloc_scratch_vreg(mod_slot);
                self.insts.push(RawInst::with_aux_imm(
                    VmOp::LoadIdxTemp,
                    dst,
                    rel_reg,
                    idx0 as u16,
                    u16::from(swz_c),
                    0,
                    idx1 as u16,
                ));
                dst
            }
            8 => self.resolve_cb_src(op, idx0, idx1, swz_c, mod_slot),
            9 => self.resolve_icb_src(op, idx0, swz_c, mod_slot),
            _ => u16::from(REG_ZERO),
        }
    }

    fn resolve_cb_src(
        &mut self,
        op: &Sm5Operand,
        cb_slot: usize,
        vec_idx: usize,
        swz_c: u8,
        mod_slot: u8,
    ) -> u16 {
        assert!(swz_c < 4);
        assert!(mod_slot < REG_DYNAMIC_BASE);
        match cb_slot {
            0 => {
                if vec_idx == 1 && (swz_c == 0 || swz_c == 3) {
                    u16::from(REG_ONE)
                } else {
                    u16::from(REG_ZERO)
                }
            }
            1 if vec_idx == 142 && swz_c == 2 => u16::from(REG_REAL_TIME),
            2 => {
                if let Some(dyn_s) = op.dynamic_index.clone() {
                    let rel_reg = self.parse_rel_reg(&dyn_s);
                    let dst = self.alloc_scratch_vreg(mod_slot);
                    self.insts.push(RawInst::with_aux_imm(
                        VmOp::LoadCb2Dyn,
                        dst,
                        rel_reg,
                        u16::from(swz_c),
                        0,
                        0,
                        vec_idx as u16,
                    ));
                    dst
                } else {
                    self.alloc_cb2(vec_idx as u16, swz_c)
                }
            }
            _ => u16::from(REG_ZERO),
        }
    }

    fn resolve_icb_src(&mut self, op: &Sm5Operand, vec_idx: usize, swz_c: u8, mod_slot: u8) -> u16 {
        assert!(swz_c < 4);
        assert!(mod_slot < REG_DYNAMIC_BASE);
        if let Some(dyn_s) = op.dynamic_index.clone() {
            let rel_reg = self.parse_rel_reg(&dyn_s);
            let dst = self.alloc_scratch_vreg(mod_slot);
            self.insts.push(RawInst::with_aux_imm(
                VmOp::LoadIcbDyn,
                dst,
                rel_reg,
                u16::from(swz_c),
                0,
                0,
                vec_idx as u16,
            ));
            dst
        } else {
            let bits = self
                .shader
                .icb_vec4s
                .get(vec_idx)
                .map_or(0, |v| v[swz_c as usize]);
            self.alloc_const(bits)
        }
    }

    fn resolve_src(&mut self, op: &Sm5Operand, comp: usize, mod_slot: u8, is_int: bool) -> u16 {
        assert!(comp < 4);
        assert!(mod_slot < REG_DYNAMIC_BASE);
        if op.op_type == 4 || op.op_type == 5 {
            let mut bits = if op.num_comps == 2 {
                op.imm_u32[comp]
            } else {
                op.imm_u32[0]
            };
            if is_int {
                if op.abs {
                    bits = (bits as i32).wrapping_abs() as u32;
                }
                if op.neg {
                    bits = (bits as i32).wrapping_neg() as u32;
                }
            } else {
                if op.abs {
                    bits &= 0x7FFF_FFFF;
                }
                if op.neg {
                    bits ^= 0x8000_0000;
                }
            }
            return self.alloc_const(bits);
        }

        let base_reg = self.resolve_raw_src(op, comp, mod_slot);
        if !op.abs && !op.neg {
            return base_reg;
        }
        if is_int {
            let mut cur = base_reg;
            if op.abs {
                let dst = self.alloc_scratch_vreg(mod_slot);
                self.insts.push(RawInst::new(VmOp::IAbs, dst, cur, 0, 0));
                cur = dst;
            }
            if op.neg {
                let dst = self.alloc_scratch_vreg(mod_slot);
                self.insts.push(RawInst::new(VmOp::INeg, dst, cur, 0, 0));
                cur = dst;
            }
            cur
        } else {
            let mod_op = match (op.neg, op.abs) {
                (false, true) => VmOp::FAbs,
                (true, false) => VmOp::FNeg,
                _ => VmOp::FNegAbs,
            };
            let dst = self.alloc_scratch_vreg(mod_slot);
            self.insts.push(RawInst::new(mod_op, dst, base_reg, 0, 0));
            dst
        }
    }

    fn alloc_dst_scalar(&mut self, dst: &Sm5Operand, comp: usize) -> Option<u16> {
        assert!(comp < 4);
        let idx0 = dst.indices.first().copied().unwrap_or(0) as usize;
        match dst.op_type {
            0 => {
                let r = idx0.min(63);
                if self.use_ssa {
                    let vreg = self.alloc_vreg(1);
                    self.ssa_map[r][comp] = vreg;
                    Some(vreg)
                } else {
                    Some(self.ssa_map[r][comp])
                }
            }
            2 if idx0 == 0 => Some(u16::from(REG_OUT_R + (comp as u8))),
            _ => None,
        }
    }

    fn emit_dst_write(&mut self, dst: &Sm5Operand, comp: usize, val_reg: u16, sat: bool) {
        assert!(comp < 4);
        let idx0 = dst.indices.first().copied().unwrap_or(0) as usize;
        let idx1 = dst.indices.get(1).copied().unwrap_or(0) as usize;
        if let Some(d_reg) = self.alloc_dst_scalar(dst, comp) {
            let op = if sat { VmOp::MovSat } else { VmOp::Mov };
            self.insts.push(RawInst::new(op, d_reg, val_reg, 0, 0));
        } else if dst.op_type == 3 {
            let rel_reg = dst
                .dynamic_index
                .as_deref()
                .map_or(u16::from(REG_ZERO), |s| self.parse_rel_reg(s));
            let src = if sat {
                let s_reg = self.alloc_scratch_vreg(REG_MOD0);
                self.insts
                    .push(RawInst::new(VmOp::MovSat, s_reg, val_reg, 0, 0));
                s_reg
            } else {
                val_reg
            };
            self.insts.push(RawInst::with_aux_imm(
                VmOp::StoreIdxTemp,
                idx0 as u16,
                rel_reg,
                src,
                comp as u16,
                0,
                idx1 as u16,
            ));
        }
    }
}

fn map_alu_op(opcode: u32) -> Option<(VmOp, bool)> {
    assert!(opcode < 256);
    let (op, is_int) = match opcode {
        0 => (VmOp::FAdd, false),
        56 => (VmOp::FMul, false),
        50 => (VmOp::FMad, false),
        14 => (VmOp::FDiv, false),
        51 => (VmOp::FMin, false),
        52 => (VmOp::FMax, false),
        25 => (VmOp::FExp2, false),
        47 => (VmOp::FLog2, false),
        26 => (VmOp::FFrc, false),
        64 => (VmOp::FRoundNe, false),
        65 => (VmOp::FFloor, false),
        66 => (VmOp::FCeil, false),
        67 => (VmOp::FTrunc, false),
        68 => (VmOp::FRsq, false),
        75 => (VmOp::FSqrt, false),
        129 => (VmOp::FRcp, false),
        11 | 122 | 123 => (VmOp::FDdx, false),
        12 | 124 | 125 => (VmOp::FDdy, false),
        54 => (VmOp::Mov, false),
        55 => (VmOp::Movc, false),
        24 => (VmOp::FCmpEq, false),
        57 => (VmOp::FCmpNe, false),
        49 => (VmOp::FCmpLt, false),
        29 => (VmOp::FCmpGe, false),
        27 => (VmOp::FToI, false),
        28 => (VmOp::FToU, false),
        130 => (VmOp::F32ToF16, false),
        32 => (VmOp::ICmpEq, true),
        39 => (VmOp::ICmpNe, true),
        34 => (VmOp::ICmpLt, true),
        33 => (VmOp::ICmpGe, true),
        79 => (VmOp::UCmpLt, true),
        80 => (VmOp::UCmpGe, true),
        1 => (VmOp::BitAnd, true),
        60 => (VmOp::BitOr, true),
        87 => (VmOp::BitXor, true),
        59 => (VmOp::BitNot, true),
        30 => (VmOp::IAdd, true),
        35 | 82 => (VmOp::IMad, true),
        36 => (VmOp::IMax, true),
        37 => (VmOp::IMin, true),
        83 => (VmOp::UMax, true),
        84 => (VmOp::UMin, true),
        40 => (VmOp::INeg, true),
        41 => (VmOp::IShl, true),
        42 => (VmOp::IShr, true),
        85 => (VmOp::UShr, true),
        43 => (VmOp::IToF, true),
        86 => (VmOp::UToF, true),
        131 => (VmOp::F16ToF32, true),
        134 => (VmOp::CountBits, true),
        135 => (VmOp::FirstBitHi, true),
        136 => (VmOp::FirstBitLo, true),
        137 => (VmOp::FirstBitShi, true),
        138 => (VmOp::Ubfe, true),
        139 => (VmOp::Ibfe, true),
        140 => (VmOp::Bfi, true),
        141 => (VmOp::Bfrev, true),
        _ => return None,
    };
    assert!(op != VmOp::Nop);
    Some((op, is_int))
}

fn lower_dot_product(b: &mut CompilerBuilder<'_>, inst: &Sm5Instruction, dim: usize) {
    assert!((2..=4).contains(&dim));
    assert!(inst.operands.len() >= 3);
    let dst = &inst.operands[0];
    let mask = dst.mask & 0x0F;
    if mask == 0 {
        return;
    }
    let a0 = b.resolve_src(&inst.operands[1], 0, REG_MOD0, false);
    let b0 = b.resolve_src(&inst.operands[2], 0, REG_MOD1, false);
    let mut acc = b.alloc_scratch_vreg(REG_DOT_ACC);
    b.insts.push(RawInst::new(VmOp::FMul, acc, a0, b0, 0));

    for c in 1..dim {
        let ak = b.resolve_src(&inst.operands[1], c, REG_MOD0, false);
        let bk = b.resolve_src(&inst.operands[2], c, REG_MOD1, false);
        let next_acc = b.alloc_scratch_vreg(REG_DOT_ACC);
        b.insts
            .push(RawInst::new(VmOp::FMad, next_acc, ak, bk, acc));
        acc = next_acc;
    }

    for c in 0..4 {
        if (mask >> c) & 1 != 0 {
            b.emit_dst_write(dst, c, acc, inst.saturate);
        }
    }
}

fn lower_multi_dst_int(b: &mut CompilerBuilder<'_>, inst: &Sm5Instruction) {
    assert!(inst.operands.len() >= 3);
    let (dst0, dst1, src0, src1) = if inst.operands.len() >= 4 {
        (
            Some(&inst.operands[0]),
            &inst.operands[1],
            &inst.operands[2],
            &inst.operands[3],
        )
    } else {
        (
            None,
            &inst.operands[0],
            &inst.operands[1],
            &inst.operands[2],
        )
    };
    assert!(dst1.num_comps <= 3);

    for c in 0..4 {
        let write0 = dst0.is_some_and(|d| d.op_type != 13 && (d.mask >> c) & 1 != 0);
        let write1 = dst1.op_type != 13 && (dst1.mask >> c) & 1 != 0;
        if !write0 && !write1 {
            continue;
        }
        let is_signed = inst.opcode == 38;
        let s0 = b.resolve_src(src0, c, REG_MOD0, is_signed);
        let s1 = b.resolve_src(src1, c, REG_MOD1, is_signed);

        if let Some(d0) = dst0
            && write0
        {
            let op0 = match inst.opcode {
                38 => VmOp::IMulHi,
                81 => VmOp::UMulHi,
                78 => VmOp::UDivQuot,
                132 | 133 => VmOp::IAdd,
                _ => VmOp::IMulLo,
            };
            let tmp = b.alloc_scratch_vreg(REG_DOT_ACC);
            b.insts.push(RawInst::new(op0, tmp, s0, s1, 0));
            b.emit_dst_write(d0, c, tmp, false);
        }
        if write1 {
            let op1 = match inst.opcode {
                38 | 81 => VmOp::IMulLo,
                78 => VmOp::UDivRem,
                132 => VmOp::UAddCarry,
                133 => VmOp::USubBorrow,
                _ => VmOp::IMulLo,
            };
            let tmp = b.alloc_scratch_vreg(REG_DOT_ACC);
            b.insts.push(RawInst::new(op1, tmp, s0, s1, 0));
            b.emit_dst_write(dst1, c, tmp, false);
        }
    }
}

fn lower_sincos_or_sample(b: &mut CompilerBuilder<'_>, inst: &Sm5Instruction) {
    assert!(!inst.operands.is_empty());
    if inst.opcode == 77 && inst.operands.len() >= 3 {
        let d_sin = &inst.operands[0];
        let d_cos = &inst.operands[1];
        let src = &inst.operands[2];
        for c in 0..4 {
            let w_sin = d_sin.op_type != 13 && (d_sin.mask >> c) & 1 != 0;
            let w_cos = d_cos.op_type != 13 && (d_cos.mask >> c) & 1 != 0;
            if !w_sin && !w_cos {
                continue;
            }
            let s = b.resolve_src(src, c, REG_MOD0, false);
            if w_sin {
                let tmp = b.alloc_scratch_vreg(REG_DOT_ACC);
                b.insts.push(RawInst::new(VmOp::FSin, tmp, s, 0, 0));
                b.emit_dst_write(d_sin, c, tmp, inst.saturate);
            }
            if w_cos {
                let tmp = b.alloc_scratch_vreg(REG_DOT_ACC);
                b.insts.push(RawInst::new(VmOp::FCos, tmp, s, 0, 0));
                b.emit_dst_write(d_cos, c, tmp, inst.saturate);
            }
        }
        return;
    }

    if matches!(inst.opcode, 69..=74 | 45 | 61 | 110) && inst.operands.len() >= 3 {
        let dst = &inst.operands[0];
        let mask = dst.mask & 0x0F;
        if mask == 0 {
            return;
        }
        let is_int_coord = inst.opcode == 45 || inst.opcode == 61;
        let u_reg = b.resolve_src(&inst.operands[1], 0, REG_MOD0, is_int_coord);
        let v_reg = b.resolve_src(&inst.operands[1], 1, REG_MOD1, is_int_coord);
        let dst_base = match dst.op_type {
            0 => {
                let r = (dst.indices.first().copied().unwrap_or(0) as usize).min(63);
                if b.use_ssa {
                    let base = b.alloc_vreg(4);
                    for (c, slot) in b.ssa_map[r].iter_mut().enumerate() {
                        if (mask >> c) & 1 != 0 {
                            *slot = base + (c as u16);
                        }
                    }
                    base
                } else {
                    b.ssa_map[r][0]
                }
            }
            2 if dst.indices.first().copied().unwrap_or(0) == 0 => u16::from(REG_OUT_R),
            _ => return,
        };
        let tex_op = &inst.operands[2];
        let t_idx = (tex_op.indices.first().copied().unwrap_or(0) & 0x0F) as u16;
        let swz = tex_op.swizzle;
        let swz_imm = ((swz[0] & 3) as u16)
            | (((swz[1] & 3) as u16) << 2)
            | (((swz[2] & 3) as u16) << 4)
            | (((swz[3] & 3) as u16) << 6)
            | if inst.saturate { 1 << 8 } else { 0 };
        let vm_op = match inst.opcode {
            45 => VmOp::TexLoad2D,
            61 => VmOp::ResInfo2D,
            110 => VmOp::Gather4,
            _ => VmOp::Sample2D,
        };
        b.insts.push(RawInst::with_aux_imm(
            vm_op, dst_base, u_reg, v_reg, t_idx, mask, swz_imm,
        ));
    }
}

fn lower_standard_alu(b: &mut CompilerBuilder<'_>, inst: &Sm5Instruction) {
    assert!(inst.operands.len() >= 2);
    let dst = &inst.operands[0];
    let mask = dst.mask & 0x0F;
    if mask == 0 {
        return;
    }
    let Some((base_op, is_int)) = map_alu_op(inst.opcode) else {
        return;
    };

    // Snapshot SSA source mapping before writing any destination channels so cross-channel
    // self-swizzles (`r0.xy = r0.yx * ...`) always read the pre-instruction channels!
    let mut results = [(0u8, 0u16); 4];
    let mut num_results = 0usize;

    for c in 0..4usize {
        if (mask >> c) & 1 == 0 {
            continue;
        }
        let mut vm_op = base_op;
        let s0 = b.resolve_src(&inst.operands[1], c, REG_MOD0, is_int);
        let s1 = if let Some(op2) = inst.operands.get(2) {
            if inst.opcode == 0 && op2.neg && !op2.abs && op2.op_type != 4 && op2.op_type != 5 {
                vm_op = VmOp::FSub;
                b.resolve_raw_src(op2, c, REG_MOD1)
            } else {
                b.resolve_src(op2, c, REG_MOD1, is_int)
            }
        } else {
            u16::from(REG_ZERO)
        };
        let s2 = inst.operands.get(3).map_or(u16::from(REG_ZERO), |op3| {
            b.resolve_src(op3, c, REG_MOD2, is_int)
        });

        if b.use_ssa && dst.op_type == 2 && dst.indices.first().copied().unwrap_or(0) == 0 {
            let out_reg = u16::from(REG_OUT_R + (c as u8));
            if !inst.saturate {
                b.insts.push(RawInst::new(vm_op, out_reg, s0, s1, s2));
            } else if vm_op == VmOp::Mov {
                b.insts.push(RawInst::new(VmOp::MovSat, out_reg, s0, 0, 0));
            } else {
                let v = b.alloc_vreg(1);
                b.insts.push(RawInst::new(vm_op, v, s0, s1, s2));
                b.insts.push(RawInst::new(VmOp::MovSat, out_reg, v, 0, 0));
            }
            continue;
        }

        let val_vreg = if b.use_ssa {
            let v = b.alloc_vreg(1);
            b.insts.push(RawInst::new(vm_op, v, s0, s1, s2));
            if inst.saturate {
                let sat_v = b.alloc_vreg(1);
                b.insts.push(RawInst::new(VmOp::MovSat, sat_v, v, 0, 0));
                sat_v
            } else {
                v
            }
        } else {
            let tmp = u16::from(REG_ALIAS_TMP0 + (c as u8));
            b.insts.push(RawInst::new(vm_op, tmp, s0, s1, s2));
            tmp
        };
        results[num_results] = (c as u8, val_vreg);
        num_results += 1;
    }

    for &(c, val_vreg) in &results[..num_results] {
        if b.use_ssa && dst.op_type == 0 {
            let r = (dst.indices.first().copied().unwrap_or(0) as usize).min(63);
            b.ssa_map[r][c as usize] = val_vreg;
        } else {
            b.emit_dst_write(dst, c as usize, val_vreg, !b.use_ssa && inst.saturate);
        }
    }
}

fn lower_control_flow(b: &mut CompilerBuilder<'_>, inst: &Sm5Instruction) -> bool {
    assert!(inst.opcode < 256);
    let cond_reg = |b: &mut CompilerBuilder<'_>| -> u16 {
        inst.operands.first().map_or(u16::from(REG_ZERO), |op| {
            b.resolve_src(op, 0, REG_MOD0, true)
        })
    };
    let nz = inst.test_nonzero;
    match inst.opcode {
        31 => {
            let c = cond_reg(b);
            let op = if nz { VmOp::IfNz } else { VmOp::IfZ };
            b.insts.push(RawInst::new(op, 0, c, 0, 0));
        }
        18 => b.insts.push(RawInst::new(VmOp::Else, 0, 0, 0, 0)),
        21 => b.insts.push(RawInst::new(VmOp::EndIf, 0, 0, 0, 0)),
        48 => b.insts.push(RawInst::new(VmOp::Loop, 0, 0, 0, 0)),
        22 => b.insts.push(RawInst::new(VmOp::EndLoop, 0, 0, 0, 0)),
        2 => b.insts.push(RawInst::new(VmOp::Break, 0, 0, 0, 0)),
        3 => {
            let c = cond_reg(b);
            let op = if nz { VmOp::BreakNz } else { VmOp::BreakZ };
            b.insts.push(RawInst::new(op, 0, c, 0, 0));
        }
        7 => b.insts.push(RawInst::new(VmOp::Continue, 0, 0, 0, 0)),
        8 => {
            let c = cond_reg(b);
            let op = if nz {
                VmOp::ContinueNz
            } else {
                VmOp::ContinueZ
            };
            b.insts.push(RawInst::new(op, 0, c, 0, 0));
        }
        13 => {
            let c = cond_reg(b);
            let op = if nz { VmOp::DiscardNz } else { VmOp::DiscardZ };
            b.insts.push(RawInst::new(op, 0, c, 0, 0));
        }
        62 => b.insts.push(RawInst::new(VmOp::Ret, 0, 0, 0, 0)),
        63 => {
            let c = cond_reg(b);
            let op = if nz { VmOp::RetNz } else { VmOp::RetZ };
            b.insts.push(RawInst::new(op, 0, c, 0, 0));
        }
        _ => return false,
    }
    true
}

fn eval_const_op(op: VmOp, ua: u32, ub: u32, uc: u32) -> Option<u32> {
    let fa = f32::from_bits(ua);
    let fb = f32::from_bits(ub);
    let fc = f32::from_bits(uc);
    let clean = |v: f32| -> u32 {
        if v.is_nan() {
            0.0f32.to_bits()
        } else {
            v.to_bits()
        }
    };
    let res = match op {
        VmOp::Mov => ua,
        VmOp::MovSat => clean(fa.clamp(0.0, 1.0)),
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
        VmOp::FAdd => clean(fa + fb),
        VmOp::FSub => clean(fa - fb),
        VmOp::FMul => clean(fa * fb),
        VmOp::FMad => clean(fa * fb + fc),
        VmOp::FDiv => clean(if fb.abs() > 1e-20 { fa / fb } else { 0.0 }),
        VmOp::FMin => clean(fa.min(fb)),
        VmOp::FMax => clean(fa.max(fb)),
        VmOp::FFrc => clean(fa - fa.floor()),
        VmOp::FRoundNe => clean(fa.round_ties_even()),
        VmOp::FFloor => clean(fa.floor()),
        VmOp::FCeil => clean(fa.ceil()),
        VmOp::FTrunc => clean(fa.trunc()),
        VmOp::FSqrt => clean(fa.max(0.0).sqrt()),
        VmOp::FRsq => clean(if fa > 1e-20 { 1.0 / fa.sqrt() } else { 0.0 }),
        VmOp::FRcp => clean(if fa.abs() > 1e-20 { 1.0 / fa } else { 0.0 }),
        VmOp::FExp2 => clean(fa.exp2()),
        VmOp::FLog2 => clean(fa.max(1.175_494_4e-38).log2()),
        VmOp::LinearToSrgb => clean(super::texture::linear_to_srgb_channel(fa)),
        VmOp::FSin => clean(fa.sin()),
        VmOp::FCos => clean(fa.cos()),
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
        VmOp::BitAnd => ua & ub,
        VmOp::BitOr => ua | ub,
        VmOp::BitXor => ua ^ ub,
        _ => return None,
    };
    assert!(res == 0 || res != 0);
    Some(res)
}

fn is_zero_bits(c: Option<u32>) -> bool {
    c.is_some_and(|b| b == 0 || b == 0x8000_0000)
}

fn is_one_bits(c: Option<u32>) -> bool {
    c == Some(0x3F80_0000)
}

fn is_approx_f32(c: Option<u32>, target: f32, tol: f32) -> bool {
    c.is_some_and(|b| (f32::from_bits(b) - target).abs() <= tol)
}

fn try_fold_movc_patterns(
    inst: &mut RawInst,
    c0: Option<u32>,
    c1: Option<u32>,
    known_const: &[Option<u32>],
    is_non_neg: &[bool],
    def_inst: &[Option<RawInst>],
) {
    assert!((inst.src0 as usize) < def_inst.len());
    if c0 == Some(0) {
        *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src2, 0, 0);
        return;
    }
    if c0.is_some_and(|v| v != 0) || inst.src1 == inst.src2 {
        *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src1, 0, 0);
        return;
    }
    let Some(cond_inst) = def_inst[inst.src0 as usize] else {
        return;
    };
    if cond_inst.op != VmOp::FCmpGe {
        return;
    }
    // Pattern 1: UE4 `0.0 >= x ? 0.0 : max(x, 0.0)` where `else` branch is already >= 0.0.
    if is_zero_bits(known_const[cond_inst.src0 as usize])
        && is_zero_bits(c1)
        && is_non_neg[inst.src2 as usize]
    {
        *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src2, 0, 0);
        return;
    }
    // Pattern 2: UE4 `LinearToSrgbBranching`: `x >= 0.0031308 ? (1.055 * x^(1/2.4) - 0.055) : (x * 12.92)`.
    if is_approx_f32(known_const[cond_inst.src1 as usize], 0.003_130_8, 1e-4)
        && let Some(hi_inst) = def_inst[inst.src1 as usize]
        && let Some(lo_inst) = def_inst[inst.src2 as usize]
        && hi_inst.op == VmOp::FMad
        && lo_inst.op == VmOp::FMul
        && is_approx_f32(known_const[hi_inst.src1 as usize], 1.055, 1e-3)
        && is_approx_f32(known_const[hi_inst.src2 as usize], -0.055, 1e-3)
        && (is_approx_f32(known_const[lo_inst.src0 as usize], 12.92, 1e-2)
            || is_approx_f32(known_const[lo_inst.src1 as usize], 12.92, 1e-2))
    {
        *inst = RawInst::new(VmOp::LinearToSrgb, inst.dst, cond_inst.src0, 0, 0);
    }
}

#[allow(clippy::too_many_arguments)]
fn simplify_algebraic_peephole(
    inst: &mut RawInst,
    c0: Option<u32>,
    c1: Option<u32>,
    c2: Option<u32>,
    known_const: &[Option<u32>],
    is_non_neg: &[bool],
    def_inst: &[Option<RawInst>],
    log2_arg: &[Option<u16>],
) {
    assert!(inst.src0 as usize <= log2_arg.len());
    let z = u16::from(REG_ZERO);
    match inst.op {
        VmOp::Movc => {
            try_fold_movc_patterns(inst, c0, c1, known_const, is_non_neg, def_inst);
        }
        VmOp::FMax => {
            if is_zero_bits(c1) && is_non_neg[inst.src0 as usize] {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src0, 0, 0);
            } else if is_zero_bits(c0) && is_non_neg[inst.src1 as usize] {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src1, 0, 0);
            }
        }
        VmOp::FMul => {
            if is_zero_bits(c0) || is_zero_bits(c1) {
                *inst = RawInst::new(VmOp::Mov, inst.dst, z, 0, 0);
            } else if is_one_bits(c0) {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src1, 0, 0);
            } else if is_one_bits(c1) {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src0, 0, 0);
            }
        }
        VmOp::FAdd => {
            if is_zero_bits(c0) {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src1, 0, 0);
            } else if is_zero_bits(c1) {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src0, 0, 0);
            }
        }
        VmOp::FSub => {
            if is_zero_bits(c1) {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src0, 0, 0);
            } else if is_zero_bits(c0) {
                *inst = RawInst::new(VmOp::FNeg, inst.dst, inst.src1, 0, 0);
            }
        }
        VmOp::FMad => {
            if is_zero_bits(c0) || is_zero_bits(c1) {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src2, 0, 0);
            } else if is_zero_bits(c2) {
                if is_one_bits(c0) {
                    *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src1, 0, 0);
                } else if is_one_bits(c1) {
                    *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src0, 0, 0);
                } else {
                    *inst = RawInst::new(VmOp::FMul, inst.dst, inst.src0, inst.src1, 0);
                }
            } else if is_one_bits(c0) {
                *inst = RawInst::new(VmOp::FAdd, inst.dst, inst.src1, inst.src2, 0);
            } else if is_one_bits(c1) {
                *inst = RawInst::new(VmOp::FAdd, inst.dst, inst.src0, inst.src2, 0);
            }
        }
        VmOp::FExp2 => {
            if let Some(orig_x) = log2_arg[inst.src0 as usize] {
                if is_non_neg[orig_x as usize] {
                    *inst = RawInst::new(VmOp::Mov, inst.dst, orig_x, 0, 0);
                } else {
                    *inst = RawInst::new(VmOp::FMax, inst.dst, orig_x, z, 0);
                }
            } else if let Some(mul_inst) = def_inst[inst.src0 as usize]
                && mul_inst.op == VmOp::FMul
            {
                let (l_reg, exp_c) = if log2_arg[mul_inst.src0 as usize].is_some() {
                    (mul_inst.src0, known_const[mul_inst.src1 as usize])
                } else {
                    (mul_inst.src1, known_const[mul_inst.src0 as usize])
                };
                if let Some(orig_x) = log2_arg[l_reg as usize] {
                    if is_approx_f32(exp_c, 2.0, 1e-5) {
                        *inst = RawInst::new(VmOp::FMul, inst.dst, orig_x, orig_x, 0);
                    } else if is_approx_f32(exp_c, 0.5, 1e-5) {
                        *inst = RawInst::new(VmOp::FSqrt, inst.dst, orig_x, 0, 0);
                    }
                }
            }
        }
        _ => {}
    }
}

fn eval_non_neg_op(op: VmOp, n0: bool, n1: bool, n2: bool) -> bool {
    match op {
        VmOp::Mov => n0,
        VmOp::MovSat
        | VmOp::FAbs
        | VmOp::FSqrt
        | VmOp::FRsq
        | VmOp::FExp2
        | VmOp::FFrc
        | VmOp::LinearToSrgb => true,
        VmOp::FMax => n0 || n1,
        VmOp::FMin | VmOp::FAdd | VmOp::FMul => n0 && n1,
        VmOp::FMad => n0 && n1 && n2,
        VmOp::Movc => n1 && n2,
        _ => false,
    }
}

fn is_pure_ssa_alu(op: VmOp) -> bool {
    !is_control_flow_or_side_effect(op)
        && !matches!(
            op,
            VmOp::Sample2D
                | VmOp::TexLoad2D
                | VmOp::ResInfo2D
                | VmOp::Gather4
                | VmOp::LoadCb2Dyn
                | VmOp::LoadIcbDyn
                | VmOp::LoadIdxTemp
                | VmOp::Bfi
        )
}

fn fold_ssa_constants_and_cse(b: &mut CompilerBuilder<'_>) {
    assert!(b.next_vreg >= u16::from(REG_DYNAMIC_BASE));
    if !b.use_ssa {
        return;
    }
    let cap = (b.next_vreg as usize) + b.insts.len() + 16;
    let mut known_const: Vec<Option<u32>> = vec![None; cap];
    let mut is_non_neg: Vec<bool> = vec![false; cap];
    let mut def_inst: Vec<Option<RawInst>> = vec![None; cap];
    let mut alias: Vec<u16> = (0..cap as u16).collect();
    let mut log2_arg: Vec<Option<u16>> = vec![None; cap];

    known_const[REG_ZERO as usize] = Some(0);
    known_const[REG_ONE as usize] = Some(0x3F80_0000);
    known_const[REG_TRUE as usize] = Some(0xFFFF_FFFF);
    known_const[REG_V0_W as usize] = Some(0x3F80_0000);
    if b.static_cb2.is_some() {
        known_const[REG_REAL_TIME as usize] = Some(0);
        is_non_neg[REG_REAL_TIME as usize] = true;
    }
    for r in [REG_ZERO, REG_ONE, REG_V0_W]
        .into_iter()
        .chain(REG_V1_R..=REG_V1_R + 3)
        .chain(REG_V3_U..=REG_V4_X + 3)
    {
        is_non_neg[r as usize] = true;
    }
    for cb in &b.const_bindings {
        known_const[cb.vreg as usize] = Some(cb.bits);
        is_non_neg[cb.vreg as usize] = (cb.bits & 0x8000_0000) == 0 || cb.bits == 0x8000_0000;
    }

    let raw_insts = std::mem::take(&mut b.insts);
    let mut out_insts: Vec<RawInst> = Vec::with_capacity(raw_insts.len());
    let mut cse_table: Vec<(VmOp, u16, u16, u16, u16)> = Vec::with_capacity(raw_insts.len());

    for mut inst in raw_insts {
        inst.src0 = alias[inst.src0 as usize];
        inst.src1 = alias[inst.src1 as usize];
        if !matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4 | VmOp::LoadIdxTemp
        ) {
            inst.src2 = alias[inst.src2 as usize];
        } else if matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        ) {
            for c in 0..4usize {
                is_non_neg[(inst.dst as usize) + c] = true;
            }
        }

        if is_pure_ssa_alu(inst.op) {
            let c0 = known_const[inst.src0 as usize];
            let c1 = known_const[inst.src1 as usize];
            let c2 = known_const[inst.src2 as usize];

            if let (Some(u0), Some(u1), Some(u2)) = (c0, c1, c2)
                && let Some(bits) = eval_const_op(inst.op, u0, u1, u2)
            {
                let c_vreg = b.alloc_const(bits);
                known_const[c_vreg as usize] = Some(bits);
                is_non_neg[c_vreg as usize] = (bits & 0x8000_0000) == 0 || bits == 0x8000_0000;
                inst = RawInst::new(VmOp::Mov, inst.dst, c_vreg, 0, 0);
            } else {
                simplify_algebraic_peephole(
                    &mut inst,
                    c0,
                    c1,
                    c2,
                    &known_const,
                    &is_non_neg,
                    &def_inst,
                    &log2_arg,
                );
            }

            if inst.op == VmOp::Mov && inst.dst >= u16::from(REG_DYNAMIC_BASE) {
                let target = alias[inst.src0 as usize];
                alias[inst.dst as usize] = target;
                known_const[inst.dst as usize] = known_const[target as usize];
                is_non_neg[inst.dst as usize] = is_non_neg[target as usize];
                def_inst[inst.dst as usize] = def_inst[target as usize];
                log2_arg[inst.dst as usize] = log2_arg[target as usize];
                continue;
            }

            if inst.dst >= u16::from(REG_DYNAMIC_BASE)
                && let Some(&(_, _, _, _, prev_dst)) =
                    cse_table.iter().find(|&&(op, s0, s1, s2, _)| {
                        op == inst.op && s0 == inst.src0 && s1 == inst.src1 && s2 == inst.src2
                    })
            {
                alias[inst.dst as usize] = prev_dst;
                known_const[inst.dst as usize] = known_const[prev_dst as usize];
                is_non_neg[inst.dst as usize] = is_non_neg[prev_dst as usize];
                def_inst[inst.dst as usize] = def_inst[prev_dst as usize];
                log2_arg[inst.dst as usize] = log2_arg[prev_dst as usize];
                continue;
            }

            is_non_neg[inst.dst as usize] = eval_non_neg_op(
                inst.op,
                is_non_neg[inst.src0 as usize],
                is_non_neg[inst.src1 as usize],
                is_non_neg[inst.src2 as usize],
            );
            def_inst[inst.dst as usize] = Some(inst);
            if inst.op == VmOp::FLog2 {
                log2_arg[inst.dst as usize] = Some(inst.src0);
            }
            if inst.dst >= u16::from(REG_DYNAMIC_BASE) {
                cse_table.push((inst.op, inst.src0, inst.src1, inst.src2, inst.dst));
            }
        }
        out_insts.push(inst);
    }
    assert!(out_insts.len() <= MAX_STAGE_INSTS * 4);
    b.insts = out_insts;
}

fn is_control_flow_or_side_effect(op: VmOp) -> bool {
    matches!(
        op,
        VmOp::IfZ
            | VmOp::IfNz
            | VmOp::Else
            | VmOp::EndIf
            | VmOp::Loop
            | VmOp::EndLoop
            | VmOp::Break
            | VmOp::BreakZ
            | VmOp::BreakNz
            | VmOp::Continue
            | VmOp::ContinueZ
            | VmOp::ContinueNz
            | VmOp::DiscardZ
            | VmOp::DiscardNz
            | VmOp::EarlyOutZero
            | VmOp::Ret
            | VmOp::RetZ
            | VmOp::RetNz
            | VmOp::StoreIdxTemp
    )
}

fn eliminate_dead_code(insts: Vec<RawInst>, num_vregs: usize, has_loops: bool) -> Vec<RawInst> {
    assert!(insts.len() <= MAX_STAGE_INSTS * 4);
    if has_loops {
        return insts;
    }
    let mut live = vec![false; num_vregs + 8];
    live[REG_OUT_R as usize] = true;
    live[REG_OUT_G as usize] = true;
    live[REG_OUT_B as usize] = true;
    live[REG_OUT_A as usize] = true;

    let mut cf_depth = 0usize;
    let mut kept_rev = Vec::with_capacity(insts.len());

    for mut inst in insts.into_iter().rev() {
        match inst.op {
            VmOp::EndIf | VmOp::EndLoop => {
                cf_depth += 1;
                kept_rev.push(inst);
                continue;
            }
            VmOp::IfZ | VmOp::IfNz | VmOp::Loop => {
                cf_depth = cf_depth.saturating_sub(1);
                live[inst.src0 as usize] = true;
                kept_rev.push(inst);
                continue;
            }
            _ if is_control_flow_or_side_effect(inst.op) => {
                live[inst.src0 as usize] = true;
                live[inst.src1 as usize] = true;
                kept_rev.push(inst);
                continue;
            }
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4 => {
                let mut narrowed_mask = 0u8;
                for c in 0..4u8 {
                    let reg = (inst.dst as usize) + (c as usize);
                    if (inst.aux >> c) & 1 != 0 && live[reg] {
                        narrowed_mask |= 1 << c;
                        if cf_depth == 0 {
                            live[reg] = false;
                        }
                    }
                }
                if narrowed_mask != 0 {
                    inst.aux = narrowed_mask;
                    live[inst.src0 as usize] = true;
                    live[inst.src1 as usize] = true;
                    kept_rev.push(inst);
                }
                continue;
            }
            _ => {}
        }

        let d = inst.dst as usize;
        if !live[d] {
            continue;
        }
        if cf_depth == 0 {
            live[d] = false;
        }
        live[inst.src0 as usize] = true;
        live[inst.src1 as usize] = true;
        live[inst.src2 as usize] = true;
        kept_rev.push(inst);
    }
    kept_rev.reverse();
    assert!(kept_rev.len() <= MAX_STAGE_INSTS * 4);
    kept_rev
}

fn split_preamble_and_kernel(
    insts: Vec<RawInst>,
    num_vregs: usize,
    use_ssa: bool,
) -> (Vec<RawInst>, Vec<RawInst>) {
    assert!(insts.len() <= MAX_STAGE_INSTS * 4);
    if !use_ssa {
        return (Vec::new(), insts);
    }
    let mut is_uniform = vec![true; num_vregs + 8];
    for r in [
        REG_OUT_R,
        REG_OUT_G,
        REG_OUT_B,
        REG_OUT_A,
        REG_V0_X,
        REG_V0_X + 1,
        REG_V2_X,
        REG_V2_X + 1,
        REG_V3_U,
        REG_V3_U + 1,
        REG_V4_X,
        REG_V4_X + 1,
        REG_V4_X + 2,
        REG_V4_X + 3,
    ] {
        is_uniform[r as usize] = false;
    }

    let mut preamble = Vec::new();
    let mut kernel = Vec::with_capacity(insts.len());

    for inst in insts {
        let is_tex = matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        );
        let is_deriv = matches!(inst.op, VmOp::FDdx | VmOp::FDdy);
        let can_hoist = !is_control_flow_or_side_effect(inst.op)
            && !is_tex
            && !is_deriv
            && inst.dst >= u16::from(REG_DYNAMIC_BASE)
            && is_uniform[inst.src0 as usize]
            && is_uniform[inst.src1 as usize]
            && is_uniform[inst.src2 as usize];

        if can_hoist {
            is_uniform[inst.dst as usize] = true;
            preamble.push(inst);
        } else {
            if is_tex {
                for c in 0..4u16 {
                    if (inst.aux >> c) & 1 != 0 {
                        is_uniform[(inst.dst + c) as usize] = false;
                    }
                }
            } else if inst.dst != u16::from(REG_NULL) {
                is_uniform[inst.dst as usize] = false;
            }
            kernel.push(inst);
        }
    }
    coalesce_kernel_output_moves(&mut kernel, num_vregs);
    hoist_zero_annihilator_mask(&mut kernel, num_vregs);
    assert!(kernel.len() <= MAX_STAGE_INSTS * 4);
    (preamble, kernel)
}

fn find_root_zero_kill_vreg(kernel: &[RawInst], def_inst: &[Option<RawInst>]) -> Option<u16> {
    assert!(def_inst.len() >= 8);
    let mut out_defs = [None; 5];
    for inst in kernel {
        if (1..=4).contains(&inst.dst) {
            out_defs[inst.dst as usize] = Some(*inst);
        }
    }
    if let Some(a_inst) = out_defs[REG_OUT_A as usize] {
        if a_inst.op == VmOp::MovSat && a_inst.src0 >= u16::from(REG_DYNAMIC_BASE) {
            return Some(a_inst.src0);
        }
        if !(a_inst.op == VmOp::Mov && a_inst.src0 == u16::from(REG_ZERO)) {
            return None;
        }
    }
    let mut rgb_mul = [None; 3];
    for c in 0..3usize {
        let o_inst = out_defs[c + 1]?;
        let lin_vreg = match o_inst.op {
            VmOp::LinearToSrgb | VmOp::Mov | VmOp::MovSat => o_inst.src0,
            _ => return None,
        };
        let m_inst = (*def_inst.get(lin_vreg as usize)?)?;
        if m_inst.op != VmOp::FMul {
            return None;
        }
        rgb_mul[c] = Some((m_inst.src0, m_inst.src1));
    }
    let (a0, b0) = rgb_mul[0]?;
    let (a1, b1) = rgb_mul[1]?;
    let (a2, b2) = rgb_mul[2]?;
    if a0 >= u16::from(REG_DYNAMIC_BASE) && (a0 == a1 || a0 == b1) && (a0 == a2 || a0 == b2) {
        Some(a0)
    } else if b0 >= u16::from(REG_DYNAMIC_BASE) && (b0 == a1 || b0 == b1) && (b0 == a2 || b0 == b2)
    {
        Some(b0)
    } else {
        None
    }
}

fn hoist_zero_annihilator_mask(kernel: &mut Vec<RawInst>, num_vregs: usize) {
    assert!(kernel.len() <= MAX_STAGE_INSTS * 4);
    let mut def_inst: Vec<Option<RawInst>> = vec![None; num_vregs + 8];
    for inst in kernel.iter() {
        if is_pure_ssa_alu(inst.op) && inst.dst >= u16::from(REG_DYNAMIC_BASE) {
            def_inst[inst.dst as usize] = Some(*inst);
        }
    }
    let Some(root_vreg) = find_root_zero_kill_vreg(kernel, &def_inst) else {
        return;
    };
    let mut zero_kills = vec![false; num_vregs + 8];
    zero_kills[root_vreg as usize] = true;
    for inst in kernel.iter().rev() {
        let d = inst.dst as usize;
        if d < zero_kills.len() && zero_kills[d] {
            match inst.op {
                VmOp::Mov
                | VmOp::MovSat
                | VmOp::LinearToSrgb
                | VmOp::FSqrt
                | VmOp::FExp2
                | VmOp::FLog2 => {
                    zero_kills[inst.src0 as usize] = true;
                }
                VmOp::FMul | VmOp::FMin => {
                    zero_kills[inst.src0 as usize] = true;
                    zero_kills[inst.src1 as usize] = true;
                }
                _ => {}
            }
        }
    }

    for sample_pc in (1..kernel.len()).rev() {
        let s_inst = kernel[sample_pc];
        if s_inst.op != VmOp::Sample2D || s_inst.dst < u16::from(REG_DYNAMIC_BASE) {
            continue;
        }
        let Some(kill_ch) = (0..4u16)
            .find(|&c| (s_inst.aux >> c) & 1 != 0 && zero_kills[(s_inst.dst + c) as usize])
        else {
            continue;
        };
        let mut need_reg = vec![false; num_vregs + 8];
        let mut in_slice = vec![false; kernel.len()];
        need_reg[s_inst.src0 as usize] = true;
        need_reg[s_inst.src1 as usize] = true;
        let mut valid = true;
        let mut slice_len = 0usize;
        for pc in (0..sample_pc).rev() {
            let inst = kernel[pc];
            let d = inst.dst as usize;
            if d >= REG_DYNAMIC_BASE as usize && need_reg[d] {
                if !is_pure_ssa_alu(inst.op) {
                    valid = false;
                    break;
                }
                in_slice[pc] = true;
                slice_len += 1;
                need_reg[inst.src0 as usize] = true;
                need_reg[inst.src1 as usize] = true;
                need_reg[inst.src2 as usize] = true;
            }
        }
        if valid && slice_len <= 16 {
            let mut reordered = Vec::with_capacity(kernel.len() + 1);
            for (pc, &inst) in kernel.iter().enumerate() {
                if in_slice[pc] {
                    reordered.push(inst);
                }
            }
            reordered.push(s_inst);
            reordered.push(RawInst::new(
                VmOp::EarlyOutZero,
                0,
                s_inst.dst + kill_ch,
                0,
                0,
            ));
            for (pc, &inst) in kernel.iter().enumerate() {
                if !in_slice[pc] && pc != sample_pc {
                    reordered.push(inst);
                }
            }
            *kernel = reordered;
            return;
        }
    }
}

fn coalesce_kernel_output_moves(kernel: &mut Vec<RawInst>, num_vregs: usize) {
    assert!(kernel.len() <= MAX_STAGE_INSTS * 4);
    let mut read_count = vec![0u16; num_vregs + 8];
    let mut def_pc: Vec<Option<usize>> = vec![None; num_vregs + 8];

    for (pc, inst) in kernel.iter().enumerate() {
        read_count[inst.src0 as usize] = read_count[inst.src0 as usize].saturating_add(1);
        read_count[inst.src1 as usize] = read_count[inst.src1 as usize].saturating_add(1);
        if !matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        ) {
            read_count[inst.src2 as usize] = read_count[inst.src2 as usize].saturating_add(1);
        }
        if is_pure_ssa_alu(inst.op) && inst.dst >= u16::from(REG_DYNAMIC_BASE) {
            def_pc[inst.dst as usize] = Some(pc);
        }
    }

    let mut drop_inst = vec![false; kernel.len()];
    for pc in 0..kernel.len() {
        let inst = kernel[pc];
        if inst.op == VmOp::Mov
            && (u16::from(REG_OUT_R)..=u16::from(REG_OUT_A)).contains(&inst.dst)
            && inst.src0 >= u16::from(REG_DYNAMIC_BASE)
            && read_count[inst.src0 as usize] == 1
            && let Some(d_pc) = def_pc[inst.src0 as usize]
        {
            kernel[d_pc].dst = inst.dst;
            drop_inst[pc] = true;
        }
    }

    let mut idx = 0usize;
    kernel.retain(|_| {
        let keep = !drop_inst[idx];
        idx += 1;
        keep
    });
}

fn map_raw_to_vm_inst(inst: &RawInst, phys: &[u8]) -> VmInst {
    assert!((inst.dst as usize) < phys.len());
    let is_tex = matches!(
        inst.op,
        VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
    );
    let dst = if is_control_flow_or_side_effect(inst.op) && inst.op != VmOp::StoreIdxTemp {
        0
    } else if inst.op == VmOp::StoreIdxTemp {
        inst.dst as u8
    } else {
        phys[inst.dst as usize]
    };
    let src0 = phys[inst.src0 as usize];
    let src1 = if matches!(
        inst.op,
        VmOp::LoadCb2Dyn | VmOp::LoadIcbDyn | VmOp::LoadIdxTemp
    ) {
        inst.src1 as u8
    } else {
        phys[inst.src1 as usize]
    };
    let src2 = if is_tex || matches!(inst.op, VmOp::LoadIdxTemp | VmOp::StoreIdxTemp) {
        inst.src2 as u8
    } else {
        phys[inst.src2 as usize]
    };
    assert_ne!(dst, 0xFF, "unmapped dst vreg {}", inst.dst);
    assert_ne!(src0, 0xFF, "unmapped src0 vreg {}", inst.src0);
    assert_ne!(src1, 0xFF, "unmapped src1 vreg {}", inst.src1);
    assert_ne!(src2, 0xFF, "unmapped src2 vreg {}", inst.src2);
    VmInst::with_aux_imm(inst.op, dst, src0, src1, src2, inst.aux, inst.imm16)
}

fn allocate_physical_regs(
    preamble_raw: &[RawInst],
    kernel_raw: &[RawInst],
    cb2_vregs: &[VregCb2Binding],
    const_vregs: &[VregConstBinding],
    num_vregs: usize,
    use_ssa: bool,
) -> (
    Vec<VmInst>,
    Vec<VmInst>,
    Vec<Cb2RegBinding>,
    Vec<ConstRegBinding>,
    usize,
    usize,
) {
    assert!(num_vregs >= REG_DYNAMIC_BASE as usize);
    let mut phys = vec![0xFFu8; num_vregs + 8];
    for (i, slot) in phys.iter_mut().enumerate().take(REG_DYNAMIC_BASE as usize) {
        *slot = i as u8;
    }

    if !use_ssa {
        for (i, slot) in phys
            .iter_mut()
            .enumerate()
            .take(num_vregs.min(REG_FILE_SIZE))
            .skip(REG_DYNAMIC_BASE as usize)
        {
            *slot = i as u8;
        }
        let used = num_vregs.min(REG_FILE_SIZE);
        let pre: Vec<VmInst> = preamble_raw
            .iter()
            .map(|r| map_raw_to_vm_inst(r, &phys))
            .collect();
        let ker: Vec<VmInst> = kernel_raw
            .iter()
            .map(|r| map_raw_to_vm_inst(r, &phys))
            .collect();
        let cb2 = cb2_vregs
            .iter()
            .map(|b| Cb2RegBinding {
                dst_reg: phys[b.vreg as usize],
                comp: b.comp,
                vec_idx: b.vec_idx,
            })
            .collect();
        let cst = const_vregs
            .iter()
            .map(|b| ConstRegBinding {
                dst_reg: phys[b.vreg as usize],
                bits: b.bits,
            })
            .collect();
        return (pre, ker, cb2, cst, used, used);
    }

    let mut read_in_pre = vec![false; num_vregs + 8];
    let mut read_in_ker = vec![false; num_vregs + 8];
    for inst in preamble_raw {
        read_in_pre[inst.src0 as usize] = true;
        read_in_pre[inst.src1 as usize] = true;
        read_in_pre[inst.src2 as usize] = true;
    }
    for inst in kernel_raw {
        read_in_ker[inst.src0 as usize] = true;
        read_in_ker[inst.src1 as usize] = true;
        if !matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        ) {
            read_in_ker[inst.src2 as usize] = true;
        }
    }

    let mut next_phys = REG_DYNAMIC_BASE as usize;
    let alloc_one = |next: &mut usize| -> u8 {
        let r = (*next).min(REG_FILE_SIZE - 1) as u8;
        if *next < REG_FILE_SIZE {
            *next += 1;
        }
        r
    };

    let mut const_out = Vec::new();
    for b in const_vregs {
        let v = b.vreg as usize;
        if read_in_pre[v] || read_in_ker[v] {
            let p = alloc_one(&mut next_phys);
            phys[v] = p;
            const_out.push(ConstRegBinding {
                dst_reg: p,
                bits: b.bits,
            });
        }
    }
    let mut cb2_out = Vec::new();
    for b in cb2_vregs {
        let v = b.vreg as usize;
        if read_in_pre[v] || read_in_ker[v] {
            let p = alloc_one(&mut next_phys);
            phys[v] = p;
            cb2_out.push(Cb2RegBinding {
                dst_reg: p,
                comp: b.comp,
                vec_idx: b.vec_idx,
            });
        }
    }
    for inst in preamble_raw {
        let v = inst.dst as usize;
        if v >= REG_DYNAMIC_BASE as usize && phys[v] == 0xFF {
            phys[v] = alloc_one(&mut next_phys);
        }
    }
    let num_uniform_regs = next_phys;

    // Allocate contiguous 4-register blocks for texture samples first, then linear-scan recycling
    // for all single-channel scalar temporaries in `kernel_raw`.
    for inst in kernel_raw {
        if matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        ) && inst.dst >= u16::from(REG_DYNAMIC_BASE)
        {
            for c in 0..4usize {
                phys[(inst.dst as usize) + c] = alloc_one(&mut next_phys);
            }
        }
    }

    let mut last_use = vec![0usize; num_vregs + 8];
    for (pc, inst) in kernel_raw.iter().enumerate() {
        last_use[inst.src0 as usize] = pc;
        last_use[inst.src1 as usize] = pc;
        if !matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        ) {
            last_use[inst.src2 as usize] = pc;
        }
    }

    let mut free_pool: Vec<u8> = Vec::new();
    let mut released = vec![false; num_vregs + 8];
    for (pc, inst) in kernel_raw.iter().enumerate() {
        let d = inst.dst as usize;
        if d >= REG_DYNAMIC_BASE as usize && phys[d] == 0xFF {
            phys[d] = free_pool.pop().unwrap_or_else(|| alloc_one(&mut next_phys));
        }
        for &src in &[inst.src0, inst.src1, inst.src2] {
            let s = src as usize;
            if s >= REG_DYNAMIC_BASE as usize
                && !released[s]
                && last_use[s] == pc
                && (phys[s] as usize) >= num_uniform_regs
            {
                // Do not recycle channels belonging to a 4-register texture sample block early
                // unless all uses of that channel have completed.
                released[s] = true;
                free_pool.push(phys[s]);
            }
        }
    }

    let pre: Vec<VmInst> = preamble_raw
        .iter()
        .map(|r| map_raw_to_vm_inst(r, &phys))
        .collect();
    let ker: Vec<VmInst> = kernel_raw
        .iter()
        .map(|r| map_raw_to_vm_inst(r, &phys))
        .collect();
    assert!(next_phys <= REG_FILE_SIZE);
    (
        pre,
        ker,
        cb2_out,
        const_out,
        num_uniform_regs,
        next_phys.max(REG_DYNAMIC_BASE as usize),
    )
}

fn link_control_flow_jumps(insts: &mut [VmInst]) {
    assert!(insts.len() <= u16::MAX as usize);
    let mut if_stack: Vec<(usize, Option<usize>)> = Vec::new();
    let mut loop_stack: Vec<(usize, usize, Vec<usize>, Vec<usize>)> = Vec::new();
    let mut cf_depth = 0usize;

    for pc in 0..insts.len() {
        match insts[pc].op {
            VmOp::IfZ | VmOp::IfNz => {
                if_stack.push((pc, None));
                cf_depth += 1;
            }
            VmOp::Else => {
                if let Some((if_pc, else_slot)) = if_stack.last_mut() {
                    insts[*if_pc].imm16 = pc as u16;
                    *else_slot = Some(pc);
                }
            }
            VmOp::EndIf => {
                cf_depth = cf_depth.saturating_sub(1);
                if let Some((if_pc, else_opt)) = if_stack.pop() {
                    if let Some(else_pc) = else_opt {
                        insts[else_pc].imm16 = (pc + 1) as u16;
                    } else {
                        insts[if_pc].imm16 = pc as u16;
                    }
                }
            }
            VmOp::Loop => {
                loop_stack.push((pc, cf_depth, Vec::new(), Vec::new()));
                cf_depth += 1;
            }
            VmOp::Break | VmOp::BreakZ | VmOp::BreakNz => {
                if let Some((_, loop_depth, breaks, _)) = loop_stack.last_mut() {
                    insts[pc].aux = *loop_depth as u8;
                    breaks.push(pc);
                }
            }
            VmOp::Continue | VmOp::ContinueZ | VmOp::ContinueNz => {
                if let Some((_, loop_depth, _, conts)) = loop_stack.last_mut() {
                    insts[pc].aux = *loop_depth as u8;
                    conts.push(pc);
                }
            }
            VmOp::EndLoop => {
                cf_depth = cf_depth.saturating_sub(1);
                if let Some((loop_pc, _, breaks, conts)) = loop_stack.pop() {
                    insts[pc].imm16 = (loop_pc + 1) as u16;
                    insts[loop_pc].imm16 = (pc + 1) as u16;
                    for b_pc in breaks {
                        insts[b_pc].imm16 = (pc + 1) as u16;
                    }
                    for c_pc in conts {
                        insts[c_pc].imm16 = pc as u16;
                    }
                }
            }
            _ => {}
        }
    }
    assert!(if_stack.is_empty() || !insts.is_empty());
}

/// Compiles a parsed `DxbcShader` into a two-stage scalarized `CompiledSm5Shader`, optionally
/// specializing static `cb2` parameters at compile time when `static_cb2` is provided.
pub fn compile_sm5_shader_with_cb2(
    shader: &DxbcShader,
    static_cb2: Option<&[[f32; 4]]>,
) -> CompiledSm5Shader {
    assert!(shader.instructions.len() <= 65_536);
    let mut builder = CompilerBuilder::new(shader, static_cb2);

    for inst in &shader.instructions {
        if matches!(inst.opcode, 88..=106 | 147..=162 | 58 | 23) {
            continue;
        }
        if lower_control_flow(&mut builder, inst) {
            continue;
        }
        match inst.opcode {
            15 => lower_dot_product(&mut builder, inst, 2),
            16 => lower_dot_product(&mut builder, inst, 3),
            17 => lower_dot_product(&mut builder, inst, 4),
            38 | 81 | 78 | 132 | 133 => lower_multi_dst_int(&mut builder, inst),
            69..=74 | 77 | 45 | 61 | 110 => lower_sincos_or_sample(&mut builder, inst),
            _ if inst.operands.len() >= 2 => lower_standard_alu(&mut builder, inst),
            _ => {}
        }
    }

    fold_ssa_constants_and_cse(&mut builder);
    let num_vregs = builder.next_vreg as usize;
    let dce_insts = eliminate_dead_code(
        builder.insts,
        num_vregs,
        builder.has_control_flow || builder.uses_indexable_temps,
    );
    let (preamble_raw, kernel_raw) =
        split_preamble_and_kernel(dce_insts, num_vregs, builder.use_ssa);
    let (preamble, mut kernel, cb2_bindings, const_bindings, num_uniform_regs, num_regs_used) =
        allocate_physical_regs(
            &preamble_raw,
            &kernel_raw,
            &builder.cb2_bindings,
            &builder.const_bindings,
            num_vregs,
            builder.use_ssa,
        );
    link_control_flow_jumps(&mut kernel);
    assert!(num_regs_used <= REG_FILE_SIZE);

    CompiledSm5Shader {
        preamble,
        kernel,
        cb2_bindings,
        const_bindings,
        icb_vec4s: shader.icb_vec4s.clone(),
        num_uniform_regs,
        num_regs_used,
        has_control_flow: builder.has_control_flow,
        uses_indexable_temps: builder.uses_indexable_temps,
    }
}

/// Compiles a parsed `DxbcShader` into a two-stage scalarized `CompiledSm5Shader` with dynamic `cb2`.
pub fn compile_sm5_shader(shader: &DxbcShader) -> CompiledSm5Shader {
    assert!(shader.instructions.len() <= 65_536);
    compile_sm5_shader_with_cb2(shader, None)
}

//! DXBC-to-Scalar-SIMT SSA Compiler, Constant/Peephole Folder, Common Subexpression Eliminator,
//! Dead-Code Eliminator, Uniform Preamble Splitter, and Linear-Scan Physical Register Allocator.

use tartarus_extract::{DxbcShader, Sm5Instruction, Sm5Operand};

use super::ir::{
    Cb2RegBinding, CompiledSm5Shader, ConstRegBinding, MAX_STAGE_INSTS, REG_ALIAS_TMP0,
    REG_DOT_ACC, REG_DYNAMIC_BASE, REG_FILE_SIZE, REG_MOD0, REG_MOD1, REG_MOD2, REG_NULL, REG_ONE,
    REG_OUT_A, REG_OUT_B, REG_OUT_G, REG_OUT_R, REG_REAL_TIME, REG_TRUE, REG_V0_W, REG_V0_X,
    REG_V1_R, REG_V2_X, REG_V3_U, REG_V4_X, REG_ZERO, ShaderSpecialization, TexSlotMeta, VmInst,
    VmOp,
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
    spec: ShaderSpecialization<'a>,
    insts: Vec<RawInst>,
    cb2_bindings: Vec<VregCb2Binding>,
    const_bindings: Vec<VregConstBinding>,
    ssa_map: [[u16; 4]; 64],
    v_input_map: [[u8; 4]; 8],
    color0_v_idx: Option<usize>,
    view_cb_slot: Option<usize>,
    material_cb_slot: Option<usize>,
    cb0_is_slate: bool,
    next_vreg: u16,
    use_ssa: bool,
    has_control_flow: bool,
    uses_indexable_temps: bool,
}

fn discover_cb_slots(shader: &DxbcShader) -> (Option<usize>, Option<usize>, bool) {
    assert!(shader.instructions.len() <= 65_536);
    let mut decls: Vec<(usize, usize)> = Vec::new();
    for inst in &shader.instructions {
        if inst.opcode == 89
            && let Some(op) = inst.operands.first()
            && op.op_type == 8
            && op.indices.len() >= 2
        {
            decls.push((op.indices[0] as usize, op.indices[1] as usize));
        }
    }
    if decls.is_empty() {
        return (Some(1), Some(2), true);
    }
    let cb0_len = decls
        .iter()
        .find(|&&(s, _)| s == 0)
        .map_or(4, |&(_, len)| len);
    let cb0_is_slate = cb0_len == 4;
    let view_cb_slot = decls
        .iter()
        .find(|&&(_, len)| len >= 142)
        .map(|&(slot, _)| slot);
    let max_cb_slot = decls.iter().map(|&(s, _)| s).max().unwrap_or(0);
    let material_cb_slot = decls
        .iter()
        .filter(|&&(slot, len)| {
            slot >= 1 && len != 143 && !(slot == 1 && len == 12 && max_cb_slot == 1 && cb0_len == 2)
        })
        .max_by_key(|&&(slot, _)| slot)
        .map(|&(slot, _)| slot);
    assert!(max_cb_slot <= 16);
    (view_cb_slot, material_cb_slot, cb0_is_slate)
}

fn build_v_input_map(shader: &DxbcShader) -> ([[u8; 4]; 8], Option<usize>) {
    assert!(shader.input_signature.len() <= 256);
    let default_map = [
        [REG_V0_X, REG_V0_X + 1, REG_ZERO, REG_V0_W],
        [REG_V1_R, REG_V1_R + 1, REG_V1_R + 2, REG_V1_R + 3],
        [REG_V2_X, REG_V2_X + 1, REG_ZERO, REG_ZERO],
        [REG_V3_U, REG_V3_U + 1, REG_ZERO, REG_ZERO],
        [REG_V4_X, REG_V4_X + 1, REG_V4_X + 2, REG_V4_X + 3],
        [REG_ZERO; 4],
        [REG_ZERO; 4],
        [REG_ZERO; 4],
    ];
    if shader.input_signature.is_empty() {
        return (default_map, Some(1));
    }
    let is_slate = shader
        .input_signature
        .iter()
        .any(|s| s.semantic_name == "ORIGINAL_POSITION");
    if is_slate {
        return (default_map, Some(1));
    }
    let mut map = [[REG_ZERO; 4]; 8];
    let mut color0_v_idx = None;
    for sig in &shader.input_signature {
        let r = sig.register as usize;
        if r >= 8 {
            continue;
        }
        match (sig.semantic_name.as_str(), sig.semantic_index) {
            ("SV_POSITION" | "SV_Position", _) => {
                map[r] = [REG_V0_X, REG_V0_X + 1, REG_ZERO, REG_V0_W];
            }
            ("COLOR", 0) => {
                map[r] = [REG_V1_R, REG_V1_R + 1, REG_V1_R + 2, REG_V1_R + 3];
                color0_v_idx = Some(r);
            }
            ("TEXCOORD", 0 | 3) => {
                map[r] = [REG_V4_X, REG_V4_X + 1, REG_V4_X + 2, REG_V4_X + 3];
            }
            ("TEXCOORD", 10) => {
                map[r] = [REG_ONE, REG_ZERO, REG_ZERO, REG_ZERO];
            }
            ("TEXCOORD", 11) => {
                map[r] = [REG_ZERO, REG_ZERO, REG_ONE, REG_ONE];
            }
            _ => {}
        }
    }
    assert!(map.len() == 8);
    (map, color0_v_idx)
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
    fn new(shader: &'a DxbcShader, spec: ShaderSpecialization<'a>) -> Self {
        assert!(shader.instructions.len() <= 65_536);
        let (has_control_flow, uses_indexable_temps) = scan_shader_hazards(shader);
        let (view_cb_slot, material_cb_slot, cb0_is_slate) = discover_cb_slots(shader);
        let (v_input_map, color0_v_idx) = build_v_input_map(shader);
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
            spec,
            insts: Vec::with_capacity(shader.instructions.len() * 3),
            cb2_bindings: Vec::new(),
            const_bindings: Vec::new(),
            ssa_map,
            v_input_map,
            color0_v_idx,
            view_cb_slot,
            material_cb_slot,
            cb0_is_slate,
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
        if let Some(cb2) = self.spec.static_cb2 {
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
            1 => {
                if Some(idx0) == self.color0_v_idx
                    && self.use_ssa
                    && let Some(v1) = self.spec.static_v1
                {
                    let val = v1[swz_c as usize];
                    let clean = if val.is_nan() { 0.0 } else { val };
                    return self.alloc_const(clean.to_bits());
                }
                u16::from(
                    self.v_input_map
                        .get(idx0)
                        .map_or(REG_ZERO, |m| m[swz_c as usize]),
                )
            }
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
        if Some(cb_slot) == self.material_cb_slot {
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
        } else if Some(cb_slot) == self.view_cb_slot && vec_idx == 142 && swz_c == 2 {
            u16::from(REG_REAL_TIME)
        } else if cb_slot == 0 && self.cb0_is_slate && vec_idx == 1 && (swz_c == 0 || swz_c == 3) {
            u16::from(REG_ONE)
        } else {
            u16::from(REG_ZERO)
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

fn lower_ssa_sample2d(
    b: &mut CompilerBuilder<'_>,
    inst: &Sm5Instruction,
    vm_op: VmOp,
    u_reg: u16,
    v_reg: u16,
    tex_op_idx: usize,
) {
    let dst = &inst.operands[0];
    let mask = dst.mask & 0x0F;
    let r = (dst.indices.first().copied().unwrap_or(0) as usize).min(63);
    let tex_op = &inst.operands[tex_op_idx];
    let raw_t_idx = (tex_op.indices.first().copied().unwrap_or(0) & 0x0F) as usize;
    let meta_opt = b.spec.tex_meta.and_then(|m| m.get(raw_t_idx)).copied();
    let t_idx = meta_opt.map_or(raw_t_idx as u16, |m| u16::from(m.canonical_slot));
    let swz = tex_op.swizzle;
    let base = b.alloc_vreg(4);
    let mut raw_ch_vreg = [base, base + 1, base + 2, base + 3];

    if !inst.saturate
        && matches!(vm_op, VmOp::Sample2D | VmOp::Gather4)
        && let Some(meta) = meta_opt
    {
        for raw_c in 0..4usize {
            if let Some(cv) = meta.const_val[raw_c] {
                let clean = if cv.is_nan() { 0.0 } else { cv };
                raw_ch_vreg[raw_c] = b.alloc_const(clean.to_bits());
            } else if raw_c > 0 && meta.eq_r[raw_c] {
                raw_ch_vreg[raw_c] = raw_ch_vreg[0];
            }
        }
    }
    let mut active_raw_mask = 0u8;
    for (c, slot) in b.ssa_map[r].iter_mut().enumerate() {
        if (mask >> c) & 1 != 0 {
            let src_c = (swz[c] & 3) as usize;
            let mapped = raw_ch_vreg[src_c];
            *slot = mapped;
            if mapped >= base && mapped < base + 4 {
                active_raw_mask |= 1 << (mapped - base);
            }
        }
    }
    assert!(active_raw_mask <= 0x0F);
    if active_raw_mask != 0 {
        let imm16 = 228u16 | if inst.saturate { 1 << 8 } else { 0 };
        b.insts.push(RawInst::with_aux_imm(
            vm_op,
            base,
            u_reg,
            v_reg,
            t_idx,
            active_raw_mask,
            imm16,
        ));
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

    if matches!(inst.opcode, 69..=74 | 45 | 61 | 110 | 167) {
        let tex_op_idx = if inst.opcode == 167 { 3 } else { 2 };
        if inst.operands.len() <= tex_op_idx {
            return;
        }
        let dst = &inst.operands[0];
        let mask = dst.mask & 0x0F;
        if mask == 0 {
            return;
        }
        let is_int_coord = matches!(inst.opcode, 45 | 61 | 167);
        let u_reg = b.resolve_src(&inst.operands[1], 0, REG_MOD0, is_int_coord);
        let v_reg = if inst.opcode == 167 {
            b.resolve_src(&inst.operands[2], 0, REG_MOD1, true)
        } else {
            b.resolve_src(&inst.operands[1], 1, REG_MOD1, is_int_coord)
        };
        let vm_op = match inst.opcode {
            45 | 167 => VmOp::TexLoad2D,
            61 => VmOp::ResInfo2D,
            110 => VmOp::Gather4,
            _ => VmOp::Sample2D,
        };
        if b.use_ssa && dst.op_type == 0 {
            lower_ssa_sample2d(b, inst, vm_op, u_reg, v_reg, tex_op_idx);
            return;
        }
        let dst_base = match dst.op_type {
            0 => b.ssa_map[(dst.indices.first().copied().unwrap_or(0) as usize).min(63)][0],
            2 if dst.indices.first().copied().unwrap_or(0) == 0 => u16::from(REG_OUT_R),
            _ => return,
        };
        let tex_op = &inst.operands[tex_op_idx];
        let t_idx = (tex_op.indices.first().copied().unwrap_or(0) & 0x0F) as u16;
        let swz = tex_op.swizzle;
        let swz_imm = ((swz[0] & 3) as u16)
            | (((swz[1] & 3) as u16) << 2)
            | (((swz[2] & 3) as u16) << 4)
            | (((swz[3] & 3) as u16) << 6)
            | if inst.saturate { 1 << 8 } else { 0 };
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
        VmOp::FMul3 => clean((fa * fb) * fc),
        VmOp::FMad => clean(fa * b_or_zero(fb) + fc),
        VmOp::FNmad => clean(fc - fa * b_or_zero(fb)),
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
        VmOp::FPow => clean((fa.max(1.175_494_4e-38).log2() * fb).exp2()),
        VmOp::FAtan2 => clean(super::texture::fxc_atan2_scalar(fa, fb)),
        VmOp::FHermite => clean((fa * fa) * (3.0 - (fa + fa))),
        VmOp::LinearToSrgb => clean(super::texture::linear_to_srgb_channel((fa * fb) * fc)),
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

#[inline]
fn b_or_zero(v: f32) -> f32 {
    if v.is_nan() { 0.0 } else { v }
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

/// Detects FXC's 29-instruction `atan2(y, x)` (`VectorToRadialValue`) sequence ending at `Movc`:
/// `cond = BitAnd(FCmpLt(FMin(x, y), -FMin(x, y)), FCmpGe(FMax(x, y), -FMax(x, y)))`,
/// `then = FNeg(angle)`, `else = angle`, where `angle = FAdd(BitAnd(FCmpLt(x, -x), -PI), atan_t)`.
fn try_match_fxc_atan2(
    inst: &RawInst,
    known_const: &[Option<u32>],
    def_inst: &[Option<RawInst>],
) -> Option<(u16, u16)> {
    assert_eq!(inst.op, VmOp::Movc);
    let neg_angle = def_inst[inst.src1 as usize]?;
    if neg_angle.op != VmOp::FNeg || neg_angle.src0 != inst.src2 {
        return None;
    }
    let cond = def_inst[inst.src0 as usize]?;
    if cond.op != VmOp::BitAnd {
        return None;
    }
    let cmp_lt = def_inst[cond.src0 as usize]?;
    let cmp_ge = def_inst[cond.src1 as usize]?;
    if cmp_lt.op != VmOp::FCmpLt || cmp_ge.op != VmOp::FCmpGe {
        return None;
    }
    let min_xy = def_inst[cmp_lt.src0 as usize]?;
    let max_xy = def_inst[cmp_ge.src0 as usize]?;
    if min_xy.op != VmOp::FMin || max_xy.op != VmOp::FMax {
        return None;
    }
    let (a, b) = (min_xy.src0, min_xy.src1);
    let angle = def_inst[inst.src2 as usize]?;
    if angle.op != VmOp::FAdd {
        return None;
    }
    for side in [angle.src0, angle.src1] {
        if let Some(and_pi) = def_inst[side as usize]
            && and_pi.op == VmOp::BitAnd
            && (is_approx_f32(
                known_const[and_pi.src0 as usize],
                -std::f32::consts::PI,
                1e-3,
            ) || is_approx_f32(
                known_const[and_pi.src1 as usize],
                -std::f32::consts::PI,
                1e-3,
            ))
        {
            for c_side in [and_pi.src0, and_pi.src1] {
                if let Some(x_lt) = def_inst[c_side as usize]
                    && x_lt.op == VmOp::FCmpLt
                {
                    let x_reg = x_lt.src0;
                    if x_reg == a {
                        return Some((b, a));
                    } else if x_reg == b {
                        return Some((a, b));
                    }
                }
            }
        }
    }
    None
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
    if let Some((y_reg, x_reg)) = try_match_fxc_atan2(inst, known_const, def_inst) {
        *inst = RawInst::new(VmOp::FAtan2, inst.dst, y_reg, x_reg, 0);
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
    {
        let one = u16::from(REG_ONE);
        *inst = RawInst::new(VmOp::LinearToSrgb, inst.dst, cond_inst.src0, one, one);
    }
}

fn try_match_hermite_arg(
    a: u16,
    b: u16,
    known_const: &[Option<u32>],
    def_inst: &[Option<RawInst>],
) -> Option<u16> {
    assert!((a as usize) < def_inst.len());
    assert!((b as usize) < def_inst.len());
    for (sq_reg, lin_reg) in [(a, b), (b, a)] {
        let Some(sq) = def_inst[sq_reg as usize] else {
            continue;
        };
        let Some(lin) = def_inst[lin_reg as usize] else {
            continue;
        };
        if sq.op == VmOp::FMul && sq.src0 == sq.src1 {
            let t = sq.src0;
            if lin.op == VmOp::FMad
                && is_approx_f32(known_const[lin.src2 as usize], 3.0, 1e-5)
                && ((lin.src0 == t && is_approx_f32(known_const[lin.src1 as usize], -2.0, 1e-5))
                    || (lin.src1 == t && is_approx_f32(known_const[lin.src0 as usize], -2.0, 1e-5)))
            {
                return Some(t);
            }
        }
    }
    None
}

fn try_cancel_neg_and_lerp(
    inst: &mut RawInst,
    def_inst: &[Option<RawInst>],
    cse_table: &[(VmOp, u16, u16, u16, u16)],
) {
    assert!((inst.src0 as usize) < def_inst.len());
    if inst.op == VmOp::FAdd {
        if let Some(d0) = def_inst[inst.src0 as usize]
            && d0.op == VmOp::FNeg
        {
            *inst = RawInst::new(VmOp::FSub, inst.dst, inst.src1, d0.src0, 0);
        } else if let Some(d1) = def_inst[inst.src1 as usize]
            && d1.op == VmOp::FNeg
        {
            *inst = RawInst::new(VmOp::FSub, inst.dst, inst.src0, d1.src0, 0);
        }
    } else if inst.op == VmOp::FMad {
        let neg_pair = if let Some(d0) = def_inst[inst.src0 as usize]
            && d0.op == VmOp::FNeg
        {
            Some((d0.src0, inst.src1))
        } else if let Some(d1) = def_inst[inst.src1 as usize]
            && d1.op == VmOp::FNeg
        {
            Some((inst.src0, d1.src0))
        } else {
            None
        };
        if let Some((a, b)) = neg_pair {
            if let Some(&(_, _, _, _, p)) = cse_table.iter().find(|&&(op, s0, s1, _, _)| {
                op == VmOp::FMul && ((s0 == a && s1 == b) || (s0 == b && s1 == a))
            }) {
                *inst = RawInst::new(VmOp::FSub, inst.dst, inst.src2, p, 0);
            } else {
                *inst = RawInst::new(VmOp::FNmad, inst.dst, a, b, inst.src2);
            }
        }
    }

    if inst.op == VmOp::FAdd {
        if let Some(d0) = def_inst[inst.src0 as usize]
            && d0.op == VmOp::FSub
            && d0.src1 == inst.src1
        {
            *inst = RawInst::new(VmOp::Mov, inst.dst, d0.src0, 0, 0);
        } else if let Some(d1) = def_inst[inst.src1 as usize]
            && d1.op == VmOp::FSub
            && d1.src1 == inst.src0
        {
            *inst = RawInst::new(VmOp::Mov, inst.dst, d1.src0, 0, 0);
        }
    } else if inst.op == VmOp::FSub && inst.src0 == inst.src1 {
        *inst = RawInst::new(VmOp::Mov, inst.dst, u16::from(REG_ZERO), 0, 0);
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
    is_unit: &[bool],
    def_inst: &[Option<RawInst>],
    log2_arg: &[Option<u16>],
) {
    assert!(inst.src0 as usize <= log2_arg.len());
    let z = u16::from(REG_ZERO);
    match inst.op {
        VmOp::MovSat => {
            if is_unit[inst.src0 as usize] {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src0, 0, 0);
            }
        }
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
        VmOp::FMin => {
            if is_unit[inst.src0 as usize] && c1.is_some_and(|b| f32::from_bits(b) >= 1.0) {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src0, 0, 0);
            } else if is_unit[inst.src1 as usize] && c0.is_some_and(|b| f32::from_bits(b) >= 1.0) {
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
            } else if let Some(t) =
                try_match_hermite_arg(inst.src0, inst.src1, known_const, def_inst)
            {
                *inst = RawInst::new(VmOp::FHermite, inst.dst, t, 0, 0);
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
        VmOp::FNmad => {
            if is_zero_bits(c0) || is_zero_bits(c1) {
                *inst = RawInst::new(VmOp::Mov, inst.dst, inst.src2, 0, 0);
            } else if is_one_bits(c0) {
                *inst = RawInst::new(VmOp::FSub, inst.dst, inst.src2, inst.src1, 0);
            } else if is_one_bits(c1) {
                *inst = RawInst::new(VmOp::FSub, inst.dst, inst.src2, inst.src0, 0);
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
                let (l_reg, exp_reg, exp_c) = if log2_arg[mul_inst.src0 as usize].is_some() {
                    (
                        mul_inst.src0,
                        mul_inst.src1,
                        known_const[mul_inst.src1 as usize],
                    )
                } else {
                    (
                        mul_inst.src1,
                        mul_inst.src0,
                        known_const[mul_inst.src0 as usize],
                    )
                };
                if let Some(orig_x) = log2_arg[l_reg as usize] {
                    if is_approx_f32(exp_c, 2.0, 1e-5) {
                        *inst = RawInst::new(VmOp::FMul, inst.dst, orig_x, orig_x, 0);
                    } else if is_approx_f32(exp_c, 0.5, 1e-5) {
                        *inst = RawInst::new(VmOp::FSqrt, inst.dst, orig_x, 0, 0);
                    } else {
                        *inst = RawInst::new(VmOp::FPow, inst.dst, orig_x, exp_reg, 0);
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
        | VmOp::FPow
        | VmOp::FFrc
        | VmOp::LinearToSrgb => true,
        VmOp::FMax => n0 || n1,
        VmOp::FMin | VmOp::FAdd | VmOp::FMul => n0 && n1,
        VmOp::FMul3 | VmOp::FMad => n0 && n1 && n2,
        VmOp::Movc => n1 && n2,
        _ => false,
    }
}

fn eval_unit_op(op: VmOp, u0: bool, u1: bool, n0: bool, n1: bool) -> bool {
    match op {
        VmOp::Mov | VmOp::FSqrt | VmOp::FHermite => u0,
        VmOp::MovSat | VmOp::FFrc | VmOp::LinearToSrgb => true,
        VmOp::FMul | VmOp::FMax => u0 && u1,
        VmOp::FMin => (u0 && n1) || (u1 && n0),
        VmOp::FPow => u0 && n1,
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

struct SsaTables {
    known_const: Vec<Option<u32>>,
    is_non_neg: Vec<bool>,
    is_unit: Vec<bool>,
    is_uniform: Vec<bool>,
    is_row_inv: Vec<bool>,
    def_inst: Vec<Option<RawInst>>,
    alias: Vec<u16>,
    log2_arg: Vec<Option<u16>>,
    cse_table: Vec<(VmOp, u16, u16, u16, u16)>,
    out_insts: Vec<RawInst>,
}

impl SsaTables {
    fn new(b: &CompilerBuilder<'_>, cap: usize, inst_cap: usize) -> Self {
        assert!(cap >= REG_DYNAMIC_BASE as usize);
        let mut known_const = vec![None; cap];
        let mut is_non_neg = vec![false; cap];
        let mut is_unit = vec![false; cap];
        let mut is_uniform = vec![false; cap];
        let mut is_row_inv = vec![false; cap];

        for r in [
            REG_ZERO,
            REG_ONE,
            REG_TRUE,
            REG_V0_W,
            REG_REAL_TIME,
            REG_V1_R,
            REG_V1_R + 1,
            REG_V1_R + 2,
            REG_V1_R + 3,
        ] {
            is_uniform[r as usize] = true;
            is_row_inv[r as usize] = true;
        }
        for r in [
            super::ir::REG_V0_Y,
            super::ir::REG_V2_Y,
            super::ir::REG_V3_V,
            super::ir::REG_V4_Y,
            super::ir::REG_V4_W,
        ] {
            is_row_inv[r as usize] = true;
        }
        for cb2_b in &b.cb2_bindings {
            is_uniform[cb2_b.vreg as usize] = true;
            is_row_inv[cb2_b.vreg as usize] = true;
        }
        known_const[REG_ZERO as usize] = Some(0);
        known_const[REG_ONE as usize] = Some(0x3F80_0000);
        known_const[REG_TRUE as usize] = Some(0xFFFF_FFFF);
        known_const[REG_V0_W as usize] = Some(0x3F80_0000);
        if b.spec.static_cb2.is_some() {
            known_const[REG_REAL_TIME as usize] = Some(0);
            is_non_neg[REG_REAL_TIME as usize] = true;
            is_unit[REG_REAL_TIME as usize] = true;
        }
        for r in [REG_ZERO, REG_ONE, REG_V0_W]
            .into_iter()
            .chain(REG_V1_R..=REG_V1_R + 3)
            .chain(REG_V3_U..=REG_V4_X + 3)
        {
            is_non_neg[r as usize] = true;
            is_unit[r as usize] = true;
        }
        for cb in &b.const_bindings {
            let idx = cb.vreg as usize;
            let f = f32::from_bits(cb.bits);
            known_const[idx] = Some(cb.bits);
            is_non_neg[idx] = (cb.bits & 0x8000_0000) == 0 || cb.bits == 0x8000_0000;
            is_unit[idx] = is_non_neg[idx] && f <= 1.0;
            is_uniform[idx] = true;
            is_row_inv[idx] = true;
        }
        Self {
            known_const,
            is_non_neg,
            is_unit,
            is_uniform,
            is_row_inv,
            def_inst: vec![None; cap],
            alias: (0..cap as u16).collect(),
            log2_arg: vec![None; cap],
            cse_table: Vec::with_capacity(inst_cap),
            out_insts: Vec::with_capacity(inst_cap),
        }
    }

    fn alloc_const(&mut self, b: &mut CompilerBuilder<'_>, bits: u32) -> u16 {
        let vreg = b.alloc_const(bits);
        let idx = vreg as usize;
        assert!(idx < self.known_const.len());
        let f = f32::from_bits(bits);
        self.known_const[idx] = Some(bits);
        self.is_non_neg[idx] = (bits & 0x8000_0000) == 0 || bits == 0x8000_0000;
        self.is_unit[idx] = self.is_non_neg[idx] && f <= 1.0;
        self.is_uniform[idx] = true;
        self.is_row_inv[idx] = true;
        vreg
    }

    fn record_alias(&mut self, d: usize, target: u16) {
        assert!(d < self.alias.len());
        let t = target as usize;
        self.alias[d] = target;
        self.known_const[d] = self.known_const[t];
        self.is_non_neg[d] = self.is_non_neg[t];
        self.is_unit[d] = self.is_unit[t];
        self.is_uniform[d] = self.is_uniform[t];
        self.is_row_inv[d] = self.is_row_inv[t];
        self.def_inst[d] = self.def_inst[t];
        self.log2_arg[d] = self.log2_arg[t];
    }

    fn record_inst(&mut self, inst: RawInst) {
        let d = inst.dst as usize;
        assert!(d < self.def_inst.len());
        let (s0, s1, s2) = (inst.src0 as usize, inst.src1 as usize, inst.src2 as usize);
        self.is_unit[d] = eval_unit_op(
            inst.op,
            self.is_unit[s0],
            self.is_unit[s1],
            self.is_non_neg[s0],
            self.is_non_neg[s1],
        );
        self.is_non_neg[d] = eval_non_neg_op(
            inst.op,
            self.is_non_neg[s0],
            self.is_non_neg[s1],
            self.is_non_neg[s2],
        ) || (inst.op == VmOp::FHermite && self.is_unit[s0]);
        let no_deriv = !matches!(inst.op, VmOp::FDdx | VmOp::FDdy);
        self.is_uniform[d] = d >= (REG_DYNAMIC_BASE as usize)
            && no_deriv
            && self.is_uniform[s0]
            && self.is_uniform[s1]
            && self.is_uniform[s2];
        self.is_row_inv[d] = d >= (REG_DYNAMIC_BASE as usize)
            && no_deriv
            && self.is_row_inv[s0]
            && self.is_row_inv[s1]
            && self.is_row_inv[s2];
        self.def_inst[d] = Some(inst);
        if inst.op == VmOp::FLog2 {
            self.log2_arg[d] = Some(inst.src0);
        }
        if inst.dst >= u16::from(REG_DYNAMIC_BASE) {
            self.cse_table
                .push((inst.op, inst.src0, inst.src1, inst.src2, inst.dst));
        }
    }

    fn canonicalize_src_order(&self, inst: &mut RawInst) {
        assert!((inst.src0 as usize) < self.is_row_inv.len());
        if matches!(inst.op, VmOp::FAdd | VmOp::FMul | VmOp::FMad) {
            let (s0, s1) = (inst.src0 as usize, inst.src1 as usize);
            let swap = (self.is_row_inv[s0] && !self.is_row_inv[s1])
                || (self.is_row_inv[s0] == self.is_row_inv[s1]
                    && self.is_uniform[s0]
                    && !self.is_uniform[s1])
                || (self.is_uniform[s0] == self.is_uniform[s1]
                    && self.known_const[s0].is_some()
                    && self.known_const[s1].is_none());
            if swap {
                std::mem::swap(&mut inst.src0, &mut inst.src1);
            }
        }
    }

    fn try_fold_affine_chains(&mut self, b: &mut CompilerBuilder<'_>, inst: &mut RawInst) {
        assert!((inst.src0 as usize) < self.def_inst.len());
        self.canonicalize_src_order(inst);
        let Some(u2) = self.known_const[inst.src1 as usize] else {
            return;
        };
        let Some(d0) = self.def_inst[inst.src0 as usize] else {
            return;
        };
        match (inst.op, d0.op) {
            (VmOp::FAdd, VmOp::FAdd) => {
                if let Some(u1) = self.known_const[d0.src1 as usize]
                    && let Some(u12) = eval_const_op(VmOp::FAdd, u1, u2, 0)
                {
                    let c12 = self.alloc_const(b, u12);
                    *inst = RawInst::new(VmOp::FAdd, inst.dst, d0.src0, c12, 0);
                }
            }
            (VmOp::FAdd, VmOp::FMul) if self.known_const[d0.src1 as usize].is_some() => {
                *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, d0.src1, inst.src1);
            }
            (VmOp::FAdd, VmOp::FMad) => {
                if let Some(u1) = self.known_const[d0.src2 as usize]
                    && let Some(u12) = eval_const_op(VmOp::FAdd, u1, u2, 0)
                {
                    let c12 = self.alloc_const(b, u12);
                    *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, d0.src1, c12);
                }
            }
            (VmOp::FMul, VmOp::FMul) => {
                if let Some(u1) = self.known_const[d0.src1 as usize]
                    && let Some(u12) = eval_const_op(VmOp::FMul, u1, u2, 0)
                {
                    let c12 = self.alloc_const(b, u12);
                    *inst = RawInst::new(VmOp::FMul, inst.dst, d0.src0, c12, 0);
                }
            }
            (VmOp::FMul, VmOp::FAdd) => {
                if let Some(u1) = self.known_const[d0.src1 as usize]
                    && let Some(u12) = eval_const_op(VmOp::FMul, u1, u2, 0)
                {
                    let c12 = self.alloc_const(b, u12);
                    *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, inst.src1, c12);
                }
            }
            (VmOp::FMul, VmOp::FMad) => {
                if let (Some(u_s), Some(u_b)) = (
                    self.known_const[d0.src1 as usize],
                    self.known_const[d0.src2 as usize],
                ) && let (Some(ns), Some(nb)) = (
                    eval_const_op(VmOp::FMul, u_s, u2, 0),
                    eval_const_op(VmOp::FMul, u_b, u2, 0),
                ) {
                    let cs = self.alloc_const(b, ns);
                    let cb = self.alloc_const(b, nb);
                    *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, cs, cb);
                }
            }
            (VmOp::FMad, VmOp::FAdd) => {
                if let (Some(u1), Some(u_bias)) = (
                    self.known_const[d0.src1 as usize],
                    self.known_const[inst.src2 as usize],
                ) && let Some(nb) = eval_const_op(VmOp::FMad, u1, u2, u_bias)
                {
                    let cb = self.alloc_const(b, nb);
                    *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, inst.src1, cb);
                }
            }
            (VmOp::FMad, VmOp::FMad) => {
                if let (Some(u_s), Some(u_b), Some(u_bias)) = (
                    self.known_const[d0.src1 as usize],
                    self.known_const[d0.src2 as usize],
                    self.known_const[inst.src2 as usize],
                ) && let (Some(ns), Some(nb)) = (
                    eval_const_op(VmOp::FMul, u_s, u2, 0),
                    eval_const_op(VmOp::FMad, u_b, u2, u_bias),
                ) {
                    let cs = self.alloc_const(b, ns);
                    let cb = self.alloc_const(b, nb);
                    *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, cs, cb);
                }
            }
            _ => {}
        }
    }

    fn emit_helper_op(
        &mut self,
        b: &mut CompilerBuilder<'_>,
        op: VmOp,
        s0: u16,
        s1: u16,
        s2: u16,
    ) -> u16 {
        assert!((s0 as usize) < self.known_const.len());
        let mut tmp = RawInst::new(op, 0, s0, s1, s2);
        try_cancel_neg_and_lerp(&mut tmp, &self.def_inst, &self.cse_table);
        self.try_fold_affine_chains(b, &mut tmp);
        let c0 = self.known_const[tmp.src0 as usize];
        let c1 = self.known_const[tmp.src1 as usize];
        let c2 = self.known_const[tmp.src2 as usize];
        if let (Some(u0), Some(u1), Some(u2)) = (c0, c1, c2)
            && let Some(bits) = eval_const_op(tmp.op, u0, u1, u2)
        {
            return self.alloc_const(b, bits);
        }
        simplify_algebraic_peephole(
            &mut tmp,
            c0,
            c1,
            c2,
            &self.known_const,
            &self.is_non_neg,
            &self.is_unit,
            &self.def_inst,
            &self.log2_arg,
        );
        self.canonicalize_src_order(&mut tmp);
        if tmp.op == VmOp::Mov {
            return self.alias[tmp.src0 as usize];
        }
        let is_comm = matches!(tmp.op, VmOp::FAdd | VmOp::FMul | VmOp::FMin | VmOp::FMax);
        if let Some(&(_, _, _, _, prev_dst)) =
            self.cse_table.iter().find(|&&(cop, a, b_op, c, _)| {
                cop == tmp.op
                    && c == tmp.src2
                    && ((a == tmp.src0 && b_op == tmp.src1)
                        || (is_comm && a == tmp.src1 && b_op == tmp.src0))
            })
        {
            return prev_dst;
        }
        let v = b.alloc_vreg(1);
        tmp.dst = v;
        self.record_inst(tmp);
        self.out_insts.push(tmp);
        v
    }

    fn try_reassociate_uniform_fmul(&mut self, b: &mut CompilerBuilder<'_>, inst: &mut RawInst) {
        assert_eq!(inst.op, VmOp::FMul);
        self.canonicalize_src_order(inst);
        if self.is_uniform[inst.src0 as usize] {
            return;
        }
        let is_var_uni = |r: u16, s: &Self| -> bool {
            s.def_inst[r as usize].is_some_and(|d| {
                d.op == VmOp::FMul
                    && !s.is_uniform[d.src0 as usize]
                    && s.is_uniform[d.src1 as usize]
            })
        };
        if !is_var_uni(inst.src0, self) {
            if !self.is_uniform[inst.src1 as usize] && is_var_uni(inst.src1, self) {
                std::mem::swap(&mut inst.src0, &mut inst.src1);
            } else {
                return;
            }
        }
        let Some(d0) = self.def_inst[inst.src0 as usize] else {
            return;
        };
        if self.is_uniform[inst.src1 as usize] {
            let u12 = self.emit_helper_op(b, VmOp::FMul, d0.src1, inst.src1, 0);
            *inst = RawInst::new(VmOp::FMul, inst.dst, d0.src0, u12, 0);
        } else if is_var_uni(inst.src1, self)
            && let Some(d1) = self.def_inst[inst.src1 as usize]
        {
            let xy = self.emit_helper_op(b, VmOp::FMul, d0.src0, d1.src0, 0);
            let u12 = self.emit_helper_op(b, VmOp::FMul, d0.src1, d1.src1, 0);
            *inst = RawInst::new(VmOp::FMul, inst.dst, xy, u12, 0);
        } else if !self.is_row_inv[inst.src0 as usize] || self.is_row_inv[inst.src1 as usize] {
            let xy = self.emit_helper_op(b, VmOp::FMul, d0.src0, inst.src1, 0);
            *inst = RawInst::new(VmOp::FMul, inst.dst, xy, d0.src1, 0);
        }
    }

    fn try_reassociate_stage_affine(
        &mut self,
        b: &mut CompilerBuilder<'_>,
        inst: &mut RawInst,
        row_stage: bool,
    ) {
        assert!((inst.src0 as usize) < self.def_inst.len());
        let inv = |r: u16, s: &Self| -> bool {
            if row_stage {
                s.is_row_inv[r as usize]
            } else {
                s.is_uniform[r as usize]
            }
        };
        let (s0, s1, s2) = (inst.src0, inst.src1, inst.src2);
        match inst.op {
            VmOp::FMul if !inv(s0, self) && inv(s1, self) => {
                if let Some(d0) = self.def_inst[s0 as usize] {
                    if d0.op == VmOp::FAdd && !inv(d0.src0, self) && inv(d0.src1, self) {
                        let c = self.emit_helper_op(b, VmOp::FMul, d0.src1, s1, 0);
                        *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, s1, c);
                    } else if d0.op == VmOp::FMad
                        && !inv(d0.src0, self)
                        && inv(d0.src1, self)
                        && inv(d0.src2, self)
                    {
                        let ns = self.emit_helper_op(b, VmOp::FMul, d0.src1, s1, 0);
                        let nb = self.emit_helper_op(b, VmOp::FMul, d0.src2, s1, 0);
                        *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, ns, nb);
                    }
                }
            }
            VmOp::FAdd if !inv(s0, self) && inv(s1, self) => {
                if let Some(d0) = self.def_inst[s0 as usize] {
                    if d0.op == VmOp::FMad
                        && !inv(d0.src0, self)
                        && inv(d0.src1, self)
                        && inv(d0.src2, self)
                    {
                        let nc = self.emit_helper_op(b, VmOp::FAdd, d0.src2, s1, 0);
                        *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, d0.src1, nc);
                    } else if d0.op == VmOp::FAdd && !inv(d0.src0, self) && inv(d0.src1, self) {
                        let nc = self.emit_helper_op(b, VmOp::FAdd, d0.src1, s1, 0);
                        *inst = RawInst::new(VmOp::FAdd, inst.dst, d0.src0, nc, 0);
                    } else if d0.op == VmOp::FMul && !is_zero_bits(self.known_const[s1 as usize]) {
                        *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, d0.src1, s1);
                    }
                }
            }
            VmOp::FMad if inv(s0, self) && inv(s1, self) && !inv(s2, self) => {
                if let Some(d2) = self.def_inst[s2 as usize] {
                    if d2.op == VmOp::FMad
                        && !inv(d2.src0, self)
                        && inv(d2.src1, self)
                        && inv(d2.src2, self)
                    {
                        let nc = self.emit_helper_op(b, VmOp::FMad, s0, s1, d2.src2);
                        *inst = RawInst::new(VmOp::FMad, inst.dst, d2.src0, d2.src1, nc);
                    } else if d2.op == VmOp::FMul && !inv(d2.src0, self) && inv(d2.src1, self) {
                        let nc = self.emit_helper_op(b, VmOp::FMul, s0, s1, 0);
                        *inst = RawInst::new(VmOp::FMad, inst.dst, d2.src0, d2.src1, nc);
                    } else if d2.op == VmOp::FAdd && !inv(d2.src0, self) && inv(d2.src1, self) {
                        let nc = self.emit_helper_op(b, VmOp::FMad, s0, s1, d2.src1);
                        *inst = RawInst::new(VmOp::FAdd, inst.dst, d2.src0, nc, 0);
                    }
                }
            }
            VmOp::FMad if !inv(s0, self) && inv(s1, self) && inv(s2, self) => {
                if let Some(d0) = self.def_inst[s0 as usize] {
                    if d0.op == VmOp::FAdd && !inv(d0.src0, self) && inv(d0.src1, self) {
                        let nc = self.emit_helper_op(b, VmOp::FMad, d0.src1, s1, s2);
                        *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, s1, nc);
                    } else if d0.op == VmOp::FMad
                        && !inv(d0.src0, self)
                        && inv(d0.src1, self)
                        && inv(d0.src2, self)
                    {
                        let ns = self.emit_helper_op(b, VmOp::FMul, d0.src1, s1, 0);
                        let nc = self.emit_helper_op(b, VmOp::FMad, d0.src2, s1, s2);
                        *inst = RawInst::new(VmOp::FMad, inst.dst, d0.src0, ns, nc);
                    }
                }
            }
            _ => {}
        }
    }

    fn try_fuse_fadd_or_hermite(&mut self, b: &mut CompilerBuilder<'_>, inst: &mut RawInst) {
        assert!((inst.src0 as usize) < self.def_inst.len());
        if inst.op == VmOp::FNmad
            && let Some(t) =
                try_match_hermite_arg(inst.src0, inst.src1, &self.known_const, &self.def_inst)
        {
            let h = self.emit_helper_op(b, VmOp::FHermite, t, 0, 0);
            *inst = RawInst::new(VmOp::FSub, inst.dst, inst.src2, h, 0);
            return;
        }
        if inst.op == VmOp::FAdd {
            let (s0, s1) = (inst.src0 as usize, inst.src1 as usize);
            let d0 = self.def_inst[s0];
            let d1 = self.def_inst[s1];
            let no_bitand = !matches!(d0.map(|d| d.op), Some(VmOp::BitAnd))
                && !matches!(d1.map(|d| d.op), Some(VmOp::BitAnd));
            if no_bitand {
                if let Some(m) = d0
                    && m.op == VmOp::FMul
                    && !self.is_uniform[s0]
                    && !(self.is_row_inv[s0] && !self.is_row_inv[s1])
                {
                    *inst = RawInst::new(VmOp::FMad, inst.dst, m.src0, m.src1, inst.src1);
                } else if let Some(m) = d1
                    && m.op == VmOp::FMul
                    && !self.is_uniform[s1]
                    && !(self.is_row_inv[s1] && !self.is_row_inv[s0])
                {
                    *inst = RawInst::new(VmOp::FMad, inst.dst, m.src0, m.src1, inst.src0);
                }
            }
        }
    }
}

fn fold_ssa_constants_and_cse(b: &mut CompilerBuilder<'_>) {
    assert!(b.next_vreg >= u16::from(REG_DYNAMIC_BASE));
    if !b.use_ssa {
        return;
    }
    let cap = (b.next_vreg as usize) + b.insts.len() * 3 + 256;
    let raw_insts = std::mem::take(&mut b.insts);
    let mut t = SsaTables::new(b, cap, raw_insts.len());

    for mut inst in raw_insts {
        inst.src0 = t.alias[inst.src0 as usize];
        inst.src1 = t.alias[inst.src1 as usize];
        if !matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4 | VmOp::LoadIdxTemp
        ) {
            inst.src2 = t.alias[inst.src2 as usize];
        } else if matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        ) {
            if inst.op == VmOp::Sample2D
                && let Some(meta) = b.spec.tex_meta.and_then(|m| m.get(inst.src2 as usize))
            {
                if meta.clamp_u
                    && let Some(d) = t.def_inst[inst.src0 as usize]
                    && d.op == VmOp::MovSat
                {
                    inst.src0 = t.alias[d.src0 as usize];
                }
                if meta.clamp_v
                    && let Some(d) = t.def_inst[inst.src1 as usize]
                    && d.op == VmOp::MovSat
                {
                    inst.src1 = t.alias[d.src0 as usize];
                }
            }
            if inst.op == VmOp::Sample2D
                && let Some(prev) = t.out_insts.iter_mut().find(|p| {
                    p.op == VmOp::Sample2D
                        && p.src0 == inst.src0
                        && p.src1 == inst.src1
                        && p.src2 == inst.src2
                        && p.imm16 == inst.imm16
                })
            {
                prev.aux |= inst.aux;
                for c in 0..4u16 {
                    let d = (inst.dst + c) as usize;
                    let target = prev.dst + c;
                    t.alias[d] = target;
                    t.is_non_neg[d] = true;
                    t.is_unit[d] = true;
                }
                continue;
            }
            for c in 0..4usize {
                t.is_non_neg[(inst.dst as usize) + c] = true;
                t.is_unit[(inst.dst as usize) + c] = true;
            }
        }

        if is_pure_ssa_alu(inst.op) {
            try_cancel_neg_and_lerp(&mut inst, &t.def_inst, &t.cse_table);
            t.try_fold_affine_chains(b, &mut inst);
            if t.is_row_inv[inst.src0 as usize]
                && t.is_row_inv[inst.src1 as usize]
                && t.is_row_inv[inst.src2 as usize]
            {
                t.try_reassociate_stage_affine(b, &mut inst, false);
            } else {
                t.try_reassociate_stage_affine(b, &mut inst, true);
            }
            t.try_fold_affine_chains(b, &mut inst);
            let c0 = t.known_const[inst.src0 as usize];
            let c1 = t.known_const[inst.src1 as usize];
            let c2 = t.known_const[inst.src2 as usize];

            if let (Some(u0), Some(u1), Some(u2)) = (c0, c1, c2)
                && let Some(bits) = eval_const_op(inst.op, u0, u1, u2)
            {
                let c_vreg = t.alloc_const(b, bits);
                inst = RawInst::new(VmOp::Mov, inst.dst, c_vreg, 0, 0);
            } else {
                simplify_algebraic_peephole(
                    &mut inst,
                    c0,
                    c1,
                    c2,
                    &t.known_const,
                    &t.is_non_neg,
                    &t.is_unit,
                    &t.def_inst,
                    &t.log2_arg,
                );
                try_cancel_neg_and_lerp(&mut inst, &t.def_inst, &t.cse_table);
                t.try_fuse_fadd_or_hermite(b, &mut inst);
                if inst.op == VmOp::MovSat && t.is_unit[inst.src0 as usize] {
                    inst = RawInst::new(VmOp::Mov, inst.dst, inst.src0, 0, 0);
                }
                if inst.op == VmOp::FMul {
                    t.try_reassociate_uniform_fmul(b, &mut inst);
                }
            }

            if inst.op == VmOp::Mov && inst.dst >= u16::from(REG_DYNAMIC_BASE) {
                let target = t.alias[inst.src0 as usize];
                t.record_alias(inst.dst as usize, target);
                continue;
            }

            let is_comm = matches!(
                inst.op,
                VmOp::FAdd
                    | VmOp::FMul
                    | VmOp::FMin
                    | VmOp::FMax
                    | VmOp::BitAnd
                    | VmOp::BitOr
                    | VmOp::BitXor
            );
            if inst.dst >= u16::from(REG_DYNAMIC_BASE)
                && let Some(&(_, _, _, _, prev_dst)) =
                    t.cse_table.iter().find(|&&(op, s0, s1, s2, _)| {
                        op == inst.op
                            && s2 == inst.src2
                            && ((s0 == inst.src0 && s1 == inst.src1)
                                || (is_comm && s0 == inst.src1 && s1 == inst.src0))
                    })
            {
                t.record_alias(inst.dst as usize, prev_dst);
                continue;
            }

            t.record_inst(inst);
        }
        t.out_insts.push(inst);
    }
    assert!(t.out_insts.len() <= MAX_STAGE_INSTS * 4);
    b.insts = t.out_insts;
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
    tex_meta: Option<&[TexSlotMeta]>,
) -> (Vec<RawInst>, Vec<RawInst>, Vec<RawInst>, u8) {
    assert!(insts.len() <= MAX_STAGE_INSTS * 4);
    if !use_ssa {
        return (Vec::new(), Vec::new(), insts, 0);
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

    let mut out_writes = [0u16; 4];
    for inst in &insts {
        if (u16::from(REG_OUT_R)..=u16::from(REG_OUT_A)).contains(&inst.dst) {
            let idx = (inst.dst - u16::from(REG_OUT_R)) as usize;
            out_writes[idx] = out_writes[idx].saturating_add(1);
        }
    }

    let mut preamble = Vec::new();
    let mut kernel = Vec::with_capacity(insts.len());

    for inst in insts {
        if inst.op == VmOp::Ret {
            continue;
        }
        let is_tex = matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        );
        let is_deriv = matches!(inst.op, VmOp::FDdx | VmOp::FDdy);
        let is_single_out_write = (u16::from(REG_OUT_R)..=u16::from(REG_OUT_A)).contains(&inst.dst)
            && out_writes[(inst.dst - u16::from(REG_OUT_R)) as usize] == 1;
        let can_hoist = !is_control_flow_or_side_effect(inst.op)
            && !is_tex
            && !is_deriv
            && (inst.dst >= u16::from(REG_DYNAMIC_BASE) || is_single_out_write)
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
    let is_additive = preamble.iter().chain(kernel.iter()).any(|inst| {
        inst.dst == u16::from(REG_OUT_A) && inst.op == VmOp::Mov && inst.src0 == u16::from(REG_ZERO)
    });
    hoist_zero_annihilator_mask(&mut kernel, num_vregs, is_additive, tex_meta);
    fuse_kernel_superinstructions(&mut kernel, num_vregs);
    let srgb_mask = extract_srgb_output_mask(&mut kernel, num_vregs);
    coalesce_kernel_output_moves(&mut kernel, num_vregs);
    let row_preamble = extract_row_preamble(&mut kernel, &is_uniform, num_vregs);
    assert!(kernel.len() <= MAX_STAGE_INSTS * 4);
    (preamble, row_preamble, kernel, srgb_mask)
}

fn does_zero_reg_kill_pixel(
    kernel: &[RawInst],
    sample_pc: usize,
    cand_reg: u16,
    is_additive: bool,
    num_vregs: usize,
) -> bool {
    assert!((cand_reg as usize) < num_vregs + 8);
    assert!(sample_pc < kernel.len());
    let mut is_zero = vec![false; num_vregs + 8];
    is_zero[REG_ZERO as usize] = true;
    is_zero[cand_reg as usize] = true;

    for inst in &kernel[(sample_pc + 1)..] {
        if matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        ) {
            for c in 0..4usize {
                is_zero[(inst.dst as usize) + c] = false;
            }
            continue;
        }
        let d = inst.dst as usize;
        if d >= is_zero.len() || is_control_flow_or_side_effect(inst.op) {
            continue;
        }
        let z0 = is_zero[inst.src0 as usize];
        let z1 = is_zero[inst.src1 as usize];
        let z2 = is_zero[inst.src2 as usize];
        is_zero[d] = match inst.op {
            VmOp::Mov
            | VmOp::MovSat
            | VmOp::FAbs
            | VmOp::FNeg
            | VmOp::FNegAbs
            | VmOp::FSqrt
            | VmOp::FFrc
            | VmOp::FRoundNe
            | VmOp::FFloor
            | VmOp::FCeil
            | VmOp::FTrunc
            | VmOp::FSin
            | VmOp::FDiv
            | VmOp::FPow
            | VmOp::FHermite => z0,
            VmOp::FMul | VmOp::FMin => z0 || z1,
            VmOp::FMul3 | VmOp::LinearToSrgb => z0 || z1 || z2,
            VmOp::FAdd | VmOp::FSub | VmOp::FMax => z0 && z1,
            VmOp::FMad | VmOp::FNmad => (z0 || z1) && z2,
            _ => false,
        };
    }

    if is_additive {
        is_zero[REG_OUT_R as usize] && is_zero[REG_OUT_G as usize] && is_zero[REG_OUT_B as usize]
    } else {
        is_zero[REG_OUT_A as usize]
    }
}

type HoistCandidate = (usize, u16, Vec<bool>, (u8, usize, usize));

fn hoist_zero_annihilator_mask(
    kernel: &mut Vec<RawInst>,
    num_vregs: usize,
    is_additive: bool,
    tex_meta: Option<&[TexSlotMeta]>,
) {
    assert!(kernel.len() <= MAX_STAGE_INSTS * 4);
    let mut zero_kills = vec![false; num_vregs + 8];
    for sample_pc in 0..kernel.len() {
        let s_inst = kernel[sample_pc];
        if s_inst.op == VmOp::Sample2D && s_inst.dst >= u16::from(REG_DYNAMIC_BASE) {
            for c in 0..4u16 {
                if (s_inst.aux >> c) & 1 != 0 {
                    let reg = s_inst.dst + c;
                    zero_kills[reg as usize] =
                        does_zero_reg_kill_pixel(kernel, sample_pc, reg, is_additive, num_vregs);
                }
            }
        }
    }
    let mut best_cand: Option<HoistCandidate> = None;

    for sample_pc in 0..kernel.len() {
        let s_inst = kernel[sample_pc];
        if s_inst.op != VmOp::Sample2D || s_inst.dst < u16::from(REG_DYNAMIC_BASE) {
            continue;
        }
        let Some((kill_ch, z_frac)) = (0..4u16)
            .filter_map(|c| {
                let zf = tex_meta
                    .and_then(|m| m.get(s_inst.src2 as usize))
                    .map_or(128, |meta| meta.zero_frac[c as usize]);
                ((s_inst.aux >> c) & 1 != 0 && zero_kills[(s_inst.dst + c) as usize] && zf > 0)
                    .then_some((c, zf))
            })
            .max_by_key(|&(_, zf)| zf)
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
            let score = (z_frac, 16 - slice_len, sample_pc);
            if best_cand.as_ref().is_none_or(|b| score > b.3) {
                best_cand = Some((sample_pc, kill_ch, in_slice, score));
            }
        }
    }

    if let Some((sample_pc, kill_ch, in_slice, _)) = best_cand {
        let mut s_inst = kernel[sample_pc];
        s_inst.aux |= 1 << (4 + kill_ch);
        let mut reordered = Vec::with_capacity(kernel.len() + 2);
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
        for (pc, &(mut inst)) in kernel.iter().enumerate() {
            if !in_slice[pc] && pc != sample_pc {
                if inst.op == VmOp::Sample2D
                    && inst.dst >= u16::from(REG_DYNAMIC_BASE)
                    && let Some(sec_c) = (0..4u16).find(|&c| {
                        let zf = tex_meta
                            .and_then(|m| m.get(inst.src2 as usize))
                            .map_or(0, |meta| meta.zero_frac[c as usize]);
                        (inst.aux >> c) & 1 != 0 && zero_kills[(inst.dst + c) as usize] && zf >= 32
                    })
                {
                    inst.aux |= 1 << (4 + sec_c);
                    reordered.push(inst);
                    reordered.push(RawInst::new(VmOp::EarlyOutZero, 0, inst.dst + sec_c, 0, 0));
                } else {
                    reordered.push(inst);
                }
            }
        }
        *kernel = reordered;
    }
}

fn fuse_kernel_superinstructions(kernel: &mut Vec<RawInst>, num_vregs: usize) {
    assert!(kernel.len() <= MAX_STAGE_INSTS * 4);
    assert!(num_vregs >= REG_DYNAMIC_BASE as usize);
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

    let one = u16::from(REG_ONE);
    let dyn_base = u16::from(REG_DYNAMIC_BASE);
    let mut drop_inst = vec![false; kernel.len()];
    let mut shared_ac: Vec<(u16, u16, u16)> = Vec::new();

    for pc in 0..kernel.len() {
        let inst = kernel[pc];
        if inst.op == VmOp::FMul {
            let (s0, s1) = (inst.src0, inst.src1);
            if s0 >= dyn_base
                && read_count[s0 as usize] == 1
                && let Some(d_pc) = def_pc[s0 as usize]
                && !drop_inst[d_pc]
                && kernel[d_pc].op == VmOp::FMul
            {
                let d = kernel[d_pc];
                kernel[pc] = RawInst::new(VmOp::FMul3, inst.dst, d.src0, d.src1, s1);
                drop_inst[d_pc] = true;
            } else if s1 >= dyn_base
                && read_count[s1 as usize] == 1
                && let Some(d_pc) = def_pc[s1 as usize]
                && !drop_inst[d_pc]
                && kernel[d_pc].op == VmOp::FMul
            {
                let d = kernel[d_pc];
                kernel[pc] = RawInst::new(VmOp::FMul3, inst.dst, d.src0, d.src1, s0);
                drop_inst[d_pc] = true;
            }
        } else if inst.op == VmOp::LinearToSrgb && inst.src1 == one && inst.src2 == one {
            try_fuse_srgb_product(
                pc,
                inst,
                kernel,
                &read_count,
                &def_pc,
                &mut drop_inst,
                &mut shared_ac,
            );
        }
    }

    let mut idx = 0usize;
    kernel.retain(|_| {
        let keep = !drop_inst[idx];
        idx += 1;
        keep
    });
}

fn try_fuse_srgb_product(
    pc: usize,
    inst: RawInst,
    kernel: &mut [RawInst],
    read_count: &[u16],
    def_pc: &[Option<usize>],
    drop_inst: &mut [bool],
    shared_ac: &mut Vec<(u16, u16, u16)>,
) {
    assert!(pc < kernel.len());
    assert_eq!(inst.op, VmOp::LinearToSrgb);
    let dyn_base = u16::from(REG_DYNAMIC_BASE);
    if inst.src0 < dyn_base || read_count[inst.src0 as usize] != 1 {
        return;
    }
    let Some(d_pc) = def_pc[inst.src0 as usize] else {
        return;
    };
    if drop_inst[d_pc] {
        return;
    }
    let d = kernel[d_pc];
    if d.op == VmOp::FMul {
        if d.src0 >= dyn_base
            && read_count[d.src0 as usize] == 1
            && let Some(g_pc) = def_pc[d.src0 as usize]
            && !drop_inst[g_pc]
            && kernel[g_pc].op == VmOp::FMul3
        {
            let g = kernel[g_pc];
            let ac_reg = if let Some(&(_, _, prev_ac)) = shared_ac
                .iter()
                .find(|&&(a, c, _)| a == g.src0 && c == g.src2)
            {
                drop_inst[g_pc] = true;
                prev_ac
            } else {
                kernel[g_pc] = RawInst::new(VmOp::FMul, g.dst, g.src0, g.src2, 0);
                shared_ac.push((g.src0, g.src2, g.dst));
                g.dst
            };
            kernel[pc] = RawInst::new(VmOp::LinearToSrgb, inst.dst, ac_reg, g.src1, d.src1);
            drop_inst[d_pc] = true;
        } else {
            kernel[pc] = RawInst::new(
                VmOp::LinearToSrgb,
                inst.dst,
                d.src0,
                d.src1,
                u16::from(REG_ONE),
            );
            drop_inst[d_pc] = true;
        }
    } else if d.op == VmOp::FMul3 {
        kernel[pc] = RawInst::new(VmOp::LinearToSrgb, inst.dst, d.src0, d.src1, d.src2);
        drop_inst[d_pc] = true;
    }
}

fn linearize_srgb_inst(inst: RawInst) -> RawInst {
    assert_eq!(inst.op, VmOp::LinearToSrgb);
    let one = u16::from(REG_ONE);
    assert!(one > 0);
    if inst.src1 == one && inst.src2 == one {
        RawInst::new(VmOp::Mov, inst.dst, inst.src0, 0, 0)
    } else if inst.src2 == one {
        RawInst::new(VmOp::FMul, inst.dst, inst.src0, inst.src1, 0)
    } else if inst.src1 == one {
        RawInst::new(VmOp::FMul, inst.dst, inst.src0, inst.src2, 0)
    } else {
        RawInst::new(VmOp::FMul3, inst.dst, inst.src0, inst.src1, inst.src2)
    }
}

fn extract_srgb_output_mask(kernel: &mut [RawInst], num_vregs: usize) -> u8 {
    assert!(kernel.len() <= MAX_STAGE_INSTS * 4);
    assert!(num_vregs >= REG_DYNAMIC_BASE as usize);
    let mut read_count = vec![0u16; num_vregs + 8];
    let mut out_mov_mask = vec![0u8; num_vregs + 8];
    let mut non_out_read = vec![false; num_vregs + 8];
    let out_r = u16::from(REG_OUT_R);
    let out_b = u16::from(REG_OUT_B);

    for inst in kernel.iter() {
        read_count[inst.src0 as usize] = read_count[inst.src0 as usize].saturating_add(1);
        read_count[inst.src1 as usize] = read_count[inst.src1 as usize].saturating_add(1);
        if !matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        ) {
            read_count[inst.src2 as usize] = read_count[inst.src2 as usize].saturating_add(1);
        }
        if inst.op == VmOp::Mov && (out_r..=out_b).contains(&inst.dst) {
            out_mov_mask[inst.src0 as usize] |= 1 << (inst.dst - out_r);
        } else {
            non_out_read[inst.src0 as usize] = true;
            non_out_read[inst.src1 as usize] = true;
            if !matches!(
                inst.op,
                VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
            ) {
                non_out_read[inst.src2 as usize] = true;
            }
        }
    }

    let mut srgb_mask = 0u8;
    for inst in kernel.iter_mut() {
        if inst.op != VmOp::LinearToSrgb {
            continue;
        }
        if (out_r..=out_b).contains(&inst.dst) && read_count[inst.dst as usize] == 0 {
            srgb_mask |= 1 << (inst.dst - out_r);
            *inst = linearize_srgb_inst(*inst);
        } else if inst.dst >= u16::from(REG_DYNAMIC_BASE)
            && out_mov_mask[inst.dst as usize] != 0
            && !non_out_read[inst.dst as usize]
        {
            srgb_mask |= out_mov_mask[inst.dst as usize];
            *inst = linearize_srgb_inst(*inst);
        }
    }
    srgb_mask
}

fn coalesce_kernel_output_moves(kernel: &mut Vec<RawInst>, num_vregs: usize) {
    assert!(kernel.len() <= MAX_STAGE_INSTS * 4);
    assert!(num_vregs >= REG_DYNAMIC_BASE as usize);
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

    if kernel.len() == 6
        && kernel[0].op == VmOp::Sample2D
        && kernel[1].op == VmOp::EarlyOutZero
        && kernel[1].src0 == kernel[0].dst + 3
        && (0..4u16).all(|c| {
            let m = kernel[2 + c as usize];
            m.op == VmOp::Mov && m.dst == u16::from(REG_OUT_R) + c && m.src0 == kernel[0].dst + c
        })
    {
        kernel[0].dst = u16::from(REG_OUT_R);
        kernel[1].src0 = u16::from(REG_OUT_A);
        kernel.truncate(2);
    }
}

fn extract_row_preamble(
    kernel: &mut Vec<RawInst>,
    is_uniform: &[bool],
    num_vregs: usize,
) -> Vec<RawInst> {
    assert!(kernel.len() <= MAX_STAGE_INSTS * 4);
    assert!(is_uniform.len() >= num_vregs);
    let mut is_row_inv = is_uniform.to_vec();
    for r in [
        super::ir::REG_V0_Y,
        super::ir::REG_V2_Y,
        super::ir::REG_V3_V,
        super::ir::REG_V4_Y,
        super::ir::REG_V4_W,
    ] {
        is_row_inv[r as usize] = true;
    }

    let mut write_count = vec![0u16; num_vregs + 8];
    for inst in kernel.iter() {
        if matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        ) {
            for c in 0..4usize {
                write_count[(inst.dst as usize) + c] =
                    write_count[(inst.dst as usize) + c].saturating_add(1);
            }
        } else {
            write_count[inst.dst as usize] = write_count[inst.dst as usize].saturating_add(1);
        }
    }

    let mut row_preamble = Vec::new();
    let mut rem_kernel = Vec::with_capacity(kernel.len());
    let dyn_base = u16::from(REG_DYNAMIC_BASE);

    for inst in kernel.drain(..) {
        if is_pure_ssa_alu(inst.op)
            && !matches!(inst.op, VmOp::FDdx | VmOp::FDdy)
            && inst.dst >= dyn_base
            && write_count[inst.dst as usize] == 1
            && is_row_inv[inst.src0 as usize]
            && is_row_inv[inst.src1 as usize]
            && is_row_inv[inst.src2 as usize]
        {
            is_row_inv[inst.dst as usize] = true;
            row_preamble.push(inst);
        } else {
            rem_kernel.push(inst);
        }
    }
    *kernel = rem_kernel;
    row_preamble
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

type LoweredStages = (
    Vec<VmInst>,
    Vec<VmInst>,
    Vec<VmInst>,
    Vec<Cb2RegBinding>,
    Vec<ConstRegBinding>,
    usize,
    usize,
    usize,
);

fn allocate_physical_regs(
    preamble_raw: &[RawInst],
    row_preamble_raw: &[RawInst],
    kernel_raw: &[RawInst],
    cb2_vregs: &[VregCb2Binding],
    const_vregs: &[VregConstBinding],
    num_vregs: usize,
    use_ssa: bool,
) -> LoweredStages {
    assert!(num_vregs >= REG_DYNAMIC_BASE as usize);
    let mut phys = vec![0xFFu8; num_vregs + 8];
    for (i, slot) in phys.iter_mut().enumerate().take(REG_DYNAMIC_BASE as usize) {
        *slot = i as u8;
    }

    if !use_ssa {
        return lower_non_ssa_stages(
            preamble_raw,
            kernel_raw,
            cb2_vregs,
            const_vregs,
            &mut phys,
            num_vregs,
        );
    }

    let (read_in_pre, read_in_row, read_in_ker) =
        mark_stage_register_reads(preamble_raw, row_preamble_raw, kernel_raw, num_vregs);
    let mut next_phys = REG_DYNAMIC_BASE as usize;
    let alloc_one = |next: &mut usize| -> u8 {
        assert!(
            *next < REG_FILE_SIZE - 1,
            "physical register file overflow at {}",
            *next
        );
        let r = *next as u8;
        *next += 1;
        r
    };

    let mut pre_next_phys = alloc_uniform_and_preamble_regs(
        preamble_raw,
        const_vregs,
        cb2_vregs,
        &read_in_pre,
        &read_in_row,
        &read_in_ker,
        &mut phys,
        &mut next_phys,
        num_vregs,
        alloc_one,
    );
    let (const_out, cb2_out) = collect_uniform_bindings(
        const_vregs,
        cb2_vregs,
        &read_in_pre,
        &read_in_row,
        &read_in_ker,
        &phys,
    );
    let num_uniform_regs = next_phys;

    for inst in row_preamble_raw {
        let v = inst.dst as usize;
        if v >= REG_DYNAMIC_BASE as usize && read_in_ker[v] && phys[v] == 0xFF {
            phys[v] = alloc_one(&mut next_phys);
        }
    }
    let num_row_regs = next_phys;
    let mut row_next_phys = num_row_regs;
    alloc_stage_temporaries(
        row_preamble_raw,
        &mut phys,
        &mut row_next_phys,
        num_row_regs,
        num_vregs,
        alloc_one,
    );

    alloc_kernel_temporaries(
        kernel_raw,
        &mut phys,
        &mut next_phys,
        num_row_regs,
        num_vregs,
        alloc_one,
    );
    pre_next_phys = pre_next_phys.max(row_next_phys);

    let pre = preamble_raw
        .iter()
        .map(|r| map_raw_to_vm_inst(r, &phys))
        .collect();
    let row_pre = row_preamble_raw
        .iter()
        .map(|r| map_raw_to_vm_inst(r, &phys))
        .collect();
    let ker = kernel_raw
        .iter()
        .map(|r| map_raw_to_vm_inst(r, &phys))
        .collect();
    let total_used = next_phys.max(pre_next_phys).max(REG_DYNAMIC_BASE as usize);
    assert!(total_used <= REG_FILE_SIZE);
    (
        pre,
        row_pre,
        ker,
        cb2_out,
        const_out,
        num_uniform_regs,
        num_row_regs,
        total_used,
    )
}

fn lower_non_ssa_stages(
    preamble_raw: &[RawInst],
    kernel_raw: &[RawInst],
    cb2_vregs: &[VregCb2Binding],
    const_vregs: &[VregConstBinding],
    phys: &mut [u8],
    num_vregs: usize,
) -> LoweredStages {
    assert!(phys.len() >= num_vregs);
    for (i, slot) in phys
        .iter_mut()
        .enumerate()
        .take(num_vregs.min(REG_FILE_SIZE))
        .skip(REG_DYNAMIC_BASE as usize)
    {
        *slot = i as u8;
    }
    let used = num_vregs.min(REG_FILE_SIZE);
    let pre = preamble_raw
        .iter()
        .map(|r| map_raw_to_vm_inst(r, phys))
        .collect();
    let ker = kernel_raw
        .iter()
        .map(|r| map_raw_to_vm_inst(r, phys))
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
    assert!(used <= REG_FILE_SIZE);
    (pre, Vec::new(), ker, cb2, cst, used, used, used)
}

fn mark_stage_register_reads(
    preamble_raw: &[RawInst],
    row_preamble_raw: &[RawInst],
    kernel_raw: &[RawInst],
    num_vregs: usize,
) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
    assert!(num_vregs >= REG_DYNAMIC_BASE as usize);
    let mut read_in_pre = vec![false; num_vregs + 8];
    let mut read_in_row = vec![false; num_vregs + 8];
    let mut read_in_ker = vec![false; num_vregs + 8];
    for inst in preamble_raw {
        read_in_pre[inst.src0 as usize] = true;
        read_in_pre[inst.src1 as usize] = true;
        read_in_pre[inst.src2 as usize] = true;
    }
    for inst in row_preamble_raw {
        read_in_row[inst.src0 as usize] = true;
        read_in_row[inst.src1 as usize] = true;
        read_in_row[inst.src2 as usize] = true;
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
    assert_eq!(read_in_pre.len(), read_in_ker.len());
    (read_in_pre, read_in_row, read_in_ker)
}

#[allow(clippy::too_many_arguments)]
fn alloc_uniform_and_preamble_regs(
    preamble_raw: &[RawInst],
    const_vregs: &[VregConstBinding],
    cb2_vregs: &[VregCb2Binding],
    read_in_pre: &[bool],
    read_in_row: &[bool],
    read_in_ker: &[bool],
    phys: &mut [u8],
    next_phys: &mut usize,
    num_vregs: usize,
    alloc_one: fn(&mut usize) -> u8,
) -> usize {
    assert!(*next_phys >= REG_DYNAMIC_BASE as usize);
    for b in const_vregs {
        let v = b.vreg as usize;
        if read_in_row[v] || read_in_ker[v] {
            phys[v] = alloc_one(next_phys);
        }
    }
    for b in cb2_vregs {
        let v = b.vreg as usize;
        if read_in_row[v] || read_in_ker[v] {
            phys[v] = alloc_one(next_phys);
        }
    }
    for inst in preamble_raw {
        let v = inst.dst as usize;
        if v >= REG_DYNAMIC_BASE as usize && (read_in_row[v] || read_in_ker[v]) && phys[v] == 0xFF {
            phys[v] = alloc_one(next_phys);
        }
    }
    let num_uniform_regs = *next_phys;
    let mut pre_next_phys = num_uniform_regs;
    for b in const_vregs {
        let v = b.vreg as usize;
        if read_in_pre[v] && phys[v] == 0xFF {
            phys[v] = alloc_one(&mut pre_next_phys);
        }
    }
    for b in cb2_vregs {
        let v = b.vreg as usize;
        if read_in_pre[v] && phys[v] == 0xFF {
            phys[v] = alloc_one(&mut pre_next_phys);
        }
    }
    alloc_stage_temporaries(
        preamble_raw,
        phys,
        &mut pre_next_phys,
        num_uniform_regs,
        num_vregs,
        alloc_one,
    );
    assert!(pre_next_phys <= REG_FILE_SIZE);
    pre_next_phys
}

fn collect_uniform_bindings(
    const_vregs: &[VregConstBinding],
    cb2_vregs: &[VregCb2Binding],
    read_in_pre: &[bool],
    read_in_row: &[bool],
    read_in_ker: &[bool],
    phys: &[u8],
) -> (Vec<ConstRegBinding>, Vec<Cb2RegBinding>) {
    assert!(phys.len() >= REG_DYNAMIC_BASE as usize);
    let mut const_out = Vec::new();
    for b in const_vregs {
        let v = b.vreg as usize;
        if read_in_pre[v] || read_in_row[v] || read_in_ker[v] {
            const_out.push(ConstRegBinding {
                dst_reg: phys[v],
                bits: b.bits,
            });
        }
    }
    let mut cb2_out = Vec::new();
    for b in cb2_vregs {
        let v = b.vreg as usize;
        if read_in_pre[v] || read_in_row[v] || read_in_ker[v] {
            cb2_out.push(Cb2RegBinding {
                dst_reg: phys[v],
                comp: b.comp,
                vec_idx: b.vec_idx,
            });
        }
    }
    assert!(const_out.len() <= const_vregs.len());
    (const_out, cb2_out)
}

fn alloc_stage_temporaries(
    stage_raw: &[RawInst],
    phys: &mut [u8],
    next_phys: &mut usize,
    stage_floor: usize,
    num_vregs: usize,
    alloc_one: fn(&mut usize) -> u8,
) {
    assert!(stage_floor >= REG_DYNAMIC_BASE as usize);
    let mut free_pool: Vec<u8> = Vec::new();
    let mut released = vec![false; num_vregs + 8];
    let mut is_read = vec![false; num_vregs + 8];
    let mut last_use = vec![0usize; num_vregs + 8];
    for (pc, inst) in stage_raw.iter().enumerate() {
        for &src in &[inst.src0, inst.src1, inst.src2] {
            is_read[src as usize] = true;
            last_use[src as usize] = pc;
        }
    }
    for (pc, inst) in stage_raw.iter().enumerate() {
        let d = inst.dst as usize;
        if d >= REG_DYNAMIC_BASE as usize && phys[d] == 0xFF {
            let p = free_pool.pop().unwrap_or_else(|| alloc_one(next_phys));
            phys[d] = p;
            if !is_read[d] {
                released[d] = true;
                free_pool.push(p);
            }
        }
        for &src in &[inst.src0, inst.src1, inst.src2] {
            let s = src as usize;
            if s >= REG_DYNAMIC_BASE as usize
                && !released[s]
                && last_use[s] == pc
                && (phys[s] as usize) >= stage_floor
            {
                released[s] = true;
                free_pool.push(phys[s]);
            }
        }
    }
    assert!(*next_phys <= REG_FILE_SIZE);
}

fn alloc_kernel_temporaries(
    kernel_raw: &[RawInst],
    phys: &mut [u8],
    next_phys: &mut usize,
    num_row_regs: usize,
    num_vregs: usize,
    alloc_one: fn(&mut usize) -> u8,
) {
    assert!(num_row_regs >= REG_DYNAMIC_BASE as usize);
    // Allocate contiguous 4-register blocks for texture samples first, then linear-scan recycling
    // for all single-channel scalar temporaries in `kernel_raw`.
    let mut free_pool: Vec<u8> = Vec::new();
    let mut released = vec![false; num_vregs + 8];
    for inst in kernel_raw {
        if matches!(
            inst.op,
            VmOp::Sample2D | VmOp::TexLoad2D | VmOp::ResInfo2D | VmOp::Gather4
        ) && inst.dst >= u16::from(REG_DYNAMIC_BASE)
        {
            for c in 0..4usize {
                let p = alloc_one(next_phys);
                let v = (inst.dst as usize) + c;
                phys[v] = p;
                if (inst.aux & (1 << c)) == 0 {
                    released[v] = true;
                    free_pool.push(p);
                }
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

    for (pc, inst) in kernel_raw.iter().enumerate() {
        let d = inst.dst as usize;
        if d >= REG_DYNAMIC_BASE as usize && phys[d] == 0xFF {
            phys[d] = free_pool.pop().unwrap_or_else(|| alloc_one(next_phys));
        }
        for &src in &[inst.src0, inst.src1, inst.src2] {
            let s = src as usize;
            if s >= REG_DYNAMIC_BASE as usize
                && !released[s]
                && last_use[s] == pc
                && (phys[s] as usize) >= num_row_regs
            {
                assert_ne!(
                    phys[s], 0xFF,
                    "unmapped kernel src vreg {s} at pc {pc} inst={inst:?} next_phys={next_phys}"
                );
                // Do not recycle channels belonging to a 4-register texture sample block early
                // unless all uses of that channel have completed.
                released[s] = true;
                free_pool.push(phys[s]);
            }
        }
    }
    assert!(*next_phys <= REG_FILE_SIZE);
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

fn tag_row_invariant_samples(
    kernel: &mut [VmInst],
    num_row_regs: usize,
    has_control_flow: bool,
) -> ([(u8, u8); 4], u8) {
    assert!(num_row_regs <= REG_FILE_SIZE);
    let mut row_y_samples = [(0u8, 0u8); 4];
    let mut num_row_y = 0usize;
    if has_control_flow {
        return (row_y_samples, 0);
    }
    for inst in kernel.iter_mut() {
        let is_row_v = matches!(
            inst.src1,
            super::ir::REG_V0_Y
                | super::ir::REG_V2_Y
                | super::ir::REG_V3_V
                | super::ir::REG_V4_Y
                | super::ir::REG_V4_W
        ) || (inst.src1 >= REG_DYNAMIC_BASE && (inst.src1 as usize) < num_row_regs);
        if inst.op == VmOp::Sample2D && inst.imm16 == 228 && is_row_v && num_row_y < 4 {
            let slot_idx = if let Some(prev) = row_y_samples[..num_row_y]
                .iter()
                .position(|&pair| pair == (inst.src2, inst.src1))
            {
                prev
            } else {
                let idx = num_row_y;
                row_y_samples[idx] = (inst.src2, inst.src1);
                num_row_y += 1;
                idx
            };
            inst.imm16 = 228 | (((slot_idx as u16) + 1) << 9);
        }
    }
    assert!(num_row_y <= 4);
    (row_y_samples, num_row_y as u8)
}

/// Compiles a parsed `DxbcShader` into a three-stage scalarized `CompiledSm5Shader` using optional
/// compile-time `cb2`, `v1` vertex color, and `TexSlotMeta` specialization hints.
pub fn compile_sm5_shader_specialized(
    shader: &DxbcShader,
    spec: ShaderSpecialization<'_>,
) -> CompiledSm5Shader {
    assert!(shader.instructions.len() <= 65_536);
    let mut builder = CompilerBuilder::new(shader, spec);

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
            69..=74 | 77 | 45 | 61 | 110 | 167 => lower_sincos_or_sample(&mut builder, inst),
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
    let (preamble_raw, row_preamble_raw, kernel_raw, srgb_encode_mask) =
        split_preamble_and_kernel(dce_insts, num_vregs, builder.use_ssa, spec.tex_meta);
    let (
        preamble,
        row_preamble,
        mut kernel,
        cb2_bindings,
        const_bindings,
        num_uniform_regs,
        num_row_regs,
        num_regs_used,
    ) = allocate_physical_regs(
        &preamble_raw,
        &row_preamble_raw,
        &kernel_raw,
        &builder.cb2_bindings,
        &builder.const_bindings,
        num_vregs,
        builder.use_ssa,
    );
    link_control_flow_jumps(&mut kernel);
    assert!(num_regs_used <= REG_FILE_SIZE);
    let (row_y_samples, num_row_y_samples) =
        tag_row_invariant_samples(&mut kernel, num_row_regs, builder.has_control_flow);

    let uv_kill_slot_ch = if !builder.has_control_flow
        && kernel.len() >= 2
        && kernel[0].op == VmOp::Sample2D
        && (kernel[0].imm16 & 0x01FF) == 228
        && matches!(
            kernel[0].src0,
            super::ir::REG_V3_U | super::ir::REG_V4_X | super::ir::REG_V4_Z
        )
        && matches!(
            kernel[0].src1,
            super::ir::REG_V3_V | super::ir::REG_V4_Y | super::ir::REG_V4_W
        )
        && kernel[1].op == VmOp::EarlyOutZero
        && kernel[1].src0 >= kernel[0].dst
        && kernel[1].src0 < kernel[0].dst + 4
    {
        Some((kernel[0].src2, kernel[1].src0 - kernel[0].dst))
    } else {
        None
    };

    let mut compiled = CompiledSm5Shader {
        preamble,
        row_preamble,
        kernel,
        cb2_bindings,
        const_bindings,
        icb_vec4s: shader.icb_vec4s.clone(),
        static_uniform_regs: Vec::new(),
        has_static_preamble: false,
        num_uniform_regs,
        num_row_regs,
        num_regs_used,
        srgb_encode_mask,
        has_control_flow: builder.has_control_flow,
        uses_indexable_temps: builder.uses_indexable_temps,
        uv_kill_slot_ch,
        row_y_samples,
        num_row_y_samples,
    };
    super::exec::populate_static_uniform_regs(&mut compiled, spec);
    compiled
}

/// Compiles a parsed `DxbcShader` into a two-stage scalarized `CompiledSm5Shader`, optionally
/// specializing static `cb2` parameters at compile time when `static_cb2` is provided.
pub fn compile_sm5_shader_with_cb2(
    shader: &DxbcShader,
    static_cb2: Option<&[[f32; 4]]>,
) -> CompiledSm5Shader {
    assert!(shader.instructions.len() <= 65_536);
    compile_sm5_shader_specialized(
        shader,
        ShaderSpecialization {
            static_cb2,
            static_v1: None,
            tex_meta: None,
        },
    )
}

/// Compiles a parsed `DxbcShader` into a two-stage scalarized `CompiledSm5Shader` with dynamic `cb2`.
pub fn compile_sm5_shader(shader: &DxbcShader) -> CompiledSm5Shader {
    assert!(shader.instructions.len() <= 65_536);
    compile_sm5_shader_with_cb2(shader, None)
}

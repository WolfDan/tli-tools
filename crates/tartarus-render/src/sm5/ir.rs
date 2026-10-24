//! Scalarized SIMT Virtual Machine Instruction Set Architecture (`VmOp`, `VmInst`) and
//! unified 256-register file layout (`CompiledSm5Shader`) for Shader Model 5 (`ps_5_0`).

use std::simd::Simd;

/// 8-lane SIMD wavefront width (`4x2` pixel tile = two adjacent `2x2` pixel quads).
///
/// Lane ordering inside every `f32x8` / `u32x8` vector:
/// - Quad 0: `[(x+0, y+0), (x+1, y+0), (x+0, y+1), (x+1, y+1)]` (lanes `0..4`)
/// - Quad 1: `[(x+2, y+0), (x+3, y+0), (x+2, y+1), (x+3, y+1)]` (lanes `4..8`)
pub const SIMD_LANES: usize = 8;

/// Total scalar registers in the unified register file (`0..=255`, indexed by `u8` with
/// zero bounds checks in safe Rust).
pub const REG_FILE_SIZE: usize = 256;

/// Maximum compiled scalar instructions per stage.
pub const MAX_STAGE_INSTS: usize = 4096;

/// Maximum dynamic loop/branch execution steps per tile to guarantee termination.
pub const MAX_EXEC_STEPS: usize = 16_384;

/// Maximum control-flow nesting depth (`if`/`else`/`loop`/`switch`).
pub const MAX_CF_DEPTH: usize = 32;

/// Maximum indexable temporary array elements (`dcl_indexableTemp x#`).
pub const MAX_INDEXABLE_TEMP_LEN: usize = 64;

#[allow(non_camel_case_types)]
pub type f32x8 = Simd<f32, SIMD_LANES>;
#[allow(non_camel_case_types)]
pub type u32x8 = Simd<u32, SIMD_LANES>;
#[allow(non_camel_case_types)]
pub type i32x8 = Simd<i32, SIMD_LANES>;

// Fixed architectural scalar register assignments in `Regs[0..41]`:
// Register 0 is `REG_ZERO` (constant 0.0 / 0u32), so unused `src1 = 0` / `src2 = 0` fields in
// unary/binary `VmInst` instructions naturally reference a uniform constant register.
pub const REG_ZERO: u8 = 0;

pub const REG_OUT_R: u8 = 1;
pub const REG_OUT_G: u8 = 2;
pub const REG_OUT_B: u8 = 3;
pub const REG_OUT_A: u8 = 4;

pub const REG_V0_X: u8 = 5;
pub const REG_V0_Y: u8 = 6;
pub const REG_V0_Z: u8 = 7;
pub const REG_V0_W: u8 = 8;

pub const REG_V1_R: u8 = 9;
pub const REG_V1_G: u8 = 10;
pub const REG_V1_B: u8 = 11;
pub const REG_V1_A: u8 = 12;

pub const REG_V2_X: u8 = 13;
pub const REG_V2_Y: u8 = 14;
pub const REG_V2_Z: u8 = 15;
pub const REG_V2_W: u8 = 16;

pub const REG_V3_U: u8 = 17;
pub const REG_V3_V: u8 = 18;
pub const REG_V3_Z: u8 = 19;
pub const REG_V3_W: u8 = 20;

pub const REG_V4_X: u8 = 21;
pub const REG_V4_Y: u8 = 22;
pub const REG_V4_Z: u8 = 23;
pub const REG_V4_W: u8 = 24;

pub const REG_CB0_1_X: u8 = 25;
pub const REG_CB0_1_Y: u8 = 26;
pub const REG_CB0_1_Z: u8 = 27;
pub const REG_CB0_1_W: u8 = 28;

pub const REG_REAL_TIME: u8 = 29;
pub const REG_ONE: u8 = 30;
pub const REG_TRUE: u8 = 31;
pub const REG_NULL: u8 = 32;

/// Scratch registers reserved for non-SSA fallback lowering.
pub const REG_MOD0: u8 = 33;
pub const REG_MOD1: u8 = 34;
pub const REG_MOD2: u8 = 35;
pub const REG_DOT_ACC: u8 = 36;
pub const REG_ALIAS_TMP0: u8 = 37;
pub const REG_ALIAS_TMP1: u8 = 38;
pub const REG_ALIAS_TMP2: u8 = 39;
pub const REG_ALIAS_TMP3: u8 = 40;

/// First dynamically allocated register for shader `r#` temps, `cb2` uniforms, and constants.
pub const REG_DYNAMIC_BASE: u8 = 41;

const _: () = {
    assert!(SIMD_LANES == 8);
    assert!(REG_FILE_SIZE == 256);
    assert!(core::mem::size_of::<VmInst>() == 8);
};

/// Scalarized SIMT virtual machine opcode (`1` byte).
/// Saturation (`_sat`) is lowered to `VmOp::MovSat` (`clamp(x, 0.0, 1.0)`) so all primary ALU
/// instructions execute without per-instruction saturation branches.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum VmOp {
    #[default]
    Nop = 0,
    Mov,
    MovSat,
    FNeg,
    FAbs,
    FNegAbs,
    Movc,

    // IEEE-754 Float Binary & Ternary ALU
    FAdd,
    FSub,
    FMul,
    FMul3,
    FMad,
    FNmad,
    FDiv,
    FMin,
    FMax,

    // Float Unary & Transcendental ALU
    FFrc,
    FRoundNe,
    FFloor,
    FCeil,
    FTrunc,
    FSqrt,
    FRsq,
    FRcp,
    FExp2,
    FLog2,
    FPow,
    FAtan2,
    LinearToSrgb,
    FSin,
    FCos,

    // 2x2 Quad Screen-Space Derivatives
    FDdx,
    FDdy,

    // Float Comparisons (0xFFFF_FFFF for true, 0x0000_0000 for false)
    FCmpEq,
    FCmpNe,
    FCmpLt,
    FCmpGe,

    // Integer Comparisons
    ICmpEq,
    ICmpNe,
    ICmpLt,
    ICmpGe,
    UCmpLt,
    UCmpGe,

    // Integer & Bitwise ALU
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    IAdd,
    INeg,
    IAbs,
    IMulLo,
    IMulHi,
    UMulHi,
    IMad,
    UDivQuot,
    UDivRem,
    IMin,
    IMax,
    UMin,
    UMax,
    IShl,
    IShr,
    UShr,
    UAddCarry,
    USubBorrow,

    // Numeric Conversions & Bit Manipulation
    FToI,
    FToU,
    IToF,
    UToF,
    F32ToF16,
    F16ToF32,
    CountBits,
    FirstBitHi,
    FirstBitLo,
    FirstBitShi,
    Bfrev,
    Ubfe,
    Ibfe,
    Bfi,

    // Dynamic Buffer & Indexable Temp Access
    LoadCb2Dyn,
    LoadIcbDyn,
    LoadIdxTemp,
    StoreIdxTemp,

    // Texture & Resource Sampling (writes up to 4 consecutive registers `dst..dst+3`)
    Sample2D,
    TexLoad2D,
    ResInfo2D,
    Gather4,

    // Structured Control Flow
    IfZ,
    IfNz,
    Else,
    EndIf,
    Loop,
    EndLoop,
    Break,
    BreakZ,
    BreakNz,
    Continue,
    ContinueZ,
    ContinueNz,
    DiscardZ,
    DiscardNz,
    EarlyOutZero,
    Ret,
    RetZ,
    RetNz,
}

/// Fixed-width 64-bit (`8`-byte) scalarized VM instruction. Eight instructions fit in one
/// 64-byte CPU L1 cache line with zero heap pointers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C, align(8))]
pub struct VmInst {
    pub op: VmOp,
    pub dst: u8,
    pub src0: u8,
    pub src1: u8,
    pub src2: u8,
    pub aux: u8,
    pub imm16: u16,
}

impl VmInst {
    #[inline]
    pub const fn new(op: VmOp, dst: u8, src0: u8, src1: u8, src2: u8) -> Self {
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
    pub const fn with_aux_imm(
        op: VmOp,
        dst: u8,
        src0: u8,
        src1: u8,
        src2: u8,
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

/// Binding entry mapping a `cb2[vec_idx].comp` scalar parameter into the unified register file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cb2RegBinding {
    pub dst_reg: u8,
    pub comp: u8,
    pub vec_idx: u16,
}

/// Binding entry mapping an immediate constant `u32` bit-pattern into the unified register file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstRegBinding {
    pub dst_reg: u8,
    pub bits: u32,
}

/// Pre-compiled, scalarized two-stage SIMT pixel shader program ready for 8-wide SIMD execution.
#[derive(Clone, Debug)]
pub struct CompiledSm5Shader {
    /// Stage 1: Uniform instructions executed **once per quad** on scalar registers.
    pub preamble: Vec<VmInst>,
    /// Stage 2: Pixel-varying instructions executed per `4x2` tile on `f32x8` / `u32x8` SIMD registers.
    pub kernel: Vec<VmInst>,
    /// Static `cb2` scalar components loaded into the register file before running `preamble`.
    pub cb2_bindings: Vec<Cb2RegBinding>,
    /// Deduplicated immediate constants loaded into the register file at setup time.
    pub const_bindings: Vec<ConstRegBinding>,
    /// Immediate constant buffer (`icb`) vec4 table for dynamic `icb[r0.x + K]` loads.
    pub icb_vec4s: Vec<[u32; 4]>,
    /// Number of persistent uniform/preamble registers in `[0..num_uniform_regs]` (`<= num_regs_used`).
    pub num_uniform_regs: usize,
    /// Total registers used in `[0..num_regs_used]` (`<= 256`).
    pub num_regs_used: usize,
    /// True if `kernel` contains any divergent branch, loop, or discard instructions.
    pub has_control_flow: bool,
    /// True if any instruction in `kernel` uses dynamic indexable temporary arrays (`x#`).
    pub uses_indexable_temps: bool,
    /// Optional `(slot_idx, kill_ch)` when `kernel[0]` samples undistorted quad `(u, v)` and
    /// `kernel[1]` is `EarlyOutZero` on channel `kill_ch`, enabling $O(1)$ UV bounding-box trimming.
    pub uv_kill_slot_ch: Option<(u8, u8)>,
}

/// Compile-time metadata for a bound `LinearTexture2D` slot.
#[derive(Clone, Copy, Debug)]
pub struct TexSlotMeta {
    /// Earliest slot index bound to the exact same texture and UV clamp mode.
    pub canonical_slot: u8,
    /// Bitmask (`bits 0..4`) of channels that contain at least one `0.0` texel.
    pub has_zeros_mask: u8,
    /// Quantized fraction (`0..=255`) of `0.0` texels in each channel (`R, G, B, A`).
    pub zero_frac: [u8; 4],
    /// Channels (`R, G, B, A`) that have a constant value across the entire texture.
    pub const_val: [Option<f32>; 4],
    /// Channels (`G, B, A`) that are bit-for-bit identical to channel `0` (`R`).
    pub eq_r: [bool; 4],
}

/// Optional compile-time specialization inputs for `compile_sm5_shader_specialized`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ShaderSpecialization<'a> {
    pub static_cb2: Option<&'a [[f32; 4]]>,
    pub static_v1: Option<[f32; 4]>,
    pub tex_meta: Option<&'a [TexSlotMeta]>,
}

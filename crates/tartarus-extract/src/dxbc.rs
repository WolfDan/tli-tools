//! Pure-Rust DirectX Shader Model 5 (`DXBC` / `SHEX` / `SHDR` / `ISGN` / `OSGN`) parser,
//! SM5 `.asm` disassembler, and structured `.hlsl` decompiler.

use byteorder::{ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};

use crate::error::ExtractError;
use crate::material::MaterialInfo;

pub const DXBC_MAGIC: u32 = 0x4342_5844; // "DXBC"
const CHUNKS_MAX: usize = 64;
const SIG_ELEMENTS_MAX: usize = 256;
const SHEX_DWORDS_MAX: usize = 262_144;
const INSTRUCTIONS_MAX: usize = 65_536;
const OPERANDS_PER_INST_MAX: usize = 8;

const _: () = {
    assert!(DXBC_MAGIC == 0x4342_5844);
    assert!(OPERANDS_PER_INST_MAX == 8);
};

/// Element in a `DXBC` `ISGN`, `OSGN`, or `PSGN` input/output signature table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DxbcSignatureElement {
    pub semantic_name: String,
    pub semantic_index: u32,
    pub system_value_type: u32,
    pub component_type: u32,
    pub register: u32,
    pub mask: u8,
    pub rw_mask: u8,
}

/// Decoded SM5 operand from a `SHEX` / `SHDR` instruction token stream.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sm5Operand {
    pub op_type: u32,
    pub num_comps: u8,
    pub sel_mode: u8,
    pub mask: u8,
    pub swizzle: [u8; 4],
    pub neg: bool,
    pub abs: bool,
    pub indices: Vec<u32>,
    pub dynamic_index: Option<String>,
    pub imm_u32: [u32; 4],
}

/// Decoded SM5 instruction from a `SHEX` / `SHDR` token stream.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sm5Instruction {
    pub opcode: u32,
    pub opcode_token0: u32,
    pub saturate: bool,
    pub test_nonzero: bool,
    pub res_dim: u8,
    pub operands: Vec<Sm5Operand>,
    pub raw_dwords: Vec<u32>,
}

/// Parsed DirectX Shader Model 4/5 `DXBC` container.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DxbcShader {
    pub program_type: u16,
    pub major_version: u8,
    pub minor_version: u8,
    pub total_size: u32,
    pub chunk_fourccs: Vec<String>,
    pub input_signature: Vec<DxbcSignatureElement>,
    pub output_signature: Vec<DxbcSignatureElement>,
    pub instructions: Vec<Sm5Instruction>,
    pub icb_vec4s: Vec<[u32; 4]>,
}

fn read_cstr(buf: &[u8], offset: usize) -> String {
    assert!(offset <= buf.len());
    let slice = &buf[offset..];
    let end = memchr::memchr(0, slice).unwrap_or(slice.len()).min(128);
    assert!(end <= slice.len());
    String::from_utf8_lossy(&slice[..end]).to_string()
}

fn parse_signature_chunk(chunk_data: &[u8]) -> Vec<DxbcSignatureElement> {
    assert!(chunk_data.len() <= 1024 * 1024);
    if chunk_data.len() < 8 {
        return Vec::new();
    }
    let count = (LittleEndian::read_u32(&chunk_data[0..4]) as usize).min(SIG_ELEMENTS_MAX);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let off = 8 + i * 24;
        let Some(entry) = chunk_data.get(off..off + 24) else {
            break;
        };
        let name_off = LittleEndian::read_u32(&entry[0..4]) as usize;
        let semantic_name = if name_off < chunk_data.len() {
            read_cstr(chunk_data, name_off)
        } else {
            String::from("UNKNOWN")
        };
        out.push(DxbcSignatureElement {
            semantic_name,
            semantic_index: LittleEndian::read_u32(&entry[4..8]),
            system_value_type: LittleEndian::read_u32(&entry[8..12]),
            component_type: LittleEndian::read_u32(&entry[12..16]),
            register: LittleEndian::read_u32(&entry[16..20]),
            mask: entry[20],
            rw_mask: entry[21],
        });
    }
    assert!(out.len() <= SIG_ELEMENTS_MAX);
    out
}

fn swizzle_str(swz: [u8; 4], count: usize) -> String {
    assert!((1..=4).contains(&count));
    const COMP: [char; 4] = ['x', 'y', 'z', 'w'];
    let mut s = String::with_capacity(4);
    for &c in swz.iter().take(count) {
        s.push(COMP[(c as usize) & 3]);
    }
    assert_eq!(s.len(), count);
    s
}

fn mask_str(mask: u8) -> String {
    const COMP: [char; 4] = ['x', 'y', 'z', 'w'];
    let mut s = String::with_capacity(4);
    for (i, &ch) in COMP.iter().enumerate() {
        if (mask >> i) & 1 != 0 {
            s.push(ch);
        }
    }
    if s.is_empty() {
        s.push_str("xyzw");
    }
    assert!(!s.is_empty());
    assert!(s.len() <= 4);
    s
}

fn decode_index_reg_token(dwords: &[u32], mut pos: usize) -> (String, usize) {
    assert!(pos < dwords.len());
    let tok0 = dwords[pos];
    pos += 1;
    let op_type = (tok0 >> 12) & 0xFF;
    let comp = ((tok0 >> 4) & 0x3) as usize;
    if (tok0 >> 31) & 1 != 0 && pos < dwords.len() {
        pos += 1;
    }
    let reg_idx = if pos < dwords.len() {
        let v = dwords[pos];
        pos += 1;
        v
    } else {
        0
    };
    let prefix = if op_type == 0 { "r" } else { "v" };
    let ch = ['x', 'y', 'z', 'w'][comp & 3];
    let formatted = format!("{prefix}{reg_idx}.{ch}");
    assert!(!formatted.is_empty());
    (formatted, pos)
}

fn decode_sm5_operand(dwords: &[u32], start: usize) -> Option<(Sm5Operand, usize)> {
    assert!(start <= dwords.len());
    let tok0 = *dwords.get(start)?;
    let mut pos = start + 1;
    let num_comps = (tok0 & 0x3) as u8;
    let sel_mode = ((tok0 >> 2) & 0x3) as u8;
    let op_type = (tok0 >> 12) & 0xFF;
    let idx_dim = ((tok0 >> 20) & 0x3) as usize;

    let mut mask = 0x0Fu8;
    let mut swizzle = [0u8, 1, 2, 3];
    if num_comps == 2 {
        match sel_mode {
            0 => mask = ((tok0 >> 4) & 0xF) as u8,
            1 => {
                for (i, dst) in swizzle.iter_mut().enumerate() {
                    *dst = ((tok0 >> (4 + i * 2)) & 0x3) as u8;
                }
            }
            2 => {
                let c = ((tok0 >> 4) & 0x3) as u8;
                swizzle = [c, c, c, c];
                mask = 1 << c;
            }
            _ => {}
        }
    }

    let mut neg = false;
    let mut abs = false;
    if (tok0 >> 31) & 1 != 0 {
        let ext = *dwords.get(pos)?;
        pos += 1;
        let mod_bits = (ext >> 6) & 0xFF;
        neg = (mod_bits & 1) != 0;
        abs = (mod_bits & 2) != 0;
    }

    let mut imm_u32 = [0u32; 4];
    if op_type == 4 || op_type == 5 {
        let n = if num_comps == 2 { 4 } else { 1 };
        for dst in imm_u32.iter_mut().take(n) {
            *dst = *dwords.get(pos)?;
            pos += 1;
        }
    }

    let mut indices = Vec::with_capacity(idx_dim);
    let mut dynamic_index = None;
    for d in 0..idx_dim {
        let rep = (tok0 >> (22 + d * 3)) & 0x7;
        match rep {
            0 => {
                indices.push(*dwords.get(pos)?);
                pos += 1;
            }
            1 => {
                indices.push(*dwords.get(pos)?);
                pos += 2;
            }
            2 => {
                if pos >= dwords.len() {
                    return None;
                }
                let (dyn_s, next_p) = decode_index_reg_token(dwords, pos);
                pos = next_p;
                dynamic_index = Some(dyn_s);
                indices.push(0);
            }
            3 => {
                let imm = *dwords.get(pos)?;
                pos += 1;
                if pos >= dwords.len() {
                    return None;
                }
                let (dyn_s, next_p) = decode_index_reg_token(dwords, pos);
                pos = next_p;
                dynamic_index = Some(format!("{dyn_s} + {imm}"));
                indices.push(imm);
            }
            _ => break,
        }
    }

    assert!(pos > start);
    Some((
        Sm5Operand {
            op_type,
            num_comps,
            sel_mode,
            mask,
            swizzle,
            neg,
            abs,
            indices,
            dynamic_index,
            imm_u32,
        },
        pos,
    ))
}

fn opcode_has_extra_header_dwords(opcode: u32) -> usize {
    let skip = match opcode {
        // dcl_temps (104), dcl_indexableTemp (105), dcl_globalFlags (106), dcl_thread_group (155)
        104 | 105 | 106 | 108 | 115 | 149 | 150 | 151 | 152 | 153 | 154 | 155 => 99,
        159 | 160 => 99,
        _ => 0,
    };
    assert!(skip <= 99);
    assert!(skip == 0 || skip == 99);
    skip
}

fn decode_shex_instruction(dwords: &[u32]) -> Sm5Instruction {
    assert!(!dwords.is_empty());
    let tok0 = dwords[0];
    let opcode = tok0 & 0x7FF;
    let saturate = ((tok0 >> 13) & 1) != 0;
    let test_nonzero = ((tok0 >> 18) & 1) != 0;
    let mut res_dim = ((tok0 >> 11) & 0x1F) as u8;

    let mut pos = 1usize;
    let mut ext_bit = (tok0 >> 31) & 1 != 0;
    for _ in 0..4 {
        if !ext_bit || pos >= dwords.len() {
            break;
        }
        let ext_tok = dwords[pos];
        pos += 1;
        if (ext_tok & 0x3F) == 2 {
            res_dim = ((ext_tok >> 6) & 0x1F) as u8;
        }
        ext_bit = (ext_tok >> 31) & 1 != 0;
    }

    let mut operands = Vec::new();
    if opcode_has_extra_header_dwords(opcode) == 0 {
        for _ in 0..OPERANDS_PER_INST_MAX {
            if pos >= dwords.len() {
                break;
            }
            if matches!(opcode, 88 | 156 | 157 | 158 | 161 | 162) && operands.len() == 1 {
                break;
            }
            let Some((op, next_pos)) = decode_sm5_operand(dwords, pos) else {
                break;
            };
            operands.push(op);
            pos = next_pos;
        }
    }

    assert!(operands.len() <= OPERANDS_PER_INST_MAX);
    Sm5Instruction {
        opcode,
        opcode_token0: tok0,
        saturate,
        test_nonzero,
        res_dim,
        operands,
        raw_dwords: dwords.to_vec(),
    }
}

struct ParsedShexProgram {
    program_type: u16,
    major: u8,
    minor: u8,
    instructions: Vec<Sm5Instruction>,
    icb_vec4s: Vec<[u32; 4]>,
}

fn parse_shex_chunk(chunk_data: &[u8]) -> Result<ParsedShexProgram, ExtractError> {
    assert!(chunk_data.len() <= SHEX_DWORDS_MAX * 4);
    if chunk_data.len() < 8 || !chunk_data.len().is_multiple_of(4) {
        return Err(ExtractError::ShaderFormat(format!(
            "Invalid SHEX/SHDR chunk byte length {}",
            chunk_data.len()
        )));
    }
    let dword_count = chunk_data.len() / 4;
    let mut dwords = vec![0u32; dword_count];
    LittleEndian::read_u32_into(chunk_data, &mut dwords);

    let ver_tok = dwords[0];
    let program_type = ((ver_tok >> 16) & 0xFFFF) as u16;
    let major = ((ver_tok >> 4) & 0xF) as u8;
    let minor = (ver_tok & 0xF) as u8;

    let mut instructions = Vec::new();
    let mut icb_vec4s = Vec::new();
    let mut idx = 2usize;
    let mut steps = 0usize;

    while idx < dwords.len() && steps < INSTRUCTIONS_MAX {
        steps += 1;
        let tok0 = dwords[idx];
        let opcode = tok0 & 0x7FF;
        if opcode == 53 {
            let custom_len = dwords.get(idx + 1).copied().unwrap_or(2).max(2) as usize;
            let end = (idx + custom_len).min(dwords.len());
            if ((tok0 >> 11) == 3) && end > idx + 2 {
                for &v4 in dwords[idx + 2..end].as_chunks::<4>().0 {
                    icb_vec4s.push(v4);
                }
            }
            idx = end;
            continue;
        }
        let ilen = (((tok0 >> 24) & 0x7F) as usize).max(1);
        let end = (idx + ilen).min(dwords.len());
        instructions.push(decode_shex_instruction(&dwords[idx..end]));
        idx = end;
    }

    assert!(instructions.len() <= INSTRUCTIONS_MAX);
    Ok(ParsedShexProgram {
        program_type,
        major,
        minor,
        instructions,
        icb_vec4s,
    })
}

/// Parses a DirectX `DXBC` container into its signatures (`ISGN`, `OSGN`) and decoded SM5
/// instructions (`SHEX` / `SHDR`).
pub fn parse_dxbc(dxbc: &[u8]) -> Result<DxbcShader, ExtractError> {
    assert!(!dxbc.is_empty());
    if dxbc.len() < 32 || LittleEndian::read_u32(&dxbc[0..4]) != DXBC_MAGIC {
        return Err(ExtractError::ShaderFormat(
            "Invalid DXBC container magic or truncated header".to_string(),
        ));
    }
    let total_size = LittleEndian::read_u32(&dxbc[24..28]);
    let chunk_count = (LittleEndian::read_u32(&dxbc[28..32]) as usize).min(CHUNKS_MAX);
    if 32 + chunk_count * 4 > dxbc.len() {
        return Err(ExtractError::ShaderFormat(
            "DXBC chunk offset table out of bounds".to_string(),
        ));
    }

    let mut chunk_fourccs = Vec::with_capacity(chunk_count);
    let mut input_signature = Vec::new();
    let mut output_signature = Vec::new();
    let mut program_type = 0u16;
    let mut major_version = 5u8;
    let mut minor_version = 0u8;
    let mut instructions = Vec::new();
    let mut icb_vec4s = Vec::new();

    for i in 0..chunk_count {
        let c_off = LittleEndian::read_u32(&dxbc[32 + i * 4..36 + i * 4]) as usize;
        let Some(c_hdr) = dxbc.get(c_off..c_off + 8) else {
            continue;
        };
        let fourcc = String::from_utf8_lossy(&c_hdr[0..4]).to_string();
        let c_size = LittleEndian::read_u32(&c_hdr[4..8]) as usize;
        let Some(c_data) = dxbc.get(c_off + 8..c_off + 8 + c_size) else {
            continue;
        };
        match fourcc.as_str() {
            "ISGN" => input_signature = parse_signature_chunk(c_data),
            "OSGN" => output_signature = parse_signature_chunk(c_data),
            "SHEX" | "SHDR" => {
                let prog = parse_shex_chunk(c_data)?;
                program_type = prog.program_type;
                major_version = prog.major;
                minor_version = prog.minor;
                instructions = prog.instructions;
                icb_vec4s = prog.icb_vec4s;
            }
            _ => {}
        }
        chunk_fourccs.push(fourcc);
    }

    assert!(chunk_fourccs.len() <= CHUNKS_MAX);
    Ok(DxbcShader {
        program_type,
        major_version,
        minor_version,
        total_size,
        chunk_fourccs,
        input_signature,
        output_signature,
        instructions,
        icb_vec4s,
    })
}

fn format_imm32_scalar(bits: u32) -> String {
    let f = f32::from_bits(bits);
    let out = if bits == 0 {
        String::from("0.0")
    } else if f.is_finite() && (1.0e-4..=1.0e6).contains(&f.abs()) {
        format!("{f:.6}")
    } else if f.is_finite() && (1.0e-12..1.0e-4).contains(&f.abs()) {
        format!("{f:.6e}")
    } else if (bits as i32).abs() <= 65536 {
        format!("{}", bits as i32)
    } else {
        format!("0x{bits:08x}")
    };
    assert!(!out.is_empty());
    assert!(out.len() <= 32);
    out
}

/// Formats a decoded `Sm5Operand` for `.asm` or `.hlsl` emission.
pub fn format_sm5_operand(op: &Sm5Operand, is_dest: bool) -> String {
    assert!(op.num_comps <= 3);
    if op.op_type == 4 {
        if op.num_comps == 2 {
            return format!(
                "float4({}, {}, {}, {})",
                format_imm32_scalar(op.imm_u32[0]),
                format_imm32_scalar(op.imm_u32[1]),
                format_imm32_scalar(op.imm_u32[2]),
                format_imm32_scalar(op.imm_u32[3]),
            );
        }
        return format_imm32_scalar(op.imm_u32[0]);
    }

    let idx0 = op.indices.first().copied().unwrap_or(0);
    let idx1 = op.indices.get(1).copied().unwrap_or(0);
    let base = match op.op_type {
        0 => format!("r{idx0}"),
        1 => format!("v{idx0}"),
        2 => format!("o{idx0}"),
        3 => {
            if let Some(dyn_s) = &op.dynamic_index {
                format!("x{idx0}[{dyn_s}]")
            } else {
                format!("x{idx0}[{idx1}]")
            }
        }
        6 => format!("s{idx0}"),
        7 => format!("t{idx0}"),
        8 => {
            if let Some(dyn_s) = &op.dynamic_index {
                format!("cb{idx0}[{dyn_s}]")
            } else {
                format!("cb{idx0}[{idx1}]")
            }
        }
        9 => {
            if let Some(dyn_s) = &op.dynamic_index {
                format!("icb[{dyn_s}]")
            } else {
                format!("icb[{idx0}]")
            }
        }
        11 => String::from("vPrim"),
        12 => String::from("oDepth"),
        13 => String::from("null"),
        15 => String::from("oMask"),
        29 => format!("u{idx0}"),
        30 => format!("g{idx0}"),
        32 => String::from("vThreadID"),
        33 => String::from("vThreadGroupID"),
        34 => String::from("vThreadIDInGroup"),
        35 => String::from("vCoverage"),
        36 => String::from("vThreadIDInGroupFlattened"),
        _ => format!("reg{}_{idx0}", op.op_type),
    };

    let suffix = if op.num_comps == 2 && !matches!(op.op_type, 6 | 7 | 13) {
        if is_dest || op.sel_mode == 0 {
            format!(".{}", mask_str(op.mask))
        } else if op.sel_mode == 2 {
            format!(".{}", swizzle_str(op.swizzle, 1))
        } else {
            format!(".{}", swizzle_str(op.swizzle, 4))
        }
    } else {
        String::new()
    };

    let mut expr = format!("{base}{suffix}");
    if op.abs {
        expr = format!("abs({expr})");
    }
    if op.neg {
        expr = format!("-{expr}");
    }
    assert!(!expr.is_empty());
    expr
}

/// Returns the canonical D3D11 SM5 mnemonic for `opcode`.
pub fn sm5_opcode_mnemonic(opcode: u32) -> &'static str {
    let name = match opcode {
        0 => "add",
        1 => "and",
        2 => "break",
        3 => "breakc",
        6 => "case",
        7 => "continue",
        8 => "continuec",
        9 => "cut",
        10 => "default",
        11 => "deriv_rtx",
        12 => "deriv_rty",
        13 => "discard",
        14 => "div",
        15 => "dp2",
        16 => "dp3",
        17 => "dp4",
        18 => "else",
        19 => "emit",
        21 => "endif",
        22 => "endloop",
        23 => "endswitch",
        24 => "eq",
        25 => "exp",
        26 => "frc",
        27 => "ftoi",
        28 => "ftou",
        29 => "ge",
        30 => "iadd",
        31 => "if",
        32 => "ieq",
        33 => "ige",
        34 => "ilt",
        35 => "imad",
        36 => "imax",
        37 => "imin",
        38 => "imul",
        39 => "ine",
        40 => "ineg",
        41 => "ishl",
        42 => "ishr",
        43 => "itof",
        44 => "label",
        45 => "ld",
        46 => "ld_ms",
        47 => "log",
        48 => "loop",
        49 => "lt",
        50 => "mad",
        51 => "min",
        52 => "max",
        54 => "mov",
        55 => "movc",
        56 => "mul",
        57 => "ne",
        58 => "nop",
        59 => "not",
        60 => "or",
        61 => "resinfo",
        62 => "ret",
        63 => "retc",
        64 => "round_ne",
        65 => "round_ni",
        66 => "round_pi",
        67 => "round_z",
        68 => "rsq",
        69 => "sample",
        70 => "sample_c",
        71 => "sample_c_lz",
        72 => "sample_l",
        73 => "sample_d",
        74 => "sample_b",
        75 => "sqrt",
        76 => "switch",
        77 => "sincos",
        78 => "udiv",
        79 => "ult",
        80 => "uge",
        81 => "umul",
        82 => "umad",
        83 => "umax",
        84 => "umin",
        85 => "ushr",
        86 => "utof",
        87 => "xor",
        88 => "dcl_resource",
        89 => "dcl_constantbuffer",
        90 => "dcl_sampler",
        91..=98 => "dcl_input",
        99..=103 => "dcl_output",
        104 => "dcl_temps",
        105 => "dcl_indexableTemp",
        106 => "dcl_globalFlags",
        109 => "lod",
        110 => "gather4",
        122 => "deriv_rtx_coarse",
        123 => "deriv_rtx_fine",
        124 => "deriv_rty_coarse",
        125 => "deriv_rty_fine",
        129 => "rcp",
        130 => "f32tof16",
        131 => "f16tof32",
        132 => "uaddc",
        133 => "usubb",
        134 => "countbits",
        135 => "firstbit_hi",
        136 => "firstbit_lo",
        137 => "firstbit_shi",
        138 => "ubfe",
        139 => "ibfe",
        140 => "bfi",
        141 => "bfrev",
        142 => "swapc",
        147 => "dcl_stream",
        155 => "dcl_thread_group",
        156 => "dcl_uav_typed",
        157 => "dcl_uav_raw",
        158 => "dcl_uav_structured",
        159 => "dcl_tgsm_raw",
        160 => "dcl_tgsm_structured",
        161 => "dcl_resource_raw",
        162 => "dcl_resource_structured",
        163 => "ld_uav_typed",
        164 => "store_uav_typed",
        165 => "ld_raw",
        166 => "store_raw",
        167 => "ld_structured",
        168 => "store_structured",
        169 => "atomic_iadd",
        173 => "atomic_imin",
        174 => "atomic_imax",
        175 => "atomic_umin",
        176 => "atomic_umax",
        177 => "imm_atomic_alloc",
        178 => "imm_atomic_consume",
        179 => "imm_atomic_iadd",
        190 => "sync",
        _ => "op_unknown",
    };
    assert!(!name.is_empty());
    name
}

fn program_prefix(program_type: u16) -> &'static str {
    let p = match program_type {
        0 => "ps",
        1 => "vs",
        2 => "gs",
        3 => "hs",
        4 => "ds",
        5 => "cs",
        _ => "sh",
    };
    assert!(!p.is_empty());
    p
}

/// Disassembles a parsed `DxbcShader` into readable D3D11 Shader Model 5 assembly (`.asm`).
pub fn disassemble_dxbc(shader: &DxbcShader) -> String {
    assert!(shader.instructions.len() <= INSTRUCTIONS_MAX);
    let mut out = String::with_capacity(4096);
    out.push_str(&format!(
        "// DirectX Shader Model {}.{} ({}_{}_{})\n",
        shader.major_version,
        shader.minor_version,
        program_prefix(shader.program_type),
        shader.major_version,
        shader.minor_version
    ));
    if !shader.input_signature.is_empty() {
        out.push_str("// Input Signature:\n");
        for e in &shader.input_signature {
            out.push_str(&format!(
                "//   v{:<2} {:<16} idx={:<2} mask=.{}\n",
                e.register,
                e.semantic_name,
                e.semantic_index,
                mask_str(e.mask)
            ));
        }
    }
    if !shader.output_signature.is_empty() {
        out.push_str("// Output Signature:\n");
        for e in &shader.output_signature {
            out.push_str(&format!(
                "//   o{:<2} {:<16} idx={:<2} mask=.{}\n",
                e.register,
                e.semantic_name,
                e.semantic_index,
                mask_str(e.mask)
            ));
        }
    }
    out.push_str(&format!(
        "{}_{}_{}\n",
        program_prefix(shader.program_type),
        shader.major_version,
        shader.minor_version
    ));

    let mut indent = 0usize;
    for inst in &shader.instructions {
        if matches!(inst.opcode, 18 | 21 | 22 | 23 | 6 | 10) {
            indent = indent.saturating_sub(1);
        }
        let pad = "  ".repeat(indent);
        let mnem = sm5_opcode_mnemonic(inst.opcode);
        let sat = if inst.saturate { "_sat" } else { "" };
        if inst.opcode == 104 && inst.raw_dwords.len() >= 2 {
            out.push_str(&format!("{pad}dcl_temps {}\n", inst.raw_dwords[1]));
        } else if inst.opcode == 155 && inst.raw_dwords.len() >= 4 {
            out.push_str(&format!(
                "{pad}dcl_thread_group {}, {}, {}\n",
                inst.raw_dwords[1], inst.raw_dwords[2], inst.raw_dwords[3]
            ));
        } else if inst.operands.is_empty() {
            out.push_str(&format!("{pad}{mnem}{sat}\n"));
        } else {
            let ops: Vec<String> = inst
                .operands
                .iter()
                .enumerate()
                .map(|(i, op)| format_sm5_operand(op, i == 0 && inst.operands.len() > 1))
                .collect();
            out.push_str(&format!("{pad}{mnem}{sat} {}\n", ops.join(", ")));
        }
        if matches!(inst.opcode, 31 | 18 | 48 | 76 | 6 | 10) {
            indent = (indent + 1).min(16);
        }
    }
    assert!(!out.is_empty());
    out
}

fn emit_hlsl_material_header(out: &mut String, mat: &MaterialInfo) {
    assert!(!mat.virtual_path.is_empty());
    out.push_str(
        "// ============================================================================\n",
    );
    out.push_str(&format!("// Material Asset : {}\n", mat.virtual_path));
    if let Some(parent) = &mat.parent_path {
        out.push_str(&format!("// Parent         : {parent}\n"));
    }
    if mat.resolved_shader_source_path != mat.virtual_path {
        out.push_str(&format!(
            "// Shader Source  : {} (inherited via Parent chain)\n",
            mat.resolved_shader_source_path
        ));
    }
    if let Some(hash) = &mat.shader_map_hash_hex {
        out.push_str(&format!("// ShaderMapHash  : {hash}\n"));
    }
    out.push_str(&format!(
        "// Blend / Shading: {:?} / {:?}\n",
        mat.blend_mode.as_deref().unwrap_or("Default"),
        mat.shading_model.as_deref().unwrap_or("Default")
    ));
    if !mat.static_switches.is_empty() {
        out.push_str("// Static Switch Permutation:\n");
        for sw in &mat.static_switches {
            out.push_str(&format!("//   [Switch] {:<28} = {}\n", sw.name, sw.value));
        }
    }
    if !mat.static_component_masks.is_empty() {
        out.push_str("// Static Component Masks:\n");
        for m in &mat.static_component_masks {
            out.push_str(&format!(
                "//   [Mask]   {:<28} = RGBA({}, {}, {}, {})\n",
                m.name, m.r as u8, m.g as u8, m.b as u8, m.a as u8
            ));
        }
    }
    if !mat.scalar_params.is_empty() {
        out.push_str("// Scalar Parameter Overrides:\n");
        for s in &mat.scalar_params {
            out.push_str(&format!("//   [Scalar] {:<28} = {:.5}\n", s.name, s.value));
        }
    }
    if !mat.vector_params.is_empty() {
        out.push_str("// Vector Parameter Overrides:\n");
        for v in &mat.vector_params {
            out.push_str(&format!(
                "//   [Vector] {:<28} = ({:.4}, {:.4}, {:.4}, {:.4})\n",
                v.name, v.rgba[0], v.rgba[1], v.rgba[2], v.rgba[3]
            ));
        }
    }
    if !mat.texture_params.is_empty() {
        out.push_str("// Texture Parameter Overrides:\n");
        for t in &mat.texture_params {
            out.push_str(&format!(
                "//   [Tex2D]  {:<28} = {}\n",
                t.name, t.texture_path
            ));
        }
    }
    if !mat.uniform_expression_names.is_empty() {
        out.push_str(&format!(
            "// Uniform Expression Set ({}): {}\n",
            mat.uniform_expression_names.len(),
            mat.uniform_expression_names.join(", ")
        ));
    }
    out.push_str(
        "// ============================================================================\n\n",
    );
}

fn emit_hlsl_declarations(out: &mut String, shader: &DxbcShader) {
    assert!(shader.instructions.len() <= INSTRUCTIONS_MAX);
    for inst in &shader.instructions {
        let Some(op0) = inst.operands.first() else {
            continue;
        };
        let idx0 = op0.indices.first().copied().unwrap_or(0);
        let idx1 = op0.indices.get(1).copied().unwrap_or(4096);
        match inst.opcode {
            89 => out.push_str(&format!(
                "cbuffer cb{idx0}_buf : register(b{idx0}) {{ float4 cb{idx0}[{}]; }};\n",
                idx1.max(1)
            )),
            90 => out.push_str(&format!("SamplerState s{idx0} : register(s{idx0});\n")),
            88 | 161 | 162 => {
                out.push_str(&format!("Texture2D<float4> t{idx0} : register(t{idx0});\n"))
            }
            156..=158 => out.push_str(&format!(
                "RWTexture2D<float4> u{idx0} : register(u{idx0});\n"
            )),
            _ => {}
        }
    }
    if !shader.icb_vec4s.is_empty() {
        out.push_str(&format!(
            "static const float4 icb[{}] = {{\n",
            shader.icb_vec4s.len()
        ));
        for v in &shader.icb_vec4s {
            out.push_str(&format!(
                "    float4({}, {}, {}, {}),\n",
                format_imm32_scalar(v[0]),
                format_imm32_scalar(v[1]),
                format_imm32_scalar(v[2]),
                format_imm32_scalar(v[3])
            ));
        }
        out.push_str("};\n");
    }
    out.push('\n');
}

fn format_hlsl_alu_rhs(inst: &Sm5Instruction, s: &[String]) -> Option<String> {
    assert!(!s.is_empty());
    let a = s.first().map(String::as_str).unwrap_or("0.0");
    let b = s.get(1).map(String::as_str).unwrap_or("0.0");
    let c = s.get(2).map(String::as_str).unwrap_or("0.0");
    let expr = match inst.opcode {
        0 | 30 => format!("{a} + {b}"),
        56 | 38 | 81 => format!("{a} * {b}"),
        50 | 35 | 82 => format!("({a} * {b} + {c})"),
        14 | 78 => format!("{a} / {b}"),
        15 => format!("dot(({a}).xy, ({b}).xy)"),
        16 => format!("dot(({a}).xyz, ({b}).xyz)"),
        17 => format!("dot({a}, {b})"),
        51 | 37 | 84 => format!("min({a}, {b})"),
        52 | 36 | 83 => format!("max({a}, {b})"),
        54 => a.to_string(),
        55 => format!("({a} ? {b} : {c})"),
        129 => format!("rcp({a})"),
        68 => format!("rsqrt({a})"),
        75 => format!("sqrt({a})"),
        25 => format!("exp2({a})"),
        47 => format!("log2({a})"),
        26 => format!("frac({a})"),
        64 => format!("round({a})"),
        65 => format!("floor({a})"),
        66 => format!("ceil({a})"),
        67 => format!("trunc({a})"),
        11 | 122 | 123 => format!("ddx({a})"),
        12 | 124 | 125 => format!("ddy({a})"),
        24 | 32 => format!("({a} == {b})"),
        57 | 39 => format!("({a} != {b})"),
        49 | 34 | 79 => format!("({a} < {b})"),
        29 | 33 | 80 => format!("({a} >= {b})"),
        1 => format!("({a} & {b})"),
        60 => format!("({a} | {b})"),
        87 => format!("({a} ^ {b})"),
        59 => format!("(~{a})"),
        40 => format!("(-{a})"),
        41 => format!("({a} << {b})"),
        42 | 85 => format!("({a} >> {b})"),
        27 | 28 | 43 | 86 | 130 | 131 => format!("{}({a})", sm5_opcode_mnemonic(inst.opcode)),
        69 => format!("{b}.Sample({c}, {a})"),
        72 => format!(
            "{b}.SampleLevel({c}, {a}, {})",
            s.get(3).map(String::as_str).unwrap_or("0.0")
        ),
        74 => format!(
            "{b}.SampleBias({c}, {a}, {})",
            s.get(3).map(String::as_str).unwrap_or("0.0")
        ),
        70 | 71 => format!(
            "{b}.SampleCmp({c}, {a}, {})",
            s.get(3).map(String::as_str).unwrap_or("0.0")
        ),
        45 | 163 | 165 => format!("{b}.Load({a})"),
        167 => format!("{c}.Load({a}, {b})"),
        _ => return None,
    };
    Some(if inst.saturate {
        format!("saturate({expr})")
    } else {
        expr
    })
}

fn emit_hlsl_instruction(out: &mut String, inst: &Sm5Instruction, indent: &mut usize) {
    assert!(*indent <= 32);
    if matches!(inst.opcode, 88..=106 | 147..=162) {
        return;
    }
    if matches!(inst.opcode, 18 | 21 | 22 | 23 | 6 | 10) {
        *indent = indent.saturating_sub(1);
    }
    let pad = "    ".repeat(*indent);
    let cond_op = |op: &Sm5Operand| {
        let s = format_sm5_operand(op, false);
        if inst.test_nonzero {
            s
        } else {
            format!("!({s})")
        }
    };

    match inst.opcode {
        31 if !inst.operands.is_empty() => {
            out.push_str(&format!("{pad}if ({}) {{\n", cond_op(&inst.operands[0])));
            *indent = (*indent + 1).min(16);
            return;
        }
        18 => {
            out.push_str(&format!("{pad}}} else {{\n"));
            *indent = (*indent + 1).min(16);
            return;
        }
        21..=23 => {
            out.push_str(&format!("{pad}}}\n"));
            return;
        }
        48 => {
            out.push_str(&format!("{pad}while (true) {{\n"));
            *indent = (*indent + 1).min(16);
            return;
        }
        2 => {
            out.push_str(&format!("{pad}break;\n"));
            return;
        }
        3 if !inst.operands.is_empty() => {
            out.push_str(&format!(
                "{pad}if ({}) break;\n",
                cond_op(&inst.operands[0])
            ));
            return;
        }
        7 => {
            out.push_str(&format!("{pad}continue;\n"));
            return;
        }
        8 if !inst.operands.is_empty() => {
            out.push_str(&format!(
                "{pad}if ({}) continue;\n",
                cond_op(&inst.operands[0])
            ));
            return;
        }
        13 if !inst.operands.is_empty() => {
            out.push_str(&format!(
                "{pad}if ({}) discard;\n",
                cond_op(&inst.operands[0])
            ));
            return;
        }
        62 => {
            out.push_str(&format!("{pad}return;\n"));
            return;
        }
        63 if !inst.operands.is_empty() => {
            out.push_str(&format!(
                "{pad}if ({}) return;\n",
                cond_op(&inst.operands[0])
            ));
            return;
        }
        77 if inst.operands.len() >= 3 => {
            let src = format_sm5_operand(&inst.operands[2], false);
            if inst.operands[0].op_type != 13 {
                out.push_str(&format!(
                    "{pad}{} = sin({src});\n",
                    format_sm5_operand(&inst.operands[0], true)
                ));
            }
            if inst.operands[1].op_type != 13 {
                out.push_str(&format!(
                    "{pad}{} = cos({src});\n",
                    format_sm5_operand(&inst.operands[1], true)
                ));
            }
            return;
        }
        164 | 166 | 168 if inst.operands.len() >= 3 => {
            let uav = format_sm5_operand(&inst.operands[0], true);
            let coord = format_sm5_operand(&inst.operands[1], false);
            let val = format_sm5_operand(&inst.operands[2], false);
            out.push_str(&format!("{pad}{uav}[{coord}] = {val};\n"));
            return;
        }
        _ => {}
    }

    if inst.operands.len() >= 2 {
        let dst = format_sm5_operand(&inst.operands[0], true);
        let srcs: Vec<String> = inst.operands[1..]
            .iter()
            .map(|op| format_sm5_operand(op, false))
            .collect();
        if let Some(rhs) = format_hlsl_alu_rhs(inst, &srcs) {
            out.push_str(&format!("{pad}{dst} = {rhs};\n"));
            return;
        }
        out.push_str(&format!(
            "{pad}{dst} = {}({});\n",
            sm5_opcode_mnemonic(inst.opcode),
            srcs.join(", ")
        ));
    }
}

/// Decompiles a parsed `DxbcShader` into structured `.hlsl` source code, optionally annotated
/// with the owning `MaterialInfo`'s static switches, parameters, and uniform expression names.
pub fn decompile_dxbc_to_hlsl(shader: &DxbcShader, material: Option<&MaterialInfo>) -> String {
    assert!(shader.instructions.len() <= INSTRUCTIONS_MAX);
    let mut out = String::with_capacity(8192);
    out.push_str(&format!(
        "// Decompiled DirectX Shader Model {}.{} ({}_{}_{})\n",
        shader.major_version,
        shader.minor_version,
        program_prefix(shader.program_type),
        shader.major_version,
        shader.minor_version
    ));
    if let Some(mat) = material {
        emit_hlsl_material_header(&mut out, mat);
    }
    emit_hlsl_declarations(&mut out, shader);

    out.push_str("void main(\n");
    for e in &shader.input_signature {
        out.push_str(&format!(
            "    in float4 v{} : {}{},\n",
            e.register, e.semantic_name, e.semantic_index
        ));
    }
    for (idx, e) in shader.output_signature.iter().enumerate() {
        let comma = if idx + 1 == shader.output_signature.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!(
            "    out float4 o{} : {}{}{comma}\n",
            e.register, e.semantic_name, e.semantic_index
        ));
    }
    out.push_str(") {\n");

    let num_temps = shader
        .instructions
        .iter()
        .find(|i| i.opcode == 104 && i.raw_dwords.len() >= 2)
        .map_or(0, |i| i.raw_dwords[1].min(256));
    if num_temps > 0 {
        let regs: Vec<String> = (0..num_temps).map(|r| format!("r{r}")).collect();
        out.push_str(&format!("    float4 {};\n\n", regs.join(", ")));
    }

    let mut indent = 1usize;
    for inst in &shader.instructions {
        emit_hlsl_instruction(&mut out, inst, &mut indent);
    }
    out.push_str("}\n");
    assert!(!out.is_empty());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_minimal_cs50_dxbc() -> Vec<u8> {
        let shex_dwords: [u32; 16] = [
            0x0005_0050, // cs_5_0
            16,          // 16 dwords
            16_779_370,  // dcl_globalFlags
            67_111_068,
            1_171_456,
            0,
            17_476, // dcl_uav_typed u0
            33_554_527,
            131_090, // dcl_input vThreadID.xy
            67_109_019,
            64,
            1,
            1,          // dcl_thread_group 64, 1, 1
            33_554_494, // ret (len=1 -> 16777278)
            16_777_278,
            16_777_278,
        ];
        let mut shex_bytes = vec![0u8; shex_dwords.len() * 4];
        LittleEndian::write_u32_into(&shex_dwords, &mut shex_bytes);

        let total_size = 32 + 4 + 8 + shex_bytes.len();
        let mut dxbc = vec![0u8; total_size];
        LittleEndian::write_u32(&mut dxbc[0..4], DXBC_MAGIC);
        LittleEndian::write_u32(&mut dxbc[20..24], 1);
        LittleEndian::write_u32(&mut dxbc[24..28], total_size as u32);
        LittleEndian::write_u32(&mut dxbc[28..32], 1);
        LittleEndian::write_u32(&mut dxbc[32..36], 36);
        dxbc[36..40].copy_from_slice(b"SHEX");
        LittleEndian::write_u32(&mut dxbc[40..44], shex_bytes.len() as u32);
        dxbc[44..].copy_from_slice(&shex_bytes);
        assert_eq!(dxbc.len(), total_size);
        dxbc
    }

    #[test]
    fn test_parse_disassemble_and_decompile_dxbc() {
        let raw = build_minimal_cs50_dxbc();
        let shader = parse_dxbc(&raw).expect("valid minimal cs_5_0 DXBC");
        assert_eq!(shader.program_type, 5);
        assert_eq!(shader.major_version, 5);
        assert_eq!(shader.minor_version, 0);

        let asm = disassemble_dxbc(&shader);
        assert!(asm.contains("cs_5_0"));
        assert!(asm.contains("dcl_thread_group 64, 1, 1"));

        let hlsl = decompile_dxbc_to_hlsl(&shader, None);
        assert!(hlsl.contains("RWTexture2D<float4> u0 : register(u0);"));
        assert!(hlsl.contains("void main("));
    }
}

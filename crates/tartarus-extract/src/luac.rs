//! UnLua / Xindong Lua 5.3 bytecode (`\x01XDI`) stream-cipher decryptor (`0x20210507`),
//! standard PUC-Lua 5.3 (`\x1bLua\x53`) bytecode transpiler, `unluac` decompiler,
//! `DataCfg/config_*.out` VM table evaluator, and `config_multi_language_*.bin` parser.

use std::collections::BTreeMap;
use std::io::Cursor;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use unluac::decompile::{DecompileDialect, DecompileOptions, DecompileStage, decompile};
use unluac::generate::GenerateMode;
use unluac::parser::StringDecodeMode;

use crate::error::ExtractError;

/// Xindong Lua 5.3 bytecode magic after decryption (`\x01XDI\x11\r\n\x1a\n\x01`).
pub const XDI_LUA_HEADER: &[u8; 10] = b"\x01XDI\x11\r\n\x1a\n\x01";

/// Standard 64-bit PUC-Lua 5.3 chunk header prefix before the 1-byte `main_upvalue_count`
/// (`\x1bLua\x53\x00\x19\x93\r\n\x1a\n\x04\x08\x04\x08\x08` + `0x5678i64` + `370.5f64`).
pub const LUA53_HEADER_PREFIX: &[u8; 33] = b"\x1bLua\x53\x00\x19\x93\r\n\x1a\n\
    \x04\x08\x04\x08\x08\x78\x56\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x28\x77\x40";

/// Bijection mapping Xindong (`\x01XDI`) 6-bit opcodes (`0..=46`) to standard PUC-Lua 5.3 opcodes,
/// extracted from `luaV_execute` (`0x293dcc0`) in `torchlight_infinite.exe`.
pub const XDI_TO_LUA53_OPCODE: [u8; 47] = [
    4,  // 0x00 -> OP_LOADNIL
    15, // 0x01 -> OP_MUL
    10, // 0x02 -> OP_SETTABLE
    33, // 0x03 -> OP_LE
    11, // 0x04 -> OP_NEWTABLE
    2,  // 0x05 -> OP_LOADKX
    1,  // 0x06 -> OP_LOADK
    17, // 0x07 -> OP_POW
    7,  // 0x08 -> OP_GETTABLE
    35, // 0x09 -> OP_TESTSET
    32, // 0x0a -> OP_LT
    31, // 0x0b -> OP_EQ
    0,  // 0x0c -> OP_MOVE
    45, // 0x0d -> OP_VARARG
    5,  // 0x0e -> OP_GETUPVAL
    3,  // 0x0f -> OP_LOADBOOL
    19, // 0x10 -> OP_IDIV
    18, // 0x11 -> OP_DIV
    6,  // 0x12 -> OP_GETTABUP
    8,  // 0x13 -> OP_SETTABUP
    21, // 0x14 -> OP_BOR
    38, // 0x15 -> OP_RETURN
    13, // 0x16 -> OP_ADD
    29, // 0x17 -> OP_CONCAT
    36, // 0x18 -> OP_CALL
    42, // 0x19 -> OP_TFORLOOP
    44, // 0x1a -> OP_CLOSURE
    40, // 0x1b -> OP_FORPREP
    43, // 0x1c -> OP_SETLIST
    37, // 0x1d -> OP_TAILCALL
    39, // 0x1e -> OP_FORLOOP
    20, // 0x1f -> OP_BAND
    9,  // 0x20 -> OP_SETUPVAL
    22, // 0x21 -> OP_BXOR
    30, // 0x22 -> OP_JMP
    16, // 0x23 -> OP_MOD
    27, // 0x24 -> OP_NOT
    12, // 0x25 -> OP_SELF
    26, // 0x26 -> OP_BNOT
    25, // 0x27 -> OP_UNM
    24, // 0x28 -> OP_SHR
    41, // 0x29 -> OP_TFORCALL
    34, // 0x2a -> OP_TEST
    28, // 0x2b -> OP_LEN
    14, // 0x2c -> OP_SUB
    23, // 0x2d -> OP_SHL
    46, // 0x2e -> OP_EXTRAARG
];

const PROTO_ITERATIONS_MAX: usize = 65_536;

const _: () = {
    assert!(XDI_LUA_HEADER.len() == 10);
    assert!(LUA53_HEADER_PREFIX.len() == 33);
    assert!(XDI_TO_LUA53_OPCODE.len() == 47);
    assert!(PROTO_ITERATIONS_MAX >= 1024);
};

/// Decrypts a `.out` Lua bytecode buffer using *Torchlight: Infinite*'s `0x20210507` stream cipher.
/// If the buffer is already decrypted (`\x01XDI`) or normalized (`\x1bLua`), returns a clone.
pub fn decrypt_luac_bytes(binary_data: &[u8]) -> Vec<u8> {
    assert!(binary_data.len() <= 256 * 1024 * 1024);
    if binary_data.starts_with(b"\x01XDI") || binary_data.starts_with(b"\x1bLua") {
        let copy = binary_data.to_vec();
        assert_eq!(copy.len(), binary_data.len());
        return copy;
    }
    let mut out = Vec::with_capacity(binary_data.len());
    let mut v11: u64 = 0;
    for (read_count, &b) in binary_data.iter().enumerate() {
        let rc = read_count as u64;
        let dec = if read_count <= 1 {
            b
        } else if read_count % 3 == 0 {
            b ^ ((rc.wrapping_add(v11 & 1).wrapping_add(v11 >> 28)) as u8)
        } else if read_count % 3 == 1 {
            b ^ ((((v11 >> 16) & 0xFF).wrapping_sub(rc)) as u8)
        } else {
            b ^ ((rc | (v11 >> 21)) as u8)
        };
        out.push(dec);
        v11 = v11.wrapping_add(0x2021_0507);
    }
    assert_eq!(out.len(), binary_data.len());
    out
}

/// Dynamic Lua constant / table value reconstructed from `DataCfg/config_*.out` bytecode.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LuaValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<LuaValue>),
    Map(BTreeMap<String, LuaValue>),
}

impl LuaValue {
    pub fn as_str(&self) -> Option<&str> {
        let res = match self {
            LuaValue::Str(s) => Some(s.as_str()),
            _ => None,
        };
        debug_assert!(res.is_some() == matches!(self, LuaValue::Str(_)));
        debug_assert!(res.is_none_or(|s| s.len() <= 16 * 1024 * 1024));
        res
    }

    pub fn as_i64(&self) -> Option<i64> {
        let res = match self {
            LuaValue::Int(v) => Some(*v),
            LuaValue::Float(v) => Some(*v as i64),
            _ => None,
        };
        debug_assert!(res.is_some() == matches!(self, LuaValue::Int(_) | LuaValue::Float(_)));
        debug_assert!(res.is_none() || !matches!(self, LuaValue::Nil));
        res
    }
}

/// Reconstructed read-only config table from
/// `UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_*.out`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LuaConfigTable {
    pub source_name: String,
    pub default_values: BTreeMap<String, LuaValue>,
    pub rows: BTreeMap<i64, BTreeMap<String, LuaValue>>,
}

/// Reads a raw string slice from either an `\x01XDI` stream (`is_xdi = true`, where short/long
/// lengths store `byte_len + 10` and `0xFF` is followed by a 4-byte `u32`) or a standard 64-bit
/// PUC-Lua 5.3 stream (`is_xdi = false`, where lengths store `byte_len + 1` and `0xFF` is `u64`).
fn read_lua_string_bytes<'a>(
    cur: &mut Cursor<&'a [u8]>,
    is_xdi: bool,
) -> Result<Option<&'a [u8]>, ExtractError> {
    let buf = *cur.get_ref();
    assert!((cur.position() as usize) <= buf.len());
    let lb = cur.read_u8().map_err(|_| {
        ExtractError::LuacFormat(format!(
            "Unexpected EOF reading string length at 0x{:x}",
            cur.position()
        ))
    })?;
    if lb == 0 {
        assert!((cur.position() as usize) <= buf.len());
        return Ok(None);
    }
    let slen = if is_xdi {
        let raw = if lb == 0xFF {
            cur.read_u32::<LittleEndian>()
                .map_err(|e| ExtractError::LuacFormat(e.to_string()))? as usize
        } else {
            lb as usize
        };
        if raw < 10 {
            return Err(ExtractError::LuacFormat(format!(
                "Invalid XDI string length prefix {raw} (< 10) at 0x{:x}",
                cur.position()
            )));
        }
        raw - 10
    } else if lb == 0xFF {
        let raw = cur
            .read_u64::<LittleEndian>()
            .map_err(|e| ExtractError::LuacFormat(e.to_string()))? as usize;
        if raw == 0 {
            return Err(ExtractError::LuacFormat(
                "Invalid Lua 5.3 long string size 0".to_string(),
            ));
        }
        raw - 1
    } else {
        (lb as usize) - 1
    };
    let pos = cur.position() as usize;
    let slice = buf.get(pos..pos.saturating_add(slen)).ok_or_else(|| {
        ExtractError::LuacFormat(format!(
            "String length {} exceeds buffer at 0x{:x} (len={})",
            slen,
            pos,
            buf.len()
        ))
    })?;
    cur.set_position((pos + slen) as u64);
    assert!((cur.position() as usize) <= buf.len());
    Ok(Some(slice))
}

fn read_lua_string(cur: &mut Cursor<&[u8]>, is_xdi: bool) -> Result<String, ExtractError> {
    assert!((cur.position() as usize) <= cur.get_ref().len());
    let bytes = read_lua_string_bytes(cur, is_xdi)?;
    assert!((cur.position() as usize) <= cur.get_ref().len());
    Ok(match bytes {
        Some(slice) => String::from_utf8_lossy(slice).into_owned(),
        None => String::new(),
    })
}

fn write_lua53_string(out: &mut Vec<u8>, bytes: Option<&[u8]>) {
    let before_len = out.len();
    match bytes {
        None => out.push(0),
        Some(slice) => {
            let size = slice.len() + 1;
            if size < 0xFF {
                out.push(size as u8);
            } else {
                out.push(0xFF);
                out.extend_from_slice(&(size as u64).to_le_bytes());
            }
            out.extend_from_slice(slice);
        }
    }
    assert!(out.len() > before_len);
    assert!(out.len() >= before_len + bytes.map_or(1, <[u8]>::len));
}

fn transpile_xdi_instruction(xdi_word: u32) -> Result<u32, ExtractError> {
    let xdi_op = (xdi_word >> 26) as usize;
    assert!(xdi_op < 64);
    let Some(&std_op) = XDI_TO_LUA53_OPCODE.get(xdi_op) else {
        return Err(ExtractError::LuacFormat(format!(
            "Unknown XDI opcode {xdi_op} in instruction 0x{xdi_word:08x}"
        )));
    };
    let std_word = ((xdi_word & 0x03FF_FFFF) << 6) | u32::from(std_op);
    assert_eq!((std_word & 0x3F) as u8, std_op);
    Ok(std_word)
}

fn copy_exact_bytes(
    cur: &mut Cursor<&[u8]>,
    out: &mut Vec<u8>,
    count: usize,
    context: &str,
) -> Result<(), ExtractError> {
    let buf = *cur.get_ref();
    let pos = cur.position() as usize;
    assert!(pos <= buf.len());
    let end = pos
        .checked_add(count)
        .ok_or_else(|| ExtractError::LuacFormat(format!("Byte count overflow in {context}")))?;
    let slice = buf.get(pos..end).ok_or_else(|| {
        ExtractError::LuacFormat(format!(
            "Unexpected EOF in {context} at 0x{pos:x} (need {count} bytes, len={})",
            buf.len()
        ))
    })?;
    out.extend_from_slice(slice);
    cur.set_position(end as u64);
    assert_eq!(cur.position() as usize, end);
    Ok(())
}

enum ProtoStep {
    EnterProto,
    FinishDebugInfo,
}

fn transpile_proto_constants(
    cur: &mut Cursor<&[u8]>,
    out: &mut Vec<u8>,
) -> Result<(), ExtractError> {
    assert!((cur.position() as usize) <= cur.get_ref().len());
    let sizek = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))?;
    out.extend_from_slice(&sizek.to_le_bytes());
    for _ in 0..(sizek as usize) {
        let tag = cur.read_u8().map_err(|_| {
            ExtractError::LuacFormat("Unexpected EOF in Lua constant pool".to_string())
        })?;
        out.push(tag);
        match tag {
            0x00 => {}
            0x01 => copy_exact_bytes(cur, out, 1, "Lua bool constant")?,
            0x03 | 0x13 => copy_exact_bytes(cur, out, 8, "Lua numeric constant")?,
            0x04 | 0x14 => {
                let s = read_lua_string_bytes(cur, true)?;
                write_lua53_string(out, s);
            }
            other => {
                return Err(ExtractError::LuacFormat(format!(
                    "Unsupported Lua constant tag 0x{:02x} at 0x{:x}",
                    other,
                    cur.position() - 1
                )));
            }
        }
    }
    assert!((cur.position() as usize) <= cur.get_ref().len());
    Ok(())
}

fn transpile_proto_body_and_queue_children(
    cur: &mut Cursor<&[u8]>,
    out: &mut Vec<u8>,
    stack: &mut Vec<ProtoStep>,
) -> Result<(), ExtractError> {
    let dec_len = cur.get_ref().len();
    assert!((cur.position() as usize) <= dec_len);
    let src = read_lua_string_bytes(cur, true)?;
    write_lua53_string(out, src);

    // linedefined (4), lastlinedefined (4), numparams (1), is_vararg (1), maxstacksize (1)
    copy_exact_bytes(cur, out, 11, "Lua proto header")?;

    let sizecode = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))?;
    out.extend_from_slice(&sizecode.to_le_bytes());
    for _ in 0..(sizecode as usize) {
        let xdi_word = cur
            .read_u32::<LittleEndian>()
            .map_err(|e| ExtractError::LuacFormat(e.to_string()))?;
        let std_word = transpile_xdi_instruction(xdi_word)?;
        out.extend_from_slice(&std_word.to_le_bytes());
    }

    transpile_proto_constants(cur, out)?;

    let sizeup = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))?;
    out.extend_from_slice(&sizeup.to_le_bytes());
    copy_exact_bytes(cur, out, (sizeup as usize) * 2, "Lua proto upvalues")?;

    let sizep = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))?;
    out.extend_from_slice(&sizep.to_le_bytes());
    let child_count = sizep as usize;
    if child_count > PROTO_ITERATIONS_MAX {
        return Err(ExtractError::LuacFormat(format!(
            "Proto child count {child_count} exceeds limit {PROTO_ITERATIONS_MAX}"
        )));
    }
    stack.push(ProtoStep::FinishDebugInfo);
    for _ in 0..child_count {
        stack.push(ProtoStep::EnterProto);
    }
    assert!((cur.position() as usize) <= dec_len);
    Ok(())
}

fn transpile_proto_debug_info(
    cur: &mut Cursor<&[u8]>,
    out: &mut Vec<u8>,
) -> Result<(), ExtractError> {
    assert!((cur.position() as usize) <= cur.get_ref().len());
    let sizeline = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))?;
    out.extend_from_slice(&sizeline.to_le_bytes());
    copy_exact_bytes(cur, out, (sizeline as usize) * 4, "Lua proto lineinfo")?;

    let sizelocvars = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))?;
    out.extend_from_slice(&sizelocvars.to_le_bytes());
    for _ in 0..(sizelocvars as usize) {
        let varname = read_lua_string_bytes(cur, true)?;
        write_lua53_string(out, varname);
        copy_exact_bytes(cur, out, 8, "Lua locvar pc range")?;
    }

    let sizeupnames = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))?;
    out.extend_from_slice(&sizeupnames.to_le_bytes());
    for _ in 0..(sizeupnames as usize) {
        let upname = read_lua_string_bytes(cur, true)?;
        write_lua53_string(out, upname);
    }
    assert!((cur.position() as usize) <= cur.get_ref().len());
    Ok(())
}

/// Converts an encrypted `.out`, decrypted `\x01XDI`, or standard `\x1bLua\x53` buffer into
/// canonical 64-bit PUC-Lua 5.3 (`\x1bLua\x53`) bytecode.
pub fn xdi_to_lua53_bytes(raw_or_dec: &[u8]) -> Result<Vec<u8>, ExtractError> {
    assert!(raw_or_dec.len() <= 256 * 1024 * 1024);
    if raw_or_dec.len() >= 34 && &raw_or_dec[..33] == LUA53_HEADER_PREFIX {
        let copy = raw_or_dec.to_vec();
        assert_eq!(copy.len(), raw_or_dec.len());
        return Ok(copy);
    }
    let dec = decrypt_luac_bytes(raw_or_dec);
    if dec.len() < 16 || dec[..9] != XDI_LUA_HEADER[..9] {
        return Err(ExtractError::LuacFormat(
            "Buffer does not start with valid \\x01XDI or Lua 5.3 header".to_string(),
        ));
    }

    let mut out = Vec::with_capacity(dec.len() + 64);
    out.extend_from_slice(LUA53_HEADER_PREFIX);
    out.push(dec[9]);

    let mut cur = Cursor::new(dec.as_slice());
    cur.set_position(10);
    let mut stack = Vec::with_capacity(32);
    stack.push(ProtoStep::EnterProto);
    let mut iterations = 0usize;

    while let Some(step) = stack.pop() {
        iterations += 1;
        if iterations > PROTO_ITERATIONS_MAX {
            return Err(ExtractError::LuacFormat(format!(
                "Proto traversal exceeded {PROTO_ITERATIONS_MAX} steps"
            )));
        }
        match step {
            ProtoStep::EnterProto => {
                transpile_proto_body_and_queue_children(&mut cur, &mut out, &mut stack)?;
            }
            ProtoStep::FinishDebugInfo => {
                transpile_proto_debug_info(&mut cur, &mut out)?;
            }
        }
    }

    if (cur.position() as usize) != dec.len() {
        return Err(ExtractError::LuacFormat(format!(
            "Trailing bytes after root proto at 0x{:x} (len={})",
            cur.position(),
            dec.len()
        )));
    }
    assert!(out.len() >= 34 && &out[..33] == LUA53_HEADER_PREFIX);
    Ok(out)
}

/// Decrypts (if needed), transpiles `\x01XDI` to standard PUC-Lua 5.3 bytecode, and decompiles it
/// into formatted Lua 5.3 source code using `unluac` (`unluac-rs`).
pub fn decompile_luac_bytes(raw_or_dec: &[u8]) -> Result<String, ExtractError> {
    assert!(raw_or_dec.len() <= 256 * 1024 * 1024);
    let lua53_bytes = xdi_to_lua53_bytes(raw_or_dec)?;
    assert!(lua53_bytes.len() >= 34);

    let mut options = DecompileOptions {
        dialect: DecompileDialect::Lua53,
        target_stage: DecompileStage::Generate,
        ..DecompileOptions::default()
    };
    options.parse.string_decode_mode = StringDecodeMode::Lossy;
    options.generate.comment = true;
    // Why: Permissive mode ensures unrecovered HIR statements (such as open-tail SETLIST inside
    // split temporary tables) emit inline `-- [unluac error]` comments rather than aborting.
    options.generate.mode = GenerateMode::Permissive;

    let result =
        decompile(&lua53_bytes, options).map_err(|e| ExtractError::LuacDecompile(e.to_string()))?;
    let generated = result.state.generated.ok_or_else(|| {
        ExtractError::LuacDecompile("unluac finished without generated source".to_string())
    })?;
    assert!(!generated.source.is_empty());
    Ok(generated.source)
}

fn open_lua_chunk(dec: &[u8]) -> Result<(Cursor<&[u8]>, bool), ExtractError> {
    assert!(dec.len() <= 256 * 1024 * 1024);
    if dec.len() >= 34 && &dec[..33] == LUA53_HEADER_PREFIX {
        let mut cur = Cursor::new(dec);
        cur.set_position(34);
        assert_eq!(cur.position(), 34);
        Ok((cur, false))
    } else if dec.len() >= 16 && dec[..9] == XDI_LUA_HEADER[..9] {
        let mut cur = Cursor::new(dec);
        cur.set_position(10);
        assert_eq!(cur.position(), 10);
        Ok((cur, true))
    } else {
        Err(ExtractError::LuacFormat(
            "Buffer does not start with valid \\x01XDI or Lua 5.3 header".to_string(),
        ))
    }
}

fn read_proto_code_and_consts(
    cur: &mut Cursor<&[u8]>,
    dec_len: usize,
    is_xdi: bool,
) -> Result<(Vec<u32>, Vec<LuaValue>), ExtractError> {
    assert!((cur.position() as usize) <= dec_len);
    // Skip linedefined (4), lastlinedefined (4), numparams (1), is_vararg (1), maxstacksize (1)
    if (cur.position() as usize) + 11 > dec_len {
        return Err(ExtractError::LuacFormat(
            "Unexpected EOF in Lua proto header".to_string(),
        ));
    }
    cur.set_position(cur.position() + 11);

    let sizecode = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))? as usize;
    if (cur.position() as usize) + sizecode * 4 > dec_len {
        return Err(ExtractError::LuacFormat(
            "Unexpected EOF in Lua code section".to_string(),
        ));
    }
    let mut code = Vec::with_capacity(sizecode);
    for _ in 0..sizecode {
        code.push(
            cur.read_u32::<LittleEndian>()
                .map_err(|e| ExtractError::LuacFormat(e.to_string()))?,
        );
    }

    let sizek = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))? as usize;
    let mut consts = Vec::with_capacity(sizek);
    for _ in 0..sizek {
        let tag = cur.read_u8().map_err(|_| {
            ExtractError::LuacFormat("Unexpected EOF in Lua constant pool".to_string())
        })?;
        match tag {
            0x00 => consts.push(LuaValue::Nil),
            0x01 => {
                let b = cur.read_u8().map_err(|_| {
                    ExtractError::LuacFormat("Unexpected EOF reading bool constant".to_string())
                })?;
                consts.push(LuaValue::Bool(b != 0));
            }
            0x03 => consts.push(LuaValue::Float(
                cur.read_f64::<LittleEndian>()
                    .map_err(|e| ExtractError::LuacFormat(e.to_string()))?,
            )),
            0x13 => consts.push(LuaValue::Int(
                cur.read_i64::<LittleEndian>()
                    .map_err(|e| ExtractError::LuacFormat(e.to_string()))?,
            )),
            0x04 | 0x14 => consts.push(LuaValue::Str(read_lua_string(cur, is_xdi)?)),
            other => {
                return Err(ExtractError::LuacFormat(format!(
                    "Unsupported Lua constant tag 0x{:02x} at 0x{:x}",
                    other,
                    cur.position() - 1
                )));
            }
        }
    }
    assert_eq!(code.len(), sizecode);
    assert_eq!(consts.len(), sizek);
    Ok((code, consts))
}

fn decode_instruction_fields(w: u32, is_xdi: bool) -> (u8, usize, usize, usize, usize) {
    if is_xdi {
        let xdi_op = ((w >> 26) & 0x3F) as usize;
        assert!(xdi_op < 64);
        let std_op = XDI_TO_LUA53_OPCODE.get(xdi_op).copied().unwrap_or(0xFF);
        let a = (w & 0xFF) as usize;
        let c = ((w >> 8) & 0x1FF) as usize;
        let b = ((w >> 17) & 0x1FF) as usize;
        let bx = ((w >> 8) & 0x3FFFF) as usize;
        assert!(a < 256);
        (std_op, a, b, c, bx)
    } else {
        let std_op = (w & 0x3F) as u8;
        let a = ((w >> 6) & 0xFF) as usize;
        let c = ((w >> 14) & 0x1FF) as usize;
        let b = ((w >> 23) & 0x1FF) as usize;
        let bx = ((w >> 14) & 0x3FFFF) as usize;
        assert!(a < 256);
        assert!(std_op < 64);
        (std_op, a, b, c, bx)
    }
}

fn eval_table_code(
    code: &[u32],
    consts: &[LuaValue],
    is_xdi: bool,
) -> (
    BTreeMap<String, LuaValue>,
    BTreeMap<i64, BTreeMap<String, LuaValue>>,
) {
    let mut regs: Vec<LuaValue> = vec![LuaValue::Nil; 256];
    assert_eq!(regs.len(), 256);
    let rk = |val: usize, regs: &[LuaValue], consts: &[LuaValue]| -> LuaValue {
        if (val & 0x100) != 0 {
            consts.get(val & 0xFF).cloned().unwrap_or(LuaValue::Nil)
        } else {
            regs.get(val).cloned().unwrap_or(LuaValue::Nil)
        }
    };

    let mut raw_rows: BTreeMap<i64, BTreeMap<String, LuaValue>> = BTreeMap::new();
    for &w in code {
        let (std_op, a, b, c, bx) = decode_instruction_fields(w, is_xdi);
        match std_op {
            11 => regs[a] = LuaValue::Map(BTreeMap::new()),
            1 => regs[a] = consts.get(bx).cloned().unwrap_or(LuaValue::Nil),
            10 => {
                let key = rk(b, &regs, consts);
                let val = rk(c, &regs, consts);
                if a == 0 {
                    if let (Some(id), LuaValue::Map(row_map)) = (key.as_i64(), val) {
                        raw_rows.insert(id, row_map);
                    }
                } else if let LuaValue::Map(ref mut m) = regs[a] {
                    let key_str = match key {
                        LuaValue::Str(s) => s,
                        LuaValue::Int(i) => i.to_string(),
                        _ => continue,
                    };
                    m.insert(key_str, val);
                }
            }
            43 => {
                let mut list = Vec::with_capacity(b);
                for i in 1..=b {
                    if a + i < regs.len() {
                        list.push(regs[a + i].clone());
                    }
                }
                regs[a] = LuaValue::List(list);
            }
            _ => {}
        }
    }

    let default_values = match &regs[1] {
        LuaValue::Map(m) => m.clone(),
        _ => BTreeMap::new(),
    };
    assert_eq!(regs.len(), 256);
    (default_values, raw_rows)
}

/// Parses an encrypted `.out`, decrypted `\x01XDI`, or standard `\x1bLua\x53`
/// `DataCfg/config_*.out` file by decoding its constant pool and executing the top-level
/// table-construction instructions (`NEWTABLE`, `LOADK`, `SETTABLE`, `SETLIST`), merging the
/// metatable `default_values` (`__index`) into every row.
pub fn parse_xdi_config_table(raw_or_dec: &[u8]) -> Result<LuaConfigTable, ExtractError> {
    let dec = decrypt_luac_bytes(raw_or_dec);
    assert_eq!(dec.len(), raw_or_dec.len());
    let (mut cur, is_xdi) = open_lua_chunk(&dec)?;
    let source_name = read_lua_string(&mut cur, is_xdi)?;
    let (code, consts) = read_proto_code_and_consts(&mut cur, dec.len(), is_xdi)?;
    let (default_values, raw_rows) = eval_table_code(&code, &consts, is_xdi);
    let raw_rows_count = raw_rows.len();

    let mut rows = BTreeMap::new();
    for (id, row_map) in raw_rows {
        let mut merged = default_values.clone();
        for (k, v) in row_map {
            merged.insert(k, v);
        }
        rows.insert(id, merged);
    }
    assert_eq!(rows.len(), raw_rows_count);

    Ok(LuaConfigTable {
        source_name,
        default_values,
        rows,
    })
}

/// Walks all function protos inside an encrypted `.out`, decrypted `\x01XDI`, or standard
/// `\x1bLua\x53` bytecode buffer and returns `(source_name, all_string_constants)`.
pub fn extract_xdi_strings(raw_or_dec: &[u8]) -> Result<(String, Vec<String>), ExtractError> {
    let dec = decrypt_luac_bytes(raw_or_dec);
    assert_eq!(dec.len(), raw_or_dec.len());
    let (mut cur, is_xdi) = open_lua_chunk(&dec)?;
    let mut strings = Vec::new();
    let source_name = parse_proto_strings(&mut cur, &mut strings, is_xdi)?;
    assert!((cur.position() as usize) <= dec.len());
    Ok((source_name, strings))
}

fn skip_proto_debug_info(cur: &mut Cursor<&[u8]>, dec_len: usize, is_xdi: bool) {
    assert!((cur.position() as usize) <= dec_len);
    if let Ok(sizeline) = cur.read_u32::<LittleEndian>() {
        cur.set_position(((cur.position() as usize) + (sizeline as usize) * 4).min(dec_len) as u64);
        if let Ok(sizelocvars) = cur.read_u32::<LittleEndian>() {
            for _ in 0..sizelocvars {
                let _ = read_lua_string(cur, is_xdi);
                cur.set_position(((cur.position() as usize) + 8).min(dec_len) as u64);
            }
        }
        if let Ok(sizeupnames) = cur.read_u32::<LittleEndian>() {
            for _ in 0..sizeupnames {
                let _ = read_lua_string(cur, is_xdi);
            }
        }
    }
    assert!((cur.position() as usize) <= dec_len);
}

fn read_proto_body_and_queue_children(
    cur: &mut Cursor<&[u8]>,
    out_strings: &mut Vec<String>,
    stack: &mut Vec<ProtoStep>,
    is_xdi: bool,
) -> Result<String, ExtractError> {
    let dec_len = cur.get_ref().len();
    assert!((cur.position() as usize) <= dec_len);
    let src = read_lua_string(cur, is_xdi)?;
    if (cur.position() as usize) + 11 > dec_len {
        return Err(ExtractError::LuacFormat("EOF in proto header".to_string()));
    }
    // linedefined (4), lastlinedefined (4), numparams (1), is_vararg (1), maxstacksize (1)
    cur.set_position(cur.position() + 11);

    let sizecode = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))? as usize;
    cur.set_position(((cur.position() as usize) + sizecode * 4).min(dec_len) as u64);

    let sizek = cur
        .read_u32::<LittleEndian>()
        .map_err(|e| ExtractError::LuacFormat(e.to_string()))? as usize;
    for _ in 0..sizek {
        let Ok(tag) = cur.read_u8() else {
            break;
        };
        match tag {
            0x00 => {}
            0x01 => cur.set_position(cur.position() + 1),
            0x03 | 0x13 => cur.set_position(cur.position() + 8),
            0x04 | 0x14 => {
                if let Ok(s) = read_lua_string(cur, is_xdi)
                    && !s.is_empty()
                {
                    out_strings.push(s);
                }
            }
            _ => break,
        }
    }

    if let Ok(sizeup) = cur.read_u32::<LittleEndian>() {
        let up_bytes = (sizeup as usize) * 2;
        if (cur.position() as usize) + up_bytes <= dec_len {
            cur.set_position(cur.position() + up_bytes as u64);
            stack.push(ProtoStep::FinishDebugInfo);
            if let Ok(sizep) = cur.read_u32::<LittleEndian>() {
                let child_count = (sizep as usize).min(PROTO_ITERATIONS_MAX);
                for _ in 0..child_count {
                    stack.push(ProtoStep::EnterProto);
                }
            }
        }
    }
    assert!((cur.position() as usize) <= dec_len);
    Ok(src)
}

fn parse_proto_strings(
    cur: &mut Cursor<&[u8]>,
    out_strings: &mut Vec<String>,
    is_xdi: bool,
) -> Result<String, ExtractError> {
    let dec_len = cur.get_ref().len();
    assert!((cur.position() as usize) <= dec_len);
    let mut stack = Vec::with_capacity(32);
    stack.push(ProtoStep::EnterProto);
    let mut root_src: Option<String> = None;
    let mut iterations = 0usize;

    while let Some(step) = stack.pop() {
        iterations += 1;
        assert!(iterations <= PROTO_ITERATIONS_MAX);
        match step {
            ProtoStep::EnterProto => {
                match read_proto_body_and_queue_children(cur, out_strings, &mut stack, is_xdi) {
                    Ok(src) => {
                        if root_src.is_none() {
                            root_src = Some(src);
                        }
                    }
                    Err(e) => {
                        if root_src.is_none() {
                            return Err(e);
                        }
                    }
                }
            }
            ProtoStep::FinishDebugInfo => {
                skip_proto_debug_info(cur, dec_len, is_xdi);
            }
        }
    }

    assert!(iterations <= PROTO_ITERATIONS_MAX);
    root_src.ok_or_else(|| ExtractError::LuacFormat("Empty proto stream".to_string()))
}

/// Parses `UE_game/Content/Script/Data/language/config_multi_language_*.bin`.
///
/// Each record in the language binary has the layout:
/// `<u32 klen><key_bytes><u32 klen><key_bytes><u32 0><u32 vlen><utf8_value_bytes>`.
/// If `prefix_filter` is `Some(prefix)`, only keys starting with `prefix` are collected.
pub fn parse_multi_language_bin(
    data: &[u8],
    prefix_filter: Option<&str>,
) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let prefix_bytes = prefix_filter.map(|s| s.as_bytes()).unwrap_or(&[]);
    let n = data.len();
    assert!(prefix_bytes.len() <= 4096);
    if n < 20 {
        assert!(map.is_empty());
        return map;
    }

    let iterations_max = n + 1;
    let mut iterations = 0usize;
    let mut i = 0usize;
    while i + 16 <= n {
        iterations += 1;
        assert!(iterations <= iterations_max);
        let klen = LittleEndian::read_u32(&data[i..i + 4]) as usize;
        if (4..=256).contains(&klen) && i + 8 + 2 * klen + 8 <= n {
            let k1_start = i + 4;
            let k1_end = k1_start + klen;
            if prefix_bytes.is_empty() || data[k1_start..k1_end].starts_with(prefix_bytes) {
                let klen2 = LittleEndian::read_u32(&data[k1_end..k1_end + 4]) as usize;
                if klen2 == klen {
                    let k2_start = k1_end + 4;
                    let k2_end = k2_start + klen;
                    if data[k1_start..k1_end] == data[k2_start..k2_end] {
                        let flag = LittleEndian::read_u32(&data[k2_end..k2_end + 4]);
                        let vlen = LittleEndian::read_u32(&data[k2_end + 4..k2_end + 8]) as usize;
                        let val_start = k2_end + 8;
                        let val_end = val_start.saturating_add(vlen);
                        if flag == 0
                            && vlen <= 32768
                            && val_end <= n
                            && let (Ok(k), Ok(v)) = (
                                std::str::from_utf8(&data[k1_start..k1_end]),
                                std::str::from_utf8(&data[val_start..val_end]),
                            )
                        {
                            map.insert(k.to_string(), v.to_string());
                            i = val_end;
                            continue;
                        }
                    }
                }
            }
        }
        i += 1;
    }

    assert!(i <= n);
    map
}

/// Strips `<RichText...>` / `</>` / `<Img...>` markup tags and normalizes escaped newlines.
pub fn strip_rich_text(input: &str) -> String {
    use std::sync::LazyLock;
    static TAG_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"<[^>]*>").unwrap());
    assert!(input.len() <= 16 * 1024 * 1024);
    let out = TAG_RE
        .replace_all(input, "")
        .replace("\\n", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(out.len() <= input.len());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_xdi_string(out: &mut Vec<u8>, s: Option<&[u8]>) {
        let before = out.len();
        match s {
            None => out.push(0),
            Some(bytes) => {
                let raw = bytes.len() + 10;
                if raw < 0xFF {
                    out.push(raw as u8);
                } else {
                    out.push(0xFF);
                    out.extend_from_slice(&(raw as u32).to_le_bytes());
                }
                out.extend_from_slice(bytes);
            }
        }
        assert!(out.len() > before);
        assert!(out.len() >= before + s.map_or(1, <[u8]>::len));
    }

    fn encode_xdi_abc(xdi_op: u8, a: u8, b: u16, c: u16) -> u32 {
        assert!(xdi_op < 47);
        assert!(b < 512 && c < 512);
        (u32::from(xdi_op) << 26) | (u32::from(b) << 17) | (u32::from(c) << 8) | u32::from(a)
    }

    fn encode_xdi_abx(xdi_op: u8, a: u8, bx: u32) -> u32 {
        assert!(xdi_op < 47);
        assert!(bx < (1 << 18));
        (u32::from(xdi_op) << 26) | (bx << 8) | u32::from(a)
    }

    fn encrypt_xdi_for_test(plain_xdi: &[u8]) -> Vec<u8> {
        assert!(plain_xdi.starts_with(XDI_LUA_HEADER));
        let mut enc = decrypt_luac_bytes(plain_xdi);
        // Flip the first byte so decrypt_luac_bytes runs the stream cipher (bytes 0..=1 are
        // unxored in the cipher, so restore byte 0 on the encrypted buffer after xoring).
        enc[0] ^= 0xFF;
        let mut cipher = decrypt_luac_bytes(&enc);
        cipher[0] = plain_xdi[0];
        // Make byte 2..4 not match "XDI" so decrypt_luac_bytes treats it as encrypted `.out`.
        assert_ne!(&cipher[..4], b"\x01XDI");
        cipher
    }

    #[test]
    fn test_opcode_table_is_bijection() {
        let mut seen = [false; 47];
        for &std_op in &XDI_TO_LUA53_OPCODE {
            assert!((std_op as usize) < 47);
            assert!(!seen[std_op as usize]);
            seen[std_op as usize] = true;
        }
        assert!(seen.iter().all(|&b| b));
    }

    #[test]
    fn test_xdi_to_lua53_and_decompile_chunk() {
        let long_str = "A".repeat(260);
        let mut xdi = Vec::new();
        xdi.extend_from_slice(XDI_LUA_HEADER);
        push_xdi_string(&mut xdi, Some(b"@TestChunk.lua"));
        xdi.extend_from_slice(&0u32.to_le_bytes()); // linedefined
        xdi.extend_from_slice(&0u32.to_le_bytes()); // lastlinedefined
        xdi.push(0); // numparams
        xdi.push(1); // is_vararg
        xdi.push(2); // maxstacksize

        // Code: LOADK R0, K0 (xdi_op=6); RETURN R0, 2 (xdi_op=21, B=2, C=0)
        xdi.extend_from_slice(&2u32.to_le_bytes());
        xdi.extend_from_slice(&encode_xdi_abx(6, 0, 0).to_le_bytes());
        xdi.extend_from_slice(&encode_xdi_abc(21, 0, 2, 0).to_le_bytes());

        // Constants: K0 = long_str (260 bytes -> exercises 0xFF + u32 long string path)
        xdi.extend_from_slice(&1u32.to_le_bytes());
        xdi.push(0x14);
        push_xdi_string(&mut xdi, Some(long_str.as_bytes()));

        // Upvalues: 1 (_ENV: instack=1, idx=0)
        xdi.extend_from_slice(&1u32.to_le_bytes());
        xdi.extend_from_slice(&[1u8, 0u8]);

        // Protos: 0
        xdi.extend_from_slice(&0u32.to_le_bytes());

        // Debug info: 2 lineinfo entries, 0 locvars, 1 upvalue name ("_ENV")
        xdi.extend_from_slice(&2u32.to_le_bytes());
        xdi.extend_from_slice(&1u32.to_le_bytes());
        xdi.extend_from_slice(&1u32.to_le_bytes());
        xdi.extend_from_slice(&0u32.to_le_bytes());
        xdi.extend_from_slice(&1u32.to_le_bytes());
        push_xdi_string(&mut xdi, Some(b"_ENV"));

        let encrypted = encrypt_xdi_for_test(&xdi);
        let lua53 = xdi_to_lua53_bytes(&encrypted).expect("xdi_to_lua53_bytes should succeed");
        assert!(lua53.starts_with(LUA53_HEADER_PREFIX));

        let src = decompile_luac_bytes(&encrypted).expect("decompile_luac_bytes should succeed");
        assert!(src.contains("TestChunk.lua"), "unexpected src: {src}");
        assert!(src.contains(&format!("return \"{long_str}\"")));

        let (src_name, strs) = extract_xdi_strings(&lua53).expect("extract_xdi_strings on lua53");
        assert_eq!(src_name, "@TestChunk.lua");
        assert_eq!(strs, vec![long_str]);
    }

    #[test]
    fn test_parse_xdi_config_table_roundtrip() {
        let mut xdi = Vec::new();
        xdi.extend_from_slice(XDI_LUA_HEADER);
        push_xdi_string(&mut xdi, Some(b"@DataCfg/config_test.lua"));
        xdi.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 8]);

        // Instructions building:
        // R0 = {} (rows table); R1 = { Priority = 10 } (default_values);
        // R2 = { Name = "RelicA" }; R0[101] = R2; return R0
        let code = [
            encode_xdi_abc(4, 0, 0, 0),         // NEWTABLE R0
            encode_xdi_abc(4, 1, 0, 0),         // NEWTABLE R1
            encode_xdi_abc(2, 1, 0x100, 0x101), // SETTABLE R1["Priority"] = 10
            encode_xdi_abc(4, 2, 0, 0),         // NEWTABLE R2
            encode_xdi_abc(2, 2, 0x102, 0x103), // SETTABLE R2["Name"] = "RelicA"
            encode_xdi_abc(2, 0, 0x104, 2),     // SETTABLE R0[101] = R2
            encode_xdi_abc(21, 0, 2, 0),        // RETURN R0, 2
        ];
        xdi.extend_from_slice(&(code.len() as u32).to_le_bytes());
        for &w in &code {
            xdi.extend_from_slice(&w.to_le_bytes());
        }

        // Constants: K0="Priority", K1=10, K2="Name", K3="RelicA", K4=101
        xdi.extend_from_slice(&5u32.to_le_bytes());
        xdi.push(0x04);
        push_xdi_string(&mut xdi, Some(b"Priority"));
        xdi.push(0x13);
        xdi.extend_from_slice(&10i64.to_le_bytes());
        xdi.push(0x04);
        push_xdi_string(&mut xdi, Some(b"Name"));
        xdi.push(0x04);
        push_xdi_string(&mut xdi, Some(b"RelicA"));
        xdi.push(0x13);
        xdi.extend_from_slice(&101i64.to_le_bytes());

        // Upvalues (1), Protos (0), Debug (0, 0, 0)
        xdi.extend_from_slice(&1u32.to_le_bytes());
        xdi.extend_from_slice(&[1u8, 0u8]);
        xdi.extend_from_slice(&0u32.to_le_bytes());
        xdi.extend_from_slice(&0u32.to_le_bytes());
        xdi.extend_from_slice(&0u32.to_le_bytes());
        xdi.extend_from_slice(&0u32.to_le_bytes());

        let tbl_xdi = parse_xdi_config_table(&xdi).expect("parse XDI config table");
        let lua53 = xdi_to_lua53_bytes(&xdi).expect("transpile config table");
        let tbl_lua53 = parse_xdi_config_table(&lua53).expect("parse Lua53 config table");
        assert_eq!(tbl_xdi.rows, tbl_lua53.rows);
        assert_eq!(tbl_xdi.rows[&101]["Priority"], LuaValue::Int(10));
        assert_eq!(
            tbl_xdi.rows[&101]["Name"],
            LuaValue::Str("RelicA".to_string())
        );

        let lua_src = decompile_luac_bytes(&lua53).expect("decompile config table");
        assert!(lua_src.contains("Priority = 10"));
        assert!(lua_src.contains("Name = \"RelicA\""));
    }
}

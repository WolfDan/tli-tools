//! Bounded `\x01XDI` bytecode decoder and constant-table VM for `DataCfg` schema extraction.

use crate::error::XtaskError;
use crate::u32_to_usize;
use zerocopy::FromBytes;
use zerocopy::byteorder::little_endian::{I64, U32};

/// Maximum number of VM instructions executed when evaluating a `DataCfg` root chunk.
pub const GEN_INSTRUCTIONS_MAX: u32 = 2_097_152;

/// Maximum number of constants allowed in a `DataCfg` root prototype.
pub const GEN_CONSTANTS_MAX: u32 = 524_288;

/// Maximum number of table allocations allowed in a `DataCfg` root chunk.
pub const GEN_TABLES_MAX: u32 = 524_288;

const XDI_HEADER_MAGIC: &[u8; 9] = b"\x01XDI\x11\r\n\x1a\n";

const XDI_TO_LUA53_OPCODE: [u8; 47] = [
    4, 15, 10, 33, 11, 2, 1, 17, 7, 35, 32, 31, 0, 45, 5, 3, 19, 18, 6, 8, 21, 38, 13, 29, 36, 42,
    44, 40, 43, 37, 39, 20, 9, 22, 30, 16, 27, 12, 26, 25, 24, 41, 34, 28, 14, 23, 46,
];

/// Abstract value tracked during `DataCfg` schema extraction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Val {
    Nil,
    Bool,
    Int(i64),
    Float,
    Str(Box<str>),
    Table(u32),
}

/// Abstract table populated by `NEWTABLE`, `SETLIST`, and `SETTABLE` instructions.
#[derive(Clone, Debug, Default)]
pub struct TableObj {
    pub list: Vec<Val>,
    pub map: Vec<(Val, Val)>,
}

/// Decoded root prototype instructions and constants for a `DataCfg` `.out` chunk.
#[derive(Debug)]
pub struct RootProto {
    pub code: Box<[u32]>,
    pub consts: Box<[Val]>,
}

const fn low_u8(x: u64) -> u8 {
    (x & 0xFF) as u8
}

/// Decrypts an encrypted `.out` buffer in place if it does not already start with `\x01XDI` or `\x1bLua`.
pub fn decrypt_in_place(buf: &mut [u8]) {
    if buf.starts_with(b"\x01XDI") || buf.starts_with(b"\x1bLua") {
        return;
    }
    for (i, byte) in buf.iter_mut().enumerate() {
        if i <= 1 {
            continue;
        }
        let idx = i as u64;
        let v = idx.wrapping_mul(0x2021_0507);
        let mask = match i % 3 {
            0 => low_u8(idx.wrapping_add(v & 1).wrapping_add(v >> 28)),
            1 => low_u8(((v >> 16) & 0xFF).wrapping_sub(idx)),
            _ => low_u8(idx | (v >> 21)),
        };
        *byte ^= mask;
    }
}

/// Fallible fat-slice byte cursor for reading `\x01XDI` root prototype fields.
pub struct Cursor<'a> {
    buf: &'a [u8],
    stem: &'a str,
}

impl<'a> Cursor<'a> {
    #[must_use]
    pub const fn new(buf: &'a [u8], stem: &'a str) -> Self {
        Self { buf, stem }
    }

    fn err(&self, context: &'static str) -> XtaskError {
        XtaskError::MalformedBytecode {
            stem: self.stem.into(),
            context,
        }
    }

    pub fn take(&mut self, len: u32) -> Result<&'a [u8], XtaskError> {
        let (slice, rest) = self
            .buf
            .split_at_checked(u32_to_usize(len))
            .ok_or_else(|| self.err("cursor eof"))?;
        self.buf = rest;
        Ok(slice)
    }

    pub fn read_u8(&mut self) -> Result<u8, XtaskError> {
        let (&b, rest) = self
            .buf
            .split_first()
            .ok_or_else(|| self.err("cursor eof"))?;
        self.buf = rest;
        Ok(b)
    }

    pub fn read_u32(&mut self) -> Result<u32, XtaskError> {
        let (v, rest) = U32::ref_from_prefix(self.buf).map_err(|_| self.err("cursor eof"))?;
        self.buf = rest;
        Ok(v.get())
    }

    pub fn read_i64(&mut self) -> Result<i64, XtaskError> {
        let (v, rest) = I64::ref_from_prefix(self.buf).map_err(|_| self.err("cursor eof"))?;
        self.buf = rest;
        Ok(v.get())
    }

    pub fn read_xdi_string(&mut self) -> Result<Option<Box<str>>, XtaskError> {
        let tag = self.read_u8()?;
        if tag == 0 {
            return Ok(None);
        }
        let byte_len = if tag < 0xFF {
            u32::from(tag).checked_sub(10)
        } else {
            self.read_u32()?.checked_sub(10)
        }
        .ok_or_else(|| self.err("xdi string length < 10"))?;
        let bytes = self.take(byte_len)?;
        Ok(Some(core::str::from_utf8(bytes)?.into()))
    }
}

/// Parses the 10-byte `\x01XDI` header and the root prototype's instruction and constant arrays.
pub fn parse_root_proto(cur: &mut Cursor<'_>) -> Result<RootProto, XtaskError> {
    let hdr = cur.take(10)?;
    if &hdr[..9] != XDI_HEADER_MAGIC {
        return Err(cur.err("invalid XDI header magic"));
    }
    let _src = cur.read_xdi_string()?;
    let _meta = cur.take(11)?;
    let size_code = cur.read_u32()?;
    if size_code > GEN_INSTRUCTIONS_MAX {
        return Err(cur.err("instruction count exceeds GEN_INSTRUCTIONS_MAX"));
    }
    let mut code = Vec::with_capacity(u32_to_usize(size_code));
    for _ in 0..size_code {
        code.push(cur.read_u32()?);
    }
    let size_k = cur.read_u32()?;
    if size_k > GEN_CONSTANTS_MAX {
        return Err(cur.err("constant count exceeds GEN_CONSTANTS_MAX"));
    }
    let mut consts = Vec::with_capacity(u32_to_usize(size_k));
    for _ in 0..size_k {
        let tag = cur.read_u8()?;
        let v = match tag {
            0x00 => Val::Nil,
            0x01 => {
                let _b = cur.read_u8()?;
                Val::Bool
            }
            0x03 => {
                let _f = cur.take(8)?;
                Val::Float
            }
            0x13 => Val::Int(cur.read_i64()?),
            0x04 | 0x14 => {
                let s = cur
                    .read_xdi_string()?
                    .ok_or_else(|| cur.err("null string constant"))?;
                Val::Str(s)
            }
            _ => return Err(cur.err("invalid constant tag")),
        };
        consts.push(v);
    }
    Ok(RootProto {
        code: code.into_boxed_slice(),
        consts: consts.into_boxed_slice(),
    })
}

fn read_rk(stem: &str, rk: u16, consts: &[Val], regs: &[Val; 256]) -> Result<Val, XtaskError> {
    if (rk & 0x100) != 0 {
        consts
            .get(usize::from(rk & 0xFF))
            .cloned()
            .ok_or_else(|| XtaskError::MalformedBytecode {
                stem: stem.into(),
                context: "rk constant index out of bounds",
            })
    } else {
        regs.get(usize::from(rk))
            .cloned()
            .ok_or_else(|| XtaskError::MalformedBytecode {
                stem: stem.into(),
                context: "rk register index out of bounds",
            })
    }
}

fn exec_setlist(
    stem: &str,
    a: u8,
    b: u16,
    c: u16,
    regs: &[Val; 256],
    tables: &mut [TableObj],
    pc: &mut u32,
) -> Result<(), XtaskError> {
    let count = if b == 0 { 50_u16 } else { b };
    let a_idx = usize::from(a);
    let Val::Table(tid) = regs[a_idx] else {
        return Err(XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "setlist on non-table",
        });
    };
    let end_reg = a_idx + usize::from(count);
    let src_slice =
        regs.get((a_idx + 1)..=end_reg)
            .ok_or_else(|| XtaskError::MalformedBytecode {
                stem: stem.into(),
                context: "setlist register span out of bounds",
            })?;
    let tbl = tables
        .get_mut(u32_to_usize(tid))
        .ok_or_else(|| XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "setlist table id out of bounds",
        })?;
    tbl.list.extend_from_slice(src_slice);
    if c == 0 {
        *pc += 1;
    }
    Ok(())
}

fn exec_settable(
    stem: &str,
    a: u8,
    b: u16,
    c: u16,
    consts: &[Val],
    regs: &[Val; 256],
    tables: &mut [TableObj],
) -> Result<(), XtaskError> {
    let k = read_rk(stem, b, consts, regs)?;
    let v = read_rk(stem, c, consts, regs)?;
    let Val::Table(tid) = regs[usize::from(a)] else {
        return Err(XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "settable on non-table",
        });
    };
    let tbl = tables
        .get_mut(u32_to_usize(tid))
        .ok_or_else(|| XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "settable table id out of bounds",
        })?;
    tbl.map.push((k, v));
    Ok(())
}

fn exec_newtable(
    stem: &str,
    a: u8,
    regs: &mut [Val; 256],
    tables: &mut Vec<TableObj>,
) -> Result<(), XtaskError> {
    if tables.len() >= u32_to_usize(GEN_TABLES_MAX) {
        return Err(XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "table count exceeds GEN_TABLES_MAX",
        });
    }
    let tid = u32::try_from(tables.len()).map_err(|_| XtaskError::MalformedBytecode {
        stem: stem.into(),
        context: "table count overflow",
    })?;
    tables.push(TableObj::default());
    regs[usize::from(a)] = Val::Table(tid);
    Ok(())
}

fn exec_loadkx(
    stem: &str,
    a: u8,
    proto: &RootProto,
    regs: &mut [Val; 256],
    pc: &mut u32,
) -> Result<(), XtaskError> {
    let next_w =
        *proto
            .code
            .get(u32_to_usize(*pc + 1))
            .ok_or_else(|| XtaskError::MalformedBytecode {
                stem: stem.into(),
                context: "loadkx missing extraarg",
            })?;
    let ax = u32_to_usize(next_w & 0x03FF_FFFF);
    regs[usize::from(a)] =
        proto
            .consts
            .get(ax)
            .cloned()
            .ok_or_else(|| XtaskError::MalformedBytecode {
                stem: stem.into(),
                context: "loadkx constant out of bounds",
            })?;
    *pc += 1;
    Ok(())
}

fn step_gen_instruction(
    stem: &str,
    proto: &RootProto,
    regs: &mut [Val; 256],
    tables: &mut Vec<TableObj>,
    pc: &mut u32,
) -> Result<bool, XtaskError> {
    let w = *proto
        .code
        .get(u32_to_usize(*pc))
        .ok_or_else(|| XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "pc out of bounds",
        })?;
    let std_op = *XDI_TO_LUA53_OPCODE
        .get(u32_to_usize(w >> 26))
        .ok_or_else(|| XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "invalid XDI opcode",
        })?;
    let a = low_u8(u64::from(w));
    let c = ((w >> 8) & 0x1FF) as u16;
    let b = ((w >> 17) & 0x1FF) as u16;
    let bx = (w >> 8) & 0x3FFFF;
    match std_op {
        11 => exec_newtable(stem, a, regs, tables)?,
        1 => {
            regs[usize::from(a)] =
                proto.consts.get(u32_to_usize(bx)).cloned().ok_or_else(|| {
                    XtaskError::MalformedBytecode {
                        stem: stem.into(),
                        context: "loadk constant out of bounds",
                    }
                })?;
        }
        2 => exec_loadkx(stem, a, proto, regs, pc)?,
        3 => {
            regs[usize::from(a)] = Val::Bool;
            if c != 0 {
                *pc += 1;
            }
        }
        0 => {
            regs[usize::from(a)] =
                regs.get(usize::from(b))
                    .cloned()
                    .ok_or_else(|| XtaskError::MalformedBytecode {
                        stem: stem.into(),
                        context: "move source register out of bounds",
                    })?;
        }
        43 => exec_setlist(stem, a, b, c, regs, tables, pc)?,
        10 => exec_settable(stem, a, b, c, &proto.consts, regs, tables)?,
        36 | 38 => return Ok(false),
        // Post-table metatable setup instructions (such as CLOSURE before CALL/RETURN) are ignored.
        _ => {}
    }
    *pc += 1;
    Ok(true)
}

/// Evaluates the root prototype instructions into `regs` and `tables`.
pub fn eval_root_slice(
    stem: &str,
    proto: &RootProto,
    regs: &mut [Val; 256],
    tables: &mut Vec<TableObj>,
) -> Result<(), XtaskError> {
    let code_len = u32::try_from(proto.code.len()).map_err(|_| XtaskError::MalformedBytecode {
        stem: stem.into(),
        context: "code length overflow",
    })?;
    let mut pc = 0_u32;
    for _ in 0..GEN_INSTRUCTIONS_MAX {
        if pc >= code_len {
            return Ok(());
        }
        if !step_gen_instruction(stem, proto, regs, tables, &mut pc)? {
            return Ok(());
        }
    }
    Err(XtaskError::StepLimitExceeded {
        stem: stem.into(),
        max: GEN_INSTRUCTIONS_MAX,
    })
}

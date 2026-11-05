//! Wire cursor and iterative `\x01XDI` -> 64-bit PUC-Lua 5.3 transpiler and validator.

use crate::luac::error::LuacError;
use crate::luac::opcode::LuaInstruction;
use crate::luac::types::{
    ByteSpan, LUA53_HEADER, LuacLimitKind, LuacLimits, PROTO_DEPTH_HARD_MAX_USIZE, XDI_MAGIC,
};
use crate::{FixedVec, bounded_len_u32, u32_to_usize};
use arrayvec::ArrayVec;
use zerocopy::byteorder::little_endian::{F64, I64, U32, U64};
use zerocopy::{FromBytes, Immutable, KnownLayout, Unaligned};

/// Checked slice-shrinking wire cursor tracking the current 32-bit byte offset.
pub(crate) struct WireCursor<'a> {
    full: &'a [u8],
    rest: &'a [u8],
    offset: u32,
}

impl<'a> WireCursor<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Result<Self, LuacError> {
        if u32::try_from(buf.len()).is_err() {
            return Err(LuacError::LimitExceeded {
                kind: LuacLimitKind::InputBytes,
                actual: buf.len() as u64,
                max: u64::from(u32::MAX),
            });
        }
        Ok(Self {
            full: buf,
            rest: buf,
            offset: 0,
        })
    }

    #[inline]
    #[must_use]
    pub(crate) const fn full_buf(&self) -> &'a [u8] {
        self.full
    }

    #[inline]
    #[must_use]
    pub(crate) const fn offset(&self) -> u32 {
        self.offset
    }

    #[inline]
    #[must_use]
    pub(crate) const fn remaining_len(&self) -> u32 {
        bounded_len_u32(self.rest.len())
    }

    #[inline]
    #[must_use]
    pub(crate) const fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }

    #[inline]
    pub(crate) fn take_bytes(&mut self, needed: u32) -> Result<&'a [u8], LuacError> {
        let Some((head, tail)) = self.rest.split_at_checked(u32_to_usize(needed)) else {
            return Err(LuacError::UnexpectedEof {
                offset: self.offset,
                needed,
                remaining: bounded_len_u32(self.rest.len()),
            });
        };
        self.rest = tail;
        self.offset += needed;
        Ok(head)
    }

    #[inline]
    fn take<T: FromBytes + KnownLayout + Immutable + Unaligned>(
        &mut self,
    ) -> Result<&'a T, LuacError> {
        let size_u32 = bounded_len_u32(core::mem::size_of::<T>());
        let Ok((val, tail)) = T::ref_from_prefix(self.rest) else {
            return Err(LuacError::UnexpectedEof {
                offset: self.offset,
                needed: size_u32,
                remaining: bounded_len_u32(self.rest.len()),
            });
        };
        self.rest = tail;
        self.offset += size_u32;
        Ok(val)
    }

    #[inline]
    pub(crate) fn read_u8(&mut self) -> Result<u8, LuacError> {
        Ok(*self.take::<u8>()?)
    }

    #[inline]
    pub(crate) fn read_u32(&mut self) -> Result<u32, LuacError> {
        Ok(self.take::<U32>()?.get())
    }

    #[inline]
    pub(crate) fn read_u64(&mut self) -> Result<u64, LuacError> {
        Ok(self.take::<U64>()?.get())
    }

    #[inline]
    pub(crate) fn read_i64(&mut self) -> Result<i64, LuacError> {
        Ok(self.take::<I64>()?.get())
    }

    #[inline]
    pub(crate) fn read_f64(&mut self) -> Result<f64, LuacError> {
        Ok(self.take::<F64>()?.get())
    }

    /// Reads a `LuaString` in `\x01XDI` or canonical 64-bit Lua 5.3 format.
    pub(crate) fn read_string(&mut self, is_xdi: bool) -> Result<Option<ByteSpan>, LuacError> {
        let tag_offset = self.offset;
        let tag = self.read_u8()?;
        if tag == 0 {
            return Ok(None);
        }
        let byte_len = if is_xdi {
            if tag < 0x0A {
                return Err(LuacError::InvalidXdiStringTag {
                    offset: tag_offset,
                    tag,
                });
            }
            if tag < 0xFF {
                u32::from(tag - 10)
            } else {
                let len_offset = self.offset;
                let raw = self.read_u32()?;
                let Some(len) = raw.checked_sub(10) else {
                    return Err(LuacError::InvalidXdiLongStringLength {
                        offset: len_offset,
                        raw_len: raw,
                    });
                };
                len
            }
        } else if tag < 0xFF {
            u32::from(tag - 1)
        } else {
            let len_offset = self.offset;
            let raw = self.read_u64()?;
            let Some(dec) = raw.checked_sub(1) else {
                return Err(LuacError::InvalidLua53StringLength {
                    offset: len_offset,
                    raw_len: raw,
                });
            };
            let Ok(len) = u32::try_from(dec) else {
                return Err(LuacError::InvalidLua53StringLength {
                    offset: len_offset,
                    raw_len: raw,
                });
            };
            len
        };
        let payload_offset = self.offset;
        let _payload = self.take_bytes(byte_len)?;
        Ok(Some(ByteSpan {
            offset: payload_offset,
            len: byte_len,
        }))
    }
}

/// Reads and validates the chunk header, returning `(is_xdi, main_upvalue_count)`.
pub(crate) fn read_chunk_header(cur: &mut WireCursor<'_>) -> Result<(bool, u8), LuacError> {
    let full = cur.full_buf();
    if full.starts_with(&XDI_MAGIC) {
        let _magic = cur.take_bytes(9)?;
        let upvals = cur.read_u8()?;
        return Ok((true, upvals));
    }
    if full.starts_with(b"\x1bLua") {
        let hdr = cur.take_bytes(33)?;
        if hdr != LUA53_HEADER {
            return Err(LuacError::InvalidLua53Header);
        }
        let upvals = cur.read_u8()?;
        return Ok((false, upvals));
    }
    if full.len() < 10 {
        return Err(LuacError::UnexpectedEof {
            offset: 0,
            needed: 10,
            remaining: bounded_len_u32(full.len()),
        });
    }
    Err(LuacError::InvalidMagic)
}

#[inline]
fn push_bytes(out: &mut FixedVec<u8>, bytes: &[u8]) -> Result<(), LuacError> {
    if !out.try_extend_from_slice(bytes) {
        let actual = u64::from(out.len()) + u64::from(bounded_len_u32(bytes.len()));
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::TranspileBytes,
            actual,
            max: u64::from(out.cap()),
        });
    }
    Ok(())
}

fn emit_lua53_string(
    full_buf: &[u8],
    span: Option<ByteSpan>,
    out: &mut FixedVec<u8>,
) -> Result<(), LuacError> {
    let Some(s) = span else {
        return push_bytes(out, &[0x00]);
    };
    let start = u32_to_usize(s.offset);
    let end = start + u32_to_usize(s.len);
    let payload = &full_buf[start..end];
    if let Ok(short_len) = u8::try_from(s.len)
        && short_len <= 253
    {
        let tag = short_len + 1;
        push_bytes(out, &[tag])?;
    } else {
        let raw_len = u64::from(s.len) + 1;
        push_bytes(out, &[0xFF])?;
        push_bytes(out, &raw_len.to_le_bytes())?;
    }
    push_bytes(out, payload)
}

struct TranspileCounters {
    protos: u32,
    instructions: u32,
    constants: u32,
    loc_vars: u32,
}

fn transpile_proto_prefix(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    proto_index: u32,
    limits: LuacLimits,
    counters: &mut TranspileCounters,
    out: &mut FixedVec<u8>,
) -> Result<u32, LuacError> {
    let src_name = cur.read_string(is_xdi)?;
    emit_lua53_string(cur.full_buf(), src_name, out)?;
    let meta = cur.take_bytes(11)?;
    push_bytes(out, meta)?;

    let size_code = cur.read_u32()?;
    counters.instructions = counters
        .instructions
        .checked_add(size_code)
        .filter(|&tot| tot <= limits.instructions_count_max())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::InstructionsCount,
            actual: u64::from(counters.instructions) + u64::from(size_code),
            max: u64::from(limits.instructions_count_max()),
        })?;
    push_bytes(out, &size_code.to_le_bytes())?;

    for pc in 0..size_code {
        let raw_w = cur.read_u32()?;
        let inst = if is_xdi {
            LuaInstruction::from_xdi_word(raw_w).map_err(|raw_op| LuacError::InvalidXdiOpcode {
                proto_index,
                pc,
                raw_op,
            })?
        } else {
            LuaInstruction::from_lua53_word(raw_w).map_err(|raw_op| {
                LuacError::InvalidLua53Opcode {
                    proto_index,
                    pc,
                    raw_op,
                }
            })?
        };
        push_bytes(out, &inst.raw().to_le_bytes())?;
    }

    transpile_constants_and_upvalues(cur, is_xdi, proto_index, limits, counters, out)
}

fn transpile_constants_and_upvalues(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    proto_index: u32,
    limits: LuacLimits,
    counters: &mut TranspileCounters,
    out: &mut FixedVec<u8>,
) -> Result<u32, LuacError> {
    let size_k = cur.read_u32()?;
    counters.constants = counters
        .constants
        .checked_add(size_k)
        .filter(|&tot| tot <= limits.constants_count_max())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::ConstantsCount,
            actual: u64::from(counters.constants) + u64::from(size_k),
            max: u64::from(limits.constants_count_max()),
        })?;
    push_bytes(out, &size_k.to_le_bytes())?;

    for const_index in 0..size_k {
        let tag_offset = cur.offset();
        let tag = cur.read_u8()?;
        push_bytes(out, &[tag])?;
        match tag {
            0x00 => {}
            0x01 => {
                let b = cur.read_u8()?;
                push_bytes(out, &[b])?;
            }
            0x03 => {
                let f = cur.read_f64()?;
                if !f.is_finite() {
                    return Err(LuacError::NonFiniteFloatConstant {
                        proto_index,
                        const_index,
                        bits: f.to_bits(),
                    });
                }
                push_bytes(out, &f.to_le_bytes())?;
            }
            0x13 => {
                let raw = cur.take_bytes(8)?;
                push_bytes(out, raw)?;
            }
            0x04 | 0x14 => {
                let s = cur.read_string(is_xdi)?;
                if s.is_none() {
                    return Err(LuacError::NullConstantString {
                        proto_index,
                        const_index,
                        tag,
                    });
                }
                emit_lua53_string(cur.full_buf(), s, out)?;
            }
            other => {
                return Err(LuacError::InvalidConstantTag {
                    offset: tag_offset,
                    proto_index,
                    const_index,
                    tag: other,
                });
            }
        }
    }

    transpile_upvalues_and_protos_count(cur, limits, out)
}

fn transpile_upvalues_and_protos_count(
    cur: &mut WireCursor<'_>,
    limits: LuacLimits,
    out: &mut FixedVec<u8>,
) -> Result<u32, LuacError> {
    let size_upvalues = cur.read_u32()?;
    if size_upvalues > limits.upvalues_count_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::UpvaluesCount,
            actual: u64::from(size_upvalues),
            max: u64::from(limits.upvalues_count_max()),
        });
    }
    assert!(limits.upvalues_count_max() <= 255);
    push_bytes(out, &size_upvalues.to_le_bytes())?;
    let upval_bytes = cur.take_bytes(size_upvalues * 2)?;
    push_bytes(out, upval_bytes)?;

    let size_p = cur.read_u32()?;
    push_bytes(out, &size_p.to_le_bytes())?;
    Ok(size_p)
}

fn transpile_debug_trailer(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    limits: LuacLimits,
    counters: &mut TranspileCounters,
    out: &mut FixedVec<u8>,
) -> Result<(), LuacError> {
    let size_line_info = cur.read_u32()?;
    if size_line_info > limits.instructions_count_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::InstructionsCount,
            actual: u64::from(size_line_info),
            max: u64::from(limits.instructions_count_max()),
        });
    }
    push_bytes(out, &size_line_info.to_le_bytes())?;
    let line_byte_len = size_line_info
        .checked_mul(4)
        .ok_or_else(|| LuacError::UnexpectedEof {
            offset: cur.offset(),
            needed: u32::MAX,
            remaining: cur.remaining_len(),
        })?;
    let line_bytes = cur.take_bytes(line_byte_len)?;
    push_bytes(out, line_bytes)?;

    let size_loc_vars = cur.read_u32()?;
    counters.loc_vars = counters
        .loc_vars
        .checked_add(size_loc_vars)
        .filter(|&tot| tot <= limits.loc_vars_count_max())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::LocVarsCount,
            actual: u64::from(counters.loc_vars) + u64::from(size_loc_vars),
            max: u64::from(limits.loc_vars_count_max()),
        })?;
    push_bytes(out, &size_loc_vars.to_le_bytes())?;
    for _ in 0..size_loc_vars {
        let name = cur.read_string(is_xdi)?;
        emit_lua53_string(cur.full_buf(), name, out)?;
        let pcs = cur.take_bytes(8)?;
        push_bytes(out, pcs)?;
    }

    let size_upval_names = cur.read_u32()?;
    if size_upval_names > limits.upvalues_count_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::UpvaluesCount,
            actual: u64::from(size_upval_names),
            max: u64::from(limits.upvalues_count_max()),
        });
    }
    push_bytes(out, &size_upval_names.to_le_bytes())?;
    for _ in 0..size_upval_names {
        let name = cur.read_string(is_xdi)?;
        emit_lua53_string(cur.full_buf(), name, out)?;
    }
    Ok(())
}

fn push_child_proto(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    limits: LuacLimits,
    counters: &mut TranspileCounters,
    stack: &mut ArrayVec<u32, PROTO_DEPTH_HARD_MAX_USIZE>,
    out: &mut FixedVec<u8>,
) -> Result<(), LuacError> {
    let next_depth = bounded_len_u32(stack.len()) + 1;
    if next_depth > limits.proto_depth_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::ProtoDepth,
            actual: u64::from(next_depth),
            max: u64::from(limits.proto_depth_max()),
        });
    }
    let proto_idx = counters.protos;
    counters.protos = counters
        .protos
        .checked_add(1)
        .filter(|&tot| tot <= limits.protos_count_max())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::ProtosCount,
            actual: u64::from(counters.protos) + 1,
            max: u64::from(limits.protos_count_max()),
        })?;
    let child_p = transpile_proto_prefix(cur, is_xdi, proto_idx, limits, counters, out)?;
    let push_res = stack.try_push(child_p);
    assert!(push_res.is_ok());
    Ok(())
}

/// Validates and transpiles a decrypted `\x01XDI` or canonical `\x1bLua` chunk into canonical 64-bit Lua 5.3 bytecode.
pub(crate) fn transpile_to_lua53_into(
    src: &[u8],
    out: &mut FixedVec<u8>,
    limits: LuacLimits,
) -> Result<(), LuacError> {
    out.clear();
    let mut cur = WireCursor::new(src)?;
    let (is_xdi, main_upvalues) = read_chunk_header(&mut cur)?;
    push_bytes(out, &LUA53_HEADER)?;
    push_bytes(out, &[main_upvalues])?;

    let mut counters = TranspileCounters {
        protos: 1,
        instructions: 0,
        constants: 0,
        loc_vars: 0,
    };
    if counters.protos > limits.protos_count_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::ProtosCount,
            actual: 1,
            max: u64::from(limits.protos_count_max()),
        });
    }

    let mut stack: ArrayVec<u32, PROTO_DEPTH_HARD_MAX_USIZE> = ArrayVec::new();
    let root_children = transpile_proto_prefix(&mut cur, is_xdi, 0, limits, &mut counters, out)?;
    let root_push = stack.try_push(root_children);
    assert!(root_push.is_ok());

    let max_steps = u64::from(limits.protos_count_max()) * 2;
    for _ in 0..max_steps {
        let Some(rem_children) = stack.last_mut() else {
            break;
        };
        if *rem_children > 0 {
            *rem_children -= 1;
            push_child_proto(&mut cur, is_xdi, limits, &mut counters, &mut stack, out)?;
        } else {
            let _popped = stack.pop();
            transpile_debug_trailer(&mut cur, is_xdi, limits, &mut counters, out)?;
        }
    }
    assert!(stack.is_empty());

    if !cur.is_empty() {
        return Err(LuacError::TrailingBytes {
            consumed: cur.offset(),
            total: bounded_len_u32(src.len()),
        });
    }
    assert_eq!(u32_to_usize(cur.offset()), src.len());
    Ok(())
}

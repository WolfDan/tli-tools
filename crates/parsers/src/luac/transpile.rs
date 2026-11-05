//! Wire cursor and iterative `\x01XDI` / 64-bit PUC-Lua 5.3 prototype arena loader and validator.

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

/// Constant entry stored in `ChunkArenas::consts`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ChunkConst {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(ByteSpan),
}

/// Upvalue descriptor and debug name span stored in `ChunkArenas::upvalues`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct UpvalDesc {
    pub(crate) name: Option<ByteSpan>,
}

/// Local variable debug descriptor stored in `ChunkArenas::loc_vars`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LocVarDesc {
    pub(crate) name: Option<ByteSpan>,
    pub(crate) start_pc: u32,
    pub(crate) end_pc: u32,
}

/// Prototype record indexing slices inside `ChunkArenas`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ProtoRecord {
    pub(crate) num_params: u8,
    pub(crate) is_vararg: u8,
    pub(crate) code_start: u32,
    pub(crate) code_len: u32,
    pub(crate) const_start: u32,
    pub(crate) const_len: u32,
    pub(crate) upval_start: u32,
    pub(crate) upval_len: u32,
    pub(crate) child_start: u32,
    pub(crate) child_len: u32,
    pub(crate) loc_start: u32,
    pub(crate) loc_len: u32,
}

/// Pre-allocated flat arenas holding all prototypes of a parsed Lua 5.3 chunk.
#[derive(Debug)]
pub(crate) struct ChunkArenas {
    pub(crate) protos: FixedVec<ProtoRecord>,
    pub(crate) code: FixedVec<LuaInstruction>,
    pub(crate) consts: FixedVec<ChunkConst>,
    pub(crate) upvalues: FixedVec<UpvalDesc>,
    pub(crate) children: FixedVec<u32>,
    pub(crate) loc_vars: FixedVec<LocVarDesc>,
}

impl ChunkArenas {
    #[must_use]
    pub(crate) fn with_limits(limits: LuacLimits) -> Self {
        let raw_upvals = (u64::from(limits.protos_count_max())
            * u64::from(limits.upvalues_count_max()))
        .min(524_288);
        let upvals_cap = (raw_upvals & 0xFFFF_FFFF) as u32;
        Self {
            protos: FixedVec::with_capacity(limits.protos_count_max()),
            code: FixedVec::with_capacity(limits.instructions_count_max()),
            consts: FixedVec::with_capacity(limits.constants_count_max()),
            upvalues: FixedVec::with_capacity(upvals_cap),
            children: FixedVec::with_capacity(limits.protos_count_max()),
            loc_vars: FixedVec::with_capacity(limits.loc_vars_count_max()),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.protos.clear();
        self.code.clear();
        self.consts.clear();
        self.upvalues.clear();
        self.children.clear();
        self.loc_vars.clear();
    }
}

#[derive(Clone, Copy)]
struct ArenaSpan {
    start: u32,
    len: u32,
}

#[derive(Clone, Copy)]
struct UpvalAndChildSpan {
    upvals: ArenaSpan,
    children: ArenaSpan,
}

#[derive(Clone, Copy)]
struct ProtoLoadFrame {
    proto_idx: u32,
    child_start: u32,
    next_child_offset: u32,
    child_len: u32,
}

fn load_proto_code(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    proto_index: u32,
    limits: LuacLimits,
    arenas: &mut ChunkArenas,
) -> Result<ArenaSpan, LuacError> {
    let size_code = cur.read_u32()?;
    let code_start = arenas.code.len();
    let _new_total = code_start
        .checked_add(size_code)
        .filter(|&tot| tot <= limits.instructions_count_max())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::InstructionsCount,
            actual: u64::from(code_start) + u64::from(size_code),
            max: u64::from(limits.instructions_count_max()),
        })?;

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
        let push_res = arenas.code.try_push(inst);
        assert!(push_res.is_ok());
    }
    Ok(ArenaSpan {
        start: code_start,
        len: size_code,
    })
}

fn load_proto_constants(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    proto_index: u32,
    limits: LuacLimits,
    arenas: &mut ChunkArenas,
) -> Result<ArenaSpan, LuacError> {
    let size_k = cur.read_u32()?;
    let const_start = arenas.consts.len();
    let _new_total = const_start
        .checked_add(size_k)
        .filter(|&tot| tot <= limits.constants_count_max())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::ConstantsCount,
            actual: u64::from(const_start) + u64::from(size_k),
            max: u64::from(limits.constants_count_max()),
        })?;

    for const_index in 0..size_k {
        let tag_offset = cur.offset();
        let tag = cur.read_u8()?;
        let entry = match tag {
            0x00 => ChunkConst::Nil,
            0x01 => ChunkConst::Bool(cur.read_u8()? != 0),
            0x03 => {
                let f = cur.read_f64()?;
                if !f.is_finite() {
                    return Err(LuacError::NonFiniteFloatConstant {
                        proto_index,
                        const_index,
                        bits: f.to_bits(),
                    });
                }
                ChunkConst::Float(f)
            }
            0x13 => ChunkConst::Int(cur.read_i64()?),
            0x04 | 0x14 => {
                let span = cur
                    .read_string(is_xdi)?
                    .ok_or(LuacError::NullConstantString {
                        proto_index,
                        const_index,
                        tag,
                    })?;
                ChunkConst::Str(span)
            }
            other => {
                return Err(LuacError::InvalidConstantTag {
                    offset: tag_offset,
                    proto_index,
                    const_index,
                    tag: other,
                });
            }
        };
        let push_res = arenas.consts.try_push(entry);
        assert!(push_res.is_ok());
    }
    Ok(ArenaSpan {
        start: const_start,
        len: size_k,
    })
}

fn load_proto_upvalues_and_children(
    cur: &mut WireCursor<'_>,
    limits: LuacLimits,
    arenas: &mut ChunkArenas,
) -> Result<UpvalAndChildSpan, LuacError> {
    let size_upvalues = cur.read_u32()?;
    if size_upvalues > limits.upvalues_count_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::UpvaluesCount,
            actual: u64::from(size_upvalues),
            max: u64::from(limits.upvalues_count_max()),
        });
    }
    assert!(limits.upvalues_count_max() <= 255);
    let _upval_bytes = cur.take_bytes(size_upvalues * 2)?;
    let upval_start = arenas.upvalues.len();
    for _ in 0..size_upvalues {
        if arenas.upvalues.try_push(UpvalDesc { name: None }).is_err() {
            return Err(LuacError::LimitExceeded {
                kind: LuacLimitKind::UpvaluesCount,
                actual: u64::from(arenas.upvalues.len()) + 1,
                max: u64::from(arenas.upvalues.cap()),
            });
        }
    }

    let size_p = cur.read_u32()?;
    let child_start = arenas.children.len();
    let _new_children = child_start
        .checked_add(size_p)
        .filter(|&tot| tot <= limits.protos_count_max())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::ProtosCount,
            actual: u64::from(arenas.protos.len()) + 1 + u64::from(size_p),
            max: u64::from(limits.protos_count_max()),
        })?;
    for _ in 0..size_p {
        let push_res = arenas.children.try_push(0);
        assert!(push_res.is_ok());
    }
    Ok(UpvalAndChildSpan {
        upvals: ArenaSpan {
            start: upval_start,
            len: size_upvalues,
        },
        children: ArenaSpan {
            start: child_start,
            len: size_p,
        },
    })
}

fn load_proto_prefix(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    proto_index: u32,
    limits: LuacLimits,
    arenas: &mut ChunkArenas,
) -> Result<ProtoLoadFrame, LuacError> {
    let _src_name = cur.read_string(is_xdi)?;
    let meta = cur.take_bytes(11)?;
    let num_params = meta[8];
    let is_vararg = meta[9];

    let code_span = load_proto_code(cur, is_xdi, proto_index, limits, arenas)?;
    let const_span = load_proto_constants(cur, is_xdi, proto_index, limits, arenas)?;
    let up_child = load_proto_upvalues_and_children(cur, limits, arenas)?;

    let push_res = arenas.protos.try_push(ProtoRecord {
        num_params,
        is_vararg,
        code_start: code_span.start,
        code_len: code_span.len,
        const_start: const_span.start,
        const_len: const_span.len,
        upval_start: up_child.upvals.start,
        upval_len: up_child.upvals.len,
        child_start: up_child.children.start,
        child_len: up_child.children.len,
        loc_start: 0,
        loc_len: 0,
    });
    assert!(push_res.is_ok());

    Ok(ProtoLoadFrame {
        proto_idx: proto_index,
        child_start: up_child.children.start,
        next_child_offset: 0,
        child_len: up_child.children.len,
    })
}

fn load_debug_trailer(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    proto_idx: u32,
    limits: LuacLimits,
    arenas: &mut ChunkArenas,
) -> Result<(), LuacError> {
    let size_line_info = cur.read_u32()?;
    if size_line_info > limits.instructions_count_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::InstructionsCount,
            actual: u64::from(size_line_info),
            max: u64::from(limits.instructions_count_max()),
        });
    }
    let line_byte_len = size_line_info
        .checked_mul(4)
        .ok_or_else(|| LuacError::UnexpectedEof {
            offset: cur.offset(),
            needed: u32::MAX,
            remaining: cur.remaining_len(),
        })?;
    let _line_bytes = cur.take_bytes(line_byte_len)?;

    let size_loc_vars = cur.read_u32()?;
    let loc_start = arenas.loc_vars.len();
    let _new_locs = loc_start
        .checked_add(size_loc_vars)
        .filter(|&tot| tot <= limits.loc_vars_count_max())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::LocVarsCount,
            actual: u64::from(loc_start) + u64::from(size_loc_vars),
            max: u64::from(limits.loc_vars_count_max()),
        })?;
    for _ in 0..size_loc_vars {
        let name = cur.read_string(is_xdi)?;
        let start_pc = cur.read_u32()?;
        let end_pc = cur.read_u32()?;
        let push_res = arenas.loc_vars.try_push(LocVarDesc {
            name,
            start_pc,
            end_pc,
        });
        assert!(push_res.is_ok());
    }

    let size_upval_names = cur.read_u32()?;
    if size_upval_names > limits.upvalues_count_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::UpvaluesCount,
            actual: u64::from(size_upval_names),
            max: u64::from(limits.upvalues_count_max()),
        });
    }
    let proto_rec = &mut arenas.protos.as_mut_slice()[u32_to_usize(proto_idx)];
    proto_rec.loc_start = loc_start;
    proto_rec.loc_len = size_loc_vars;
    let upval_start = proto_rec.upval_start;
    let upval_len = proto_rec.upval_len;

    for i in 0..size_upval_names {
        let name = cur.read_string(is_xdi)?;
        if i < upval_len {
            arenas.upvalues.as_mut_slice()[u32_to_usize(upval_start + i)].name = name;
        }
    }
    Ok(())
}

fn push_child_proto(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    slot_idx: u32,
    limits: LuacLimits,
    stack: &mut ArrayVec<ProtoLoadFrame, PROTO_DEPTH_HARD_MAX_USIZE>,
    arenas: &mut ChunkArenas,
) -> Result<(), LuacError> {
    let next_depth = bounded_len_u32(stack.len()) + 1;
    if next_depth > limits.proto_depth_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::ProtoDepth,
            actual: u64::from(next_depth),
            max: u64::from(limits.proto_depth_max()),
        });
    }
    let proto_idx = arenas.protos.len();
    let _new_protos = proto_idx
        .checked_add(1)
        .filter(|&tot| tot <= limits.protos_count_max())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::ProtosCount,
            actual: u64::from(proto_idx) + 1,
            max: u64::from(limits.protos_count_max()),
        })?;
    arenas.children.as_mut_slice()[u32_to_usize(slot_idx)] = proto_idx;
    let child_frame = load_proto_prefix(cur, is_xdi, proto_idx, limits, arenas)?;
    let push_res = stack.try_push(child_frame);
    assert!(push_res.is_ok());
    Ok(())
}

/// Validates and loads a decrypted `\x01XDI` or canonical `\x1bLua` chunk into flat `ChunkArenas`.
pub(crate) fn load_chunk_into(
    src: &[u8],
    arenas: &mut ChunkArenas,
    limits: LuacLimits,
) -> Result<(), LuacError> {
    arenas.clear();
    let mut cur = WireCursor::new(src)?;
    let (is_xdi, _main_upvalues) = read_chunk_header(&mut cur)?;

    if limits.protos_count_max() == 0 {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::ProtosCount,
            actual: 1,
            max: 0,
        });
    }

    let mut stack: ArrayVec<ProtoLoadFrame, PROTO_DEPTH_HARD_MAX_USIZE> = ArrayVec::new();
    let root_frame = load_proto_prefix(&mut cur, is_xdi, 0, limits, arenas)?;
    let root_push = stack.try_push(root_frame);
    assert!(root_push.is_ok());

    let max_steps = u64::from(limits.protos_count_max()) * 2;
    for _ in 0..max_steps {
        let Some(top) = stack.last_mut() else {
            break;
        };
        if top.next_child_offset < top.child_len {
            let slot_idx = top.child_start + top.next_child_offset;
            top.next_child_offset += 1;
            push_child_proto(&mut cur, is_xdi, slot_idx, limits, &mut stack, arenas)?;
        } else {
            let finished = top.proto_idx;
            let _popped = stack.pop();
            load_debug_trailer(&mut cur, is_xdi, finished, limits, arenas)?;
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

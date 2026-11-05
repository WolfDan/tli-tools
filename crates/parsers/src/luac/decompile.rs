//! From-scratch TigerStyle Lua 5.3 bytecode decompiler for `.out` / `\x01XDI` and canonical `\x1bLua` chunks.

use crate::luac::crypto::decrypt_luac_in_place;
use crate::luac::error::{DecompileError, LuacError};
use crate::luac::opcode::{LuaInstruction, LuaOpcode};
use crate::luac::transpile::{
    ChunkArenas, ChunkConst, LocVarDesc, ProtoRecord, UpvalDesc, load_chunk_into,
};
use crate::luac::types::{ByteSpan, DECOMPILE_BLOCK_DEPTH_MAX, LuacLimitKind, LuacLimits};
use crate::{FixedVec, bounded_len_u32, u32_to_usize};

/// Indicates whether the decompiler produced pure Lua source or diagnostic fallback pseudocode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecompiledChunkKind {
    /// Complete target-dialect Lua 5.3 source code.
    Source,
    /// Permissive recovery output containing diagnostic fallback constructs.
    DiagnosticPseudocode,
}

/// Decompiled Lua 5.3 script output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompiledLua {
    source: Box<str>,
    kind: DecompiledChunkKind,
}

impl DecompiledLua {
    /// Returns the decompiled Lua source text.
    #[inline]
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Consumes `self` and returns the boxed Lua source string.
    #[inline]
    #[must_use]
    pub fn into_source(self) -> Box<str> {
        self.source
    }

    /// Returns whether the generated chunk is pure source or diagnostic pseudocode.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> DecompiledChunkKind {
        self.kind
    }
}

/// Configuration for `decompile_luac`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecompileConfig {
    limits: LuacLimits,
    strict_mode: bool,
    include_comments: bool,
    indent_width: u8,
    max_line_length: u16,
}

impl DecompileConfig {
    /// Default configuration (`LuacLimits::DEFAULT`, permissive mode, diagnostic comments disabled).
    pub const DEFAULT: Self = Self {
        limits: LuacLimits::DEFAULT,
        strict_mode: false,
        include_comments: false,
        indent_width: 4,
        max_line_length: 100,
    };

    #[must_use]
    pub const fn with_limits(mut self, limits: LuacLimits) -> Self {
        self.limits = limits;
        self
    }

    #[must_use]
    pub const fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.strict_mode = strict_mode;
        self
    }

    #[must_use]
    pub const fn with_include_comments(mut self, include_comments: bool) -> Self {
        self.include_comments = include_comments;
        self
    }

    #[must_use]
    pub const fn with_indent_width(mut self, indent_width: u8) -> Self {
        self.indent_width = indent_width;
        self
    }

    #[must_use]
    pub const fn with_max_line_length(mut self, max_line_length: u16) -> Self {
        self.max_line_length = max_line_length;
        self
    }

    #[must_use]
    pub const fn limits(&self) -> LuacLimits {
        self.limits
    }

    #[must_use]
    pub const fn strict_mode(&self) -> bool {
        self.strict_mode
    }

    #[must_use]
    pub const fn include_comments(&self) -> bool {
        self.include_comments
    }

    #[must_use]
    pub const fn indent_width(&self) -> u8 {
        self.indent_width
    }

    #[must_use]
    pub const fn max_line_length(&self) -> u16 {
        self.max_line_length
    }
}

/// Reusable pre-allocated scratch arenas and output buffer for Lua 5.3 decompilation.
#[derive(Debug)]
pub struct DecompileScratch {
    pub(crate) arenas: ChunkArenas,
    pub(crate) out_bytes: FixedVec<u8>,
    child_sources: FixedVec<Box<str>>,
    blocks: FixedVec<BlockFrame>,
    regs: Box<[String; 256]>,
}

impl DecompileScratch {
    /// Allocates a reusable `DecompileScratch` bounded by `limits`.
    #[must_use]
    pub fn with_limits(limits: LuacLimits) -> Self {
        Self {
            arenas: ChunkArenas::with_limits(limits),
            out_bytes: FixedVec::with_capacity(limits.transpile_bytes_max()),
            child_sources: FixedVec::with_capacity(limits.protos_count_max()),
            blocks: FixedVec::with_capacity(DECOMPILE_BLOCK_DEPTH_MAX),
            regs: Box::new(core::array::from_fn(|i| format!("r{i}"))),
        }
    }
}

#[inline]
const fn low_u8(val: u16) -> u8 {
    (val & 0xFF) as u8
}

#[inline]
const fn jump_target(pc: u32, inst: LuaInstruction) -> Option<u32> {
    let Some(next_pc) = pc.checked_add(1) else {
        return None;
    };
    let Some(sum) = next_pc.checked_add(inst.bx()) else {
        return None;
    };
    sum.checked_sub(131_071)
}

fn is_valid_lua_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[inline]
fn push_hex_escaped_byte(out: &mut String, b: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push_str("\\x");
    out.push(char::from(HEX[usize::from(b >> 4)]));
    out.push(char::from(HEX[usize::from(b & 0x0F)]));
}

fn format_escaped_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() + 2);
    out.push('"');
    for &b in bytes {
        match b {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7E => out.push(char::from(b)),
            _ => push_hex_escaped_byte(&mut out, b),
        }
    }
    out.push('"');
    out
}

fn format_lua_string(buf: &[u8], span: ByteSpan) -> String {
    let s = u32_to_usize(span.offset);
    let e = s + u32_to_usize(span.len);
    let bytes = &buf[s..e];
    let Ok(text) = core::str::from_utf8(bytes) else {
        return format_escaped_bytes(bytes);
    };
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let mut utf8 = [0_u8; 4];
                for &b in c.encode_utf8(&mut utf8).as_bytes() {
                    push_hex_escaped_byte(&mut out, b);
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn format_const(buf: &[u8], c: ChunkConst) -> String {
    match c {
        ChunkConst::Nil => "nil".to_owned(),
        ChunkConst::Bool(b) => if b { "true" } else { "false" }.to_owned(),
        ChunkConst::Int(i) => i.to_string(),
        ChunkConst::Float(f) => {
            if f.fract() == 0.0 {
                format!("{f:.1}")
            } else {
                f.to_string()
            }
        }
        ChunkConst::Str(sp) => format_lua_string(buf, sp),
    }
}

#[derive(Debug, Clone, Copy)]
enum BlockFrame {
    IfEnd { end_pc: u32 },
    IfElse { else_pc: u32, end_pc: u32 },
    WhileEnd { end_pc: u32 },
}

#[derive(Clone, Copy)]
enum CondKind {
    Eq,
    Lt,
    Le,
    Test,
    TestSet,
}

struct ProtoDecompiler<'a> {
    buf: &'a [u8],
    proto_idx: u32,
    proto: ProtoRecord,
    code: &'a [LuaInstruction],
    consts: &'a [ChunkConst],
    upvals: &'a [UpvalDesc],
    children: &'a [u32],
    loc_vars: &'a [LocVarDesc],
    child_sources: &'a [Box<str>],
    blocks: &'a mut FixedVec<BlockFrame>,
    regs: &'a mut [String; 256],
    out: &'a mut FixedVec<u8>,
    config: &'a DecompileConfig,
    is_root: bool,
    pc: u32,
    depth: u32,
    top_open_reg: Option<u8>,
    kind: DecompiledChunkKind,
}

impl<'a> ProtoDecompiler<'a> {
    fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), LuacError> {
        let max_bytes = self.config.limits().transpile_bytes_max();
        let next_len = u64::from(self.out.len()) + u64::from(bounded_len_u32(bytes.len()));
        if next_len > u64::from(max_bytes) || !self.out.try_extend_from_slice(bytes) {
            return Err(LuacError::LimitExceeded {
                kind: LuacLimitKind::TranspileBytes,
                actual: next_len,
                max: u64::from(max_bytes),
            });
        }
        Ok(())
    }

    fn push_str(&mut self, s: &str) -> Result<(), LuacError> {
        self.push_bytes(s.as_bytes())
    }

    fn write_indent(&mut self, depth: u32) -> Result<(), LuacError> {
        const SPACES: &[u8; 64] = &[b' '; 64];
        assert!(depth <= DECOMPILE_BLOCK_DEPTH_MAX + 1);
        let total = u32_to_usize(depth * u32::from(self.config.indent_width()));
        let mut rem = total;
        while rem > 0 {
            let chunk = rem.min(SPACES.len());
            self.push_bytes(&SPACES[..chunk])?;
            rem -= chunk;
        }
        Ok(())
    }

    fn write_line(&mut self, depth: u32, line: &str) -> Result<(), LuacError> {
        self.write_indent(depth)?;
        self.push_str(line)?;
        self.push_str("\n")
    }

    fn checked_reg_add(&mut self, base: u8, offset: u16) -> Result<Option<u8>, LuacError> {
        let sum = u16::from(base) + offset;
        if sum <= 255 {
            return Ok(Some(low_u8(sum)));
        }
        if self.config.strict_mode() {
            return Err(DecompileError::RegisterSpanOutOfBounds {
                proto_index: self.proto_idx,
                pc: self.pc,
                reg: base,
                offset,
            }
            .into());
        }
        self.kind = DecompiledChunkKind::DiagnosticPseudocode;
        Ok(None)
    }

    fn pop_loop_depth(&mut self) -> Result<(), LuacError> {
        let min_depth = u32::from(!self.is_root);
        if self.depth <= min_depth {
            if self.config.strict_mode() {
                return Err(DecompileError::UnmatchedBlockEnd {
                    proto_index: self.proto_idx,
                    pc: self.pc,
                }
                .into());
            }
            self.kind = DecompiledChunkKind::DiagnosticPseudocode;
            return self.write_line(min_depth, "end");
        }
        self.depth -= 1;
        self.write_line(self.depth, "end")
    }

    fn span_ident(&mut self, span: ByteSpan) -> Result<Option<&'a str>, LuacError> {
        let s = u32_to_usize(span.offset);
        let e = s + u32_to_usize(span.len);
        match core::str::from_utf8(&self.buf[s..e]) {
            Ok(text) => Ok(Some(text)),
            Err(err) => {
                if self.config.strict_mode() {
                    Err(LuacError::InvalidUtf8 { source: err })
                } else {
                    self.kind = DecompiledChunkKind::DiagnosticPseudocode;
                    Ok(None)
                }
            }
        }
    }

    fn resolve_local_name(&mut self, reg: u8, pc: u32) -> Result<Option<String>, LuacError> {
        let mut slot = 0_u32;
        let target = u32::from(reg);
        for &lv in self.loc_vars {
            if lv.start_pc <= pc && pc < lv.end_pc {
                if slot == target {
                    let Some(sp) = lv.name else {
                        return Ok(Some(format!("_loc_{reg}_{}", lv.start_pc)));
                    };
                    let Some(raw) = self.span_ident(sp)? else {
                        return Ok(Some(format!("_loc_{reg}_{}", lv.start_pc)));
                    };
                    if !raw.starts_with('(') {
                        return Ok(Some(raw.to_owned()));
                    }
                    return Ok(None);
                }
                slot += 1;
            }
        }
        Ok(None)
    }

    fn flush_new_locals(&mut self, next_pc: u32) -> Result<(), LuacError> {
        let mut slot = 0_u32;
        for &lv in self.loc_vars {
            if lv.start_pc <= next_pc && next_pc <= lv.end_pc {
                if lv.start_pc == next_pc
                    && slot <= 255
                    && let Some(sp) = lv.name
                    && let Some(raw) = self.span_ident(sp)?
                    && !raw.starts_with('(')
                {
                    let slot_idx = u32_to_usize(slot);
                    let raw_owned = raw.to_owned();
                    let init_val = self.regs[slot_idx].clone();
                    if init_val != raw_owned {
                        let line = format!("local {raw_owned} = {init_val}");
                        self.write_line(self.depth, &line)?;
                    } else {
                        let line = format!("local {raw_owned}");
                        self.write_line(self.depth, &line)?;
                    }
                    self.regs[slot_idx] = raw_owned;
                }
                slot += 1;
            }
        }
        Ok(())
    }

    fn resolve_upval_name(&mut self, pc: u32, idx: u16) -> Result<String, LuacError> {
        let Some(u) = self.upvals.get(usize::from(idx)) else {
            if self.config.strict_mode() {
                return Err(DecompileError::UpvalueIndexOutOfBounds {
                    proto_index: self.proto_idx,
                    pc,
                    index: idx,
                    count: self.proto.upval_len,
                }
                .into());
            }
            self.kind = DecompiledChunkKind::DiagnosticPseudocode;
            return Ok(format!("_upval_{idx}"));
        };
        if let Some(sp) = u.name
            && let Some(raw) = self.span_ident(sp)?
        {
            return Ok(raw.to_owned());
        }
        Ok(format!("_upval_{idx}"))
    }

    fn read_const(&mut self, pc: u32, k_idx: u32) -> Result<String, LuacError> {
        let Some(&cv) = self.consts.get(u32_to_usize(k_idx)) else {
            if self.config.strict_mode() {
                return Err(DecompileError::ConstantIndexOutOfBounds {
                    proto_index: self.proto_idx,
                    pc,
                    index: k_idx,
                    count: self.proto.const_len,
                }
                .into());
            }
            self.kind = DecompiledChunkKind::DiagnosticPseudocode;
            return Ok("nil".to_owned());
        };
        Ok(format_const(self.buf, cv))
    }

    fn read_reg(&mut self, reg: u8, pc: u32) -> Result<String, LuacError> {
        if let Some(loc) = self.resolve_local_name(reg, pc)? {
            return Ok(loc);
        }
        Ok(self.regs[usize::from(reg)].clone())
    }

    fn read_rk(&mut self, rk_val: u16, pc: u32) -> Result<String, LuacError> {
        if (rk_val & 0x100) != 0 {
            let k_idx = u32::from(rk_val & 0xFF);
            self.read_const(pc, k_idx)
        } else {
            self.read_reg(low_u8(rk_val), pc)
        }
    }

    fn assign_reg(&mut self, a: u8, expr: String, pc: u32) -> Result<(), LuacError> {
        if let Some(existing_loc) = self.resolve_local_name(a, pc)? {
            if existing_loc != expr {
                let line = format!("{existing_loc} = {expr}");
                self.write_line(self.depth, &line)?;
            }
            self.regs[usize::from(a)] = existing_loc;
        } else {
            self.regs[usize::from(a)] = expr;
        }
        Ok(())
    }

    fn format_table_access(
        &mut self,
        table_expr: &str,
        key_rk: u16,
        pc: u32,
        is_env: bool,
    ) -> Result<String, LuacError> {
        if (key_rk & 0x100) != 0
            && let Some(&ChunkConst::Str(sp)) = self.consts.get(usize::from(key_rk & 0xFF))
        {
            let s = u32_to_usize(sp.offset);
            let e = s + u32_to_usize(sp.len);
            if let Ok(ident) = core::str::from_utf8(&self.buf[s..e])
                && is_valid_lua_ident(ident)
            {
                if is_env {
                    return Ok(ident.to_owned());
                }
                return Ok(format!("{table_expr}.{ident}"));
            }
            let quoted = format_lua_string(self.buf, sp);
            return Ok(format!("{table_expr}[{quoted}]"));
        }
        let key_expr = self.read_rk(key_rk, pc)?;
        Ok(format!("{table_expr}[{key_expr}]"))
    }

    fn push_block(&mut self, frame: BlockFrame) -> Result<(), LuacError> {
        if self.blocks.try_push(frame).is_err() {
            return Err(DecompileError::BlockDepthExceeded {
                proto_index: self.proto_idx,
                pc: self.pc,
                max: DECOMPILE_BLOCK_DEPTH_MAX,
            }
            .into());
        }
        Ok(())
    }

    fn close_blocks_at_pc(&mut self) -> Result<(), LuacError> {
        while let Some(&top) = self.blocks.as_slice().last() {
            match top {
                BlockFrame::IfElse { else_pc, end_pc } => {
                    if self.pc == else_pc {
                        let _popped = self.blocks.pop();
                        assert!(self.depth > 0);
                        self.write_line(self.depth - 1, "else")?;
                        self.push_block(BlockFrame::IfEnd { end_pc })?;
                    } else if self.pc >= end_pc {
                        let _popped = self.blocks.pop();
                        assert!(self.depth > 0);
                        self.depth -= 1;
                        self.write_line(self.depth, "end")?;
                    } else {
                        break;
                    }
                }
                BlockFrame::IfEnd { end_pc } | BlockFrame::WhileEnd { end_pc } => {
                    if self.pc >= end_pc {
                        let _popped = self.blocks.pop();
                        assert!(self.depth > 0);
                        self.depth -= 1;
                        self.write_line(self.depth, "end")?;
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn op_load_kx(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let load_pc = self.pc;
        let Some(&next_inst) = self.code.get(u32_to_usize(self.pc + 1)) else {
            if self.config.strict_mode() {
                return Err(DecompileError::MissingExtraArg {
                    proto_index: self.proto_idx,
                    pc: load_pc,
                }
                .into());
            }
            self.kind = DecompiledChunkKind::DiagnosticPseudocode;
            return self.assign_reg(inst.a(), "nil".to_owned(), load_pc);
        };
        if next_inst.opcode() != LuaOpcode::ExtraArg {
            if self.config.strict_mode() {
                return Err(DecompileError::MissingExtraArg {
                    proto_index: self.proto_idx,
                    pc: load_pc,
                }
                .into());
            }
            self.kind = DecompiledChunkKind::DiagnosticPseudocode;
            return self.assign_reg(inst.a(), "nil".to_owned(), load_pc);
        }
        self.pc += 1;
        let rhs = self.read_const(load_pc, next_inst.ax())?;
        self.assign_reg(inst.a(), rhs, self.pc)
    }

    fn op_load_bool_or_nil(&mut self, inst: LuaInstruction, is_nil: bool) -> Result<(), LuacError> {
        let a = inst.a();
        let b = inst.b();
        if is_nil {
            let Some(end_reg) = self.checked_reg_add(a, b)? else {
                return self.assign_reg(a, "nil".to_owned(), self.pc);
            };
            for r in a..=end_reg {
                self.assign_reg(r, "nil".to_owned(), self.pc)?;
            }
        } else {
            let rhs = if b != 0 { "true" } else { "false" }.to_owned();
            self.assign_reg(a, rhs, self.pc)?;
            if inst.c() != 0 && u32_to_usize(self.pc + 2) <= self.code.len() {
                self.pc += 1;
            }
        }
        Ok(())
    }

    fn op_get_tab_up(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let up = self.resolve_upval_name(self.pc, inst.b())?;
        let is_env = up == "_ENV";
        let expr = self.format_table_access(&up, inst.c(), self.pc, is_env)?;
        self.assign_reg(inst.a(), expr, self.pc)
    }

    fn op_get_table(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let tbl = self.read_reg(low_u8(inst.b()), self.pc)?;
        let expr = self.format_table_access(&tbl, inst.c(), self.pc, false)?;
        self.assign_reg(inst.a(), expr, self.pc)
    }

    fn op_set_tab_up(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let up = self.resolve_upval_name(self.pc, u16::from(inst.a()))?;
        let is_env = up == "_ENV";
        let lhs = self.format_table_access(&up, inst.b(), self.pc, is_env)?;
        let rhs = self.read_rk(inst.c(), self.pc)?;
        let line = format!("{lhs} = {rhs}");
        self.write_line(self.depth, &line)
    }

    fn op_set_upval(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let up = self.resolve_upval_name(self.pc, inst.b())?;
        let rhs = self.read_reg(inst.a(), self.pc)?;
        let line = format!("{up} = {rhs}");
        self.write_line(self.depth, &line)
    }

    fn op_set_table(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let tbl = self.read_reg(inst.a(), self.pc)?;
        let lhs = self.format_table_access(&tbl, inst.b(), self.pc, false)?;
        let rhs = self.read_rk(inst.c(), self.pc)?;
        let line = format!("{lhs} = {rhs}");
        self.write_line(self.depth, &line)
    }

    fn op_set_list(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let a = inst.a();
        let b = inst.b();
        let tbl = self.read_reg(a, self.pc)?;
        let count = if b == 0 {
            match self.top_open_reg {
                Some(top) if top > a => u16::from(top - a),
                _ => 1,
            }
        } else {
            b
        };
        if self.checked_reg_add(a, count)?.is_some() {
            for off in 1..=count {
                let item = self.read_reg(a + low_u8(off), self.pc)?;
                let line = format!("table.insert({tbl}, {item})");
                self.write_line(self.depth, &line)?;
            }
        }
        if inst.c() == 0
            && let Some(&next_inst) = self.code.get(u32_to_usize(self.pc + 1))
            && next_inst.opcode() == LuaOpcode::ExtraArg
        {
            self.pc += 1;
        }
        Ok(())
    }

    fn op_self(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let a = inst.a();
        let c = inst.c();
        let Some(self_reg) = self.checked_reg_add(a, 1)? else {
            return Ok(());
        };
        let obj = self.read_reg(low_u8(inst.b()), self.pc)?;
        self.regs[usize::from(self_reg)] = obj.clone();
        if (c & 0x100) != 0
            && let Some(&ChunkConst::Str(sp)) = self.consts.get(usize::from(c & 0xFF))
            && let Some(ident) = self.span_ident(sp)?
            && is_valid_lua_ident(ident)
        {
            self.regs[usize::from(a)] = format!("{obj}:{ident}");
            return Ok(());
        }
        let expr = self.format_table_access(&obj, c, self.pc, false)?;
        self.regs[usize::from(a)] = expr;
        Ok(())
    }

    fn op_binop(&mut self, inst: LuaInstruction, op_str: &'static str) -> Result<(), LuacError> {
        let lhs = self.read_rk(inst.b(), self.pc)?;
        let rhs = self.read_rk(inst.c(), self.pc)?;
        let expr = format!("({lhs} {op_str} {rhs})");
        self.assign_reg(inst.a(), expr, self.pc)
    }

    fn op_unop(&mut self, inst: LuaInstruction, op_str: &'static str) -> Result<(), LuacError> {
        let rhs = self.read_reg(low_u8(inst.b()), self.pc)?;
        let expr = format!("({op_str}{rhs})");
        self.assign_reg(inst.a(), expr, self.pc)
    }

    fn op_concat(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let b = low_u8(inst.b());
        let c = low_u8(inst.c());
        if b > c {
            let rhs = self.read_reg(b, self.pc)?;
            return self.assign_reg(inst.a(), rhs, self.pc);
        }
        let mut parts = Vec::with_capacity(usize::from(c - b) + 1);
        for r in b..=c {
            parts.push(self.read_reg(r, self.pc)?);
        }
        self.assign_reg(inst.a(), parts.join(" .. "), self.pc)
    }

    fn op_for_prep(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let a = inst.a();
        let Some(loop_reg) = self.checked_reg_add(a, 3)? else {
            return Ok(());
        };
        let init = self.read_reg(a, self.pc)?;
        let limit = self.read_reg(a + 1, self.pc)?;
        let step = self.read_reg(a + 2, self.pc)?;
        let var = match self.resolve_local_name(loop_reg, self.pc + 1)? {
            Some(name) => name,
            None => format!("i_{a}"),
        };
        self.regs[usize::from(loop_reg)] = var.clone();
        let line = if step == "1" {
            format!("for {var} = {init}, {limit} do")
        } else {
            format!("for {var} = {init}, {limit}, {step} do")
        };
        self.write_line(self.depth, &line)?;
        self.depth += 1;
        Ok(())
    }

    fn op_jmp(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let Some(target) = jump_target(self.pc, inst) else {
            return Ok(());
        };
        let Some(&tf_inst) = self.code.get(u32_to_usize(target)) else {
            return Ok(());
        };
        if tf_inst.opcode() != LuaOpcode::TForCall {
            return Ok(());
        }
        let tf_a = tf_inst.a();
        let tf_c = tf_inst.c();
        if self.checked_reg_add(tf_a, 2 + tf_c)?.is_none() {
            return Ok(());
        }
        let iter_expr = self.regs[usize::from(tf_a)].clone();
        let mut vars = Vec::with_capacity(usize::from(tf_c));
        for idx in 1..=tf_c {
            let r = tf_a + 2 + low_u8(idx);
            let vname = match self.resolve_local_name(r, self.pc + 1)? {
                Some(name) => name,
                None => format!("v_{r}"),
            };
            self.regs[usize::from(r)] = vname.clone();
            vars.push(vname);
        }
        let line = format!("for {} in {iter_expr} do", vars.join(", "));
        self.write_line(self.depth, &line)?;
        self.depth += 1;
        Ok(())
    }

    fn format_cond_expr(
        &mut self,
        inst: LuaInstruction,
        kind: CondKind,
    ) -> Result<String, LuacError> {
        let a = inst.a();
        let b = inst.b();
        let c = inst.c();
        match kind {
            CondKind::Eq => {
                let op = if a == 0 { "==" } else { "~=" };
                Ok(format!(
                    "{} {op} {}",
                    self.read_rk(b, self.pc)?,
                    self.read_rk(c, self.pc)?
                ))
            }
            CondKind::Lt => {
                let op = if a == 0 { "<" } else { ">=" };
                Ok(format!(
                    "{} {op} {}",
                    self.read_rk(b, self.pc)?,
                    self.read_rk(c, self.pc)?
                ))
            }
            CondKind::Le => {
                let op = if a == 0 { "<=" } else { ">" };
                Ok(format!(
                    "{} {op} {}",
                    self.read_rk(b, self.pc)?,
                    self.read_rk(c, self.pc)?
                ))
            }
            CondKind::Test => {
                let v = self.read_reg(a, self.pc)?;
                Ok(if c == 0 { v } else { format!("not {v}") })
            }
            CondKind::TestSet => {
                let v = self.read_reg(low_u8(b), self.pc)?;
                self.regs[usize::from(a)] = v.clone();
                Ok(if c == 0 { v } else { format!("not {v}") })
            }
        }
    }

    fn op_cond(&mut self, inst: LuaInstruction, kind: CondKind) -> Result<(), LuacError> {
        let cond = self.format_cond_expr(inst, kind)?;
        let Some(&jmp_inst) = self.code.get(u32_to_usize(self.pc + 1)) else {
            return Ok(());
        };
        if jmp_inst.opcode() != LuaOpcode::Jmp {
            return Ok(());
        }
        let cond_pc = self.pc;
        self.pc += 1;
        let Some(target) = jump_target(cond_pc + 1, jmp_inst) else {
            return Ok(());
        };
        if target <= cond_pc + 2 {
            return Ok(());
        }
        if target >= 2
            && let Some(&end_jmp) = self.code.get(u32_to_usize(target - 1))
            && end_jmp.opcode() == LuaOpcode::Jmp
            && let Some(end_target) = jump_target(target - 1, end_jmp)
        {
            if end_target <= cond_pc {
                let line = format!("while {cond} do");
                self.write_line(self.depth, &line)?;
                self.depth += 1;
                return self.push_block(BlockFrame::WhileEnd { end_pc: target });
            }
            if end_target > target {
                let line = format!("if {cond} then");
                self.write_line(self.depth, &line)?;
                self.depth += 1;
                return self.push_block(BlockFrame::IfElse {
                    else_pc: target,
                    end_pc: end_target,
                });
            }
        }
        let line = format!("if {cond} then");
        self.write_line(self.depth, &line)?;
        self.depth += 1;
        self.push_block(BlockFrame::IfEnd { end_pc: target })
    }

    fn collect_call_args(&mut self, a: u8, b: u16, is_method: bool) -> Result<String, LuacError> {
        let start_off = if is_method { 2_u16 } else { 1_u16 };
        let mut args = Vec::new();
        if b > 1 {
            if self.checked_reg_add(a, b - 1)?.is_none() {
                return Ok(String::new());
            }
            for off in start_off..b {
                let r = a + low_u8(off);
                args.push(self.read_reg(r, self.pc)?);
            }
        } else if b == 0 {
            let Some(start_r) = self.checked_reg_add(a, start_off)? else {
                return Ok(String::new());
            };
            let end_r = match self.top_open_reg {
                Some(top) if top >= start_r => top,
                _ => start_r,
            };
            for r in start_r..=end_r {
                args.push(self.read_reg(r, self.pc)?);
            }
        }
        Ok(args.join(", "))
    }

    fn is_next_tforcall_jmp(&self) -> bool {
        let Some(&next_inst) = self.code.get(u32_to_usize(self.pc + 1)) else {
            return false;
        };
        if next_inst.opcode() != LuaOpcode::Jmp {
            return false;
        }
        let Some(target) = jump_target(self.pc + 1, next_inst) else {
            return false;
        };
        match self.code.get(u32_to_usize(target)) {
            Some(tf) => tf.opcode() == LuaOpcode::TForCall,
            None => false,
        }
    }

    fn op_call(&mut self, inst: LuaInstruction, is_tail: bool) -> Result<(), LuacError> {
        let a = inst.a();
        let c = inst.c();
        let callee = self.read_reg(a, self.pc)?;
        let is_method = callee.contains(':');
        let args_str = self.collect_call_args(a, inst.b(), is_method)?;
        let call_expr = format!("{callee}({args_str})");
        self.top_open_reg = None;

        if is_tail {
            let line = format!("return {call_expr}");
            self.write_line(self.depth, &line)?;
            if let Some(&next_inst) = self.code.get(u32_to_usize(self.pc + 1))
                && next_inst.opcode() == LuaOpcode::Return
            {
                self.pc += 1;
            }
            return Ok(());
        }
        if c == 0 {
            self.regs[usize::from(a)] = call_expr;
            self.top_open_reg = Some(a);
            return Ok(());
        }
        if c == 1 {
            return self.write_line(self.depth, &call_expr);
        }
        if c == 2 {
            return self.assign_reg(a, call_expr, self.pc);
        }
        if self.is_next_tforcall_jmp() {
            self.regs[usize::from(a)] = call_expr;
            return Ok(());
        }
        if self.checked_reg_add(a, c - 2)?.is_none() {
            return Ok(());
        }
        let mut existing_lhs = Vec::new();
        for i in 0..(c - 1) {
            let r = a + low_u8(i);
            if let Some(existing) = self.resolve_local_name(r, self.pc)? {
                existing_lhs.push(existing);
            } else if i == 0 {
                self.regs[usize::from(r)] = call_expr.clone();
            } else {
                self.regs[usize::from(r)] = format!("select({}, {call_expr})", i + 1);
            }
        }
        if !existing_lhs.is_empty() {
            let line = format!("{} = {call_expr}", existing_lhs.join(", "));
            self.write_line(self.depth, &line)?;
        }
        Ok(())
    }

    fn op_return(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let a = inst.a();
        let b = inst.b();
        if b == 1 {
            if u32_to_usize(self.pc + 1) < self.code.len() || self.out.len() == 0 {
                self.write_line(self.depth, "return")?;
            }
        } else if b > 1 {
            if self.checked_reg_add(a, b - 2)?.is_none() {
                return Ok(());
            }
            let mut rets = Vec::with_capacity(usize::from(b - 1));
            for i in 0..(b - 1) {
                rets.push(self.read_reg(a + low_u8(i), self.pc)?);
            }
            let line = format!("return {}", rets.join(", "));
            self.write_line(self.depth, &line)?;
        } else {
            let val = self.read_reg(a, self.pc)?;
            let line = format!("return {val}");
            self.write_line(self.depth, &line)?;
        }
        Ok(())
    }

    fn op_closure(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        let bx = inst.bx();
        let Some(&child_id) = self.children.get(u32_to_usize(bx)) else {
            if self.config.strict_mode() {
                return Err(DecompileError::ChildProtoIndexOutOfBounds {
                    proto_index: self.proto_idx,
                    pc: self.pc,
                    index: bx,
                    count: self.proto.child_len,
                }
                .into());
            }
            self.kind = DecompiledChunkKind::DiagnosticPseudocode;
            return self.assign_reg(inst.a(), "function() end".to_owned(), self.pc);
        };
        let rhs = self.child_sources[u32_to_usize(child_id)].to_string();
        self.assign_reg(inst.a(), rhs, self.pc)
    }

    fn step_instruction(&mut self, inst: LuaInstruction) -> Result<(), LuacError> {
        match inst.opcode() {
            LuaOpcode::Move => {
                let rhs = self.read_reg(low_u8(inst.b()), self.pc)?;
                self.assign_reg(inst.a(), rhs, self.pc)
            }
            LuaOpcode::LoadK => {
                let rhs = self.read_const(self.pc, inst.bx())?;
                self.assign_reg(inst.a(), rhs, self.pc)
            }
            LuaOpcode::LoadKx => self.op_load_kx(inst),
            LuaOpcode::LoadBool => self.op_load_bool_or_nil(inst, false),
            LuaOpcode::LoadNil => self.op_load_bool_or_nil(inst, true),
            LuaOpcode::GetUpval => {
                let rhs = self.resolve_upval_name(self.pc, inst.b())?;
                self.assign_reg(inst.a(), rhs, self.pc)
            }
            LuaOpcode::GetTabUp => self.op_get_tab_up(inst),
            LuaOpcode::GetTable => self.op_get_table(inst),
            LuaOpcode::SetTabUp => self.op_set_tab_up(inst),
            LuaOpcode::SetUpval => self.op_set_upval(inst),
            LuaOpcode::SetTable => self.op_set_table(inst),
            LuaOpcode::NewTable => self.assign_reg(inst.a(), "{}".to_owned(), self.pc),
            LuaOpcode::SelfOp => self.op_self(inst),
            LuaOpcode::Add => self.op_binop(inst, "+"),
            LuaOpcode::Sub => self.op_binop(inst, "-"),
            LuaOpcode::Mul => self.op_binop(inst, "*"),
            LuaOpcode::Mod => self.op_binop(inst, "%"),
            LuaOpcode::Pow => self.op_binop(inst, "^"),
            LuaOpcode::Div => self.op_binop(inst, "/"),
            LuaOpcode::IDiv => self.op_binop(inst, "//"),
            LuaOpcode::BAnd => self.op_binop(inst, "&"),
            LuaOpcode::BOr => self.op_binop(inst, "|"),
            LuaOpcode::BXor => self.op_binop(inst, "~"),
            LuaOpcode::Shl => self.op_binop(inst, "<<"),
            LuaOpcode::Shr => self.op_binop(inst, ">>"),
            LuaOpcode::Unm => self.op_unop(inst, "-"),
            LuaOpcode::BNot => self.op_unop(inst, "~"),
            LuaOpcode::Not => self.op_unop(inst, "not "),
            LuaOpcode::Len => self.op_unop(inst, "#"),
            LuaOpcode::Concat => self.op_concat(inst),
            LuaOpcode::Jmp => self.op_jmp(inst),
            LuaOpcode::Eq => self.op_cond(inst, CondKind::Eq),
            LuaOpcode::Lt => self.op_cond(inst, CondKind::Lt),
            LuaOpcode::Le => self.op_cond(inst, CondKind::Le),
            LuaOpcode::Test => self.op_cond(inst, CondKind::Test),
            LuaOpcode::TestSet => self.op_cond(inst, CondKind::TestSet),
            LuaOpcode::Call => self.op_call(inst, false),
            LuaOpcode::TailCall => self.op_call(inst, true),
            LuaOpcode::Return => self.op_return(inst),
            LuaOpcode::ForPrep => self.op_for_prep(inst),
            LuaOpcode::ForLoop | LuaOpcode::TForLoop => self.pop_loop_depth(),
            LuaOpcode::SetList => self.op_set_list(inst),
            LuaOpcode::Closure => self.op_closure(inst),
            LuaOpcode::VarArg => {
                if inst.b() == 0 {
                    self.top_open_reg = Some(inst.a());
                }
                self.assign_reg(inst.a(), "...".to_owned(), self.pc)
            }
            LuaOpcode::TForCall | LuaOpcode::ExtraArg => Ok(()),
        }
    }

    fn emit_proto(&mut self) -> Result<DecompiledChunkKind, LuacError> {
        self.out.clear();
        self.blocks.clear();
        for (i, slot) in self.regs.iter_mut().enumerate() {
            slot.clear();
            slot.push('r');
            slot.push_str(&i.to_string());
        }
        if self.config.include_comments() && self.is_root {
            let comment = format!(
                "-- proto {} (params={}, code={})",
                self.proto_idx, self.proto.num_params, self.proto.code_len
            );
            self.write_line(0, &comment)?;
        }
        if !self.is_root {
            let mut params = Vec::with_capacity(usize::from(self.proto.num_params) + 1);
            for r in 0..self.proto.num_params {
                let pname = match self.resolve_local_name(r, 0)? {
                    Some(name) => name,
                    None => format!("arg_{r}"),
                };
                self.regs[usize::from(r)] = pname.clone();
                params.push(pname);
            }
            if self.proto.is_vararg != 0 {
                params.push("...".to_owned());
            }
            let header = format!("function({})\n", params.join(", "));
            self.push_str(&header)?;
            self.depth = 1;
        } else {
            for r in 0..self.proto.num_params {
                if let Some(name) = self.resolve_local_name(r, 0)? {
                    self.regs[usize::from(r)] = name;
                }
            }
        }

        let code_len = self.proto.code_len;
        while self.pc < code_len {
            self.close_blocks_at_pc()?;
            let inst = self.code[u32_to_usize(self.pc)];
            self.step_instruction(inst)?;
            self.flush_new_locals(self.pc + 1)?;
            self.pc += 1;
        }

        while self.blocks.pop().is_some() {
            assert!(self.depth > 0);
            self.depth -= 1;
            self.write_line(self.depth, "end")?;
        }
        if !self.is_root {
            self.push_str("end")?;
        }
        Ok(self.kind)
    }
}

fn decompile_single_proto(
    buf: &[u8],
    proto_idx: u32,
    scratch: &mut DecompileScratch,
    config: &DecompileConfig,
) -> Result<(Box<str>, DecompiledChunkKind), LuacError> {
    let arenas = &scratch.arenas;
    let proto = arenas.protos.as_slice()[u32_to_usize(proto_idx)];
    let cs = u32_to_usize(proto.code_start);
    let ce = cs + u32_to_usize(proto.code_len);
    let ks = u32_to_usize(proto.const_start);
    let ke = ks + u32_to_usize(proto.const_len);
    let us = u32_to_usize(proto.upval_start);
    let ue = us + u32_to_usize(proto.upval_len);
    let ps = u32_to_usize(proto.child_start);
    let pe = ps + u32_to_usize(proto.child_len);
    let ls = u32_to_usize(proto.loc_start);
    let le = ls + u32_to_usize(proto.loc_len);

    let mut dec = ProtoDecompiler {
        buf,
        proto_idx,
        proto,
        code: &arenas.code.as_slice()[cs..ce],
        consts: &arenas.consts.as_slice()[ks..ke],
        upvals: &arenas.upvalues.as_slice()[us..ue],
        children: &arenas.children.as_slice()[ps..pe],
        loc_vars: &arenas.loc_vars.as_slice()[ls..le],
        child_sources: scratch.child_sources.as_slice(),
        blocks: &mut scratch.blocks,
        regs: &mut scratch.regs,
        out: &mut scratch.out_bytes,
        config,
        is_root: proto_idx == 0,
        pc: 0,
        depth: 0,
        top_open_reg: None,
        kind: DecompiledChunkKind::Source,
    };
    let kind = dec.emit_proto()?;
    let text = core::str::from_utf8(scratch.out_bytes.as_slice())
        .map_err(|source| LuacError::InvalidUtf8 { source })?;
    Ok((Box::from(text), kind))
}

/// Decrypts, validates, and decompiles a `.out` / `\x01XDI` or canonical `\x1bLua` buffer into Lua 5.3 source.
pub fn decompile_luac(
    buf: &mut [u8],
    scratch: &mut DecompileScratch,
    config: &DecompileConfig,
) -> Result<DecompiledLua, LuacError> {
    let _cipher = decrypt_luac_in_place(buf);
    load_chunk_into(buf, &mut scratch.arenas, config.limits())?;

    let protos_count = scratch.arenas.protos.len();
    scratch.child_sources.clear();
    for _ in 0..protos_count {
        if scratch.child_sources.try_push(Box::from("")).is_err() {
            return Err(LuacError::LimitExceeded {
                kind: LuacLimitKind::ProtosCount,
                actual: u64::from(protos_count),
                max: u64::from(scratch.child_sources.cap()),
            });
        }
    }

    let mut overall_kind = DecompiledChunkKind::Source;
    for proto_idx in (0..protos_count).rev() {
        let (src, kind) = decompile_single_proto(buf, proto_idx, scratch, config)?;
        if kind == DecompiledChunkKind::DiagnosticPseudocode {
            overall_kind = DecompiledChunkKind::DiagnosticPseudocode;
        }
        if proto_idx == 0 {
            return Ok(DecompiledLua {
                source: src,
                kind: overall_kind,
            });
        }
        scratch.child_sources.as_mut_slice()[u32_to_usize(proto_idx)] = src;
    }

    Ok(DecompiledLua {
        source: Box::from(""),
        kind: overall_kind,
    })
}

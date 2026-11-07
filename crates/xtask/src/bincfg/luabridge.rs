//! Address-free `LuaBridge` (`"wolf"` namespace) struct and field offset extractor.

use crate::bincfg::domains::{WireType, normalize_field_name};
use crate::bincfg::pe::{PeImage, normalize_stem};
use crate::error::XtaskError;
use iced_x86::{Decoder, DecoderOptions, Mnemonic, OpKind, Register};
use std::collections::{BTreeMap, BTreeSet};

/// Single property registered on a `LuaBridge` C++ struct (`F*Info` or `FConfig*`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuaFieldInfo {
    pub offset: u32,
    pub name: String,
    pub is_translate_str: bool,
}

/// Discovers the `"wolf"` namespace registration helper and all `F*` class registrations in `.text`.
pub fn extract_luabridge_structs(
    pe: &PeImage,
) -> Result<BTreeMap<String, (String, Vec<LuaFieldInfo>)>, XtaskError> {
    let (_, text_slice, text_va) =
        pe.section_by_name(b".text\0\0\0")
            .ok_or_else(|| XtaskError::PeExtract {
                context: "extract_luabridge_structs",
                detail: "missing .text section".into(),
            })?;
    let (rdata_sec, _, rdata_va) =
        pe.section_by_name(b".rdata\0\0")
            .ok_or_else(|| XtaskError::PeExtract {
                context: "extract_luabridge_structs",
                detail: "missing .rdata section".into(),
            })?;
    let rdata_end_va = rdata_va + u64::from(rdata_sec.vsize.max(rdata_sec.raw_size));

    let wolf_fn_va = find_wolf_namespace_fn(pe, text_slice, text_va)?;
    let mut decoder = Decoder::with_ip(64, text_slice, text_va, DecoderOptions::NONE);
    let mut wolf_countdown = 0u32;
    let mut lua_regs = Vec::<(String, u64, u64)>::new();

    while decoder.can_decode() {
        let insn = decoder.decode();
        if insn.mnemonic() == Mnemonic::Call
            && insn.op0_kind() == OpKind::NearBranch64
            && insn.near_branch64() == wolf_fn_va
        {
            wolf_countdown = 6;
            continue;
        }
        if wolf_countdown > 0 {
            wolf_countdown -= 1;
            if insn.mnemonic() == Mnemonic::Lea
                && insn.op0_register() == Register::R8
                && insn.is_ip_rel_memory_operand()
            {
                if let Some(s) = pe.cstr_at_va(insn.ip_rel_memory_address())
                    && let Some((fstart, _)) = pe.find_fn(insn.ip())
                {
                    lua_regs.push((s.to_owned(), fstart, insn.ip()));
                }
                wolf_countdown = 0;
            }
        }
    }

    let mut out = BTreeMap::new();
    for (class_name, fstart, lea_ip) in lua_regs {
        if class_name.starts_with('F') {
            let fields =
                extract_lua_struct_fields(pe, fstart, lea_ip, wolf_fn_va, rdata_va, rdata_end_va);
            let stem = normalize_stem(&class_name);
            out.insert(stem, (class_name, fields));
        }
    }
    Ok(out)
}

fn find_wolf_namespace_fn(
    pe: &PeImage,
    text_slice: &[u8],
    text_va: u64,
) -> Result<u64, XtaskError> {
    let mut decoder = Decoder::with_ip(64, text_slice, text_va, DecoderOptions::NONE);
    let mut counts = BTreeMap::<u64, u32>::new();
    let mut last_call_va = None;
    let mut countdown = 0u32;
    while decoder.can_decode() {
        let ins = decoder.decode();
        if ins.mnemonic() == Mnemonic::Call && ins.op0_kind() == OpKind::NearBranch64 {
            last_call_va = Some(ins.near_branch64());
            countdown = 6;
            continue;
        }
        if countdown > 0 {
            countdown -= 1;
            if ins.mnemonic() == Mnemonic::Lea
                && ins.op0_register() == Register::R8
                && ins.is_ip_rel_memory_operand()
                && let Some(s) = pe.cstr_at_va(ins.ip_rel_memory_address())
                && s.starts_with('F')
                && s.ends_with("Info")
                && let Some(target_va) = last_call_va
            {
                *counts.entry(target_va).or_insert(0) += 1;
                countdown = 0;
            }
        }
    }
    counts
        .into_iter()
        .max_by_key(|&(_, c)| c)
        .map(|(va, _)| va)
        .ok_or_else(|| XtaskError::PeExtract {
            context: "find_wolf_namespace_fn",
            detail: "failed to locate LuaBridge 'wolf' namespace helper".into(),
        })
}

fn extract_translate_str_offset(pe: &PeImage, vtable_va: u64) -> Option<u32> {
    let off = pe.va_to_off(vtable_va)?;
    let fn2_va = pe.read_u64_at(off + 16)?;
    let (fstart, _) = pe.find_fn(fn2_va)?;
    if fstart != fn2_va {
        return None;
    }
    let insns = pe.fn_insns(fn2_va);
    if insns.is_empty() {
        return None;
    }
    for ins in insns {
        if ins.mnemonic() == Mnemonic::Add && ins.op0_register() == Register::RDX {
            let imm = if ins.op1_kind() == OpKind::Immediate8to64 {
                ins.immediate8to64()
            } else {
                ins.immediate64().cast_signed()
            };
            let Ok(v) = u32::try_from(imm) else {
                return None;
            };
            return Some(v);
        }
        if ins.mnemonic() == Mnemonic::Sub && ins.op0_register() == Register::RDX {
            let imm = if ins.op1_kind() == OpKind::Immediate8to64 {
                ins.immediate8to64()
            } else {
                ins.immediate64().cast_signed()
            };
            let Ok(v) = u32::try_from(-imm) else {
                return None;
            };
            return Some(v);
        }
    }
    Some(0)
}

#[derive(Default)]
struct LuaFieldScanState {
    last_rdx_str: Option<String>,
    r9b_is_one: bool,
    r8_imm_offset: Option<u32>,
    last_prop_offset: Option<u32>,
    last_vtable_va: Option<u64>,
    after_userdata_alloc: bool,
    edx_was_four: bool,
}

impl LuaFieldScanState {
    fn update_pre_call(
        &mut self,
        pe: &PeImage,
        ins: &iced_x86::Instruction,
        rdata_va: u64,
        rdata_end_va: u64,
    ) {
        if self.after_userdata_alloc
            && ins.mnemonic() == Mnemonic::Mov
            && ins.op0_kind() == OpKind::Memory
            && ins.memory_base() == Register::RAX
            && ins.memory_displacement64() == 0
        {
            if ins.op1_kind() == OpKind::Immediate32 {
                self.last_prop_offset = Some(ins.immediate32());
            } else if ins.op1_kind() == OpKind::Register && ins.op1_register().size() == 4 {
                self.last_prop_offset = Some(0);
            }
            self.after_userdata_alloc = false;
        }
        if ins.mnemonic() == Mnemonic::Mov
            && ins.op0_register() == Register::EDX
            && ins.op1_kind() == OpKind::Immediate32
            && ins.immediate32() == 4
        {
            self.edx_was_four = true;
        }
        if ins.mnemonic() == Mnemonic::Mov
            && ins.op0_register() == Register::R9L
            && ins.op1_kind() == OpKind::Immediate8
            && ins.immediate8() == 1
        {
            self.r9b_is_one = true;
            self.r8_imm_offset = None;
        }
        if ins.mnemonic() == Mnemonic::Mov
            && ins.op0_register() == Register::R8D
            && ins.op1_kind() == OpKind::Immediate32
        {
            self.r8_imm_offset = Some(ins.immediate32());
        } else if ins.mnemonic() == Mnemonic::Xor
            && ins.op0_register() == Register::R8D
            && ins.op1_register() == Register::R8D
        {
            self.r8_imm_offset = Some(0);
        } else if ins.mnemonic() == Mnemonic::Lea && ins.op0_register() == Register::R8 {
            self.r8_imm_offset = None;
        }
        if ins.mnemonic() == Mnemonic::Lea && ins.is_ip_rel_memory_operand() {
            let va = ins.ip_rel_memory_address();
            if va >= rdata_va && va < rdata_end_va {
                if ins.op0_register() == Register::RAX
                    && extract_translate_str_offset(pe, va).is_some()
                {
                    self.last_vtable_va = Some(va);
                } else if ins.op0_register() == Register::RDX
                    && let Some(s) = pe.cstr_at_va(va)
                    && !s.is_empty()
                {
                    self.last_rdx_str = Some(s.to_owned());
                }
            }
        }
    }

    fn finish_call(&mut self, pe: &PeImage, fields: &mut Vec<LuaFieldInfo>) {
        if self.edx_was_four {
            self.after_userdata_alloc = true;
            self.edx_was_four = false;
        }
        let Some(name) = self.last_rdx_str.take() else {
            return;
        };
        if self.r9b_is_one && self.r8_imm_offset.is_some() {
            if let Some(off) = self.r8_imm_offset.take()
                && !fields.iter().any(|f| f.offset == off)
            {
                fields.push(LuaFieldInfo {
                    offset: off,
                    name,
                    is_translate_str: false,
                });
            }
            self.last_prop_offset = None;
            self.last_vtable_va = None;
        } else if let Some(off) = self.last_prop_offset.take() {
            if !fields.iter().any(|f| f.offset == off) {
                fields.push(LuaFieldInfo {
                    offset: off,
                    name,
                    is_translate_str: false,
                });
            }
            self.last_vtable_va = None;
        } else if let Some(vt) = self.last_vtable_va.take()
            && let Some(off) = extract_translate_str_offset(pe, vt)
            && !fields.iter().any(|f| f.offset == off)
        {
            fields.push(LuaFieldInfo {
                offset: off,
                name,
                is_translate_str: true,
            });
        }
        self.r9b_is_one = false;
    }
}

fn extract_lua_struct_fields(
    pe: &PeImage,
    fva: u64,
    lea_ip: u64,
    wolf_fn_va: u64,
    rdata_va: u64,
    rdata_end_va: u64,
) -> Vec<LuaFieldInfo> {
    let all_insns = pe.fn_insns(fva);
    let Some(start_idx) = all_insns.iter().position(|i| i.ip() == lea_ip) else {
        return Vec::new();
    };
    let mut fields = Vec::<LuaFieldInfo>::new();
    let mut state = LuaFieldScanState::default();

    for ins in &all_insns[start_idx..] {
        if ins.mnemonic() == Mnemonic::Call
            && ins.op0_kind() == OpKind::NearBranch64
            && ins.near_branch64() == wolf_fn_va
        {
            break;
        }
        state.update_pre_call(pe, ins, rdata_va, rdata_end_va);
        if ins.mnemonic() == Mnemonic::Call && ins.op0_kind() == OpKind::NearBranch64 {
            state.finish_call(pe, &mut fields);
        }
    }
    fields
}

/// Resolves unique Rust `snake_case` field names for `cols` (`(WireType, Option<struct_offset>)`)
/// using the table's `LuaBridge` field registrations.
#[must_use]
pub fn resolve_column_names(
    cols: &[(WireType, Option<u32>)],
    lua_fields: Option<&[LuaFieldInfo]>,
) -> Vec<(String, WireType, bool)> {
    let mut matched_lua_offsets = BTreeSet::new();
    let mut raw_assigned: Vec<Option<(String, bool)>> = vec![None; cols.len()];

    if let Some(lfields) = lua_fields {
        for (idx, &(_, maybe_off)) in cols.iter().enumerate() {
            if let Some(col_off) = maybe_off
                && let Some(lf) = lfields.iter().find(|f| f.offset == col_off)
            {
                raw_assigned[idx] = Some((lf.name.clone(), lf.is_translate_str));
                matched_lua_offsets.insert(lf.offset);
            }
        }
        // For any column whose offset was None (e.g. first string key-mirror field in
        // FLanguageInfo / FUiSafeZoneInfo), pair in order with remaining unmatched Lua fields.
        let mut unmatched_lua: Vec<&LuaFieldInfo> = lfields
            .iter()
            .filter(|f| !matched_lua_offsets.contains(&f.offset))
            .collect();
        unmatched_lua.sort_by_key(|f| f.offset);
        let mut u_iter = unmatched_lua.into_iter();
        for (idx, &(_, maybe_off)) in cols.iter().enumerate() {
            if raw_assigned[idx].is_none()
                && maybe_off.is_none()
                && let Some(lf) = u_iter.next()
            {
                raw_assigned[idx] = Some((lf.name.clone(), lf.is_translate_str));
            }
        }
    }

    let mut used_names = BTreeSet::new();
    let mut out = Vec::with_capacity(cols.len());
    for (idx, (kind, _)) in cols.iter().enumerate() {
        let idx_u32 = crate::bounded_len_u32(idx);
        let (base_name, is_trans) = match &raw_assigned[idx] {
            Some((raw_n, is_t)) => (normalize_field_name(raw_n, idx_u32), *is_t),
            None => (format!("col_{idx_u32}"), false),
        };
        let mut final_name = base_name.clone();
        let mut suffix = 2u32;
        while !used_names.insert(final_name.clone()) {
            final_name = format!("{base_name}_{suffix}");
            suffix += 1;
        }
        out.push((final_name, kind.clone(), is_trans));
    }
    out
}

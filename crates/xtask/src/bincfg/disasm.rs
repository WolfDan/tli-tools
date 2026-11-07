//! Address-free structural classifier for `ReadBin`, `ReadTreeNode` (`AffixOp3`), and `ReadActionData`.

use crate::bincfg::domains::{MapKeyWire, SubOp, WireType};
use crate::bincfg::pe::{ExtractedKeySchema, PeImage, RawOp, extract_ops, normalize_stem};
use crate::error::XtaskError;
use crate::u32_to_usize;
use iced_x86::{Decoder, DecoderOptions, FlowControl, Instruction, Mnemonic, OpKind, Register};
use std::collections::BTreeMap;

/// Extracted table schema from `torchlight_infinite.exe`.
#[derive(Debug, Clone)]
pub struct ExtractedTableSchema {
    pub stem: String,
    pub cpp_name: String,
    pub is_map_table: bool,
    pub key_schema: ExtractedKeySchema,
    pub cols: Vec<(WireType, Option<u32>)>,
}

/// Discovered framework function virtual addresses (found dynamically from `.rdata` + call graph).
#[derive(Debug, Clone, Copy)]
pub struct FrameworkFns {
    pub open_chunk_bin: u64,
    pub close_chunk_bin: u64,
    pub log_error: u64,
    pub alloc_row: u64,
    pub open_map_seek: u64,
    pub read_str: u64,
    pub read_tree_node: u64,
    pub read_action_data: u64,
}

/// Complete binary extraction result from `torchlight_infinite.exe`.
pub struct BinarySchemas {
    pub framework: FrameworkFns,
    pub op3_table: Vec<Vec<SubOp>>,
    pub action_table: Vec<Vec<WireType>>,
    pub chunk_schemas: BTreeMap<String, ExtractedTableSchema>,
    pub map_schemas: BTreeMap<String, ExtractedTableSchema>,
}

#[derive(Debug, Clone, Copy)]
struct BaseFrameworkFns {
    open_chunk_bin: u64,
    close_chunk_bin: u64,
    log_error: u64,
    alloc_row: u64,
    open_map_seek: u64,
    read_str: u64,
}

#[inline(always)]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "x86_64 32-bit displacement / immediate low-word extraction"
)]
const fn u32_low_bits(x: u64) -> u32 {
    x as u32
}

fn unique_sorted_fns(pe: &PeImage, ips: &[u64]) -> Vec<u64> {
    let mut fns: Vec<u64> = ips
        .iter()
        .filter_map(|&ip| pe.find_fn(ip).map(|x| x.0))
        .collect();
    fns.sort_unstable();
    fns.dedup();
    fns
}

/// Scans `.text` for RIP-relative references to `"/Public/Config/*Info.cpp"` strings in `.rdata`.
pub fn collect_cpp_refs(pe: &PeImage) -> Result<BTreeMap<String, Vec<u64>>, XtaskError> {
    let (_, text_slice, text_va) =
        pe.section_by_name(b".text\0\0\0")
            .ok_or_else(|| XtaskError::PeExtract {
                context: ".text",
                detail: "missing .text section".into(),
            })?;
    let mut decoder = Decoder::with_ip(64, text_slice, text_va, DecoderOptions::NONE);
    let mut cpp_refs = BTreeMap::<String, Vec<u64>>::new();
    while decoder.can_decode() {
        let insn = decoder.decode();
        if insn.is_ip_rel_memory_operand()
            && let Some(s) = pe.cstr_at_va(insn.ip_rel_memory_address())
            && s.contains("/Public/Config/")
            && s.ends_with(".cpp")
            && let Some((_, fname)) = s.rsplit_once('/')
        {
            cpp_refs
                .entry(fname.to_owned())
                .or_default()
                .push(insn.ip());
        }
    }
    if cpp_refs.is_empty() {
        return Err(XtaskError::PeExtract {
            context: "collect_cpp_refs",
            detail: "no /Public/Config/*.cpp references found".into(),
        });
    }
    Ok(cpp_refs)
}

fn direct_calls_in_fn(insns: &[Instruction]) -> Vec<u64> {
    insns
        .iter()
        .filter(|i| i.mnemonic() == Mnemonic::Call && i.op0_kind() == OpKind::NearBranch64)
        .map(Instruction::near_branch64)
        .collect()
}

fn has_direct_vtable_read(insns: &[Instruction]) -> bool {
    let mut has_r8d_imm = false;
    let mut has_disp_40 = false;
    for ins in insns {
        if ins.mnemonic() == Mnemonic::Mov
            && ins.op0_kind() == OpKind::Register
            && ins.op0_register() == Register::R8D
            && ins.op1_kind() == OpKind::Immediate32
        {
            let imm = ins.immediate32();
            if matches!(imm, 1 | 4 | 8 | 16 | 24) {
                has_r8d_imm = true;
            }
        }
        if (ins.op0_kind() == OpKind::Memory && ins.memory_displacement64() == 0x40)
            || (ins.mnemonic() == Mnemonic::Mov
                && ins.op1_kind() == OpKind::Memory
                && ins.memory_displacement64() == 0x40)
        {
            has_disp_40 = true;
        }
    }
    has_r8d_imm && has_disp_40
}

fn has_stream_read(pe: &PeImage, va: u64, cache: &mut BTreeMap<u64, bool>, depth: u8) -> bool {
    if let Some(&cached) = cache.get(&va) {
        return cached;
    }
    if depth > 3 {
        return false;
    }
    cache.insert(va, false);
    let insns = pe.fn_insns(va);
    if has_direct_vtable_read(&insns) {
        cache.insert(va, true);
        return true;
    }
    if insns.len() == 1 {
        for op in extract_ops(&insns) {
            if let RawOp::Call(c) = op
                && has_stream_read(pe, c, cache, depth + 1)
            {
                cache.insert(va, true);
                return true;
            }
        }
    } else {
        for ins in &insns {
            if ins.mnemonic() == Mnemonic::Call
                && ins.op0_kind() == OpKind::NearBranch64
                && has_stream_read(pe, ins.near_branch64(), cache, depth + 1)
            {
                cache.insert(va, true);
                return true;
            }
        }
    }
    false
}

fn is_vec_dim_fn(insns: &[Instruction], stream_ops: &[(usize, RawOp)]) -> bool {
    if stream_ops.len() != 4 {
        return false;
    }
    let is_u8_f64x3 = stream_ops[0].1 == RawOp::VtableRead(1)
        && stream_ops[1].1 == RawOp::VtableRead(8)
        && stream_ops[2].1 == RawOp::VtableRead(8)
        && stream_ops[3].1 == RawOp::VtableRead(8);
    let has_tag_branch = insns[stream_ops[0].0..stream_ops[3].0]
        .iter()
        .any(|i| i.flow_control() == FlowControl::ConditionalBranch);
    is_u8_f64x3 && has_tag_branch
}

fn is_action_data_fn(insns: &[Instruction]) -> bool {
    insns.iter().any(|ins| {
        (ins.mnemonic() == Mnemonic::Sub
            && ins.op1_kind() == OpKind::Immediate32
            && ins.immediate32() == 1000)
            || (ins.mnemonic() == Mnemonic::Add
                && ins.op1_kind() == OpKind::Immediate32
                && ins.immediate32() == (-1000i32).cast_unsigned())
    })
}

fn is_row_alloc_site(insns: &[Instruction], idx: usize) -> bool {
    let ins = &insns[idx];
    if ins.mnemonic() != Mnemonic::Call || ins.op0_kind() != OpKind::NearBranch64 {
        return false;
    }
    let next_slice = &insns[idx + 1..(idx + 7).min(insns.len())];
    let has_je = next_slice.iter().any(|n| n.mnemonic() == Mnemonic::Je);
    let has_mov_rax = next_slice.iter().any(|n| {
        n.mnemonic() == Mnemonic::Mov
            && n.op0_kind() == OpKind::Register
            && n.op0_register() != Register::RCX
            && n.op1_kind() == OpKind::Register
            && n.op1_register() == Register::RAX
    });
    has_je && has_mov_rax
}

fn match_chunk_triple_in_fn(insns: &[Instruction], alloc_row: u64) -> Option<(u64, u64, u64)> {
    let (log_pos, log_call, after_log_ip) = insns.iter().enumerate().find_map(|(idx, ins)| {
        if ins.mnemonic() == Mnemonic::Call && ins.op0_kind() == OpKind::NearBranch64 {
            let tail = &insns[idx + 1..(idx + 10).min(insns.len())];
            if let Some(jbe) = tail.iter().find(|n| n.mnemonic() == Mnemonic::Jbe) {
                return Some((idx, ins.near_branch64(), jbe.near_branch64()));
            }
        }
        None
    })?;
    let body_start = insns.iter().position(|i| i.ip() == after_log_ip)?;
    let apos = (body_start..insns.len())
        .find(|&idx| is_row_alloc_not_log(insns, idx, alloc_row, log_call))?;
    let pre_ops = extract_ops(&insns[body_start..apos]);
    let vtable4_count = pre_ops
        .iter()
        .filter(|&&op| op == RawOp::VtableRead(4))
        .count();
    if vtable4_count != 3 || extract_ops(insns).contains(&RawOp::VtableRead(2)) {
        return None;
    }
    let last_back_jmp = (apos + 1..insns.len()).rfind(|&idx| {
        let i = &insns[idx];
        matches!(i.mnemonic(), Mnemonic::Jne | Mnemonic::Jb | Mnemonic::Jl)
            && matches!(
                i.op0_kind(),
                OpKind::NearBranch64 | OpKind::NearBranch32 | OpKind::NearBranch16
            )
            && i.near_branch_target() >= after_log_ip
            && i.near_branch_target() < insns[apos].ip()
    })?;
    let open_fn = direct_calls_in_fn(&insns[..log_pos])
        .into_iter()
        .rfind(|&c| c != alloc_row)?;
    let close_fn = direct_calls_in_fn(&insns[last_back_jmp + 1..])
        .into_iter()
        .next()?;
    Some((open_fn, log_call, close_fn))
}

fn discover_chunk_triple(
    pe: &PeImage,
    cpp_refs: &BTreeMap<String, Vec<u64>>,
    alloc_row: u64,
) -> Result<(u64, u64, u64), XtaskError> {
    let mut triple_counts = BTreeMap::<(u64, u64, u64), u32>::new();
    for ips in cpp_refs.values() {
        for fva in unique_sorted_fns(pe, ips) {
            if let Some(triple) = match_chunk_triple_in_fn(&pe.fn_insns(fva), alloc_row) {
                *triple_counts.entry(triple).or_insert(0) += 1;
            }
        }
    }
    let (&(open_chunk_bin, log_error, close_chunk_bin), &best_count) = triple_counts
        .iter()
        .max_by_key(|&(_, &c)| c)
        .ok_or_else(|| XtaskError::PeExtract {
            context: "discover_base_framework",
            detail: "failed to locate OpenChunkBin/LogError/CloseChunkBin".into(),
        })?;
    if best_count < 100 {
        return Err(XtaskError::PeExtract {
            context: "discover_base_framework",
            detail: "OpenChunkBin triple frequency too low".into(),
        });
    }
    Ok((open_chunk_bin, log_error, close_chunk_bin))
}

fn discover_alloc_row(
    pe: &PeImage,
    cpp_refs: &BTreeMap<String, Vec<u64>>,
) -> Result<u64, XtaskError> {
    let mut alloc_counts = BTreeMap::<u64, u32>::new();
    for ips in cpp_refs.values() {
        for fva in unique_sorted_fns(pe, ips) {
            let insns = pe.fn_insns(fva);
            for idx in 0..insns.len() {
                if is_row_alloc_site(&insns, idx) {
                    *alloc_counts.entry(insns[idx].near_branch64()).or_insert(0) += 1;
                }
            }
        }
    }
    let (&alloc_row, _) = alloc_counts
        .iter()
        .max_by_key(|&(_, &c)| c)
        .ok_or_else(|| XtaskError::PeExtract {
            context: "discover_base_framework",
            detail: "failed to locate AllocRow".into(),
        })?;
    Ok(alloc_row)
}

fn is_row_alloc_not_log(insns: &[Instruction], idx: usize, alloc_row: u64, log_error: u64) -> bool {
    let ins = &insns[idx];
    if ins.mnemonic() != Mnemonic::Call
        || ins.op0_kind() != OpKind::NearBranch64
        || ins.near_branch64() != alloc_row
    {
        return false;
    }
    let next_slice = &insns[idx + 1..(idx + 12).min(insns.len())];
    !next_slice.iter().any(|n| {
        n.mnemonic() == Mnemonic::Call
            && n.op0_kind() == OpKind::NearBranch64
            && n.near_branch64() == log_error
    })
}

fn count_map_seek_in_fn(
    pe: &PeImage,
    insns: &[Instruction],
    alloc_row: u64,
    log_error: u64,
    map_seek_counts: &mut BTreeMap<u64, u32>,
) {
    for (seek_pos, ins) in insns.iter().enumerate() {
        if ins.mnemonic() == Mnemonic::Call && ins.op0_kind() == OpKind::NearBranch64 {
            let cand = ins.near_branch64();
            let after_calls = direct_calls_in_fn(&insns[seek_pos + 1..]);
            if cand != alloc_row
                && cand != log_error
                && after_calls.contains(&alloc_row)
                && after_calls.contains(&log_error)
            {
                let pre_ops = extract_ops(&insns[..seek_pos]);
                let is_seek = match pre_ops.get(10) {
                    Some(&RawOp::VtableRead(4)) => true,
                    Some(&RawOp::Call(iva)) => idx_reader_depth(pe, iva, 0).is_some(),
                    None | Some(&RawOp::VtableRead(_)) => false,
                };
                if is_seek {
                    *map_seek_counts.entry(cand).or_insert(0) += 1;
                }
            }
        }
    }
}

fn discover_str_and_map_seek(
    pe: &PeImage,
    cpp_refs: &BTreeMap<String, Vec<u64>>,
    open_chunk_bin: u64,
    close_chunk_bin: u64,
    log_error: u64,
    alloc_row: u64,
) -> Result<(u64, u64), XtaskError> {
    let mut str_key_counts = BTreeMap::<u64, u32>::new();
    let mut map_seek_counts = BTreeMap::<u64, u32>::new();
    for ips in cpp_refs.values() {
        for fva in unique_sorted_fns(pe, ips) {
            let insns = pe.fn_insns(fva);
            let calls = direct_calls_in_fn(&insns);
            if calls.contains(&open_chunk_bin)
                && calls.contains(&close_chunk_bin)
                && let Some(log_pos) = insns.iter().position(|i| {
                    i.mnemonic() == Mnemonic::Call
                        && i.op0_kind() == OpKind::NearBranch64
                        && i.near_branch64() == log_error
                })
                && let Some(jbe) = insns[log_pos + 1..(log_pos + 10).min(insns.len())]
                    .iter()
                    .find(|i| i.mnemonic() == Mnemonic::Jbe)
                && let Some(body_start) = insns.iter().position(|i| i.ip() == jbe.near_branch64())
                && let Some(close_pos) = insns.iter().position(|i| {
                    i.mnemonic() == Mnemonic::Call
                        && i.op0_kind() == OpKind::NearBranch64
                        && i.near_branch64() == close_chunk_bin
                })
                && body_start < close_pos
            {
                let mut body_insns = insns[body_start..close_pos].to_vec();
                if let [single_call] = direct_calls_in_fn(&body_insns).as_slice() {
                    body_insns = pe.fn_insns(*single_call);
                }
                if let Some(apos) = (0..body_insns.len())
                    .find(|&idx| is_row_alloc_not_log(&body_insns, idx, alloc_row, log_error))
                {
                    for op in extract_ops(&body_insns[..apos]) {
                        if let RawOp::Call(c) = op
                            && c != open_chunk_bin
                            && c != log_error
                            && c != alloc_row
                            && has_direct_vtable_read(&pe.fn_insns(c))
                        {
                            *str_key_counts.entry(c).or_insert(0) += 1;
                        }
                    }
                }
            } else if calls.contains(&alloc_row) && calls.contains(&log_error) {
                count_map_seek_in_fn(pe, &insns, alloc_row, log_error, &mut map_seek_counts);
            }
        }
    }
    let (&read_str, _) = str_key_counts
        .iter()
        .max_by_key(|&(_, &c)| c)
        .ok_or_else(|| XtaskError::PeExtract {
            context: "discover_base_framework",
            detail: "failed to locate ReadStr".into(),
        })?;
    let (&open_map_seek, _) = map_seek_counts
        .iter()
        .filter(|&(&c, _)| c != read_str)
        .max_by_key(|&(_, &cnt)| cnt)
        .ok_or_else(|| XtaskError::PeExtract {
            context: "discover_base_framework",
            detail: "failed to locate OpenMapSeek".into(),
        })?;
    Ok((open_map_seek, read_str))
}

/// Discovers `OpenChunkBin`, `CloseChunkBin`, `LogError`, `AllocRow`, `OpenMapSeek`, and `ReadStr`
/// from the call graph of `*Info.cpp` manager functions without any hardcoded addresses.
fn discover_base_framework(
    pe: &PeImage,
    cpp_refs: &BTreeMap<String, Vec<u64>>,
) -> Result<BaseFrameworkFns, XtaskError> {
    let alloc_row = discover_alloc_row(pe, cpp_refs)?;
    let (open_chunk_bin, log_error, close_chunk_bin) =
        discover_chunk_triple(pe, cpp_refs, alloc_row)?;
    let (open_map_seek, read_str) = discover_str_and_map_seek(
        pe,
        cpp_refs,
        open_chunk_bin,
        close_chunk_bin,
        log_error,
        alloc_row,
    )?;
    Ok(BaseFrameworkFns {
        open_chunk_bin,
        close_chunk_bin,
        log_error,
        alloc_row,
        open_map_seek,
        read_str,
    })
}

/// Switch jump-table metadata extracted from a function (`max_index`, `jt_va`, `default_ip`).
#[derive(Debug, Clone, Copy)]
pub struct SwitchTableInfo {
    pub max_index: u32,
    pub jt_va: u64,
    pub default_ip: u64,
}

/// Finds a switch jump table inside `fn_va` whose `cmp reg, max_index` is `>= min_cases`.
#[must_use]
pub fn find_switch_table(pe: &PeImage, fn_va: u64, min_cases: u32) -> Option<SwitchTableInfo> {
    let insns = pe.fn_insns(fn_va);
    for (idx, ins) in insns.iter().enumerate() {
        if ins.mnemonic() != Mnemonic::Cmp {
            continue;
        }
        let max_idx = if ins.op1_kind() == OpKind::Immediate8 {
            u32::from(ins.immediate8())
        } else if ins.op1_kind() == OpKind::Immediate8to32 {
            ins.immediate8to32().cast_unsigned()
        } else if ins.op1_kind() == OpKind::Immediate8to64 {
            u32_low_bits(ins.immediate8to64().cast_unsigned())
        } else if ins.op1_kind() == OpKind::Immediate32 {
            ins.immediate32()
        } else {
            continue;
        };
        if max_idx < min_cases || max_idx > 1024 {
            continue;
        }
        let window = &insns[idx + 1..(idx + 12).min(insns.len())];
        let Some(ja_ins) = window.iter().find(|i| i.mnemonic() == Mnemonic::Ja) else {
            continue;
        };
        let default_ip = ja_ins.near_branch_target();
        for m in window {
            if m.op1_kind() == OpKind::Memory && m.memory_index_scale() == 4 {
                let disp = m.memory_displacement64();
                let jt_va = if disp < pe.image_base() {
                    pe.image_base() + disp
                } else {
                    disp
                };
                if pe.va_to_off(jt_va).is_some() {
                    return Some(SwitchTableInfo {
                        max_index: max_idx,
                        jt_va,
                        default_ip,
                    });
                }
            }
        }
    }
    None
}

fn is_affix_entry_fn(pe: &PeImage, stream_ops: &[(usize, RawOp)]) -> Option<(u64, u64)> {
    if stream_ops.len() != 3 || stream_ops[0].1 != RawOp::VtableRead(4) {
        return None;
    }
    let RawOp::Call(arr_formula_va) = stream_ops[2].1 else {
        return None;
    };
    for op1 in extract_ops(&pe.fn_insns(arr_formula_va)) {
        if let RawOp::Call(tree_nodes_va) = op1 {
            for op2 in extract_ops(&pe.fn_insns(tree_nodes_va)) {
                if let RawOp::Call(single_node_va) = op2 {
                    for op3 in extract_ops(&pe.fn_insns(single_node_va)) {
                        if let RawOp::Call(op3_switch_va) = op3
                            && find_switch_table(pe, op3_switch_va, 64).is_some()
                        {
                            return Some((tree_nodes_va, op3_switch_va));
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_indexed_ops(insns: &[Instruction]) -> Vec<(usize, RawOp)> {
    let Some(first_ip) = insns.first().map(Instruction::ip) else {
        return Vec::new();
    };
    let last_ip = match insns.last() {
        Some(ins) => ins.ip(),
        None => first_ip,
    };
    let mut ops = Vec::new();
    let mut last_r8d = 0u32;
    for (idx, ins) in insns.iter().enumerate() {
        if ins.mnemonic() == Mnemonic::Mov
            && ins.op0_kind() == OpKind::Register
            && ins.op0_register() == Register::R8D
            && ins.op1_kind() == OpKind::Immediate32
        {
            last_r8d = ins.immediate32();
        }
        if ins.mnemonic() == Mnemonic::Call
            || (!ins.is_call_near() && ins.flow_control() == FlowControl::UnconditionalBranch)
        {
            if (ins.op0_kind() == OpKind::Memory && ins.memory_displacement64() == 0x40)
                || (ins.mnemonic() == Mnemonic::Call && ins.op0_kind() == OpKind::Register)
            {
                ops.push((idx, RawOp::VtableRead(last_r8d)));
            } else if ins.op0_kind() == OpKind::NearBranch64
                && (ins.mnemonic() == Mnemonic::Call
                    || ins.near_branch64() < first_ip
                    || ins.near_branch64() > last_ip)
            {
                ops.push((idx, RawOp::Call(ins.near_branch64())));
            }
        }
    }
    ops
}

fn find_outer_stream_loops(
    insns: &[Instruction],
    stream_ops: &[(usize, RawOp)],
    start_idx: usize,
    end_idx: usize,
) -> Vec<(usize, usize)> {
    let mut raw_loops = Vec::<(usize, usize)>::new();
    for (tail_idx, ins) in insns.iter().enumerate().take(end_idx).skip(start_idx) {
        if matches!(ins.mnemonic(), Mnemonic::Jne | Mnemonic::Jb | Mnemonic::Jl)
            && matches!(
                ins.op0_kind(),
                OpKind::NearBranch64 | OpKind::NearBranch32 | OpKind::NearBranch16
            )
        {
            let target_ip = ins.near_branch_target();
            if target_ip < ins.ip()
                && let Some(rel_head) = insns[start_idx..=tail_idx]
                    .iter()
                    .position(|i| i.ip() >= target_ip)
            {
                let head_idx = start_idx + rel_head;
                let has_stream_op = stream_ops
                    .iter()
                    .any(|&(op_idx, _)| op_idx >= head_idx && op_idx <= tail_idx);
                if has_stream_op {
                    raw_loops.push((head_idx, tail_idx));
                }
            }
        }
    }
    let mut outer = Vec::new();
    for &(h, t) in &raw_loops {
        let is_nested = raw_loops
            .iter()
            .any(|&(h2, t2)| (h2, t2) != (h, t) && h2 <= h && t2 >= t);
        if !is_nested && !outer.contains(&(h, t)) {
            outer.push((h, t));
        }
    }
    outer.sort_by_key(|&(h, _)| h);
    outer
}

fn flatten_struct_fields(types: Vec<WireType>) -> Vec<WireType> {
    let mut out = Vec::with_capacity(types.len());
    for ty in types {
        if let WireType::Struct(sub) = ty {
            out.extend(sub);
        } else {
            out.push(ty);
        }
    }
    out
}

fn wire_type_to_map_key(wt: &WireType) -> Option<MapKeyWire> {
    match wt {
        WireType::I32 => Some(MapKeyWire::I32),
        WireType::Str => Some(MapKeyWire::Str),
        WireType::Bool
        | WireType::F64
        | WireType::Bytes
        | WireType::VecDim
        | WireType::AffixEntry
        | WireType::ActionData
        | WireType::Array(_)
        | WireType::Map(..)
        | WireType::Struct(_) => None,
    }
}

fn build_loop_wire_type(
    va: u64,
    is_top_range: bool,
    seq: &mut Vec<WireType>,
    body_ops: Vec<WireType>,
) -> Result<(), XtaskError> {
    let flat_body = flatten_struct_fields(body_ops);
    if is_top_range
        && seq.as_slice() == [WireType::I32, WireType::I32]
        && flat_body.len() == 2
        && let Some(kw) = wire_type_to_map_key(&flat_body[0])
    {
        seq.clear();
        let val_ty = flat_body
            .into_iter()
            .nth(1)
            .ok_or_else(|| XtaskError::PeExtract {
                context: "build_loop_wire_type",
                detail: "missing map value type".into(),
            })?;
        seq.push(WireType::Map(kw, Box::new(val_ty)));
        return Ok(());
    }
    if seq.last() != Some(&WireType::I32) {
        return Err(XtaskError::PeExtract {
            context: "build_loop_wire_type",
            detail: format!("loop at 0x{va:x} missing preceding I32 count (seq={seq:?})").into(),
        });
    }
    seq.pop();
    let elem_ty = if flat_body.len() == 1 {
        flat_body
            .into_iter()
            .next()
            .ok_or_else(|| XtaskError::PeExtract {
                context: "build_loop_wire_type",
                detail: "empty loop body".into(),
            })?
    } else {
        WireType::Struct(flat_body)
    };
    seq.push(WireType::Array(Box::new(elem_ty)));
    Ok(())
}

/// Stateful structural classifier that caches helper classifications across a `PeImage`.
pub struct StructuralClassifier {
    pub read_str_va: u64,
    pub read_tree_nodes_va: Option<u64>,
    pub read_affix_op3_va: Option<u64>,
    pub read_action_data_va: Option<u64>,
    stream_cache: BTreeMap<u64, bool>,
    kind_cache: BTreeMap<u64, WireType>,
}

impl StructuralClassifier {
    #[must_use]
    pub fn new(read_str_va: u64) -> Self {
        let mut kind_cache = BTreeMap::new();
        kind_cache.insert(read_str_va, WireType::Str);
        let mut stream_cache = BTreeMap::new();
        stream_cache.insert(read_str_va, true);
        Self {
            read_str_va,
            read_tree_nodes_va: None,
            read_affix_op3_va: None,
            read_action_data_va: None,
            stream_cache,
            kind_cache,
        }
    }

    pub fn is_stream_fn(&mut self, pe: &PeImage, va: u64) -> bool {
        has_stream_read(pe, va, &mut self.stream_cache, 0)
    }

    pub fn classify_fn(&mut self, pe: &PeImage, va: u64) -> Result<WireType, XtaskError> {
        self.classify_at_depth(pe, va, 0)
    }

    fn decode_single_op(
        &mut self,
        pe: &PeImage,
        va: u64,
        op: RawOp,
        depth: u8,
    ) -> Result<WireType, XtaskError> {
        match op {
            RawOp::VtableRead(1) => Ok(WireType::Bool),
            RawOp::VtableRead(4) => Ok(WireType::I32),
            RawOp::VtableRead(8) => Ok(WireType::F64),
            RawOp::VtableRead(other) => Err(XtaskError::PeExtract {
                context: "classify_fn",
                detail: format!("unexpected VtableRead({other}) in 0x{va:x}").into(),
            }),
            RawOp::Call(c) => self.classify_at_depth(pe, c, depth + 1),
        }
    }

    fn decode_range(
        &mut self,
        pe: &PeImage,
        va: u64,
        insns: &[Instruction],
        stream_ops: &[(usize, RawOp)],
        range: (usize, usize),
        depth: u8,
    ) -> Result<Vec<WireType>, XtaskError> {
        let (start_idx, end_idx) = range;
        let loops = find_outer_stream_loops(insns, stream_ops, start_idx, end_idx);
        let mut seq = Vec::new();
        let mut loop_cursor = 0usize;
        let mut op_cursor = 0usize;

        while op_cursor < stream_ops.len() {
            let (op_idx, op) = stream_ops[op_cursor];
            if op_idx < start_idx {
                op_cursor += 1;
                continue;
            }
            if op_idx >= end_idx {
                break;
            }
            if let Some(&(h, t)) = loops.get(loop_cursor)
                && op_idx >= h
            {
                let body_ops = self.decode_range(pe, va, insns, stream_ops, (h, t), depth + 1)?;
                build_loop_wire_type(va, start_idx == 0 && loops.len() == 1, &mut seq, body_ops)?;
                while op_cursor < stream_ops.len() && stream_ops[op_cursor].0 <= t {
                    op_cursor += 1;
                }
                loop_cursor += 1;
                continue;
            }
            let ty = self.decode_single_op(pe, va, op, depth)?;
            seq.push(ty);
            op_cursor += 1;
        }
        Ok(seq)
    }

    fn classify_at_depth(
        &mut self,
        pe: &PeImage,
        va: u64,
        depth: u8,
    ) -> Result<WireType, XtaskError> {
        if let Some(k) = self.kind_cache.get(&va) {
            return Ok(k.clone());
        }
        if depth > 6 {
            return Err(XtaskError::PeExtract {
                context: "classify_fn",
                detail: format!("call depth exceeded at 0x{va:x}").into(),
            });
        }
        let insns = pe.fn_insns(va);
        if is_action_data_fn(&insns) {
            self.read_action_data_va = Some(va);
            self.kind_cache.insert(va, WireType::ActionData);
            return Ok(WireType::ActionData);
        }
        let raw_ops = extract_indexed_ops(&insns);
        let mut stream_ops = Vec::new();
        for (idx, op) in raw_ops {
            match op {
                RawOp::VtableRead(_) => stream_ops.push((idx, op)),
                RawOp::Call(c) => {
                    if self.is_stream_fn(pe, c) {
                        stream_ops.push((idx, op));
                    }
                }
            }
        }
        if is_vec_dim_fn(&insns, &stream_ops) {
            self.kind_cache.insert(va, WireType::VecDim);
            return Ok(WireType::VecDim);
        }
        if let Some((tree_nodes_va, op3_switch_va)) = is_affix_entry_fn(pe, &stream_ops) {
            self.read_tree_nodes_va = Some(tree_nodes_va);
            self.read_affix_op3_va = Some(op3_switch_va);
            self.kind_cache.insert(va, WireType::AffixEntry);
            return Ok(WireType::AffixEntry);
        }
        let seq = self.decode_range(pe, va, &insns, &stream_ops, (0, insns.len()), depth)?;
        let kind = if seq.as_slice() == [WireType::I32, WireType::ActionData] {
            WireType::ActionData
        } else if seq.len() == 1 {
            seq.into_iter()
                .next()
                .ok_or_else(|| XtaskError::PeExtract {
                    context: "classify_fn",
                    detail: "empty single sequence".into(),
                })?
        } else {
            let flat = flatten_struct_fields(seq);
            if flat.is_empty() {
                return Err(XtaskError::PeExtract {
                    context: "classify_fn",
                    detail: format!("empty stream sequence in 0x{va:x}").into(),
                });
            }
            WireType::Struct(flat)
        };
        self.kind_cache.insert(va, kind.clone());
        Ok(kind)
    }
}

fn update_row_reg_offsets(
    ins: &Instruction,
    row_reg: Register,
    reg_offs: &mut BTreeMap<Register, u32>,
) {
    if ins.mnemonic() == Mnemonic::Lea && ins.op0_kind() == OpKind::Register {
        if let Some(&base_off) = reg_offs.get(&ins.memory_base()) {
            let disp = u32_low_bits(ins.memory_displacement64());
            reg_offs.insert(ins.op0_register(), base_off.wrapping_add(disp));
        } else if ins.op0_register() != row_reg {
            reg_offs.remove(&ins.op0_register());
        }
    } else if ins.mnemonic() == Mnemonic::Mov
        && ins.op0_kind() == OpKind::Register
        && ins.op0_register() != row_reg
    {
        if ins.op1_kind() == OpKind::Register {
            if let Some(&off) = reg_offs.get(&ins.op1_register()) {
                reg_offs.insert(ins.op0_register(), off);
            } else {
                reg_offs.remove(&ins.op0_register());
            }
        } else {
            reg_offs.remove(&ins.op0_register());
        }
    }
}

fn extract_raw_ops_with_offsets(
    insns: &[Instruction],
    row_reg: Register,
) -> Vec<(RawOp, Option<u32>)> {
    let Some(first_ip) = insns.first().map(Instruction::ip) else {
        return Vec::new();
    };
    let last_ip = match insns.last() {
        Some(ins) => ins.ip(),
        None => first_ip,
    };
    let mut raw = Vec::<(RawOp, Option<u32>)>::new();
    let mut last_r8d = 0u32;
    let mut reg_offs = BTreeMap::<Register, u32>::new();
    reg_offs.insert(row_reg, 0);
    reg_offs.insert(Register::RAX, 0);

    for (idx, ins) in insns.iter().enumerate() {
        if ins.mnemonic() == Mnemonic::Mov
            && ins.op0_kind() == OpKind::Register
            && ins.op0_register() == Register::R8D
            && ins.op1_kind() == OpKind::Immediate32
        {
            last_r8d = ins.immediate32();
        }
        update_row_reg_offsets(ins, row_reg, &mut reg_offs);

        if ins.mnemonic() == Mnemonic::Call
            || (!ins.is_call_near() && ins.flow_control() == FlowControl::UnconditionalBranch)
        {
            if (ins.op0_kind() == OpKind::Memory && ins.memory_displacement64() == 0x40)
                || (ins.mnemonic() == Mnemonic::Call && ins.op0_kind() == OpKind::Register)
            {
                let mut store_off = None;
                for m in &insns[idx + 1..(idx + 12).min(insns.len())] {
                    if m.mnemonic() == Mnemonic::Call {
                        break;
                    }
                    if matches!(m.mnemonic(), Mnemonic::Mov | Mnemonic::Movsd)
                        && m.op0_kind() == OpKind::Memory
                        && m.memory_base() == row_reg
                    {
                        store_off = Some(u32_low_bits(m.memory_displacement64()));
                        break;
                    }
                }
                let off = store_off.or_else(|| reg_offs.get(&Register::RDX).copied());
                raw.push((RawOp::VtableRead(last_r8d), off));
            } else if ins.op0_kind() == OpKind::NearBranch64
                && (ins.mnemonic() == Mnemonic::Call
                    || ins.near_branch64() < first_ip
                    || ins.near_branch64() > last_ip)
            {
                let target = ins.near_branch64();
                let rdx_off = reg_offs.get(&Register::RDX).copied();
                let rcx_off = reg_offs.get(&Register::RCX).copied();
                raw.push((RawOp::Call(target), rdx_off.or(rcx_off)));
            }
        }
    }
    raw
}

fn try_decode_inlined_map(
    pe: &PeImage,
    classifier: &mut StructuralClassifier,
    raw: &[(RawOp, Option<u32>)],
    i: &mut usize,
) -> Result<Option<(WireType, Option<u32>)>, XtaskError> {
    let idx = *i;
    if raw[idx].0 != RawOp::VtableRead(4) || idx + 2 >= raw.len() {
        return Ok(None);
    }
    let RawOp::Call(empty_va) = raw[idx + 1].0 else {
        return Ok(None);
    };
    if classifier.is_stream_fn(pe, empty_va) {
        return Ok(None);
    }
    let mut map_off = raw[idx + 1].1;
    let mut p = idx + 1;
    while p < raw.len()
        && let RawOp::Call(c) = raw[p].0
        && !classifier.is_stream_fn(pe, c)
    {
        map_off = map_off.or(raw[p].1);
        p += 1;
    }
    if p >= raw.len() || raw[p].0 != RawOp::VtableRead(4) {
        return Ok(None);
    }
    let mut cur = p + 1;
    let mut kv = Vec::with_capacity(2);
    while cur < raw.len() && kv.len() < 2 {
        match raw[cur].0 {
            RawOp::VtableRead(1) => kv.push(WireType::Bool),
            RawOp::VtableRead(4) => kv.push(WireType::I32),
            RawOp::VtableRead(8) => kv.push(WireType::F64),
            RawOp::VtableRead(other) => {
                return Err(XtaskError::PeExtract {
                    context: "decode_cols_with_offsets",
                    detail: format!("unexpected map VtableRead({other})").into(),
                });
            }
            RawOp::Call(c) if classifier.is_stream_fn(pe, c) => {
                kv.push(classifier.classify_fn(pe, c)?);
            }
            RawOp::Call(_) => map_off = map_off.or(raw[cur].1),
        }
        cur += 1;
    }
    let mut trailing_calls = 0usize;
    while cur < raw.len()
        && let RawOp::Call(c) = raw[cur].0
        && !classifier.is_stream_fn(pe, c)
    {
        map_off = map_off.or(raw[cur].1);
        trailing_calls += 1;
        cur += 1;
    }
    if kv.len() != 2 || trailing_calls == 0 {
        return Ok(None);
    }
    let mut kv_iter = kv.into_iter();
    let k_ty = kv_iter.next().ok_or_else(|| XtaskError::PeExtract {
        context: "try_decode_inlined_map",
        detail: "missing map key".into(),
    })?;
    let v_ty = kv_iter.next().ok_or_else(|| XtaskError::PeExtract {
        context: "try_decode_inlined_map",
        detail: "missing map val".into(),
    })?;
    let Some(key_wire) = wire_type_to_map_key(&k_ty) else {
        return Ok(None);
    };
    *i = cur;
    Ok(Some((WireType::Map(key_wire, Box::new(v_ty)), map_off)))
}

/// Decodes an instruction slice into `(WireType, Option<struct_offset>)` columns using `classifier`.
pub fn decode_cols_with_offsets(
    pe: &PeImage,
    classifier: &mut StructuralClassifier,
    insns: &[Instruction],
    row_reg: Register,
) -> Result<Vec<(WireType, Option<u32>)>, XtaskError> {
    let raw = extract_raw_ops_with_offsets(insns, row_reg);
    let mut out = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        if let Some(col) = try_decode_inlined_map(pe, classifier, &raw, &mut i)? {
            out.push(col);
            continue;
        }
        let kind = match raw[i].0 {
            RawOp::VtableRead(1) => WireType::Bool,
            RawOp::VtableRead(4) => WireType::I32,
            RawOp::VtableRead(8) => WireType::F64,
            RawOp::VtableRead(other) => {
                return Err(XtaskError::PeExtract {
                    context: "decode_cols_with_offsets",
                    detail: format!("unexpected VtableRead({other})").into(),
                });
            }
            RawOp::Call(va) => {
                if !classifier.is_stream_fn(pe, va) {
                    i += 1;
                    continue;
                }
                classifier.classify_fn(pe, va)?
            }
        };
        out.push((kind, raw[i].1));
        i += 1;
    }
    Ok(out)
}

fn classify_sub_op_helper(
    pe: &PeImage,
    va: u64,
    read_tree_nodes_va: u64,
    classifier: &mut StructuralClassifier,
) -> Result<SubOp, XtaskError> {
    let insns = pe.fn_insns(va);
    let raw_ops = extract_ops(&insns);
    let first_stream_op = raw_ops.iter().copied().find(|&op| match op {
        RawOp::VtableRead(_) => true,
        RawOp::Call(c) => classifier.is_stream_fn(pe, c),
    });
    if first_stream_op == Some(RawOp::VtableRead(4)) {
        return Ok(SubOp::ArrayFormulaOrI32);
    }
    let mut has_vtable_1 = false;
    let mut kinds = Vec::new();
    for op in raw_ops {
        match op {
            RawOp::VtableRead(1) => {
                has_vtable_1 = true;
                kinds.push(WireType::Bool);
            }
            RawOp::VtableRead(4) => kinds.push(WireType::I32),
            RawOp::VtableRead(8) => kinds.push(WireType::F64),
            RawOp::VtableRead(_) => {}
            RawOp::Call(c) => {
                if c != read_tree_nodes_va && classifier.is_stream_fn(pe, c) {
                    kinds.push(classifier.classify_fn(pe, c)?);
                }
            }
        }
    }
    if kinds.contains(&WireType::F64) {
        return Ok(SubOp::FormulaOrF64);
    }
    if kinds.contains(&WireType::I32) {
        return Ok(SubOp::FormulaOrI32);
    }
    if has_vtable_1 {
        return Ok(SubOp::FormulaOrU8);
    }
    if kinds.len() == 2 && kinds.iter().all(|k| *k == WireType::Bool) {
        return Ok(SubOp::FormulaOrBool);
    }
    Err(XtaskError::PeExtract {
        context: "classify_sub_op_helper",
        detail: format!("unrecognized SubOp helper at 0x{va:x}: {kinds:?}").into(),
    })
}

fn decode_op3_reader(
    pe: &PeImage,
    reader_va: u64,
    read_tree_nodes_va: u64,
    classifier: &mut StructuralClassifier,
    slot: &mut Vec<SubOp>,
) -> Result<(), XtaskError> {
    let raw_ops = extract_ops(&pe.fn_insns(reader_va));
    let mut j = 0;
    while j < raw_ops.len() {
        if let RawOp::Call(c) = raw_ops[j] {
            if !classifier.is_stream_fn(pe, c) || c == read_tree_nodes_va {
                j += 1;
                continue;
            }
            if extract_ops(&pe.fn_insns(c)).as_slice() == [RawOp::VtableRead(1)] {
                while j < raw_ops.len() && !matches!(raw_ops[j], RawOp::VtableRead(_)) {
                    j += 1;
                }
                if j < raw_ops.len() {
                    match raw_ops[j] {
                        RawOp::VtableRead(4) => slot.push(SubOp::FormulaOrI32),
                        RawOp::VtableRead(1) => slot.push(SubOp::FormulaOrU8),
                        other @ (RawOp::VtableRead(_) | RawOp::Call(_)) => {
                            return Err(XtaskError::PeExtract {
                                context: "extract_affix_op3_table",
                                detail: format!("unexpected inline literal {other:?}").into(),
                            });
                        }
                    }
                }
                j += 1;
                continue;
            }
            slot.push(classify_sub_op_helper(
                pe,
                c,
                read_tree_nodes_va,
                classifier,
            )?);
        }
        j += 1;
    }
    Ok(())
}

/// Extracts the `AffixOp3` switch table (`0..=max_op3`) from `read_affix_op3_va`.
pub fn extract_affix_op3_table(
    pe: &PeImage,
    read_tree_nodes_va: u64,
    read_affix_op3_va: u64,
    classifier: &mut StructuralClassifier,
) -> Result<Vec<Vec<SubOp>>, XtaskError> {
    let sw = find_switch_table(pe, read_affix_op3_va, 64).ok_or_else(|| XtaskError::PeExtract {
        context: "extract_affix_op3_table",
        detail: "failed to locate tag-3 switch table in ReadAffixOp3".into(),
    })?;
    let jt_off = pe
        .va_to_off(sw.jt_va)
        .ok_or_else(|| XtaskError::PeExtract {
            context: "extract_affix_op3_table",
            detail: "invalid jump table VA".into(),
        })?;
    let count = u32_to_usize(sw.max_index + 1);
    let mut table = vec![Vec::new(); count];

    for (op_idx, slot) in table.iter_mut().enumerate() {
        let rva = pe
            .read_u32_at(jt_off + op_idx * 4)
            .ok_or_else(|| XtaskError::PeExtract {
                context: "extract_affix_op3_table",
                detail: "truncated jump table".into(),
            })?;
        let target_va = pe.image_base() + u64::from(rva);
        let off = pe
            .va_to_off(target_va)
            .ok_or_else(|| XtaskError::PeExtract {
                context: "extract_affix_op3_table",
                detail: "case target VA out of bounds".into(),
            })?;
        let end_off = (off + 0x200).min(pe.bytes().len());
        let mut dec = Decoder::with_ip(
            64,
            &pe.bytes()[off..end_off],
            target_va,
            DecoderOptions::NONE,
        );
        let mut stream_calls = Vec::new();
        while dec.can_decode() {
            let ins = dec.decode();
            if ins.ip() >= sw.default_ip {
                break;
            }
            if ins.mnemonic() == Mnemonic::Call && ins.op0_kind() == OpKind::NearBranch64 {
                let c = ins.near_branch64();
                let c_insns = pe.fn_insns(c);
                let is_reader = classifier.is_stream_fn(pe, c)
                    || (c_insns.len() <= 4
                        && extract_ops(&c_insns).into_iter().any(|op| {
                            matches!(op, RawOp::Call(sub_c) if classifier.is_stream_fn(pe, sub_c))
                        }));
                if is_reader {
                    stream_calls.push(c);
                }
            }
            if (ins.mnemonic() == Mnemonic::Jmp
                && ins.op0_kind() == OpKind::NearBranch64
                && ins.near_branch64() >= sw.default_ip)
                || ins.flow_control() == FlowControl::Return
            {
                break;
            }
        }
        if let Some(&reader_va) = stream_calls.last() {
            decode_op3_reader(pe, reader_va, read_tree_nodes_va, classifier, slot)?;
        }
    }
    Ok(table)
}

/// Extracts the `ActionData` switch table (`0..=max_action_idx`) from `read_action_data_va`.
pub fn extract_action_data_table(
    pe: &PeImage,
    read_action_data_va: u64,
    classifier: &mut StructuralClassifier,
) -> Result<Vec<Vec<WireType>>, XtaskError> {
    let sw =
        find_switch_table(pe, read_action_data_va, 16).ok_or_else(|| XtaskError::PeExtract {
            context: "extract_action_data_table",
            detail: "failed to locate switch table in ReadActionData".into(),
        })?;
    let jt_off = pe
        .va_to_off(sw.jt_va)
        .ok_or_else(|| XtaskError::PeExtract {
            context: "extract_action_data_table",
            detail: "invalid jump table VA".into(),
        })?;
    let count = u32_to_usize(sw.max_index + 1);
    let mut table = vec![Vec::new(); count];

    for (idx, slot) in table.iter_mut().enumerate() {
        let rva = pe
            .read_u32_at(jt_off + idx * 4)
            .ok_or_else(|| XtaskError::PeExtract {
                context: "extract_action_data_table",
                detail: "truncated jump table".into(),
            })?;
        let mut target_va = pe.image_base() + u64::from(rva);
        if target_va == sw.default_ip {
            continue;
        }
        let mut case_insns = Vec::new();
        for _ in 0..8 {
            let off = pe
                .va_to_off(target_va)
                .ok_or_else(|| XtaskError::PeExtract {
                    context: "extract_action_data_table",
                    detail: "case target out of bounds".into(),
                })?;
            let end_off = (off + 0x200).min(pe.bytes().len());
            let mut dec = Decoder::with_ip(
                64,
                &pe.bytes()[off..end_off],
                target_va,
                DecoderOptions::NONE,
            );
            let mut next_jmp = None;
            while dec.can_decode() {
                let ins = dec.decode();
                if ins.mnemonic() == Mnemonic::Jmp && ins.op0_kind() == OpKind::NearBranch64 {
                    let t = ins.near_branch64();
                    if t.abs_diff(sw.default_ip) > 8 {
                        next_jmp = Some(t);
                    }
                    break;
                }
                case_insns.push(ins);
            }
            if let Some(t) = next_jmp {
                target_va = t;
            } else {
                break;
            }
        }
        let cols = decode_cols_with_offsets(pe, classifier, &case_insns, Register::RBX)?;
        *slot = cols.into_iter().map(|(k, _)| k).collect();
    }
    Ok(table)
}

fn idx_reader_depth(pe: &PeImage, va: u64, depth: u8) -> Option<u8> {
    if depth > 4 {
        return None;
    }
    let insns = pe.fn_insns(va);
    let ops = extract_ops(&insns);
    if ops.contains(&RawOp::VtableRead(2)) {
        return Some(1);
    }
    for op in ops {
        if let RawOp::Call(child_va) = op
            && let Some(child_depth) = idx_reader_depth(pe, child_va, depth + 1)
        {
            return Some(child_depth + 1);
        }
    }
    None
}

fn find_after_ctor_idx(
    insns: &[Instruction],
    alloc_idx: usize,
    is_map_table: bool,
) -> Result<usize, XtaskError> {
    let Some(je_pos) = insns[alloc_idx + 1..(alloc_idx + 7).min(insns.len())]
        .iter()
        .position(|ins| ins.mnemonic() == Mnemonic::Je)
    else {
        return Ok(alloc_idx + 1);
    };
    let je_ins = &insns[alloc_idx + 1 + je_pos];
    let null_target = je_ins.near_branch64();
    let null_idx = insns
        .iter()
        .position(|ins| ins.ip() == null_target)
        .ok_or_else(|| XtaskError::PeExtract {
            context: "find_after_ctor_idx",
            detail: "missing null_target IP".into(),
        })?;
    let after_ctor_ip = if insns[null_idx - 1].mnemonic() == Mnemonic::Jmp {
        insns[null_idx - 1].near_branch64()
    } else if is_map_table {
        insns[null_idx + 1].ip()
    } else {
        insns[null_idx].ip()
    };
    insns
        .iter()
        .position(|ins| ins.ip() == after_ctor_ip)
        .ok_or_else(|| XtaskError::PeExtract {
            context: "find_after_ctor_idx",
            detail: "missing after_ctor_ip".into(),
        })
}

fn find_moved_reg(insns: &[Instruction], src_reg: Register, default_reg: Register) -> Register {
    for ins in insns {
        if ins.mnemonic() == Mnemonic::Mov
            && ins.op0_kind() == OpKind::Register
            && ins.op1_kind() == OpKind::Register
            && ins.op1_register() == src_reg
            && (src_reg != Register::RAX || ins.op0_register() != Register::RCX)
        {
            return ins.op0_register();
        }
    }
    default_reg
}

fn find_row_body_slice(
    pe: &PeImage,
    map_insns: &[Instruction],
    alloc_row: u64,
    classifier: &mut StructuralClassifier,
) -> Result<(Vec<Instruction>, Register), XtaskError> {
    let idx = map_insns
        .iter()
        .position(|i| {
            i.mnemonic() == Mnemonic::Call
                && i.op0_kind() == OpKind::NearBranch64
                && i.near_branch64() == alloc_row
        })
        .ok_or_else(|| XtaskError::PeExtract {
            context: "find_row_body_slice",
            detail: "missing AllocRow in leaf loop".into(),
        })?;
    let start_idx = find_after_ctor_idx(map_insns, idx, false)?;

    let mut map_insert_idx = None;
    for j in start_idx..map_insns.len() {
        if matches!(
            map_insns[j].mnemonic(),
            Mnemonic::Jne | Mnemonic::Jb | Mnemonic::Jl
        ) && map_insns[j].op0_kind() == OpKind::NearBranch64
            && map_insns[j].near_branch64() < map_insns[idx].ip()
        {
            for k in (start_idx..j).rev() {
                if map_insns[k].mnemonic() == Mnemonic::Call
                    && map_insns[k].op0_kind() == OpKind::NearBranch64
                {
                    map_insert_idx = Some(k);
                    break;
                }
            }
            break;
        }
    }
    let end_idx = map_insert_idx.ok_or_else(|| XtaskError::PeExtract {
        context: "find_row_body_slice",
        detail: "missing leaf map insert call".into(),
    })?;
    let mut row_slice: Vec<Instruction> = map_insns[idx + 1..start_idx]
        .iter()
        .filter(|i| i.mnemonic() == Mnemonic::Lea)
        .copied()
        .chain(map_insns[start_idx..end_idx].iter().copied())
        .collect();
    let mut row_reg = find_moved_reg(
        &map_insns[idx + 1..(idx + 6).min(map_insns.len())],
        Register::RAX,
        Register::RBX,
    );

    let row_ops = extract_ops(&row_slice);
    if row_ops.len() == 1
        && let RawOp::Call(fn_va) = row_ops[0]
    {
        let rcx_is_row = map_insns[start_idx..end_idx].iter().any(|ins| {
            ins.mnemonic() == Mnemonic::Mov
                && ins.op0_register() == Register::RCX
                && matches!(ins.op1_register(), Register::RBX | Register::RAX)
        });
        if rcx_is_row && classifier.is_stream_fn(pe, fn_va) {
            let sub_insns = pe.fn_insns(fn_va);
            row_reg = find_moved_reg(
                &sub_insns[..15.min(sub_insns.len())],
                Register::RCX,
                Register::RSI,
            );
            row_slice = sub_insns;
        }
    }
    Ok((row_slice, row_reg))
}

fn trace_chunk_hierarchy(
    pe: &PeImage,
    fname: &str,
    mut map_insns: Vec<Instruction>,
    fw: &BaseFrameworkFns,
    classifier: &mut StructuralClassifier,
) -> Result<(Vec<Instruction>, Vec<RawOp>), XtaskError> {
    let mut pre_alloc_ops = Vec::new();
    for _ in 0..8 {
        if let Some(alloc_pos) = map_insns.iter().position(|i| {
            i.mnemonic() == Mnemonic::Call
                && i.op0_kind() == OpKind::NearBranch64
                && i.near_branch64() == fw.alloc_row
        }) {
            for op in extract_ops(&map_insns[..alloc_pos]) {
                if op == RawOp::VtableRead(4) || op == RawOp::Call(fw.read_str) {
                    pre_alloc_ops.push(op);
                }
            }
            break;
        }
        let calls_in_body = direct_calls_in_fn(&map_insns);
        if calls_in_body.len() == 1 {
            map_insns = pe.fn_insns(calls_in_body[0]);
            continue;
        }
        let mut inner_map_fn = None;
        let mut inner_call_pos = 0;
        for (j, ins) in map_insns.iter().enumerate() {
            if ins.mnemonic() == Mnemonic::Jne
                && ins.op0_kind() == OpKind::NearBranch64
                && ins.near_branch64() < ins.ip()
            {
                let prev_calls: Vec<(usize, u64)> = map_insns[..j]
                    .iter()
                    .enumerate()
                    .filter(|(_, i)| {
                        i.mnemonic() == Mnemonic::Call
                            && i.op0_kind() == OpKind::NearBranch64
                            && classifier.is_stream_fn(pe, i.near_branch64())
                    })
                    .map(|(k, i)| (k, i.near_branch64()))
                    .collect();
                if let Some(&(pos, target)) = prev_calls.last() {
                    inner_call_pos = pos;
                    inner_map_fn = Some(target);
                    break;
                }
            }
        }
        for op in extract_ops(&map_insns[..inner_call_pos]) {
            if op == RawOp::VtableRead(4) || op == RawOp::Call(fw.read_str) {
                pre_alloc_ops.push(op);
            }
        }
        let next_va = inner_map_fn.ok_or_else(|| XtaskError::PeExtract {
            context: "extract_chunk_schema",
            detail: format!("missing inner_map_fn in {fname}").into(),
        })?;
        map_insns = pe.fn_insns(next_va);
    }
    Ok((map_insns, pre_alloc_ops))
}

fn find_call_pos(
    insns: &[Instruction],
    target_va: u64,
    context: &'static str,
    detail: String,
) -> Result<usize, XtaskError> {
    insns
        .iter()
        .position(|i| {
            i.mnemonic() == Mnemonic::Call
                && i.op0_kind() == OpKind::NearBranch64
                && i.near_branch64() == target_va
        })
        .ok_or_else(|| XtaskError::PeExtract {
            context,
            detail: detail.into(),
        })
}

fn extract_chunk_schema(
    pe: &PeImage,
    fname: &str,
    read_bin_va: u64,
    fw: &BaseFrameworkFns,
    classifier: &mut StructuralClassifier,
) -> Result<ExtractedTableSchema, XtaskError> {
    let rb_insns = pe.fn_insns(read_bin_va);
    let log_pos = find_call_pos(
        &rb_insns,
        fw.log_error,
        "extract_chunk_schema",
        format!("missing LogError in {fname}"),
    )?;
    let jbe_ins = rb_insns[log_pos + 1..(log_pos + 10).min(rb_insns.len())]
        .iter()
        .find(|i| i.mnemonic() == Mnemonic::Jbe)
        .ok_or_else(|| XtaskError::PeExtract {
            context: "extract_chunk_schema",
            detail: format!("missing jbe after LogError in {fname}").into(),
        })?;
    let body_start = rb_insns
        .iter()
        .position(|i| i.ip() == jbe_ins.near_branch64())
        .ok_or_else(|| XtaskError::PeExtract {
            context: "extract_chunk_schema",
            detail: format!("missing body_start in {fname}").into(),
        })?;
    let close_pos = find_call_pos(
        &rb_insns,
        fw.close_chunk_bin,
        "extract_chunk_schema",
        format!("missing CloseChunkBin in {fname}"),
    )?;

    let (map_insns, pre_alloc_ops) = trace_chunk_hierarchy(
        pe,
        fname,
        rb_insns[body_start..close_pos].to_vec(),
        fw,
        classifier,
    )?;
    if pre_alloc_ops.is_empty() || pre_alloc_ops.len() % 3 != 0 {
        return Err(XtaskError::PeExtract {
            context: "extract_chunk_schema",
            detail: format!("invalid pre_alloc_ops {pre_alloc_ops:?} in {fname}").into(),
        });
    }
    let depth = u8::try_from(pre_alloc_ops.len() / 3).map_err(|_| XtaskError::PeExtract {
        context: "extract_chunk_schema",
        detail: format!("key depth overflow in {fname}").into(),
    })?;
    let key_schema = if pre_alloc_ops[2] == RawOp::Call(fw.read_str) {
        if depth == 1 {
            ExtractedKeySchema::Str1
        } else {
            ExtractedKeySchema::StrInt(depth - 1)
        }
    } else {
        ExtractedKeySchema::Int(depth)
    };
    let (row_slice, row_reg) = find_row_body_slice(pe, &map_insns, fw.alloc_row, classifier)?;
    let cols = decode_cols_with_offsets(pe, classifier, &row_slice, row_reg)?;
    Ok(ExtractedTableSchema {
        stem: normalize_stem(fname),
        cpp_name: fname.to_owned(),
        is_map_table: false,
        key_schema,
        cols,
    })
}

fn find_rcall_pos(
    insns: &[Instruction],
    target_va: u64,
    context: &'static str,
    detail: String,
) -> Result<usize, XtaskError> {
    insns
        .iter()
        .rposition(|i| {
            i.mnemonic() == Mnemonic::Call
                && i.op0_kind() == OpKind::NearBranch64
                && i.near_branch64() == target_va
        })
        .ok_or_else(|| XtaskError::PeExtract {
            context,
            detail: detail.into(),
        })
}

fn extract_map_schema(
    pe: &PeImage,
    fname: &str,
    seek_fn_va: u64,
    fw: &BaseFrameworkFns,
    classifier: &mut StructuralClassifier,
) -> Result<ExtractedTableSchema, XtaskError> {
    let f1_insns = pe.fn_insns(seek_fn_va);
    let seek_pos = find_call_pos(
        &f1_insns,
        fw.open_map_seek,
        "extract_map_schema",
        format!("missing OpenMapSeek in {fname}"),
    )?;
    let pre_seek_ops = extract_ops(&f1_insns[..seek_pos]);
    let key_depth = match pre_seek_ops.get(10) {
        Some(&RawOp::Call(idx_va)) => {
            idx_reader_depth(pe, idx_va, 0).ok_or_else(|| XtaskError::PeExtract {
                context: "extract_map_schema",
                detail: format!("failed idx_reader_depth in {fname}").into(),
            })?
        }
        Some(&RawOp::VtableRead(4)) => 3,
        other => {
            return Err(XtaskError::PeExtract {
                context: "extract_map_schema",
                detail: format!("unexpected pre_seek_ops[10] = {other:?} in {fname}").into(),
            });
        }
    };
    let after_seek = &f1_insns[seek_pos + 1..];
    let alloc_pos = find_call_pos(
        after_seek,
        fw.alloc_row,
        "extract_map_schema",
        format!("missing AllocRow after OpenMapSeek in {fname}"),
    )?;
    let start_idx = find_after_ctor_idx(after_seek, alloc_pos, true)?;
    let log_pos = find_call_pos(
        &after_seek[start_idx..],
        fw.log_error,
        "extract_map_schema",
        format!("missing LogError in {fname}"),
    )?;
    let log_alloc = find_rcall_pos(
        &after_seek[start_idx..start_idx + log_pos],
        fw.alloc_row,
        "extract_map_schema",
        format!("missing log AllocRow in {fname}"),
    )?;
    let mut row_slice = after_seek[start_idx..start_idx + log_alloc].to_vec();
    let mut row_reg = find_moved_reg(
        &after_seek[alloc_pos + 1..(alloc_pos + 6).min(after_seek.len())],
        Register::RAX,
        Register::RBX,
    );
    if let [RawOp::Call(fn_va)] = extract_ops(&row_slice).as_slice()
        && classifier.is_stream_fn(pe, *fn_va)
    {
        let sub_insns = pe.fn_insns(*fn_va);
        row_reg = find_moved_reg(
            &sub_insns[..15.min(sub_insns.len())],
            Register::RCX,
            Register::RSI,
        );
        row_slice = sub_insns;
    }
    let cols = decode_cols_with_offsets(pe, classifier, &row_slice, row_reg)?;
    Ok(ExtractedTableSchema {
        stem: normalize_stem(fname),
        cpp_name: fname.to_owned(),
        is_map_table: true,
        key_schema: ExtractedKeySchema::Int(key_depth),
        cols,
    })
}

fn is_map_seek_fn(pe: &PeImage, fva: u64, fw: &BaseFrameworkFns) -> bool {
    let insns = pe.fn_insns(fva);
    insns
        .iter()
        .position(|i| {
            i.mnemonic() == Mnemonic::Call
                && i.op0_kind() == OpKind::NearBranch64
                && i.near_branch64() == fw.open_map_seek
        })
        .is_some_and(|seek_pos| {
            let after_calls = direct_calls_in_fn(&insns[seek_pos + 1..]);
            let pre_ops = extract_ops(&insns[..seek_pos]);
            after_calls.contains(&fw.alloc_row)
                && after_calls.contains(&fw.log_error)
                && match pre_ops.get(10) {
                    Some(&RawOp::VtableRead(4)) => true,
                    Some(&RawOp::Call(iva)) => idx_reader_depth(pe, iva, 0).is_some(),
                    None | Some(&RawOp::VtableRead(_)) => false,
                }
        })
}

/// Extracts all `ChunkA0` schemas, `Map` schemas, `AffixOp3` opcodes, and `ActionData` variants
/// from `pe` using zero hardcoded virtual addresses.
pub fn extract_all_binary_schemas(pe: &PeImage) -> Result<BinarySchemas, XtaskError> {
    let cpp_refs = collect_cpp_refs(pe)?;
    let base_fw = discover_base_framework(pe, &cpp_refs)?;
    let mut classifier = StructuralClassifier::new(base_fw.read_str);

    let mut chunk_schemas = BTreeMap::new();
    let mut map_schemas = BTreeMap::new();

    for (fname, ips) in &cpp_refs {
        let fns = unique_sorted_fns(pe, ips);
        if let Some(&chunk_fva) = fns.iter().find(|&&fva| {
            let calls = direct_calls_in_fn(&pe.fn_insns(fva));
            calls.contains(&base_fw.open_chunk_bin) && calls.contains(&base_fw.close_chunk_bin)
        }) {
            let schema = extract_chunk_schema(pe, fname, chunk_fva, &base_fw, &mut classifier)?;
            chunk_schemas.insert(schema.stem.clone(), schema);
        } else if let Some(&map_fva) = fns.iter().find(|&&fva| is_map_seek_fn(pe, fva, &base_fw)) {
            let schema = extract_map_schema(pe, fname, map_fva, &base_fw, &mut classifier)?;
            map_schemas.insert(schema.stem.clone(), schema);
        }
    }

    let read_tree_nodes_va =
        classifier
            .read_tree_nodes_va
            .ok_or_else(|| XtaskError::PeExtract {
                context: "extract_all_binary_schemas",
                detail: "ReadTreeNodes VA not discovered during table scan".into(),
            })?;
    let read_affix_op3_va = classifier
        .read_affix_op3_va
        .ok_or_else(|| XtaskError::PeExtract {
            context: "extract_all_binary_schemas",
            detail: "ReadAffixOp3 VA not discovered during table scan".into(),
        })?;
    let read_action_data_va =
        classifier
            .read_action_data_va
            .ok_or_else(|| XtaskError::PeExtract {
                context: "extract_all_binary_schemas",
                detail: "ReadActionData VA not discovered during table scan".into(),
            })?;

    let op3_table =
        extract_affix_op3_table(pe, read_tree_nodes_va, read_affix_op3_va, &mut classifier)?;
    let action_table = extract_action_data_table(pe, read_action_data_va, &mut classifier)?;

    Ok(BinarySchemas {
        framework: FrameworkFns {
            open_chunk_bin: base_fw.open_chunk_bin,
            close_chunk_bin: base_fw.close_chunk_bin,
            log_error: base_fw.log_error,
            alloc_row: base_fw.alloc_row,
            open_map_seek: base_fw.open_map_seek,
            read_str: base_fw.read_str,
            read_tree_node: read_tree_nodes_va,
            read_action_data: read_action_data_va,
        },
        op3_table,
        action_table,
        chunk_schemas,
        map_schemas,
    })
}

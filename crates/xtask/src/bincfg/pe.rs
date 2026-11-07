//! Address-free PE64 loader, `.pdata` function bounds lookup, and `.bin` key schema helpers.

use crate::error::XtaskError;
use crate::u32_to_usize;
use iced_x86::{Decoder, DecoderOptions, FlowControl, Instruction, Mnemonic, OpKind, Register};
use proc_macro2::TokenStream;
use quote::quote;
use std::fs;
use std::path::Path;
use zerocopy::byteorder::little_endian::{U16, U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

const DOS_MAGIC: u16 = 0x5A4D;
const PE_MAGIC: u32 = 0x0000_4550;
const PE32_PLUS_MAGIC: u16 = 0x020B;
const EXCEPTION_DIR_INDEX: usize = 3;
const FN_DISASM_BYTES_MAX: usize = 0x10000;
const NO_PDATA_BYTES_MAX: usize = 0x200;

#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
struct RawDosHeader {
    e_magic: U16,
    _pad: [u8; 58],
    e_lfanew: U32,
}
const _: () = assert!(core::mem::size_of::<RawDosHeader>() == 64);

#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
struct RawPeHeader {
    signature: U32,
    machine: U16,
    number_of_sections: U16,
    time_date_stamp: U32,
    pointer_to_symbol_table: U32,
    number_of_symbols: U32,
    size_of_optional_header: U16,
    characteristics: U16,
}
const _: () = assert!(core::mem::size_of::<RawPeHeader>() == 24);

#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
struct RawOptHeader64Prefix {
    magic: U16,
    _pad0: [u8; 22],
    image_base: U64,
    _pad1: [u8; 80],
}
const _: () = assert!(core::mem::size_of::<RawOptHeader64Prefix>() == 112);

#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
struct RawDataDirectory {
    virtual_address: U32,
    size: U32,
}
const _: () = assert!(core::mem::size_of::<RawDataDirectory>() == 8);

#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
struct RawSectionHeader {
    name: [u8; 8],
    virtual_size: U32,
    virtual_address: U32,
    size_of_raw_data: U32,
    pointer_to_raw_data: U32,
    _reloc_and_flags: [u8; 16],
}
const _: () = assert!(core::mem::size_of::<RawSectionHeader>() == 40);

#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
struct RawRuntimeFunction {
    begin_address: U32,
    end_address: U32,
    unwind_info_address: U32,
}
const _: () = assert!(core::mem::size_of::<RawRuntimeFunction>() == 12);

/// Parsed PE64 section descriptor.
#[derive(Debug, Clone, Copy)]
pub struct PeSection {
    pub name: [u8; 8],
    pub vaddr: u32,
    pub vsize: u32,
    pub raw_ptr: u32,
    pub raw_size: u32,
}

/// Loaded PE64 executable image with `.pdata` function bounds and VA translation.
pub struct PeImage {
    bytes: Box<[u8]>,
    image_base: u64,
    sections: Box<[PeSection]>,
    pdata: Box<[(u32, u32)]>,
}

impl PeImage {
    /// Loads and validates a PE64 executable from `path`.
    pub fn load(path: &Path) -> Result<Self, XtaskError> {
        let bytes = fs::read(path)?.into_boxed_slice();
        let (dos, _) =
            RawDosHeader::ref_from_prefix(&bytes).map_err(|_| XtaskError::PeExtract {
                context: "DOS header",
                detail: "truncated DOS header".into(),
            })?;
        if dos.e_magic.get() != DOS_MAGIC {
            return Err(XtaskError::PeExtract {
                context: "DOS header",
                detail: "invalid MZ magic".into(),
            });
        }
        let pe_off = u32_to_usize(dos.e_lfanew.get());
        let pe_tail = bytes.get(pe_off..).ok_or_else(|| XtaskError::PeExtract {
            context: "PE header",
            detail: "e_lfanew out of bounds".into(),
        })?;
        let (pe_hdr, opt_tail) =
            RawPeHeader::ref_from_prefix(pe_tail).map_err(|_| XtaskError::PeExtract {
                context: "PE header",
                detail: "truncated COFF header".into(),
            })?;
        if pe_hdr.signature.get() != PE_MAGIC {
            return Err(XtaskError::PeExtract {
                context: "PE header",
                detail: "invalid PE\\0\\0 signature".into(),
            });
        }
        let (image_base, exc_dir, sections) = Self::parse_sections_and_pdata(pe_hdr, opt_tail)?;
        let mut pe = Self {
            bytes,
            image_base,
            sections,
            pdata: Box::new([]),
        };
        pe.load_pdata(exc_dir.virtual_address.get(), exc_dir.size.get())?;
        Ok(pe)
    }

    fn parse_sections_and_pdata(
        pe_hdr: &RawPeHeader,
        opt_tail: &[u8],
    ) -> Result<(u64, RawDataDirectory, Box<[PeSection]>), XtaskError> {
        let opt_size = usize::from(pe_hdr.size_of_optional_header.get());
        let (opt_bytes, sec_tail) =
            opt_tail
                .split_at_checked(opt_size)
                .ok_or_else(|| XtaskError::PeExtract {
                    context: "OptionalHeader64",
                    detail: "optional header out of bounds".into(),
                })?;
        let (opt_prefix, dir_tail) =
            RawOptHeader64Prefix::ref_from_prefix(opt_bytes).map_err(|_| {
                XtaskError::PeExtract {
                    context: "OptionalHeader64",
                    detail: "truncated OptionalHeader64".into(),
                }
            })?;
        if opt_prefix.magic.get() != PE32_PLUS_MAGIC {
            return Err(XtaskError::PeExtract {
                context: "OptionalHeader64",
                detail: "expected PE32+ (64-bit) image".into(),
            });
        }
        let (dirs, _) =
            <[RawDataDirectory]>::ref_from_prefix_with_elems(dir_tail, EXCEPTION_DIR_INDEX + 1)
                .map_err(|_| XtaskError::PeExtract {
                    context: "DataDirectories",
                    detail: "missing exception directory".into(),
                })?;
        let exc_dir = dirs[EXCEPTION_DIR_INDEX];
        let num_sec = usize::from(pe_hdr.number_of_sections.get());
        let (raw_secs, _) = <[RawSectionHeader]>::ref_from_prefix_with_elems(sec_tail, num_sec)
            .map_err(|_| XtaskError::PeExtract {
                context: "SectionTable",
                detail: "truncated section table".into(),
            })?;
        let sections: Box<[PeSection]> = raw_secs
            .iter()
            .map(|s| PeSection {
                name: s.name,
                vaddr: s.virtual_address.get(),
                vsize: s.virtual_size.get(),
                raw_ptr: s.pointer_to_raw_data.get(),
                raw_size: s.size_of_raw_data.get(),
            })
            .collect();
        Ok((opt_prefix.image_base.get(), exc_dir, sections))
    }

    fn load_pdata(&mut self, exc_rva: u32, exc_size: u32) -> Result<(), XtaskError> {
        let Some(off) = self.va_to_off(self.image_base + u64::from(exc_rva)) else {
            return Ok(());
        };
        let count = u32_to_usize(exc_size / 12);
        let tail = self.bytes.get(off..).ok_or_else(|| XtaskError::PeExtract {
            context: ".pdata",
            detail: "exception table offset out of bounds".into(),
        })?;
        let (funcs, _) =
            <[RawRuntimeFunction]>::ref_from_prefix_with_elems(tail, count).map_err(|_| {
                XtaskError::PeExtract {
                    context: ".pdata",
                    detail: "truncated RUNTIME_FUNCTION table".into(),
                }
            })?;
        self.pdata = funcs
            .iter()
            .map(|f| (f.begin_address.get(), f.end_address.get()))
            .collect();
        Ok(())
    }

    #[inline]
    #[must_use]
    pub const fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    #[must_use]
    pub const fn image_base(&self) -> u64 {
        self.image_base
    }

    /// Finds a section by its 8-byte header name (e.g. `b".text\0\0\0"`).
    #[must_use]
    pub fn section_by_name(&self, name: &[u8; 8]) -> Option<(PeSection, &[u8], u64)> {
        for &sec in &self.sections {
            if &sec.name == name {
                let start = u32_to_usize(sec.raw_ptr);
                let end = start.checked_add(u32_to_usize(sec.raw_size))?;
                let slice = self.bytes.get(start..end)?;
                return Some((sec, slice, self.image_base + u64::from(sec.vaddr)));
            }
        }
        None
    }

    /// Translates a virtual address (`va`) into a file offset inside `self.bytes`.
    #[must_use]
    pub fn va_to_off(&self, va: u64) -> Option<usize> {
        let rva_u64 = va.checked_sub(self.image_base)?;
        let Ok(rva) = u32::try_from(rva_u64) else {
            return None;
        };
        for &sec in &self.sections {
            let span = sec.vsize.max(sec.raw_size);
            if rva >= sec.vaddr && rva < sec.vaddr.saturating_add(span) {
                let delta = rva - sec.vaddr;
                if delta < sec.raw_size {
                    return Some(u32_to_usize(sec.raw_ptr + delta));
                }
            }
        }
        None
    }

    /// Looks up the enclosing `.pdata` function `(start_va, end_va)` for `va`.
    #[must_use]
    pub fn find_fn(&self, va: u64) -> Option<(u64, u64)> {
        let Ok(rva) = u32::try_from(va.checked_sub(self.image_base)?) else {
            return None;
        };
        let idx = self.pdata.partition_point(|&(b, _)| b <= rva);
        if idx > 0 {
            let (b, e) = self.pdata[idx - 1];
            if rva >= b && rva < e {
                return Some((
                    self.image_base + u64::from(b),
                    self.image_base + u64::from(e),
                ));
            }
        }
        None
    }

    /// Reads a NUL-terminated UTF-8 C-string at `va` if valid.
    #[must_use]
    pub fn cstr_at_va(&self, va: u64) -> Option<&str> {
        let off = self.va_to_off(va)?;
        let tail = self.bytes.get(off..)?;
        let end = tail.iter().take(512).position(|&c| c == 0)?;
        let slice = tail.get(..end)?;
        if let Ok(s) = core::str::from_utf8(slice) {
            return Some(s);
        }
        None
    }

    /// Reads a little-endian `u32` at file offset `off`.
    #[must_use]
    pub fn read_u32_at(&self, off: usize) -> Option<u32> {
        let tail = self.bytes.get(off..)?;
        let Ok((v, _)) = U32::ref_from_prefix(tail) else {
            return None;
        };
        Some(v.get())
    }

    /// Reads a little-endian `u64` at file offset `off`.
    #[must_use]
    pub fn read_u64_at(&self, off: usize) -> Option<u64> {
        let tail = self.bytes.get(off..)?;
        let Ok((v, _)) = U64::ref_from_prefix(tail) else {
            return None;
        };
        Some(v.get())
    }

    /// Disassembles the function starting at `va`, following `.pdata` chained unwind blocks.
    #[must_use]
    pub fn fn_insns(&self, va: u64) -> Vec<Instruction> {
        let start_va = va;
        let found = self.find_fn(va);
        let has_pdata = found.is_some();
        let mut end_va = match found {
            Some((_, e)) => e,
            None => va + NO_PDATA_BYTES_MAX as u64,
        };
        let Some(off) = self.va_to_off(start_va) else {
            return Vec::new();
        };
        for _ in 0..8 {
            let Ok(span) = usize::try_from(end_va.saturating_sub(start_va)) else {
                return Vec::new();
            };
            let len = span.min(FN_DISASM_BYTES_MAX);
            let end_off = off.saturating_add(len).min(self.bytes.len());
            let mut decoder = Decoder::with_ip(
                64,
                &self.bytes[off..end_off],
                start_va,
                DecoderOptions::NONE,
            );
            let mut out = Vec::new();
            let mut max_target = if has_pdata { end_va } else { start_va };
            let branch_cap = start_va
                + (if has_pdata {
                    FN_DISASM_BYTES_MAX
                } else {
                    NO_PDATA_BYTES_MAX
                }) as u64;
            while decoder.can_decode() {
                let ins = decoder.decode();
                if ins.mnemonic() != Mnemonic::Call
                    && matches!(
                        ins.op0_kind(),
                        OpKind::NearBranch64 | OpKind::NearBranch32 | OpKind::NearBranch16
                    )
                {
                    let t = ins.near_branch_target();
                    if t > max_target && t < branch_cap {
                        max_target = t;
                    }
                }
                let is_ret = ins.flow_control() == FlowControl::Return
                    || (!has_pdata
                        && ins.flow_control() == FlowControl::UnconditionalBranch
                        && (ins.near_branch_target() < start_va
                            || ins.near_branch_target() >= branch_cap));
                let ip_after = ins.next_ip();
                out.push(ins);
                if !has_pdata && is_ret && ip_after >= max_target {
                    break;
                }
            }
            if has_pdata
                && max_target > end_va
                && let Some((_, next_end)) = self.find_fn(max_target)
                && next_end > end_va
            {
                end_va = next_end;
                continue;
            }
            return out;
        }
        Vec::new()
    }
}

/// Primary-key hierarchy layout extracted from a table's `ReadBin` or `_idx` loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractedKeySchema {
    Int(u8),
    Str1,
    StrInt(u8),
}

impl ExtractedKeySchema {
    /// Returns the Rust primary key type token stream for `BinTable<K, Row>`.
    #[must_use]
    pub fn rust_key_type_tokens(self) -> TokenStream {
        match self {
            Self::Int(1) => quote! { i32 },
            Self::Int(2) => quote! { (i32, i32) },
            Self::Int(3) => quote! { (i32, i32, i32) },
            Self::Int(_) => quote! { (i32, i32, i32, i32) },
            Self::Str1 => quote! { Box<str> },
            Self::StrInt(1) => quote! { (Box<str>, i32) },
            Self::StrInt(2) => quote! { (Box<str>, i32, i32) },
            Self::StrInt(_) => quote! { (Box<str>, i32, i32, i32) },
        }
    }

    /// Returns the hierarchy depth (`1..=4`).
    #[must_use]
    pub const fn depth(self) -> u8 {
        match self {
            Self::Int(d) => d,
            Self::Str1 => 1,
            Self::StrInt(n) => 1 + n,
        }
    }
}

/// Low-level stream or call operation inside a `ReadBin` instruction sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RawOp {
    VtableRead(u32),
    Call(u64),
}

/// Extracts `VtableRead(byte_len)` and `Call(target_va)` operations from `insns`.
#[must_use]
pub fn extract_ops(insns: &[Instruction]) -> Vec<RawOp> {
    let Some(first_ip) = insns.first().map(Instruction::ip) else {
        return Vec::new();
    };
    let last_ip = match insns.last() {
        Some(ins) => ins.ip(),
        None => first_ip,
    };
    let mut ops = Vec::new();
    let mut last_r8d = 0u32;
    for ins in insns {
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
                ops.push(RawOp::VtableRead(last_r8d));
            } else if ins.op0_kind() == OpKind::NearBranch64
                && (ins.mnemonic() == Mnemonic::Call
                    || ins.near_branch64() < first_ip
                    || ins.near_branch64() > last_ip)
            {
                ops.push(RawOp::Call(ins.near_branch64()));
            }
        }
    }
    ops
}

const LOCALE_STEM_SUFFIXES: [&str; 19] = [
    "_cn", "_de", "_en", "_es", "_fr", "_hk", "_id", "_ja", "_jp", "_ko", "_kr", "_pt", "_ru",
    "_th", "_tr", "_tw", "_vi", "_vn", "_zh",
];

/// Normalizes a C++ `*Info.cpp`, `F*Info`, or `config_*.bin` name to its canonical stem (`a-z0-9` without `_`).
#[must_use]
pub fn normalize_stem(input: &str) -> String {
    let fname = match input.rsplit(['/', '\\']).next() {
        Some(n) => n,
        None => input,
    };
    let orig_starts_uf_upper = {
        let bytes = fname.as_bytes();
        bytes.len() >= 2
            && (bytes[0] == b'U' || bytes[0] == b'F')
            && bytes[1].is_ascii_uppercase()
            && !fname.starts_with("UI")
    };
    let mut s = fname.to_ascii_lowercase();
    let is_chunk_bin = s.ends_with("_chunka0.bin");
    if let Some(stripped) = s.strip_suffix(".cpp") {
        s = stripped.to_owned();
    }
    for suf in ["_chunka0.bin", ".bin", ".out", "infomgr", "info"] {
        if let Some(stripped) = s.strip_suffix(suf) {
            s = stripped.to_owned();
            break;
        }
    }
    if is_chunk_bin {
        for loc in LOCALE_STEM_SUFFIXES {
            if let Some(stripped) = s.strip_suffix(loc) {
                s = stripped.to_owned();
                break;
            }
        }
    }
    if let Some(stripped) = s.strip_prefix("config_") {
        s = stripped.to_owned();
    } else if orig_starts_uf_upper {
        s = s[1..].to_owned();
    }
    if let Some(stripped) = s.strip_prefix("enum") {
        s = stripped.to_owned();
    }
    s.retain(|c| c != '_');
    s
}

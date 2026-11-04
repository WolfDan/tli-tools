//! Zero-copy binary wire layouts and slice-shrinking decoders for `.pak` footers and entries.

use arrayvec::{ArrayString, ArrayVec};
use zerocopy::byteorder::little_endian::{I32, U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

use crate::error::{EncryptedSectionKind, LimitKind, PakError, WireContext};
use crate::types::{
    BLOCKS_PER_ENTRY_COUNT_MAX, COMPRESSION_METHODS_COUNT_MAX, CompressionMethod,
    EncryptionKeyGuid, EntryFlags, FOOTER_V8A_BYTES, FOOTER_V8B_V10_V11_BYTES, FOOTER_V9_V12_BYTES,
    PATH_UTF16_UNITS_MAX, PakDialect, PakVersion, Sha1Hash,
};
use crate::{align_up_16, bounded_len_u32, u32_to_usize};

pub(crate) const MAGIC_UE4: u32 = 0x5A6F_12E1;
pub(crate) const MAGIC_TLI_V12: u32 = 0x6B2A_56B8;

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub(crate) struct RawFooterPrefix {
    pub(crate) encryption_key_guid: [u8; 16],
    pub(crate) is_index_encrypted: u8,
}
const _: () = assert!(core::mem::size_of::<RawFooterPrefix>() == 17);

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub(crate) struct RawFooterCoreStd {
    pub(crate) magic: U32,
    pub(crate) version: U32,
    pub(crate) index_offset: U64,
    pub(crate) index_size: U64,
    pub(crate) index_hash: [u8; 20],
}
const _: () = assert!(core::mem::size_of::<RawFooterCoreStd>() == 44);

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub(crate) struct RawFooterCoreV12 {
    pub(crate) magic: U32,
    pub(crate) version: U32,
    pub(crate) v12_extra: u8,
    pub(crate) index_offset: U64,
    pub(crate) index_size: U64,
    pub(crate) index_hash: [u8; 20],
}
const _: () = assert!(core::mem::size_of::<RawFooterCoreV12>() == 45);

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub(crate) struct RawSubIndexHeader {
    pub(crate) offset: U64,
    pub(crate) size: U64,
    pub(crate) hash: [u8; 20],
}
const _: () = assert!(core::mem::size_of::<RawSubIndexHeader>() == 36);

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub(crate) struct RawPathHashRecord {
    pub(crate) hash: U64,
    pub(crate) location: I32,
}
const _: () = assert!(core::mem::size_of::<RawPathHashRecord>() == 12);

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub(crate) struct RawEntryCore {
    pub(crate) offset: U64,
    pub(crate) compressed_size: U64,
    pub(crate) uncompressed_size: U64,
    pub(crate) compression_method: U32,
    pub(crate) hash: [u8; 20],
}
const _: () = assert!(core::mem::size_of::<RawEntryCore>() == 48);

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub(crate) struct RawCompressedBlock {
    pub(crate) start: U64,
    pub(crate) end: U64,
}
const _: () = assert!(core::mem::size_of::<RawCompressedBlock>() == 16);

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub(crate) struct RawEntryTail {
    pub(crate) flags: u8,
    pub(crate) compression_block_size: U32,
}
const _: () = assert!(core::mem::size_of::<RawEntryTail>() == 5);

/// Single compression block offset (relative to entry `payload_offset`) and unpadded byte length.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompressionBlock {
    pub(crate) rel_offset: u64,
    pub(crate) compressed_bytes: u32,
}
const _: () = assert!(core::mem::size_of::<CompressionBlock>() == 16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MethodSlot {
    Empty,
    Known(CompressionMethod),
    Unsupported(ArrayString<32>),
}

/// Validated footer compression method lookup table (`1..=slots.len()`).
#[derive(Clone, Debug)]
pub(crate) struct MethodTable {
    slots: ArrayVec<MethodSlot, { u32_to_usize(COMPRESSION_METHODS_COUNT_MAX) }>,
}

impl MethodTable {
    pub(crate) fn resolve(&self, raw_index: u32) -> Result<CompressionMethod, PakError> {
        if raw_index == 0 {
            return Ok(CompressionMethod::None);
        }
        let table_len = bounded_len_u32(self.slots.len());
        let slot_idx = u32_to_usize(raw_index - 1);
        let Some(&slot) = self.slots.get(slot_idx) else {
            return Err(PakError::InvalidCompressionMethodIndex {
                raw_index,
                table_len,
            });
        };
        match slot {
            MethodSlot::Known(method) => Ok(method),
            MethodSlot::Empty => Err(PakError::InvalidCompressionMethodIndex {
                raw_index,
                table_len,
            }),
            MethodSlot::Unsupported(name) => Err(PakError::UnsupportedCompressionMethod { name }),
        }
    }
}

/// Parsed and validated `FPakInfo` footer metadata.
#[derive(Clone, Debug)]
pub(crate) struct ParsedFooter {
    pub(crate) dialect: PakDialect,
    pub(crate) version: PakVersion,
    pub(crate) footer_bytes: u32,
    pub(crate) encryption_key_guid: EncryptionKeyGuid,
    pub(crate) is_index_encrypted: bool,
    pub(crate) index_offset: u64,
    pub(crate) index_size: u64,
    pub(crate) index_hash: Sha1Hash,
    pub(crate) methods: MethodTable,
}

/// Decoded per-entry metadata ready for structural boundary checks before `EntryTable` insertion.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DecodedEntry {
    pub(crate) header_offset: u64,
    pub(crate) payload_offset: u64,
    pub(crate) compressed_bytes: u64,
    pub(crate) disk_span_bytes: u64,
    pub(crate) uncompressed_bytes: u64,
    pub(crate) compression_block_bytes: u32,
    pub(crate) block_count: u16,
    pub(crate) method: CompressionMethod,
    pub(crate) flags: EntryFlags,
}

/// Advances `buf` past a zero-copy unaligned wire struct `T`.
#[inline]
pub(crate) fn take<'a, T>(buf: &mut &'a [u8], context: WireContext) -> Result<&'a T, PakError>
where
    T: FromBytes + KnownLayout + Immutable + Unaligned,
{
    let (item, rest) =
        T::ref_from_prefix(buf).map_err(|_| PakError::CorruptWireLayout { context })?;
    *buf = rest;
    Ok(item)
}

/// Advances `buf` by `len` bytes, returning the borrowed prefix sub-slice.
#[inline]
pub(crate) fn take_slice<'a>(
    buf: &mut &'a [u8],
    len: u32,
    context: WireContext,
) -> Result<&'a [u8], PakError> {
    let (head, rest) = buf
        .split_at_checked(u32_to_usize(len))
        .ok_or(PakError::CorruptWireLayout { context })?;
    *buf = rest;
    Ok(head)
}

#[inline]
pub(crate) fn take_u8(buf: &mut &[u8], context: WireContext) -> Result<u8, PakError> {
    let slice = take_slice(buf, 1, context)?;
    Ok(slice[0])
}

#[inline]
pub(crate) fn take_u32_le(buf: &mut &[u8], context: WireContext) -> Result<u32, PakError> {
    Ok(take::<U32>(buf, context)?.get())
}

#[inline]
pub(crate) fn take_i32_le(buf: &mut &[u8], context: WireContext) -> Result<i32, PakError> {
    Ok(take::<I32>(buf, context)?.get())
}

#[inline]
pub(crate) fn take_u64_le(buf: &mut &[u8], context: WireContext) -> Result<u64, PakError> {
    Ok(take::<U64>(buf, context)?.get())
}

/// Decodes a serialized Unreal `FString` (UTF-8 or UTF-16LE) into `out` without lossy replacement.
pub(crate) fn take_fstring_into(
    buf: &mut &[u8],
    max_utf8_bytes: u32,
    limit_kind: LimitKind,
    context: WireContext,
    out: &mut String,
) -> Result<(), PakError> {
    out.clear();
    let raw_len = take_i32_le(buf, context)?;
    if raw_len == 0 {
        return Ok(());
    }
    if raw_len > 0 {
        let byte_count =
            u32::try_from(raw_len).map_err(|_| PakError::CorruptWireLayout { context })?;
        return take_fstring_utf8(buf, byte_count, max_utf8_bytes, limit_kind, context, out);
    }
    let neg_i32 = raw_len
        .checked_neg()
        .ok_or(PakError::CorruptWireLayout { context })?;
    let unit_count = u32::try_from(neg_i32).map_err(|_| PakError::CorruptWireLayout { context })?;
    take_fstring_utf16(buf, unit_count, max_utf8_bytes, limit_kind, context, out)
}

fn take_fstring_utf8(
    buf: &mut &[u8],
    byte_count: u32,
    max_utf8_bytes: u32,
    limit_kind: LimitKind,
    context: WireContext,
    out: &mut String,
) -> Result<(), PakError> {
    let content_bytes = byte_count
        .checked_sub(1)
        .ok_or(PakError::CorruptWireLayout { context })?;
    if content_bytes > max_utf8_bytes {
        return Err(PakError::LimitExceeded {
            kind: limit_kind,
            actual: u64::from(content_bytes),
            max: u64::from(max_utf8_bytes),
        });
    }
    let raw = take_slice(buf, byte_count, context)?;
    let content_end = u32_to_usize(content_bytes);
    if raw[content_end] != 0 {
        return Err(PakError::CorruptWireLayout { context });
    }
    let content = &raw[..content_end];
    let text =
        core::str::from_utf8(content).map_err(|_| PakError::InvalidUtf8String { context })?;
    out.push_str(text);
    Ok(())
}

fn take_fstring_utf16(
    buf: &mut &[u8],
    unit_count: u32,
    max_utf8_bytes: u32,
    limit_kind: LimitKind,
    context: WireContext,
    out: &mut String,
) -> Result<(), PakError> {
    let content_units = unit_count
        .checked_sub(1)
        .ok_or(PakError::CorruptWireLayout { context })?;
    if content_units > PATH_UTF16_UNITS_MAX {
        return Err(PakError::LimitExceeded {
            kind: LimitKind::PathUtf16Units,
            actual: u64::from(content_units),
            max: u64::from(PATH_UTF16_UNITS_MAX),
        });
    }
    let raw_bytes = unit_count
        .checked_mul(2)
        .ok_or(PakError::CorruptWireLayout { context })?;
    let raw = take_slice(buf, raw_bytes, context)?;
    let (pairs, rem) = raw.as_chunks::<2>();
    assert!(rem.is_empty());
    let term_idx = u32_to_usize(content_units);
    if pairs[term_idx] != [0, 0] {
        return Err(PakError::CorruptWireLayout { context });
    }
    let mut encode_buf = [0_u8; 4];
    for ch_res in char::decode_utf16(pairs[..term_idx].iter().map(|p| u16::from_le_bytes(*p))) {
        let ch = ch_res.map_err(|_| PakError::InvalidUtf16String { context })?;
        let encoded = ch.encode_utf8(&mut encode_buf);
        let new_len = out
            .len()
            .checked_add(encoded.len())
            .ok_or(PakError::CorruptWireLayout { context })?;
        if new_len > u32_to_usize(max_utf8_bytes) {
            let actual =
                u64::try_from(new_len).map_err(|_| PakError::CorruptWireLayout { context })?;
            return Err(PakError::LimitExceeded {
                kind: limit_kind,
                actual,
                max: u64::from(max_utf8_bytes),
            });
        }
        out.push_str(encoded);
    }
    Ok(())
}

/// Probes the trailing `<= 222` bytes of a `.pak` file across all supported footer layouts.
pub(crate) fn parse_footer_tail(tail: &[u8]) -> Result<ParsedFooter, PakError> {
    let probes = [
        probe_tli_v12(tail),
        probe_ue4_221(tail),
        probe_ue4_v9(tail),
        probe_ue4_v8a(tail),
    ];
    let mut first_err = None;
    for candidate in probes {
        match candidate {
            Some(Ok(footer)) => return Ok(footer),
            Some(Err(err)) => {
                first_err.get_or_insert(err);
            }
            None => {}
        }
    }
    match first_err {
        Some(err) => Err(err),
        None => Err(PakError::InvalidFooterMagic),
    }
}

fn tail_suffix(tail: &[u8], suffix_bytes: u32) -> Option<&[u8]> {
    let need = u32_to_usize(suffix_bytes);
    let start = tail.len().checked_sub(need)?;
    tail.get(start..)
}

fn decode_wire_bool(raw: u8) -> Result<bool, PakError> {
    match raw {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(PakError::CorruptWireLayout {
            context: WireContext::Footer,
        }),
    }
}

fn parse_method_table(buf: &mut &[u8], slot_count: u32) -> Result<MethodTable, PakError> {
    assert!(slot_count >= 4);
    assert!(slot_count <= COMPRESSION_METHODS_COUNT_MAX);
    let mut slots = ArrayVec::<MethodSlot, { u32_to_usize(COMPRESSION_METHODS_COUNT_MAX) }>::new();
    for _ in 0..slot_count {
        let raw_bytes = take::<[u8; 32]>(buf, WireContext::Footer)?;
        slots.push(parse_method_slot(raw_bytes)?);
    }
    Ok(MethodTable { slots })
}

fn parse_method_slot(raw: &[u8; 32]) -> Result<MethodSlot, PakError> {
    let prefix = match raw.iter().position(|&b| b == 0) {
        Some(idx) => &raw[..idx],
        None => &raw[..],
    };
    if prefix.is_empty() {
        return Ok(MethodSlot::Empty);
    }
    let name = core::str::from_utf8(prefix).map_err(|_| PakError::InvalidUtf8String {
        context: WireContext::Footer,
    })?;
    if !name.is_ascii() {
        return Err(PakError::InvalidUtf8String {
            context: WireContext::Footer,
        });
    }
    if name.eq_ignore_ascii_case("zlib") {
        return Ok(MethodSlot::Known(CompressionMethod::Zlib));
    }
    if name.eq_ignore_ascii_case("oodle") {
        return Ok(MethodSlot::Known(CompressionMethod::Oodle));
    }
    let stored = ArrayString::<32>::from(name).map_err(|_| PakError::CorruptWireLayout {
        context: WireContext::Footer,
    })?;
    Ok(MethodSlot::Unsupported(stored))
}

fn probe_tli_v12(tail: &[u8]) -> Option<Result<ParsedFooter, PakError>> {
    let mut cur = tail_suffix(tail, FOOTER_V9_V12_BYTES)?;
    let prefix = match take::<RawFooterPrefix>(&mut cur, WireContext::Footer) {
        Ok(val) => val,
        Err(err) => return Some(Err(err)),
    };
    let core = match take::<RawFooterCoreV12>(&mut cur, WireContext::Footer) {
        Ok(val) => val,
        Err(err) => return Some(Err(err)),
    };
    if core.magic.get() != MAGIC_TLI_V12 {
        return None;
    }
    let raw_version = core.version.get();
    if raw_version != 12 {
        return Some(Err(PakError::UnsupportedVersion {
            dialect: PakDialect::TorchlightInfiniteV12,
            raw_version,
        }));
    }
    let is_encrypted = match decode_wire_bool(prefix.is_index_encrypted) {
        Ok(flag) => flag,
        Err(err) => return Some(Err(err)),
    };
    let methods = match parse_method_table(&mut cur, 5) {
        Ok(table) => table,
        Err(err) => return Some(Err(err)),
    };
    assert!(cur.is_empty());
    Some(Ok(ParsedFooter {
        dialect: PakDialect::TorchlightInfiniteV12,
        version: PakVersion::V12,
        footer_bytes: FOOTER_V9_V12_BYTES,
        encryption_key_guid: EncryptionKeyGuid::from_bytes(prefix.encryption_key_guid),
        is_index_encrypted: is_encrypted,
        index_offset: core.index_offset.get(),
        index_size: core.index_size.get(),
        index_hash: Sha1Hash::from_bytes(core.index_hash),
        methods,
    }))
}

fn probe_ue4_221(tail: &[u8]) -> Option<Result<ParsedFooter, PakError>> {
    let mut cur = tail_suffix(tail, FOOTER_V8B_V10_V11_BYTES)?;
    let prefix = match take::<RawFooterPrefix>(&mut cur, WireContext::Footer) {
        Ok(val) => val,
        Err(err) => return Some(Err(err)),
    };
    let core = match take::<RawFooterCoreStd>(&mut cur, WireContext::Footer) {
        Ok(val) => val,
        Err(err) => return Some(Err(err)),
    };
    if core.magic.get() != MAGIC_UE4 {
        return None;
    }
    let version = match core.version.get() {
        8 => PakVersion::V8B,
        10 => PakVersion::V10,
        11 => PakVersion::V11,
        raw_version => {
            return Some(Err(PakError::UnsupportedVersion {
                dialect: PakDialect::StandardUe4,
                raw_version,
            }));
        }
    };
    Some(finish_std_footer(
        prefix,
        core,
        version,
        FOOTER_V8B_V10_V11_BYTES,
        &mut cur,
        5,
    ))
}

fn probe_ue4_v9(tail: &[u8]) -> Option<Result<ParsedFooter, PakError>> {
    let mut cur = tail_suffix(tail, FOOTER_V9_V12_BYTES)?;
    let prefix = match take::<RawFooterPrefix>(&mut cur, WireContext::Footer) {
        Ok(val) => val,
        Err(err) => return Some(Err(err)),
    };
    let core = match take::<RawFooterCoreStd>(&mut cur, WireContext::Footer) {
        Ok(val) => val,
        Err(err) => return Some(Err(err)),
    };
    if core.magic.get() != MAGIC_UE4 {
        return None;
    }
    let raw_version = core.version.get();
    if raw_version != 9 {
        return Some(Err(PakError::UnsupportedVersion {
            dialect: PakDialect::StandardUe4,
            raw_version,
        }));
    }
    let is_frozen = match take_u8(&mut cur, WireContext::Footer) {
        Ok(val) => val,
        Err(err) => return Some(Err(err)),
    };
    if is_frozen != 0 {
        return Some(Err(PakError::FrozenIndexUnsupported));
    }
    Some(finish_std_footer(
        prefix,
        core,
        PakVersion::V9,
        FOOTER_V9_V12_BYTES,
        &mut cur,
        5,
    ))
}

fn probe_ue4_v8a(tail: &[u8]) -> Option<Result<ParsedFooter, PakError>> {
    let mut cur = tail_suffix(tail, FOOTER_V8A_BYTES)?;
    let prefix = match take::<RawFooterPrefix>(&mut cur, WireContext::Footer) {
        Ok(val) => val,
        Err(err) => return Some(Err(err)),
    };
    let core = match take::<RawFooterCoreStd>(&mut cur, WireContext::Footer) {
        Ok(val) => val,
        Err(err) => return Some(Err(err)),
    };
    if core.magic.get() != MAGIC_UE4 {
        return None;
    }
    let raw_version = core.version.get();
    if raw_version != 8 {
        return Some(Err(PakError::UnsupportedVersion {
            dialect: PakDialect::StandardUe4,
            raw_version,
        }));
    }
    Some(finish_std_footer(
        prefix,
        core,
        PakVersion::V8A,
        FOOTER_V8A_BYTES,
        &mut cur,
        4,
    ))
}

fn finish_std_footer(
    prefix: &RawFooterPrefix,
    core: &RawFooterCoreStd,
    version: PakVersion,
    footer_bytes: u32,
    cur: &mut &[u8],
    slot_count: u32,
) -> Result<ParsedFooter, PakError> {
    let is_encrypted = decode_wire_bool(prefix.is_index_encrypted)?;
    let methods = parse_method_table(cur, slot_count)?;
    assert!(cur.is_empty());
    Ok(ParsedFooter {
        dialect: PakDialect::StandardUe4,
        version,
        footer_bytes,
        encryption_key_guid: EncryptionKeyGuid::from_bytes(prefix.encryption_key_guid),
        is_index_encrypted: is_encrypted,
        index_offset: core.index_offset.get(),
        index_size: core.index_size.get(),
        index_hash: Sha1Hash::from_bytes(core.index_hash),
        methods,
    })
}

/// Computes the exact serialized byte size of an inline `FPakEntry` header on disk.
#[inline]
#[must_use]
pub(crate) fn inline_header_bytes(
    dialect: PakDialect,
    has_compression: bool,
    block_count: u16,
) -> u32 {
    let base: u32 = match dialect {
        PakDialect::StandardUe4 => 53,
        PakDialect::TorchlightInfiniteV12 => 54,
    };
    if has_compression {
        let block_bytes = u32::from(block_count) * 16;
        base + 4 + block_bytes
    } else {
        base
    }
}

pub(crate) const INLINE_HEADER_HASH_PREFIX_BYTES_MAX: u32 = 49;

/// Exact byte prefix of an inline `FPakEntry` header required by [`parse_inline_entry_sha1`].
#[inline]
#[must_use]
pub(crate) const fn inline_header_hash_prefix_bytes(dialect: PakDialect) -> u32 {
    match dialect {
        PakDialect::StandardUe4 => 48,
        PakDialect::TorchlightInfiniteV12 => 49,
    }
}

/// Extracts the 20-byte SHA-1 payload digest from an inline `FPakEntry` header slice.
pub(crate) fn parse_inline_entry_sha1(
    mut buf: &[u8],
    dialect: PakDialect,
) -> Result<Sha1Hash, PakError> {
    if dialect == PakDialect::TorchlightInfiniteV12 {
        let _prefix = take_u8(&mut buf, WireContext::InlineEntryHeader)?;
    }
    let core = take::<RawEntryCore>(&mut buf, WireContext::InlineEntryHeader)?;
    Ok(Sha1Hash::from_bytes(core.hash))
}

struct UnencodedBlocksSpec {
    hdr_bytes: u32,
    block_count: u16,
    is_encrypted: bool,
    uncompressed_bytes: u64,
    method: CompressionMethod,
    blocks_total_max: u32,
}

/// Decodes a full unencoded `FPakEntry` (used in `v8–v9` indexes and `v10–v12` `UnencodedEntries`),
/// appending any compression blocks directly into `blocks_out`.
pub(crate) fn decode_unencoded_entry(
    buf: &mut &[u8],
    dialect: PakDialect,
    methods: &MethodTable,
    blocks_total_max: u32,
    blocks_out: &mut Vec<CompressionBlock>,
) -> Result<DecodedEntry, PakError> {
    if dialect == PakDialect::TorchlightInfiniteV12 {
        let _prefix = take_u8(buf, WireContext::UnencodedEntry)?;
    }
    let core = take::<RawEntryCore>(buf, WireContext::UnencodedEntry)?;
    let raw_method = core.compression_method.get();
    let method = methods.resolve(raw_method)?;
    let (block_count, raw_blocks_slice) = if raw_method != 0 {
        let count = take_u32_le(buf, WireContext::UnencodedEntry)?;
        let Ok(count_u16) = u16::try_from(count) else {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::BlocksPerEntryCount,
                actual: u64::from(count),
                max: u64::from(BLOCKS_PER_ENTRY_COUNT_MAX),
            });
        };
        let byte_len = count.checked_mul(16).ok_or(PakError::CorruptWireLayout {
            context: WireContext::UnencodedEntry,
        })?;
        let slice = take_slice(buf, byte_len, WireContext::UnencodedEntry)?;
        (count_u16, slice)
    } else {
        (0, &[][..])
    };
    let tail = take::<RawEntryTail>(buf, WireContext::UnencodedEntry)?;
    let flags = EntryFlags::from_bits_retain(tail.flags);
    let is_encrypted = flags.contains(EntryFlags::ENCRYPTED);
    let hdr_bytes = inline_header_bytes(dialect, raw_method != 0, block_count);
    let header_offset = core.offset.get();
    let payload_offset =
        header_offset
            .checked_add(u64::from(hdr_bytes))
            .ok_or(PakError::CorruptWireLayout {
                context: WireContext::UnencodedEntry,
            })?;
    let disk_span_bytes = push_unencoded_blocks(
        raw_blocks_slice,
        UnencodedBlocksSpec {
            hdr_bytes,
            block_count,
            is_encrypted,
            uncompressed_bytes: core.uncompressed_size.get(),
            method,
            blocks_total_max,
        },
        blocks_out,
    )?;
    Ok(DecodedEntry {
        header_offset,
        payload_offset,
        compressed_bytes: core.compressed_size.get(),
        disk_span_bytes,
        uncompressed_bytes: core.uncompressed_size.get(),
        compression_block_bytes: tail.compression_block_size.get(),
        block_count,
        method,
        flags,
    })
}

fn push_unencoded_blocks(
    mut raw_blocks_bytes: &[u8],
    spec: UnencodedBlocksSpec,
    blocks_out: &mut Vec<CompressionBlock>,
) -> Result<u64, PakError> {
    if spec.method == CompressionMethod::None {
        if spec.is_encrypted {
            return align_up_16(spec.uncompressed_bytes).ok_or(PakError::CorruptWireLayout {
                context: WireContext::UnencodedEntry,
            });
        }
        return Ok(spec.uncompressed_bytes);
    }
    let hdr_u64 = u64::from(spec.hdr_bytes);
    let mut prev_end: u64 = 0;
    for _ in 0..spec.block_count {
        let blk = take::<RawCompressedBlock>(&mut raw_blocks_bytes, WireContext::UnencodedEntry)?;
        let start = blk.start.get();
        let end = blk.end.get();
        let rel_offset = start
            .checked_sub(hdr_u64)
            .ok_or(PakError::CorruptWireLayout {
                context: WireContext::UnencodedEntry,
            })?;
        let raw_len = end.checked_sub(start).ok_or(PakError::CorruptWireLayout {
            context: WireContext::UnencodedEntry,
        })?;
        let compressed_bytes = u32::try_from(raw_len).map_err(|_| PakError::CorruptWireLayout {
            context: WireContext::UnencodedEntry,
        })?;
        if rel_offset < prev_end {
            return Err(PakError::CorruptWireLayout {
                context: WireContext::UnencodedEntry,
            });
        }
        if spec.is_encrypted && !rel_offset.is_multiple_of(16) {
            return Err(PakError::UnalignedEncryptedSize {
                context: EncryptedSectionKind::EntryPayload,
                byte_len: rel_offset,
            });
        }
        let stride = if spec.is_encrypted {
            align_up_16(raw_len).ok_or(PakError::CorruptWireLayout {
                context: WireContext::UnencodedEntry,
            })?
        } else {
            raw_len
        };
        prev_end = rel_offset
            .checked_add(stride)
            .ok_or(PakError::CorruptWireLayout {
                context: WireContext::UnencodedEntry,
            })?;
        push_block_checked(
            blocks_out,
            CompressionBlock {
                rel_offset,
                compressed_bytes,
            },
            spec.blocks_total_max,
        )?;
    }
    assert!(raw_blocks_bytes.is_empty());
    Ok(prev_end)
}

struct EncodedBlocksSpec {
    method: CompressionMethod,
    is_encrypted: bool,
    block_count: u16,
    compressed_bytes: u64,
    uncompressed_bytes: u64,
    blocks_total_max: u32,
}

/// Decodes a UE4 `v10–v12` bit-packed `EncodedPakEntries` record from a pre-sliced buffer `buf`,
/// appending any compression blocks directly into `blocks_out`.
pub(crate) fn decode_encoded_entry(
    mut buf: &[u8],
    dialect: PakDialect,
    methods: &MethodTable,
    blocks_total_max: u32,
    blocks_out: &mut Vec<CompressionBlock>,
) -> Result<DecodedEntry, PakError> {
    let bits = take_u32_le(&mut buf, WireContext::EncodedEntry)?;
    let raw_method = (bits >> 23) & 0x3F;
    let method = methods.resolve(raw_method)?;
    let is_encrypted = ((bits >> 22) & 1) != 0;
    let block_count = ((bits >> 6) & 0xFFFF) as u16;
    let block_size_code = bits & 0x3F;
    let compression_block_bytes = if block_size_code == 0x3F {
        take_u32_le(&mut buf, WireContext::EncodedEntry)?
    } else {
        block_size_code << 11
    };
    let header_offset = take_packed_size(&mut buf, (bits & (1 << 31)) != 0)?;
    let uncompressed_bytes = take_packed_size(&mut buf, (bits & (1 << 30)) != 0)?;
    let compressed_bytes = if raw_method != 0 {
        take_packed_size(&mut buf, (bits & (1 << 29)) != 0)?
    } else {
        uncompressed_bytes
    };
    let hdr_bytes = inline_header_bytes(dialect, raw_method != 0, block_count);
    let payload_offset =
        header_offset
            .checked_add(u64::from(hdr_bytes))
            .ok_or(PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            })?;
    let disk_span_bytes = push_encoded_blocks(
        &mut buf,
        EncodedBlocksSpec {
            method,
            is_encrypted,
            block_count,
            compressed_bytes,
            uncompressed_bytes,
            blocks_total_max,
        },
        blocks_out,
    )?;
    let flags = if is_encrypted {
        EntryFlags::ENCRYPTED
    } else {
        EntryFlags::empty()
    };
    Ok(DecodedEntry {
        header_offset,
        payload_offset,
        compressed_bytes,
        disk_span_bytes,
        uncompressed_bytes,
        compression_block_bytes,
        block_count,
        method,
        flags,
    })
}

#[inline]
fn take_packed_size(buf: &mut &[u8], is_32bit: bool) -> Result<u64, PakError> {
    if is_32bit {
        Ok(u64::from(take_u32_le(buf, WireContext::EncodedEntry)?))
    } else {
        take_u64_le(buf, WireContext::EncodedEntry)
    }
}

fn push_encoded_blocks(
    buf: &mut &[u8],
    spec: EncodedBlocksSpec,
    blocks_out: &mut Vec<CompressionBlock>,
) -> Result<u64, PakError> {
    if spec.method == CompressionMethod::None {
        if spec.block_count != 0 {
            return Err(PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            });
        }
        return if spec.is_encrypted {
            align_up_16(spec.uncompressed_bytes).ok_or(PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            })
        } else {
            Ok(spec.uncompressed_bytes)
        };
    }
    if spec.block_count == 1 && !spec.is_encrypted {
        let blk_len =
            u32::try_from(spec.compressed_bytes).map_err(|_| PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            })?;
        push_block_checked(
            blocks_out,
            CompressionBlock {
                rel_offset: 0,
                compressed_bytes: blk_len,
            },
            spec.blocks_total_max,
        )?;
        return Ok(spec.compressed_bytes);
    }
    let mut cur_rel: u64 = 0;
    for _ in 0..spec.block_count {
        let blk_len = take_u32_le(buf, WireContext::EncodedEntry)?;
        push_block_checked(
            blocks_out,
            CompressionBlock {
                rel_offset: cur_rel,
                compressed_bytes: blk_len,
            },
            spec.blocks_total_max,
        )?;
        let stride = if spec.is_encrypted {
            align_up_16(u64::from(blk_len)).ok_or(PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            })?
        } else {
            u64::from(blk_len)
        };
        cur_rel = cur_rel
            .checked_add(stride)
            .ok_or(PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            })?;
    }
    Ok(cur_rel)
}

#[inline]
fn push_block_checked(
    blocks_out: &mut Vec<CompressionBlock>,
    block: CompressionBlock,
    blocks_total_max: u32,
) -> Result<(), PakError> {
    if blocks_out.len() >= u32_to_usize(blocks_total_max) {
        let next_count = u64::from(blocks_total_max) + 1;
        return Err(PakError::LimitExceeded {
            kind: LimitKind::BlocksTotalCount,
            actual: next_count,
            max: u64::from(blocks_total_max),
        });
    }
    blocks_out.push(block);
    Ok(())
}

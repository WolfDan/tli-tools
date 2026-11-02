//! Access-temperature-grouped entry catalog (`EntryTable`) and `v8–v12` `.pak` index loaders.

use core::num::NonZeroU32;

use crate::crypto::{PakCipher, compute_sha1};
use crate::error::{EncryptedSectionKind, IndexSectionKind, LimitKind, PakError, WireContext};
use crate::io::PositionedFile;
use crate::path::{
    PathArena, PathArenaBuilder, PathLookupTable, compute_ue4_path_hash,
    normalize_join_dir_file_into, normalize_rel_path_into,
};
use crate::types::{
    CompressionMethod, ENTRIES_COUNT_HARD_MAX, EntryFlags, EntryId, EntryMeta,
    MOUNT_POINT_BYTES_MAX, OpenOptions, PATH_BYTES_MAX, PakDialect, PakLimits, PakVersion,
    PathHash, Sha1Hash, StreamingEntry,
};
use crate::wire::{
    CompressionBlock, DecodedEntry, MethodTable, ParsedFooter, RawPathHashRecord,
    RawSubIndexHeader, decode_encoded_entry, decode_unencoded_entry, inline_header_bytes, take,
    take_fstring_into, take_i32_le, take_slice, take_u32_le, take_u64_le,
};
use crate::{bounded_len_u32, u32_to_usize};

/// Compact 8-byte descriptor slicing into `EntryTable::blocks` for one entry.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct BlockRange {
    pub(crate) start_idx: u32,
    pub(crate) count: u16,
    pub(crate) method: CompressionMethod,
    pub(crate) flags: EntryFlags,
}
const _: () = assert!(core::mem::size_of::<BlockRange>() == 8);

/// Compact 40-byte point-lookup row storing pre-validated entry metadata and block slice span.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct EntryRecord {
    pub(crate) payload_offset: u64,
    pub(crate) compressed_bytes: u64,
    pub(crate) uncompressed_bytes: u64,
    pub(crate) compression_block_bytes: u32,
    pub(crate) block_range: BlockRange,
}
const _: () = assert!(core::mem::size_of::<EntryRecord>() == 40);

/// Immutable point-lookup entry catalog storing contiguous [`EntryRecord`] rows and flat block spans.
///
/// `header_offset` and `disk_span_bytes` are derived in `O(1)` inside [`EntryTable::meta`] rather
/// than stored as redundant `u64` fields, saving `16` bytes per entry across multi-archive VFS mounts.
#[derive(Debug)]
pub(crate) struct EntryTable {
    pub(crate) records: Box<[EntryRecord]>,
    pub(crate) blocks: Box<[CompressionBlock]>,
    dialect: PakDialect,
}

impl EntryTable {
    #[inline]
    #[must_use]
    pub(crate) const fn len(&self) -> u32 {
        bounded_len_u32(self.records.len())
    }

    #[inline]
    #[must_use]
    pub(crate) fn record(&self, id: EntryId) -> EntryRecord {
        self.records[u32_to_usize(id.index())]
    }

    #[inline]
    #[must_use]
    pub(crate) fn meta(&self, id: EntryId) -> EntryMeta {
        let rec = self.record(id);
        let range = rec.block_range;
        let entry_blocks = self.blocks_for(range);
        let header_offset = derive_header_offset(self.dialect, rec.payload_offset, range);
        let disk_span_bytes = derive_disk_span_bytes(range, rec.uncompressed_bytes, entry_blocks);
        assert!(header_offset < rec.payload_offset);
        assert!(rec.compressed_bytes <= disk_span_bytes);
        EntryMeta {
            header_offset,
            payload_offset: rec.payload_offset,
            compressed_bytes: rec.compressed_bytes,
            disk_span_bytes,
            uncompressed_bytes: rec.uncompressed_bytes,
            compression_block_bytes: rec.compression_block_bytes,
            block_count: range.count,
            compression_method: range.method,
            flags: range.flags,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn blocks_for(&self, range: BlockRange) -> &[CompressionBlock] {
        let start = u32_to_usize(range.start_idx);
        let end = start.strict_add(usize::from(range.count));
        &self.blocks[start..end]
    }
}

#[inline]
fn derive_header_offset(dialect: PakDialect, payload_offset: u64, range: BlockRange) -> u64 {
    let hdr_bytes = inline_header_bytes(
        dialect,
        range.method != CompressionMethod::None,
        range.count,
    );
    payload_offset.strict_sub(u64::from(hdr_bytes))
}

#[inline]
fn derive_disk_span_bytes(
    range: BlockRange,
    uncompressed_bytes: u64,
    entry_blocks: &[CompressionBlock],
) -> u64 {
    let is_encrypted = range.flags.contains(EntryFlags::ENCRYPTED);
    if range.method == CompressionMethod::None {
        return pad_encrypted_stride(uncompressed_bytes, is_encrypted);
    }
    match entry_blocks.last() {
        Some(last_blk) => {
            let stride = pad_encrypted_stride(u64::from(last_blk.compressed_bytes), is_encrypted);
            last_blk.rel_offset.strict_add(stride)
        }
        None => 0,
    }
}

#[inline]
const fn pad_encrypted_stride(raw_len: u64, is_encrypted: bool) -> u64 {
    if !is_encrypted {
        return raw_len;
    }
    let rem = raw_len & 15;
    if rem == 0 {
        raw_len
    } else {
        raw_len.strict_add(16_u64.strict_sub(rem))
    }
}

pub(crate) struct LoadedCatalog {
    pub(crate) mount_point: Box<str>,
    pub(crate) path_hash_seed: u64,
    pub(crate) entries: EntryTable,
    pub(crate) paths: PathArena,
    pub(crate) lookup: PathLookupTable,
}

pub(crate) struct LoadedStreamingCatalog {
    pub(crate) mount_point: Box<str>,
    pub(crate) entries: EntryTable,
}

struct EntryTableBuilder {
    records: Vec<EntryRecord>,
    blocks: Vec<CompressionBlock>,
    dialect: PakDialect,
}

impl EntryTableBuilder {
    fn with_capacity(entry_cap: u32, dialect: PakDialect) -> Self {
        let cap = u32_to_usize(entry_cap);
        Self {
            records: Vec::with_capacity(cap),
            blocks: Vec::with_capacity(cap),
            dialect,
        }
    }

    #[inline]
    const fn current_block_start(&self) -> u32 {
        bounded_len_u32(self.blocks.len())
    }

    fn validate_and_push(
        &mut self,
        block_start_idx: u32,
        entry: DecodedEntry,
        file_bytes: u64,
        limits: PakLimits,
        context: WireContext,
    ) -> Result<EntryId, PakError> {
        let idx = bounded_len_u32(self.records.len());
        if idx >= limits.entries_count_max() {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::EntriesCount,
                actual: u64::from(idx).strict_add(1),
                max: u64::from(limits.entries_count_max()),
            });
        }
        validate_entry_bounds(&entry, file_bytes, limits, context)?;
        let entry_blocks = &self.blocks[u32_to_usize(block_start_idx)..];
        validate_entry_blocks(&entry, entry_blocks, context)?;
        let range = BlockRange {
            start_idx: block_start_idx,
            count: entry.block_count,
            method: entry.method,
            flags: entry.flags,
        };
        assert_eq!(
            derive_header_offset(self.dialect, entry.payload_offset, range),
            entry.header_offset
        );
        assert_eq!(
            derive_disk_span_bytes(range, entry.uncompressed_bytes, entry_blocks),
            entry.disk_span_bytes
        );
        self.records.push(EntryRecord {
            payload_offset: entry.payload_offset,
            compressed_bytes: entry.compressed_bytes,
            uncompressed_bytes: entry.uncompressed_bytes,
            compression_block_bytes: entry.compression_block_bytes,
            block_range: range,
        });
        Ok(EntryId::new(idx))
    }

    fn finish(self) -> EntryTable {
        assert!(self.records.len() <= u32_to_usize(ENTRIES_COUNT_HARD_MAX));
        assert!(self.blocks.len() <= u32_to_usize(u32::MAX));
        EntryTable {
            records: self.records.into_boxed_slice(),
            blocks: self.blocks.into_boxed_slice(),
            dialect: self.dialect,
        }
    }
}

fn validate_entry_bounds(
    entry: &DecodedEntry,
    file_bytes: u64,
    limits: PakLimits,
    context: WireContext,
) -> Result<(), PakError> {
    if entry.uncompressed_bytes > limits.entry_uncompressed_bytes_max() {
        return Err(PakError::LimitExceeded {
            kind: LimitKind::EntryUncompressedBytes,
            actual: entry.uncompressed_bytes,
            max: limits.entry_uncompressed_bytes_max(),
        });
    }
    if entry.compressed_bytes > entry.disk_span_bytes {
        return Err(PakError::CorruptWireLayout { context });
    }
    let payload_end = entry
        .payload_offset
        .checked_add(entry.disk_span_bytes)
        .ok_or(PakError::OutOfBoundsRange {
            offset: entry.payload_offset,
            byte_len: entry.disk_span_bytes,
            container_bytes: file_bytes,
        })?;
    if payload_end > file_bytes {
        return Err(PakError::OutOfBoundsRange {
            offset: entry.payload_offset,
            byte_len: entry.disk_span_bytes,
            container_bytes: file_bytes,
        });
    }
    if entry.flags.contains(EntryFlags::ENCRYPTED) && !entry.disk_span_bytes.is_multiple_of(16) {
        return Err(PakError::UnalignedEncryptedSize {
            context: EncryptedSectionKind::EntryPayload,
            byte_len: entry.disk_span_bytes,
        });
    }
    Ok(())
}

fn validate_entry_blocks(
    entry: &DecodedEntry,
    blocks: &[CompressionBlock],
    context: WireContext,
) -> Result<(), PakError> {
    assert_eq!(blocks.len(), usize::from(entry.block_count));
    if entry.method == CompressionMethod::None {
        if entry.block_count != 0 {
            return Err(PakError::CorruptWireLayout { context });
        }
        return Ok(());
    }
    if entry.uncompressed_bytes == 0 {
        if entry.block_count != 0 {
            return Err(PakError::CorruptWireLayout { context });
        }
        return Ok(());
    }
    if entry.block_count == 0 {
        return Err(PakError::CorruptWireLayout { context });
    }
    if entry.block_count > 1 {
        if entry.compression_block_bytes == 0 {
            return Err(PakError::CorruptWireLayout { context });
        }
        let non_final = u64::from(entry.block_count.strict_sub(1));
        let covered = non_final
            .checked_mul(u64::from(entry.compression_block_bytes))
            .ok_or(PakError::CorruptWireLayout { context })?;
        if covered >= entry.uncompressed_bytes {
            return Err(PakError::CorruptWireLayout { context });
        }
        let final_bytes = entry.uncompressed_bytes.strict_sub(covered);
        if final_bytes > u64::from(entry.compression_block_bytes) {
            return Err(PakError::CorruptWireLayout { context });
        }
    }
    let mut prev_end: u64 = 0;
    for blk in blocks {
        if blk.rel_offset < prev_end {
            return Err(PakError::CorruptWireLayout { context });
        }
        let blk_end = blk
            .rel_offset
            .checked_add(u64::from(blk.compressed_bytes))
            .ok_or(PakError::CorruptWireLayout { context })?;
        if blk_end > entry.disk_span_bytes {
            return Err(PakError::CorruptWireLayout { context });
        }
        prev_end = blk_end;
    }
    Ok(())
}

struct IndexSectionSpec {
    offset: u64,
    size: u64,
    expected_hash: Sha1Hash,
    enc_kind: EncryptedSectionKind,
    hash_kind: IndexSectionKind,
}

fn read_index_section(
    file: &PositionedFile,
    footer: &ParsedFooter,
    cipher: Option<&PakCipher>,
    options: &OpenOptions,
    spec: IndexSectionSpec,
) -> Result<Vec<u8>, PakError> {
    let max_bytes = options.limits().index_bytes_max();
    let size_u32 = match u32::try_from(spec.size) {
        Ok(n) if n <= max_bytes => n,
        Ok(_) | Err(_) => {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::IndexBytes,
                actual: spec.size,
                max: u64::from(max_bytes),
            });
        }
    };
    let content_limit = file
        .file_bytes()
        .checked_sub(u64::from(footer.footer_bytes))
        .ok_or(PakError::OutOfBoundsRange {
            offset: spec.offset,
            byte_len: spec.size,
            container_bytes: file.file_bytes(),
        })?;
    let section_end = spec
        .offset
        .checked_add(spec.size)
        .ok_or(PakError::OutOfBoundsRange {
            offset: spec.offset,
            byte_len: spec.size,
            container_bytes: content_limit,
        })?;
    if section_end > content_limit {
        return Err(PakError::OutOfBoundsRange {
            offset: spec.offset,
            byte_len: spec.size,
            container_bytes: content_limit,
        });
    }
    if footer.is_index_encrypted && !spec.size.is_multiple_of(16) {
        return Err(PakError::UnalignedEncryptedSize {
            context: spec.enc_kind,
            byte_len: spec.size,
        });
    }
    let active_cipher = if footer.is_index_encrypted {
        Some(cipher.ok_or(PakError::EncryptionKeyRequired {
            guid: footer.encryption_key_guid,
        })?)
    } else {
        None
    };
    let mut buf = vec![0_u8; u32_to_usize(size_u32)];
    file.read_exact_at(spec.offset, &mut buf)?;
    if let Some(dec) = active_cipher {
        dec.decrypt_in_place(spec.enc_kind, &mut buf)?;
    }
    if options.verify_index_hashes() {
        let actual = compute_sha1(&buf);
        if actual != spec.expected_hash {
            return Err(PakError::IndexHashMismatch {
                section: spec.hash_kind,
                expected: spec.expected_hash,
                actual,
            });
        }
    }
    Ok(buf)
}

/// Loads and validates either the legacy (`v8–v9`) or modern split (`v10–v12`) index catalog.
pub(crate) fn load_archive_catalog(
    file: &PositionedFile,
    footer: &ParsedFooter,
    cipher: Option<&PakCipher>,
    options: &OpenOptions,
) -> Result<LoadedCatalog, PakError> {
    match footer.version {
        PakVersion::V8A | PakVersion::V8B | PakVersion::V9 => {
            load_legacy_index(file, footer, cipher, options)
        }
        PakVersion::V10 | PakVersion::V11 | PakVersion::V12 => {
            load_split_index(file, footer, cipher, options)
        }
    }
}

fn load_legacy_index(
    file: &PositionedFile,
    footer: &ParsedFooter,
    cipher: Option<&PakCipher>,
    options: &OpenOptions,
) -> Result<LoadedCatalog, PakError> {
    let raw = read_index_section(
        file,
        footer,
        cipher,
        options,
        IndexSectionSpec {
            offset: footer.index_offset,
            size: footer.index_size,
            expected_hash: footer.index_hash,
            enc_kind: EncryptedSectionKind::LegacyDirectoryIndex,
            hash_kind: IndexSectionKind::PrimaryIndex,
        },
    )?;
    let limits = options.limits();
    let mut cur = raw.as_slice();
    let mut str_buf = Vec::with_capacity(128);
    take_fstring_into(
        &mut cur,
        MOUNT_POINT_BYTES_MAX,
        LimitKind::MountPointBytes,
        WireContext::LegacyDirectoryIndex,
        &mut str_buf,
    )?;
    let mount_point = decode_validated_utf8(&str_buf, WireContext::LegacyDirectoryIndex)?
        .to_owned()
        .into_boxed_str();
    let entry_count = take_entry_count(&mut cur, limits, WireContext::LegacyDirectoryIndex)?;
    let (entries, paths, lookup) =
        parse_legacy_entries(&mut cur, entry_count, file.file_bytes(), footer, limits)?;
    Ok(LoadedCatalog {
        mount_point,
        path_hash_seed: 0,
        entries,
        paths,
        lookup,
    })
}

fn parse_legacy_entries(
    cur: &mut &[u8],
    entry_count: u32,
    file_bytes: u64,
    footer: &ParsedFooter,
    limits: PakLimits,
) -> Result<(EntryTable, PathArena, PathLookupTable), PakError> {
    let mut str_buf = Vec::with_capacity(128);
    let mut norm_buf = Vec::with_capacity(128);
    let mut table = EntryTableBuilder::with_capacity(entry_count, footer.dialect);
    let mut paths = PathArenaBuilder::with_capacity(entry_count);
    let mut lookup = PathLookupTable::with_entry_capacity(entry_count);
    for _ in 0..entry_count {
        take_fstring_into(
            cur,
            PATH_BYTES_MAX,
            LimitKind::PathBytes,
            WireContext::LegacyDirectoryIndex,
            &mut str_buf,
        )?;
        normalize_rel_path_into(&str_buf, &mut norm_buf)?;
        let rel_path = decode_validated_utf8(&norm_buf, WireContext::LegacyDirectoryIndex)?;
        let blk_start = table.current_block_start();
        let decoded = decode_unencoded_entry(
            cur,
            footer.dialect,
            &footer.methods,
            limits.blocks_total_count_max(),
            &mut table.blocks,
        )?;
        let id = table.validate_and_push(
            blk_start,
            decoded,
            file_bytes,
            limits,
            WireContext::LegacyDirectoryIndex,
        )?;
        paths.push_path(rel_path)?;
        let hash = compute_ue4_path_hash(rel_path, 0)?;
        lookup.insert(hash, id);
    }
    let finished_entries = table.finish();
    let finished_paths = paths.finish(finished_entries.len());
    Ok((finished_entries, finished_paths, lookup))
}

struct ParsedPrimaryIndex<'a> {
    mount_point: Box<str>,
    declared_entries: u32,
    path_hash_seed: u64,
    path_hash_hdr: Option<RawSubIndexHeader>,
    full_dir_hdr: Option<RawSubIndexHeader>,
    encoded_entries: &'a [u8],
    unencoded_list: UnencodedPool,
}

struct PredecodedUnencoded {
    entry: DecodedEntry,
    block_start: u32,
}

struct UnencodedPool {
    items: Vec<PredecodedUnencoded>,
    blocks: Vec<CompressionBlock>,
}

fn parse_primary_index<'a>(
    mut cur: &'a [u8],
    footer: &ParsedFooter,
    limits: PakLimits,
) -> Result<ParsedPrimaryIndex<'a>, PakError> {
    let mut str_buf = Vec::with_capacity(128);
    take_fstring_into(
        &mut cur,
        MOUNT_POINT_BYTES_MAX,
        LimitKind::MountPointBytes,
        WireContext::PrimaryIndex,
        &mut str_buf,
    )?;
    let mount_point = decode_validated_utf8(&str_buf, WireContext::PrimaryIndex)?
        .to_owned()
        .into_boxed_str();
    let declared_entries = take_entry_count(&mut cur, limits, WireContext::PrimaryIndex)?;
    let path_hash_seed = take_u64_le(&mut cur, WireContext::PrimaryIndex)?;
    let path_hash_hdr = take_optional_subindex(&mut cur)?;
    let full_dir_hdr = take_optional_subindex(&mut cur)?;
    let enc_bytes_i32 = take_i32_le(&mut cur, WireContext::PrimaryIndex)?;
    let enc_bytes_len = u32::try_from(enc_bytes_i32).map_err(|_| PakError::CorruptWireLayout {
        context: WireContext::PrimaryIndex,
    })?;
    let encoded_entries = take_slice(&mut cur, enc_bytes_len, WireContext::PrimaryIndex)?;
    let unenc_count = take_entry_count(&mut cur, limits, WireContext::PrimaryIndex)?;
    let unencoded_list = parse_unencoded_pool(
        &mut cur,
        unenc_count,
        footer.dialect,
        &footer.methods,
        limits.blocks_total_count_max(),
    )?;
    Ok(ParsedPrimaryIndex {
        mount_point,
        declared_entries,
        path_hash_seed,
        path_hash_hdr,
        full_dir_hdr,
        encoded_entries,
        unencoded_list,
    })
}

fn parse_unencoded_pool(
    cur: &mut &[u8],
    count: u32,
    dialect: PakDialect,
    methods: &MethodTable,
    blocks_total_max: u32,
) -> Result<UnencodedPool, PakError> {
    let mut items = Vec::with_capacity(u32_to_usize(count));
    let mut blocks = Vec::new();
    for _ in 0..count {
        let block_start = bounded_len_u32(blocks.len());
        let entry = decode_unencoded_entry(cur, dialect, methods, blocks_total_max, &mut blocks)?;
        items.push(PredecodedUnencoded { entry, block_start });
    }
    Ok(UnencodedPool { items, blocks })
}

fn take_optional_subindex(cur: &mut &[u8]) -> Result<Option<RawSubIndexHeader>, PakError> {
    match take_u32_le(cur, WireContext::PrimaryIndex)? {
        0 => Ok(None),
        1 => Ok(Some(*take::<RawSubIndexHeader>(
            cur,
            WireContext::PrimaryIndex,
        )?)),
        _ => Err(PakError::CorruptWireLayout {
            context: WireContext::PrimaryIndex,
        }),
    }
}

fn take_entry_count(
    cur: &mut &[u8],
    limits: PakLimits,
    context: WireContext,
) -> Result<u32, PakError> {
    let raw = take_i32_le(cur, context)?;
    let count = u32::try_from(raw).map_err(|_| PakError::CorruptWireLayout { context })?;
    if count > limits.entries_count_max() {
        return Err(PakError::LimitExceeded {
            kind: LimitKind::EntriesCount,
            actual: u64::from(count),
            max: u64::from(limits.entries_count_max()),
        });
    }
    Ok(count)
}

fn decode_validated_utf8(bytes: &[u8], context: WireContext) -> Result<&str, PakError> {
    core::str::from_utf8(bytes).map_err(|_| PakError::InvalidUtf8String { context })
}

fn load_split_index(
    file: &PositionedFile,
    footer: &ParsedFooter,
    cipher: Option<&PakCipher>,
    options: &OpenOptions,
) -> Result<LoadedCatalog, PakError> {
    let primary_raw = read_index_section(
        file,
        footer,
        cipher,
        options,
        IndexSectionSpec {
            offset: footer.index_offset,
            size: footer.index_size,
            expected_hash: footer.index_hash,
            enc_kind: EncryptedSectionKind::PrimaryIndex,
            hash_kind: IndexSectionKind::PrimaryIndex,
        },
    )?;
    let limits = options.limits();
    let primary = parse_primary_index(&primary_raw, footer, limits)?;
    let mut ctx = SplitBuildContext {
        env: SplitIndexEnv {
            file_bytes: file.file_bytes(),
            footer,
            limits,
            encoded_entries: primary.encoded_entries,
            unencoded_pool: &primary.unencoded_list,
        },
        seed: primary.path_hash_seed,
        table: EntryTableBuilder::with_capacity(primary.declared_entries, footer.dialect),
        paths: PathArenaBuilder::with_capacity(primary.declared_entries),
        entry_hashes: Vec::with_capacity(u32_to_usize(primary.declared_entries)),
        pending_hashes: Vec::with_capacity(u32_to_usize(primary.declared_entries)),
        loc_map: LocationMap::with_capacity(primary.declared_entries),
    };
    if let Some(fdi_hdr) = primary.full_dir_hdr {
        let fdi_raw = read_index_section(
            file,
            footer,
            cipher,
            options,
            IndexSectionSpec {
                offset: fdi_hdr.offset.get(),
                size: fdi_hdr.size.get(),
                expected_hash: Sha1Hash::from_bytes(fdi_hdr.hash),
                enc_kind: EncryptedSectionKind::FullDirectoryIndex,
                hash_kind: IndexSectionKind::FullDirectoryIndex,
            },
        )?;
        populate_from_full_directory(&fdi_raw, &mut ctx)?;
    }
    if let Some(phi_hdr) = primary.path_hash_hdr {
        let phi_raw = read_index_section(
            file,
            footer,
            cipher,
            options,
            IndexSectionSpec {
                offset: phi_hdr.offset.get(),
                size: phi_hdr.size.get(),
                expected_hash: Sha1Hash::from_bytes(phi_hdr.hash),
                enc_kind: EncryptedSectionKind::PathHashIndex,
                hash_kind: IndexSectionKind::PathHashIndex,
            },
        )?;
        populate_from_path_hash_index(&phi_raw, &mut ctx)?;
    }
    Ok(finish_split_catalog(
        primary.mount_point,
        primary.path_hash_seed,
        ctx,
    ))
}

fn finish_split_catalog(
    mount_point: Box<str>,
    path_hash_seed: u64,
    ctx: SplitBuildContext<'_>,
) -> LoadedCatalog {
    let finished_entries = ctx.table.finish();
    let finished_paths = ctx.paths.finish(finished_entries.len());
    assert_eq!(ctx.entry_hashes.len(), u32_to_usize(finished_entries.len()));
    let pending_count = bounded_len_u32(ctx.pending_hashes.len());
    let mut lookup = PathLookupTable::with_entry_capacity(pending_count);
    for &(hash, id) in &ctx.pending_hashes {
        lookup.insert(hash, id);
    }
    LoadedCatalog {
        mount_point,
        path_hash_seed,
        entries: finished_entries,
        paths: finished_paths,
        lookup,
    }
}

#[derive(Clone, Copy)]
struct SplitIndexEnv<'a> {
    file_bytes: u64,
    footer: &'a ParsedFooter,
    limits: PakLimits,
    encoded_entries: &'a [u8],
    unencoded_pool: &'a UnencodedPool,
}

struct SplitBuildContext<'a> {
    env: SplitIndexEnv<'a>,
    seed: u64,
    table: EntryTableBuilder,
    paths: PathArenaBuilder,
    entry_hashes: Vec<PathHash>,
    pending_hashes: Vec<(PathHash, EntryId)>,
    loc_map: LocationMap,
}

impl SplitBuildContext<'_> {
    fn push_pending_hash(&mut self, hash: PathHash, id: EntryId) -> Result<(), PakError> {
        let hash_max = self
            .env
            .limits
            .entries_count_max()
            .saturating_mul(2)
            .min(ENTRIES_COUNT_HARD_MAX);
        let cur_pending = bounded_len_u32(self.pending_hashes.len());
        if cur_pending >= hash_max {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::EntriesCount,
                actual: u64::from(cur_pending).strict_add(1),
                max: u64::from(hash_max),
            });
        }
        self.pending_hashes.push((hash, id));
        Ok(())
    }

    fn decode_location(
        &mut self,
        location: i32,
        context: WireContext,
    ) -> Result<EntryId, PakError> {
        decode_split_location(&mut self.table, self.env, location, context)
    }
}

fn decode_split_location(
    table: &mut EntryTableBuilder,
    env: SplitIndexEnv<'_>,
    location: i32,
    context: WireContext,
) -> Result<EntryId, PakError> {
    let blk_start = table.current_block_start();
    let decoded = if location >= 0 {
        let offset =
            u32::try_from(location).map_err(|_| PakError::CorruptWireLayout { context })?;
        let encoded_tail =
            env.encoded_entries
                .get(u32_to_usize(offset)..)
                .ok_or(PakError::OutOfBoundsRange {
                    offset: u64::from(offset),
                    byte_len: 4,
                    container_bytes: u64::try_from(env.encoded_entries.len())
                        .map_err(|_| PakError::CorruptWireLayout { context })?,
                })?;
        decode_encoded_entry(
            encoded_tail,
            env.footer.dialect,
            &env.footer.methods,
            env.limits.blocks_total_count_max(),
            &mut table.blocks,
        )?
    } else {
        let unenc_idx_i64 = (-1_i64)
            .checked_sub(i64::from(location))
            .ok_or(PakError::CorruptWireLayout { context })?;
        let unenc_idx =
            u32::try_from(unenc_idx_i64).map_err(|_| PakError::CorruptWireLayout { context })?;
        let item = env
            .unencoded_pool
            .items
            .get(u32_to_usize(unenc_idx))
            .ok_or(PakError::CorruptWireLayout { context })?;
        let src_start = u32_to_usize(item.block_start);
        let src_end = src_start.strict_add(usize::from(item.entry.block_count));
        let needed = table
            .blocks
            .len()
            .strict_add(usize::from(item.entry.block_count));
        if needed > u32_to_usize(env.limits.blocks_total_count_max()) {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::BlocksTotalCount,
                actual: u64::from(env.limits.blocks_total_count_max()).strict_add(1),
                max: u64::from(env.limits.blocks_total_count_max()),
            });
        }
        table
            .blocks
            .extend_from_slice(&env.unencoded_pool.blocks[src_start..src_end]);
        item.entry
    };
    table.validate_and_push(blk_start, decoded, env.file_bytes, env.limits, context)
}

fn populate_from_full_directory(
    mut cur: &[u8],
    ctx: &mut SplitBuildContext<'_>,
) -> Result<(), PakError> {
    let dir_count = take_u32_le(&mut cur, WireContext::FullDirectoryIndex)?;
    let dir_max = ctx.env.limits.entries_count_max().saturating_add(65_536);
    if dir_count > dir_max {
        return Err(PakError::LimitExceeded {
            kind: LimitKind::EntriesCount,
            actual: u64::from(dir_count),
            max: u64::from(dir_max),
        });
    }
    let mut dir_buf = Vec::with_capacity(128);
    let mut file_buf = Vec::with_capacity(64);
    let mut norm_buf = Vec::with_capacity(192);
    for _ in 0..dir_count {
        take_fstring_into(
            &mut cur,
            PATH_BYTES_MAX,
            LimitKind::PathBytes,
            WireContext::FullDirectoryIndex,
            &mut dir_buf,
        )?;
        let file_count = take_u32_le(&mut cur, WireContext::FullDirectoryIndex)?;
        if file_count > ctx.env.limits.entries_count_max() {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::EntriesCount,
                actual: u64::from(file_count),
                max: u64::from(ctx.env.limits.entries_count_max()),
            });
        }
        for _ in 0..file_count {
            take_fstring_into(
                &mut cur,
                PATH_BYTES_MAX,
                LimitKind::PathBytes,
                WireContext::FullDirectoryIndex,
                &mut file_buf,
            )?;
            let location = take_i32_le(&mut cur, WireContext::FullDirectoryIndex)?;
            normalize_join_dir_file_into(&dir_buf, &file_buf, &mut norm_buf)?;
            let rel_path = decode_validated_utf8(&norm_buf, WireContext::FullDirectoryIndex)?;
            let id = ctx.decode_location(location, WireContext::FullDirectoryIndex)?;
            let hash = compute_ue4_path_hash(rel_path, ctx.seed)?;
            ctx.paths.push_path(rel_path)?;
            ctx.entry_hashes.push(hash);
            ctx.loc_map.insert(location, id, ctx.env.limits)?;
            ctx.push_pending_hash(hash, id)?;
        }
    }
    Ok(())
}

fn populate_from_path_hash_index(
    mut cur: &[u8],
    ctx: &mut SplitBuildContext<'_>,
) -> Result<(), PakError> {
    let count = take_u32_le(&mut cur, WireContext::PathHashIndex)?;
    if count > ctx.env.limits.entries_count_max() {
        return Err(PakError::LimitExceeded {
            kind: LimitKind::EntriesCount,
            actual: u64::from(count),
            max: u64::from(ctx.env.limits.entries_count_max()),
        });
    }
    for _ in 0..count {
        let rec = take::<RawPathHashRecord>(&mut cur, WireContext::PathHashIndex)?;
        let hash = PathHash::from_raw(rec.hash.get());
        let location = rec.location.get();
        if let Some(existing_id) = ctx.loc_map.get(location) {
            if ctx.entry_hashes[u32_to_usize(existing_id.index())] != hash {
                ctx.push_pending_hash(hash, existing_id)?;
            }
        } else {
            let new_id = ctx.decode_location(location, WireContext::PathHashIndex)?;
            ctx.paths.push_missing();
            ctx.entry_hashes.push(hash);
            ctx.loc_map.insert(location, new_id, ctx.env.limits)?;
            ctx.push_pending_hash(hash, new_id)?;
        }
    }
    Ok(())
}

/// Streams all entries in `file` to `on_entry` without allocating `PathArena` or `PathLookupTable`.
pub(crate) fn load_streaming_catalog<E, F>(
    file: &PositionedFile,
    footer: &ParsedFooter,
    cipher: Option<&PakCipher>,
    options: &OpenOptions,
    mut on_entry: F,
) -> Result<LoadedStreamingCatalog, E>
where
    E: From<PakError>,
    F: FnMut(&str, StreamingEntry<'_>) -> Result<(), E>,
{
    match footer.version {
        PakVersion::V8A | PakVersion::V8B | PakVersion::V9 => {
            load_legacy_index_streaming(file, footer, cipher, options, &mut on_entry)
        }
        PakVersion::V10 | PakVersion::V11 | PakVersion::V12 => {
            load_split_index_streaming(file, footer, cipher, options, &mut on_entry)
        }
    }
}

fn load_legacy_index_streaming<E, F>(
    file: &PositionedFile,
    footer: &ParsedFooter,
    cipher: Option<&PakCipher>,
    options: &OpenOptions,
    on_entry: &mut F,
) -> Result<LoadedStreamingCatalog, E>
where
    E: From<PakError>,
    F: FnMut(&str, StreamingEntry<'_>) -> Result<(), E>,
{
    let raw = read_index_section(
        file,
        footer,
        cipher,
        options,
        IndexSectionSpec {
            offset: footer.index_offset,
            size: footer.index_size,
            expected_hash: footer.index_hash,
            enc_kind: EncryptedSectionKind::LegacyDirectoryIndex,
            hash_kind: IndexSectionKind::PrimaryIndex,
        },
    )?;
    let limits = options.limits();
    let mut cur = raw.as_slice();
    let mut str_buf = Vec::with_capacity(128);
    take_fstring_into(
        &mut cur,
        MOUNT_POINT_BYTES_MAX,
        LimitKind::MountPointBytes,
        WireContext::LegacyDirectoryIndex,
        &mut str_buf,
    )?;
    let mount_point = decode_validated_utf8(&str_buf, WireContext::LegacyDirectoryIndex)?
        .to_owned()
        .into_boxed_str();
    let entry_count = take_entry_count(&mut cur, limits, WireContext::LegacyDirectoryIndex)?;
    let entries = stream_legacy_entries(
        &mut cur,
        &mount_point,
        entry_count,
        file.file_bytes(),
        footer,
        limits,
        on_entry,
    )?;
    Ok(LoadedStreamingCatalog {
        mount_point,
        entries,
    })
}

fn stream_legacy_entries<E, F>(
    cur: &mut &[u8],
    mount_point: &str,
    entry_count: u32,
    file_bytes: u64,
    footer: &ParsedFooter,
    limits: PakLimits,
    on_entry: &mut F,
) -> Result<EntryTable, E>
where
    E: From<PakError>,
    F: FnMut(&str, StreamingEntry<'_>) -> Result<(), E>,
{
    let mut str_buf = Vec::with_capacity(128);
    let mut table = EntryTableBuilder::with_capacity(entry_count, footer.dialect);
    for _ in 0..entry_count {
        take_fstring_into(
            cur,
            PATH_BYTES_MAX,
            LimitKind::PathBytes,
            WireContext::LegacyDirectoryIndex,
            &mut str_buf,
        )?;
        let rel_path = decode_validated_utf8(&str_buf, WireContext::LegacyDirectoryIndex)?;
        let blk_start = table.current_block_start();
        let decoded = decode_unencoded_entry(
            cur,
            footer.dialect,
            &footer.methods,
            limits.blocks_total_count_max(),
            &mut table.blocks,
        )?;
        let id = table.validate_and_push(
            blk_start,
            decoded,
            file_bytes,
            limits,
            WireContext::LegacyDirectoryIndex,
        )?;
        on_entry(
            mount_point,
            StreamingEntry::Named {
                id,
                dir: "",
                file: rel_path,
            },
        )?;
    }
    Ok(table.finish())
}

struct StreamingSplitContext<'a> {
    env: SplitIndexEnv<'a>,
    table: EntryTableBuilder,
    loc_map: LocationMap,
}

impl StreamingSplitContext<'_> {
    fn decode_location(
        &mut self,
        location: i32,
        context: WireContext,
    ) -> Result<EntryId, PakError> {
        decode_split_location(&mut self.table, self.env, location, context)
    }
}

fn load_split_index_streaming<E, F>(
    file: &PositionedFile,
    footer: &ParsedFooter,
    cipher: Option<&PakCipher>,
    options: &OpenOptions,
    on_entry: &mut F,
) -> Result<LoadedStreamingCatalog, E>
where
    E: From<PakError>,
    F: FnMut(&str, StreamingEntry<'_>) -> Result<(), E>,
{
    let primary_raw = read_index_section(
        file,
        footer,
        cipher,
        options,
        IndexSectionSpec {
            offset: footer.index_offset,
            size: footer.index_size,
            expected_hash: footer.index_hash,
            enc_kind: EncryptedSectionKind::PrimaryIndex,
            hash_kind: IndexSectionKind::PrimaryIndex,
        },
    )?;
    let limits = options.limits();
    let primary = parse_primary_index(&primary_raw, footer, limits)?;
    let mount_point_str = &*primary.mount_point;
    let mut ctx = StreamingSplitContext {
        env: SplitIndexEnv {
            file_bytes: file.file_bytes(),
            footer,
            limits,
            encoded_entries: primary.encoded_entries,
            unencoded_pool: &primary.unencoded_list,
        },
        table: EntryTableBuilder::with_capacity(primary.declared_entries, footer.dialect),
        loc_map: LocationMap::with_capacity(primary.declared_entries),
    };
    if let Some(fdi_hdr) = primary.full_dir_hdr {
        let fdi_raw = read_index_section(
            file,
            footer,
            cipher,
            options,
            IndexSectionSpec {
                offset: fdi_hdr.offset.get(),
                size: fdi_hdr.size.get(),
                expected_hash: Sha1Hash::from_bytes(fdi_hdr.hash),
                enc_kind: EncryptedSectionKind::FullDirectoryIndex,
                hash_kind: IndexSectionKind::FullDirectoryIndex,
            },
        )?;
        stream_full_directory(&fdi_raw, mount_point_str, &mut ctx, on_entry)?;
    }
    if let Some(phi_hdr) = primary.path_hash_hdr {
        let phi_raw = read_index_section(
            file,
            footer,
            cipher,
            options,
            IndexSectionSpec {
                offset: phi_hdr.offset.get(),
                size: phi_hdr.size.get(),
                expected_hash: Sha1Hash::from_bytes(phi_hdr.hash),
                enc_kind: EncryptedSectionKind::PathHashIndex,
                hash_kind: IndexSectionKind::PathHashIndex,
            },
        )?;
        stream_path_hash_pruned(&phi_raw, mount_point_str, &mut ctx, on_entry)?;
    }
    Ok(LoadedStreamingCatalog {
        mount_point: primary.mount_point,
        entries: ctx.table.finish(),
    })
}

fn stream_full_directory<E, F>(
    mut cur: &[u8],
    mount_point: &str,
    ctx: &mut StreamingSplitContext<'_>,
    on_entry: &mut F,
) -> Result<(), E>
where
    E: From<PakError>,
    F: FnMut(&str, StreamingEntry<'_>) -> Result<(), E>,
{
    let dir_count = take_u32_le(&mut cur, WireContext::FullDirectoryIndex)?;
    let dir_max = ctx.env.limits.entries_count_max().saturating_add(65_536);
    if dir_count > dir_max {
        return Err(E::from(PakError::LimitExceeded {
            kind: LimitKind::EntriesCount,
            actual: u64::from(dir_count),
            max: u64::from(dir_max),
        }));
    }
    let mut dir_buf = Vec::with_capacity(128);
    let mut file_buf = Vec::with_capacity(64);
    for _ in 0..dir_count {
        take_fstring_into(
            &mut cur,
            PATH_BYTES_MAX,
            LimitKind::PathBytes,
            WireContext::FullDirectoryIndex,
            &mut dir_buf,
        )?;
        let dir_str = decode_validated_utf8(&dir_buf, WireContext::FullDirectoryIndex)?;
        let file_count = take_u32_le(&mut cur, WireContext::FullDirectoryIndex)?;
        if file_count > ctx.env.limits.entries_count_max() {
            return Err(E::from(PakError::LimitExceeded {
                kind: LimitKind::EntriesCount,
                actual: u64::from(file_count),
                max: u64::from(ctx.env.limits.entries_count_max()),
            }));
        }
        for _ in 0..file_count {
            take_fstring_into(
                &mut cur,
                PATH_BYTES_MAX,
                LimitKind::PathBytes,
                WireContext::FullDirectoryIndex,
                &mut file_buf,
            )?;
            let location = take_i32_le(&mut cur, WireContext::FullDirectoryIndex)?;
            let file_str = decode_validated_utf8(&file_buf, WireContext::FullDirectoryIndex)?;
            validate_joined_rel_len(dir_str, file_str)?;
            let id = ctx.decode_location(location, WireContext::FullDirectoryIndex)?;
            ctx.loc_map.insert(location, id, ctx.env.limits)?;
            on_entry(
                mount_point,
                StreamingEntry::Named {
                    id,
                    dir: dir_str,
                    file: file_str,
                },
            )?;
        }
    }
    Ok(())
}

fn validate_joined_rel_len(dir_str: &str, file_str: &str) -> Result<(), PakError> {
    let dir_trim = dir_str.trim_start_matches(['/', '\\']);
    let file_trim = file_str.trim_start_matches(['/', '\\']);
    let needs_sep =
        !dir_trim.is_empty() && !file_trim.is_empty() && !dir_trim.ends_with(['/', '\\']);
    let joined_len = dir_trim
        .len()
        .checked_add(usize::from(needs_sep))
        .and_then(|n| n.checked_add(file_trim.len()))
        .ok_or(PakError::LimitExceeded {
            kind: LimitKind::PathBytes,
            actual: u64::MAX,
            max: u64::from(PATH_BYTES_MAX),
        })?;
    if joined_len > u32_to_usize(PATH_BYTES_MAX) {
        let actual = u64::try_from(joined_len).map_err(|_| PakError::LimitExceeded {
            kind: LimitKind::PathBytes,
            actual: u64::MAX,
            max: u64::from(PATH_BYTES_MAX),
        })?;
        return Err(PakError::LimitExceeded {
            kind: LimitKind::PathBytes,
            actual,
            max: u64::from(PATH_BYTES_MAX),
        });
    }
    Ok(())
}

fn stream_path_hash_pruned<E, F>(
    mut cur: &[u8],
    mount_point: &str,
    ctx: &mut StreamingSplitContext<'_>,
    on_entry: &mut F,
) -> Result<(), E>
where
    E: From<PakError>,
    F: FnMut(&str, StreamingEntry<'_>) -> Result<(), E>,
{
    let count = take_u32_le(&mut cur, WireContext::PathHashIndex)?;
    if count > ctx.env.limits.entries_count_max() {
        return Err(E::from(PakError::LimitExceeded {
            kind: LimitKind::EntriesCount,
            actual: u64::from(count),
            max: u64::from(ctx.env.limits.entries_count_max()),
        }));
    }
    for _ in 0..count {
        let rec = take::<RawPathHashRecord>(&mut cur, WireContext::PathHashIndex)?;
        let location = rec.location.get();
        if ctx.loc_map.get(location).is_none() {
            let new_id = ctx.decode_location(location, WireContext::PathHashIndex)?;
            ctx.loc_map.insert(location, new_id, ctx.env.limits)?;
            on_entry(mount_point, StreamingEntry::Pruned { id: new_id })?;
        }
    }
    Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct OccupiedLocation {
    entry_plus_one: NonZeroU32,
    location: i32,
}
const _: () = assert!(core::mem::size_of::<Option<OccupiedLocation>>() == 8);

impl OccupiedLocation {
    #[inline]
    #[must_use]
    const fn new(location: i32, entry: EntryId) -> Self {
        let idx = entry.index();
        assert!(idx < u32::MAX);
        Self {
            entry_plus_one: NonZeroU32::MIN.saturating_add(idx),
            location,
        }
    }

    #[inline]
    #[must_use]
    const fn entry(self) -> EntryId {
        EntryId::new(self.entry_plus_one.get().strict_sub(1))
    }
}

/// Temporary open-addressed `FPakEntryLocation -> EntryId` table used only during split-index open.
struct LocationMap {
    buckets: Box<[Option<OccupiedLocation>]>,
    occupied_count: u32,
}

impl LocationMap {
    fn with_capacity(declared_entries: u32) -> Self {
        let cap_entries = declared_entries
            .saturating_add(16)
            .min(ENTRIES_COUNT_HARD_MAX);
        let min_slots = (cap_entries.saturating_mul(2)).max(4);
        let table_len = min_slots.next_power_of_two();
        Self {
            buckets: vec![None; u32_to_usize(table_len)].into_boxed_slice(),
            occupied_count: 0,
        }
    }

    #[inline]
    const fn table_len(&self) -> u32 {
        bounded_len_u32(self.buckets.len())
    }

    #[inline]
    const fn mask(&self) -> u32 {
        self.table_len().strict_sub(1)
    }

    #[inline]
    fn start_slot(&self, location: i32) -> u32 {
        let bits = location.cast_unsigned();
        let mixed = u64::from(bits).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        (((mixed ^ (mixed >> 32)) & 0xFFFF_FFFF) as u32) & self.mask()
    }

    fn grow(&mut self, limits: PakLimits) -> Result<(), PakError> {
        let cur_len = self.table_len();
        let max_slots = (limits
            .entries_count_max()
            .min(ENTRIES_COUNT_HARD_MAX)
            .saturating_mul(2))
        .next_power_of_two();
        if cur_len >= max_slots {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::EntriesCount,
                actual: u64::from(self.occupied_count).strict_add(1),
                max: u64::from(limits.entries_count_max()),
            });
        }
        let next_len = cur_len.strict_mul(2);
        let old = core::mem::replace(
            &mut self.buckets,
            vec![None; u32_to_usize(next_len)].into_boxed_slice(),
        );
        self.occupied_count = 0;
        for &bucket in &old {
            if let Some(occ) = bucket {
                self.insert_no_grow(occ.location, occ.entry());
            }
        }
        Ok(())
    }

    fn insert(&mut self, location: i32, entry: EntryId, limits: PakLimits) -> Result<(), PakError> {
        let table_len = self.table_len();
        if self.occupied_count.saturating_mul(2) >= table_len {
            self.grow(limits)?;
        }
        self.insert_no_grow(location, entry);
        Ok(())
    }

    fn insert_no_grow(&mut self, location: i32, entry: EntryId) {
        let table_len = self.table_len();
        assert!(self.occupied_count < table_len);
        let mask = self.mask();
        let start = self.start_slot(location);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let idx = u32_to_usize(start.wrapping_add(step) & mask);
            match self.buckets[idx] {
                None => {
                    self.buckets[idx] = Some(OccupiedLocation::new(location, entry));
                    self.occupied_count = self.occupied_count.strict_add(1);
                    return;
                }
                Some(occ) => {
                    if occ.location == location {
                        return;
                    }
                    probed = probed.strict_add(1);
                }
            }
        }
        assert!(probed < table_len);
    }

    fn get(&self, location: i32) -> Option<EntryId> {
        let table_len = self.table_len();
        let mask = self.mask();
        let start = self.start_slot(location);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let idx = u32_to_usize(start.wrapping_add(step) & mask);
            let occ = self.buckets[idx]?;
            if occ.location == location {
                return Some(occ.entry());
            }
            probed = probed.strict_add(1);
        }
        assert!(probed < table_len);
        None
    }
}

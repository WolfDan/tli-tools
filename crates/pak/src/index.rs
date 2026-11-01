//! Struct-of-Arrays (SoA) entry catalog and `v8–v12` `.pak` index loaders.

use crate::crypto::{PakCipher, compute_sha1};
use crate::error::{EncryptedSectionKind, IndexSectionKind, LimitKind, PakError, WireContext};
use crate::io::PositionedFile;
use crate::low_u32;
use crate::path::{
    PathArena, PathArenaBuilder, PathLookupTable, compute_ue4_path_hash,
    normalize_join_dir_file_into, normalize_rel_path_into,
};
use crate::types::{
    CompressionMethod, ENTRIES_COUNT_HARD_MAX, EntryFlags, EntryId, EntryMeta,
    MOUNT_POINT_BYTES_MAX, OpenOptions, PATH_BYTES_MAX, PakDialect, PakLimits, PakVersion,
    PathHash, Sha1Hash,
};
use crate::u32_to_usize;
use crate::wire::{
    CompressionBlock, DecodedEntry, MethodTable, ParsedFooter, RawPathHashRecord,
    RawSubIndexHeader, decode_encoded_entry, decode_unencoded_entry, take, take_fstring_into,
    take_i32_le, take_slice, take_u32_le, take_u64_le,
};

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

/// Immutable Struct-of-Arrays catalog storing pre-validated entry metadata and flat block spans.
#[derive(Debug)]
pub(crate) struct EntryTable {
    pub(crate) header_offsets: Box<[u64]>,
    pub(crate) payload_offsets: Box<[u64]>,
    pub(crate) compressed_bytes: Box<[u64]>,
    pub(crate) disk_span_bytes: Box<[u64]>,
    pub(crate) uncompressed_bytes: Box<[u64]>,
    pub(crate) compression_block_bytes: Box<[u32]>,
    pub(crate) block_ranges: Box<[BlockRange]>,
    pub(crate) blocks: Box<[CompressionBlock]>,
    entry_count: u32,
}

impl EntryTable {
    #[inline]
    #[must_use]
    pub(crate) const fn len(&self) -> u32 {
        self.entry_count
    }

    #[inline]
    #[must_use]
    pub(crate) fn meta(&self, id: EntryId) -> EntryMeta {
        let idx = u32_to_usize(id.index());
        let range = self.block_ranges[idx];
        EntryMeta {
            header_offset: self.header_offsets[idx],
            payload_offset: self.payload_offsets[idx],
            compressed_bytes: self.compressed_bytes[idx],
            disk_span_bytes: self.disk_span_bytes[idx],
            uncompressed_bytes: self.uncompressed_bytes[idx],
            compression_block_bytes: self.compression_block_bytes[idx],
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

pub(crate) struct LoadedCatalog {
    pub(crate) mount_point: Box<str>,
    pub(crate) path_hash_seed: u64,
    pub(crate) entries: EntryTable,
    pub(crate) paths: PathArena,
    pub(crate) lookup: PathLookupTable,
}

struct EntryTableBuilder {
    header_offsets: Vec<u64>,
    payload_offsets: Vec<u64>,
    compressed_bytes: Vec<u64>,
    disk_span_bytes: Vec<u64>,
    uncompressed_bytes: Vec<u64>,
    compression_block_bytes: Vec<u32>,
    block_ranges: Vec<BlockRange>,
    blocks: Vec<CompressionBlock>,
    entry_count: u32,
}

impl EntryTableBuilder {
    fn with_capacity(entry_cap: u32) -> Self {
        let cap = u32_to_usize(entry_cap);
        Self {
            header_offsets: Vec::with_capacity(cap),
            payload_offsets: Vec::with_capacity(cap),
            compressed_bytes: Vec::with_capacity(cap),
            disk_span_bytes: Vec::with_capacity(cap),
            uncompressed_bytes: Vec::with_capacity(cap),
            compression_block_bytes: Vec::with_capacity(cap),
            block_ranges: Vec::with_capacity(cap),
            blocks: Vec::with_capacity(cap),
            entry_count: 0,
        }
    }

    fn current_block_start(&self) -> Result<u32, PakError> {
        u32::try_from(self.blocks.len()).map_err(|_| PakError::LimitExceeded {
            kind: LimitKind::BlocksTotalCount,
            actual: u64::from(u32::MAX) + 1,
            max: u64::from(u32::MAX),
        })
    }

    fn validate_and_push(
        &mut self,
        block_start_idx: u32,
        entry: DecodedEntry,
        file_bytes: u64,
        limits: PakLimits,
        context: WireContext,
    ) -> Result<EntryId, PakError> {
        let idx = self.entry_count;
        if idx >= limits.entries_count_max() {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::EntriesCount,
                actual: u64::from(idx) + 1,
                max: u64::from(limits.entries_count_max()),
            });
        }
        validate_entry_bounds(&entry, file_bytes, limits, context)?;
        validate_entry_blocks(
            &entry,
            &self.blocks[u32_to_usize(block_start_idx)..],
            context,
        )?;
        self.header_offsets.push(entry.header_offset);
        self.payload_offsets.push(entry.payload_offset);
        self.compressed_bytes.push(entry.compressed_bytes);
        self.disk_span_bytes.push(entry.disk_span_bytes);
        self.uncompressed_bytes.push(entry.uncompressed_bytes);
        self.compression_block_bytes
            .push(entry.compression_block_bytes);
        self.block_ranges.push(BlockRange {
            start_idx: block_start_idx,
            count: entry.block_count,
            method: entry.method,
            flags: entry.flags,
        });
        self.entry_count = self.entry_count.strict_add(1);
        Ok(EntryId::new(idx))
    }

    fn finish(self) -> EntryTable {
        let count_usize = u32_to_usize(self.entry_count);
        assert_eq!(self.header_offsets.len(), count_usize);
        assert_eq!(self.payload_offsets.len(), count_usize);
        assert_eq!(self.compressed_bytes.len(), count_usize);
        assert_eq!(self.disk_span_bytes.len(), count_usize);
        assert_eq!(self.uncompressed_bytes.len(), count_usize);
        assert_eq!(self.compression_block_bytes.len(), count_usize);
        assert_eq!(self.block_ranges.len(), count_usize);
        EntryTable {
            header_offsets: self.header_offsets.into_boxed_slice(),
            payload_offsets: self.payload_offsets.into_boxed_slice(),
            compressed_bytes: self.compressed_bytes.into_boxed_slice(),
            disk_span_bytes: self.disk_span_bytes.into_boxed_slice(),
            uncompressed_bytes: self.uncompressed_bytes.into_boxed_slice(),
            compression_block_bytes: self.compression_block_bytes.into_boxed_slice(),
            block_ranges: self.block_ranges.into_boxed_slice(),
            blocks: self.blocks.into_boxed_slice(),
            entry_count: self.entry_count,
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
        let non_final = u64::from(entry.block_count - 1);
        let covered = non_final
            .checked_mul(u64::from(entry.compression_block_bytes))
            .ok_or(PakError::CorruptWireLayout { context })?;
        if covered >= entry.uncompressed_bytes {
            return Err(PakError::CorruptWireLayout { context });
        }
        let final_bytes = entry.uncompressed_bytes - covered;
        if final_bytes > u64::from(entry.compression_block_bytes) {
            return Err(PakError::CorruptWireLayout { context });
        }
    }
    for blk in blocks {
        let blk_end = blk
            .rel_offset
            .checked_add(u64::from(blk.compressed_bytes))
            .ok_or(PakError::CorruptWireLayout { context })?;
        if blk_end > entry.disk_span_bytes {
            return Err(PakError::CorruptWireLayout { context });
        }
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
    if spec.size > u64::from(max_bytes) {
        return Err(PakError::LimitExceeded {
            kind: LimitKind::IndexBytes,
            actual: spec.size,
            max: u64::from(max_bytes),
        });
    }
    let size_u32 = u32::try_from(spec.size).map_err(|_| PakError::LimitExceeded {
        kind: LimitKind::IndexBytes,
        actual: spec.size,
        max: u64::from(max_bytes),
    })?;
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
    let mut table = EntryTableBuilder::with_capacity(entry_count);
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
        let blk_start = table.current_block_start()?;
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
        let block_start = u32::try_from(blocks.len()).map_err(|_| PakError::LimitExceeded {
            kind: LimitKind::BlocksTotalCount,
            actual: u64::from(u32::MAX) + 1,
            max: u64::from(blocks_total_max),
        })?;
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
        file,
        footer,
        limits,
        seed: primary.path_hash_seed,
        encoded_entries: primary.encoded_entries,
        unencoded_pool: &primary.unencoded_list,
        table: EntryTableBuilder::with_capacity(primary.declared_entries),
        paths: PathArenaBuilder::with_capacity(primary.declared_entries),
        entry_hashes: Vec::with_capacity(u32_to_usize(primary.declared_entries)),
        pending_hashes: Vec::with_capacity(u32_to_usize(primary.declared_entries)),
        pending_hash_count: 0,
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
    assert_eq!(
        ctx.pending_hashes.len(),
        u32_to_usize(ctx.pending_hash_count)
    );
    let mut lookup = PathLookupTable::with_entry_capacity(ctx.pending_hash_count);
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

struct SplitBuildContext<'a> {
    file: &'a PositionedFile,
    footer: &'a ParsedFooter,
    limits: PakLimits,
    seed: u64,
    encoded_entries: &'a [u8],
    unencoded_pool: &'a UnencodedPool,
    table: EntryTableBuilder,
    paths: PathArenaBuilder,
    entry_hashes: Vec<PathHash>,
    pending_hashes: Vec<(PathHash, EntryId)>,
    pending_hash_count: u32,
    loc_map: LocationMap,
}

impl SplitBuildContext<'_> {
    fn push_pending_hash(&mut self, hash: PathHash, id: EntryId) -> Result<(), PakError> {
        let hash_max = self
            .limits
            .entries_count_max()
            .saturating_mul(2)
            .min(ENTRIES_COUNT_HARD_MAX);
        if self.pending_hash_count >= hash_max {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::EntriesCount,
                actual: u64::from(self.pending_hash_count) + 1,
                max: u64::from(hash_max),
            });
        }
        self.pending_hashes.push((hash, id));
        self.pending_hash_count = self.pending_hash_count.strict_add(1);
        Ok(())
    }

    fn decode_location(
        &mut self,
        location: i32,
        context: WireContext,
    ) -> Result<EntryId, PakError> {
        let blk_start = self.table.current_block_start()?;
        let decoded = if location >= 0 {
            let offset =
                u32::try_from(location).map_err(|_| PakError::CorruptWireLayout { context })?;
            decode_encoded_entry(
                self.encoded_entries,
                offset,
                self.footer.dialect,
                &self.footer.methods,
                self.limits.blocks_total_count_max(),
                &mut self.table.blocks,
            )?
        } else {
            let unenc_idx_i64 = -(i64::from(location) + 1);
            let unenc_idx = u32::try_from(unenc_idx_i64)
                .map_err(|_| PakError::CorruptWireLayout { context })?;
            let item = self
                .unencoded_pool
                .items
                .get(u32_to_usize(unenc_idx))
                .ok_or(PakError::CorruptWireLayout { context })?;
            let src_start = u32_to_usize(item.block_start);
            let src_end = src_start.strict_add(usize::from(item.entry.block_count));
            let needed = self
                .table
                .blocks
                .len()
                .strict_add(usize::from(item.entry.block_count));
            if needed > u32_to_usize(self.limits.blocks_total_count_max()) {
                return Err(PakError::LimitExceeded {
                    kind: LimitKind::BlocksTotalCount,
                    actual: u64::from(self.limits.blocks_total_count_max()) + 1,
                    max: u64::from(self.limits.blocks_total_count_max()),
                });
            }
            self.table
                .blocks
                .extend_from_slice(&self.unencoded_pool.blocks[src_start..src_end]);
            item.entry
        };
        self.table.validate_and_push(
            blk_start,
            decoded,
            self.file.file_bytes(),
            self.limits,
            context,
        )
    }
}

fn populate_from_full_directory(
    mut cur: &[u8],
    ctx: &mut SplitBuildContext<'_>,
) -> Result<(), PakError> {
    let dir_count = take_u32_le(&mut cur, WireContext::FullDirectoryIndex)?;
    let dir_max = ctx.limits.entries_count_max().saturating_add(65_536);
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
        if file_count > ctx.limits.entries_count_max() {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::EntriesCount,
                actual: u64::from(file_count),
                max: u64::from(ctx.limits.entries_count_max()),
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
            ctx.loc_map.insert(location, id, ctx.limits)?;
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
    if count > ctx.limits.entries_count_max() {
        return Err(PakError::LimitExceeded {
            kind: LimitKind::EntriesCount,
            actual: u64::from(count),
            max: u64::from(ctx.limits.entries_count_max()),
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
            ctx.loc_map.insert(location, new_id, ctx.limits)?;
            ctx.push_pending_hash(hash, new_id)?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum LocationBucket {
    Empty,
    Occupied { location: i32, entry: EntryId },
}

/// Temporary open-addressed `FPakEntryLocation -> EntryId` table used only during split-index open.
struct LocationMap {
    buckets: Box<[LocationBucket]>,
    mask: u32,
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
            buckets: vec![LocationBucket::Empty; u32_to_usize(table_len)].into_boxed_slice(),
            mask: table_len - 1,
            occupied_count: 0,
        }
    }

    #[inline]
    fn start_slot(&self, location: i32) -> u32 {
        let bits = u32::from_le_bytes(location.to_le_bytes());
        let mixed = u64::from(bits).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        low_u32(mixed ^ (mixed >> 32)) & self.mask
    }

    fn grow(&mut self, limits: PakLimits) -> Result<(), PakError> {
        let cur_len = self.mask.strict_add(1);
        let max_slots = (limits
            .entries_count_max()
            .min(ENTRIES_COUNT_HARD_MAX)
            .saturating_mul(2))
        .next_power_of_two();
        if cur_len >= max_slots {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::EntriesCount,
                actual: u64::from(self.occupied_count) + 1,
                max: u64::from(limits.entries_count_max()),
            });
        }
        let next_len = cur_len.checked_mul(2).ok_or(PakError::CorruptWireLayout {
            context: WireContext::PrimaryIndex,
        })?;
        let old = core::mem::replace(
            &mut self.buckets,
            vec![LocationBucket::Empty; u32_to_usize(next_len)].into_boxed_slice(),
        );
        self.mask = next_len - 1;
        self.occupied_count = 0;
        for &bucket in &old {
            if let LocationBucket::Occupied { location, entry } = bucket {
                self.insert_no_grow(location, entry);
            }
        }
        Ok(())
    }

    fn insert(&mut self, location: i32, entry: EntryId, limits: PakLimits) -> Result<(), PakError> {
        let table_len = self.mask.strict_add(1);
        if self.occupied_count.saturating_mul(2) >= table_len {
            self.grow(limits)?;
        }
        self.insert_no_grow(location, entry);
        Ok(())
    }

    fn insert_no_grow(&mut self, location: i32, entry: EntryId) {
        let table_len = self.mask.strict_add(1);
        let start = self.start_slot(location);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let idx = u32_to_usize(start.wrapping_add(step) & self.mask);
            match self.buckets[idx] {
                LocationBucket::Empty => {
                    self.buckets[idx] = LocationBucket::Occupied { location, entry };
                    self.occupied_count = self.occupied_count.strict_add(1);
                    return;
                }
                LocationBucket::Occupied {
                    location: existing_loc,
                    ..
                } => {
                    if existing_loc == location {
                        return;
                    }
                    probed = probed.strict_add(1);
                }
            }
        }
        assert!(self.occupied_count < table_len);
        assert!(probed <= self.occupied_count);
    }

    fn get(&self, location: i32) -> Option<EntryId> {
        let table_len = self.mask.strict_add(1);
        let start = self.start_slot(location);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let idx = u32_to_usize(start.wrapping_add(step) & self.mask);
            match self.buckets[idx] {
                LocationBucket::Empty => return None,
                LocationBucket::Occupied {
                    location: existing_loc,
                    entry,
                } => {
                    if existing_loc == location {
                        return Some(entry);
                    }
                    probed = probed.strict_add(1);
                }
            }
        }
        assert!(self.occupied_count < table_len);
        assert!(probed <= self.occupied_count);
        None
    }
}

//! Lock-free concurrent `.pak` archive reader (`PakArchive`), payload reader (`PakPayloadReader`),
//! and entry iterator (`EntryIter`).

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use flate2::{Decompress, FlushDecompress, Status};
use oozextract::Extractor;

use crate::crypto::{PakCipher, Sha1Hasher, compute_sha1};
use crate::error::{EncryptedSectionKind, LimitKind, PakError, WireContext};
use crate::index::{EntryTable, load_archive_catalog, load_streaming_catalog};
use crate::io::PositionedFile;
use crate::path::{PathArena, PathLookupTable, compute_ue4_path_hash};
use crate::scratch::{CachedBlockKey, RangeScratchParts, ReadScratch};
use crate::types::{
    AesKey, BatchReadOutcome, BatchReadRequest, CompressionMethod, EncryptionKeyGuid, EntryId,
    EntryMeta, FOOTER_BYTES_MAX, FOOTER_V8A_BYTES, OpenOptions, PakDialect, PakVersion, PathHash,
    StreamingEntry,
};
use crate::wire::{
    CompressionBlock, INLINE_HEADER_HASH_PREFIX_BYTES_MAX, ParsedFooter,
    inline_header_hash_prefix_bytes, parse_footer_tail, parse_inline_entry_sha1,
};
use crate::{align_up_16, bounded_len_u32, u32_to_usize};

const SHA1_STREAM_CHUNK_BYTES_MAX: u32 = 4 * 1024 * 1024;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<PakPayloadReader>();
const _: () = assert_send_sync::<PakArchive>();

static NEXT_READER_COOKIE: AtomicU64 = AtomicU64::new(1);

fn next_reader_cookie() -> u64 {
    let cookie = NEXT_READER_COOKIE.fetch_add(1, Ordering::Relaxed);
    assert_ne!(cookie, u64::MAX);
    cookie
}

fn open_file_and_footer(path: &Path) -> Result<(PositionedFile, ParsedFooter), PakError> {
    let file = PositionedFile::open(path)?;
    let file_bytes = file.file_bytes();
    if file_bytes < u64::from(FOOTER_V8A_BYTES) {
        return Err(PakError::FileTooSmall {
            file_bytes,
            min_bytes: FOOTER_V8A_BYTES,
        });
    }
    let tail_bytes_u32 = match u32::try_from(file_bytes) {
        Ok(n) => n.min(FOOTER_BYTES_MAX),
        Err(_) => FOOTER_BYTES_MAX,
    };
    let tail_bytes_u64 = u64::from(tail_bytes_u32);
    let tail_usize = u32_to_usize(tail_bytes_u32);
    let tail_offset = file_bytes.strict_sub(tail_bytes_u64);
    let mut tail_buf = [0_u8; u32_to_usize(FOOTER_BYTES_MAX)];
    let tail_slice = &mut tail_buf[..tail_usize];
    file.read_exact_at(tail_offset, tail_slice)?;
    let footer = parse_footer_tail(tail_slice)?;
    Ok((file, footer))
}

/// Lean, thread-safe `.pak` payload reader retaining only `EntryTable` point-lookup metadata and the file handle.
///
/// Omits per-archive string arenas and hash lookup tables so multi-archive virtual file systems can
/// mount dozens of `.pak` files with minimal steady-state memory.
#[derive(Debug)]
pub struct PakPayloadReader {
    reader_cookie: u64,
    file: PositionedFile,
    cipher: Option<PakCipher>,
    dialect: PakDialect,
    version: PakVersion,
    encryption_key_guid: EncryptionKeyGuid,
    entries: EntryTable,
}

impl PakPayloadReader {
    /// Opens a `.pak` archive at `path`, streaming each directory entry along with the archive's
    /// `mount_point` to `on_entry` without allocating a per-archive `PathArena` or `PathLookupTable`.
    ///
    /// Returns the constructed [`PakPayloadReader`] together with the archive's raw mount-point string.
    pub fn open_streaming<E, F>(
        path: &Path,
        options: &OpenOptions,
        on_entry: F,
    ) -> Result<(Self, Box<str>), E>
    where
        E: From<PakError>,
        F: FnMut(&str, StreamingEntry<'_>) -> Result<(), E>,
    {
        let (file, footer) = open_file_and_footer(path)?;
        let cipher = options.key().map(PakCipher::new);
        let catalog = load_streaming_catalog(&file, &footer, cipher.as_ref(), options, on_entry)?;
        let reader = Self {
            reader_cookie: next_reader_cookie(),
            file,
            cipher,
            dialect: footer.dialect,
            version: footer.version,
            encryption_key_guid: footer.encryption_key_guid,
            entries: catalog.entries,
        };
        Ok((reader, catalog.mount_point))
    }

    #[inline]
    #[must_use]
    pub const fn dialect(&self) -> PakDialect {
        self.dialect
    }

    #[inline]
    #[must_use]
    pub const fn version(&self) -> PakVersion {
        self.version
    }

    #[inline]
    #[must_use]
    pub const fn encryption_key_guid(&self) -> EncryptionKeyGuid {
        self.encryption_key_guid
    }

    #[inline]
    #[must_use]
    pub const fn file_bytes(&self) -> u64 {
        self.file.file_bytes()
    }

    #[inline]
    #[must_use]
    pub const fn entry_count(&self) -> u32 {
        self.entries.len()
    }

    /// Validates a 0-based entry index against `0..entry_count`, returning `Some(EntryId)` if in range.
    #[inline]
    #[must_use]
    pub const fn entry_id_at(&self, index: u32) -> Option<EntryId> {
        if index < self.entries.len() {
            Some(EntryId::new(index))
        } else {
            None
        }
    }

    /// Returns pre-validated size, offset, block-count, codec, and encryption metadata for `id`.
    #[inline]
    #[must_use]
    pub fn entry_meta(&self, id: EntryId) -> EntryMeta {
        assert!(
            id.index() < self.entries.len(),
            "EntryId out of bounds for this PakPayloadReader"
        );
        self.entries.meta(id)
    }

    /// Reads, decrypts, and decompresses entry `id` into `dst` (which must have length `uncompressed_bytes`).
    ///
    /// Performs zero heap allocations when `scratch` already has sufficient capacity, and reads
    /// uncompressed unencrypted entries directly into `dst` without touching `scratch`.
    pub fn read_entry_into(
        &self,
        id: EntryId,
        scratch: &mut ReadScratch,
        dst: &mut [u8],
    ) -> Result<u64, PakError> {
        assert!(
            id.index() < self.entries.len(),
            "EntryId out of bounds for this PakPayloadReader"
        );
        let meta = self.entries.meta(id);
        let dst_len_u64 =
            u64::try_from(dst.len()).map_err(|_| PakError::DestinationSizeMismatch {
                expected: meta.uncompressed_bytes,
                actual: u64::MAX,
            })?;
        if dst_len_u64 != meta.uncompressed_bytes {
            return Err(PakError::DestinationSizeMismatch {
                expected: meta.uncompressed_bytes,
                actual: dst_len_u64,
            });
        }
        if meta.uncompressed_bytes == 0 {
            return Ok(0);
        }
        if meta.compression_method == CompressionMethod::None && !meta.is_encrypted() {
            self.file.read_exact_at(meta.payload_offset, dst)?;
            return Ok(meta.uncompressed_bytes);
        }
        let parts = scratch.prepare_parts(meta.disk_span_bytes)?;
        self.file.read_exact_at(meta.payload_offset, parts.io_buf)?;
        if meta.is_encrypted() {
            self.decrypt_payload_buf(parts.io_buf)?;
        }
        if meta.compression_method == CompressionMethod::None {
            dst.copy_from_slice(&parts.io_buf[..dst.len()]);
            return Ok(meta.uncompressed_bytes);
        }
        let range = self.entries.record(id).block_range;
        let blocks = self.entries.blocks_for(range);
        decompress_blocks(
            BlockDecodeJob {
                id,
                method: meta.compression_method,
                block_cap_bytes: meta.compression_block_bytes,
                blocks,
                io_buf: parts.io_buf,
            },
            parts.oodle,
            parts.zlib,
            dst,
        )?;
        Ok(meta.uncompressed_bytes)
    }

    /// Reads an arbitrary uncompressed byte window `[offset .. offset + dst.len())` of entry `id` into `dst`.
    ///
    /// Decrypts and decompresses strictly the intersecting `16`-byte AES blocks or compressed payload blocks,
    /// caching single-block partial decodes in `scratch` so sequential small reads avoid repeated disk and
    /// codec work. Total over empty `dst` (`&mut []`) when `offset <= uncompressed_bytes`.
    pub fn read_entry_range_into(
        &self,
        id: EntryId,
        offset: u64,
        scratch: &mut ReadScratch,
        dst: &mut [u8],
    ) -> Result<(), PakError> {
        assert!(
            id.index() < self.entries.len(),
            "EntryId out of bounds for this PakPayloadReader"
        );
        let meta = self.entries.meta(id);
        let dst_len_u64 = u64::try_from(dst.len()).map_err(|_| PakError::OutOfBoundsRange {
            offset,
            byte_len: u64::MAX,
            container_bytes: meta.uncompressed_bytes,
        })?;
        let req_end = offset
            .checked_add(dst_len_u64)
            .ok_or(PakError::OutOfBoundsRange {
                offset,
                byte_len: dst_len_u64,
                container_bytes: meta.uncompressed_bytes,
            })?;
        if req_end > meta.uncompressed_bytes {
            return Err(PakError::OutOfBoundsRange {
                offset,
                byte_len: dst_len_u64,
                container_bytes: meta.uncompressed_bytes,
            });
        }
        if dst.is_empty() {
            return Ok(());
        }
        if meta.compression_method == CompressionMethod::None {
            if !meta.is_encrypted() {
                return self
                    .file
                    .read_exact_at(meta.payload_offset.strict_add(offset), dst);
            }
            return self.read_encrypted_raw_range(meta, offset, req_end, scratch, dst);
        }
        self.read_compressed_range(id, meta, offset, req_end, scratch, dst)
    }

    fn read_encrypted_raw_range(
        &self,
        meta: EntryMeta,
        offset: u64,
        req_end: u64,
        scratch: &mut ReadScratch,
        dst: &mut [u8],
    ) -> Result<(), PakError> {
        let aligned_start = offset & !15_u64;
        let aligned_end = align_up_16(req_end).ok_or(PakError::OutOfBoundsRange {
            offset,
            byte_len: req_end.strict_sub(offset),
            container_bytes: meta.uncompressed_bytes,
        })?;
        let aligned_span = aligned_end.strict_sub(aligned_start);
        let parts = scratch.prepare_parts(aligned_span)?;
        self.file
            .read_exact_at(meta.payload_offset.strict_add(aligned_start), parts.io_buf)?;
        self.decrypt_payload_buf(parts.io_buf)?;
        let head_skip = u32_to_usize((offset & 15) as u32);
        let copy_end = head_skip.strict_add(dst.len());
        dst.copy_from_slice(&parts.io_buf[head_skip..copy_end]);
        Ok(())
    }

    fn read_compressed_range(
        &self,
        id: EntryId,
        meta: EntryMeta,
        offset: u64,
        req_end: u64,
        scratch: &mut ReadScratch,
        dst: &mut [u8],
    ) -> Result<(), PakError> {
        let (first_blk, last_blk) = compressed_block_span(&meta, offset, req_end)?;
        if first_blk == last_blk {
            let (blk_start, blk_len_u64) = block_uncompressed_window(&meta, first_blk);
            if let Some(cached) =
                scratch.cached_block_slice(self.reader_cookie, id, first_blk, blk_len_u64)
            {
                let slice_start = usize::try_from(offset.strict_sub(blk_start)).map_err(|_| {
                    PakError::CorruptWireLayout {
                        context: WireContext::EncodedEntry,
                    }
                })?;
                let slice_end = slice_start.strict_add(dst.len());
                dst.copy_from_slice(&cached[slice_start..slice_end]);
                return Ok(());
            }
        }
        let range = self.entries.record(id).block_range;
        let all_blocks = self.entries.blocks_for(range);
        let span_blocks = &all_blocks[u32_to_usize(first_blk)..=u32_to_usize(last_blk)];
        let first_disk_rel = span_blocks[0].rel_offset;
        let last_disk_blk = span_blocks[span_blocks.len().strict_sub(1)];
        let last_stride = if meta.is_encrypted() {
            align_up_16(u64::from(last_disk_blk.compressed_bytes)).ok_or(
                PakError::CorruptWireLayout {
                    context: WireContext::EncodedEntry,
                },
            )?
        } else {
            u64::from(last_disk_blk.compressed_bytes)
        };
        let io_span = last_disk_blk
            .rel_offset
            .strict_add(last_stride)
            .strict_sub(first_disk_rel);
        let needed_blk_buf =
            required_partial_block_bytes(&meta, first_blk, last_blk, offset, req_end)?;
        let mut parts = scratch.prepare_range_parts(io_span, needed_blk_buf)?;
        self.file
            .read_exact_at(meta.payload_offset.strict_add(first_disk_rel), parts.io_buf)?;
        if meta.is_encrypted() {
            self.decrypt_payload_buf(parts.io_buf)?;
        }
        decode_range_blocks(
            RangeDecodeJob {
                reader_cookie: self.reader_cookie,
                id,
                meta,
                first_blk,
                first_disk_rel,
                offset,
                req_end,
                span_blocks,
            },
            &mut parts,
            dst,
        )
    }

    fn decrypt_payload_buf(&self, buf: &mut [u8]) -> Result<(), PakError> {
        assert_eq!(buf.len() % 16, 0);
        let cipher = self
            .cipher
            .as_ref()
            .ok_or(PakError::EncryptionKeyRequired {
                guid: self.encryption_key_guid,
            })?;
        cipher.decrypt_in_place(EncryptedSectionKind::EntryPayload, buf)
    }

    /// Executes a batch of entry reads reusing `scratch` and recording per-item outcomes into `outcomes`.
    ///
    /// Total over empty slices (`&mut []`). Asserts `requests.len() == outcomes.len()`.
    pub fn read_batch_into(
        &self,
        requests: &mut [BatchReadRequest<'_>],
        scratch: &mut ReadScratch,
        outcomes: &mut [BatchReadOutcome],
    ) {
        assert_eq!(requests.len(), outcomes.len());
        for (req, slot) in requests.iter_mut().zip(outcomes.iter_mut()) {
            *slot = match self.read_entry_into(req.id, scratch, req.dst) {
                Ok(bytes_written) => BatchReadOutcome::Ok { bytes_written },
                Err(err) => BatchReadOutcome::Err(err),
            };
        }
    }

    /// Reads the inline `FPakEntry` header and hashed on-disk payload (`compressed_bytes`) and verifies
    /// its SHA-1 digest, using a single syscall when `total_span <= 4 MiB` and bounded chunk streaming
    /// for larger payloads or constrained [`ReadScratch`] capacities.
    pub fn verify_entry_sha1(
        &self,
        id: EntryId,
        scratch: &mut ReadScratch,
    ) -> Result<(), PakError> {
        assert!(
            id.index() < self.entries.len(),
            "EntryId out of bounds for this PakPayloadReader"
        );
        let meta = self.entries.meta(id);
        let hdr_len = meta.payload_offset.strict_sub(meta.header_offset);
        let total_span =
            hdr_len
                .checked_add(meta.compressed_bytes)
                .ok_or(PakError::OutOfBoundsRange {
                    offset: meta.header_offset,
                    byte_len: u64::MAX,
                    container_bytes: self.file.file_bytes(),
                })?;
        let stream_cap = u64::from(scratch.io_bytes_max().min(SHA1_STREAM_CHUNK_BYTES_MAX));
        assert!(stream_cap > 0);
        if total_span > stream_cap {
            return self.verify_entry_sha1_streaming(id, &meta, stream_cap, scratch);
        }
        let parts = scratch.prepare_parts(total_span)?;
        self.file.read_exact_at(meta.header_offset, parts.io_buf)?;
        let hdr_usize = usize::try_from(hdr_len).map_err(|_| PakError::CorruptWireLayout {
            context: WireContext::InlineEntryHeader,
        })?;
        let (hdr_bytes, payload_bytes) =
            parts
                .io_buf
                .split_at_checked(hdr_usize)
                .ok_or(PakError::CorruptWireLayout {
                    context: WireContext::InlineEntryHeader,
                })?;
        let expected = parse_inline_entry_sha1(hdr_bytes, self.dialect)?;
        let actual = compute_sha1(payload_bytes);
        if actual != expected {
            return Err(PakError::PayloadHashMismatch {
                entry: id,
                expected,
                actual,
            });
        }
        Ok(())
    }

    fn verify_entry_sha1_streaming(
        &self,
        id: EntryId,
        meta: &EntryMeta,
        stream_cap: u64,
        scratch: &mut ReadScratch,
    ) -> Result<(), PakError> {
        let mut hdr_buf = [0_u8; u32_to_usize(INLINE_HEADER_HASH_PREFIX_BYTES_MAX)];
        let prefix_len = u32_to_usize(inline_header_hash_prefix_bytes(self.dialect));
        let hdr_slice = &mut hdr_buf[..prefix_len];
        self.file.read_exact_at(meta.header_offset, hdr_slice)?;
        let expected = parse_inline_entry_sha1(hdr_slice, self.dialect)?;

        let mut hasher = Sha1Hasher::new();
        let mut cur_offset = meta.payload_offset;
        let mut rem_bytes = meta.compressed_bytes;
        let max_steps = rem_bytes.div_ceil(stream_cap);
        for _ in 0..max_steps {
            let chunk_bytes = rem_bytes.min(stream_cap);
            let parts = scratch.prepare_parts(chunk_bytes)?;
            self.file.read_exact_at(cur_offset, parts.io_buf)?;
            hasher.update(parts.io_buf);
            cur_offset = cur_offset.strict_add(chunk_bytes);
            rem_bytes = rem_bytes.strict_sub(chunk_bytes);
        }
        assert_eq!(rem_bytes, 0);
        let actual = hasher.finalize();
        if actual != expected {
            return Err(PakError::PayloadHashMismatch {
                entry: id,
                expected,
                actual,
            });
        }
        Ok(())
    }
}

/// Immutable, thread-safe Unreal Engine `.pak` reader backed by flat access-temperature-grouped metadata tables.
///
/// Holds only compact index metadata in memory and performs positioned OS reads (`pread` / `seek_read`)
/// strictly on demand when reading specific entries.
#[derive(Debug)]
pub struct PakArchive {
    reader: PakPayloadReader,
    path_hash_seed: u64,
    mount_point: Box<str>,
    paths: PathArena,
    lookup: PathLookupTable,
}

impl PakArchive {
    /// Opens a `.pak` archive at `path` using default [`OpenOptions`] and an optional [`AesKey`].
    pub fn open(path: &Path, key: Option<AesKey>) -> Result<Self, PakError> {
        let options = OpenOptions::new(key);
        Self::open_with_options(path, &options)
    }

    /// Opens a `.pak` archive at `path` with explicit limits and hash verification settings.
    pub fn open_with_options(path: &Path, options: &OpenOptions) -> Result<Self, PakError> {
        let (file, footer) = open_file_and_footer(path)?;
        let cipher = options.key().map(PakCipher::new);
        let catalog = load_archive_catalog(&file, &footer, cipher.as_ref(), options)?;
        let reader = PakPayloadReader {
            reader_cookie: next_reader_cookie(),
            file,
            cipher,
            dialect: footer.dialect,
            version: footer.version,
            encryption_key_guid: footer.encryption_key_guid,
            entries: catalog.entries,
        };
        Ok(Self {
            reader,
            path_hash_seed: catalog.path_hash_seed,
            mount_point: catalog.mount_point,
            paths: catalog.paths,
            lookup: catalog.lookup,
        })
    }

    /// Borrows the underlying [`PakPayloadReader`].
    #[inline]
    #[must_use]
    pub const fn payload_reader(&self) -> &PakPayloadReader {
        &self.reader
    }

    /// Consumes this [`PakArchive`], dropping its string arena and hash table while retaining the [`PakPayloadReader`].
    #[inline]
    #[must_use]
    pub fn into_payload_reader(self) -> PakPayloadReader {
        self.reader
    }

    #[inline]
    #[must_use]
    pub const fn dialect(&self) -> PakDialect {
        self.reader.dialect()
    }

    #[inline]
    #[must_use]
    pub const fn version(&self) -> PakVersion {
        self.reader.version()
    }

    #[inline]
    #[must_use]
    pub fn mount_point(&self) -> &str {
        &self.mount_point
    }

    #[inline]
    #[must_use]
    pub const fn encryption_key_guid(&self) -> EncryptionKeyGuid {
        self.reader.encryption_key_guid()
    }

    #[inline]
    #[must_use]
    pub const fn path_hash_seed(&self) -> u64 {
        self.path_hash_seed
    }

    #[inline]
    #[must_use]
    pub const fn file_bytes(&self) -> u64 {
        self.reader.file_bytes()
    }

    #[inline]
    #[must_use]
    pub const fn entry_count(&self) -> u32 {
        self.reader.entry_count()
    }

    /// Validates a 0-based entry index against `0..entry_count`, returning `Some(EntryId)` if in range.
    #[inline]
    #[must_use]
    pub const fn entry_id_at(&self, index: u32) -> Option<EntryId> {
        self.reader.entry_id_at(index)
    }

    /// Iterates sequentially over all `0..entry_count` entries and their optional stored relative paths.
    #[inline]
    #[must_use]
    pub const fn iter_entries(&self) -> EntryIter<'_> {
        EntryIter {
            archive: self,
            next_idx: 0,
            end_idx: self.entry_count(),
        }
    }

    /// Looks up an archive-relative path (case-insensitive, `\` and `/` normalized) in `O(1)` time.
    pub fn find(&self, rel_path: &str) -> Result<Option<EntryId>, PakError> {
        let hash = compute_ue4_path_hash(rel_path, self.path_hash_seed)?;
        Ok(self.lookup.find(rel_path, hash, &self.paths))
    }

    /// Looks up an entry directly by its precomputed 64-bit UE4 [`PathHash`].
    #[inline]
    #[must_use]
    pub fn find_by_hash(&self, hash: PathHash) -> Option<EntryId> {
        self.lookup.find_by_hash(hash)
    }

    /// Returns pre-validated size, offset, block-count, codec, and encryption metadata for `id`.
    #[inline]
    #[must_use]
    pub fn entry_meta(&self, id: EntryId) -> EntryMeta {
        self.reader.entry_meta(id)
    }

    /// Returns the stored normalized relative path for `id` (`None` only for pruned hash-only entries).
    #[inline]
    #[must_use]
    pub fn entry_path(&self, id: EntryId) -> Option<&str> {
        assert!(
            id.index() < self.reader.entry_count(),
            "EntryId out of bounds for this PakArchive"
        );
        self.paths.get(id)
    }

    /// Reads, decrypts, and decompresses entry `id` into `dst` (which must have length `uncompressed_bytes`).
    ///
    /// Performs zero heap allocations when `scratch` already has sufficient capacity, and reads
    /// uncompressed unencrypted entries directly into `dst` without touching `scratch`.
    pub fn read_entry_into(
        &self,
        id: EntryId,
        scratch: &mut ReadScratch,
        dst: &mut [u8],
    ) -> Result<u64, PakError> {
        self.reader.read_entry_into(id, scratch, dst)
    }

    /// Reads an arbitrary uncompressed byte window `[offset .. offset + dst.len())` of entry `id` into `dst`.
    pub fn read_entry_range_into(
        &self,
        id: EntryId,
        offset: u64,
        scratch: &mut ReadScratch,
        dst: &mut [u8],
    ) -> Result<(), PakError> {
        self.reader.read_entry_range_into(id, offset, scratch, dst)
    }

    /// Executes a batch of entry reads reusing `scratch` and recording per-item outcomes into `outcomes`.
    ///
    /// Total over empty slices (`&mut []`). Asserts `requests.len() == outcomes.len()`.
    pub fn read_batch_into(
        &self,
        requests: &mut [BatchReadRequest<'_>],
        scratch: &mut ReadScratch,
        outcomes: &mut [BatchReadOutcome],
    ) {
        self.reader.read_batch_into(requests, scratch, outcomes);
    }

    /// Reads the inline `FPakEntry` header and hashed on-disk payload (`compressed_bytes`) and verifies
    /// its SHA-1 digest, using a single syscall when `total_span <= 4 MiB` and bounded chunk streaming
    /// for larger payloads or constrained [`ReadScratch`] capacities.
    ///
    /// UnrealPak computes `FPakEntry::Hash` over `compressed_bytes` of the on-disk payload (which equals
    /// `disk_span_bytes` for compressed entries and `uncompressed_bytes` prior to 16-byte AES tail padding
    /// for uncompressed encrypted entries).
    pub fn verify_entry_sha1(
        &self,
        id: EntryId,
        scratch: &mut ReadScratch,
    ) -> Result<(), PakError> {
        self.reader.verify_entry_sha1(id, scratch)
    }
}

fn compressed_block_span(
    meta: &EntryMeta,
    offset: u64,
    req_end: u64,
) -> Result<(u32, u32), PakError> {
    if meta.block_count == 1 {
        return Ok((0, 0));
    }
    let block_size = u64::from(meta.compression_block_bytes);
    assert!(block_size > 0);
    let first = u32::try_from(offset / block_size).map_err(|_| PakError::CorruptWireLayout {
        context: WireContext::EncodedEntry,
    })?;
    let last = u32::try_from(req_end.strict_sub(1) / block_size).map_err(|_| {
        PakError::CorruptWireLayout {
            context: WireContext::EncodedEntry,
        }
    })?;
    assert!(first <= last);
    assert!(last < u32::from(meta.block_count));
    Ok((first, last))
}

fn block_uncompressed_window(meta: &EntryMeta, block_idx: u32) -> (u64, u64) {
    if meta.block_count == 1 {
        assert_eq!(block_idx, 0);
        return (0, meta.uncompressed_bytes);
    }
    let std_len = u64::from(meta.compression_block_bytes);
    let blk_start = u64::from(block_idx).strict_mul(std_len);
    let is_last = block_idx.strict_add(1) == u32::from(meta.block_count);
    let blk_len = if is_last {
        meta.uncompressed_bytes.strict_sub(blk_start)
    } else {
        std_len
    };
    (blk_start, blk_len)
}

fn required_partial_block_bytes(
    meta: &EntryMeta,
    first_blk: u32,
    last_blk: u32,
    offset: u64,
    req_end: u64,
) -> Result<u32, PakError> {
    let mut max_needed: u64 = 0;
    let (start0, len0) = block_uncompressed_window(meta, first_blk);
    if offset > start0 || req_end < start0.strict_add(len0) {
        max_needed = max_needed.max(len0);
    }
    if last_blk > first_blk {
        let (start_last, len_last) = block_uncompressed_window(meta, last_blk);
        if req_end < start_last.strict_add(len_last) {
            max_needed = max_needed.max(len_last);
        }
    }
    u32::try_from(max_needed).map_err(|_| PakError::LimitExceeded {
        kind: LimitKind::ScratchBlockBytes,
        actual: max_needed,
        max: u64::from(u32::MAX),
    })
}

struct RangeDecodeJob<'a> {
    reader_cookie: u64,
    id: EntryId,
    meta: EntryMeta,
    first_blk: u32,
    first_disk_rel: u64,
    offset: u64,
    req_end: u64,
    span_blocks: &'a [CompressionBlock],
}

fn decode_range_blocks(
    job: RangeDecodeJob<'_>,
    parts: &mut RangeScratchParts<'_>,
    mut rem_dst: &mut [u8],
) -> Result<(), PakError> {
    let span_count = bounded_len_u32(job.span_blocks.len());
    for (step_u32, &blk) in (0..span_count).zip(job.span_blocks.iter()) {
        let blk_idx = job.first_blk.strict_add(step_u32);
        let src = slice_range_block_src(parts.io_buf, job.first_disk_rel, blk)?;
        let (blk_start, blk_len_u64) = block_uncompressed_window(&job.meta, blk_idx);
        let blk_end = blk_start.strict_add(blk_len_u64);
        let copy_start = job.offset.max(blk_start);
        let copy_end = job.req_end.min(blk_end);
        let want_usize = usize::try_from(copy_end.strict_sub(copy_start)).map_err(|_| {
            PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            }
        })?;
        let (dst_chunk, rest) =
            rem_dst
                .split_at_mut_checked(want_usize)
                .ok_or(PakError::CorruptWireLayout {
                    context: WireContext::EncodedEntry,
                })?;
        rem_dst = rest;
        if copy_start == blk_start && copy_end == blk_end {
            decode_single_block(
                job.meta.compression_method,
                parts.oodle,
                parts.zlib,
                job.id,
                blk_idx,
                src,
                dst_chunk,
            )?;
        } else {
            let blk_len_u32 =
                u32::try_from(blk_len_u64).map_err(|_| PakError::CorruptWireLayout {
                    context: WireContext::EncodedEntry,
                })?;
            let blk_out = &mut parts.block_buf[..u32_to_usize(blk_len_u32)];
            *parts.cached_block = None;
            decode_single_block(
                job.meta.compression_method,
                parts.oodle,
                parts.zlib,
                job.id,
                blk_idx,
                src,
                blk_out,
            )?;
            *parts.cached_block = Some(CachedBlockKey {
                reader_cookie: job.reader_cookie,
                entry: job.id,
                block_index: blk_idx,
                valid_bytes: blk_len_u32,
            });
            let in_blk_start = usize::try_from(copy_start.strict_sub(blk_start)).map_err(|_| {
                PakError::CorruptWireLayout {
                    context: WireContext::EncodedEntry,
                }
            })?;
            dst_chunk.copy_from_slice(&blk_out[in_blk_start..in_blk_start.strict_add(want_usize)]);
        }
    }
    assert!(rem_dst.is_empty());
    Ok(())
}

fn slice_range_block_src(
    io_buf: &[u8],
    first_disk_rel: u64,
    blk: CompressionBlock,
) -> Result<&[u8], PakError> {
    let rel_start =
        blk.rel_offset
            .checked_sub(first_disk_rel)
            .ok_or(PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            })?;
    let src_start = usize::try_from(rel_start).map_err(|_| PakError::CorruptWireLayout {
        context: WireContext::EncodedEntry,
    })?;
    let src_end = src_start.strict_add(u32_to_usize(blk.compressed_bytes));
    io_buf
        .get(src_start..src_end)
        .ok_or(PakError::CorruptWireLayout {
            context: WireContext::EncodedEntry,
        })
}

struct BlockDecodeJob<'a> {
    id: EntryId,
    method: CompressionMethod,
    block_cap_bytes: u32,
    blocks: &'a [CompressionBlock],
    io_buf: &'a [u8],
}

fn decompress_blocks(
    job: BlockDecodeJob<'_>,
    oodle: &mut Extractor,
    zlib: &mut Decompress,
    mut rem_dst: &mut [u8],
) -> Result<(), PakError> {
    let std_chunk_usize = u32_to_usize(job.block_cap_bytes);
    let block_total = bounded_len_u32(job.blocks.len());
    for (block_idx, blk) in (0..block_total).zip(job.blocks.iter()) {
        let src_start =
            usize::try_from(blk.rel_offset).map_err(|_| PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            })?;
        let src_end = src_start.strict_add(u32_to_usize(blk.compressed_bytes));
        let src = job
            .io_buf
            .get(src_start..src_end)
            .ok_or(PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            })?;
        let is_last = block_idx.strict_add(1) == block_total;
        let out_len = if is_last {
            rem_dst.len()
        } else {
            std_chunk_usize
        };
        let (out_chunk, rest) =
            rem_dst
                .split_at_mut_checked(out_len)
                .ok_or(PakError::CorruptWireLayout {
                    context: WireContext::EncodedEntry,
                })?;
        rem_dst = rest;
        decode_single_block(job.method, oodle, zlib, job.id, block_idx, src, out_chunk)?;
    }
    assert!(rem_dst.is_empty());
    Ok(())
}

fn decode_single_block(
    method: CompressionMethod,
    oodle: &mut Extractor,
    zlib: &mut Decompress,
    entry: EntryId,
    block_index: u32,
    src: &[u8],
    out_chunk: &mut [u8],
) -> Result<(), PakError> {
    match method {
        CompressionMethod::Zlib => decompress_zlib_block(zlib, entry, block_index, src, out_chunk),
        CompressionMethod::Oodle => {
            decompress_oodle_block(oodle, entry, block_index, src, out_chunk)
        }
        CompressionMethod::None => {
            assert_ne!(method, CompressionMethod::None);
            Ok(())
        }
    }
}

fn decompress_zlib_block(
    zlib: &mut Decompress,
    entry: EntryId,
    block_index: u32,
    src: &[u8],
    out_chunk: &mut [u8],
) -> Result<(), PakError> {
    zlib.reset(true);
    let status = zlib
        .decompress(src, out_chunk, FlushDecompress::Finish)
        .map_err(|_| PakError::ZlibDecompressFailed { entry, block_index })?;
    let expected_out = u64::try_from(out_chunk.len())
        .map_err(|_| PakError::ZlibDecompressFailed { entry, block_index })?;
    if status != Status::StreamEnd || zlib.total_out() != expected_out {
        return Err(PakError::ZlibDecompressFailed { entry, block_index });
    }
    Ok(())
}

fn decompress_oodle_block(
    oodle: &mut Extractor,
    entry: EntryId,
    block_index: u32,
    src: &[u8],
    out_chunk: &mut [u8],
) -> Result<(), PakError> {
    let written = oodle
        .read_from_slice(src, out_chunk)
        .map_err(|_| PakError::OodleDecompressFailed { entry, block_index })?;
    if written != out_chunk.len() {
        return Err(PakError::OodleDecompressFailed { entry, block_index });
    }
    Ok(())
}

/// Exact-size iterator over all entries in a [`PakArchive`], yielding `(EntryId, Option<&str>)`.
#[derive(Clone, Copy, Debug)]
pub struct EntryIter<'a> {
    archive: &'a PakArchive,
    next_idx: u32,
    end_idx: u32,
}

impl<'a> Iterator for EntryIter<'a> {
    type Item = (EntryId, Option<&'a str>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.end_idx {
            return None;
        }
        let id = EntryId::new(self.next_idx);
        self.next_idx = self.next_idx.strict_add(1);
        Some((id, self.archive.paths.get(id)))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = u32_to_usize(self.end_idx.strict_sub(self.next_idx));
        (rem, Some(rem))
    }
}

impl DoubleEndedIterator for EntryIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.end_idx {
            return None;
        }
        self.end_idx = self.end_idx.strict_sub(1);
        let id = EntryId::new(self.end_idx);
        Some((id, self.archive.paths.get(id)))
    }
}

impl ExactSizeIterator for EntryIter<'_> {}

//! Lock-free concurrent `.pak` archive reader (`PakArchive`) and entry iterator (`EntryIter`).

use std::path::Path;

use flate2::{Decompress, FlushDecompress, Status};
use oozextract::Extractor;

use crate::crypto::{PakCipher, compute_sha1};
use crate::error::{EncryptedSectionKind, PakError, WireContext};
use crate::index::{EntryTable, load_archive_catalog};
use crate::io::PositionedFile;
use crate::path::{PathArena, PathLookupTable, compute_ue4_path_hash};
use crate::scratch::ReadScratch;
use crate::types::{
    AesKey, BatchReadOutcome, BatchReadRequest, CompressionMethod, EncryptionKeyGuid, EntryId,
    EntryMeta, FOOTER_BYTES_MAX, FOOTER_V8A_BYTES, OpenOptions, PakDialect, PakVersion, PathHash,
};
use crate::u32_to_usize;
use crate::wire::{CompressionBlock, parse_footer_tail, parse_inline_entry_sha1};

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<PakArchive>();

/// Immutable, thread-safe Unreal Engine `.pak` reader backed by flat Struct-of-Arrays metadata.
///
/// Holds only compact index metadata in memory and performs positioned OS reads (`pread` / `seek_read`)
/// strictly on demand when reading specific entries.
#[derive(Debug)]
pub struct PakArchive {
    file: PositionedFile,
    cipher: Option<PakCipher>,
    dialect: PakDialect,
    version: PakVersion,
    encryption_key_guid: EncryptionKeyGuid,
    path_hash_seed: u64,
    mount_point: Box<str>,
    entries: EntryTable,
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
        let file = PositionedFile::open(path)?;
        let file_bytes = file.file_bytes();
        if file_bytes < u64::from(FOOTER_V8A_BYTES) {
            return Err(PakError::FileTooSmall {
                file_bytes,
                min_bytes: FOOTER_V8A_BYTES,
            });
        }
        let tail_bytes_u64 = file_bytes.min(u64::from(FOOTER_BYTES_MAX));
        let tail_bytes_u32 =
            u32::try_from(tail_bytes_u64).map_err(|_| PakError::CorruptWireLayout {
                context: WireContext::Footer,
            })?;
        let tail_usize = u32_to_usize(tail_bytes_u32);
        let tail_offset = file_bytes.strict_sub(tail_bytes_u64);
        let mut tail_buf = [0_u8; u32_to_usize(FOOTER_BYTES_MAX)];
        file.read_exact_at(tail_offset, &mut tail_buf[..tail_usize])?;
        let footer = parse_footer_tail(&tail_buf[..tail_usize])?;
        let cipher = options.key().map(PakCipher::new);
        let catalog = load_archive_catalog(&file, &footer, cipher.as_ref(), options)?;
        Ok(Self {
            file,
            cipher,
            dialect: footer.dialect,
            version: footer.version,
            encryption_key_guid: footer.encryption_key_guid,
            path_hash_seed: catalog.path_hash_seed,
            mount_point: catalog.mount_point,
            entries: catalog.entries,
            paths: catalog.paths,
            lookup: catalog.lookup,
        })
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
    pub fn mount_point(&self) -> &str {
        &self.mount_point
    }

    #[inline]
    #[must_use]
    pub const fn encryption_key_guid(&self) -> EncryptionKeyGuid {
        self.encryption_key_guid
    }

    #[inline]
    #[must_use]
    pub const fn path_hash_seed(&self) -> u64 {
        self.path_hash_seed
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

    /// Iterates sequentially over all `0..entry_count` entries and their optional stored relative paths.
    #[inline]
    #[must_use]
    pub const fn iter_entries(&self) -> EntryIter<'_> {
        EntryIter {
            archive: self,
            next_idx: 0,
            end_idx: self.entries.len(),
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
        assert!(
            id.index() < self.entries.len(),
            "EntryId out of bounds for this PakArchive"
        );
        self.entries.meta(id)
    }

    /// Returns the stored normalized relative path for `id` (`None` only for pruned hash-only entries).
    #[inline]
    #[must_use]
    pub fn entry_path(&self, id: EntryId) -> Option<&str> {
        assert!(
            id.index() < self.entries.len(),
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
        assert!(
            id.index() < self.entries.len(),
            "EntryId out of bounds for this PakArchive"
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
            assert_eq!(parts.io_buf.len() % 16, 0);
            let cipher = self
                .cipher
                .as_ref()
                .ok_or(PakError::EncryptionKeyRequired {
                    guid: self.encryption_key_guid,
                })?;
            cipher.decrypt_in_place(EncryptedSectionKind::EntryPayload, parts.io_buf)?;
        }
        if meta.compression_method == CompressionMethod::None {
            dst.copy_from_slice(&parts.io_buf[..dst.len()]);
            return Ok(meta.uncompressed_bytes);
        }
        let range = self.entries.block_ranges[u32_to_usize(id.index())];
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

    /// Reads the inline `FPakEntry` header and hashed on-disk payload (`compressed_bytes`) in one syscall
    /// and verifies its SHA-1 digest.
    ///
    /// UnrealPak computes `FPakEntry::Hash` over `compressed_bytes` of the on-disk payload (which equals
    /// `disk_span_bytes` for compressed entries and `uncompressed_bytes` prior to 16-byte AES tail padding
    /// for uncompressed encrypted entries).
    pub fn verify_entry_sha1(
        &self,
        id: EntryId,
        scratch: &mut ReadScratch,
    ) -> Result<(), PakError> {
        assert!(
            id.index() < self.entries.len(),
            "EntryId out of bounds for this PakArchive"
        );
        let meta = self.entries.meta(id);
        assert!(meta.compressed_bytes <= meta.disk_span_bytes);
        let hdr_len = meta.payload_offset.strict_sub(meta.header_offset);
        let total_span =
            hdr_len
                .checked_add(meta.compressed_bytes)
                .ok_or(PakError::OutOfBoundsRange {
                    offset: meta.header_offset,
                    byte_len: u64::MAX,
                    container_bytes: self.file.file_bytes(),
                })?;
        let parts = scratch.prepare_parts(total_span)?;
        self.file.read_exact_at(meta.header_offset, parts.io_buf)?;
        let hdr_usize = usize::try_from(hdr_len).map_err(|_| PakError::CorruptWireLayout {
            context: WireContext::InlineEntryHeader,
        })?;
        let (hdr_bytes, payload_bytes) = parts.io_buf.split_at(hdr_usize);
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
    let block_total =
        u32::from(
            u16::try_from(job.blocks.len()).map_err(|_| PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            })?,
        );
    for (block_idx, blk) in (0..block_total).zip(job.blocks.iter()) {
        let src_start =
            usize::try_from(blk.rel_offset).map_err(|_| PakError::CorruptWireLayout {
                context: WireContext::EncodedEntry,
            })?;
        let src_end = src_start.strict_add(u32_to_usize(blk.compressed_bytes));
        let src = &job.io_buf[src_start..src_end];
        let is_last = block_idx.strict_add(1) == block_total;
        let out_len = if is_last {
            rem_dst.len()
        } else {
            std_chunk_usize
        };
        let (out_chunk, rest) = rem_dst.split_at_mut(out_len);
        rem_dst = rest;
        match job.method {
            CompressionMethod::Zlib => {
                decompress_zlib_block(zlib, job.id, block_idx, src, out_chunk)?
            }
            CompressionMethod::Oodle => {
                decompress_oodle_block(oodle, job.id, block_idx, src, out_chunk)?
            }
            CompressionMethod::None => {
                assert_ne!(job.method, CompressionMethod::None);
            }
        }
    }
    assert!(rem_dst.is_empty());
    Ok(())
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

impl ExactSizeIterator for EntryIter<'_> {}

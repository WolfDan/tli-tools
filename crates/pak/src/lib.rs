//! Data-oriented TigerStyle Unreal Engine `.pak` archive reader.
//!
//! `pak` reads both stock Unreal Engine 4 `.pak` files (`v8A`, `v8B`, `v9` non-frozen, `v10`, `v11`)
//! and Torchlight Infinite's custom `v12` `.pak` dialect (`magic = 0x6B2A56B8`).
//!
//! # Design
//! - **Metadata-only open:** [`PakArchive::open`] reads only the trailing footer and index tables,
//!   flattening all metadata into immutable Struct-of-Arrays `Box<[T]>` columns with zero payload caching.
//! - **Lock-free concurrency:** [`PakArchive`] is `Send + Sync` and reads directly via positioned OS
//!   syscalls (`pread` on Unix, `OVERLAPPED` `seek_read` on Windows).
//! - **Explicit scratch reuse:** Callers allocate [`ReadScratch`] once per thread and reuse it across
//!   [`PakArchive::read_entry_into`] and [`PakArchive::read_batch_into`] calls for zero post-startup
//!   heap allocations.

mod archive;
mod crypto;
mod error;
mod index;
mod io;
mod path;
mod scratch;
#[cfg(test)]
mod tests;
mod types;
mod wire;

pub use archive::{EntryIter, PakArchive};
pub use error::{
    EncryptedSectionKind, IndexSectionKind, KeyParseErrorKind, LimitKind, PakError, WireContext,
};
pub use scratch::{ReadScratch, ScratchLimits};
pub use types::{
    AES_BLOCK_BYTES, AesKey, BLOCKS_PER_ENTRY_COUNT_MAX, BLOCKS_TOTAL_COUNT_MAX_DEFAULT,
    BatchReadOutcome, BatchReadRequest, COMPRESSION_METHOD_NAME_BYTES_MAX,
    COMPRESSION_METHODS_COUNT_MAX, CompressionMethod, ENTRIES_COUNT_HARD_MAX,
    ENTRIES_COUNT_MAX_DEFAULT, ENTRY_UNCOMPRESSED_BYTES_MAX_DEFAULT, EncryptionKeyGuid, EntryFlags,
    EntryId, EntryMeta, FOOTER_BYTES_MAX, FOOTER_V8A_BYTES, FOOTER_V8B_V10_V11_BYTES,
    FOOTER_V9_V12_BYTES, INDEX_BYTES_MAX_DEFAULT, MOUNT_POINT_BYTES_MAX, OpenOptions,
    PATH_BYTES_MAX, PATH_UTF16_UNITS_MAX, PakDialect, PakLimits, PakVersion, PathHash,
    SCRATCH_IO_BYTES_INITIAL_DEFAULT, SCRATCH_IO_BYTES_MAX_DEFAULT, Sha1Hash,
};

const _: () = assert!(usize::BITS >= 32);

/// Canonical lossless bridge from `u32` domain counts/offsets to `usize` slice indices.
#[inline(always)]
#[must_use]
pub const fn u32_to_usize(x: u32) -> usize {
    x as usize
}

/// Extracts the lower 32 bits of a `u64` without narrowing `as` casts.
#[inline(always)]
#[must_use]
pub(crate) const fn low_u32(x: u64) -> u32 {
    let b = x.to_le_bytes();
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// Rounds `x` up to the next 16-byte AES block boundary, returning `None` on `u64` overflow.
#[inline]
#[must_use]
pub(crate) const fn align_up_16(x: u64) -> Option<u64> {
    let rem = x & 15;
    if rem == 0 {
        Some(x)
    } else {
        x.checked_add(16 - rem)
    }
}

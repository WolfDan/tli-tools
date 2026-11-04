//! High-performance, low-memory Virtual File System over Unreal Engine and Torchlight Infinite `.pak` archives.

mod arena;
mod builder;
mod cursor;
mod discover;
mod error;
mod lookup;
mod mount_alias;
mod path;
mod table;
mod types;
mod vfs;

#[cfg(test)]
mod tests;

pub use cursor::VfsCursor;
pub use error::{PathErrorKind, UePathErrorKind, VfsError, VfsIoAction, VfsLimitKind};
pub use pak::{
    AesKey, CompressionMethod, EncryptionKeyGuid, EntryId, EntryMeta, OpenOptions, PakDialect,
    PakError, PakLimits, PakVersion, ReadScratch, ScratchLimits, Sha1Hash,
};
pub use table::{DirFilesIter, DirSubdirsIter, DirView};
pub use types::{
    ARENA_BYTES_HARD_MAX, ARENA_BYTES_MAX_DEFAULT, DIRS_COUNT_HARD_MAX, DIRS_COUNT_MAX_DEFAULT,
    DirId, EXTS_COUNT_HARD_MAX, EXTS_COUNT_MAX_DEFAULT, FILES_COUNT_HARD_MAX,
    FILES_COUNT_MAX_DEFAULT, FileId, MOUNT_ALIASES_COUNT_HARD_MAX, MOUNT_ALIASES_COUNT_MAX_DEFAULT,
    MountAlias, MountSource, MountTier, PAKS_COUNT_HARD_MAX, PAKS_COUNT_MAX_DEFAULT,
    PATH_DEPTH_HARD_MAX, PATH_DEPTH_MAX_DEFAULT, PackageKind, PakId, PakMountInfo, ResolvedPackage,
    VFS_PATH_BYTES_HARD_MAX, VFS_PATH_BYTES_MAX_DEFAULT, VfsBatchOutcome, VfsFileMeta, VfsLimits,
    VfsOpenOptions, VfsReadRequest,
};
pub use vfs::{ExtFileIter, MountAliasIter, PakVfs, SubtreeFileIter};

const _: () = assert!(usize::BITS >= 32);

/// Canonical lossless bridge from `u32` domain indices and sizes to `usize` slice bounds.
#[inline(always)]
#[must_use]
pub const fn u32_to_usize(x: u32) -> usize {
    x as usize
}

/// Converts a bounded `usize` slice length (`<= u32::MAX`) to `u32` with an explicit invariant assertion.
#[inline(always)]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "Invariant: bounded VFS table and arena lengths are validated <= u32::MAX"
)]
pub(crate) const fn bounded_len_u32(len: usize) -> u32 {
    assert!(len <= u32_to_usize(u32::MAX));
    len as u32
}

/// Converts a bounded `usize` slice length (`<= u16::MAX`) to `u16` with an explicit invariant assertion.
#[inline(always)]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "Invariant: bounded archive, extension, and mount alias counts are validated <= u16::MAX"
)]
pub(crate) const fn bounded_len_u16(len: usize) -> u16 {
    assert!(len <= u32_to_usize(0xFFFF));
    len as u16
}

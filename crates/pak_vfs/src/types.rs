//! Domain newtypes, explicit `_MAX` bounds, Unreal package descriptors, and batch I/O types.

use core::num::NonZeroU16;
use std::path::Path;

use pak::{AesKey, CompressionMethod, EntryId, EntryMeta, OpenOptions, PakDialect, PakVersion};

use crate::error::{VfsError, VfsLimitKind};
use crate::u32_to_usize;

/// Default upper bound on mounted `.pak` archives (`1,024`).
pub const PAKS_COUNT_MAX_DEFAULT: u16 = 1_024;
/// Hard ceiling on mounted `.pak` archives (`65,535`).
pub const PAKS_COUNT_HARD_MAX: u16 = u16::MAX;

/// Default upper bound on virtual directories (`262,144`).
pub const DIRS_COUNT_MAX_DEFAULT: u32 = 262_144;
/// Hard ceiling on virtual directories so power-of-two `u32` hash tables cannot overflow.
pub const DIRS_COUNT_HARD_MAX: u32 = 1 << 26;

/// Default upper bound on unique winning virtual files (`4,194,304`).
pub const FILES_COUNT_MAX_DEFAULT: u32 = 4_194_304;
/// Hard ceiling on unique winning virtual files so power-of-two `u32` hash tables cannot overflow.
pub const FILES_COUNT_HARD_MAX: u32 = 1 << 28;

/// Default upper bound on distinct case-preserved file extensions (`4,096`).
pub const EXTS_COUNT_MAX_DEFAULT: u16 = 4_096;
/// Hard ceiling on distinct file extensions (`65,534`, reserving `0` for `Option<ExtSlotId>` niche).
pub const EXTS_COUNT_HARD_MAX: u16 = u16::MAX - 1;

/// Default upper bound on deduplicated VFS string arena capacity (`64 MiB`; TLI uses `~5.18 MiB`).
pub const ARENA_BYTES_MAX_DEFAULT: u32 = 64 * 1024 * 1024;
/// Hard ceiling on VFS string arena capacity (`1 GiB`).
pub const ARENA_BYTES_HARD_MAX: u32 = 1024 * 1024 * 1024;

/// Default upper bound on normalized root-relative virtual path UTF-8 byte length (`2,048`).
pub const VFS_PATH_BYTES_MAX_DEFAULT: u32 = 2_048;
/// Hard ceiling on normalized virtual path UTF-8 byte length (`4,096`).
pub const VFS_PATH_BYTES_HARD_MAX: u32 = 4_096;

/// Default upper bound on virtual directory nesting depth (`64`).
pub const PATH_DEPTH_MAX_DEFAULT: u16 = 64;
/// Hard ceiling on directory nesting depth matching inline `ArrayVec<_, 64>` traversal stacks.
pub const PATH_DEPTH_HARD_MAX: u16 = 64;

/// Default upper bound on discovered Unreal logical mount aliases (`1,024`).
pub const MOUNT_ALIASES_COUNT_MAX_DEFAULT: u16 = 1_024;
/// Hard ceiling on discovered Unreal logical mount aliases (`16,384`).
pub const MOUNT_ALIASES_COUNT_HARD_MAX: u16 = 16_384;

/// Dense 0-based handle into `PakVfs`'s mounted archive table (`0..pak_count`, highest priority `0`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PakId(u16);

impl PakId {
    #[inline]
    #[must_use]
    pub(crate) const fn new(index: u16) -> Self {
        Self(index)
    }

    /// Returns the 0-based dense priority index of this mounted archive (`0` = highest priority).
    #[inline]
    #[must_use]
    pub const fn index(self) -> u16 {
        self.0
    }
}

/// Dense 0-based handle into `PakVfs`'s contiguous BFS directory tree (`0..dir_count`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirId(u32);

impl DirId {
    /// Virtual root directory (`""`), guaranteed to occupy index `0` in every [`crate::PakVfs`].
    pub const ROOT: Self = Self(0);

    #[inline]
    #[must_use]
    pub(crate) const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the 0-based dense index of this directory within [`crate::PakVfs`].
    #[inline]
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }

    /// Returns `true` if this handle refers to [`DirId::ROOT`].
    #[inline]
    #[must_use]
    pub const fn is_root(self) -> bool {
        self.0 == 0
    }
}

/// Dense 0-based handle into `PakVfs`'s access-temperature-grouped winning file table (`0..file_count`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(u32);

impl FileId {
    #[inline]
    #[must_use]
    pub(crate) const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the 0-based dense index of this winning virtual file within [`crate::PakVfs`].
    #[inline]
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// Niche-optimized 1-based handle into `ExtTable` so `Option<ExtSlotId>` occupies `2` bytes.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ExtSlotId(NonZeroU16);
const _: () = assert!(core::mem::size_of::<Option<ExtSlotId>>() == 2);

impl ExtSlotId {
    #[inline]
    #[must_use]
    pub(crate) const fn try_from_index(index: u16) -> Option<Self> {
        match index.checked_add(1) {
            Some(nz) => match NonZeroU16::new(nz) {
                Some(id) => Some(Self(id)),
                None => None,
            },
            None => None,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) const fn index(self) -> u16 {
        self.0.get().strict_sub(1)
    }
}

/// Validated explicit `_MAX` bounds enforced during [`crate::PakVfs`] construction and queries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VfsLimits {
    paks_count_max: u16,
    dirs_count_max: u32,
    files_count_max: u32,
    exts_count_max: u16,
    arena_bytes_max: u32,
    path_bytes_max: u32,
    path_depth_max: u16,
    mount_aliases_count_max: u16,
}

impl VfsLimits {
    pub const DEFAULT: Self = Self {
        paks_count_max: PAKS_COUNT_MAX_DEFAULT,
        dirs_count_max: DIRS_COUNT_MAX_DEFAULT,
        files_count_max: FILES_COUNT_MAX_DEFAULT,
        exts_count_max: EXTS_COUNT_MAX_DEFAULT,
        arena_bytes_max: ARENA_BYTES_MAX_DEFAULT,
        path_bytes_max: VFS_PATH_BYTES_MAX_DEFAULT,
        path_depth_max: PATH_DEPTH_MAX_DEFAULT,
        mount_aliases_count_max: MOUNT_ALIASES_COUNT_MAX_DEFAULT,
    };

    pub fn with_paks_count_max(mut self, max: u16) -> Result<Self, VfsError> {
        validate_nonzero_u64(
            u64::from(max),
            u64::from(PAKS_COUNT_HARD_MAX),
            VfsLimitKind::PaksCount,
        )?;
        self.paks_count_max = max;
        Ok(self)
    }

    pub fn with_dirs_count_max(mut self, max: u32) -> Result<Self, VfsError> {
        validate_nonzero_u64(
            u64::from(max),
            u64::from(DIRS_COUNT_HARD_MAX),
            VfsLimitKind::DirsCount,
        )?;
        self.dirs_count_max = max;
        Ok(self)
    }

    pub fn with_files_count_max(mut self, max: u32) -> Result<Self, VfsError> {
        validate_nonzero_u64(
            u64::from(max),
            u64::from(FILES_COUNT_HARD_MAX),
            VfsLimitKind::FilesCount,
        )?;
        self.files_count_max = max;
        Ok(self)
    }

    pub fn with_exts_count_max(mut self, max: u16) -> Result<Self, VfsError> {
        validate_nonzero_u64(
            u64::from(max),
            u64::from(EXTS_COUNT_HARD_MAX),
            VfsLimitKind::ExtsCount,
        )?;
        self.exts_count_max = max;
        Ok(self)
    }

    pub fn with_arena_bytes_max(mut self, max: u32) -> Result<Self, VfsError> {
        validate_nonzero_u64(
            u64::from(max),
            u64::from(ARENA_BYTES_HARD_MAX),
            VfsLimitKind::ArenaBytes,
        )?;
        self.arena_bytes_max = max;
        Ok(self)
    }

    pub fn with_path_bytes_max(mut self, max: u32) -> Result<Self, VfsError> {
        validate_nonzero_u64(
            u64::from(max),
            u64::from(VFS_PATH_BYTES_HARD_MAX),
            VfsLimitKind::PathBytes,
        )?;
        self.path_bytes_max = max;
        Ok(self)
    }

    pub fn with_path_depth_max(mut self, max: u16) -> Result<Self, VfsError> {
        validate_nonzero_u64(
            u64::from(max),
            u64::from(PATH_DEPTH_HARD_MAX),
            VfsLimitKind::PathDepth,
        )?;
        self.path_depth_max = max;
        Ok(self)
    }

    pub fn with_mount_aliases_count_max(mut self, max: u16) -> Result<Self, VfsError> {
        validate_nonzero_u64(
            u64::from(max),
            u64::from(MOUNT_ALIASES_COUNT_HARD_MAX),
            VfsLimitKind::MountAliasesCount,
        )?;
        self.mount_aliases_count_max = max;
        Ok(self)
    }

    #[inline]
    #[must_use]
    pub const fn paks_count_max(self) -> u16 {
        self.paks_count_max
    }

    #[inline]
    #[must_use]
    pub const fn dirs_count_max(self) -> u32 {
        self.dirs_count_max
    }

    #[inline]
    #[must_use]
    pub const fn files_count_max(self) -> u32 {
        self.files_count_max
    }

    #[inline]
    #[must_use]
    pub const fn exts_count_max(self) -> u16 {
        self.exts_count_max
    }

    #[inline]
    #[must_use]
    pub const fn arena_bytes_max(self) -> u32 {
        self.arena_bytes_max
    }

    #[inline]
    #[must_use]
    pub const fn path_bytes_max(self) -> u32 {
        self.path_bytes_max
    }

    #[inline]
    #[must_use]
    pub const fn path_depth_max(self) -> u16 {
        self.path_depth_max
    }

    #[inline]
    #[must_use]
    pub const fn mount_aliases_count_max(self) -> u16 {
        self.mount_aliases_count_max
    }
}

fn validate_nonzero_u64(val: u64, hard_max: u64, kind: VfsLimitKind) -> Result<(), VfsError> {
    if val == 0 {
        return Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::LimitsConfig,
            actual: 0,
            max: 1,
        });
    }
    if val > hard_max {
        return Err(VfsError::LimitExceeded {
            kind,
            actual: val,
            max: hard_max,
        });
    }
    Ok(())
}

/// Configuration options for mounting a [`crate::PakVfs`].
#[derive(Clone, Debug)]
pub struct VfsOpenOptions {
    pak_options: OpenOptions,
    limits: VfsLimits,
    game_project_name: Option<Box<str>>,
}

impl VfsOpenOptions {
    #[inline]
    #[must_use]
    pub const fn new(key: Option<AesKey>) -> Self {
        Self {
            pak_options: OpenOptions::new(key),
            limits: VfsLimits::DEFAULT,
            game_project_name: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn with_pak_options(mut self, pak_options: OpenOptions) -> Self {
        self.pak_options = pak_options;
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_verify_index_hashes(mut self, verify: bool) -> Self {
        self.pak_options = self.pak_options.with_verify_index_hashes(verify);
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_limits(mut self, limits: VfsLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Overrides automatic `/Game` project-root detection with an explicit top-level directory name
    /// (for example `"UE_game"` so `/Game/...` maps to `"UE_game/Content/..."`).
    pub fn with_game_project_name(mut self, project_name: &str) -> Result<Self, VfsError> {
        if project_name.is_empty()
            || project_name == "."
            || project_name == ".."
            || project_name.len() > u32_to_usize(VFS_PATH_BYTES_HARD_MAX)
            || project_name.contains(['/', '\\', '\0'])
        {
            return Err(VfsError::InvalidProjectName);
        }
        self.game_project_name = Some(project_name.to_owned().into_boxed_str());
        Ok(self)
    }

    #[inline]
    #[must_use]
    pub const fn pak_options(&self) -> &OpenOptions {
        &self.pak_options
    }

    #[inline]
    #[must_use]
    pub const fn limits(&self) -> VfsLimits {
        self.limits
    }

    #[inline]
    #[must_use]
    pub fn game_project_name(&self) -> Option<&str> {
        self.game_project_name.as_deref()
    }
}

/// Priority classification of a mounted `.pak` archive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountTier {
    /// Hot-update patch archive from `PreMountPak` (`UE_game_<N>_P.pak`).
    PreMountPatch { patch_version: u64 },
    /// Patch archive (`*_P.pak`) located in `Content/Paks`.
    GameContentPatch { patch_version: u64 },
    /// Base or chunk archive located in `Content/Paks`.
    GameContentChunk,
    /// Engine or plugin archive located in `Engine/Plugins/Paks`.
    EnginePlugin,
    /// Caller-supplied archive mounted via [`crate::PakVfs::open_mounts`] (`0` = highest priority).
    Explicit { priority_rank: u16 },
}

/// Explicit archive mount descriptor passed to [`crate::PakVfs::open_mounts`].
#[derive(Clone, Copy, Debug)]
pub struct MountSource<'a> {
    path: &'a Path,
}

impl<'a> MountSource<'a> {
    #[inline]
    #[must_use]
    pub const fn new(path: &'a Path) -> Self {
        Self { path }
    }

    #[inline]
    #[must_use]
    pub const fn path(self) -> &'a Path {
        self.path
    }
}

/// Read-only metadata snapshot describing one mounted `.pak` archive inside [`crate::PakVfs`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PakMountInfo<'a> {
    pub(crate) pak_id: PakId,
    pub(crate) tier: MountTier,
    pub(crate) path: &'a Path,
    pub(crate) mount_point: &'a str,
    pub(crate) dialect: PakDialect,
    pub(crate) version: PakVersion,
    pub(crate) file_bytes: u64,
    pub(crate) total_entry_count: u32,
    pub(crate) winning_file_count: u32,
}

impl<'a> PakMountInfo<'a> {
    #[inline]
    #[must_use]
    pub const fn pak_id(&self) -> PakId {
        self.pak_id
    }

    #[inline]
    #[must_use]
    pub const fn tier(&self) -> MountTier {
        self.tier
    }

    #[inline]
    #[must_use]
    pub const fn path(&self) -> &'a Path {
        self.path
    }

    #[inline]
    #[must_use]
    pub const fn mount_point(&self) -> &'a str {
        self.mount_point
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
    pub const fn file_bytes(&self) -> u64 {
        self.file_bytes
    }

    #[inline]
    #[must_use]
    pub const fn total_entry_count(&self) -> u32 {
        self.total_entry_count
    }

    #[inline]
    #[must_use]
    pub const fn winning_file_count(&self) -> u32 {
        self.winning_file_count
    }
}

/// Read-only metadata snapshot describing a single winning virtual file in [`crate::PakVfs`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VfsFileMeta {
    pub(crate) file: FileId,
    pub(crate) dir: DirId,
    pub(crate) pak: PakId,
    pub(crate) entry: EntryId,
    pub(crate) entry_meta: EntryMeta,
}

impl VfsFileMeta {
    #[inline]
    #[must_use]
    pub const fn file_id(&self) -> FileId {
        self.file
    }

    #[inline]
    #[must_use]
    pub const fn dir_id(&self) -> DirId {
        self.dir
    }

    #[inline]
    #[must_use]
    pub const fn pak_id(&self) -> PakId {
        self.pak
    }

    #[inline]
    #[must_use]
    pub const fn entry_id(&self) -> EntryId {
        self.entry
    }

    #[inline]
    #[must_use]
    pub const fn entry_meta(&self) -> EntryMeta {
        self.entry_meta
    }

    #[inline]
    #[must_use]
    pub const fn uncompressed_bytes(&self) -> u64 {
        self.entry_meta.uncompressed_bytes()
    }

    #[inline]
    #[must_use]
    pub const fn compressed_bytes(&self) -> u64 {
        self.entry_meta.compressed_bytes()
    }

    #[inline]
    #[must_use]
    pub const fn disk_span_bytes(&self) -> u64 {
        self.entry_meta.disk_span_bytes()
    }

    #[inline]
    #[must_use]
    pub const fn compression_method(&self) -> CompressionMethod {
        self.entry_meta.compression_method()
    }

    #[inline]
    #[must_use]
    pub const fn is_encrypted(&self) -> bool {
        self.entry_meta.is_encrypted()
    }
}

/// Discriminates Unreal `.uasset` data/blueprint packages from `.umap` level packages.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageKind {
    Asset = 0,
    Map = 1,
}

/// Co-located Unreal Engine cooked package files resolved in `O(1)` time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedPackage {
    pub(crate) kind: PackageKind,
    pub(crate) header: FileId,
    pub(crate) uexp: Option<FileId>,
    pub(crate) ubulk: Option<FileId>,
    pub(crate) uptnl: Option<FileId>,
}

impl ResolvedPackage {
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> PackageKind {
        self.kind
    }

    /// Returns the primary package header file (`.uasset` or `.umap`).
    #[inline]
    #[must_use]
    pub const fn header(&self) -> FileId {
        self.header
    }

    /// Returns the cooked export payload sidecar (`.uexp`), if present.
    #[inline]
    #[must_use]
    pub const fn uexp(&self) -> Option<FileId> {
        self.uexp
    }

    /// Returns the bulk-data sidecar (`.ubulk`), if present.
    #[inline]
    #[must_use]
    pub const fn ubulk(&self) -> Option<FileId> {
        self.ubulk
    }

    /// Returns the optional bulk-data sidecar (`.uptnl`), if present.
    #[inline]
    #[must_use]
    pub const fn uptnl(&self) -> Option<FileId> {
        self.uptnl
    }
}

/// Discovered Unreal Engine logical mount alias (for example `"Game"` -> `UE_game/Content`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MountAlias<'a> {
    pub(crate) alias: &'a str,
    pub(crate) target_dir: DirId,
}

impl<'a> MountAlias<'a> {
    /// Logical mount name without leading or trailing slashes (for example `"Game"` or `"Engine"`).
    #[inline]
    #[must_use]
    pub const fn alias(&self) -> &'a str {
        self.alias
    }

    /// Physical virtual directory corresponding to `/<alias>/`.
    #[inline]
    #[must_use]
    pub const fn target_dir(&self) -> DirId {
        self.target_dir
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReadWindow {
    WholeFile,
    ByteRange { offset: u64 },
}

/// Caller-owned destination descriptor for [`crate::PakVfs::read_batch_into`].
#[derive(Debug)]
pub struct VfsReadRequest<'a> {
    pub(crate) file: FileId,
    pub(crate) window: ReadWindow,
    pub(crate) dst: &'a mut [u8],
}

impl<'a> VfsReadRequest<'a> {
    /// Reads the entire uncompressed file `file` into `dst` (which must match `uncompressed_bytes`).
    #[inline]
    #[must_use]
    pub fn whole_file(file: FileId, dst: &'a mut [u8]) -> Self {
        Self {
            file,
            window: ReadWindow::WholeFile,
            dst,
        }
    }

    /// Reads the uncompressed sub-range `[offset .. offset + dst.len())` of `file` into `dst`.
    #[inline]
    #[must_use]
    pub fn byte_range(file: FileId, offset: u64, dst: &'a mut [u8]) -> Self {
        Self {
            file,
            window: ReadWindow::ByteRange { offset },
            dst,
        }
    }

    #[inline]
    #[must_use]
    pub const fn file(&self) -> FileId {
        self.file
    }
}

/// Per-item outcome written by [`crate::PakVfs::read_batch_into`].
#[must_use]
#[derive(Debug)]
pub enum VfsBatchOutcome {
    /// Initial placeholder state before [`crate::PakVfs::read_batch_into`] populates the slot.
    Pending,
    /// Requested file or sub-range was read, decrypted, and decompressed into `dst`.
    Ok { bytes_read: u64 },
    /// Reading or decompressing this specific request failed without aborting the batch.
    Err(VfsError),
}

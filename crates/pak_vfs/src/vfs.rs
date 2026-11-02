//! Immutable access-temperature-grouped Virtual File System (`PakVfs`) across mounted `.pak` archives.

use std::path::Path;

use arrayvec::ArrayVec;
use pak::{AesKey, CompressionMethod, ReadScratch};

use crate::arena::{ExtTable, StringArena};
use crate::builder::{MountInput, MountedPakRecord, build_pak_vfs};
use crate::cursor::VfsCursor;
use crate::discover::discover_tli_paks;
use crate::error::{VfsError, VfsLimitKind};
use crate::lookup::DirLookupTable;
use crate::mount_alias::MountAliasTable;
use crate::path::{
    DirPrefixState, ParsedFileQuery, parse_dir_query, parse_file_query, parse_relative_dir_tail,
    parse_relative_file_tail, split_stem_ext, split_ue_logical_path,
};
use crate::table::{DirFilesIter, DirSubdirsIter, DirTable, DirView, FileTable};
use crate::types::{
    DirId, ExtSlotId, FileId, MountAlias, MountSource, MountTier, PATH_DEPTH_HARD_MAX, PakId,
    PakMountInfo, ReadWindow, ResolvedPackage, VfsBatchOutcome, VfsFileMeta, VfsLimits,
    VfsOpenOptions, VfsReadRequest,
};
use crate::{bounded_len_u16, bounded_len_u32, u32_to_usize};

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<PakVfs>();

const BATCH_SORT_CHUNK_MAX: u32 = 64;

/// Immutable, thread-safe Virtual File System unifying an ordered stack of Unreal `.pak` archives.
///
/// Retains strictly winning `(PakId, EntryId)` pointers, a flat BFS directory tree (`DirTable`),
/// an access-temperature-grouped file catalog (`FileTable`), a per-directory stem-deduplicated
/// string arena, and a `<= 50%` load-factor open-addressed directory lookup table.
#[derive(Debug)]
pub struct PakVfs {
    paks: Box<[MountedPakRecord]>,
    dirs: DirTable,
    files: FileTable,
    exts: ExtTable,
    arena: StringArena,
    dir_lookup: DirLookupTable,
    mount_aliases: MountAliasTable,
    limits: VfsLimits,
}

impl PakVfs {
    /// Discovers and mounts all `.pak` archives under a Torchlight Infinite installation root
    /// (`PreMountPak` patches -> `UE_game/Content/Paks` -> `Engine/Plugins/Paks`).
    pub fn open_tli_game_dir(game_root: &Path, key: Option<AesKey>) -> Result<Self, VfsError> {
        let options = VfsOpenOptions::new(key);
        Self::open_tli_game_dir_with_options(game_root, &options)
    }

    /// Discovers and mounts all `.pak` archives under `game_root` with explicit [`VfsOpenOptions`].
    pub fn open_tli_game_dir_with_options(
        game_root: &Path,
        options: &VfsOpenOptions,
    ) -> Result<Self, VfsError> {
        let discovered = discover_tli_paks(game_root, options.limits())?;
        let mut mounts = Vec::with_capacity(discovered.len());
        for item in discovered {
            mounts.push(MountInput {
                path: item.path,
                tier: item.tier,
            });
        }
        Self::from_mount_inputs(mounts, options)
    }

    /// Mounts an explicit caller-ordered slice of `.pak` archives (`mounts[0]` has highest override priority).
    ///
    /// Total over an empty slice (`&[]`), returning a valid empty [`PakVfs`] containing only [`DirId::ROOT`].
    pub fn open_mounts(
        mounts: &[MountSource<'_>],
        options: &VfsOpenOptions,
    ) -> Result<Self, VfsError> {
        let limits = options.limits();
        let count_u16 = match u16::try_from(mounts.len()) {
            Ok(c) if c <= limits.paks_count_max() => c,
            Ok(_) | Err(_) => {
                let actual = u64::try_from(mounts.len()).map_err(|_| VfsError::LimitExceeded {
                    kind: VfsLimitKind::PaksCount,
                    actual: u64::MAX,
                    max: u64::from(limits.paks_count_max()),
                })?;
                return Err(VfsError::LimitExceeded {
                    kind: VfsLimitKind::PaksCount,
                    actual,
                    max: u64::from(limits.paks_count_max()),
                });
            }
        };
        let mut inputs = Vec::with_capacity(mounts.len());
        for (rank, src) in (0..count_u16).zip(mounts.iter()) {
            inputs.push(MountInput {
                path: src.path().to_path_buf(),
                tier: MountTier::Explicit {
                    priority_rank: rank,
                },
            });
        }
        Self::from_mount_inputs(inputs, options)
    }

    fn from_mount_inputs(
        mounts: Vec<MountInput>,
        options: &VfsOpenOptions,
    ) -> Result<Self, VfsError> {
        let built = build_pak_vfs(mounts, options)?;
        Ok(Self {
            paks: built.paks,
            dirs: built.dirs,
            files: built.files,
            exts: built.exts,
            arena: built.arena,
            dir_lookup: built.dir_lookup,
            mount_aliases: built.mount_aliases,
            limits: built.limits,
        })
    }

    /// Number of mounted `.pak` archives (`0..=paks_count_max`).
    #[inline]
    #[must_use]
    pub const fn pak_count(&self) -> u16 {
        bounded_len_u16(self.paks.len())
    }

    /// Validates a 0-based archive index against `0..pak_count`.
    #[inline]
    #[must_use]
    pub const fn pak_id_at(&self, index: u16) -> Option<PakId> {
        if index < self.pak_count() {
            Some(PakId::new(index))
        } else {
            None
        }
    }

    /// Returns metadata describing mounted archive `pak`.
    #[inline]
    #[must_use]
    pub fn pak_info(&self, pak: PakId) -> PakMountInfo<'_> {
        let rec = self.pak_record(pak);
        PakMountInfo {
            pak_id: pak,
            tier: rec.tier,
            path: &rec.path,
            mount_point: &rec.mount_point,
            dialect: rec.reader.dialect(),
            version: rec.reader.version(),
            file_bytes: rec.reader.file_bytes(),
            total_entry_count: rec.reader.entry_count(),
            winning_file_count: rec.winning_file_count,
        }
    }

    /// Total number of virtual directories, including [`DirId::ROOT`] (`>= 1`).
    #[inline]
    #[must_use]
    pub const fn dir_count(&self) -> u32 {
        self.dirs.len()
    }

    /// Validates a 0-based directory index against `0..dir_count`.
    #[inline]
    #[must_use]
    pub const fn dir_id_at(&self, index: u32) -> Option<DirId> {
        if index < self.dirs.len() {
            Some(DirId::new(index))
        } else {
            None
        }
    }

    /// Total number of unique winning virtual files across all mounted archives.
    #[inline]
    #[must_use]
    pub const fn file_count(&self) -> u32 {
        self.files.len()
    }

    /// Validates a 0-based file index against `0..file_count`.
    #[inline]
    #[must_use]
    pub const fn file_id_at(&self, index: u32) -> Option<FileId> {
        if index < self.files.len() {
            Some(FileId::new(index))
        } else {
            None
        }
    }

    /// Total UTF-8 bytes stored in the deduplicated VFS string arena (`~5.18 MiB` for TLI).
    #[inline]
    #[must_use]
    pub const fn arena_bytes(&self) -> u32 {
        self.arena.byte_len()
    }

    /// Number of distinct case-preserved file extension spellings interned in [`PakVfs`].
    #[inline]
    #[must_use]
    pub const fn distinct_extension_count(&self) -> u16 {
        self.exts.count()
    }

    /// Number of discovered Unreal Engine logical mount aliases (`/Game`, `/Engine`, `/<Plugin>`).
    #[inline]
    #[must_use]
    pub const fn mount_alias_count(&self) -> u16 {
        self.mount_aliases.len()
    }

    /// Exact-size iterator over all discovered Unreal logical mount aliases in sorted order.
    #[inline]
    #[must_use]
    pub const fn iter_mount_aliases(&self) -> MountAliasIter<'_> {
        MountAliasIter {
            vfs: self,
            next_idx: 0,
            end_idx: self.mount_aliases.len(),
        }
    }

    /// Looks up a winning file by root-relative physical virtual path (case-insensitive, `/` and `\` normalized)
    /// via `O(1)` directory resolution and top-down slice narrowing with zero heap allocation.
    pub fn find(&self, vfs_path: &str) -> Result<Option<FileId>, VfsError> {
        let parsed = parse_file_query(vfs_path, self.limits)?;
        Ok(self.find_parsed_file(&parsed))
    }

    /// Looks up a directory by root-relative physical virtual path (`""` or `"/"` resolves to [`DirId::ROOT`])
    /// in `O(1)` time with zero heap allocation.
    pub fn find_dir(&self, vfs_dir_path: &str) -> Result<Option<DirId>, VfsError> {
        let parsed = parse_dir_query(vfs_dir_path, self.limits)?;
        Ok(self.dir_lookup.find(
            parsed.base_dir,
            &parsed.rel_segments,
            parsed.dir_hash,
            &self.dirs,
            &self.arena,
        ))
    }

    /// Resolves an Unreal cooked package (`.uasset` or `.umap` header plus co-located `.uexp`, `.ubulk`,
    /// and `.uptnl` companions) by root-relative physical path (with or without `.uasset`/`.umap` suffix).
    pub fn resolve_package(
        &self,
        vfs_package_path: &str,
    ) -> Result<Option<ResolvedPackage>, VfsError> {
        let parsed = parse_file_query(vfs_package_path, self.limits)?;
        Ok(self.resolve_package_leaf(&parsed))
    }

    /// Assembles the [`ResolvedPackage`] containing `file` if `file` belongs to a cooked `.uasset`/`.umap`
    /// stem group in its directory.
    #[must_use]
    pub fn package_for_file(&self, file: FileId) -> Option<ResolvedPackage> {
        self.assert_valid_file(file);
        self.files.package_around(file, &self.dirs, &self.arena)
    }

    /// Resolves a bare Unreal mount alias name (such as `"Game"`, `"Engine"`, or `"AntiAddiction"`,
    /// with optional leading/trailing slashes stripped) to its target [`DirId`].
    #[must_use]
    pub fn resolve_ue_mount_root(&self, alias: &str) -> Option<DirId> {
        let trimmed = alias.trim_matches(['/', '\\']);
        if trimmed.is_empty() {
            return None;
        }
        self.mount_aliases.resolve(trimmed, &self.arena)
    }

    /// Looks up a winning file by Unreal logical mount path (such as `"/Game/Art/Hero.uasset"`,
    /// `"/Engine/..."`, or `"/<PluginName>/..."`) with zero heap allocation.
    pub fn find_ue(&self, ue_path: &str) -> Result<Option<FileId>, VfsError> {
        let (alias, rel_path) = split_ue_logical_path(ue_path)?;
        let Some(prefix) = self.mount_aliases.resolve_prefix(alias, &self.arena) else {
            let _validated = parse_relative_file_tail(DirPrefixState::ROOT, rel_path, self.limits)?;
            return Ok(None);
        };
        let parsed = parse_relative_file_tail(prefix, rel_path, self.limits)?;
        Ok(self.find_parsed_file(&parsed))
    }

    /// Looks up a virtual directory by Unreal logical mount path (such as `"/Game"`, `"/Game/UI"`,
    /// or `"/Engine/Materials"`) in `O(1)` time with zero heap allocation.
    pub fn find_ue_dir(&self, ue_dir_path: &str) -> Result<Option<DirId>, VfsError> {
        let (alias, rel_path) = split_ue_logical_path(ue_dir_path)?;
        let Some(prefix) = self.mount_aliases.resolve_prefix(alias, &self.arena) else {
            let _validated = parse_relative_dir_tail(DirPrefixState::ROOT, rel_path, self.limits)?;
            return Ok(None);
        };
        let parsed = parse_relative_dir_tail(prefix, rel_path, self.limits)?;
        Ok(self.dir_lookup.find(
            parsed.base_dir,
            &parsed.rel_segments,
            parsed.dir_hash,
            &self.dirs,
            &self.arena,
        ))
    }

    /// Resolves an Unreal cooked package by logical mount path (such as `"/Game/Art/Hero"`,
    /// `"/Game/Art/Hero.uasset"`, or `"/Game/Maps/Town.umap"`) in `O(1)` time with zero heap allocation.
    pub fn resolve_ue_package(
        &self,
        ue_package_path: &str,
    ) -> Result<Option<ResolvedPackage>, VfsError> {
        let (alias, rel_path) = split_ue_logical_path(ue_package_path)?;
        let Some(prefix) = self.mount_aliases.resolve_prefix(alias, &self.arena) else {
            let _validated = parse_relative_file_tail(DirPrefixState::ROOT, rel_path, self.limits)?;
            return Ok(None);
        };
        let parsed = parse_relative_file_tail(prefix, rel_path, self.limits)?;
        Ok(self.resolve_package_leaf(&parsed))
    }

    /// Returns a zero-allocation [`DirView`] listing the sorted direct subdirectories and files of `dir`.
    #[inline]
    #[must_use]
    pub fn read_dir(&self, dir: DirId) -> DirView {
        self.assert_valid_dir(dir);
        self.dirs.view(dir)
    }

    /// Exact-size iterator over every winning [`FileId`] (`0..file_count`) in BFS directory-grouped order.
    #[inline]
    #[must_use]
    pub const fn iter_files(&self) -> DirFilesIter {
        DirFilesIter {
            next_idx: 0,
            end_idx: self.files.len(),
        }
    }

    /// Pre-order DFS iterator over all winning [`FileId`]s inside `root_dir` and all its descendant
    /// subdirectories, backed by an inline `ArrayVec` range stack with zero recursion or heap allocation.
    #[must_use]
    pub fn iter_subtree_files(&self, root_dir: DirId) -> SubtreeFileIter<'_> {
        self.assert_valid_dir(root_dir);
        let root_view = self.dirs.view(root_dir);
        let mut stack = ArrayVec::<DirSubdirsIter, { PATH_DEPTH_HARD_MAX as usize }>::new();
        let subdirs = root_view.subdirs();
        if subdirs.len() > 0 {
            let pushed = stack.try_push(subdirs);
            assert!(pushed.is_ok());
        }
        SubtreeFileIter {
            dirs: &self.dirs,
            current_files: root_view.files(),
            stack,
        }
    }

    /// Zero-allocation iterator over all winning [`FileId`]s matching `ext` case-insensitively
    /// (accepting both `"uasset"` and `".uasset"`).
    #[must_use]
    pub fn iter_files_with_ext(&self, ext: &str) -> ExtFileIter<'_> {
        let target_canonical = self.exts.find_canonical_slot(ext, &self.arena);
        ExtFileIter {
            files: &self.files,
            exts: &self.exts,
            target_canonical,
            next_idx: 0,
        }
    }

    /// Case-preserved single segment name of `dir` (`""` for [`DirId::ROOT`]).
    #[inline]
    #[must_use]
    pub fn dir_name(&self, dir: DirId) -> &str {
        self.assert_valid_dir(dir);
        self.dirs.name(dir, &self.arena)
    }

    /// Parent directory [`DirId`] containing `file`.
    #[inline]
    #[must_use]
    pub fn file_dir(&self, file: FileId) -> DirId {
        self.assert_valid_file(file);
        self.files.dir(file)
    }

    /// Case-preserved filename stem of `file` (for `"Hero.v2.uasset"`, returns `"Hero.v2"`).
    #[inline]
    #[must_use]
    pub fn file_stem(&self, file: FileId) -> &str {
        self.assert_valid_file(file);
        self.files.stem(file, &self.arena)
    }

    /// Case-preserved extension of `file` without the leading dot (or `None` for extensionless files).
    #[inline]
    #[must_use]
    pub fn file_extension(&self, file: FileId) -> Option<&str> {
        self.assert_valid_file(file);
        self.files.extension(file, &self.exts, &self.arena)
    }

    /// Appends the canonical case-preserved root-relative directory path of `dir` to `out`
    /// without clearing `out` (appends `0` bytes for [`DirId::ROOT`]).
    pub fn write_dir_path(&self, dir: DirId, out: &mut Vec<u8>) {
        self.assert_valid_dir(dir);
        self.dirs.write_dir_path(dir, &self.arena, out);
    }

    /// Appends the canonical case-preserved root-relative virtual path of `file` to `out`
    /// without clearing `out`.
    pub fn write_file_path(&self, file: FileId, out: &mut Vec<u8>) {
        self.assert_valid_file(file);
        self.files
            .write_file_path(file, &self.dirs, &self.exts, &self.arena, out);
    }

    /// Returns archive, entry, size, compression, and encryption metadata for winning file `file`.
    #[inline]
    #[must_use]
    pub fn file_meta(&self, file: FileId) -> VfsFileMeta {
        self.assert_valid_file(file);
        let eref = self.files.entry_ref(file);
        let rec = self.pak_record(eref.pak_id);
        let entry_meta = rec.reader.entry_meta(eref.entry_id);
        VfsFileMeta {
            file,
            dir: eref.dir,
            pak: eref.pak_id,
            entry: eref.entry_id,
            entry_meta,
        }
    }

    /// Reads, decrypts, and decompresses the entire winning file `file` into `dst`
    /// (`dst.len()` must equal `file_meta(file).uncompressed_bytes()`).
    pub fn read_into(
        &self,
        file: FileId,
        scratch: &mut ReadScratch,
        dst: &mut [u8],
    ) -> Result<u64, VfsError> {
        self.assert_valid_file(file);
        let eref = self.files.entry_ref(file);
        let rec = self.pak_record(eref.pak_id);
        let meta = rec.reader.entry_meta(eref.entry_id);
        let dst_len_u64 =
            u64::try_from(dst.len()).map_err(|_| VfsError::DestinationSizeMismatch {
                file,
                expected: meta.uncompressed_bytes(),
                actual: u64::MAX,
            })?;
        if dst_len_u64 != meta.uncompressed_bytes() {
            return Err(VfsError::DestinationSizeMismatch {
                file,
                expected: meta.uncompressed_bytes(),
                actual: dst_len_u64,
            });
        }
        rec.reader
            .read_entry_into(eref.entry_id, scratch, dst)
            .map_err(|source| VfsError::Pak {
                pak_path: rec.path.to_path_buf(),
                source,
            })
    }

    /// Reads an arbitrary uncompressed sub-range `[offset .. offset + dst.len())` of `file` into `dst`,
    /// decrypting and decompressing strictly the intersecting `16`-byte AES or `64 KiB` codec blocks.
    pub fn read_range_into(
        &self,
        file: FileId,
        offset: u64,
        scratch: &mut ReadScratch,
        dst: &mut [u8],
    ) -> Result<(), VfsError> {
        self.assert_valid_file(file);
        let eref = self.files.entry_ref(file);
        let rec = self.pak_record(eref.pak_id);
        let meta = rec.reader.entry_meta(eref.entry_id);
        let file_bytes = meta.uncompressed_bytes();
        let dst_len_u64 = u64::try_from(dst.len()).map_err(|_| VfsError::OutOfBoundsRange {
            file,
            offset,
            byte_len: u64::MAX,
            file_bytes,
        })?;
        let req_end = offset
            .checked_add(dst_len_u64)
            .ok_or(VfsError::OutOfBoundsRange {
                file,
                offset,
                byte_len: dst_len_u64,
                file_bytes,
            })?;
        if req_end > file_bytes {
            return Err(VfsError::OutOfBoundsRange {
                file,
                offset,
                byte_len: dst_len_u64,
                file_bytes,
            });
        }
        rec.reader
            .read_entry_range_into(eref.entry_id, offset, scratch, dst)
            .map_err(|source| VfsError::Pak {
                pak_path: rec.path.to_path_buf(),
                source,
            })
    }

    /// Executes a batch of whole-file and sub-range reads using 64-item stack-sorted `(PakId, disk_offset)`
    /// elevator scheduling with zero heap allocation.
    ///
    /// Total over empty slices (`&mut []`, `&mut []`); aborts on internal contract violation if
    /// `requests.len() != outcomes.len()` or any [`FileId`] is out of bounds.
    pub fn read_batch_into(
        &self,
        requests: &mut [VfsReadRequest<'_>],
        scratch: &mut ReadScratch,
        outcomes: &mut [VfsBatchOutcome],
    ) {
        assert_eq!(
            requests.len(),
            outcomes.len(),
            "requests and outcomes slices must have identical length"
        );
        for req in requests.iter() {
            self.assert_valid_file(req.file);
        }
        let chunk_max = u32_to_usize(BATCH_SORT_CHUNK_MAX);
        for (req_chunk, out_chunk) in requests
            .chunks_mut(chunk_max)
            .zip(outcomes.chunks_mut(chunk_max))
        {
            self.execute_sorted_batch_chunk(req_chunk, scratch, out_chunk);
        }
    }

    /// Opens a zero-allocation streaming [`VfsCursor`] (`std::io::Read + std::io::Seek`) over `file`,
    /// backed by `scratch`'s single-block decoded window.
    #[inline]
    #[must_use]
    pub fn open_cursor<'vfs, 'scratch>(
        &'vfs self,
        file: FileId,
        scratch: &'scratch mut ReadScratch,
    ) -> VfsCursor<'vfs, 'scratch> {
        VfsCursor::new(self, scratch, file)
    }

    /// Verifies the stored inline `FPakEntry` SHA-1 digest of `file`'s raw on-disk payload bytes.
    pub fn verify_file_sha1(
        &self,
        file: FileId,
        scratch: &mut ReadScratch,
    ) -> Result<(), VfsError> {
        self.assert_valid_file(file);
        let eref = self.files.entry_ref(file);
        let rec = self.pak_record(eref.pak_id);
        rec.reader
            .verify_entry_sha1(eref.entry_id, scratch)
            .map_err(|source| VfsError::Pak {
                pak_path: rec.path.to_path_buf(),
                source,
            })
    }

    #[inline]
    fn pak_record(&self, pak: PakId) -> &MountedPakRecord {
        assert!(
            pak.index() < self.pak_count(),
            "PakId out of bounds for this PakVfs"
        );
        &self.paks[usize::from(pak.index())]
    }

    #[inline]
    fn assert_valid_dir(&self, dir: DirId) {
        assert!(
            dir.index() < self.dirs.len(),
            "DirId out of bounds for this PakVfs"
        );
    }

    #[inline]
    fn assert_valid_file(&self, file: FileId) {
        assert!(
            file.index() < self.files.len(),
            "FileId out of bounds for this PakVfs"
        );
    }

    #[inline]
    fn find_parsed_file(&self, parsed: &ParsedFileQuery<'_>) -> Option<FileId> {
        let dir = self.dir_lookup.find(
            parsed.base_dir,
            &parsed.rel_segments,
            parsed.dir_hash,
            &self.dirs,
            &self.arena,
        )?;
        let dir_range = self.dirs.file_range(dir);
        let (stem, ext) = split_stem_ext(parsed.leaf);
        self.files
            .find_in_dir(dir_range, stem, ext, &self.exts, &self.arena)
    }

    fn resolve_package_leaf(&self, parsed: &ParsedFileQuery<'_>) -> Option<ResolvedPackage> {
        let dir = self.dir_lookup.find(
            parsed.base_dir,
            &parsed.rel_segments,
            parsed.dir_hash,
            &self.dirs,
            &self.arena,
        )?;
        let dir_range = self.dirs.file_range(dir);
        let leaf = parsed.leaf;
        let (stem, ext) = split_stem_ext(leaf);
        let anchor = match ext {
            Some(e) if e.eq_ignore_ascii_case("uasset") => {
                self.files
                    .find_in_dir(dir_range, stem, Some("uasset"), &self.exts, &self.arena)
            }
            Some(e) if e.eq_ignore_ascii_case("umap") => {
                self.files
                    .find_in_dir(dir_range, stem, Some("umap"), &self.exts, &self.arena)
            }
            Some(_) | None => match self.files.find_in_dir(
                dir_range,
                leaf,
                Some("uasset"),
                &self.exts,
                &self.arena,
            ) {
                Some(asset_id) => Some(asset_id),
                None => {
                    self.files
                        .find_in_dir(dir_range, leaf, Some("umap"), &self.exts, &self.arena)
                }
            },
        }?;
        self.files.package_around(anchor, &self.dirs, &self.arena)
    }

    fn execute_sorted_batch_chunk(
        &self,
        req_chunk: &mut [VfsReadRequest<'_>],
        scratch: &mut ReadScratch,
        out_chunk: &mut [VfsBatchOutcome],
    ) {
        assert_eq!(req_chunk.len(), out_chunk.len());
        assert!(req_chunk.len() <= u32_to_usize(BATCH_SORT_CHUNK_MAX));
        let mut order = ArrayVec::<BatchOrderSlot, { BATCH_SORT_CHUNK_MAX as usize }>::new();
        let chunk_len = bounded_len_u32(req_chunk.len());
        for (local_idx, req) in (0..chunk_len).zip(req_chunk.iter()) {
            let eref = self.files.entry_ref(req.file);
            let meta = self
                .pak_record(eref.pak_id)
                .reader
                .entry_meta(eref.entry_id);
            let disk_offset = match req.window {
                ReadWindow::WholeFile => meta.payload_offset(),
                ReadWindow::ByteRange { offset }
                    if meta.compression_method() == CompressionMethod::None =>
                {
                    meta.payload_offset().saturating_add(offset)
                }
                ReadWindow::ByteRange { .. } => meta.payload_offset(),
            };
            let pushed = order.try_push(BatchOrderSlot {
                local_idx,
                pak_id: eref.pak_id,
                disk_offset,
            });
            assert!(pushed.is_ok());
        }
        order.sort_unstable_by_key(|s| (s.pak_id, s.disk_offset, s.local_idx));

        for slot in &order {
            let i = u32_to_usize(slot.local_idx);
            let req = &mut req_chunk[i];
            out_chunk[i] = match req.window {
                ReadWindow::WholeFile => match self.read_into(req.file, scratch, req.dst) {
                    Ok(bytes_read) => VfsBatchOutcome::Ok { bytes_read },
                    Err(err) => VfsBatchOutcome::Err(err),
                },
                ReadWindow::ByteRange { offset } => match u64::try_from(req.dst.len()) {
                    Ok(req_bytes) => match self.read_range_into(req.file, offset, scratch, req.dst)
                    {
                        Ok(()) => VfsBatchOutcome::Ok {
                            bytes_read: req_bytes,
                        },
                        Err(err) => VfsBatchOutcome::Err(err),
                    },
                    Err(_) => VfsBatchOutcome::Err(VfsError::OutOfBoundsRange {
                        file: req.file,
                        offset,
                        byte_len: u64::MAX,
                        file_bytes: self.file_meta(req.file).uncompressed_bytes(),
                    }),
                },
            };
        }
    }
}

#[derive(Clone, Copy)]
struct BatchOrderSlot {
    local_idx: u32,
    pak_id: PakId,
    disk_offset: u64,
}

/// Exact-size iterator over discovered [`MountAlias`] entries in a [`PakVfs`].
#[derive(Clone, Copy, Debug)]
pub struct MountAliasIter<'a> {
    vfs: &'a PakVfs,
    next_idx: u16,
    end_idx: u16,
}

impl<'a> Iterator for MountAliasIter<'a> {
    type Item = MountAlias<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.end_idx {
            return None;
        }
        let idx = self.next_idx;
        self.next_idx = self.next_idx.strict_add(1);
        Some(self.vfs.mount_aliases.get(idx, &self.vfs.arena))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = usize::from(self.end_idx.strict_sub(self.next_idx));
        (rem, Some(rem))
    }
}

impl DoubleEndedIterator for MountAliasIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.end_idx {
            return None;
        }
        self.end_idx = self.end_idx.strict_sub(1);
        Some(self.vfs.mount_aliases.get(self.end_idx, &self.vfs.arena))
    }
}

impl ExactSizeIterator for MountAliasIter<'_> {}

/// Non-recursive pre-order DFS iterator over all winning [`FileId`]s inside a directory subtree.
///
/// Stores at most one remaining [`DirSubdirsIter`] range cursor (`8` bytes) per tree depth level
/// (`<= 64` frames), bounding stack usage regardless of directory branching factor.
#[derive(Clone, Debug)]
pub struct SubtreeFileIter<'a> {
    dirs: &'a DirTable,
    current_files: DirFilesIter,
    stack: ArrayVec<DirSubdirsIter, { PATH_DEPTH_HARD_MAX as usize }>,
}

impl Iterator for SubtreeFileIter<'_> {
    type Item = FileId;

    fn next(&mut self) -> Option<Self::Item> {
        for _ in 0..=self.dirs.len() {
            if let Some(fid) = self.current_files.next() {
                return Some(fid);
            }
            let top_iter = self.stack.last_mut()?;
            assert!(top_iter.len() > 0);
            let next_dir = top_iter.next()?;
            if top_iter.len() == 0 {
                let popped = self.stack.pop();
                assert!(popped.is_some());
            }
            let view = self.dirs.view(next_dir);
            let child_subdirs = view.subdirs();
            if child_subdirs.len() > 0 {
                let pushed = self.stack.try_push(child_subdirs);
                assert!(pushed.is_ok());
            }
            self.current_files = view.files();
        }
        assert!(self.stack.is_empty());
        None
    }
}

/// Zero-allocation iterator over all winning [`FileId`]s matching a case-insensitive file extension.
#[derive(Clone, Copy, Debug)]
pub struct ExtFileIter<'a> {
    files: &'a FileTable,
    exts: &'a ExtTable,
    target_canonical: Option<ExtSlotId>,
    next_idx: u32,
}

impl Iterator for ExtFileIter<'_> {
    type Item = FileId;

    fn next(&mut self) -> Option<Self::Item> {
        let target = self.target_canonical?;
        let start = u32_to_usize(self.next_idx);
        for (idx, key) in (self.next_idx..self.files.len()).zip(self.files.keys[start..].iter()) {
            self.next_idx = idx.strict_add(1);
            if self.exts.canonical_for(key.ext_slot) == Some(target) {
                return Some(FileId::new(idx));
            }
        }
        None
    }
}

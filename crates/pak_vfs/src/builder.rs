//! Single-threaded two-phase VFS builder: streaming winner-takes-all ingest + BFS table freeze.

use core::num::NonZeroU32;
use std::path::{Path, PathBuf};

use pak::{EntryId, PakError, PakPayloadReader, StreamingEntry};

use crate::arena::{ArenaSpan, ExtBuilder, ExtTable, StringArena, StringArenaBuilder};
use crate::error::{PathErrorKind, VfsError, VfsLimitKind};
use crate::lookup::DirLookupTable;
use crate::mount_alias::{MountAliasTable, discover_mount_aliases};
use crate::path::{
    FNV1A_64_OFFSET_BASIS, add_segment_byte_budget, cmp_folded_str, cmp_folded_then_exact,
    eq_folded_opt_str, eq_folded_str, extend_path_hash, hash_folded_bytes, parse_mount_prefix,
    split_stem_ext, validate_named_segment,
};
use crate::table::{DirNode, DirTable, FileKey, FileTable, IdRange, PakEntryRef};
use crate::types::{
    DIRS_COUNT_HARD_MAX, DirId, ExtSlotId, FILES_COUNT_HARD_MAX, MountTier, PakId, VfsLimits,
    VfsOpenOptions,
};
use crate::{bounded_len_u32, hash_hi32, hash_lo32, u32_to_usize};

pub(crate) struct MountInput {
    pub(crate) path: PathBuf,
    pub(crate) tier: MountTier,
}

#[derive(Debug)]
pub(crate) struct MountedPakRecord {
    pub(crate) reader: PakPayloadReader,
    pub(crate) path: Box<Path>,
    pub(crate) mount_point: Box<str>,
    pub(crate) tier: MountTier,
    pub(crate) winning_file_count: u32,
}

pub(crate) struct BuiltVfs {
    pub(crate) paks: Box<[MountedPakRecord]>,
    pub(crate) dirs: DirTable,
    pub(crate) files: FileTable,
    pub(crate) exts: ExtTable,
    pub(crate) arena: StringArena,
    pub(crate) dir_lookup: DirLookupTable,
    pub(crate) mount_aliases: MountAliasTable,
    pub(crate) limits: VfsLimits,
}

enum StreamIngestError {
    Pak(PakError),
    Vfs(VfsError),
}

impl From<PakError> for StreamIngestError {
    fn from(err: PakError) -> Self {
        Self::Pak(err)
    }
}

impl From<VfsError> for StreamIngestError {
    fn from(err: VfsError) -> Self {
        Self::Vfs(err)
    }
}

/// Mounts an ordered list of `.pak` archives (highest override priority first) into immutable access-temperature-grouped tables.
pub(crate) fn build_pak_vfs(
    mounts: Vec<MountInput>,
    options: &VfsOpenOptions,
) -> Result<BuiltVfs, VfsError> {
    let limits = options.limits();
    let pak_count = match u16::try_from(mounts.len()) {
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

    let mut staging = StagingState::new(limits);
    let mut paks = Vec::with_capacity(mounts.len());
    let mut cache = ArchiveIngestCache::new();

    for (pak_u16, mount) in (0..pak_count).zip(mounts) {
        let pak_id = PakId::new(pak_u16);
        let record = ingest_single_pak(pak_id, mount, options, &mut staging, &mut cache)?;
        paks.push(record);
    }

    freeze_staging(paks, staging, options)
}

fn ingest_single_pak(
    pak_id: PakId,
    mount: MountInput,
    options: &VfsOpenOptions,
    staging: &mut StagingState,
    cache: &mut ArchiveIngestCache,
) -> Result<MountedPakRecord, VfsError> {
    cache.reset_for_archive();
    let mut winning_file_count: u32 = 0;
    let pak_path_ref = mount.path.as_path();

    let open_res = PakPayloadReader::open_streaming(
        pak_path_ref,
        options.pak_options(),
        |mount_point, item| match item {
            StreamingEntry::Named { id, dir, file } => {
                let won = staging.ingest_named_entry(cache, pak_id, mount_point, dir, file, id)?;
                if won {
                    winning_file_count = winning_file_count.strict_add(1);
                }
                Ok::<(), StreamIngestError>(())
            }
            StreamingEntry::Pruned { id } => {
                Err(StreamIngestError::Vfs(VfsError::PrunedEntryUnsupported {
                    pak_path: pak_path_ref.to_path_buf(),
                    entry: id,
                }))
            }
        },
    );

    let (reader, mount_point) = match open_res {
        Ok(pair) => pair,
        Err(StreamIngestError::Pak(source)) => {
            return Err(VfsError::Pak {
                pak_path: mount.path,
                source,
            });
        }
        Err(StreamIngestError::Vfs(err)) => return Err(err),
    };

    Ok(MountedPakRecord {
        reader,
        path: mount.path.into_boxed_path(),
        mount_point,
        tier: mount.tier,
        winning_file_count,
    })
}

struct ArchiveIngestCache {
    mount_dir_id: Option<u32>,
    last_raw_dir: String,
    last_resolved_dir: Option<u32>,
}

impl ArchiveIngestCache {
    fn new() -> Self {
        Self {
            mount_dir_id: None,
            last_raw_dir: String::with_capacity(128),
            last_resolved_dir: None,
        }
    }

    fn reset_for_archive(&mut self) {
        self.mount_dir_id = None;
        self.last_raw_dir.clear();
        self.last_resolved_dir = None;
    }
}

#[derive(Clone, Copy)]
struct StagingDirNode {
    parent: Option<u32>,
    name_span: ArenaSpan,
    depth: u16,
    has_subdirs: bool,
    has_files: bool,
    norm_bytes: u32,
    dir_hash: Option<u64>,
}
const _: () = assert!(core::mem::size_of::<StagingDirNode>() == 40);

impl StagingDirNode {
    const ROOT: Self = Self {
        parent: None,
        name_span: ArenaSpan::EMPTY,
        depth: 0,
        has_subdirs: false,
        has_files: false,
        norm_bytes: 0,
        dir_hash: None,
    };
}

#[derive(Clone, Copy)]
struct StagingFile {
    staging_dir: u32,
    stem_span: ArenaSpan,
    ext_slot: Option<ExtSlotId>,
    pak_id: PakId,
    entry_id: EntryId,
    path_hash: u64,
}

struct StagingState {
    text: String,
    dirs: Vec<StagingDirNode>,
    dir_map: StagingDirMap,
    files: Vec<StagingFile>,
    winner_map: WinnerFileMap,
    exts: ExtBuilder,
    limits: VfsLimits,
}

impl StagingState {
    fn new(limits: VfsLimits) -> Self {
        let mut dirs = Vec::with_capacity(1024);
        dirs.push(StagingDirNode::ROOT);
        Self {
            text: String::with_capacity(64 * 1024),
            dirs,
            dir_map: StagingDirMap::with_initial_slots(2048),
            files: Vec::with_capacity(4096),
            winner_map: WinnerFileMap::with_initial_slots(8192),
            exts: ExtBuilder::new(),
            limits,
        }
    }

    #[inline]
    const fn dir_count(&self) -> u32 {
        bounded_len_u32(self.dirs.len())
    }

    #[inline]
    const fn file_count(&self) -> u32 {
        bounded_len_u32(self.files.len())
    }

    fn push_staging_str(&mut self, s: &str) -> Result<ArenaSpan, VfsError> {
        let Ok(seg_len) = u16::try_from(s.len()) else {
            let actual = u64::try_from(s.len()).map_err(|_| VfsError::LimitExceeded {
                kind: VfsLimitKind::PathBytes,
                actual: u64::MAX,
                max: u64::from(u16::MAX),
            })?;
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::PathBytes,
                actual,
                max: u64::from(u16::MAX),
            });
        };
        let cur_u32 = bounded_len_u32(self.text.len());
        if !s.is_empty() && self.text.ends_with(s) {
            let reuse_off = cur_u32.strict_sub(u32::from(seg_len));
            return Ok(ArenaSpan::new(reuse_off, seg_len));
        }
        let staging_max = self.limits.arena_bytes_max().saturating_mul(4);
        let next_u64 = u64::from(cur_u32).strict_add(u64::from(seg_len));
        if next_u64 > u64::from(staging_max) {
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::ArenaBytes,
                actual: next_u64,
                max: u64::from(self.limits.arena_bytes_max()),
            });
        }
        self.text.push_str(s);
        Ok(ArenaSpan::new(cur_u32, seg_len))
    }

    fn ingest_named_entry(
        &mut self,
        cache: &mut ArchiveIngestCache,
        pak_id: PakId,
        mount_point: &str,
        raw_dir: &str,
        raw_file: &str,
        entry_id: EntryId,
    ) -> Result<bool, VfsError> {
        if raw_file.as_bytes().contains(&0) {
            return Err(VfsError::InvalidVirtualPath {
                reason: PathErrorKind::ContainsNul,
            });
        }
        if raw_file.is_empty() || raw_file.ends_with(['/', '\\']) {
            return Err(VfsError::InvalidVirtualPath {
                reason: PathErrorKind::MissingFileName,
            });
        }
        let mount_dir = match cache.mount_dir_id {
            Some(id) => id,
            None => {
                let id = self.intern_mount_prefix(mount_point)?;
                cache.mount_dir_id = Some(id);
                id
            }
        };
        let base_dir = if raw_dir.is_empty() {
            mount_dir
        } else if let Some(cached) = cache.last_resolved_dir
            && cache.last_raw_dir == raw_dir
        {
            cached
        } else {
            let resolved = self.intern_dir_chain(mount_dir, raw_dir)?;
            cache.last_raw_dir.clear();
            cache.last_raw_dir.push_str(raw_dir);
            cache.last_resolved_dir = Some(resolved);
            resolved
        };
        let (parent_dir, leaf) = self.intern_file_subdirs_and_leaf(base_dir, raw_file)?;
        self.try_record_winning_file(parent_dir, leaf, pak_id, entry_id)
    }

    fn intern_mount_prefix(&mut self, mount_point: &str) -> Result<u32, VfsError> {
        let stack = parse_mount_prefix(mount_point, self.limits)?;
        let mut cur: u32 = 0;
        for &seg in &stack {
            cur = self.intern_child_dir(cur, seg)?;
        }
        Ok(cur)
    }

    fn intern_dir_chain(&mut self, mut cur_dir: u32, raw_dir: &str) -> Result<u32, VfsError> {
        if raw_dir.as_bytes().contains(&0) {
            return Err(VfsError::InvalidVirtualPath {
                reason: PathErrorKind::ContainsNul,
            });
        }
        for seg in raw_dir.split(['/', '\\']) {
            if seg.is_empty() {
                continue;
            }
            cur_dir = self.intern_child_dir(cur_dir, seg)?;
        }
        Ok(cur_dir)
    }

    fn intern_file_subdirs_and_leaf<'a>(
        &mut self,
        mut cur_dir: u32,
        raw_file: &'a str,
    ) -> Result<(u32, &'a str), VfsError> {
        let mut pending: Option<&'a str> = None;
        for seg in raw_file.split(['/', '\\']) {
            if seg.is_empty() {
                continue;
            }
            if let Some(dir_seg) = pending.replace(seg) {
                cur_dir = self.intern_child_dir(cur_dir, dir_seg)?;
            }
        }
        let leaf = pending.ok_or(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::MissingFileName,
        })?;
        validate_named_segment(leaf)?;
        Ok((cur_dir, leaf))
    }

    fn intern_child_dir(&mut self, parent_dir: u32, seg: &str) -> Result<u32, VfsError> {
        validate_named_segment(seg)?;
        let edge_hash = dir_edge_hash(parent_dir, seg);
        if let Some(existing) = self
            .dir_map
            .find_with_hash(parent_dir, seg, edge_hash, &self.dirs, &self.text)
        {
            return Ok(existing);
        }
        let parent_node = self.dirs[u32_to_usize(parent_dir)];
        let next_depth = parent_node.depth.strict_add(1);
        if next_depth > self.limits.path_depth_max() {
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::PathDepth,
                actual: u64::from(next_depth),
                max: u64::from(self.limits.path_depth_max()),
            });
        }
        let norm_bytes =
            add_segment_byte_budget(parent_node.norm_bytes, seg, parent_dir != 0, self.limits)?;
        let cur_dir_count = self.dir_count();
        if cur_dir_count >= self.limits.dirs_count_max() {
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::DirsCount,
                actual: u64::from(self.limits.dirs_count_max()).strict_add(1),
                max: u64::from(self.limits.dirs_count_max()),
            });
        }
        let dir_hash = extend_path_hash(parent_node.dir_hash, seg);
        if parent_node.has_files {
            let (seg_stem, seg_ext) = split_stem_ext(seg);
            let probe_key = StagingFileKey {
                path_hash: dir_hash,
                staging_dir: parent_dir,
                stem: seg_stem,
                ext: seg_ext,
            };
            if self
                .winner_map
                .contains(probe_key, &self.files, &self.text, &self.exts)
            {
                return Err(VfsError::PathNamespaceConflict);
            }
        }
        let name_span = self.push_staging_str(seg)?;
        let new_idx = cur_dir_count;
        self.dirs[u32_to_usize(parent_dir)].has_subdirs = true;
        self.dirs.push(StagingDirNode {
            parent: Some(parent_dir),
            name_span,
            depth: next_depth,
            has_subdirs: false,
            has_files: false,
            norm_bytes,
            dir_hash: Some(dir_hash),
        });
        self.dir_map
            .insert_with_hash(edge_hash, new_idx, &self.dirs, &self.text, self.limits)?;
        Ok(new_idx)
    }

    fn try_record_winning_file(
        &mut self,
        parent_dir: u32,
        leaf: &str,
        pak_id: PakId,
        entry_id: EntryId,
    ) -> Result<bool, VfsError> {
        let parent_node = self.dirs[u32_to_usize(parent_dir)];
        let _total_bytes =
            add_segment_byte_budget(parent_node.norm_bytes, leaf, parent_dir != 0, self.limits)?;
        let (stem, ext) = split_stem_ext(leaf);
        let path_hash = extend_path_hash(parent_node.dir_hash, leaf);
        let probe_key = StagingFileKey {
            path_hash,
            staging_dir: parent_dir,
            stem,
            ext,
        };
        if self
            .winner_map
            .contains(probe_key, &self.files, &self.text, &self.exts)
        {
            return Ok(false);
        }
        if parent_node.has_subdirs
            && self
                .dir_map
                .find_with_hash(
                    parent_dir,
                    leaf,
                    dir_edge_hash(parent_dir, leaf),
                    &self.dirs,
                    &self.text,
                )
                .is_some()
        {
            return Err(VfsError::PathNamespaceConflict);
        }
        let cur_file_count = self.file_count();
        if cur_file_count >= self.limits.files_count_max() {
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::FilesCount,
                actual: u64::from(self.limits.files_count_max()).strict_add(1),
                max: u64::from(self.limits.files_count_max()),
            });
        }
        let ext_slot = self.exts.intern(ext, self.limits)?;
        let stem_span = self.push_staging_str(stem)?;
        let file_idx = cur_file_count;
        self.dirs[u32_to_usize(parent_dir)].has_files = true;
        self.files.push(StagingFile {
            staging_dir: parent_dir,
            stem_span,
            ext_slot,
            pak_id,
            entry_id,
            path_hash,
        });
        self.winner_map
            .insert(path_hash, file_idx, &self.files, self.limits)?;
        Ok(true)
    }
}

#[inline]
fn staging_slice(text: &str, span: ArenaSpan) -> &str {
    let start = u32_to_usize(span.offset);
    let end = start.strict_add(usize::from(span.len));
    &text[start..end]
}

#[inline]
fn dir_edge_hash(parent_dir: u32, seg: &str) -> u64 {
    let seg_hash = hash_folded_bytes(FNV1A_64_OFFSET_BASIS, seg);
    let mixed_parent = u64::from(parent_dir).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    seg_hash ^ mixed_parent.rotate_left(21)
}

#[repr(C)]
#[derive(Clone, Copy)]
struct OccupiedStagingDirSlot {
    non_root_dir: NonZeroU32,
    key_hi: u32,
}
const _: () = assert!(core::mem::size_of::<Option<OccupiedStagingDirSlot>>() == 8);

impl OccupiedStagingDirSlot {
    #[inline]
    #[must_use]
    const fn new(dir_idx: u32, key_hi: u32) -> Self {
        assert!(dir_idx > 0);
        Self {
            non_root_dir: NonZeroU32::MIN.saturating_add(dir_idx.strict_sub(1)),
            key_hi,
        }
    }

    #[inline]
    #[must_use]
    const fn dir_idx(self) -> u32 {
        self.non_root_dir.get()
    }
}

struct StagingDirMap {
    buckets: Box<[Option<OccupiedStagingDirSlot>]>,
    occupied: u32,
}

impl StagingDirMap {
    fn with_initial_slots(slots: u32) -> Self {
        let len = slots.max(4).next_power_of_two();
        Self {
            buckets: vec![None; u32_to_usize(len)].into_boxed_slice(),
            occupied: 0,
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

    fn find_with_hash(
        &self,
        parent_dir: u32,
        seg: &str,
        key_hash: u64,
        nodes: &[StagingDirNode],
        text: &str,
    ) -> Option<u32> {
        let target_hi = hash_hi32(key_hash);
        let table_len = self.table_len();
        let mask = self.mask();
        let start = hash_lo32(key_hash) & mask;
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let slot = u32_to_usize(start.wrapping_add(step) & mask);
            let occ = self.buckets[slot]?;
            if occ.key_hi == target_hi {
                let dir_idx = occ.dir_idx();
                let node = nodes[u32_to_usize(dir_idx)];
                if node.parent == Some(parent_dir)
                    && eq_folded_str(staging_slice(text, node.name_span), seg)
                {
                    return Some(dir_idx);
                }
            }
            probed = probed.strict_add(1);
        }
        assert!(probed < table_len);
        None
    }

    fn insert_with_hash(
        &mut self,
        key_hash: u64,
        dir_idx: u32,
        nodes: &[StagingDirNode],
        text: &str,
        limits: VfsLimits,
    ) -> Result<(), VfsError> {
        let table_len = self.table_len();
        if self.occupied.saturating_mul(2) >= table_len {
            self.grow(nodes, text, limits)?;
        }
        self.insert_raw(key_hash, dir_idx);
        Ok(())
    }

    fn grow(
        &mut self,
        nodes: &[StagingDirNode],
        text: &str,
        limits: VfsLimits,
    ) -> Result<(), VfsError> {
        let cur_len = self.table_len();
        let max_slots = (DIRS_COUNT_HARD_MAX.saturating_mul(2)).next_power_of_two();
        if cur_len >= max_slots {
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::DirsCount,
                actual: u64::from(self.occupied).strict_add(1),
                max: u64::from(limits.dirs_count_max()),
            });
        }
        let next_len = cur_len.strict_mul(2);
        let old = core::mem::replace(
            &mut self.buckets,
            vec![None; u32_to_usize(next_len)].into_boxed_slice(),
        );
        self.occupied = 0;
        for &b in &old {
            if let Some(occ) = b {
                let dir_idx = occ.dir_idx();
                let node = nodes[u32_to_usize(dir_idx)];
                if let Some(parent) = node.parent {
                    let name = staging_slice(text, node.name_span);
                    self.insert_raw(dir_edge_hash(parent, name), dir_idx);
                }
            }
        }
        Ok(())
    }

    fn insert_raw(&mut self, key_hash: u64, dir_idx: u32) {
        let table_len = self.table_len();
        let mask = self.mask();
        assert!(self.occupied < table_len);
        let start = hash_lo32(key_hash) & mask;
        let key_hi = hash_hi32(key_hash);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let slot = u32_to_usize(start.wrapping_add(step) & mask);
            if self.buckets[slot].is_none() {
                self.buckets[slot] = Some(OccupiedStagingDirSlot::new(dir_idx, key_hi));
                self.occupied = self.occupied.strict_add(1);
                return;
            }
            probed = probed.strict_add(1);
        }
        assert!(probed < table_len);
    }
}

#[derive(Clone, Copy)]
struct StagingFileKey<'a> {
    path_hash: u64,
    staging_dir: u32,
    stem: &'a str,
    ext: Option<&'a str>,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct OccupiedWinnerFileSlot {
    file_plus_one: NonZeroU32,
    hash_hi: u32,
}
const _: () = assert!(core::mem::size_of::<Option<OccupiedWinnerFileSlot>>() == 8);

impl OccupiedWinnerFileSlot {
    #[inline]
    #[must_use]
    const fn new(file_idx: u32, hash_hi: u32) -> Self {
        assert!(file_idx < u32::MAX);
        Self {
            file_plus_one: NonZeroU32::MIN.saturating_add(file_idx),
            hash_hi,
        }
    }

    #[inline]
    #[must_use]
    const fn file_idx(self) -> u32 {
        self.file_plus_one.get().strict_sub(1)
    }
}

struct WinnerFileMap {
    buckets: Box<[Option<OccupiedWinnerFileSlot>]>,
    occupied: u32,
}

impl WinnerFileMap {
    fn with_initial_slots(slots: u32) -> Self {
        let len = slots.max(4).next_power_of_two();
        Self {
            buckets: vec![None; u32_to_usize(len)].into_boxed_slice(),
            occupied: 0,
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

    fn contains(
        &self,
        key: StagingFileKey<'_>,
        files: &[StagingFile],
        text: &str,
        exts: &ExtBuilder,
    ) -> bool {
        let target_hi = hash_hi32(key.path_hash);
        let table_len = self.table_len();
        let mask = self.mask();
        let start = hash_lo32(key.path_hash) & mask;
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let slot = u32_to_usize(start.wrapping_add(step) & mask);
            match self.buckets[slot] {
                None => return false,
                Some(occ) => {
                    if occ.hash_hi == target_hi {
                        let f = files[u32_to_usize(occ.file_idx())];
                        if f.staging_dir == key.staging_dir
                            && eq_folded_opt_str(exts.get_str(f.ext_slot), key.ext)
                            && eq_folded_str(staging_slice(text, f.stem_span), key.stem)
                        {
                            return true;
                        }
                    }
                    probed = probed.strict_add(1);
                }
            }
        }
        assert!(probed < table_len);
        false
    }

    fn insert(
        &mut self,
        path_hash: u64,
        file_idx: u32,
        files: &[StagingFile],
        limits: VfsLimits,
    ) -> Result<(), VfsError> {
        let table_len = self.table_len();
        if self.occupied.saturating_mul(2) >= table_len {
            self.grow(files, limits)?;
        }
        self.insert_raw(path_hash, file_idx);
        Ok(())
    }

    fn grow(&mut self, files: &[StagingFile], limits: VfsLimits) -> Result<(), VfsError> {
        let cur_len = self.table_len();
        let max_slots = (FILES_COUNT_HARD_MAX.saturating_mul(2)).next_power_of_two();
        if cur_len >= max_slots {
            return Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::FilesCount,
                actual: u64::from(self.occupied).strict_add(1),
                max: u64::from(limits.files_count_max()),
            });
        }
        let next_len = cur_len.strict_mul(2);
        let old = core::mem::replace(
            &mut self.buckets,
            vec![None; u32_to_usize(next_len)].into_boxed_slice(),
        );
        self.occupied = 0;
        for &b in &old {
            if let Some(occ) = b {
                let file_idx = occ.file_idx();
                let f = files[u32_to_usize(file_idx)];
                self.insert_raw(f.path_hash, file_idx);
            }
        }
        Ok(())
    }

    fn insert_raw(&mut self, path_hash: u64, file_idx: u32) {
        let table_len = self.table_len();
        let mask = self.mask();
        assert!(self.occupied < table_len);
        let start = hash_lo32(path_hash) & mask;
        let hash_hi = hash_hi32(path_hash);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let slot = u32_to_usize(start.wrapping_add(step) & mask);
            if self.buckets[slot].is_none() {
                self.buckets[slot] = Some(OccupiedWinnerFileSlot::new(file_idx, hash_hi));
                self.occupied = self.occupied.strict_add(1);
                return;
            }
            probed = probed.strict_add(1);
        }
        assert!(probed < table_len);
    }
}

struct BfsDirLayout {
    old_to_new: Vec<DirId>,
    nodes: Vec<DirNode>,
    dir_lookup: DirLookupTable,
}

fn freeze_staging(
    paks: Vec<MountedPakRecord>,
    staging: StagingState,
    options: &VfsOpenOptions,
) -> Result<BuiltVfs, VfsError> {
    let limits = staging.limits;
    drop(staging.dir_map);
    drop(staging.winner_map);

    let est_arena_bytes = (bounded_len_u32(staging.text.len()) / 2)
        .saturating_add(1024)
        .min(limits.arena_bytes_max());
    let mut arena_builder =
        StringArenaBuilder::with_capacity(est_arena_bytes, limits.arena_bytes_max());

    let mut dir_layout = freeze_dirs_bfs(&staging.dirs, &staging.text, &mut arena_builder)?;
    let files = freeze_files_grouped(
        &staging.files,
        &staging.text,
        &staging.exts,
        &dir_layout.old_to_new,
        &mut dir_layout.nodes,
        &mut arena_builder,
    )?;
    let exts = staging.exts.finish(&mut arena_builder)?;
    drop(staging.text);
    drop(staging.dirs);
    drop(staging.files);

    let dirs = DirTable {
        nodes: dir_layout.nodes.into_boxed_slice(),
    };

    let mount_aliases = discover_mount_aliases(
        &dirs,
        &files,
        &mut arena_builder,
        options.game_project_name(),
        limits,
    )?;
    let arena = arena_builder.finish();

    Ok(BuiltVfs {
        paks: paks.into_boxed_slice(),
        dirs,
        files,
        exts,
        arena,
        dir_lookup: dir_layout.dir_lookup,
        mount_aliases,
        limits,
    })
}

fn freeze_dirs_bfs(
    stg_dirs: &[StagingDirNode],
    stg_text: &str,
    arena: &mut StringArenaBuilder,
) -> Result<BfsDirLayout, VfsError> {
    let dir_count = bounded_len_u32(stg_dirs.len());
    let dir_count_usize = stg_dirs.len();
    let (child_offsets, children_flat) = build_sorted_dir_children_csr(stg_dirs, stg_text);

    let mut bfs_queue = Vec::<u32>::with_capacity(dir_count_usize);
    bfs_queue.push(0);
    let mut old_to_new = vec![DirId::ROOT; dir_count_usize];
    let mut nodes = Vec::<DirNode>::with_capacity(dir_count_usize);
    let mut dir_lookup = DirLookupTable::with_dir_capacity(dir_count.saturating_sub(1));

    for new_idx in 0..dir_count {
        let new_dir = DirId::new(new_idx);
        let stg_id = bfs_queue[u32_to_usize(new_idx)];
        old_to_new[u32_to_usize(stg_id)] = new_dir;

        let c_start_u32 = child_offsets[u32_to_usize(stg_id)];
        let c_end_u32 = child_offsets[u32_to_usize(stg_id).strict_add(1)];
        let sub_start = bounded_len_u32(bfs_queue.len());
        let sub_count = c_end_u32.strict_sub(c_start_u32);
        bfs_queue
            .extend_from_slice(&children_flat[u32_to_usize(c_start_u32)..u32_to_usize(c_end_u32)]);
        let sub_range = IdRange {
            start: sub_start,
            count: sub_count,
        };

        let node = stg_dirs[u32_to_usize(stg_id)];
        let parent = node.parent.map(|p| {
            let mapped = old_to_new[u32_to_usize(p)];
            assert!(mapped.index() < new_idx);
            mapped
        });
        let name_span = if new_dir.is_root() {
            ArenaSpan::EMPTY
        } else {
            let seg = staging_slice(stg_text, node.name_span);
            let span = arena.push_str(seg)?;
            if let Some(hash) = node.dir_hash {
                dir_lookup.insert(hash, new_dir);
            }
            span
        };
        nodes.push(DirNode::new(
            parent,
            name_span,
            node.depth,
            sub_range,
            IdRange { start: 0, count: 0 },
        ));
    }
    assert_eq!(bfs_queue.len(), dir_count_usize);
    assert_eq!(nodes.len(), dir_count_usize);

    Ok(BfsDirLayout {
        old_to_new,
        nodes,
        dir_lookup,
    })
}

fn build_sorted_dir_children_csr(
    stg_dirs: &[StagingDirNode],
    stg_text: &str,
) -> (Vec<u32>, Vec<u32>) {
    let dir_count = bounded_len_u32(stg_dirs.len());
    let n = stg_dirs.len();
    let mut counts = vec![0_u32; n];
    for node in stg_dirs.iter().skip(1) {
        if let Some(p) = node.parent {
            counts[u32_to_usize(p)] = counts[u32_to_usize(p)].strict_add(1);
        }
    }
    let mut offsets = Vec::with_capacity(n.strict_add(1));
    offsets.push(0);
    let mut running: u32 = 0;
    for &c in &counts {
        running = running.strict_add(c);
        offsets.push(running);
    }
    let mut cursors = offsets[..n].to_vec();
    let mut flat = vec![0_u32; u32_to_usize(running)];
    for (idx_u32, node) in (1..dir_count).zip(stg_dirs.iter().skip(1)) {
        if let Some(p) = node.parent {
            let pos = cursors[u32_to_usize(p)];
            flat[u32_to_usize(pos)] = idx_u32;
            cursors[u32_to_usize(p)] = pos.strict_add(1);
        }
    }
    for pair in offsets.windows(2) {
        let s = u32_to_usize(pair[0]);
        let e = u32_to_usize(pair[1]);
        flat[s..e].sort_by(|&a, &b| {
            let na = stg_dirs[u32_to_usize(a)];
            let nb = stg_dirs[u32_to_usize(b)];
            cmp_folded_then_exact(
                staging_slice(stg_text, na.name_span),
                staging_slice(stg_text, nb.name_span),
            )
        });
    }
    (offsets, flat)
}

fn freeze_files_grouped(
    stg_files: &[StagingFile],
    stg_text: &str,
    exts: &ExtBuilder,
    old_to_new_dir: &[DirId],
    dir_nodes: &mut [DirNode],
    arena: &mut StringArenaBuilder,
) -> Result<FileTable, VfsError> {
    let file_count_usize = stg_files.len();
    let dir_count = bounded_len_u32(old_to_new_dir.len());
    let (file_ranges, mut grouped_indices) = group_files_by_final_dir(stg_files, old_to_new_dir);

    let mut out_keys = Vec::with_capacity(file_count_usize);
    let mut out_payloads = Vec::with_capacity(file_count_usize);

    assert_eq!(old_to_new_dir.len(), file_ranges.len());
    assert_eq!(dir_nodes.len(), file_ranges.len());
    for ((dir_idx, &range), dir_node) in (0..dir_count)
        .zip(file_ranges.iter())
        .zip(dir_nodes.iter_mut())
    {
        dir_node.file_range = range;
        let final_dir = DirId::new(dir_idx);
        let start = u32_to_usize(range.start);
        let end = start.strict_add(u32_to_usize(range.count));
        let slice = &mut grouped_indices[start..end];
        sort_dir_file_indices(slice, stg_files, stg_text, exts);

        let mut prev_stem: Option<(&str, ArenaSpan)> = None;
        for &stg_f_idx in &*slice {
            let f = stg_files[u32_to_usize(stg_f_idx)];
            let stem_str = staging_slice(stg_text, f.stem_span);
            let stem_span = match prev_stem {
                Some((prev_str, span)) if prev_str == stem_str => span,
                Some(_) | None => {
                    let span = arena.push_str(stem_str)?;
                    prev_stem = Some((stem_str, span));
                    span
                }
            };
            out_keys.push(FileKey::new(stem_span, f.ext_slot));
            out_payloads.push(PakEntryRef {
                dir: final_dir,
                entry_id: f.entry_id,
                pak_id: f.pak_id,
                known_ext: exts.known_for(f.ext_slot),
            });
        }
    }
    assert_eq!(out_keys.len(), file_count_usize);
    assert_eq!(out_payloads.len(), file_count_usize);

    Ok(FileTable {
        keys: out_keys.into_boxed_slice(),
        payloads: out_payloads.into_boxed_slice(),
    })
}

fn group_files_by_final_dir(
    stg_files: &[StagingFile],
    old_to_new_dir: &[DirId],
) -> (Box<[IdRange]>, Vec<u32>) {
    let d_usize = old_to_new_dir.len();
    let file_count = bounded_len_u32(stg_files.len());
    let mut counts = vec![0_u32; d_usize];
    for f in stg_files {
        let new_d = old_to_new_dir[u32_to_usize(f.staging_dir)];
        counts[u32_to_usize(new_d.index())] = counts[u32_to_usize(new_d.index())].strict_add(1);
    }
    let mut ranges = Vec::with_capacity(d_usize);
    let mut cursors = Vec::with_capacity(d_usize);
    let mut running: u32 = 0;
    for &c in &counts {
        ranges.push(IdRange {
            start: running,
            count: c,
        });
        cursors.push(running);
        running = running.strict_add(c);
    }
    let mut grouped = vec![0_u32; stg_files.len()];
    for (idx_u32, f) in (0..file_count).zip(stg_files.iter()) {
        let d_idx = u32_to_usize(old_to_new_dir[u32_to_usize(f.staging_dir)].index());
        let pos = cursors[d_idx];
        grouped[u32_to_usize(pos)] = idx_u32;
        cursors[d_idx] = pos.strict_add(1);
    }
    (ranges.into_boxed_slice(), grouped)
}

fn sort_dir_file_indices(
    indices: &mut [u32],
    stg_files: &[StagingFile],
    stg_text: &str,
    exts: &ExtBuilder,
) {
    indices.sort_by(|&a, &b| {
        let fa = stg_files[u32_to_usize(a)];
        let fb = stg_files[u32_to_usize(b)];
        let stem_a = staging_slice(stg_text, fa.stem_span);
        let stem_b = staging_slice(stg_text, fb.stem_span);
        cmp_folded_str(stem_a, stem_b)
            .then_with(|| {
                exts.known_for(fa.ext_slot)
                    .cmp(&exts.known_for(fb.ext_slot))
            })
            .then_with(
                || match (exts.get_str(fa.ext_slot), exts.get_str(fb.ext_slot)) {
                    (None, None) => core::cmp::Ordering::Equal,
                    (None, Some(_)) => core::cmp::Ordering::Less,
                    (Some(_), None) => core::cmp::Ordering::Greater,
                    (Some(ea), Some(eb)) => cmp_folded_then_exact(ea, eb),
                },
            )
            .then_with(|| stem_a.cmp(stem_b))
    });
}

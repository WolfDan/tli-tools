//! Access-temperature-grouped directory tree (`DirTable`) and winning file catalog (`FileTable`).

use arrayvec::ArrayVec;
use pak::EntryId;

use crate::arena::{ArenaSpan, ExtTable, KnownExt, StringArena};
use crate::path::{cmp_folded_str, eq_folded_opt_str, eq_folded_str};
use crate::types::{
    DirId, ExtSlotId, FileId, PATH_DEPTH_HARD_MAX, PackageKind, PakId, ResolvedPackage,
};
use crate::{bounded_len_u32, u32_to_usize};

/// Contiguous half-open `[start .. start + count)` index span inside a flat table.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IdRange {
    pub(crate) start: u32,
    pub(crate) count: u32,
}
const _: () = assert!(core::mem::size_of::<IdRange>() == 8);

/// Packed 8-byte hot lookup key storing stem span and interned extension slot.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct FileKey {
    stem_offset: u32,
    stem_len: u16,
    pub(crate) ext_slot: Option<ExtSlotId>,
}
const _: () = assert!(core::mem::size_of::<FileKey>() == 8);

impl FileKey {
    #[inline]
    #[must_use]
    pub(crate) const fn new(stem: ArenaSpan, ext_slot: Option<ExtSlotId>) -> Self {
        Self {
            stem_offset: stem.offset,
            stem_len: stem.len,
            ext_slot,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) const fn stem_span(self) -> ArenaSpan {
        ArenaSpan::new(self.stem_offset, self.stem_len)
    }
}

/// Packed 12-byte cold payload record storing inline parent [`DirId`], winning `(PakId, EntryId)`, and cached extension kind.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct PakEntryRef {
    pub(crate) dir: DirId,
    pub(crate) entry_id: EntryId,
    pub(crate) pak_id: PakId,
    pub(crate) known_ext: KnownExt,
}
const _: () = assert!(core::mem::size_of::<PakEntryRef>() == 12);

/// Packed 28-byte point-lookup directory record storing inline parent link, name span, depth, and child ranges.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct DirNode {
    parent: DirId,
    name_offset: u32,
    name_len: u16,
    pub(crate) depth: u16,
    pub(crate) subdir_range: IdRange,
    pub(crate) file_range: IdRange,
}
const _: () = assert!(core::mem::size_of::<DirNode>() == 28);

impl DirNode {
    #[inline]
    #[must_use]
    pub(crate) const fn new(
        parent: Option<DirId>,
        name_span: ArenaSpan,
        depth: u16,
        subdir_range: IdRange,
        file_range: IdRange,
    ) -> Self {
        let raw_parent = match parent {
            Some(p) => p,
            None => DirId::ROOT,
        };
        Self {
            parent: raw_parent,
            name_offset: name_span.offset,
            name_len: name_span.len,
            depth,
            subdir_range,
            file_range,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) const fn parent(self, dir: DirId) -> Option<DirId> {
        if dir.is_root() {
            None
        } else {
            Some(self.parent)
        }
    }

    #[inline]
    #[must_use]
    pub(crate) const fn name_span(self) -> ArenaSpan {
        ArenaSpan::new(self.name_offset, self.name_len)
    }
}

/// Immutable BFS-ordered directory hierarchy backed by contiguous [`DirNode`] rows.
///
/// Because non-root `DirId`s are assigned in BFS parent-grouped order, every directory's sorted
/// child subdirectories occupy a contiguous `subdir_range: IdRange` slice in `nodes`.
#[derive(Debug)]
pub(crate) struct DirTable {
    pub(crate) nodes: Box<[DirNode]>,
}

impl DirTable {
    #[inline]
    #[must_use]
    pub(crate) const fn len(&self) -> u32 {
        bounded_len_u32(self.nodes.len())
    }

    #[inline]
    #[must_use]
    pub(crate) fn node(&self, dir: DirId) -> DirNode {
        self.nodes[u32_to_usize(dir.index())]
    }

    #[inline]
    #[must_use]
    pub(crate) fn parent(&self, dir: DirId) -> Option<DirId> {
        self.node(dir).parent(dir)
    }

    #[inline]
    #[must_use]
    pub(crate) fn name_span(&self, dir: DirId) -> ArenaSpan {
        self.node(dir).name_span()
    }

    #[inline]
    #[must_use]
    pub(crate) fn name<'a>(&self, dir: DirId, arena: &'a StringArena) -> &'a str {
        arena.get(self.name_span(dir))
    }

    #[inline]
    #[must_use]
    pub(crate) fn subdir_ids(&self, dir: DirId) -> DirSubdirsIter {
        let range = self.node(dir).subdir_range;
        DirSubdirsIter {
            next_idx: range.start,
            end_idx: range.start.strict_add(range.count),
        }
    }

    /// Top-down child directory lookup inside `parent`'s contiguous `subdir_range` (`O(log K)` binary search on `nodes`).
    #[must_use]
    pub(crate) fn find_child(
        &self,
        parent: DirId,
        child_name: &str,
        arena_text: &str,
    ) -> Option<DirId> {
        let range = self.node(parent).subdir_range;
        let start = u32_to_usize(range.start);
        let end = start.strict_add(u32_to_usize(range.count));
        let child_nodes = &self.nodes[start..end];
        let search_res = child_nodes.binary_search_by(|node| {
            let span = node.name_span();
            let s_start = u32_to_usize(span.offset);
            let s_end = s_start.strict_add(usize::from(span.len));
            cmp_folded_str(&arena_text[s_start..s_end], child_name)
        });
        match search_res {
            Ok(rel) => Some(DirId::new(range.start.strict_add(bounded_len_u32(rel)))),
            Err(_) => None,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn file_range(&self, dir: DirId) -> IdRange {
        self.node(dir).file_range
    }

    #[inline]
    #[must_use]
    pub(crate) fn view(&self, dir: DirId) -> DirView {
        let node = self.node(dir);
        DirView {
            dir,
            parent: node.parent(dir),
            subdir_range: node.subdir_range,
            file_range: node.file_range,
        }
    }

    /// Collects the ancestor chain of `dir` (excluding `DirId::ROOT`) in top-down root-to-leaf order
    /// using an inline `ArrayVec<DirId, 64>` work buffer with zero recursion.
    #[must_use]
    pub(crate) fn ancestor_chain(
        &self,
        dir: DirId,
    ) -> ArrayVec<DirId, { PATH_DEPTH_HARD_MAX as usize }> {
        let mut rev_chain = ArrayVec::<DirId, { PATH_DEPTH_HARD_MAX as usize }>::new();
        let mut cur = dir;
        for _ in 0..PATH_DEPTH_HARD_MAX {
            if cur.is_root() {
                break;
            }
            let pushed = rev_chain.try_push(cur);
            assert!(pushed.is_ok());
            match self.parent(cur) {
                Some(p) => cur = p,
                None => break,
            }
        }
        assert!(cur.is_root());
        rev_chain.reverse();
        rev_chain
    }

    /// Appends the canonical case-preserved root-relative directory path of `dir` into `out`.
    pub(crate) fn write_dir_path(&self, dir: DirId, arena: &StringArena, out: &mut Vec<u8>) {
        let chain = self.ancestor_chain(dir);
        if let Some((&first, rest)) = chain.split_first() {
            out.extend_from_slice(self.name(first, arena).as_bytes());
            for &node in rest {
                out.push(b'/');
                out.extend_from_slice(self.name(node, arena).as_bytes());
            }
        }
    }
}

/// Zero-allocation directory listing view returned by [`crate::PakVfs::read_dir`].
#[derive(Clone, Copy, Debug)]
pub struct DirView {
    dir: DirId,
    parent: Option<DirId>,
    subdir_range: IdRange,
    file_range: IdRange,
}

impl DirView {
    #[inline]
    #[must_use]
    pub const fn dir_id(&self) -> DirId {
        self.dir
    }

    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<DirId> {
        self.parent
    }

    /// Number of direct child directories under this directory.
    #[inline]
    #[must_use]
    pub const fn subdir_count(&self) -> u32 {
        self.subdir_range.count
    }

    /// Returns the `offset`-th direct child [`DirId`] (`0..subdir_count`) in this directory.
    #[inline]
    #[must_use]
    pub const fn subdir_at(&self, offset: u32) -> Option<DirId> {
        if offset < self.subdir_range.count {
            Some(DirId::new(self.subdir_range.start.strict_add(offset)))
        } else {
            None
        }
    }

    /// Exact-size iterator over all direct child [`DirId`]s in this directory in sorted order.
    #[inline]
    #[must_use]
    pub const fn subdirs(&self) -> DirSubdirsIter {
        DirSubdirsIter::from_range(self.subdir_range)
    }

    /// Number of winning files residing directly in this directory.
    #[inline]
    #[must_use]
    pub const fn file_count(&self) -> u32 {
        self.file_range.count
    }

    /// Returns the `offset`-th direct [`FileId`] (`0..file_count`) in this directory.
    #[inline]
    #[must_use]
    pub const fn file_at(&self, offset: u32) -> Option<FileId> {
        if offset < self.file_range.count {
            Some(FileId::new(self.file_range.start.strict_add(offset)))
        } else {
            None
        }
    }

    /// Exact-size iterator over all direct [`FileId`]s in this directory in sorted `(stem, ext)` order.
    #[inline]
    #[must_use]
    pub const fn files(&self) -> DirFilesIter {
        DirFilesIter {
            next_idx: self.file_range.start,
            end_idx: self.file_range.start.strict_add(self.file_range.count),
        }
    }
}

/// Exact-size iterator over contiguous child [`DirId`]s inside a single directory.
#[derive(Clone, Copy, Debug)]
pub struct DirSubdirsIter {
    pub(crate) next_idx: u32,
    pub(crate) end_idx: u32,
}

impl DirSubdirsIter {
    #[inline]
    #[must_use]
    pub(crate) const fn from_range(range: IdRange) -> Self {
        Self {
            next_idx: range.start,
            end_idx: range.start.strict_add(range.count),
        }
    }
}

impl Iterator for DirSubdirsIter {
    type Item = DirId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.end_idx {
            return None;
        }
        let id = DirId::new(self.next_idx);
        self.next_idx = self.next_idx.strict_add(1);
        Some(id)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = u32_to_usize(self.end_idx.strict_sub(self.next_idx));
        (rem, Some(rem))
    }
}

impl DoubleEndedIterator for DirSubdirsIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.end_idx {
            return None;
        }
        self.end_idx = self.end_idx.strict_sub(1);
        Some(DirId::new(self.end_idx))
    }
}

impl ExactSizeIterator for DirSubdirsIter {}

/// Exact-size iterator over contiguous [`FileId`]s inside a single directory (or the entire VFS).
#[derive(Clone, Copy, Debug)]
pub struct DirFilesIter {
    pub(crate) next_idx: u32,
    pub(crate) end_idx: u32,
}

impl Iterator for DirFilesIter {
    type Item = FileId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.end_idx {
            return None;
        }
        let id = FileId::new(self.next_idx);
        self.next_idx = self.next_idx.strict_add(1);
        Some(id)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = u32_to_usize(self.end_idx.strict_sub(self.next_idx));
        (rem, Some(rem))
    }
}

impl DoubleEndedIterator for DirFilesIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.end_idx {
            return None;
        }
        self.end_idx = self.end_idx.strict_sub(1);
        Some(FileId::new(self.end_idx))
    }
}

impl ExactSizeIterator for DirFilesIter {}

/// Immutable catalog of winning virtual files grouped by access temperature (`8`-byte hot [`FileKey`] + `12`-byte cold [`PakEntryRef`]).
#[derive(Debug)]
pub(crate) struct FileTable {
    pub(crate) keys: Box<[FileKey]>,
    pub(crate) payloads: Box<[PakEntryRef]>,
}

impl FileTable {
    #[inline]
    #[must_use]
    pub(crate) const fn len(&self) -> u32 {
        bounded_len_u32(self.keys.len())
    }

    #[inline]
    #[must_use]
    pub(crate) fn key(&self, file: FileId) -> FileKey {
        self.keys[u32_to_usize(file.index())]
    }

    #[inline]
    #[must_use]
    pub(crate) fn dir(&self, file: FileId) -> DirId {
        self.entry_ref(file).dir
    }

    #[inline]
    #[must_use]
    pub(crate) fn entry_ref(&self, file: FileId) -> PakEntryRef {
        self.payloads[u32_to_usize(file.index())]
    }

    #[inline]
    #[must_use]
    pub(crate) fn stem<'a>(&self, file: FileId, arena: &'a StringArena) -> &'a str {
        arena.get(self.key(file).stem_span())
    }

    #[inline]
    #[must_use]
    pub(crate) fn extension<'a>(
        &self,
        file: FileId,
        exts: &ExtTable,
        arena: &'a StringArena,
    ) -> Option<&'a str> {
        exts.get(self.key(file).ext_slot, arena)
    }

    /// Top-down file lookup inside a known `dir`'s contiguous `file_range` (`O(log M)` binary search on `keys`).
    #[must_use]
    pub(crate) fn find_in_dir(
        &self,
        dir_range: IdRange,
        query_stem: &str,
        query_ext: Option<&str>,
        exts: &ExtTable,
        arena: &StringArena,
    ) -> Option<FileId> {
        let start = u32_to_usize(dir_range.start);
        let end = start.strict_add(u32_to_usize(dir_range.count));
        let dir_keys = &self.keys[start..end];
        let rel = dir_keys
            .partition_point(|k| cmp_folded_str(arena.get(k.stem_span()), query_stem).is_lt());
        let tail = &dir_keys[rel..];
        let (&first_key, _) = tail.split_first()?;
        if !eq_folded_str(arena.get(first_key.stem_span()), query_stem) {
            return None;
        }
        let first_idx = dir_range.start.strict_add(bounded_len_u32(rel));
        for (fid_u32, &key) in
            (first_idx..dir_range.start.strict_add(dir_range.count)).zip(tail.iter())
        {
            if fid_u32 > first_idx && !same_key_stem(key, first_key, query_stem, arena) {
                break;
            }
            if eq_folded_opt_str(exts.get(key.ext_slot, arena), query_ext) {
                return Some(FileId::new(fid_u32));
            }
        }
        None
    }

    /// Appends the full case-preserved root-relative path of `file` into `out` with zero recursion.
    pub(crate) fn write_file_path(
        &self,
        file: FileId,
        dirs: &DirTable,
        exts: &ExtTable,
        arena: &StringArena,
        out: &mut Vec<u8>,
    ) {
        let dir = self.dir(file);
        let key = self.key(file);
        if !dir.is_root() {
            dirs.write_dir_path(dir, arena, out);
            out.push(b'/');
        }
        out.extend_from_slice(arena.get(key.stem_span()).as_bytes());
        if let Some(ext_str) = exts.get(key.ext_slot, arena) {
            out.push(b'.');
            out.extend_from_slice(ext_str.as_bytes());
        }
    }

    /// Scans the adjacent co-located stem group around `anchor` in `O(1)` L1 reads to assemble
    /// its [`ResolvedPackage`] (`.uasset`/`.umap` header + `.uexp`, `.ubulk`, `.uptnl` companions).
    #[must_use]
    pub(crate) fn package_around(
        &self,
        anchor: FileId,
        dirs: &DirTable,
        arena: &StringArena,
    ) -> Option<ResolvedPackage> {
        assert_eq!(self.keys.len(), self.payloads.len());
        let anchor_ref = self.entry_ref(anchor);
        if !anchor_ref.known_ext.is_package_member() {
            return None;
        }
        let anchor_idx = anchor.index();
        let anchor_key = self.key(anchor);
        let dir_range = dirs.file_range(anchor_ref.dir);
        let dir_start = dir_range.start;
        let dir_end = dir_range.start.strict_add(dir_range.count);
        let anchor_stem = arena.get(anchor_key.stem_span());

        let mut group_start = anchor_idx;
        let prev_keys = &self.keys[u32_to_usize(dir_start)..u32_to_usize(anchor_idx)];
        for (prev_idx, &prev_key) in (dir_start..anchor_idx).zip(prev_keys.iter()).rev() {
            if !same_key_stem(prev_key, anchor_key, anchor_stem, arena) {
                break;
            }
            group_start = prev_idx;
        }

        let mut asset_hdr: Option<FileId> = None;
        let mut map_hdr: Option<FileId> = None;
        let mut uexp: Option<FileId> = None;
        let mut ubulk: Option<FileId> = None;
        let mut uptnl: Option<FileId> = None;

        let g_start = u32_to_usize(group_start);
        let g_end = u32_to_usize(dir_end);
        for ((cur, &key), &payload) in (group_start..dir_end)
            .zip(self.keys[g_start..g_end].iter())
            .zip(self.payloads[g_start..g_end].iter())
        {
            if cur > anchor_idx && !same_key_stem(key, anchor_key, anchor_stem, arena) {
                break;
            }
            let fid = FileId::new(cur);
            match payload.known_ext {
                KnownExt::Uasset => asset_hdr = Some(fid),
                KnownExt::Umap => map_hdr = Some(fid),
                KnownExt::Uexp => uexp = Some(fid),
                KnownExt::Ubulk => ubulk = Some(fid),
                KnownExt::Uptnl => uptnl = Some(fid),
                KnownExt::None | KnownExt::Uplugin | KnownExt::Other => {}
            }
        }

        let (kind, header) = if anchor_ref.known_ext == KnownExt::Umap {
            (PackageKind::Map, map_hdr?)
        } else if let Some(h) = asset_hdr {
            (PackageKind::Asset, h)
        } else {
            (PackageKind::Map, map_hdr?)
        };
        Some(ResolvedPackage {
            kind,
            header,
            uexp,
            ubulk,
            uptnl,
        })
    }
}

#[inline]
fn same_key_stem(
    other_key: FileKey,
    ref_key: FileKey,
    ref_stem: &str,
    arena: &StringArena,
) -> bool {
    if other_key.stem_span() == ref_key.stem_span() {
        return true;
    }
    eq_folded_str(arena.get(other_key.stem_span()), ref_stem)
}

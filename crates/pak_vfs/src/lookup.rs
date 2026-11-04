//! Cache-friendly [`HashTable<u32>`] `O(1)` directory lookup table (`DirLookupTable`).

use hashbrown::HashTable;

use crate::arena::StringArena;
use crate::path::eq_folded_str;
use crate::table::DirTable;
use crate::types::{DIRS_COUNT_HARD_MAX, DirId};
use crate::{bounded_len_u32, u32_to_usize};

pub(crate) fn matches_dir_ancestry(
    mut cur_dir: DirId,
    base_dir: DirId,
    rel_segments: &[&str],
    dirs: &DirTable,
    arena: &StringArena,
) -> bool {
    let base_depth = u32::from(dirs.node(base_dir).depth);
    let expected_depth = base_depth + bounded_len_u32(rel_segments.len());
    if u32::from(dirs.node(cur_dir).depth) != expected_depth {
        return false;
    }
    for &seg in rel_segments.iter().rev() {
        if cur_dir.is_root() {
            return false;
        }
        let node = dirs.node(cur_dir);
        if !eq_folded_str(arena.get(node.name_span()), seg) {
            return false;
        }
        match node.parent(cur_dir) {
            Some(parent) => cur_dir = parent,
            None => return false,
        }
    }
    cur_dir == base_dir
}

/// Immutable [`HashTable<u32>`] lookup table mapping non-root virtual directory paths to [`DirId`].
#[derive(Debug)]
pub(crate) struct DirLookupTable {
    table: HashTable<u32>,
}

impl DirLookupTable {
    #[must_use]
    pub(crate) fn with_dir_capacity(non_root_dir_count: u32) -> Self {
        assert!(non_root_dir_count <= DIRS_COUNT_HARD_MAX);
        Self {
            table: HashTable::with_capacity(u32_to_usize(non_root_dir_count)),
        }
    }

    pub(crate) fn insert(&mut self, dir_hash: u64, dir: DirId, dir_hashes: &[u64]) {
        assert!(!dir.is_root());
        self.table
            .insert_unique(dir_hash, dir.index(), |&idx| dir_hashes[u32_to_usize(idx)]);
    }

    #[must_use]
    pub(crate) fn find(
        &self,
        base_dir: DirId,
        rel_segments: &[&str],
        dir_hash: Option<u64>,
        dirs: &DirTable,
        arena: &StringArena,
    ) -> Option<DirId> {
        if rel_segments.is_empty() {
            return Some(base_dir);
        }
        let hash = dir_hash?;
        let idx = *self.table.find(hash, |&dir_idx| {
            matches_dir_ancestry(DirId::new(dir_idx), base_dir, rel_segments, dirs, arena)
        })?;
        Some(DirId::new(idx))
    }
}

//! Cache-friendly open-addressed `O(1)` directory lookup table (`DirLookupTable`).

use core::num::NonZeroU32;

use crate::arena::StringArena;
use crate::path::eq_folded_str;
use crate::table::DirTable;
use crate::types::{DIRS_COUNT_HARD_MAX, DirId};
use crate::{bounded_len_u32, hash_hi32, hash_lo32, u32_to_usize};

pub(crate) fn matches_dir_ancestry(
    mut cur_dir: DirId,
    base_dir: DirId,
    rel_segments: &[&str],
    dirs: &DirTable,
    arena: &StringArena,
) -> bool {
    let base_depth = usize::from(dirs.node(base_dir).depth);
    let expected_depth = base_depth.strict_add(rel_segments.len());
    if usize::from(dirs.node(cur_dir).depth) != expected_depth {
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct OccupiedDirSlot {
    non_root_dir: NonZeroU32,
    hash_hi: u32,
}
const _: () = assert!(core::mem::size_of::<Option<OccupiedDirSlot>>() == 8);

impl OccupiedDirSlot {
    #[inline]
    #[must_use]
    const fn new(dir: DirId, hash_hi: u32) -> Self {
        let idx = dir.index();
        assert!(idx > 0);
        Self {
            non_root_dir: NonZeroU32::MIN.saturating_add(idx.strict_sub(1)),
            hash_hi,
        }
    }

    #[inline]
    #[must_use]
    const fn dir(self) -> DirId {
        DirId::new(self.non_root_dir.get())
    }
}

/// Immutable power-of-two linear-probing lookup table (`<= 50%` load factor, `8` bytes/slot)
/// mapping non-root virtual directory paths to [`DirId`].
#[derive(Debug)]
pub(crate) struct DirLookupTable {
    buckets: Box<[Option<OccupiedDirSlot>]>,
    occupied_count: u32,
}

impl DirLookupTable {
    #[must_use]
    pub(crate) fn with_dir_capacity(non_root_dir_count: u32) -> Self {
        assert!(non_root_dir_count <= DIRS_COUNT_HARD_MAX);
        let min_slots = (non_root_dir_count.saturating_mul(2)).max(4);
        let table_len = min_slots.next_power_of_two();
        Self {
            buckets: vec![None; u32_to_usize(table_len)].into_boxed_slice(),
            occupied_count: 0,
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

    pub(crate) fn insert(&mut self, dir_hash: u64, dir: DirId) {
        assert!(!dir.is_root());
        let table_len = self.table_len();
        let mask = self.mask();
        assert!(self.occupied_count < table_len);
        let start = hash_lo32(dir_hash) & mask;
        let hi = hash_hi32(dir_hash);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let idx = u32_to_usize(start.wrapping_add(step) & mask);
            match self.buckets[idx] {
                None => {
                    self.buckets[idx] = Some(OccupiedDirSlot::new(dir, hi));
                    self.occupied_count = self.occupied_count.strict_add(1);
                    return;
                }
                Some(_) => {
                    probed = probed.strict_add(1);
                }
            }
        }
        assert!(probed < table_len);
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
        let table_len = self.table_len();
        let mask = self.mask();
        let start = hash_lo32(hash) & mask;
        let target_hi = hash_hi32(hash);
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let idx = u32_to_usize(start.wrapping_add(step) & mask);
            let occ = self.buckets[idx]?;
            let dir = occ.dir();
            if occ.hash_hi == target_hi
                && matches_dir_ancestry(dir, base_dir, rel_segments, dirs, arena)
            {
                return Some(dir);
            }
            probed = probed.strict_add(1);
        }
        assert!(probed < table_len);
        None
    }
}

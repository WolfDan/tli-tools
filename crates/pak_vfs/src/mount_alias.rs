//! Zero-I/O Unreal Engine logical mount alias discovery (`/Game`, `/Engine`, `/<PluginName>`).

use core::num::NonZeroU16;

use crate::arena::{ArenaSpan, KnownExt, StringArena, StringArenaBuilder};
use crate::error::{VfsError, VfsLimitKind};
use crate::path::{
    DirPrefixState, FNV1A_64_OFFSET_BASIS, add_segment_byte_budget, cmp_folded_then_exact,
    eq_folded_str, extend_path_hash, hash_folded_bytes,
};
use crate::table::{DirTable, FileTable};
use crate::types::{DirId, MountAlias, VfsLimits};
use crate::{bounded_len_u16, bounded_len_u32, hash_lo32, u32_to_usize};

#[derive(Clone, Copy, Debug)]
struct StoredAlias {
    name_span: ArenaSpan,
    prefix: DirPrefixState,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
struct OccupiedAliasSlot(NonZeroU16);
const _: () = assert!(core::mem::size_of::<Option<OccupiedAliasSlot>>() == 2);

impl OccupiedAliasSlot {
    #[inline]
    #[must_use]
    const fn new(alias_idx: u16) -> Self {
        assert!(alias_idx < u16::MAX);
        Self(NonZeroU16::MIN.saturating_add(alias_idx))
    }

    #[inline]
    #[must_use]
    const fn alias_idx(self) -> u16 {
        self.0.get().strict_sub(1)
    }
}

/// Immutable table of discovered Unreal Engine logical mount roots (`/Game`, `/Engine`, `/<Plugin>`).
#[derive(Debug)]
pub(crate) struct MountAliasTable {
    entries: Box<[StoredAlias]>,
    buckets: Box<[Option<OccupiedAliasSlot>]>,
}

impl MountAliasTable {
    #[inline]
    #[must_use]
    pub(crate) const fn len(&self) -> u16 {
        bounded_len_u16(self.entries.len())
    }

    #[inline]
    const fn table_len(&self) -> u32 {
        bounded_len_u32(self.buckets.len())
    }

    #[inline]
    const fn mask(&self) -> u32 {
        self.table_len().strict_sub(1)
    }

    #[inline]
    #[must_use]
    pub(crate) fn get<'a>(&self, index: u16, arena: &'a StringArena) -> MountAlias<'a> {
        let item = self.entries[usize::from(index)];
        MountAlias {
            alias: arena.get(item.name_span),
            target_dir: item.prefix.base_dir,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn resolve(&self, alias: &str, arena: &StringArena) -> Option<DirId> {
        self.resolve_prefix(alias, arena).map(|p| p.base_dir)
    }

    #[must_use]
    pub(crate) fn resolve_prefix(
        &self,
        alias: &str,
        arena: &StringArena,
    ) -> Option<DirPrefixState> {
        if self.entries.is_empty() {
            return None;
        }
        let hash = hash_folded_bytes(FNV1A_64_OFFSET_BASIS, alias);
        let table_len = self.table_len();
        let mask = self.mask();
        let start = hash_lo32(hash) & mask;
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let slot = u32_to_usize(start.wrapping_add(step) & mask);
            let occ = self.buckets[slot]?;
            let entry = self.entries[usize::from(occ.alias_idx())];
            let stored = arena.get(entry.name_span);
            if eq_folded_str(stored, alias) {
                return Some(entry.prefix);
            }
            probed = probed.strict_add(1);
        }
        assert!(probed < table_len);
        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AliasOriginRank {
    CoreRoot = 0,
    ProjectPlugin = 1,
    EnginePlugin = 2,
}

#[derive(Clone, Copy)]
enum AliasNameSource {
    Existing(ArenaSpan),
    Static(&'static str),
}

impl AliasNameSource {
    #[inline]
    fn as_str(self, arena: &StringArenaBuilder) -> &str {
        match self {
            Self::Existing(span) => arena.get(span),
            Self::Static(s) => s,
        }
    }

    #[inline]
    fn into_span(self, arena: &mut StringArenaBuilder) -> Result<ArenaSpan, VfsError> {
        match self {
            Self::Existing(span) => Ok(span),
            Self::Static(s) => arena.push_str(s),
        }
    }
}

#[derive(Clone, Copy)]
struct CandidateAlias {
    name: AliasNameSource,
    target_dir: DirId,
    origin: AliasOriginRank,
}

/// Discovers `/Engine`, `/Game`, and `/<PluginName>` mount roots directly from `DirTable` and
/// `FileTable` without reading any payload bytes, allocating heap strings, or duplicating plugin
/// names already present in `arena_builder`.
pub(crate) fn discover_mount_aliases(
    dirs: &DirTable,
    files: &FileTable,
    arena_builder: &mut StringArenaBuilder,
    explicit_project: Option<&str>,
    limits: VfsLimits,
) -> Result<MountAliasTable, VfsError> {
    let mut candidates = Vec::<CandidateAlias>::with_capacity(64);
    collect_core_aliases(dirs, arena_builder, explicit_project, &mut candidates);
    collect_uplugin_aliases(dirs, files, arena_builder, &mut candidates);
    collect_plugins_dir_aliases(dirs, arena_builder, &mut candidates);

    candidates.sort_by(|lhs, rhs| {
        cmp_folded_then_exact(
            lhs.name.as_str(arena_builder),
            rhs.name.as_str(arena_builder),
        )
        .then_with(|| lhs.origin.cmp(&rhs.origin))
        .then_with(|| lhs.target_dir.index().cmp(&rhs.target_dir.index()))
    });

    let mut entries = Vec::<StoredAlias>::with_capacity(candidates.len());
    for cand in candidates {
        let cand_name = cand.name.as_str(arena_builder);
        let already_have = match entries.last() {
            Some(prev) => eq_folded_str(arena_builder.get(prev.name_span), cand_name),
            None => false,
        };
        if !already_have {
            if bounded_len_u16(entries.len()) >= limits.mount_aliases_count_max() {
                return Err(VfsError::LimitExceeded {
                    kind: VfsLimitKind::MountAliasesCount,
                    actual: u64::from(limits.mount_aliases_count_max()).strict_add(1),
                    max: u64::from(limits.mount_aliases_count_max()),
                });
            }
            let name_span = cand.name.into_span(arena_builder)?;
            let prefix = compute_dir_prefix_state(cand.target_dir, dirs, arena_builder, limits)?;
            entries.push(StoredAlias { name_span, prefix });
        }
    }
    Ok(build_alias_hash_table(entries, arena_builder))
}

fn compute_dir_prefix_state(
    target_dir: DirId,
    dirs: &DirTable,
    arena: &StringArenaBuilder,
    limits: VfsLimits,
) -> Result<DirPrefixState, VfsError> {
    let chain = dirs.ancestor_chain(target_dir);
    let mut dir_hash: Option<u64> = None;
    let mut norm_bytes: u32 = 0;
    for (idx, &node) in chain.iter().enumerate() {
        let seg = arena.get(dirs.name_span(node));
        norm_bytes = add_segment_byte_budget(norm_bytes, seg, idx > 0, limits)?;
        dir_hash = Some(extend_path_hash(dir_hash, seg));
    }
    Ok(DirPrefixState {
        base_dir: target_dir,
        dir_hash,
        norm_bytes,
        base_depth: dirs.node(target_dir).depth,
    })
}

fn build_alias_hash_table(
    entries: Vec<StoredAlias>,
    arena: &StringArenaBuilder,
) -> MountAliasTable {
    let count_u16 = bounded_len_u16(entries.len());
    let min_slots = (u32::from(count_u16).saturating_mul(2)).max(4);
    let table_len = min_slots.next_power_of_two();
    let mask = table_len.strict_sub(1);
    let mut buckets = vec![None; u32_to_usize(table_len)].into_boxed_slice();
    for (idx_u16, entry) in (0..count_u16).zip(entries.iter()) {
        let name = arena.get(entry.name_span);
        let hash = hash_folded_bytes(FNV1A_64_OFFSET_BASIS, name);
        let start = hash_lo32(hash) & mask;
        let mut probed: u32 = 0;
        for step in 0..table_len {
            let slot = u32_to_usize(start.wrapping_add(step) & mask);
            if buckets[slot].is_none() {
                buckets[slot] = Some(OccupiedAliasSlot::new(idx_u16));
                break;
            }
            probed = probed.strict_add(1);
        }
        assert!(probed < table_len);
    }
    MountAliasTable {
        entries: entries.into_boxed_slice(),
        buckets,
    }
}

fn collect_core_aliases(
    dirs: &DirTable,
    arena: &StringArenaBuilder,
    explicit_project: Option<&str>,
    out: &mut Vec<CandidateAlias>,
) {
    let arena_text = arena.as_str();
    if let Some(engine_dir) = dirs.find_child(DirId::ROOT, "Engine", arena_text)
        && let Some(engine_content) = dirs.find_child(engine_dir, "Content", arena_text)
    {
        let engine_span = dirs.name_span(engine_dir);
        let name = if arena.get(engine_span) == "Engine" {
            AliasNameSource::Existing(engine_span)
        } else {
            AliasNameSource::Static("Engine")
        };
        out.push(CandidateAlias {
            name,
            target_dir: engine_content,
            origin: AliasOriginRank::CoreRoot,
        });
    }

    if let Some(proj_name) = explicit_project {
        if let Some(proj_dir) = dirs.find_child(DirId::ROOT, proj_name, arena_text)
            && let Some(proj_content) = dirs.find_child(proj_dir, "Content", arena_text)
        {
            out.push(CandidateAlias {
                name: AliasNameSource::Static("Game"),
                target_dir: proj_content,
                origin: AliasOriginRank::CoreRoot,
            });
        }
        return;
    }

    for top_dir in dirs.subdir_ids(DirId::ROOT) {
        let top_name = arena.get(dirs.name_span(top_dir));
        if eq_folded_str(top_name, "Engine") {
            continue;
        }
        if let Some(content_dir) = dirs.find_child(top_dir, "Content", arena_text) {
            out.push(CandidateAlias {
                name: AliasNameSource::Static("Game"),
                target_dir: content_dir,
                origin: AliasOriginRank::CoreRoot,
            });
            break;
        }
    }
}

fn collect_uplugin_aliases(
    dirs: &DirTable,
    files: &FileTable,
    arena: &StringArenaBuilder,
    out: &mut Vec<CandidateAlias>,
) {
    assert_eq!(files.payloads.len(), files.keys.len());
    let arena_text = arena.as_str();
    for (eref, key) in files.payloads.iter().zip(files.keys.iter()) {
        if eref.known_ext != KnownExt::Uplugin {
            continue;
        }
        let stem_span = key.stem_span();
        let plugin_name = arena.get(stem_span);
        if eq_folded_str(plugin_name, "Engine") || eq_folded_str(plugin_name, "Game") {
            continue;
        }
        let Some(content_dir) = dirs.find_child(eref.dir, "Content", arena_text) else {
            continue;
        };
        let origin = classify_plugin_origin(eref.dir, dirs, arena);
        out.push(CandidateAlias {
            name: AliasNameSource::Existing(stem_span),
            target_dir: content_dir,
            origin,
        });
    }
}

fn collect_plugins_dir_aliases(
    dirs: &DirTable,
    arena: &StringArenaBuilder,
    out: &mut Vec<CandidateAlias>,
) {
    let arena_text = arena.as_str();
    for top_dir in dirs.subdir_ids(DirId::ROOT) {
        let origin = if eq_folded_str(arena.get(dirs.name_span(top_dir)), "Engine") {
            AliasOriginRank::EnginePlugin
        } else {
            AliasOriginRank::ProjectPlugin
        };
        let Some(plugins_root) = dirs.find_child(top_dir, "Plugins", arena_text) else {
            continue;
        };
        for plugin_dir in dirs.subdir_ids(plugins_root) {
            let dir_span = dirs.name_span(plugin_dir);
            let plugin_name = arena.get(dir_span);
            if eq_folded_str(plugin_name, "Engine") || eq_folded_str(plugin_name, "Game") {
                continue;
            }
            if let Some(content_dir) = dirs.find_child(plugin_dir, "Content", arena_text) {
                out.push(CandidateAlias {
                    name: AliasNameSource::Existing(dir_span),
                    target_dir: content_dir,
                    origin,
                });
            }
        }
    }
}

fn classify_plugin_origin(
    plugin_dir: DirId,
    dirs: &DirTable,
    arena: &StringArenaBuilder,
) -> AliasOriginRank {
    let chain = dirs.ancestor_chain(plugin_dir);
    if let Some(&top) = chain.first()
        && eq_folded_str(arena.get(dirs.name_span(top)), "Engine")
    {
        return AliasOriginRank::EnginePlugin;
    }
    AliasOriginRank::ProjectPlugin
}

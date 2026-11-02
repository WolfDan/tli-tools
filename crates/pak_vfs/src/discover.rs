//! Single-threaded Torchlight Infinite `.pak` archive discovery and Unreal mount priority ordering.

use core::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{VfsError, VfsIoAction, VfsLimitKind};
use crate::path::cmp_folded_then_exact;
use crate::types::{MountTier, VfsLimits};

#[derive(Debug)]
pub(crate) struct DiscoveredPak {
    pub(crate) path: PathBuf,
    pub(crate) file_name: Box<str>,
    pub(crate) tier: MountTier,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScanDirKind {
    PreMountPak,
    GameContentPaks,
    EnginePluginsPaks,
}

/// Discovers all Unreal `.pak` archives under a Torchlight Infinite installation root and orders
/// them from highest override priority (`0`) to lowest override priority (`pak_count - 1`).
///
/// Scans strictly the three UE `.pak` directories (`PreMountPak` optional, `UE_game/Content/Paks`
/// required, `Engine/Plugins/Paks` optional), avoiding unrelated third-party `.pak` bundles such as
/// Chromium CEF resource packs under `Engine/Binaries/ThirdParty/CEF3`.
pub(crate) fn discover_tli_paks(
    game_root: &Path,
    limits: VfsLimits,
) -> Result<Vec<DiscoveredPak>, VfsError> {
    let premount_dir = game_root.join("UE_game/TorchLight/Saved/PersistentDownloadDir/PreMountPak");
    let content_paks_dir = game_root.join("UE_game/Content/Paks");
    let engine_plugins_dir = game_root.join("Engine/Plugins/Paks");

    let mut out = Vec::with_capacity(64);
    scan_pak_dir(
        &premount_dir,
        ScanDirKind::PreMountPak,
        false,
        limits,
        &mut out,
    )?;
    scan_pak_dir(
        &content_paks_dir,
        ScanDirKind::GameContentPaks,
        true,
        limits,
        &mut out,
    )?;
    scan_pak_dir(
        &engine_plugins_dir,
        ScanDirKind::EnginePluginsPaks,
        false,
        limits,
        &mut out,
    )?;

    if out.is_empty() {
        return Err(VfsError::NoPakArchivesFound {
            root_dir: game_root.to_path_buf(),
        });
    }
    out.sort_by(cmp_discovered_priority);
    Ok(out)
}

fn scan_pak_dir(
    dir: &Path,
    kind: ScanDirKind,
    required: bool,
    limits: VfsLimits,
    out: &mut Vec<DiscoveredPak>,
) -> Result<(), VfsError> {
    match fs::metadata(dir) {
        Ok(meta) => {
            if !meta.is_dir() {
                if required {
                    return Err(VfsError::MissingContentPaksDir {
                        expected_dir: dir.to_path_buf(),
                    });
                }
                return Ok(());
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            if required {
                return Err(VfsError::MissingContentPaksDir {
                    expected_dir: dir.to_path_buf(),
                });
            }
            return Ok(());
        }
        Err(source) => {
            return Err(VfsError::Io {
                action: VfsIoAction::StatDirectory,
                path: dir.to_path_buf(),
                source,
            });
        }
    }

    let read_dir = fs::read_dir(dir).map_err(|source| VfsError::Io {
        action: VfsIoAction::ReadDirectory,
        path: dir.to_path_buf(),
        source,
    })?;
    for entry_res in read_dir {
        let entry = entry_res.map_err(|source| VfsError::Io {
            action: VfsIoAction::ReadDirectoryEntry,
            path: dir.to_path_buf(),
            source,
        })?;
        collect_dir_entry(&entry, kind, limits, out)?;
    }
    Ok(())
}

fn collect_dir_entry(
    entry: &fs::DirEntry,
    kind: ScanDirKind,
    limits: VfsLimits,
    out: &mut Vec<DiscoveredPak>,
) -> Result<(), VfsError> {
    let path = entry.path();
    let meta = fs::metadata(&path).map_err(|source| VfsError::Io {
        action: VfsIoAction::StatPakFile,
        path: path.clone(),
        source,
    })?;
    if !meta.is_file() {
        return Ok(());
    }
    let Some(os_name) = path.file_name() else {
        return Ok(());
    };
    let Some(file_name) = os_name.to_str() else {
        return Ok(());
    };
    if !has_pak_extension(file_name) {
        return Ok(());
    }
    if out.len() >= usize::from(limits.paks_count_max()) {
        return Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::PaksCount,
            actual: u64::from(limits.paks_count_max()).strict_add(1),
            max: u64::from(limits.paks_count_max()),
        });
    }
    let tier = classify_pak_tier(file_name, kind);
    let file_name_box = file_name.to_owned().into_boxed_str();
    out.push(DiscoveredPak {
        path,
        file_name: file_name_box,
        tier,
    });
    Ok(())
}

#[inline]
#[must_use]
fn has_pak_extension(file_name: &str) -> bool {
    if file_name.len() < 5 {
        return false;
    }
    let tail_start = file_name.len().strict_sub(4);
    if !file_name.is_char_boundary(tail_start) {
        return false;
    }
    file_name[tail_start..].eq_ignore_ascii_case(".pak")
}

#[must_use]
fn classify_pak_tier(file_name: &str, kind: ScanDirKind) -> MountTier {
    match kind {
        ScanDirKind::PreMountPak => MountTier::PreMountPatch {
            patch_version: parse_patch_suffix_version(file_name),
        },
        ScanDirKind::GameContentPaks => {
            if is_patch_pak_name(file_name) {
                MountTier::GameContentPatch {
                    patch_version: parse_patch_suffix_version(file_name),
                }
            } else {
                MountTier::GameContentChunk
            }
        }
        ScanDirKind::EnginePluginsPaks => MountTier::EnginePlugin,
    }
}

/// Checks whether `file_name` ends with `_P.pak` (case-insensitively).
#[inline]
#[must_use]
pub(crate) fn is_patch_pak_name(file_name: &str) -> bool {
    if file_name.len() < 7 {
        return false;
    }
    let suffix_start = file_name.len().strict_sub(6);
    if !file_name.is_char_boundary(suffix_start) {
        return false;
    }
    file_name[suffix_start..].eq_ignore_ascii_case("_p.pak")
}

/// Extracts `<N>` from `*_<N>_P.pak` (such as `"UE_game_6269408_P.pak"` -> `6_269_408`), returning
/// `0` for `*_P.pak` filenames that carry no numeric build version before `_P.pak`.
#[must_use]
pub(crate) fn parse_patch_suffix_version(file_name: &str) -> u64 {
    if !is_patch_pak_name(file_name) {
        return 0;
    }
    let stem_before_p = &file_name[..file_name.len().strict_sub(6)];
    let Some(underscore_idx) = stem_before_p.rfind('_') else {
        return 0;
    };
    let digits = &stem_before_p[underscore_idx.strict_add(1)..];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return 0;
    }
    let mut acc: u64 = 0;
    for byte in digits.bytes() {
        let Some(next) = acc
            .checked_mul(10)
            .and_then(|v| v.checked_add(u64::from(byte.strict_sub(b'0'))))
        else {
            return 0;
        };
        acc = next;
    }
    acc
}

const fn tier_rank(tier: MountTier) -> u8 {
    match tier {
        MountTier::PreMountPatch { .. } => 0,
        MountTier::GameContentPatch { .. } => 1,
        MountTier::GameContentChunk => 2,
        MountTier::EnginePlugin => 3,
        MountTier::Explicit { .. } => 4,
    }
}

const fn tier_patch_version(tier: MountTier) -> u64 {
    match tier {
        MountTier::PreMountPatch { patch_version }
        | MountTier::GameContentPatch { patch_version } => patch_version,
        MountTier::GameContentChunk | MountTier::EnginePlugin | MountTier::Explicit { .. } => 0,
    }
}

/// Orders archives from highest priority (`Ordering::Less`) to lowest priority (`Ordering::Greater`),
/// matching Torchlight Infinite's `FPakPlatformFile` mount sequence recorded in `UE_game.log`.
fn cmp_discovered_priority(lhs: &DiscoveredPak, rhs: &DiscoveredPak) -> Ordering {
    tier_rank(lhs.tier)
        .cmp(&tier_rank(rhs.tier))
        .then_with(|| tier_patch_version(rhs.tier).cmp(&tier_patch_version(lhs.tier)))
        .then_with(|| cmp_filename_reverse_ue(&lhs.file_name, &rhs.file_name))
}

fn cmp_filename_reverse_ue(lhs: &str, rhs: &str) -> Ordering {
    cmp_folded_then_exact(rhs, lhs)
}

//! Zero-allocation virtual path normalization, stem/extension splitting, and streaming `FxHasher` hashing.

use core::cmp::Ordering;
use core::hash::Hasher as _;

use arrayvec::ArrayVec;
use rustc_hash::FxHasher;

use crate::error::{PathErrorKind, UePathErrorKind, VfsError, VfsLimitKind};
use crate::types::{DirId, PATH_DEPTH_HARD_MAX_USIZE, VfsLimits};
use crate::{bounded_len_u32, u32_to_usize};

pub(crate) const PATH_HASH_SEED: u64 = 0;

pub(crate) type SegmentStack<'a> = ArrayVec<&'a str, PATH_DEPTH_HARD_MAX_USIZE>;

/// Precomputed directory prefix state for root-relative ([`DirPrefixState::ROOT`]) or Unreal
/// mount-alias-relative (`/<Alias>/...`) path parsing.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DirPrefixState {
    pub(crate) base_dir: DirId,
    pub(crate) dir_hash: Option<u64>,
    pub(crate) norm_bytes: u32,
    pub(crate) base_depth: u16,
}

impl DirPrefixState {
    pub(crate) const ROOT: Self = Self {
        base_dir: DirId::ROOT,
        dir_hash: None,
        norm_bytes: 0,
        base_depth: 0,
    };
}

/// Case-folds a single Unicode scalar value using Unreal Engine's 1-to-1 `FChar::ToLower` rule.
#[inline]
#[must_use]
pub(crate) fn fold_char(ch: char) -> char {
    if ch.is_ascii() {
        return ch.to_ascii_lowercase();
    }
    let mut lower = ch.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(single), None) => single,
        (None, _) | (Some(_), Some(_)) => ch,
    }
}

/// Compares two path segments or filenames under case-insensitive equivalence.
#[inline]
#[must_use]
pub(crate) fn eq_folded_str(a: &str, b: &str) -> bool {
    if a.is_ascii() && b.is_ascii() {
        return a.eq_ignore_ascii_case(b);
    }
    a.chars().map(fold_char).eq(b.chars().map(fold_char))
}

#[inline]
#[must_use]
pub(crate) fn eq_folded_opt_str(a: Option<&str>, b: Option<&str>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(lhs), Some(rhs)) => eq_folded_str(lhs, rhs),
        (None, Some(_)) | (Some(_), None) => false,
    }
}

/// Orders two strings case-insensitively without breaking ties by exact case.
#[must_use]
pub(crate) fn cmp_folded_str(a: &str, b: &str) -> Ordering {
    if a.is_ascii() && b.is_ascii() {
        a.bytes()
            .map(|byte| byte.to_ascii_lowercase())
            .cmp(b.bytes().map(|byte| byte.to_ascii_lowercase()))
    } else {
        a.chars().map(fold_char).cmp(b.chars().map(fold_char))
    }
}

/// Orders two strings case-insensitively, breaking ties deterministically by exact UTF-8 bytes.
#[must_use]
pub(crate) fn cmp_folded_then_exact(a: &str, b: &str) -> Ordering {
    cmp_folded_str(a, b).then_with(|| a.cmp(b))
}

/// Feeds case-folded UTF-8 bytes of `seg` into an active 64-bit `FxHasher` accumulator `state`.
#[inline]
#[must_use]
pub(crate) fn hash_folded_bytes(state: u64, seg: &str) -> u64 {
    let mut hasher = FxHasher::default();
    hasher.write_u64(state);
    if seg.is_ascii() {
        for &byte in seg.as_bytes() {
            hasher.write_u8(byte.to_ascii_lowercase());
        }
    } else {
        let mut encode_buf = [0_u8; 4];
        for ch in seg.chars() {
            let encoded = fold_char(ch).encode_utf8(&mut encode_buf);
            hasher.write(encoded.as_bytes());
        }
    }
    hasher.finish()
}

/// Appends one normalized path segment onto `parent_hash` (inserting `'/'` when `parent_hash.is_some()`).
#[inline]
#[must_use]
pub(crate) fn extend_path_hash(parent_hash: Option<u64>, seg: &str) -> u64 {
    let base = match parent_hash {
        Some(prev) => {
            let mut hasher = FxHasher::default();
            hasher.write_u64(prev);
            hasher.write_u8(b'/');
            hasher.finish()
        }
        None => PATH_HASH_SEED,
    };
    hash_folded_bytes(base, seg)
}

/// Bijectively splits a validated leaf filename into `(stem, Option<ext>)`.
///
/// Splits at the final `'.'` strictly when both `stem` and `ext` are non-empty, guaranteeing
/// exact case-preserved filename reconstruction (`"{stem}"` or `"{stem}.{ext}"`).
#[inline]
#[must_use]
pub(crate) fn split_stem_ext(leaf: &str) -> (&str, Option<&str>) {
    if let Some(dot_idx) = leaf.rfind('.') {
        let stem = &leaf[..dot_idx];
        let ext = &leaf[dot_idx + 1..];
        if !stem.is_empty() && !ext.is_empty() {
            return (stem, Some(ext));
        }
    }
    (leaf, None)
}

/// Parses an archive `mount_point` into normalized root-relative directory segments, stripping
/// leading `"../"` and `"./"` workspace-escape prefixes while rejecting interior `".."` or NULs.
pub(crate) fn parse_mount_prefix(
    mount_point: &str,
    limits: VfsLimits,
) -> Result<SegmentStack<'_>, VfsError> {
    if mount_point.as_bytes().contains(&0) {
        return Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::ContainsNul,
        });
    }
    let mut stack = SegmentStack::new();
    let mut norm_bytes: u32 = 0;
    let mut cur_depth: u16 = 0;
    for seg in mount_point.split(['/', '\\']) {
        if seg.is_empty() {
            continue;
        }
        if stack.is_empty() && (seg == "." || seg == "..") {
            continue;
        }
        validate_named_segment(seg)?;
        norm_bytes = add_segment_byte_budget(norm_bytes, seg, cur_depth > 0, limits)?;
        cur_depth = push_dir_segment(&mut stack, cur_depth, seg, limits)?;
    }
    Ok(stack)
}

pub(crate) struct ParsedFileQuery<'a> {
    pub(crate) base_dir: DirId,
    pub(crate) rel_segments: SegmentStack<'a>,
    pub(crate) dir_hash: Option<u64>,
    pub(crate) leaf: &'a str,
}

/// Parses and validates a physical root-relative file query path without heap allocation.
pub(crate) fn parse_file_query<'a>(
    path: &'a str,
    limits: VfsLimits,
) -> Result<ParsedFileQuery<'a>, VfsError> {
    parse_relative_file_tail(DirPrefixState::ROOT, path, limits)
}

/// Appends a relative file path tail onto a precomputed [`DirPrefixState`].
pub(crate) fn parse_relative_file_tail<'a>(
    prefix: DirPrefixState,
    rel_path: &'a str,
    limits: VfsLimits,
) -> Result<ParsedFileQuery<'a>, VfsError> {
    if rel_path.as_bytes().contains(&0) {
        return Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::ContainsNul,
        });
    }
    if rel_path.ends_with(['/', '\\']) {
        return Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::TrailingSeparatorOnFile,
        });
    }
    let mut rel_segments = SegmentStack::new();
    let mut dir_hash = prefix.dir_hash;
    let mut norm_bytes = prefix.norm_bytes;
    let mut cur_depth = prefix.base_depth;
    let mut pending_leaf: Option<&'a str> = None;
    for seg in rel_path.split(['/', '\\']) {
        if seg.is_empty() {
            continue;
        }
        validate_named_segment(seg)?;
        if let Some(prev_dir_seg) = pending_leaf.replace(seg) {
            norm_bytes = add_segment_byte_budget(norm_bytes, prev_dir_seg, cur_depth > 0, limits)?;
            cur_depth = push_dir_segment(&mut rel_segments, cur_depth, prev_dir_seg, limits)?;
            dir_hash = Some(extend_path_hash(dir_hash, prev_dir_seg));
        }
    }
    let leaf = pending_leaf.ok_or(VfsError::InvalidVirtualPath {
        reason: PathErrorKind::EmptyPath,
    })?;
    let _total_bytes = add_segment_byte_budget(norm_bytes, leaf, cur_depth > 0, limits)?;
    Ok(ParsedFileQuery {
        base_dir: prefix.base_dir,
        rel_segments,
        dir_hash,
        leaf,
    })
}

pub(crate) struct ParsedDirQuery<'a> {
    pub(crate) base_dir: DirId,
    pub(crate) rel_segments: SegmentStack<'a>,
    pub(crate) dir_hash: Option<u64>,
}

/// Parses and validates a physical root-relative directory query (`""` or `"/"` -> `DirId::ROOT`).
pub(crate) fn parse_dir_query<'a>(
    path: &'a str,
    limits: VfsLimits,
) -> Result<ParsedDirQuery<'a>, VfsError> {
    parse_relative_dir_tail(DirPrefixState::ROOT, path, limits)
}

/// Appends a relative directory path tail onto a precomputed [`DirPrefixState`].
pub(crate) fn parse_relative_dir_tail<'a>(
    prefix: DirPrefixState,
    rel_path: &'a str,
    limits: VfsLimits,
) -> Result<ParsedDirQuery<'a>, VfsError> {
    if rel_path.as_bytes().contains(&0) {
        return Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::ContainsNul,
        });
    }
    let mut rel_segments = SegmentStack::new();
    let mut dir_hash = prefix.dir_hash;
    let mut norm_bytes = prefix.norm_bytes;
    let mut cur_depth = prefix.base_depth;
    for seg in rel_path.split(['/', '\\']) {
        if seg.is_empty() {
            continue;
        }
        validate_named_segment(seg)?;
        norm_bytes = add_segment_byte_budget(norm_bytes, seg, cur_depth > 0, limits)?;
        cur_depth = push_dir_segment(&mut rel_segments, cur_depth, seg, limits)?;
        dir_hash = Some(extend_path_hash(dir_hash, seg));
    }
    Ok(ParsedDirQuery {
        base_dir: prefix.base_dir,
        rel_segments,
        dir_hash,
    })
}

/// Splits an Unreal logical path `"/<MountAlias>/<RelPath>"` into `("<MountAlias>", "<RelPath>")`.
pub(crate) fn split_ue_logical_path(
    ue_path: &str,
    limits: VfsLimits,
) -> Result<(&str, &str), VfsError> {
    if ue_path.as_bytes().contains(&0) {
        return Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::ContainsNul,
        });
    }
    let after_lead = ue_path
        .strip_prefix('/')
        .or_else(|| ue_path.strip_prefix('\\'))
        .ok_or(VfsError::InvalidUeMountPath {
            reason: UePathErrorKind::MissingLeadingSlash,
        })?;
    let (alias, rest) = match after_lead.find(['/', '\\']) {
        Some(sep_idx) => (&after_lead[..sep_idx], &after_lead[sep_idx + 1..]),
        None => (after_lead, ""),
    };
    if alias.is_empty() {
        return Err(VfsError::InvalidUeMountPath {
            reason: UePathErrorKind::EmptyMountPoint,
        });
    }
    validate_named_segment(alias)?;
    let _alias_bytes = add_segment_byte_budget(0, alias, false, limits)?;
    Ok((alias, rest))
}

#[inline]
pub(crate) fn validate_named_segment(seg: &str) -> Result<(), VfsError> {
    if seg == "." {
        return Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::DotSegment,
        });
    }
    if seg == ".." {
        return Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::ParentTraversal,
        });
    }
    Ok(())
}

#[inline]
pub(crate) fn add_segment_byte_budget(
    cur_bytes: u32,
    seg: &str,
    needs_slash: bool,
    limits: VfsLimits,
) -> Result<u32, VfsError> {
    let max_bytes = limits.path_bytes_max();
    if seg.len() > u32_to_usize(max_bytes) {
        let actual = u64::try_from(seg.len()).map_err(|_| VfsError::LimitExceeded {
            kind: VfsLimitKind::PathBytes,
            actual: u64::MAX,
            max: u64::from(max_bytes),
        })?;
        return Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::PathBytes,
            actual,
            max: u64::from(max_bytes),
        });
    }
    let seg_u32 = bounded_len_u32(seg.len());
    let next = cur_bytes + u32::from(needs_slash) + seg_u32;
    if next > max_bytes {
        return Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::PathBytes,
            actual: u64::from(next),
            max: u64::from(max_bytes),
        });
    }
    Ok(next)
}

#[inline]
pub(crate) fn push_dir_segment<'a>(
    stack: &mut SegmentStack<'a>,
    cur_depth: u16,
    seg: &'a str,
    limits: VfsLimits,
) -> Result<u16, VfsError> {
    let max_depth = limits.path_depth_max();
    if cur_depth >= max_depth {
        return Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::PathDepth,
            actual: u64::from(cur_depth) + 1,
            max: u64::from(max_depth),
        });
    }
    let push_res = stack.try_push(seg);
    assert!(push_res.is_ok());
    Ok(cur_depth + 1)
}

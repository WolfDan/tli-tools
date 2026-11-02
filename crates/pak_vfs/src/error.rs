//! Exhaustive structured error hierarchy for `pak_vfs` discovery, mounting, lookup, and I/O.

use std::path::PathBuf;

use pak::{EntryId, PakError};

use crate::types::FileId;

/// Identifies the filesystem action being performed when an OS directory/file error occurred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfsIoAction {
    StatDirectory,
    ReadDirectory,
    ReadDirectoryEntry,
    StatPakFile,
}

impl core::fmt::Display for VfsIoAction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::StatDirectory => "stat directory",
            Self::ReadDirectory => "open directory listing",
            Self::ReadDirectoryEntry => "read directory entry",
            Self::StatPakFile => "stat .pak file entry",
        };
        f.write_str(label)
    }
}

/// Identifies which explicit `_MAX` capacity bound in [`crate::VfsLimits`] was exceeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfsLimitKind {
    LimitsConfig,
    PaksCount,
    DirsCount,
    FilesCount,
    ExtsCount,
    ArenaBytes,
    PathBytes,
    PathDepth,
    MountAliasesCount,
}

impl core::fmt::Display for VfsLimitKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::LimitsConfig => "configured VFS limit",
            Self::PaksCount => "mounted .pak archive count",
            Self::DirsCount => "virtual directory count",
            Self::FilesCount => "virtual file count",
            Self::ExtsCount => "distinct file extension count",
            Self::ArenaBytes => "VFS string arena byte length",
            Self::PathBytes => "virtual path UTF-8 byte length",
            Self::PathDepth => "virtual directory nesting depth",
            Self::MountAliasesCount => "Unreal logical mount alias count",
        };
        f.write_str(label)
    }
}

/// Reason a physical virtual path or archive mount/entry path was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathErrorKind {
    EmptyPath,
    MissingFileName,
    TrailingSeparatorOnFile,
    ContainsNul,
    DotSegment,
    ParentTraversal,
}

impl core::fmt::Display for PathErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::EmptyPath => "virtual file path is empty",
            Self::MissingFileName => "archive entry is missing a leaf filename",
            Self::TrailingSeparatorOnFile => "virtual file path ends with a path separator",
            Self::ContainsNul => "virtual path contains an embedded NUL byte",
            Self::DotSegment => "virtual path contains a '.' current-directory segment",
            Self::ParentTraversal => "virtual path contains a '..' parent-traversal segment",
        };
        f.write_str(label)
    }
}

/// Reason an Unreal Engine logical mount path (such as `"/Game/..."`) was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UePathErrorKind {
    MissingLeadingSlash,
    EmptyMountPoint,
}

impl core::fmt::Display for UePathErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::MissingLeadingSlash => "Unreal logical path must start with '/'",
            Self::EmptyMountPoint => "Unreal logical path has an empty mount point name",
        };
        f.write_str(label)
    }
}

/// Concrete error enum covering all fallible `pak_vfs` mount, query, and payload read operations.
#[derive(Debug, thiserror::Error)]
pub enum VfsError {
    #[error("failed to {action} at {}", path.display())]
    Io {
        action: VfsIoAction,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(".pak archive error in {}", pak_path.display())]
    Pak {
        pak_path: PathBuf,
        #[source]
        source: PakError,
    },

    #[error("required Content/Paks directory not found at {}", expected_dir.display())]
    MissingContentPaksDir { expected_dir: PathBuf },

    #[error("no .pak archives found under {}", root_dir.display())]
    NoPakArchivesFound { root_dir: PathBuf },

    #[error(
        "pruned hash-only entry {} in {} is missing a FullDirectoryIndex string path",
        entry.index(),
        pak_path.display()
    )]
    PrunedEntryUnsupported { pak_path: PathBuf, entry: EntryId },

    #[error("invalid virtual path: {reason}")]
    InvalidVirtualPath { reason: PathErrorKind },

    #[error("invalid Unreal logical mount path: {reason}")]
    InvalidUeMountPath { reason: UePathErrorKind },

    #[error("explicit game project name must be a single non-empty path segment without NUL")]
    InvalidProjectName,

    #[error("virtual path namespace conflict: same path is both a file and a directory")]
    PathNamespaceConflict,

    #[error("{kind} ({actual}) exceeds configured maximum ({max})")]
    LimitExceeded {
        kind: VfsLimitKind,
        actual: u64,
        max: u64,
    },

    #[error(
        "destination slice length {actual} does not match file {} uncompressed size {expected}",
        file.index()
    )]
    DestinationSizeMismatch {
        file: FileId,
        expected: u64,
        actual: u64,
    },

    #[error(
        "byte range [{offset}..{offset}+{byte_len}) exceeds file {} uncompressed size {file_bytes}",
        file.index()
    )]
    OutOfBoundsRange {
        file: FileId,
        offset: u64,
        byte_len: u64,
        file_bytes: u64,
    },
}

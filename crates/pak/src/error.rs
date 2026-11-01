//! Exhaustive structured error hierarchy for `.pak` archive parsing and reading.

use arrayvec::ArrayString;

use crate::types::{EncryptionKeyGuid, EntryId, PakDialect, Sha1Hash};

/// Identifies which encrypted byte region failed AES block-alignment checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncryptedSectionKind {
    PrimaryIndex,
    PathHashIndex,
    FullDirectoryIndex,
    LegacyDirectoryIndex,
    EntryPayload,
}

impl core::fmt::Display for EncryptedSectionKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::PrimaryIndex => "primary index",
            Self::PathHashIndex => "path hash index",
            Self::FullDirectoryIndex => "full directory index",
            Self::LegacyDirectoryIndex => "legacy directory index",
            Self::EntryPayload => "entry payload",
        };
        f.write_str(label)
    }
}

/// Identifies which index section failed SHA-1 digest verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexSectionKind {
    PrimaryIndex,
    PathHashIndex,
    FullDirectoryIndex,
}

impl core::fmt::Display for IndexSectionKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::PrimaryIndex => "primary index",
            Self::PathHashIndex => "path hash index",
            Self::FullDirectoryIndex => "full directory index",
        };
        f.write_str(label)
    }
}

/// Identifies which explicit `_MAX` capacity bound was exceeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitKind {
    LimitsConfig,
    IndexBytes,
    EntriesCount,
    BlocksPerEntryCount,
    BlocksTotalCount,
    EntryUncompressedBytes,
    MountPointBytes,
    PathBytes,
    PathUtf16Units,
    PathArenaBytes,
}

impl core::fmt::Display for LimitKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::LimitsConfig => "configured limit",
            Self::IndexBytes => "index byte length",
            Self::EntriesCount => "entry count",
            Self::BlocksPerEntryCount => "blocks per entry",
            Self::BlocksTotalCount => "total compression blocks",
            Self::EntryUncompressedBytes => "entry uncompressed byte length",
            Self::MountPointBytes => "mount point byte length",
            Self::PathBytes => "path byte length",
            Self::PathUtf16Units => "path UTF-16 code unit count",
            Self::PathArenaBytes => "path arena total byte length",
        };
        f.write_str(label)
    }
}

/// Identifies the wire structure being decoded when a layout or string error occurred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireContext {
    Footer,
    PrimaryIndex,
    PathHashIndex,
    FullDirectoryIndex,
    LegacyDirectoryIndex,
    EncodedEntry,
    UnencodedEntry,
    InlineEntryHeader,
    FString,
}

impl core::fmt::Display for WireContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::Footer => "FPakInfo footer",
            Self::PrimaryIndex => "primary index",
            Self::PathHashIndex => "path hash index",
            Self::FullDirectoryIndex => "full directory index",
            Self::LegacyDirectoryIndex => "legacy directory index",
            Self::EncodedEntry => "bit-packed encoded FPakEntry",
            Self::UnencodedEntry => "unencoded FPakEntry",
            Self::InlineEntryHeader => "inline FPakEntry header",
            Self::FString => "serialized FString",
        };
        f.write_str(label)
    }
}

/// Reason an [`crate::types::AesKey`] byte slice or hex string was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyParseErrorKind {
    InvalidSliceLength { byte_len: u64 },
    InvalidHexLength { byte_len: u64 },
    InvalidHexDigit,
}

impl core::fmt::Display for KeyParseErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidSliceLength { byte_len } => {
                write!(
                    f,
                    "expected 32-byte AES-256 key slice, got {byte_len} bytes"
                )
            }
            Self::InvalidHexLength { byte_len } => {
                write!(
                    f,
                    "expected 64 hex ASCII digits for AES-256 key, got {byte_len} bytes"
                )
            }
            Self::InvalidHexDigit => f.write_str("AES-256 hex key contains non-hexadecimal byte"),
        }
    }
}

/// Concrete error enum covering all fallible `.pak` open, lookup, and read operations.
#[derive(Debug, thiserror::Error)]
pub enum PakError {
    #[error("I/O failure at file offset {offset} for {len} bytes")]
    Io {
        offset: u64,
        len: u64,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid AES-256 key: {kind}")]
    InvalidAesKey { kind: KeyParseErrorKind },

    #[error(".pak file is {file_bytes} bytes, smaller than minimum {min_bytes}-byte footer")]
    FileTooSmall { file_bytes: u64, min_bytes: u32 },

    #[error("no valid Unreal Engine or Torchlight Infinite .pak magic found in file footer")]
    InvalidFooterMagic,

    #[error("unsupported .pak format version {raw_version} for dialect {dialect:?}")]
    UnsupportedVersion {
        dialect: PakDialect,
        raw_version: u32,
    },

    #[error("UE4 v9 frozen memory-mapped pak indexes are not supported")]
    FrozenIndexUnsupported,

    #[error("AES-256 key required for encrypted .pak data (key GUID {guid})")]
    EncryptionKeyRequired { guid: EncryptionKeyGuid },

    #[error("encrypted {context} length {byte_len} is not a multiple of 16 bytes")]
    UnalignedEncryptedSize {
        context: EncryptedSectionKind,
        byte_len: u64,
    },

    #[error("SHA-1 mismatch in {section} (expected {expected}, got {actual})")]
    IndexHashMismatch {
        section: IndexSectionKind,
        expected: Sha1Hash,
        actual: Sha1Hash,
    },

    #[error("SHA-1 mismatch in entry {} payload (expected {expected}, got {actual})", entry.index())]
    PayloadHashMismatch {
        entry: EntryId,
        expected: Sha1Hash,
        actual: Sha1Hash,
    },

    #[error("{kind} ({actual}) exceeds configured maximum ({max})")]
    LimitExceeded {
        kind: LimitKind,
        actual: u64,
        max: u64,
    },

    #[error("byte range [{offset}..{offset}+{byte_len}) exceeds container size {container_bytes}")]
    OutOfBoundsRange {
        offset: u64,
        byte_len: u64,
        container_bytes: u64,
    },

    #[error("corrupt binary wire layout while decoding {context}")]
    CorruptWireLayout { context: WireContext },

    #[error("invalid UTF-8 string while decoding {context}")]
    InvalidUtf8String { context: WireContext },

    #[error("invalid UTF-16LE string while decoding {context}")]
    InvalidUtf16String { context: WireContext },

    #[error("unsupported compression method {name:?}")]
    UnsupportedCompressionMethod { name: ArrayString<32> },

    #[error("invalid compression method index {raw_index} for table of length {table_len}")]
    InvalidCompressionMethodIndex { raw_index: u32, table_len: u32 },

    #[error("destination slice length {actual} does not match uncompressed size {expected}")]
    DestinationSizeMismatch { expected: u64, actual: u64 },

    #[error(
        "entry requires {required_bytes} scratch I/O bytes, exceeding ScratchLimits max {max_bytes}"
    )]
    ScratchCapacityExceeded { required_bytes: u64, max_bytes: u32 },

    #[error("Zlib decompression failed for entry {} block {block_index}", entry.index())]
    ZlibDecompressFailed { entry: EntryId, block_index: u32 },

    #[error("Oodle decompression failed for entry {} block {block_index}", entry.index())]
    OodleDecompressFailed { entry: EntryId, block_index: u32 },
}

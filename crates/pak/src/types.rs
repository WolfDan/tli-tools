//! Domain newtypes, algebraic sum types, and explicit `_MAX` bounds for `.pak` archives.

use crate::error::{KeyParseErrorKind, LimitKind, PakError};

/// Maximum byte length across all supported `.pak` footer dialects (`v9` and TLI `v12`).
pub const FOOTER_BYTES_MAX: u32 = 222;
/// Exact byte length of a standard UE4 `v8A` footer (`4` compression method slots).
pub const FOOTER_V8A_BYTES: u32 = 189;
/// Exact byte length of a standard UE4 `v8B`, `v10`, or `v11` footer (`5` method slots).
pub const FOOTER_V8B_V10_V11_BYTES: u32 = 221;
/// Exact byte length of a standard UE4 `v9` or Torchlight Infinite `v12` footer.
pub const FOOTER_V9_V12_BYTES: u32 = 222;

/// AES-256 block size in bytes required for encrypted index sections and entry payloads.
pub const AES_BLOCK_BYTES: u32 = 16;

/// Maximum number of named compression method slots stored in a `.pak` footer.
pub const COMPRESSION_METHODS_COUNT_MAX: u32 = 5;
/// Fixed byte width of each NUL-padded ASCII compression method slot in `FPakInfo`.
pub const COMPRESSION_METHOD_NAME_BYTES_MAX: u32 = 32;

/// Upper bound on serialized archive mount-point UTF-8 byte length.
pub const MOUNT_POINT_BYTES_MAX: u32 = 1024;
/// Upper bound on normalized relative virtual path UTF-8 byte length.
pub const PATH_BYTES_MAX: u32 = 2048;
/// Upper bound on normalized relative virtual path UTF-16LE code unit count.
pub const PATH_UTF16_UNITS_MAX: u32 = 2048;

/// Default upper bound on any single index section (`256 MiB`).
pub const INDEX_BYTES_MAX_DEFAULT: u32 = 256 * 1024 * 1024;
/// Default upper bound on entries per archive (`4,194,304`).
pub const ENTRIES_COUNT_MAX_DEFAULT: u32 = 4_194_304;
/// Hard ceiling on entries per archive so power-of-two `u32` hash tables cannot overflow.
pub const ENTRIES_COUNT_HARD_MAX: u32 = 1 << 28;
/// Architectural `u16` limit on compression blocks in a single `EncodedPakEntries` record.
pub const BLOCKS_PER_ENTRY_COUNT_MAX: u32 = 65_535;
/// Default upper bound on total compression blocks across all entries (`16,777,216`).
pub const BLOCKS_TOTAL_COUNT_MAX_DEFAULT: u32 = 16_777_216;
/// Default upper bound on uncompressed bytes of a single entry (`4 GiB`).
pub const ENTRY_UNCOMPRESSED_BYTES_MAX_DEFAULT: u64 = 4 * 1024 * 1024 * 1024;

/// Default initial `ReadScratch` I/O buffer capacity (`256 KiB`).
pub const SCRATCH_IO_BYTES_INITIAL_DEFAULT: u32 = 256 * 1024;
/// Default maximum `ReadScratch` I/O buffer capacity (`256 MiB`).
pub const SCRATCH_IO_BYTES_MAX_DEFAULT: u32 = 256 * 1024 * 1024;

/// Dense 0-based handle into `PakArchive`'s Struct-of-Arrays entry catalog (`0..entry_count`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(u32);

impl EntryId {
    #[inline]
    #[must_use]
    pub(crate) const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the 0-based dense index of this entry within its originating [`crate::PakArchive`].
    #[inline]
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// 64-bit UE4 FNV-1a digest of a lowercased UTF-16LE normalized relative path.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PathHash(u64);

impl PathHash {
    #[inline]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// 20-byte SHA-1 digest used by UE4 `FPakInfo` and `FPakEntry` integrity fields.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sha1Hash([u8; 20]);

impl Sha1Hash {
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl core::fmt::Display for Sha1Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for &byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// 16-byte `FGuid` identifying which AES-256 key encrypts a `.pak` archive.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EncryptionKeyGuid([u8; 16]);

impl EncryptionKeyGuid {
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    #[inline]
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        let mut i: usize = 0;
        while i < 16 {
            if self.0[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl core::fmt::Display for EncryptionKeyGuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for &byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// 256-bit (32-byte) AES key supplied by the caller to decrypt indexes and payloads.
#[derive(Clone, PartialEq, Eq)]
pub struct AesKey([u8; 32]);

impl AesKey {
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Validates that `slice` is exactly 32 bytes and constructs an [`AesKey`].
    pub fn try_from_slice(slice: &[u8]) -> Result<Self, PakError> {
        let arr: [u8; 32] = match slice.try_into() {
            Ok(bytes) => bytes,
            Err(_) => {
                let len_u64 = u64::try_from(slice.len()).map_err(|_| PakError::InvalidAesKey {
                    kind: KeyParseErrorKind::InvalidSliceLength { byte_len: u64::MAX },
                })?;
                return Err(PakError::InvalidAesKey {
                    kind: KeyParseErrorKind::InvalidSliceLength { byte_len: len_u64 },
                });
            }
        };
        Ok(Self(arr))
    }

    /// Parses a 64-character hexadecimal string (with optional leading `0x` or `0X`) into an [`AesKey`].
    pub fn try_from_hex(hex_str: &str) -> Result<Self, PakError> {
        let trimmed = hex_str.trim();
        let digits = match trimmed.strip_prefix("0x") {
            Some(rest) => rest,
            None => match trimmed.strip_prefix("0X") {
                Some(rest) => rest,
                None => trimmed,
            },
        };
        let raw = digits.as_bytes();
        if raw.len() != 64 {
            let len_u64 = u64::try_from(raw.len()).map_err(|_| PakError::InvalidAesKey {
                kind: KeyParseErrorKind::InvalidHexLength { byte_len: u64::MAX },
            })?;
            return Err(PakError::InvalidAesKey {
                kind: KeyParseErrorKind::InvalidHexLength { byte_len: len_u64 },
            });
        }
        let (pairs, rem) = raw.as_chunks::<2>();
        assert!(rem.is_empty());
        let mut out = [0_u8; 32];
        for (dst, pair) in out.iter_mut().zip(pairs.iter()) {
            let hi = decode_hex_nibble(pair[0])?;
            let lo = decode_hex_nibble(pair[1])?;
            *dst = (hi << 4) | lo;
        }
        Ok(Self(out))
    }

    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl core::fmt::Debug for AesKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("AesKey([REDACTED; 32])")
    }
}

const fn decode_hex_nibble(byte: u8) -> Result<u8, PakError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(PakError::InvalidAesKey {
            kind: KeyParseErrorKind::InvalidHexDigit,
        }),
    }
}

/// Wire format family detected from the footer magic and layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PakDialect {
    /// Stock Unreal Engine 4 `.pak` magic `0x5A6F12E1` (`v8A` through `v11`).
    StandardUe4,
    /// Torchlight Infinite custom `.pak` magic `0x6B2A56B8` (`v12` with `+1 u8` footer and entry prefix).
    TorchlightInfiniteV12,
}

/// Exact `.pak` format version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PakVersion {
    V8A,
    V8B,
    V9,
    V10,
    V11,
    V12,
}

impl PakVersion {
    #[inline]
    #[must_use]
    pub const fn raw_number(self) -> u32 {
        match self {
            Self::V8A | Self::V8B => 8,
            Self::V9 => 9,
            Self::V10 => 10,
            Self::V11 => 11,
            Self::V12 => 12,
        }
    }
}

/// Supported payload block compression codecs.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompressionMethod {
    None = 0,
    Zlib = 1,
    Oodle = 2,
}

bitflags::bitflags! {
    /// Per-entry bitflags stored in `FPakEntry` and reconstructed from `EncodedPakEntries`.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EntryFlags: u8 {
        /// Payload blocks are padded to 16-byte boundaries and encrypted with AES-256-ECB.
        const ENCRYPTED = 0x01;
        /// Deleted tombstone flag in stock UE4 unencoded entries (or custom `0x02` bit in TLI inline headers).
        const DELETED_OR_CUSTOM = 0x02;
    }
}

/// Validated upper bounds enforced during [`crate::PakArchive::open_with_options`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PakLimits {
    index_bytes_max: u32,
    entries_count_max: u32,
    blocks_total_count_max: u32,
    entry_uncompressed_bytes_max: u64,
}

impl PakLimits {
    pub const DEFAULT: Self = Self {
        index_bytes_max: INDEX_BYTES_MAX_DEFAULT,
        entries_count_max: ENTRIES_COUNT_MAX_DEFAULT,
        blocks_total_count_max: BLOCKS_TOTAL_COUNT_MAX_DEFAULT,
        entry_uncompressed_bytes_max: ENTRY_UNCOMPRESSED_BYTES_MAX_DEFAULT,
    };

    /// Constructs validated archive limits, rejecting zero capacities or `entries_count_max > ENTRIES_COUNT_HARD_MAX`.
    pub fn try_new(
        index_bytes_max: u32,
        entries_count_max: u32,
        blocks_total_count_max: u32,
        entry_uncompressed_bytes_max: u64,
    ) -> Result<Self, PakError> {
        if index_bytes_max == 0
            || entries_count_max == 0
            || blocks_total_count_max == 0
            || entry_uncompressed_bytes_max == 0
        {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::LimitsConfig,
                actual: 0,
                max: 1,
            });
        }
        if entries_count_max > ENTRIES_COUNT_HARD_MAX {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::EntriesCount,
                actual: u64::from(entries_count_max),
                max: u64::from(ENTRIES_COUNT_HARD_MAX),
            });
        }
        Ok(Self {
            index_bytes_max,
            entries_count_max,
            blocks_total_count_max,
            entry_uncompressed_bytes_max,
        })
    }

    #[inline]
    #[must_use]
    pub const fn index_bytes_max(self) -> u32 {
        self.index_bytes_max
    }

    #[inline]
    #[must_use]
    pub const fn entries_count_max(self) -> u32 {
        self.entries_count_max
    }

    #[inline]
    #[must_use]
    pub const fn blocks_total_count_max(self) -> u32 {
        self.blocks_total_count_max
    }

    #[inline]
    #[must_use]
    pub const fn entry_uncompressed_bytes_max(self) -> u64 {
        self.entry_uncompressed_bytes_max
    }
}

impl Default for PakLimits {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Configuration options for opening a [`crate::PakArchive`].
#[derive(Clone, Debug)]
pub struct OpenOptions {
    key: Option<AesKey>,
    verify_index_hashes: bool,
    limits: PakLimits,
}

impl OpenOptions {
    #[inline]
    #[must_use]
    pub const fn new(key: Option<AesKey>) -> Self {
        Self {
            key,
            verify_index_hashes: true,
            limits: PakLimits::DEFAULT,
        }
    }

    #[inline]
    #[must_use]
    pub const fn with_verify_index_hashes(mut self, verify: bool) -> Self {
        self.verify_index_hashes = verify;
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_limits(mut self, limits: PakLimits) -> Self {
        self.limits = limits;
        self
    }

    #[inline]
    #[must_use]
    pub const fn key(&self) -> Option<&AesKey> {
        self.key.as_ref()
    }

    #[inline]
    #[must_use]
    pub const fn verify_index_hashes(&self) -> bool {
        self.verify_index_hashes
    }

    #[inline]
    #[must_use]
    pub const fn limits(&self) -> PakLimits {
        self.limits
    }
}

/// Read-only metadata snapshot describing a single entry in a [`crate::PakArchive`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntryMeta {
    pub(crate) header_offset: u64,
    pub(crate) payload_offset: u64,
    pub(crate) compressed_bytes: u64,
    pub(crate) disk_span_bytes: u64,
    pub(crate) uncompressed_bytes: u64,
    pub(crate) compression_block_bytes: u32,
    pub(crate) block_count: u16,
    pub(crate) compression_method: CompressionMethod,
    pub(crate) flags: EntryFlags,
}

impl EntryMeta {
    #[inline]
    #[must_use]
    pub const fn header_offset(&self) -> u64 {
        self.header_offset
    }

    #[inline]
    #[must_use]
    pub const fn payload_offset(&self) -> u64 {
        self.payload_offset
    }

    #[inline]
    #[must_use]
    pub const fn compressed_bytes(&self) -> u64 {
        self.compressed_bytes
    }

    #[inline]
    #[must_use]
    pub const fn disk_span_bytes(&self) -> u64 {
        self.disk_span_bytes
    }

    #[inline]
    #[must_use]
    pub const fn uncompressed_bytes(&self) -> u64 {
        self.uncompressed_bytes
    }

    #[inline]
    #[must_use]
    pub const fn compression_block_bytes(&self) -> u32 {
        self.compression_block_bytes
    }

    #[inline]
    #[must_use]
    pub const fn block_count(&self) -> u16 {
        self.block_count
    }

    #[inline]
    #[must_use]
    pub const fn compression_method(&self) -> CompressionMethod {
        self.compression_method
    }

    #[inline]
    #[must_use]
    pub const fn flags(&self) -> EntryFlags {
        self.flags
    }

    #[inline]
    #[must_use]
    pub const fn is_encrypted(&self) -> bool {
        self.flags.contains(EntryFlags::ENCRYPTED)
    }
}

/// Caller-owned destination descriptor for [`crate::PakArchive::read_batch_into`].
#[derive(Debug)]
pub struct BatchReadRequest<'a> {
    pub(crate) id: EntryId,
    pub(crate) dst: &'a mut [u8],
}

impl<'a> BatchReadRequest<'a> {
    #[inline]
    #[must_use]
    pub fn new(id: EntryId, dst: &'a mut [u8]) -> Self {
        Self { id, dst }
    }

    #[inline]
    #[must_use]
    pub const fn id(&self) -> EntryId {
        self.id
    }
}

/// Per-item outcome written by [`crate::PakArchive::read_batch_into`].
#[must_use]
#[derive(Debug)]
pub enum BatchReadOutcome {
    /// Initial placeholder state before [`crate::PakArchive::read_batch_into`] populates the slot.
    Pending,
    /// Entry was read, decrypted, and decompressed into the request's destination slice.
    Ok { bytes_written: u64 },
    /// Reading or decompressing this specific entry failed without aborting the batch.
    Err(PakError),
}

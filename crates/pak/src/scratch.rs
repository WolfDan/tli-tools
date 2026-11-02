//! Caller-reusable bounded I/O buffer and codec decompression state (`ReadScratch`).

use flate2::Decompress;
use oozextract::Extractor;

use crate::error::{LimitKind, PakError};
use crate::types::{
    EntryId, SCRATCH_BLOCK_BYTES_MAX_DEFAULT, SCRATCH_IO_BYTES_INITIAL_DEFAULT,
    SCRATCH_IO_BYTES_MAX_DEFAULT,
};
use crate::u32_to_usize;

/// Initial capacity (`64 KiB`) reserved for `ReadScratch`'s single-block decoded window.
const SCRATCH_BLOCK_BYTES_INITIAL: u32 = 64 * 1024;

/// Explicit capacity bounds for a reusable [`ReadScratch`] instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScratchLimits {
    io_initial_bytes: u32,
    io_bytes_max: u32,
    block_bytes_max: u32,
}

impl ScratchLimits {
    pub const DEFAULT: Self = Self {
        io_initial_bytes: SCRATCH_IO_BYTES_INITIAL_DEFAULT,
        io_bytes_max: SCRATCH_IO_BYTES_MAX_DEFAULT,
        block_bytes_max: SCRATCH_BLOCK_BYTES_MAX_DEFAULT,
    };

    /// Validates that `io_bytes_max > 0` and `io_initial_bytes <= io_bytes_max`.
    pub fn try_new(io_initial_bytes: u32, io_bytes_max: u32) -> Result<Self, PakError> {
        if io_bytes_max == 0 || io_initial_bytes > io_bytes_max {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::LimitsConfig,
                actual: u64::from(io_initial_bytes),
                max: u64::from(io_bytes_max),
            });
        }
        Ok(Self {
            io_initial_bytes,
            io_bytes_max,
            block_bytes_max: SCRATCH_BLOCK_BYTES_MAX_DEFAULT,
        })
    }

    /// Sets an explicit upper bound on the single-block decoded window buffer (`block_bytes_max > 0`).
    pub fn with_block_bytes_max(mut self, block_bytes_max: u32) -> Result<Self, PakError> {
        if block_bytes_max == 0 {
            return Err(PakError::LimitExceeded {
                kind: LimitKind::ScratchBlockBytes,
                actual: 0,
                max: 1,
            });
        }
        self.block_bytes_max = block_bytes_max;
        Ok(self)
    }

    #[inline]
    #[must_use]
    pub const fn io_initial_bytes(self) -> u32 {
        self.io_initial_bytes
    }

    #[inline]
    #[must_use]
    pub const fn io_bytes_max(self) -> u32 {
        self.io_bytes_max
    }

    #[inline]
    #[must_use]
    pub const fn block_bytes_max(self) -> u32 {
        self.block_bytes_max
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CachedBlockKey {
    pub(crate) reader_cookie: u64,
    pub(crate) entry: EntryId,
    pub(crate) block_index: u32,
    pub(crate) valid_bytes: u32,
}

pub(crate) struct ScratchParts<'a> {
    pub(crate) io_buf: &'a mut [u8],
    pub(crate) oodle: &'a mut Extractor,
    pub(crate) zlib: &'a mut Decompress,
}

pub(crate) struct RangeScratchParts<'a> {
    pub(crate) io_buf: &'a mut [u8],
    pub(crate) block_buf: &'a mut [u8],
    pub(crate) cached_block: &'a mut Option<CachedBlockKey>,
    pub(crate) oodle: &'a mut Extractor,
    pub(crate) zlib: &'a mut Decompress,
}

/// Per-thread reusable buffer and codec state eliminating post-startup heap allocations on hot reads.
///
/// Construct once per worker thread via [`ReadScratch::with_default_limits`] or
/// [`ReadScratch::with_limits`] and pass `&mut ReadScratch` to [`crate::PakArchive::read_entry_into`],
/// [`crate::PakArchive::read_entry_range_into`], [`crate::PakArchive::read_batch_into`], and
/// [`crate::PakArchive::verify_entry_sha1`] (and their [`crate::PakPayloadReader`] counterparts).
pub struct ReadScratch {
    io_buf: Vec<u8>,
    io_bytes_max: u32,
    block_buf: Vec<u8>,
    block_bytes_max: u32,
    cached_block: Option<CachedBlockKey>,
    oodle: Extractor,
    zlib: Decompress,
}

impl core::fmt::Debug for ReadScratch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReadScratch")
            .field("io_buf_len", &self.io_buf.len())
            .field("io_buf_cap", &self.io_buf.capacity())
            .field("io_bytes_max", &self.io_bytes_max)
            .field("block_buf_len", &self.block_buf.len())
            .field("block_buf_cap", &self.block_buf.capacity())
            .field("block_bytes_max", &self.block_bytes_max)
            .finish_non_exhaustive()
    }
}

impl ReadScratch {
    /// Allocates reusable Oodle (`~768 KiB`), Zlib, single-block window, and bounded I/O scratch buffers upfront.
    #[must_use]
    pub fn with_limits(limits: ScratchLimits) -> Self {
        let initial_io = u32_to_usize(limits.io_initial_bytes());
        let initial_blk = u32_to_usize(SCRATCH_BLOCK_BYTES_INITIAL.min(limits.block_bytes_max()));
        Self {
            io_buf: vec![0_u8; initial_io],
            io_bytes_max: limits.io_bytes_max(),
            block_buf: vec![0_u8; initial_blk],
            block_bytes_max: limits.block_bytes_max(),
            cached_block: None,
            oodle: Extractor::new(),
            zlib: Decompress::new(true),
        }
    }

    /// Allocates a [`ReadScratch`] using [`ScratchLimits::DEFAULT`].
    #[must_use]
    pub fn with_default_limits() -> Self {
        Self::with_limits(ScratchLimits::DEFAULT)
    }

    #[inline]
    #[must_use]
    pub const fn io_bytes_max(&self) -> u32 {
        self.io_bytes_max
    }

    #[inline]
    #[must_use]
    pub const fn block_bytes_max(&self) -> u32 {
        self.block_bytes_max
    }

    #[inline]
    #[must_use]
    pub(crate) fn cached_block_slice(
        &self,
        reader_cookie: u64,
        entry: EntryId,
        block_index: u32,
        expected_valid_bytes: u64,
    ) -> Option<&[u8]> {
        let key = self.cached_block?;
        if key.reader_cookie == reader_cookie
            && key.entry == entry
            && key.block_index == block_index
            && u64::from(key.valid_bytes) == expected_valid_bytes
        {
            let end = u32_to_usize(key.valid_bytes);
            assert!(end <= self.block_buf.len());
            Some(&self.block_buf[..end])
        } else {
            None
        }
    }

    pub(crate) fn prepare_parts(
        &mut self,
        required_bytes: u64,
    ) -> Result<ScratchParts<'_>, PakError> {
        let io_buf = ensure_io_buf(&mut self.io_buf, self.io_bytes_max, required_bytes)?;
        Ok(ScratchParts {
            io_buf,
            oodle: &mut self.oodle,
            zlib: &mut self.zlib,
        })
    }

    pub(crate) fn prepare_range_parts(
        &mut self,
        required_io_bytes: u64,
        required_block_bytes: u32,
    ) -> Result<RangeScratchParts<'_>, PakError> {
        let io_buf = ensure_io_buf(&mut self.io_buf, self.io_bytes_max, required_io_bytes)?;
        let block_buf = ensure_block_buf(
            &mut self.block_buf,
            self.block_bytes_max,
            required_block_bytes,
        )?;
        if required_block_bytes > 0 {
            // Invalidate any previously cached block before mutating `block_buf` so failed
            // decompressions can never expose partially overwritten bytes on subsequent reads.
            self.cached_block = None;
        }
        Ok(RangeScratchParts {
            io_buf,
            block_buf,
            cached_block: &mut self.cached_block,
            oodle: &mut self.oodle,
            zlib: &mut self.zlib,
        })
    }
}

fn ensure_io_buf(
    io_buf: &mut Vec<u8>,
    io_bytes_max: u32,
    required_bytes: u64,
) -> Result<&mut [u8], PakError> {
    let req_u32 = match u32::try_from(required_bytes) {
        Ok(n) if n <= io_bytes_max => n,
        Ok(_) | Err(_) => {
            return Err(PakError::ScratchCapacityExceeded {
                required_bytes,
                max_bytes: io_bytes_max,
            });
        }
    };
    let req_usize = u32_to_usize(req_u32);
    let max_usize = u32_to_usize(io_bytes_max);
    grow_bounded_vec(io_buf, req_usize, max_usize);
    Ok(&mut io_buf[..req_usize])
}

fn ensure_block_buf(
    block_buf: &mut Vec<u8>,
    block_bytes_max: u32,
    required_bytes: u32,
) -> Result<&mut [u8], PakError> {
    if required_bytes > block_bytes_max {
        return Err(PakError::LimitExceeded {
            kind: LimitKind::ScratchBlockBytes,
            actual: u64::from(required_bytes),
            max: u64::from(block_bytes_max),
        });
    }
    let req_usize = u32_to_usize(required_bytes);
    let max_usize = u32_to_usize(block_bytes_max);
    grow_bounded_vec(block_buf, req_usize, max_usize);
    Ok(&mut block_buf[..req_usize])
}

fn grow_bounded_vec(buf: &mut Vec<u8>, req_usize: usize, max_usize: usize) {
    if buf.len() < req_usize {
        if buf.capacity() < req_usize {
            let target_cap = req_usize.saturating_mul(2).min(max_usize);
            let additional = target_cap.strict_sub(buf.len());
            buf.reserve_exact(additional);
        }
        buf.resize(req_usize, 0);
    }
    assert!(buf.len() >= req_usize);
    assert!(buf.len() <= max_usize);
}

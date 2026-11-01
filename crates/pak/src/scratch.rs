//! Caller-reusable bounded I/O buffer and codec decompression state (`ReadScratch`).

use flate2::Decompress;
use oozextract::Extractor;

use crate::error::{LimitKind, PakError};
use crate::types::{SCRATCH_IO_BYTES_INITIAL_DEFAULT, SCRATCH_IO_BYTES_MAX_DEFAULT};
use crate::u32_to_usize;

/// Explicit capacity bounds for a reusable [`ReadScratch`] instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScratchLimits {
    io_initial_bytes: u32,
    io_bytes_max: u32,
}

impl ScratchLimits {
    pub const DEFAULT: Self = Self {
        io_initial_bytes: SCRATCH_IO_BYTES_INITIAL_DEFAULT,
        io_bytes_max: SCRATCH_IO_BYTES_MAX_DEFAULT,
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
        })
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
}

impl Default for ScratchLimits {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub(crate) struct ScratchParts<'a> {
    pub(crate) io_buf: &'a mut [u8],
    pub(crate) oodle: &'a mut Extractor,
    pub(crate) zlib: &'a mut Decompress,
}

/// Per-thread reusable buffer and codec state eliminating post-startup heap allocations on hot reads.
///
/// Construct once per worker thread via [`ReadScratch::with_default_limits`] or
/// [`ReadScratch::with_limits`] and pass `&mut ReadScratch` to [`crate::PakArchive::read_entry_into`]
/// and [`crate::PakArchive::read_batch_into`].
pub struct ReadScratch {
    io_buf: Vec<u8>,
    io_bytes_max: u32,
    oodle: Extractor,
    zlib: Decompress,
}

impl core::fmt::Debug for ReadScratch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReadScratch")
            .field("io_buf_len", &self.io_buf.len())
            .field("io_buf_cap", &self.io_buf.capacity())
            .field("io_bytes_max", &self.io_bytes_max)
            .finish_non_exhaustive()
    }
}

impl ReadScratch {
    /// Allocates reusable Oodle (`~768 KiB`), Zlib, and bounded I/O scratch buffers upfront.
    #[must_use]
    pub fn with_limits(limits: ScratchLimits) -> Self {
        let initial = u32_to_usize(limits.io_initial_bytes());
        Self {
            io_buf: vec![0_u8; initial],
            io_bytes_max: limits.io_bytes_max(),
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

    pub(crate) fn prepare_parts(
        &mut self,
        required_bytes: u64,
    ) -> Result<ScratchParts<'_>, PakError> {
        if required_bytes > u64::from(self.io_bytes_max) {
            return Err(PakError::ScratchCapacityExceeded {
                required_bytes,
                max_bytes: self.io_bytes_max,
            });
        }
        let req_u32 =
            u32::try_from(required_bytes).map_err(|_| PakError::ScratchCapacityExceeded {
                required_bytes,
                max_bytes: self.io_bytes_max,
            })?;
        let req_usize = u32_to_usize(req_u32);
        let max_usize = u32_to_usize(self.io_bytes_max);
        if self.io_buf.len() < req_usize {
            if self.io_buf.capacity() < req_usize {
                let target_cap = req_usize.saturating_mul(2).min(max_usize);
                let additional = target_cap.strict_sub(self.io_buf.len());
                self.io_buf.reserve_exact(additional);
            }
            self.io_buf.resize(req_usize, 0);
        }
        assert!(self.io_buf.len() <= max_usize);
        Ok(ScratchParts {
            io_buf: &mut self.io_buf[..req_usize],
            oodle: &mut self.oodle,
            zlib: &mut self.zlib,
        })
    }
}

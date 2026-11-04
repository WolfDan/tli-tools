//! Block-caching random-access reader cursor (`VfsCursor`) implementing `std::io::Read + Seek`.

use std::io::{Read, Seek, SeekFrom};

use pak::ReadScratch;

use crate::error::VfsError;
use crate::types::FileId;
use crate::vfs::PakVfs;

/// Zero-allocation streaming cursor over a single virtual file inside [`PakVfs`].
///
/// Reuses [`ReadScratch`]'s single-block decoded window so sequential small `read` calls across
/// `64 KiB` Oodle or Zlib blocks avoid redundant positioned syscalls, AES decryptions, and
/// block decompressions.
#[derive(Debug)]
pub struct VfsCursor<'vfs, 'scratch> {
    vfs: &'vfs PakVfs,
    scratch: &'scratch mut ReadScratch,
    file: FileId,
    pos: u64,
    file_bytes: u64,
}

impl<'vfs, 'scratch> VfsCursor<'vfs, 'scratch> {
    #[inline]
    #[must_use]
    pub(crate) fn new(vfs: &'vfs PakVfs, scratch: &'scratch mut ReadScratch, file: FileId) -> Self {
        let file_bytes = vfs.file_meta(file).uncompressed_bytes();
        Self {
            vfs,
            scratch,
            file,
            pos: 0,
            file_bytes,
        }
    }

    #[inline]
    #[must_use]
    pub const fn file_id(&self) -> FileId {
        self.file
    }

    #[inline]
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.pos
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> u64 {
        self.file_bytes
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.file_bytes == 0
    }

    /// Number of unread logical bytes remaining (`0` if positioned at or past EOF).
    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> u64 {
        self.file_bytes.saturating_sub(self.pos)
    }

    /// Reads exactly `dst.len()` bytes starting at [`VfsCursor::position`] and advances the cursor,
    /// returning a typed [`VfsError`] without boxing into [`std::io::Error`].
    pub fn read_exact_slice(&mut self, dst: &mut [u8]) -> Result<(), VfsError> {
        let len_u64 = u64::try_from(dst.len()).map_err(|_| VfsError::OutOfBoundsRange {
            file: self.file,
            offset: self.pos,
            byte_len: u64::MAX,
            file_bytes: self.file_bytes,
        })?;
        self.vfs
            .read_range_into(self.file, self.pos, self.scratch, dst)?;
        self.pos += len_u64;
        Ok(())
    }
}

impl Read for VfsCursor<'_, '_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() || self.pos >= self.file_bytes {
            return Ok(0);
        }
        let avail = self.file_bytes - self.pos;
        let to_read_usize = match usize::try_from(avail) {
            Ok(avail_usize) => buf.len().min(avail_usize),
            Err(_) => buf.len(),
        };
        let to_read_u64 = match u64::try_from(to_read_usize) {
            Ok(n) => n,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "read buffer size exceeds u64::MAX",
                ));
            }
        };
        self.vfs
            .read_range_into(self.file, self.pos, self.scratch, &mut buf[..to_read_usize])
            .map_err(std::io::Error::other)?;
        self.pos += to_read_u64;
        Ok(to_read_usize)
    }
}

impl Seek for VfsCursor<'_, '_> {
    fn seek(&mut self, style: SeekFrom) -> std::io::Result<u64> {
        let next_pos = match style {
            SeekFrom::Start(off) => Some(off),
            SeekFrom::Current(delta) => checked_add_signed_u64(self.pos, delta),
            SeekFrom::End(delta) => checked_add_signed_u64(self.file_bytes, delta),
        };
        match next_pos {
            Some(pos) => {
                self.pos = pos;
                Ok(pos)
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }
}

#[inline]
const fn checked_add_signed_u64(base: u64, delta: i64) -> Option<u64> {
    if delta >= 0 {
        base.checked_add(delta.unsigned_abs())
    } else {
        base.checked_sub(delta.unsigned_abs())
    }
}

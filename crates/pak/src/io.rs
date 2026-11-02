//! Cross-platform positioned file I/O (`pread` on Unix, `OVERLAPPED` `seek_read` on Windows).

use std::fs::File;
use std::path::Path;

use crate::error::PakError;

#[cfg(windows)]
const WINDOWS_READ_SYSCALL_STEPS_MAX: u32 = 1_048_576;

/// Immutable file handle supporting lock-free concurrent positioned reads across threads.
#[derive(Debug)]
pub(crate) struct PositionedFile {
    file: File,
    file_bytes: u64,
}

impl PositionedFile {
    pub(crate) fn open(path: &Path) -> Result<Self, PakError> {
        let file = File::open(path).map_err(|source| PakError::Io {
            offset: 0,
            len: 0,
            source,
        })?;
        let metadata = file.metadata().map_err(|source| PakError::Io {
            offset: 0,
            len: 0,
            source,
        })?;
        Ok(Self {
            file,
            file_bytes: metadata.len(),
        })
    }

    #[inline]
    #[must_use]
    pub(crate) const fn file_bytes(&self) -> u64 {
        self.file_bytes
    }

    /// Reads `dst.len()` bytes starting at `offset` without mutating the OS file cursor.
    ///
    /// Total over empty slices (`&mut []`), validating `offset <= file_bytes` upfront.
    pub(crate) fn read_exact_at(&self, offset: u64, dst: &mut [u8]) -> Result<(), PakError> {
        let len_u64 = u64::try_from(dst.len()).map_err(|_| PakError::OutOfBoundsRange {
            offset,
            byte_len: u64::MAX,
            container_bytes: self.file_bytes,
        })?;
        let end = offset
            .checked_add(len_u64)
            .ok_or(PakError::OutOfBoundsRange {
                offset,
                byte_len: len_u64,
                container_bytes: self.file_bytes,
            })?;
        if end > self.file_bytes {
            return Err(PakError::OutOfBoundsRange {
                offset,
                byte_len: len_u64,
                container_bytes: self.file_bytes,
            });
        }
        if dst.is_empty() {
            return Ok(());
        }
        sys_read_exact_at(&self.file, offset, dst).map_err(|source| PakError::Io {
            offset,
            len: len_u64,
            source,
        })
    }
}

#[cfg(unix)]
fn sys_read_exact_at(file: &File, offset: u64, dst: &mut [u8]) -> std::io::Result<()> {
    // Unix `FileExt::read_exact_at` handles `EINTR` and partial-read retries internally via `pread`.
    std::os::unix::fs::FileExt::read_exact_at(file, dst, offset)
}

#[cfg(windows)]
fn sys_read_exact_at(file: &File, mut offset: u64, mut dst: &mut [u8]) -> std::io::Result<()> {
    for _ in 0..WINDOWS_READ_SYSCALL_STEPS_MAX {
        if dst.is_empty() {
            return Ok(());
        }
        let bytes_read = std::os::windows::fs::FileExt::seek_read(file, dst, offset)?;
        if bytes_read == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        let read_u64 = u64::try_from(bytes_read)
            .map_err(|_| std::io::Error::other("positioned read byte count overflowed u64"))?;
        offset = offset
            .checked_add(read_u64)
            .ok_or_else(|| std::io::Error::other("positioned read offset overflowed u64"))?;
        let (_, remaining) = dst
            .split_at_mut_checked(bytes_read)
            .ok_or_else(|| std::io::Error::other("positioned read count exceeded buffer"))?;
        dst = remaining;
    }
    if dst.is_empty() {
        return Ok(());
    }
    Err(std::io::Error::other(
        "Windows positioned read exceeded bounded syscall step limit",
    ))
}

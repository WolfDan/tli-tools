//! High-performance, data-oriented TigerStyle parsers for files extracted from `.pak` archives.

pub mod bin;
pub mod luac;

const _: () = assert!(usize::BITS >= 32);
const _: () = assert!(usize::BITS <= 64);

/// Converts a `u32` into `usize` without loss on 32-bit or 64-bit targets.
///
/// This is the sole permitted `as usize` conversion in the crate.
#[inline(always)]
#[must_use]
pub const fn u32_to_usize(x: u32) -> usize {
    x as usize
}

/// Converts a slice length bounded by a `u32` capacity into `u32`.
#[inline(always)]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "Caller guarantees len is bounded by a u32 capacity limit"
)]
pub(crate) const fn bounded_len_u32(len: usize) -> u32 {
    assert!(len <= u32_to_usize(u32::MAX));
    len as u32
}

/// Encapsulated fixed-capacity vector whose logical capacity is bounded by an explicit `u32` limit.
#[derive(Debug, Clone)]
pub(crate) struct FixedVec<T> {
    inner: Vec<T>,
    cap: u32,
}

impl<T> FixedVec<T> {
    #[must_use]
    pub(crate) fn with_capacity(cap: u32) -> Self {
        Self {
            inner: Vec::with_capacity(u32_to_usize(cap)),
            cap,
        }
    }

    #[inline]
    pub(crate) fn try_push(&mut self, item: T) -> Result<(), T> {
        if self.inner.len() == u32_to_usize(self.cap) {
            return Err(item);
        }
        let raw_cap_before = self.inner.capacity();
        self.inner.push(item);
        assert!(self.inner.capacity() == raw_cap_before);
        Ok(())
    }

    #[inline]
    #[must_use]
    pub(crate) fn len(&self) -> u32 {
        bounded_len_u32(self.inner.len())
    }

    #[inline]
    #[must_use]
    pub(crate) fn cap(&self) -> u32 {
        self.cap
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub(crate) fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    #[inline]
    #[must_use]
    pub(crate) fn as_slice(&self) -> &[T] {
        &self.inner
    }

    #[inline]
    #[must_use]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.inner
    }
}

impl FixedVec<u8> {
    #[inline]
    pub(crate) fn try_extend_from_slice(&mut self, bytes: &[u8]) -> bool {
        let Some(new_len) = self.inner.len().checked_add(bytes.len()) else {
            return false;
        };
        if new_len > u32_to_usize(self.cap) {
            return false;
        }
        let raw_cap_before = self.inner.capacity();
        self.inner.extend_from_slice(bytes);
        assert!(self.inner.capacity() == raw_cap_before);
        true
    }

    #[inline]
    pub(crate) fn resize_within_cap(&mut self, new_len: u32, val: u8) -> bool {
        if new_len > self.cap {
            return false;
        }
        let raw_cap_before = self.inner.capacity();
        self.inner.resize(u32_to_usize(new_len), val);
        assert!(self.inner.capacity() == raw_cap_before);
        true
    }
}

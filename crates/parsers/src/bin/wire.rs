//! `#[repr(C)]` `zerocopy` wire structures and `WireCursor` column decoders.

use crate::bin::error::{BinError, BinLimitKind, WireContext};
use crate::bin::types::{BinLimits, VecDim};
use crate::u32_to_usize;
use rustc_hash::{FxBuildHasher, FxHashMap};
use zerocopy::byteorder::little_endian::{F64, I32, U32};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

/// 8-byte header at offset `0` of every `*_ChunkA0.bin` file.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RawChunkHeader {
    /// Number of `RawBlockDesc` entries immediately following this header.
    pub block_count: U32,
    /// Hash bucket count for the outer key table.
    pub hash_buckets: U32,
}
const _: () = assert!(core::mem::size_of::<RawChunkHeader>() == 8);

/// 8-byte block descriptor entry in the `*_ChunkA0.bin` block table.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RawBlockDesc {
    /// Byte offset of this block's compressed frame relative to the start of the payload section.
    pub offset_compressed: U32,
    /// Expected uncompressed byte size after Zstandard frame decompression (`0` for empty block).
    pub size_uncompressed: U32,
}
const _: () = assert!(core::mem::size_of::<RawBlockDesc>() == 8);

/// 8-byte header at offset `0` of every uncompressed `Maps/<Map>/config_*.bin` file.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RawMapHeader {
    /// Hash bucket count.
    pub buckets: U32,
    /// Number of top-level keys.
    pub top_count: U32,
}
const _: () = assert!(core::mem::size_of::<RawMapHeader>() == 8);

/// 8-byte little-endian `(i32, i32)` wire pair (`FConfigDamage`, `FConfigSpecialValue`, etc.).
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RawPairI32 {
    /// First `i32` field.
    pub a: I32,
    /// Second `i32` field.
    pub b: I32,
}
const _: () = assert!(core::mem::size_of::<RawPairI32>() == 8);

/// 12-byte little-endian `(i32, f64)` wire pair (`FConfigAttr`, `FConfigDropShowItem`, etc.).
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RawI32F64 {
    /// 32-bit signed integer field.
    pub a: I32,
    /// 64-bit IEEE-754 float field.
    pub b: F64,
}
const _: () = assert!(core::mem::size_of::<RawI32F64>() == 12);

/// 32-byte little-endian RGBA `f64` color (`FLinearColor`).
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RawLinearColorF64 {
    /// Red channel.
    pub r: F64,
    /// Green channel.
    pub g: F64,
    /// Blue channel.
    pub b: F64,
    /// Alpha channel.
    pub a: F64,
}
const _: () = assert!(core::mem::size_of::<RawLinearColorF64>() == 32);

/// 16-byte little-endian 2D `f64` vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RawVec2F64 {
    /// X coordinate.
    pub x: F64,
    /// Y coordinate.
    pub y: F64,
}
const _: () = assert!(core::mem::size_of::<RawVec2F64>() == 16);

/// 24-byte little-endian 3D `f64` vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RawVec3F64 {
    /// X coordinate.
    pub x: F64,
    /// Y coordinate.
    pub y: F64,
    /// Z coordinate.
    pub z: F64,
}
const _: () = assert!(core::mem::size_of::<RawVec3F64>() == 24);

/// 12-byte entry inside an `FConfigAffix` tag-4 lookup table node.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RawTreeNodeMapEntry {
    /// Map lookup key.
    pub key: I32,
    /// First mapped value.
    pub val_a: I32,
    /// Second mapped value.
    pub val_b: I32,
}
const _: () = assert!(core::mem::size_of::<RawTreeNodeMapEntry>() == 12);

/// 20-byte fixed prefix of a `config_affix_effect.bin` record.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RawAffixEffectFixed {
    /// Outer affix ID.
    pub id: I32,
    /// Outer sub-ID.
    pub sub_id: I32,
    /// Inner sub-ID.
    pub sub_id_inner: I32,
    /// Inner affix ID.
    pub id_inner: I32,
    /// Leading effect parameter 0.
    pub param0: I32,
}
const _: () = assert!(core::mem::size_of::<RawAffixEffectFixed>() == 20);

/// Forward-only fat-slice cursor that shrinks `&'a [u8]` via `zerocopy` and checked sub-slicing.
#[derive(Debug, Clone, Copy)]
pub struct WireCursor<'a> {
    buf: &'a [u8],
    limits: BinLimits,
}

impl<'a> WireCursor<'a> {
    /// Creates a new `WireCursor` over `buf` governed by `limits`.
    #[inline]
    #[must_use]
    pub const fn new(buf: &'a [u8], limits: BinLimits) -> Self {
        Self { buf, limits }
    }

    /// Returns the unconsumed tail slice.
    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> &'a [u8] {
        self.buf
    }

    /// Returns `true` if all bytes have been consumed.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Returns the active [`BinLimits`].
    #[inline]
    #[must_use]
    pub const fn limits(&self) -> BinLimits {
        self.limits
    }

    /// Consumes and returns a zero-copy reference to `T` from the front of the slice.
    #[inline]
    pub fn take<T>(&mut self, context: WireContext) -> Result<&'a T, BinError>
    where
        T: FromBytes + KnownLayout + Immutable + Unaligned,
    {
        let (item, rest) =
            T::ref_from_prefix(self.buf).map_err(|_| BinError::CorruptWireLayout { context })?;
        self.buf = rest;
        Ok(item)
    }

    /// Consumes and returns a zero-copy slice of `count` elements of `T`.
    #[inline]
    pub fn take_slice_elems<T>(
        &mut self,
        count: u32,
        context: WireContext,
    ) -> Result<&'a [T], BinError>
    where
        T: FromBytes + KnownLayout + Immutable + Unaligned,
    {
        let (slice, rest) = <[T]>::ref_from_prefix_with_elems(self.buf, u32_to_usize(count))
            .map_err(|_| BinError::CorruptWireLayout { context })?;
        self.buf = rest;
        Ok(slice)
    }

    /// Consumes and returns `len` raw bytes from the front of the slice.
    #[inline]
    pub fn take_bytes(&mut self, len: u32, context: WireContext) -> Result<&'a [u8], BinError> {
        let (head, tail) = self
            .buf
            .split_at_checked(u32_to_usize(len))
            .ok_or(BinError::CorruptWireLayout { context })?;
        self.buf = tail;
        Ok(head)
    }

    /// Reads a single `u8` in `context`.
    #[inline]
    pub fn read_u8(&mut self, context: WireContext) -> Result<u8, BinError> {
        let (&b, rest) = self
            .buf
            .split_first()
            .ok_or(BinError::CorruptWireLayout { context })?;
        self.buf = rest;
        Ok(b)
    }

    /// Reads a 1-byte boolean column (`0 -> false`, non-zero -> `true`).
    #[inline]
    pub fn read_bool(&mut self) -> Result<bool, BinError> {
        Ok(self.read_u8(WireContext::ScalarColumn)? != 0)
    }

    /// Reads a little-endian `u32` in `context`.
    #[inline]
    pub fn read_u32(&mut self, context: WireContext) -> Result<u32, BinError> {
        Ok(self.take::<U32>(context)?.get())
    }

    /// Reads a little-endian `i32` in `context`.
    #[inline]
    pub fn read_i32_ctx(&mut self, context: WireContext) -> Result<i32, BinError> {
        Ok(self.take::<I32>(context)?.get())
    }

    /// Reads a little-endian `i32` scalar column.
    #[inline]
    pub fn read_i32(&mut self) -> Result<i32, BinError> {
        self.read_i32_ctx(WireContext::ScalarColumn)
    }

    /// Reads a little-endian `f64` scalar column.
    #[inline]
    pub fn read_f64(&mut self) -> Result<f64, BinError> {
        Ok(self.take::<F64>(WireContext::ScalarColumn)?.get())
    }

    /// Reads an array/collection element count and validates it against `collection_count_max`.
    #[inline]
    pub fn read_collection_count(&mut self, context: WireContext) -> Result<u32, BinError> {
        let count = self.read_u32(context)?;
        if count > self.limits.collection_count_max() {
            return Err(BinError::LimitExceeded {
                kind: BinLimitKind::CollectionCount,
                actual: u64::from(count),
                max: u64::from(self.limits.collection_count_max()),
            });
        }
        Ok(count)
    }

    /// Reads a map column's `<buckets: u32><count: u32>` prefix and validates `count`.
    #[inline]
    pub fn read_map_count(&mut self) -> Result<u32, BinError> {
        let _buckets = self.read_u32(WireContext::MapColumn)?;
        self.read_collection_count(WireContext::MapColumn)
    }

    /// Reads a length-prefixed byte slice bounded by `string_bytes_max`.
    #[inline(never)]
    pub fn read_bytes_slice(&mut self, context: WireContext) -> Result<&'a [u8], BinError> {
        let len = self.read_u32(context)?;
        if len > self.limits.string_bytes_max() {
            return Err(BinError::LimitExceeded {
                kind: BinLimitKind::StringBytes,
                actual: u64::from(len),
                max: u64::from(self.limits.string_bytes_max()),
            });
        }
        self.take_bytes(len, context)
    }

    /// Reads a length-prefixed UTF-8 string in `context`.
    #[inline(never)]
    pub fn read_str_ctx(&mut self, context: WireContext) -> Result<Box<str>, BinError> {
        let bytes = self.read_bytes_slice(context)?;
        let s = core::str::from_utf8(bytes)?;
        Ok(s.into())
    }

    /// Reads a length-prefixed UTF-8 string column.
    #[inline(never)]
    pub fn read_str(&mut self) -> Result<Box<str>, BinError> {
        self.read_str_ctx(WireContext::StringColumn)
    }

    /// Reads a length-prefixed raw byte slice column (`Box<[u8]>`).
    #[inline(never)]
    pub fn read_bytes(&mut self) -> Result<Box<[u8]>, BinError> {
        let bytes = self.read_bytes_slice(WireContext::StringColumn)?;
        Ok(bytes.into())
    }

    /// Reads a `VecDim` column (`<dim: u8>` + `f64[2]` or `f64[3]`).
    #[inline(never)]
    pub fn read_vec_dim(&mut self) -> Result<VecDim, BinError> {
        let dim = self.read_u8(WireContext::VecDimColumn)?;
        match dim {
            0 => Ok(VecDim::Vec2([0.0, 0.0])),
            2 => {
                let raw = self.take::<RawVec2F64>(WireContext::VecDimColumn)?;
                Ok(VecDim::Vec2([raw.x.get(), raw.y.get()]))
            }
            3 => {
                let raw = self.take::<RawVec3F64>(WireContext::VecDimColumn)?;
                Ok(VecDim::Vec3([raw.x.get(), raw.y.get(), raw.z.get()]))
            }
            other => Err(BinError::InvalidVecDim { raw_dim: other }),
        }
    }

    /// Reads a `<count: u32><i32[count]>` array column.
    #[inline(never)]
    pub fn read_array_i32(&mut self) -> Result<Box<[i32]>, BinError> {
        let count = self.read_collection_count(WireContext::ArrayColumn)?;
        let raw = self.take_slice_elems::<I32>(count, WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for elem in raw {
            out.push(elem.get());
        }
        Ok(out.into_boxed_slice())
    }

    /// Reads a `<count: u32><f64[count]>` array column.
    #[inline(never)]
    pub fn read_array_f64(&mut self) -> Result<Box<[f64]>, BinError> {
        let count = self.read_collection_count(WireContext::ArrayColumn)?;
        let raw = self.take_slice_elems::<F64>(count, WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for elem in raw {
            out.push(elem.get());
        }
        Ok(out.into_boxed_slice())
    }

    /// Reads a `<count: u32><Str[count]>` array column.
    #[inline(never)]
    pub fn read_array_str(&mut self) -> Result<Box<[Box<str>]>, BinError> {
        let count = self.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(self.read_str()?);
        }
        Ok(out.into_boxed_slice())
    }

    /// Reads a `<count: u32><VecDim[count]>` array column.
    #[inline(never)]
    pub fn read_array_vec_dim(&mut self) -> Result<Box<[VecDim]>, BinError> {
        let count = self.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(self.read_vec_dim()?);
        }
        Ok(out.into_boxed_slice())
    }

    /// Reads a 2D `<count: u32><ArrayI32[count]>` column.
    #[inline(never)]
    pub fn read_array_2d_i32(&mut self) -> Result<Box<[Box<[i32]>]>, BinError> {
        let count = self.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(self.read_array_i32()?);
        }
        Ok(out.into_boxed_slice())
    }

    /// Reads a 2D `<count: u32><ArrayF64[count]>` column.
    #[inline(never)]
    pub fn read_array_2d_f64(&mut self) -> Result<Box<[Box<[f64]>]>, BinError> {
        let count = self.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(self.read_array_f64()?);
        }
        Ok(out.into_boxed_slice())
    }

    /// Reads a `<buckets: u32><count: u32><(i32, i32)[count]>` map column.
    #[inline(never)]
    pub fn read_map_i32_i32(&mut self) -> Result<FxHashMap<i32, i32>, BinError> {
        let count = self.read_map_count()?;
        let pairs = self.take_slice_elems::<RawPairI32>(count, WireContext::MapColumn)?;
        let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(count), FxBuildHasher);
        for pair in pairs {
            if map.insert(pair.a.get(), pair.b.get()).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::MapColumn,
                });
            }
        }
        Ok(map)
    }

    /// Reads a `<buckets: u32><count: u32><(i32, ArrayI32)[count]>` map column.
    #[inline(never)]
    pub fn read_map_i32_array_i32(&mut self) -> Result<FxHashMap<i32, Box<[i32]>>, BinError> {
        let count = self.read_map_count()?;
        let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(count), FxBuildHasher);
        for _ in 0..count {
            let k = self.read_i32_ctx(WireContext::MapColumn)?;
            let v = self.read_array_i32()?;
            if map.insert(k, v).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::MapColumn,
                });
            }
        }
        Ok(map)
    }

    /// Reads a `<buckets: u32><count: u32><(i32, ArrayF64)[count]>` map column.
    #[inline(never)]
    pub fn read_map_i32_array_f64(&mut self) -> Result<FxHashMap<i32, Box<[f64]>>, BinError> {
        let count = self.read_map_count()?;
        let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(count), FxBuildHasher);
        for _ in 0..count {
            let k = self.read_i32_ctx(WireContext::MapColumn)?;
            let v = self.read_array_f64()?;
            if map.insert(k, v).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::MapColumn,
                });
            }
        }
        Ok(map)
    }

    /// Reads a `<buckets: u32><count: u32><(i32, Str)[count]>` map column.
    #[inline(never)]
    pub fn read_map_i32_str(&mut self) -> Result<FxHashMap<i32, Box<str>>, BinError> {
        let count = self.read_map_count()?;
        let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(count), FxBuildHasher);
        for _ in 0..count {
            let k = self.read_i32_ctx(WireContext::MapColumn)?;
            let v = self.read_str()?;
            if map.insert(k, v).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::MapColumn,
                });
            }
        }
        Ok(map)
    }

    /// Reads a `<buckets: u32><count: u32><(Str, i32)[count]>` map column.
    #[inline(never)]
    pub fn read_map_str_i32(&mut self) -> Result<FxHashMap<Box<str>, i32>, BinError> {
        let count = self.read_map_count()?;
        let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(count), FxBuildHasher);
        for _ in 0..count {
            let k = self.read_str_ctx(WireContext::MapColumn)?;
            let v = self.read_i32_ctx(WireContext::MapColumn)?;
            if map.insert(k, v).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::MapColumn,
                });
            }
        }
        Ok(map)
    }

    /// Reads a `<buckets: u32><count: u32><(Str, Str)[count]>` map column.
    #[inline(never)]
    pub fn read_map_str_str(&mut self) -> Result<FxHashMap<Box<str>, Box<str>>, BinError> {
        let count = self.read_map_count()?;
        let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(count), FxBuildHasher);
        for _ in 0..count {
            let k = self.read_str_ctx(WireContext::MapColumn)?;
            let v = self.read_str()?;
            if map.insert(k, v).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::MapColumn,
                });
            }
        }
        Ok(map)
    }

    /// Reads a `<buckets: u32><count: u32><(Str, ArrayI32)[count]>` map column.
    #[inline(never)]
    pub fn read_map_str_array_i32(&mut self) -> Result<FxHashMap<Box<str>, Box<[i32]>>, BinError> {
        let count = self.read_map_count()?;
        let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(count), FxBuildHasher);
        for _ in 0..count {
            let k = self.read_str_ctx(WireContext::MapColumn)?;
            let v = self.read_array_i32()?;
            if map.insert(k, v).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::MapColumn,
                });
            }
        }
        Ok(map)
    }

    /// Reads a `<buckets: u32><count: u32><(Str, f64)[count]>` map column.
    #[inline(never)]
    pub fn read_map_str_f64(&mut self) -> Result<FxHashMap<Box<str>, f64>, BinError> {
        let count = self.read_map_count()?;
        let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(count), FxBuildHasher);
        for _ in 0..count {
            let k = self.read_str_ctx(WireContext::MapColumn)?;
            let v = self.read_f64()?;
            if map.insert(k, v).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::MapColumn,
                });
            }
        }
        Ok(map)
    }

    /// Reads a `<count: u32><T[count]>` array column using `read_elem`.
    pub fn read_array_with<T, F>(&mut self, mut read_elem: F) -> Result<Box<[T]>, BinError>
    where
        F: FnMut(&mut Self) -> Result<T, BinError>,
    {
        let count = self.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(read_elem(self)?);
        }
        Ok(out.into_boxed_slice())
    }

    /// Reads a `<buckets: u32><count: u32><(i32, V)[count]>` map column using `read_val`.
    pub fn read_map_i32_with<V, F>(
        &mut self,
        mut read_val: F,
    ) -> Result<FxHashMap<i32, V>, BinError>
    where
        F: FnMut(&mut Self) -> Result<V, BinError>,
    {
        let count = self.read_map_count()?;
        let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(count), FxBuildHasher);
        for _ in 0..count {
            let k = self.read_i32_ctx(WireContext::MapColumn)?;
            let v = read_val(self)?;
            if map.insert(k, v).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::MapColumn,
                });
            }
        }
        Ok(map)
    }

    /// Reads a `<buckets: u32><count: u32><(Str, V)[count]>` map column using `read_val`.
    pub fn read_map_str_with<V, F>(
        &mut self,
        mut read_val: F,
    ) -> Result<FxHashMap<Box<str>, V>, BinError>
    where
        F: FnMut(&mut Self) -> Result<V, BinError>,
    {
        let count = self.read_map_count()?;
        let mut map = FxHashMap::with_capacity_and_hasher(u32_to_usize(count), FxBuildHasher);
        for _ in 0..count {
            let k = self.read_str_ctx(WireContext::MapColumn)?;
            let v = read_val(self)?;
            if map.insert(k, v).is_some() {
                return Err(BinError::DuplicateKey {
                    context: WireContext::MapColumn,
                });
            }
        }
        Ok(map)
    }
}

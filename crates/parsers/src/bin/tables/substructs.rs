//! Auto-generated compound `FConfig*` sub-structs synthesized from `.bin` schemas.
use crate::bin::affix::FConfigAffix;
use crate::bin::error::{BinError, WireContext};
use crate::bin::types::VecDim;
use crate::bin::wire::{RawI32F64, RawLinearColorF64, RawPairI32, WireCursor};
use crate::u32_to_usize;
/// Synthesized `FConfigI32I32` compound sub-struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FConfigI32I32 {
    pub f0: i32,
    pub f1: i32,
}
impl FConfigI32I32 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let raw = cur.take::<RawPairI32>(WireContext::SubStructColumn)?;
        Ok(Self {
            f0: raw.a.get(),
            f1: raw.b.get(),
        })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let slice = cur.take_slice_elems::<RawPairI32>(count, WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for item in slice {
            out.push(Self {
                f0: item.a.get(),
                f1: item.b.get(),
            });
        }
        Ok(out.into_boxed_slice())
    }
    /// Decodes a 2D length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array_2d(
        cur: &mut WireCursor<'_>,
    ) -> Result<Box<[Box<[Self]>]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read_array(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigI32ArrayStr` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FConfigI32ArrayStr {
    pub f1: Box<[Box<str>]>,
    pub f0: i32,
}
impl FConfigI32ArrayStr {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_array_str()?;
        Ok(Self { f1, f0 })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigI32F64` compound sub-struct.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FConfigI32F64 {
    pub f1: f64,
    pub f0: i32,
}
impl FConfigI32F64 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let raw = cur.take::<RawI32F64>(WireContext::SubStructColumn)?;
        Ok(Self {
            f1: raw.b.get(),
            f0: raw.a.get(),
        })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let slice = cur.take_slice_elems::<RawI32F64>(count, WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for item in slice {
            out.push(Self {
                f1: item.b.get(),
                f0: item.a.get(),
            });
        }
        Ok(out.into_boxed_slice())
    }
    /// Decodes a 2D length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array_2d(
        cur: &mut WireCursor<'_>,
    ) -> Result<Box<[Box<[Self]>]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read_array(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigStrArrayI32` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FConfigStrArrayI32 {
    pub f0: Box<str>,
    pub f1: Box<[i32]>,
}
impl FConfigStrArrayI32 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_str()?;
        let f1 = cur.read_array_i32()?;
        Ok(Self { f0, f1 })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigI32Str` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FConfigI32Str {
    pub f1: Box<str>,
    pub f0: i32,
}
impl FConfigI32Str {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_str()?;
        Ok(Self { f1, f0 })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigStrI32` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FConfigStrI32 {
    pub f0: Box<str>,
    pub f1: i32,
}
impl FConfigStrI32 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_str()?;
        let f1 = cur.read_i32()?;
        Ok(Self { f0, f1 })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigI32ArrayI32` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FConfigI32ArrayI32 {
    pub f1: Box<[i32]>,
    pub f0: i32,
}
impl FConfigI32ArrayI32 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_array_i32()?;
        Ok(Self { f1, f0 })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigF64x4` compound sub-struct.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FConfigF64x4 {
    pub f0: f64,
    pub f1: f64,
    pub f2: f64,
    pub f3: f64,
}
impl FConfigF64x4 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let raw = cur.take::<RawLinearColorF64>(WireContext::SubStructColumn)?;
        Ok(Self {
            f0: raw.r.get(),
            f1: raw.g.get(),
            f2: raw.b.get(),
            f3: raw.a.get(),
        })
    }
}
/// Synthesized `FConfigI32ArrayI32ArrayI32F64F64` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FConfigI32ArrayI32ArrayI32F64F64 {
    pub f1: Box<[i32]>,
    pub f2: Box<[i32]>,
    pub f3: f64,
    pub f4: f64,
    pub f0: i32,
}
impl FConfigI32ArrayI32ArrayI32F64F64 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_array_i32()?;
        let f2 = cur.read_array_i32()?;
        let f3 = cur.read_f64()?;
        let f4 = cur.read_f64()?;
        Ok(Self { f1, f2, f3, f4, f0 })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigMapMonsterPointPointInfo` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FConfigMapMonsterPointPointInfo {
    pub f5: f64,
    pub f6: f64,
    pub f7: Box<str>,
    pub f8: VecDim,
    pub f9: VecDim,
    pub f1: i32,
    pub f2: i32,
    pub f3: i32,
    pub f4: i32,
    pub f0: bool,
}
impl FConfigMapMonsterPointPointInfo {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_bool()?;
        let f1 = cur.read_i32()?;
        let f2 = cur.read_i32()?;
        let f3 = cur.read_i32()?;
        let f4 = cur.read_i32()?;
        let f5 = cur.read_f64()?;
        let f6 = cur.read_f64()?;
        let f7 = cur.read_str()?;
        let f8 = cur.read_vec_dim()?;
        let f9 = cur.read_vec_dim()?;
        Ok(Self {
            f5,
            f6,
            f7,
            f8,
            f9,
            f1,
            f2,
            f3,
            f4,
            f0,
        })
    }
}
/// Synthesized `FConfigI32ArrayI32F64x3` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FConfigI32ArrayI32F64x3 {
    pub f1: Box<[i32]>,
    pub f2: f64,
    pub f3: f64,
    pub f4: f64,
    pub f0: i32,
}
impl FConfigI32ArrayI32F64x3 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = cur.read_array_i32()?;
        let f2 = cur.read_f64()?;
        let f3 = cur.read_f64()?;
        let f4 = cur.read_f64()?;
        Ok(Self { f1, f2, f3, f4, f0 })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigI32Array2dI32F64` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FConfigI32Array2dI32F64 {
    pub f1: Box<[Box<[FConfigI32F64]>]>,
    pub f0: i32,
}
impl FConfigI32Array2dI32F64 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = FConfigI32F64::read_array_2d(cur)?;
        Ok(Self { f1, f0 })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigBoolArrayI32ArrayI32` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FConfigBoolArrayI32ArrayI32 {
    pub f1: Box<[i32]>,
    pub f2: Box<[i32]>,
    pub f0: bool,
}
impl FConfigBoolArrayI32ArrayI32 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_bool()?;
        let f1 = cur.read_array_i32()?;
        let f2 = cur.read_array_i32()?;
        Ok(Self { f1, f2, f0 })
    }
}
/// Synthesized `FConfigI32Array2dI32I32` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FConfigI32Array2dI32I32 {
    pub f1: Box<[Box<[FConfigI32I32]>]>,
    pub f0: i32,
}
impl FConfigI32Array2dI32I32 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = cur.read_i32()?;
        let f1 = FConfigI32I32::read_array_2d(cur)?;
        Ok(Self { f1, f0 })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}
/// Synthesized `FConfigAffixI32I32` compound sub-struct.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FConfigAffixI32I32 {
    pub f0: FConfigAffix,
    pub f1: i32,
    pub f2: i32,
}
impl FConfigAffixI32I32 {
    /// Decodes a single instance from `cur`.
    #[inline(never)]
    pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
        let f0 = FConfigAffix::read(cur)?;
        let f1 = cur.read_i32()?;
        let f2 = cur.read_i32()?;
        Ok(Self { f0, f1, f2 })
    }
    /// Decodes a length-prefixed array of `Self` from `cur`.
    #[inline(never)]
    pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
        let count = cur.read_collection_count(WireContext::ArrayColumn)?;
        let mut out = Vec::with_capacity(u32_to_usize(count));
        for _ in 0..count {
            out.push(Self::read(cur)?);
        }
        Ok(out.into_boxed_slice())
    }
}

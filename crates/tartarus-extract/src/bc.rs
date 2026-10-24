//! SIMD-accelerated decoders for Unreal Engine 4 texture formats (`PF_BC7`, `PF_DXT5`/`BC3`,
//! `PF_DXT1`/`BC1`, `PF_BC4`, `PF_BC5`, `PF_B8G8R8A8`, `PF_R8G8B8A8`, `PF_G8`) and
//! RGBA8 PNG codec.
//!
//! Why: Texture decoding converts millions of texels from BGRA32 or G8 into RGBA8. Processing
//! 16 pixels (64 bytes = 1 CPU cache line) per iteration via `std::simd` (`u8x64` swizzles and
//! `u32x16` channel splats) eliminates scalar per-pixel loop overhead.

use std::simd::prelude::*;
use std::simd::{simd_swizzle, u8x64};

use serde::Serialize;

use crate::error::ExtractError;

/// Maximum texture dimension (width or height) supported by the extractor.
pub const TEXTURE_DIM_MAX: usize = 65_536;

/// Maximum number of 16-pixel SIMD chunks in a single texture mip (`65536 * 65536 / 16`).
const SIMD_PIXEL_BATCHES_MAX: usize = 268_435_456;

const BGRA_TO_RGBA_SWIZZLE_64: [usize; 64] = [
    2, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15, 18, 17, 16, 19, 22, 21, 20, 23, 26, 25,
    24, 27, 30, 29, 28, 31, 34, 33, 32, 35, 38, 37, 36, 39, 42, 41, 40, 43, 46, 45, 44, 47, 50, 49,
    48, 51, 54, 53, 52, 55, 58, 57, 56, 59, 62, 61, 60, 63,
];

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PixelFormat {
    BC7 = 0,
    BC3 = 1, // DXT5
    BC1 = 2, // DXT1
    BC4 = 3,
    BC5 = 4,
    B8G8R8A8 = 5,
    R8G8B8A8 = 6,
    G8 = 7,
}

const _: () = {
    assert!(std::mem::size_of::<PixelFormat>() == 1);
    assert!(BGRA_TO_RGBA_SWIZZLE_64.len() == 64);
};

impl PixelFormat {
    /// Parses a UE4 pixel format identifier (`"PF_BC7"`, `"PF_DXT5"`, etc.).
    pub fn from_ue_name(name: &str) -> Option<Self> {
        assert!(name.len() <= 4096);
        let format = match name {
            "PF_BC7" => Some(Self::BC7),
            "PF_DXT5" => Some(Self::BC3),
            "PF_DXT1" => Some(Self::BC1),
            "PF_BC4" => Some(Self::BC4),
            "PF_BC5" => Some(Self::BC5),
            "PF_B8G8R8A8" => Some(Self::B8G8R8A8),
            "PF_R8G8B8A8" => Some(Self::R8G8B8A8),
            "PF_G8" => Some(Self::G8),
            _ => None,
        };
        assert!(format.is_none() || name.starts_with("PF_"));
        format
    }

    /// Alias for [`Self::from_ue_name`].
    pub fn from_name(name: &str) -> Option<Self> {
        assert!(name.len() <= 4096);
        let format = Self::from_ue_name(name);
        assert!(format.is_none() || name.starts_with("PF_"));
        format
    }

    /// Computes the exact byte size of a mip level with dimensions `width` × `height`.
    pub fn byte_size_for_dimensions(self, width: usize, height: usize) -> usize {
        assert!(width <= TEXTURE_DIM_MAX);
        assert!(height <= TEXTURE_DIM_MAX);
        let blocks_x = width.div_ceil(4);
        let blocks_y = height.div_ceil(4);
        match self {
            Self::BC7 | Self::BC3 | Self::BC5 => blocks_x * blocks_y * 16,
            Self::BC1 | Self::BC4 => blocks_x * blocks_y * 8,
            Self::B8G8R8A8 | Self::R8G8B8A8 => width * height * 4,
            Self::G8 => width * height,
        }
    }

    /// Alias for [`Self::byte_size_for_dimensions`].
    pub fn mip_byte_size(self, width: usize, height: usize) -> usize {
        assert!(width <= TEXTURE_DIM_MAX);
        assert!(height <= TEXTURE_DIM_MAX);
        self.byte_size_for_dimensions(width, height)
    }
}

/// Decodes a raw mipmap slice into RGBA8 (`width * height * 4` bytes).
pub fn decode_mip_to_rgba8(
    source: &[u8],
    width: usize,
    height: usize,
    format: PixelFormat,
) -> Result<Vec<u8>, ExtractError> {
    assert!(width <= TEXTURE_DIM_MAX);
    assert!(height <= TEXTURE_DIM_MAX);
    decode_texture_to_rgba8(format, width, height, source)
}

#[inline]
fn convert_bgra_u32_to_rgba8_simd(source: &[u32], target: &mut [u8]) {
    assert_eq!(target.len(), source.len() * 4);
    assert!(source.len() <= SIMD_PIXEL_BATCHES_MAX * 16);

    let (source_batches, source_tail) = source.as_chunks::<16>();
    let (target_batches, target_tail) = target.as_chunks_mut::<64>();
    let mut iterations = 0usize;

    for (src_16, dst_64) in source_batches.iter().zip(target_batches.iter_mut()) {
        iterations += 1;
        assert!(iterations <= SIMD_PIXEL_BATCHES_MAX);

        let v = u32x16::from_slice(src_16);
        let ga = v & u32x16::splat(0xFF00_FF00);
        let r = (v >> Simd::splat(16)) & u32x16::splat(0x0000_00FF);
        let b = (v & u32x16::splat(0x0000_00FF)) << Simd::splat(16);
        let rgba = ga | r | b;

        for (px_dst, &px_val) in dst_64
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(rgba.as_array())
        {
            *px_dst = px_val.to_le_bytes();
        }
    }

    for (px_dst, &px_val) in target_tail
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(source_tail)
    {
        let [b, g, r, a] = px_val.to_le_bytes();
        *px_dst = [r, g, b, a];
    }
}

#[inline]
fn convert_bc4_u32_to_rgba8_simd(source: &[u32], target: &mut [u8]) {
    assert_eq!(target.len(), source.len() * 4);
    assert!(source.len() <= SIMD_PIXEL_BATCHES_MAX * 16);

    let (source_batches, source_tail) = source.as_chunks::<16>();
    let (target_batches, target_tail) = target.as_chunks_mut::<64>();
    let mut iterations = 0usize;

    for (src_16, dst_64) in source_batches.iter().zip(target_batches.iter_mut()) {
        iterations += 1;
        assert!(iterations <= SIMD_PIXEL_BATCHES_MAX);

        let v = u32x16::from_slice(src_16);
        let r = (v >> Simd::splat(16)) & u32x16::splat(0x0000_00FF);
        let rgba = (r * u32x16::splat(0x0001_0101)) | u32x16::splat(0xFF00_0000);

        for (px_dst, &px_val) in dst_64
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(rgba.as_array())
        {
            *px_dst = px_val.to_le_bytes();
        }
    }

    for (px_dst, &px_val) in target_tail
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(source_tail)
    {
        let [_, _, r, _] = px_val.to_le_bytes();
        *px_dst = [r, r, r, 255];
    }
}

#[inline]
fn swizzle_bgra8_to_rgba8_simd(source: &[u8], target: &mut [u8]) {
    let byte_len = target.len();
    assert!(source.len() >= byte_len);
    assert!(byte_len.is_multiple_of(4));

    let source_bounded = &source[..byte_len];
    let (source_batches, source_tail) = source_bounded.as_chunks::<64>();
    let (target_batches, target_tail) = target.as_chunks_mut::<64>();
    let mut iterations = 0usize;

    for (src_64, dst_64) in source_batches.iter().zip(target_batches.iter_mut()) {
        iterations += 1;
        assert!(iterations <= SIMD_PIXEL_BATCHES_MAX);

        let vec_in = u8x64::from_slice(src_64);
        let vec_out = simd_swizzle!(vec_in, BGRA_TO_RGBA_SWIZZLE_64);
        vec_out.copy_to_slice(dst_64);
    }

    for (px_dst, px_src) in target_tail
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(source_tail.as_chunks::<4>().0)
    {
        *px_dst = [px_src[2], px_src[1], px_src[0], px_src[3]];
    }
}

#[inline]
fn expand_g8_to_rgba8_simd(source: &[u8], target: &mut [u8]) {
    let pixel_count = target.len() / 4;
    assert_eq!(target.len(), pixel_count * 4);
    assert!(source.len() >= pixel_count);

    let source_bounded = &source[..pixel_count];
    let (source_batches, source_tail) = source_bounded.as_chunks::<16>();
    let (target_batches, target_tail) = target.as_chunks_mut::<64>();
    let mut iterations = 0usize;

    for (src_16, dst_64) in source_batches.iter().zip(target_batches.iter_mut()) {
        iterations += 1;
        assert!(iterations <= SIMD_PIXEL_BATCHES_MAX);

        let g_u8 = u8x16::from_slice(src_16);
        let g_u32: u32x16 = g_u8.cast();
        let rgba = (g_u32 * u32x16::splat(0x0001_0101)) | u32x16::splat(0xFF00_0000);

        for (px_dst, &px_val) in dst_64
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(rgba.as_array())
        {
            *px_dst = px_val.to_le_bytes();
        }
    }

    for (px_dst, &g) in target_tail
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(source_tail)
    {
        *px_dst = [g, g, g, 255];
    }
}

fn decode_bc_blocks_to_rgba8(
    format: PixelFormat,
    width: usize,
    height: usize,
    source: &[u8],
    target: &mut [u8],
) -> Result<(), ExtractError> {
    assert_eq!(target.len(), width * height * 4);
    assert!(!source.is_empty() || width == 0 || height == 0);

    let mut bgra = vec![0u32; width * height];
    if format == PixelFormat::BC4 {
        texture2ddecoder::decode_bc4(source, width, height, &mut bgra)
            .map_err(|e| ExtractError::TextureDecode(e.to_string()))?;
        convert_bc4_u32_to_rgba8_simd(&bgra, target);
        return Ok(());
    }

    let res = match format {
        PixelFormat::BC1 => texture2ddecoder::decode_bc1a(source, width, height, &mut bgra),
        PixelFormat::BC3 => texture2ddecoder::decode_bc3(source, width, height, &mut bgra),
        PixelFormat::BC5 => texture2ddecoder::decode_bc5(source, width, height, &mut bgra),
        PixelFormat::BC7 => texture2ddecoder::decode_bc7(source, width, height, &mut bgra),
        _ => unreachable!(),
    };
    res.map_err(|e| ExtractError::TextureDecode(e.to_string()))?;
    convert_bgra_u32_to_rgba8_simd(&bgra, target);
    Ok(())
}

/// Decodes a texture buffer of the specified [`PixelFormat`] into an RGBA8 byte vector.
pub fn decode_texture_to_rgba8(
    format: PixelFormat,
    width: usize,
    height: usize,
    source: &[u8],
) -> Result<Vec<u8>, ExtractError> {
    assert!(width <= TEXTURE_DIM_MAX);
    assert!(height <= TEXTURE_DIM_MAX);

    let expected = format.byte_size_for_dimensions(width, height);
    if source.len() < expected {
        return Err(ExtractError::TextureDecode(format!(
            "Source buffer too small for {format:?} {width}x{height}: need {expected}B, got {}B",
            source.len()
        )));
    }

    let mut target = vec![0u8; width * height * 4];
    match format {
        PixelFormat::B8G8R8A8 => swizzle_bgra8_to_rgba8_simd(source, &mut target),
        PixelFormat::R8G8B8A8 => target.copy_from_slice(&source[..width * height * 4]),
        PixelFormat::G8 => expand_g8_to_rgba8_simd(source, &mut target),
        PixelFormat::BC1
        | PixelFormat::BC3
        | PixelFormat::BC4
        | PixelFormat::BC5
        | PixelFormat::BC7 => {
            decode_bc_blocks_to_rgba8(format, width, height, source, &mut target)?;
        }
    }
    assert_eq!(target.len(), width * height * 4);
    Ok(target)
}

/// Crops a sub-rectangle `(x, y, w, h)` from an RGBA8 buffer.
pub fn crop_rgba8(
    source: &[u8],
    src_w: usize,
    src_h: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) -> Vec<u8> {
    assert!(src_w <= TEXTURE_DIM_MAX);
    assert!(source.len() >= src_w * src_h * 4);

    let crop_w = w.min(src_w.saturating_sub(x));
    let crop_h = h.min(src_h.saturating_sub(y));
    let mut target = vec![0u8; crop_w * crop_h * 4];
    for row in 0..crop_h {
        let src_off = ((y + row) * src_w + x) * 4;
        let dst_off = row * crop_w * 4;
        target[dst_off..dst_off + crop_w * 4]
            .copy_from_slice(&source[src_off..src_off + crop_w * 4]);
    }
    assert_eq!(target.len(), crop_w * crop_h * 4);
    target
}

/// Encodes an RGBA8 buffer (`width * height * 4` bytes) into a lossless `.png` image.
pub fn encode_png_rgba8(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, ExtractError> {
    assert!(width as usize <= TEXTURE_DIM_MAX);
    assert!(height as usize <= TEXTURE_DIM_MAX);

    let expected = (width as usize) * (height as usize) * 4;
    if rgba.len() < expected {
        return Err(ExtractError::PngCodec(
            "RGBA buffer shorter than width * height * 4".to_string(),
        ));
    }
    let mut png_bytes = Vec::with_capacity(expected / 2 + 128);
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| ExtractError::PngCodec(e.to_string()))?;
        writer
            .write_image_data(&rgba[..expected])
            .map_err(|e| ExtractError::PngCodec(e.to_string()))?;
    }
    assert!(!png_bytes.is_empty());
    Ok(png_bytes)
}

/// Decodes an 8-bit RGB/RGBA/Grayscale `.png` image into `(width, height, rgba8)`.
pub fn decode_png_rgba8(png_bytes: &[u8]) -> Result<(usize, usize, Vec<u8>), ExtractError> {
    assert!(!png_bytes.is_empty());
    assert!(png_bytes.len() <= 256 * 1024 * 1024);

    let mut decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|e| ExtractError::PngCodec(e.to_string()))?;
    let buf_size = reader
        .output_buffer_size()
        .ok_or_else(|| ExtractError::PngCodec("PNG buffer size overflow".to_string()))?;
    let mut buffer = vec![0u8; buf_size];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|e| ExtractError::PngCodec(e.to_string()))?;
    buffer.truncate(info.buffer_size());

    let width = info.width as usize;
    let height = info.height as usize;
    let rgba = convert_png_frame_to_rgba8(info.color_type, width, height, buffer)?;
    assert_eq!(rgba.len(), width * height * 4);
    Ok((width, height, rgba))
}

fn convert_png_frame_to_rgba8(
    color_type: png::ColorType,
    width: usize,
    height: usize,
    buffer: Vec<u8>,
) -> Result<Vec<u8>, ExtractError> {
    assert!(width <= TEXTURE_DIM_MAX);
    assert!(height <= TEXTURE_DIM_MAX);

    let pixel_count = width * height;
    match color_type {
        png::ColorType::Rgba => Ok(buffer),
        png::ColorType::Rgb => {
            let mut target = Vec::with_capacity(pixel_count * 4);
            for &[r, g, b] in buffer.as_chunks::<3>().0 {
                target.extend_from_slice(&[r, g, b, 255]);
            }
            Ok(target)
        }
        png::ColorType::Grayscale => {
            let mut target = vec![0u8; pixel_count * 4];
            expand_g8_to_rgba8_simd(&buffer, &mut target);
            Ok(target)
        }
        png::ColorType::GrayscaleAlpha => {
            let mut target = Vec::with_capacity(pixel_count * 4);
            for &[g, a] in buffer.as_chunks::<2>().0 {
                target.extend_from_slice(&[g, g, g, a]);
            }
            Ok(target)
        }
        other => Err(ExtractError::PngCodec(format!(
            "Unsupported PNG color type: {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_bgra8_and_g8_conversions() {
        // 19 pixels exercises one 16-pixel SIMD vector + 3 tail pixels.
        let mut bgra = Vec::with_capacity(19 * 4);
        let mut g8 = Vec::with_capacity(19);
        for i in 0..19u8 {
            bgra.extend_from_slice(&[i + 1, i + 2, i + 3, i + 4]);
            g8.push(i * 10);
        }

        let rgba_from_bgra =
            decode_texture_to_rgba8(PixelFormat::B8G8R8A8, 19, 1, &bgra).expect("BGRA8");
        let rgba_from_g8 = decode_texture_to_rgba8(PixelFormat::G8, 19, 1, &g8).expect("G8");

        for i in 0..19usize {
            let b_px = &rgba_from_bgra[i * 4..i * 4 + 4];
            assert_eq!(
                b_px,
                &[(i as u8) + 3, (i as u8) + 2, (i as u8) + 1, (i as u8) + 4]
            );

            let g_px = &rgba_from_g8[i * 4..i * 4 + 4];
            let v = (i as u8) * 10;
            assert_eq!(g_px, &[v, v, v, 255]);
        }
    }
}

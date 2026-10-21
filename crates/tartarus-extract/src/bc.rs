//! Pure-Rust decoders for Unreal Engine 4 texture formats (`PF_BC7`, `PF_DXT5`/`BC3`,
//! `PF_DXT1`/`BC1`, `PF_BC4`, `PF_BC5`, `PF_B8G8R8A8`, `PF_R8G8B8A8`, `PF_G8`) and RGBA8 PNG encoder.

use serde::Serialize;

use crate::error::ExtractError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PixelFormat {
    BC7,
    BC3, // DXT5
    BC1, // DXT1
    BC4,
    BC5,
    B8G8R8A8,
    R8G8B8A8,
    G8,
}

impl PixelFormat {
    pub fn from_ue_name(s: &str) -> Option<Self> {
        match s {
            "PF_BC7" => Some(Self::BC7),
            "PF_DXT5" => Some(Self::BC3),
            "PF_DXT1" => Some(Self::BC1),
            "PF_BC4" => Some(Self::BC4),
            "PF_BC5" => Some(Self::BC5),
            "PF_B8G8R8A8" => Some(Self::B8G8R8A8),
            "PF_R8G8B8A8" => Some(Self::R8G8B8A8),
            "PF_G8" => Some(Self::G8),
            _ => None,
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        Self::from_ue_name(s)
    }

    pub fn byte_size_for_dimensions(self, width: usize, height: usize) -> usize {
        let bx = width.div_ceil(4);
        let by = height.div_ceil(4);
        match self {
            Self::BC7 | Self::BC3 | Self::BC5 => bx * by * 16,
            Self::BC1 | Self::BC4 => bx * by * 8,
            Self::B8G8R8A8 | Self::R8G8B8A8 => width * height * 4,
            Self::G8 => width * height,
        }
    }

    pub fn mip_byte_size(self, width: usize, height: usize) -> usize {
        self.byte_size_for_dimensions(width, height)
    }
}

pub fn decode_mip_to_rgba8(
    src: &[u8],
    width: usize,
    height: usize,
    fmt: PixelFormat,
) -> Result<Vec<u8>, ExtractError> {
    decode_texture_to_rgba8(fmt, width, height, src)
}

pub fn decode_texture_to_rgba8(
    fmt: PixelFormat,
    width: usize,
    height: usize,
    src: &[u8],
) -> Result<Vec<u8>, ExtractError> {
    let expected = fmt.byte_size_for_dimensions(width, height);
    if src.len() < expected {
        return Err(ExtractError::TextureDecode(format!(
            "Source buffer too small for {fmt:?} {width}x{height}: need {expected} bytes, got {}",
            src.len()
        )));
    }
    let mut out = vec![0u8; width * height * 4];
    match fmt {
        PixelFormat::B8G8R8A8 => {
            for (dst, px) in out
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(src.as_chunks::<4>().0)
            {
                dst[0] = px[2];
                dst[1] = px[1];
                dst[2] = px[0];
                dst[3] = px[3];
            }
        }
        PixelFormat::R8G8B8A8 => {
            out.copy_from_slice(&src[..width * height * 4]);
        }
        PixelFormat::G8 => {
            for (dst, &g) in out
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(src.iter().take(width * height))
            {
                dst[0] = g;
                dst[1] = g;
                dst[2] = g;
                dst[3] = 255;
            }
        }
        PixelFormat::BC1 | PixelFormat::BC3 | PixelFormat::BC5 | PixelFormat::BC7 => {
            let mut bgra = vec![0u32; width * height];
            let res = match fmt {
                PixelFormat::BC1 => texture2ddecoder::decode_bc1a(src, width, height, &mut bgra),
                PixelFormat::BC3 => texture2ddecoder::decode_bc3(src, width, height, &mut bgra),
                PixelFormat::BC5 => texture2ddecoder::decode_bc5(src, width, height, &mut bgra),
                PixelFormat::BC7 => texture2ddecoder::decode_bc7(src, width, height, &mut bgra),
                _ => unreachable!(),
            };
            res.map_err(|e| ExtractError::TextureDecode(e.to_string()))?;
            for (dst, &px) in out.as_chunks_mut::<4>().0.iter_mut().zip(bgra.iter()) {
                let [b, g, r, a] = px.to_le_bytes();
                *dst = [r, g, b, a];
            }
        }
        PixelFormat::BC4 => {
            let mut bgra = vec![0u32; width * height];
            texture2ddecoder::decode_bc4(src, width, height, &mut bgra)
                .map_err(|e| ExtractError::TextureDecode(e.to_string()))?;
            for (dst, &px) in out.as_chunks_mut::<4>().0.iter_mut().zip(bgra.iter()) {
                let [_, _, r, _] = px.to_le_bytes();
                *dst = [r, r, r, 255];
            }
        }
    }
    Ok(out)
}

/// Crops a sub-rectangle `(x, y, w, h)` from an RGBA8 buffer.
pub fn crop_rgba8(
    src: &[u8],
    src_w: usize,
    src_h: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) -> Vec<u8> {
    let cw = w.min(src_w.saturating_sub(x));
    let ch = h.min(src_h.saturating_sub(y));
    let mut out = vec![0u8; cw * ch * 4];
    for row in 0..ch {
        let s_off = ((y + row) * src_w + x) * 4;
        let d_off = row * cw * 4;
        out[d_off..d_off + cw * 4].copy_from_slice(&src[s_off..s_off + cw * 4]);
    }
    out
}

/// Encodes an RGBA8 buffer (`width * height * 4` bytes) into a standard lossless `.png` file.
pub fn encode_png_rgba8(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, ExtractError> {
    let expected = (width as usize) * (height as usize) * 4;
    if rgba.len() < expected {
        return Err(ExtractError::PngCodec(
            "RGBA buffer shorter than width * height * 4".to_string(),
        ));
    }
    let mut png = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| ExtractError::PngCodec(e.to_string()))?;
        writer
            .write_image_data(&rgba[..expected])
            .map_err(|e| ExtractError::PngCodec(e.to_string()))?;
    }
    Ok(png)
}

/// Decodes a standard 8-bit RGB or RGBA `.png` file into `(width, height, rgba8)`.
pub fn decode_png_rgba8(png: &[u8]) -> Result<(usize, usize, Vec<u8>), ExtractError> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(png));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|e| ExtractError::PngCodec(e.to_string()))?;
    let mut buf =
        vec![
            0u8;
            reader
                .output_buffer_size()
                .ok_or_else(|| ExtractError::PngCodec("PNG buffer size overflow".to_string()))?
        ];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| ExtractError::PngCodec(e.to_string()))?;
    buf.truncate(info.buffer_size());

    let w = info.width as usize;
    let h = info.height as usize;
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity(w * h * 4);
            for &[r, g, b] in buf.as_chunks::<3>().0 {
                out.extend_from_slice(&[r, g, b, 255]);
            }
            out
        }
        png::ColorType::Grayscale => {
            let mut out = Vec::with_capacity(w * h * 4);
            for &g in &buf {
                out.extend_from_slice(&[g, g, g, 255]);
            }
            out
        }
        png::ColorType::GrayscaleAlpha => {
            let mut out = Vec::with_capacity(w * h * 4);
            for &[g, a] in buf.as_chunks::<2>().0 {
                out.extend_from_slice(&[g, g, g, a]);
            }
            out
        }
        other => {
            return Err(ExtractError::PngCodec(format!(
                "Unsupported PNG color type: {other:?}"
            )));
        }
    };
    Ok((w, h, rgba))
}

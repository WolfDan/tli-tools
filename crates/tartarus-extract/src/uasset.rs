//! Cooked Unreal Engine 4.27 `.uasset` / `.uexp` / `.ubulk` parser for `UTexture2D` and `UPaperSprite`.

use serde::Serialize;

use crate::bc::{PixelFormat, decode_mip_to_rgba8, encode_png_rgba8};

pub const PACKAGE_FILE_TAG: u32 = 0x9E2A_83C1;

#[derive(Clone, Debug, Serialize)]
pub struct PaperSpriteInfo {
    pub atlas_virtual_path: String,
    pub uv_x: u32,
    pub uv_y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DecodedTexture {
    pub width: usize,
    pub height: usize,
    pub format: PixelFormat,
    pub num_mips: usize,
    pub names: Vec<String>,
    #[serde(skip)]
    pub rgba8: Vec<u8>,
}

impl DecodedTexture {
    pub fn to_png_bytes(&self) -> Vec<u8> {
        encode_png_rgba8(self.width as u32, self.height as u32, &self.rgba8).unwrap_or_default()
    }
}

/// Parses a cooked Paper2D `UPaperSprite` (`.uasset` + `.uexp`) to extract its parent atlas path
/// and pixel crop rectangle (`BakedSourceUV`, `BakedSourceDimension`).
pub fn parse_paper_sprite(uasset: &[u8], uexp: &[u8]) -> Option<PaperSpriteInfo> {
    let names = parse_uasset_names(uasset);
    let uv_idx = names.iter().position(|n| n == "BakedSourceUV")? as i32;
    let dim_idx = names.iter().position(|n| n == "BakedSourceDimension")? as i32;
    let struct_idx = names.iter().position(|n| n == "StructProperty")? as i32;
    let vec2d_idx = names.iter().position(|n| n == "Vector2D")? as i32;

    let atlas_game_path = names
        .iter()
        .find(|n| n.starts_with("/Game/") && !n.ends_with("_Sprite"))?;
    let atlas_virtual_path = atlas_game_path.replacen("/Game/", "UE_game/Content/", 1);

    let find_vec2d_prop = |prop_name_idx: i32| -> Option<(f32, f32)> {
        let mut pat = [0u8; 32];
        pat[0..4].copy_from_slice(&prop_name_idx.to_le_bytes());
        pat[8..12].copy_from_slice(&struct_idx.to_le_bytes());
        pat[16..20].copy_from_slice(&8i32.to_le_bytes());
        pat[24..28].copy_from_slice(&vec2d_idx.to_le_bytes());

        for i in 0..uexp.len().saturating_sub(57) {
            if uexp[i..i + 32] == pat {
                let val_off = i + 32 + 17; // 16-byte StructGuid + 1-byte PropertyGuid flag
                let x = f32::from_le_bytes(uexp[val_off..val_off + 4].try_into().ok()?);
                let y = f32::from_le_bytes(uexp[val_off + 4..val_off + 8].try_into().ok()?);
                return Some((x, y));
            }
        }
        None
    };

    let (uv_x, uv_y) = find_vec2d_prop(uv_idx).unwrap_or((0.0, 0.0));
    let (w, h) = find_vec2d_prop(dim_idx)?;
    if w <= 0.0 || h <= 0.0 {
        return None;
    }

    Some(PaperSpriteInfo {
        atlas_virtual_path,
        uv_x: uv_x.round().max(0.0) as u32,
        uv_y: uv_y.round().max(0.0) as u32,
        width: w.round() as u32,
        height: h.round() as u32,
    })
}

fn read_u32_le(buf: &[u8], off: usize) -> Option<u32> {
    let b: [u8; 4] = buf.get(off..off + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(b))
}

fn read_i32_le(buf: &[u8], off: usize) -> Option<i32> {
    let b: [u8; 4] = buf.get(off..off + 4)?.try_into().ok()?;
    Some(i32::from_le_bytes(b))
}

fn read_i64_le(buf: &[u8], off: usize) -> Option<i64> {
    let b: [u8; 8] = buf.get(off..off + 8)?.try_into().ok()?;
    Some(i64::from_le_bytes(b))
}

fn read_fstring(buf: &[u8], off: usize) -> Option<(String, usize)> {
    let len = read_i32_le(buf, off)?;
    let start = off + 4;
    if len == 0 {
        return Some((String::new(), start));
    }
    if len < 0 {
        let chars = (-len) as usize;
        if chars > 4096 {
            return None;
        }
        let byte_len = chars * 2;
        let slice = buf.get(start..start + byte_len)?;
        let u16s: Vec<u16> = slice
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let s = String::from_utf16_lossy(&u16s)
            .trim_end_matches('\0')
            .to_string();
        Some((s, start + byte_len))
    } else {
        let byte_len = len as usize;
        if byte_len > 4096 {
            return None;
        }
        let slice = buf.get(start..start + byte_len)?;
        let s = String::from_utf8_lossy(slice)
            .trim_end_matches('\0')
            .to_string();
        Some((s, start + byte_len))
    }
}

/// Parses the `FName` string table out of a cooked UE 4.27 `.uasset` header.
pub fn parse_uasset_names(uasset: &[u8]) -> Vec<String> {
    let Some(tag) = read_u32_le(uasset, 0) else {
        return Vec::new();
    };
    if tag != PACKAGE_FILE_TAG {
        return Vec::new();
    }

    // Try standard UE4 FPackageFileSummary layout and fallback scan for (name_count, name_offset)
    let mut candidates = Vec::new();
    if let Some(legacy_ver) = read_i32_le(uasset, 4) {
        let mut off = 8usize;
        if legacy_ver != -4 {
            off += 4; // LegacyUE3Version
        }
        off += 4 + 4 + 4; // FileVersionUE4, FileVersionLicenseeUE4, CustomVersionCount
        if let Some(custom_count) = read_i32_le(uasset, off - 4)
            && (0..=256).contains(&custom_count)
        {
            off += (custom_count as usize) * 20; // FCustomVersion = GUID(16) + ver(4)
            off += 4; // TotalHeaderSize
            if let Some((_, next_off)) = read_fstring(uasset, off) {
                off = next_off;
                off += 4; // PackageFlags
                if let (Some(nc), Some(no)) = (read_i32_le(uasset, off), read_i32_le(uasset, off + 4))
                    && nc > 0
                    && no > 0
                {
                    candidates.push((nc as usize, no as usize));
                }
            }
        }
    }

    for scan_off in (24..uasset.len().min(256)).step_by(4) {
        if let (Some(nc), Some(no)) = (
            read_i32_le(uasset, scan_off),
            read_i32_le(uasset, scan_off + 4),
        ) && (1..=4096).contains(&nc)
            && (64..=uasset.len() as i32).contains(&no)
        {
            candidates.push((nc as usize, no as usize));
        }
    }

    for (name_count, name_offset) in candidates {
        let mut names = Vec::with_capacity(name_count);
        let mut cur = name_offset;
        let mut ok = true;
        for _ in 0..name_count {
            if let Some((s, next)) = read_fstring(uasset, cur)
                && !s.is_empty()
                && next + 4 <= uasset.len()
            {
                names.push(s);
                cur = next + 4; // NonCasePreservingHash(u16) + CasePreservingHash(u16)
            } else {
                ok = false;
                break;
            }
        }
        if ok && !names.is_empty() {
            return names;
        }
    }

    Vec::new()
}

/// Decodes a cooked UE 4.27 `UTexture2D` from its `.uasset`, `.uexp`, and optional `.ubulk` buffers.
pub fn decode_texture2d(
    uasset: &[u8],
    uexp: &[u8],
    ubulk: Option<&[u8]>,
) -> Result<DecodedTexture, String> {
    let names = parse_uasset_names(uasset);

    // 1. Primary path: locate `FTexturePlatformData`'s `PixelFormatString` (`"PF_..."` FString) inside `.uexp`.
    if let Some(tex) = try_parse_platform_data(uexp, ubulk, &names) {
        return Ok(tex);
    }

    // 2. Fallback path: determine PixelFormat from `.uasset` FNames (or ASCII scan) and scan `.uexp` for FByteBulkData mip headers.
    let format = names
        .iter()
        .find_map(|n| PixelFormat::from_name(n))
        .or_else(|| find_ascii_pixel_format(uexp))
        .or_else(|| find_ascii_pixel_format(uasset))
        .ok_or_else(|| "Could not determine texture PixelFormat (no PF_* found)".to_string())?;

    try_scan_mip_in_uexp(uexp, ubulk, format, &names)
}

fn find_ascii_pixel_format(buf: &[u8]) -> Option<PixelFormat> {
    for i in 0..buf.len().saturating_sub(6) {
        if &buf[i..i + 3] == b"PF_" {
            let end = buf[i.. (i + 24).min(buf.len())]
                .iter()
                .position(|&b| b == 0 || !b.is_ascii_graphic())
                .unwrap_or(0);
            if end > 3 {
                let s = std::str::from_utf8(&buf[i..i + end]).ok()?;
                if let Some(pf) = PixelFormat::from_name(s) {
                    return Some(pf);
                }
            }
        }
    }
    None
}

fn try_parse_platform_data(
    uexp: &[u8],
    ubulk: Option<&[u8]>,
    names: &[String],
) -> Option<DecodedTexture> {
    for i in 16..uexp.len().saturating_sub(16) {
        if &uexp[i..i + 3] != b"PF_" {
            continue;
        }
        let str_len = read_i32_le(uexp, i - 4)?;
        if !(4..=32).contains(&str_len) {
            continue;
        }
        let str_end = i + (str_len as usize);
        if str_end > uexp.len() || uexp[str_end - 1] != 0 {
            continue;
        }
        let pf_str = std::str::from_utf8(&uexp[i..str_end - 1]).ok()?;
        let format = PixelFormat::from_name(pf_str)?;

        let plat_size_x = read_i32_le(uexp, i - 16)?;
        let plat_size_y = read_i32_le(uexp, i - 12)?;
        if !(1..=16384).contains(&plat_size_x) || !(1..=16384).contains(&plat_size_y) {
            continue;
        }

        // After PixelFormatString:
        // FirstMipToSerialize (i32), NumMips (i32)
        // Note: Some UE4 builds have an extra SkipOffset (i64 or i32) before FirstMipToSerialize; check both offsets!
        for header_skip in [0usize, 4usize, 8usize] {
            let mip_table_off = str_end + header_skip;
            let Some(_first_mip) = read_i32_le(uexp, mip_table_off) else {
                continue;
            };
            let Some(num_mips) = read_i32_le(uexp, mip_table_off + 4) else {
                continue;
            };
            if !(1..=16).contains(&num_mips) {
                continue;
            }
            let mip0_off = mip_table_off + 8;
            if let Some((width, height, mip_bytes)) =
                parse_single_mip(uexp, ubulk, mip0_off, format, plat_size_x as usize, plat_size_y as usize)
                && let Ok(rgba8) = decode_mip_to_rgba8(mip_bytes, width, height, format)
            {
                return Some(DecodedTexture {
                    width,
                    height,
                    format,
                    num_mips: num_mips as usize,
                    names: names.to_vec(),
                    rgba8,
                });
            }
        }
    }
    None
}

fn parse_single_mip<'a>(
    uexp: &'a [u8],
    ubulk: Option<&'a [u8]>,
    mip_off: usize,
    format: PixelFormat,
    expected_w: usize,
    expected_h: usize,
) -> Option<(usize, usize, &'a [u8])> {
    let b_cooked = read_i32_le(uexp, mip_off)?;
    if b_cooked != 1 {
        return None;
    }
    let bulk_flags = read_u32_le(uexp, mip_off + 4)?;
    let is_64bit = (bulk_flags & 0x2000) != 0;

    let (elem_count, size_on_disk, after_bulk_hdr) = if is_64bit {
        let ec = read_i64_le(uexp, mip_off + 8)? as usize;
        let sod = read_i64_le(uexp, mip_off + 16)? as usize;
        let _offset_in_file = read_i64_le(uexp, mip_off + 24)?;
        (ec, sod, mip_off + 32)
    } else {
        let ec = read_i32_le(uexp, mip_off + 8)? as usize;
        let sod = read_i32_le(uexp, mip_off + 12)? as usize;
        let _offset_in_file = read_i64_le(uexp, mip_off + 16)?;
        (ec, sod, mip_off + 24)
    };

    let is_inline = (bulk_flags & 0x01) == 0 && (bulk_flags & 0x100) == 0;
    if is_inline && after_bulk_hdr + elem_count + 12 <= uexp.len() {
        let sx = read_i32_le(uexp, after_bulk_hdr + elem_count)? as usize;
        let sy = read_i32_le(uexp, after_bulk_hdr + elem_count + 4)? as usize;
        let sz = read_i32_le(uexp, after_bulk_hdr + elem_count + 8).unwrap_or(1);
        if sx > 0
            && sy > 0
            && (1..=64).contains(&sz)
            && format.mip_byte_size(sx, sy) == elem_count
        {
            return Some((sx, sy, &uexp[after_bulk_hdr..after_bulk_hdr + elem_count]));
        }
    }

    // External `.ubulk` payload: SizeX, SizeY, SizeZ immediately follow the BulkData header in `.uexp`
    if let Some(ubulk_bytes) = ubulk
        && after_bulk_hdr + 12 <= uexp.len()
    {
        let sx = read_i32_le(uexp, after_bulk_hdr)? as usize;
        let sy = read_i32_le(uexp, after_bulk_hdr + 4)? as usize;
        let width = if sx > 0 { sx } else { expected_w };
        let height = if sy > 0 { sy } else { expected_h };
        let needed = format.mip_byte_size(width, height).max(size_on_disk);
        if ubulk_bytes.len() >= needed && needed > 0 {
            return Some((width, height, &ubulk_bytes[..needed]));
        }
    }

    None
}

fn try_scan_mip_in_uexp(
    uexp: &[u8],
    ubulk: Option<&[u8]>,
    format: PixelFormat,
    names: &[String],
) -> Result<DecodedTexture, String> {
    for off in 0..uexp.len().saturating_sub(36) {
        if read_i32_le(uexp, off) == Some(1)
            && let Some((w, h, mip_bytes)) = parse_single_mip(uexp, ubulk, off, format, 0, 0)
            && let Ok(rgba8) = decode_mip_to_rgba8(mip_bytes, w, h, format)
        {
            return Ok(DecodedTexture {
                width: w,
                height: h,
                format,
                num_mips: 1,
                names: names.to_vec(),
                rgba8,
            });
        }
    }

    Err(format!(
        "Failed to locate valid {format:?} mipmap payload inside .uexp ({} bytes)",
        uexp.len()
    ))
}

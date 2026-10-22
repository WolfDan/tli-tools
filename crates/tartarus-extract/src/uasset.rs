//! Cooked Unreal Engine 4.27 `.uasset` / `.uexp` / `.ubulk` parser for `UTexture2D`
//! and `UPaperSprite`.

use std::io::Cursor;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use serde::Serialize;

use crate::bc::{PixelFormat, decode_mip_to_rgba8, encode_png_rgba8};
use crate::error::ExtractError;

pub const PACKAGE_FILE_TAG: u32 = 0x9E2A_83C1;

const _: () = {
    assert!(PACKAGE_FILE_TAG == 0x9E2A_83C1);
    assert!(std::mem::size_of::<PaperSpriteInfo>() >= 16);
};

#[derive(Clone, Debug, Serialize)]
pub struct PaperSpriteInfo {
    pub atlas_virtual_path: String,
    pub uv_x: u32,
    pub uv_y: u32,
    pub width: u32,
    pub height: u32,
}

/// Entry from a cooked `.uasset` `FObjectImport` table (`28` bytes per entry).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UassetImportEntry {
    pub class_package: String,
    pub class_name: String,
    pub outer_index: i32,
    pub object_name: String,
    pub package_path: String,
}

/// Entry from a cooked `.uasset` `FObjectExport` table (`104` bytes per entry in UE 4.27).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UassetExportEntry {
    pub export_index: usize,
    pub class_index: i32,
    pub class_name: String,
    pub super_index: i32,
    pub template_index: i32,
    pub outer_index: i32,
    pub object_name: String,
    pub serial_size: usize,
    pub serial_offset: usize,
    pub uexp_offset: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct DecodedTexture {
    pub width: usize,
    pub height: usize,
    pub format: PixelFormat,
    pub num_mips: usize,
    pub srgb: bool,
    pub clamp_x: bool,
    pub clamp_y: bool,
    pub names: Vec<String>,
    #[serde(skip)]
    pub rgba8: Vec<u8>,
}

impl DecodedTexture {
    pub fn to_png_bytes(&self) -> Vec<u8> {
        assert!(self.width <= 65536);
        assert!(self.height <= 65536);
        encode_png_rgba8(self.width as u32, self.height as u32, &self.rgba8).unwrap_or_default()
    }
}

/// Parses a cooked Paper2D `UPaperSprite` (`.uasset` + `.uexp`) to extract its parent atlas path
/// and pixel crop rectangle (`BakedSourceUV`, `BakedSourceDimension`).
pub fn parse_paper_sprite(uasset: &[u8], uexp: &[u8]) -> Option<PaperSpriteInfo> {
    assert!(uasset.len() <= 256 * 1024 * 1024);
    assert!(uexp.len() <= 256 * 1024 * 1024);
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
        LittleEndian::write_i32(&mut pat[0..4], prop_name_idx);
        LittleEndian::write_i32(&mut pat[8..12], struct_idx);
        LittleEndian::write_i32(&mut pat[16..20], 8);
        LittleEndian::write_i32(&mut pat[24..28], vec2d_idx);

        let i = memchr::memmem::find(uexp, &pat)?;
        let val_off = i + 32 + 17; // 16-byte StructGuid + 1-byte PropertyGuid flag
        if val_off + 8 > uexp.len() {
            return None;
        }
        let x = LittleEndian::read_f32(&uexp[val_off..val_off + 4]);
        let y = LittleEndian::read_f32(&uexp[val_off + 4..val_off + 8]);
        Some((x, y))
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

fn read_fstring(cur: &mut Cursor<&[u8]>) -> Option<String> {
    assert!((cur.position() as usize) <= cur.get_ref().len());
    let len = cur.read_i32::<LittleEndian>().ok()?;
    if len == 0 {
        return Some(String::new());
    }
    let start = cur.position() as usize;
    let buf = *cur.get_ref();
    if len < 0 {
        let chars = (-len) as usize;
        if chars > 4096 {
            return None;
        }
        let byte_len = chars * 2;
        let slice = buf.get(start..start + byte_len)?;
        cur.set_position((start + byte_len) as u64);
        let mut u16s = vec![0u16; chars];
        LittleEndian::read_u16_into(slice, &mut u16s);
        assert!((cur.position() as usize) <= buf.len());
        Some(
            String::from_utf16_lossy(&u16s)
                .trim_end_matches('\0')
                .to_string(),
        )
    } else {
        let byte_len = len as usize;
        if byte_len > 4096 {
            return None;
        }
        let slice = buf.get(start..start + byte_len)?;
        cur.set_position((start + byte_len) as u64);
        assert!((cur.position() as usize) <= buf.len());
        Some(
            String::from_utf8_lossy(slice)
                .trim_end_matches('\0')
                .to_string(),
        )
    }
}

fn collect_name_table_candidates(uasset: &[u8]) -> Vec<(usize, usize)> {
    assert!(uasset.len() >= 8);
    let mut candidates = Vec::new();
    let legacy_ver = LittleEndian::read_i32(&uasset[4..]);
    let mut off = 8usize;
    if legacy_ver != -4 {
        off += 4;
    }
    off += 4 + 4 + 4;
    if let Some(cc_slice) = uasset.get(off - 4..off) {
        let custom_count = LittleEndian::read_i32(cc_slice);
        if (0..=256).contains(&custom_count) {
            off += (custom_count as usize) * 20 + 4;
            let mut cur = Cursor::new(uasset);
            cur.set_position(off as u64);
            if read_fstring(&mut cur).is_some() {
                cur.set_position(cur.position() + 4);
                if let (Ok(nc), Ok(no)) = (
                    cur.read_i32::<LittleEndian>(),
                    cur.read_i32::<LittleEndian>(),
                ) && nc > 0
                    && no > 0
                {
                    candidates.push((nc as usize, no as usize));
                }
            }
        }
    }

    for scan_off in (24..uasset.len().min(256)).step_by(4) {
        if let Some(slice) = uasset.get(scan_off..scan_off + 8) {
            let nc = LittleEndian::read_i32(&slice[0..4]);
            let no = LittleEndian::read_i32(&slice[4..8]);
            if (1..=4096).contains(&nc) && (64..=uasset.len() as i32).contains(&no) {
                candidates.push((nc as usize, no as usize));
            }
        }
    }
    assert!(candidates.len() <= 128);
    candidates
}

/// Parses the `FName` string table out of a cooked UE 4.27 `.uasset` header.
pub fn parse_uasset_names(uasset: &[u8]) -> Vec<String> {
    assert!(uasset.len() <= 256 * 1024 * 1024);
    if uasset.len() < 8 || LittleEndian::read_u32(uasset) != PACKAGE_FILE_TAG {
        return Vec::new();
    }

    let candidates = collect_name_table_candidates(uasset);
    for (name_count, name_offset) in candidates {
        let mut names = Vec::with_capacity(name_count);
        let mut cur = Cursor::new(uasset);
        cur.set_position(name_offset as u64);
        let mut ok = true;
        for _ in 0..name_count {
            if let Some(s) = read_fstring(&mut cur)
                && !s.is_empty()
                && (cur.position() as usize) + 4 <= uasset.len()
            {
                names.push(s);
                // NonCasePreservingHash(u16) + CasePreservingHash(u16)
                cur.set_position(cur.position() + 4);
            } else {
                ok = false;
                break;
            }
        }
        if ok && !names.is_empty() {
            assert_eq!(names.len(), name_count);
            return names;
        }
    }

    Vec::new()
}

/// Formats an `(index, number)` `FName` pair using UE4's `number - 1` suffix rule.
pub fn format_fname_idx(names: &[String], idx: u32, number: u32) -> String {
    assert!(!names.is_empty());
    assert!(names.len() <= 65_536);
    let base = names
        .get(idx as usize)
        .map(String::as_str)
        .unwrap_or("None");
    if number == 0 {
        base.to_string()
    } else {
        format!("{base}_{}", number - 1)
    }
}

fn find_table_bounds(uasset: &[u8], names_len: usize) -> Option<(usize, usize, usize, usize)> {
    assert!(uasset.len() >= 8);
    assert!(names_len <= 65_536);
    for scan_off in (24..uasset.len().min(256)).step_by(1) {
        let Some(hdr) = uasset.get(scan_off..scan_off + 32) else {
            break;
        };
        let nc = LittleEndian::read_i32(&hdr[0..4]);
        let no = LittleEndian::read_i32(&hdr[4..8]);
        let ec = LittleEndian::read_i32(&hdr[16..20]);
        let eo = LittleEndian::read_i32(&hdr[20..24]);
        let ic = LittleEndian::read_i32(&hdr[24..28]);
        let io = LittleEndian::read_i32(&hdr[28..32]);
        if nc as usize == names_len
            && (64..=uasset.len() as i32).contains(&no)
            && (1..=65_536).contains(&ec)
            && (64..=uasset.len() as i32).contains(&eo)
            && (1..=65_536).contains(&ic)
            && (64..=uasset.len() as i32).contains(&io)
            && (io as usize) + (ic as usize) * 28 <= uasset.len()
        {
            return Some((ec as usize, eo as usize, ic as usize, io as usize));
        }
    }
    None
}

/// Parses the `FObjectImport` table from a cooked UE 4.27 `.uasset` header and resolves
/// each import's outer package path.
pub fn parse_uasset_imports(uasset: &[u8], names: &[String]) -> Vec<UassetImportEntry> {
    assert!(uasset.len() <= 256 * 1024 * 1024);
    assert!(names.len() <= 65_536);
    if uasset.len() < 64 || names.is_empty() {
        return Vec::new();
    }
    let Some((_, _, import_count, import_offset)) = find_table_bounds(uasset, names.len()) else {
        return Vec::new();
    };
    assert!(import_count <= 65_536);

    let mut raw_imports = Vec::with_capacity(import_count);
    for i in 0..import_count {
        let off = import_offset + i * 28;
        let Some(slice) = uasset.get(off..off + 28) else {
            return Vec::new();
        };
        let cp_idx = LittleEndian::read_u32(&slice[0..4]);
        let cp_num = LittleEndian::read_u32(&slice[4..8]);
        let cn_idx = LittleEndian::read_u32(&slice[8..12]);
        let cn_num = LittleEndian::read_u32(&slice[12..16]);
        let outer_index = LittleEndian::read_i32(&slice[16..20]);
        let on_idx = LittleEndian::read_u32(&slice[20..24]);
        let on_num = LittleEndian::read_u32(&slice[24..28]);
        if cp_idx as usize >= names.len()
            || cn_idx as usize >= names.len()
            || on_idx as usize >= names.len()
        {
            return Vec::new();
        }
        raw_imports.push((
            format_fname_idx(names, cp_idx, cp_num),
            format_fname_idx(names, cn_idx, cn_num),
            outer_index,
            format_fname_idx(names, on_idx, on_num),
        ));
    }

    let mut out = Vec::with_capacity(raw_imports.len());
    for (class_package, class_name, outer_index, object_name) in &raw_imports {
        let mut pkg_path = object_name.clone();
        let mut cur_outer = *outer_index;
        for _ in 0..16 {
            if cur_outer >= 0 {
                break;
            }
            let oidx = (-cur_outer - 1) as usize;
            let Some((_, _, next_outer, outer_name)) = raw_imports.get(oidx) else {
                break;
            };
            if *next_outer == 0 {
                pkg_path = outer_name.clone();
                break;
            }
            cur_outer = *next_outer;
        }
        out.push(UassetImportEntry {
            class_package: class_package.clone(),
            class_name: class_name.clone(),
            outer_index: *outer_index,
            object_name: object_name.clone(),
            package_path: pkg_path,
        });
    }
    assert_eq!(out.len(), import_count);
    out
}

fn detect_export_stride(
    uasset: &[u8],
    export_offset: usize,
    export_count: usize,
    names_len: usize,
) -> Option<usize> {
    assert!(export_count > 0);
    for stride in [104usize, 96, 100, 92, 108, 88] {
        if export_offset + export_count * stride > uasset.len() {
            continue;
        }
        let mut ok = true;
        for i in 0..export_count {
            let off = export_offset + i * stride;
            let Some(s) = uasset.get(off..off + 44) else {
                ok = false;
                break;
            };
            let on_idx = LittleEndian::read_u32(&s[16..20]) as usize;
            let on_num = LittleEndian::read_u32(&s[20..24]);
            let serial_size = LittleEndian::read_i64(&s[28..36]);
            let serial_offset = LittleEndian::read_i64(&s[36..44]);
            if on_idx >= names_len
                || on_num > 65_536
                || !(0..=256 * 1024 * 1024).contains(&serial_size)
                || serial_offset < (uasset.len() as i64)
            {
                ok = false;
                break;
            }
        }
        if ok {
            return Some(stride);
        }
    }
    None
}

/// Parses the `FObjectExport` table from a cooked UE 4.26/4.27 `.uasset` header.
pub fn parse_uasset_exports(
    uasset: &[u8],
    names: &[String],
    imports: &[UassetImportEntry],
) -> Vec<UassetExportEntry> {
    assert!(uasset.len() <= 256 * 1024 * 1024);
    assert!(names.len() <= 65_536);
    if uasset.len() < 64 || names.is_empty() {
        return Vec::new();
    }
    let Some((export_count, export_offset, _, _)) = find_table_bounds(uasset, names.len()) else {
        return Vec::new();
    };
    let Some(stride) = detect_export_stride(uasset, export_offset, export_count, names.len())
    else {
        return Vec::new();
    };
    assert!(export_count <= 65_536);

    let mut out = Vec::with_capacity(export_count);
    let base_serial_offset = uasset.len();
    for i in 0..export_count {
        let off = export_offset + i * stride;
        let Some(slice) = uasset.get(off..off + 44) else {
            return Vec::new();
        };
        let class_index = LittleEndian::read_i32(&slice[0..4]);
        let super_index = LittleEndian::read_i32(&slice[4..8]);
        let template_index = LittleEndian::read_i32(&slice[8..12]);
        let outer_index = LittleEndian::read_i32(&slice[12..16]);
        let on_idx = LittleEndian::read_u32(&slice[16..20]);
        let on_num = LittleEndian::read_u32(&slice[20..24]);
        let serial_size = LittleEndian::read_i64(&slice[28..36]).max(0) as usize;
        let serial_offset = LittleEndian::read_i64(&slice[36..44]).max(0) as usize;
        let class_name = if class_index < 0 {
            imports
                .get((-class_index - 1) as usize)
                .map(|imp| imp.object_name.clone())
                .unwrap_or_default()
        } else {
            String::new()
        };
        out.push(UassetExportEntry {
            export_index: i + 1,
            class_index,
            class_name,
            super_index,
            template_index,
            outer_index,
            object_name: format_fname_idx(names, on_idx, on_num),
            serial_size,
            serial_offset,
            uexp_offset: serial_offset.saturating_sub(base_serial_offset),
        });
    }
    assert_eq!(out.len(), export_count);
    out
}

/// Resolves a UE4 `FPackageIndex` (`< 0` for `ImportTable[-idx - 1]`) to its package path.
pub fn resolve_package_index(imports: &[UassetImportEntry], pkg_index: i32) -> Option<String> {
    assert!(imports.len() <= 65_536);
    if pkg_index >= 0 {
        return None;
    }
    let idx = (-pkg_index - 1) as usize;
    assert!(idx < 65_536);
    imports.get(idx).map(|imp| imp.package_path.clone())
}

/// Decodes a cooked UE 4.27 `UTexture2D` from its `.uasset`, `.uexp`, and optional
/// `.ubulk` buffers.
pub fn decode_texture2d(
    uasset: &[u8],
    uexp: &[u8],
    ubulk: Option<&[u8]>,
) -> Result<DecodedTexture, ExtractError> {
    assert!(uasset.len() <= 256 * 1024 * 1024);
    assert!(uexp.len() <= 256 * 1024 * 1024);
    let names = parse_uasset_names(uasset);

    // 1. Primary path: locate `FTexturePlatformData`'s `PixelFormatString` (`"PF_..."` FString)
    // inside `.uexp`.
    if let Some(tex) = try_parse_platform_data(uexp, ubulk, &names) {
        return Ok(tex);
    }

    // 2. Fallback path: determine PixelFormat from `.uasset` FNames (or ASCII scan) and scan
    // `.uexp` for FByteBulkData mip headers.
    let format = names
        .iter()
        .find_map(|n| PixelFormat::from_name(n))
        .or_else(|| find_ascii_pixel_format(uexp))
        .or_else(|| find_ascii_pixel_format(uasset))
        .ok_or_else(|| {
            ExtractError::UassetFormat(
                "Could not determine texture PixelFormat (no PF_* found)".to_string(),
            )
        })?;

    try_scan_mip_in_uexp(uexp, ubulk, format, &names)
}

fn find_ascii_pixel_format(buf: &[u8]) -> Option<PixelFormat> {
    assert!(buf.len() <= 256 * 1024 * 1024);
    for i in memchr::memmem::find_iter(buf, b"PF_") {
        let end = buf[i..(i + 24).min(buf.len())]
            .iter()
            .position(|&b| b == 0 || !b.is_ascii_graphic())
            .unwrap_or(0);
        if end > 3 {
            assert!(i + end <= buf.len());
            let s = std::str::from_utf8(&buf[i..i + end]).ok()?;
            if let Some(pf) = PixelFormat::from_name(s) {
                return Some(pf);
            }
        }
    }
    None
}

fn detect_texture_srgb(uexp: &[u8], names: &[String]) -> bool {
    assert!(uexp.len() <= 256 * 1024 * 1024);
    assert!(names.len() <= 65_536);
    if names.iter().any(|n| {
        matches!(
            n.as_str(),
            "TC_Masks"
                | "TC_Normalmap"
                | "TC_Grayscale"
                | "TC_Alpha"
                | "TC_Displacementmap"
                | "TC_VectorDisplacementmap"
                | "TC_HDR"
        )
    }) {
        return false;
    }
    if let (Some(srgb_idx), Some(bool_idx)) = (
        names.iter().position(|n| n == "SRGB"),
        names.iter().position(|n| n == "BoolProperty"),
    ) {
        let mut pat = [0u8; 24];
        LittleEndian::write_i32(&mut pat[0..4], srgb_idx as i32);
        LittleEndian::write_i32(&mut pat[8..12], bool_idx as i32);
        let head = &uexp[..uexp.len().min(1024)];
        if let Some(pos) = memchr::memmem::find(head, &pat)
            && let Some(&val) = head.get(pos + 24)
        {
            return val != 0;
        }
    }
    true
}

fn detect_texture_address_clamp(uexp: &[u8], names: &[String], prop_name: &str) -> bool {
    assert!(uexp.len() <= 256 * 1024 * 1024);
    assert!(names.len() <= 65_536);
    let (Some(prop_idx), Some(byte_idx)) = (
        names.iter().position(|n| n == prop_name),
        names.iter().position(|n| n == "ByteProperty"),
    ) else {
        return false;
    };
    let mut pat = [0u8; 24];
    LittleEndian::write_i32(&mut pat[0..4], prop_idx as i32);
    LittleEndian::write_i32(&mut pat[8..12], byte_idx as i32);
    LittleEndian::write_i32(&mut pat[16..20], 8);
    let head = &uexp[..uexp.len().min(1024)];
    if let Some(pos) = memchr::memmem::find(head, &pat)
        && let Some(val_slice) = head.get(pos + 33..pos + 37)
    {
        let val_idx = LittleEndian::read_i32(val_slice) as usize;
        if let Some(val_name) = names.get(val_idx) {
            return val_name.ends_with("TA_Clamp");
        }
    }
    false
}

fn try_parse_platform_data(
    uexp: &[u8],
    ubulk: Option<&[u8]>,
    names: &[String],
) -> Option<DecodedTexture> {
    assert!(uexp.len() <= 256 * 1024 * 1024);
    let srgb = detect_texture_srgb(uexp, names);
    let clamp_x = detect_texture_address_clamp(uexp, names, "AddressX");
    let clamp_y = detect_texture_address_clamp(uexp, names, "AddressY");
    for i in memchr::memmem::find_iter(uexp, b"PF_") {
        if i < 16 || i + 16 > uexp.len() {
            continue;
        }
        let str_len = LittleEndian::read_i32(&uexp[i - 4..i]);
        if !(4..=32).contains(&str_len) {
            continue;
        }
        let str_end = i + (str_len as usize);
        if str_end > uexp.len() || uexp[str_end - 1] != 0 {
            continue;
        }
        let pf_str = std::str::from_utf8(&uexp[i..str_end - 1]).ok()?;
        let format = PixelFormat::from_name(pf_str)?;

        let plat_size_x = LittleEndian::read_i32(&uexp[i - 16..i - 12]);
        let plat_size_y = LittleEndian::read_i32(&uexp[i - 12..i - 8]);
        if !(1..=16384).contains(&plat_size_x) || !(1..=16384).contains(&plat_size_y) {
            continue;
        }

        // After PixelFormatString: FirstMipToSerialize (i32), NumMips (i32).
        // Some UE4 builds have an extra SkipOffset (i64 or i32) before FirstMipToSerialize.
        for header_skip in [0usize, 4usize, 8usize] {
            let mip_table_off = str_end + header_skip;
            let Some(hdr_slice) = uexp.get(mip_table_off..mip_table_off + 8) else {
                continue;
            };
            let _first_mip = LittleEndian::read_i32(&hdr_slice[0..4]);
            let num_mips = LittleEndian::read_i32(&hdr_slice[4..8]);
            if !(1..=16).contains(&num_mips) {
                continue;
            }
            let mip0_off = mip_table_off + 8;
            if let Some((width, height, mip_bytes)) = parse_single_mip(
                uexp,
                ubulk,
                mip0_off,
                format,
                plat_size_x as usize,
                plat_size_y as usize,
            ) && let Ok(rgba8) = decode_mip_to_rgba8(mip_bytes, width, height, format)
            {
                assert_eq!(rgba8.len(), width * height * 4);
                return Some(DecodedTexture {
                    width,
                    height,
                    format,
                    num_mips: num_mips as usize,
                    srgb,
                    clamp_x,
                    clamp_y,
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
    assert!(expected_w <= 65536 && expected_h <= 65536);
    let mut cur = Cursor::new(uexp.get(mip_off..)?);
    let b_cooked = cur.read_i32::<LittleEndian>().ok()?;
    if b_cooked != 1 {
        return None;
    }
    let bulk_flags = cur.read_u32::<LittleEndian>().ok()?;
    let is_64bit = (bulk_flags & 0x2000) != 0;

    let (elem_count, size_on_disk, after_bulk_hdr) = if is_64bit {
        let ec = cur.read_i64::<LittleEndian>().ok()? as usize;
        let sod = cur.read_i64::<LittleEndian>().ok()? as usize;
        let _offset_in_file = cur.read_i64::<LittleEndian>().ok()?;
        (ec, sod, mip_off + 32)
    } else {
        let ec = cur.read_i32::<LittleEndian>().ok()? as usize;
        let sod = cur.read_i32::<LittleEndian>().ok()? as usize;
        let _offset_in_file = cur.read_i64::<LittleEndian>().ok()?;
        (ec, sod, mip_off + 24)
    };

    let is_inline = (bulk_flags & 0x01) == 0 && (bulk_flags & 0x100) == 0;
    if is_inline
        && let Some(dim_slice) =
            uexp.get(after_bulk_hdr + elem_count..after_bulk_hdr + elem_count + 12)
    {
        let sx = LittleEndian::read_i32(&dim_slice[0..4]) as usize;
        let sy = LittleEndian::read_i32(&dim_slice[4..8]) as usize;
        let sz = LittleEndian::read_i32(&dim_slice[8..12]);
        if sx > 0 && sy > 0 && (1..=64).contains(&sz) && format.mip_byte_size(sx, sy) == elem_count
        {
            return Some((sx, sy, &uexp[after_bulk_hdr..after_bulk_hdr + elem_count]));
        }
    }

    // External `.ubulk` payload: SizeX, SizeY, SizeZ immediately follow the BulkData header.
    if let Some(ubulk_bytes) = ubulk
        && let Some(dim_slice) = uexp.get(after_bulk_hdr..after_bulk_hdr + 12)
    {
        let sx = LittleEndian::read_i32(&dim_slice[0..4]) as usize;
        let sy = LittleEndian::read_i32(&dim_slice[4..8]) as usize;
        let width = if sx > 0 { sx } else { expected_w };
        let height = if sy > 0 { sy } else { expected_h };
        let needed = format.mip_byte_size(width, height).max(size_on_disk);
        if ubulk_bytes.len() >= needed && needed > 0 {
            assert!(width > 0 && height > 0);
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
) -> Result<DecodedTexture, ExtractError> {
    assert!(uexp.len() <= 256 * 1024 * 1024);
    let srgb = detect_texture_srgb(uexp, names);
    let clamp_x = detect_texture_address_clamp(uexp, names, "AddressX");
    let clamp_y = detect_texture_address_clamp(uexp, names, "AddressY");
    for off in 0..uexp.len().saturating_sub(36) {
        if LittleEndian::read_i32(&uexp[off..off + 4]) == 1
            && let Some((w, h, mip_bytes)) = parse_single_mip(uexp, ubulk, off, format, 0, 0)
            && let Ok(rgba8) = decode_mip_to_rgba8(mip_bytes, w, h, format)
        {
            assert_eq!(rgba8.len(), w * h * 4);
            return Ok(DecodedTexture {
                width: w,
                height: h,
                format,
                num_mips: 1,
                srgb,
                clamp_x,
                clamp_y,
                names: names.to_vec(),
                rgba8,
            });
        }
    }

    Err(ExtractError::UassetFormat(format!(
        "Failed to locate valid {format:?} mipmap payload inside .uexp ({} bytes)",
        uexp.len()
    )))
}

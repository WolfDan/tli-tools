//! Cooked Unreal Engine 4.27 `UMaterial` and `UMaterialInstanceConstant` (`.uasset` + `.uexp`)
//! static-switch permutation, parameter override, uniform expression, and `ShaderMapHash` parser.

use std::io::Cursor;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::error::ExtractError;
use crate::uasset::{
    PACKAGE_FILE_TAG, UassetImportEntry, parse_uasset_imports, parse_uasset_names,
    resolve_package_index,
};

const UASSET_BYTES_MAX: usize = 256 * 1024 * 1024;
const UEXP_BYTES_MAX: usize = 256 * 1024 * 1024;
const PROPS_ITER_MAX: usize = 2048;
const ARRAY_ELEMS_MAX: usize = 1024;
const UNIFORM_NAMES_MAX: usize = 2048;

const _: () = {
    assert!(UASSET_BYTES_MAX == 268_435_456);
    assert!(PROPS_ITER_MAX == 2048);
};

/// Static boolean switch override (`FStaticSwitchParameter`) in a `UMaterialInstanceConstant`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticSwitchParam {
    pub name: String,
    pub value: bool,
}

/// Static RGBA channel mask override (`FStaticComponentMaskParameter`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticComponentMaskParam {
    pub name: String,
    pub r: bool,
    pub g: bool,
    pub b: bool,
    pub a: bool,
}

/// Scalar float parameter override (`FScalarParameterValue`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScalarParam {
    pub name: String,
    pub value: f32,
}

/// RGBA `FLinearColor` vector parameter override (`FVectorParameterValue`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VectorParam {
    pub name: String,
    pub rgba: [f32; 4],
}

/// `UTexture` parameter override (`FTextureParameterValue`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureParam {
    pub name: String,
    pub texture_path: String,
}

/// Extracted metadata, static switch permutation state, parameter overrides, uniform expression
/// names, and resolved 20-byte `ShaderMapHash` for a cooked `UMaterial` or `UMaterialInstance`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialInfo {
    pub virtual_path: String,
    pub parent_path: Option<String>,
    pub parent_chain: Vec<String>,
    pub resolved_shader_source_path: String,
    pub has_static_permutation_resource: bool,
    pub blend_mode: Option<String>,
    pub shading_model: Option<String>,
    pub static_switches: Vec<StaticSwitchParam>,
    pub static_component_masks: Vec<StaticComponentMaskParam>,
    pub scalar_params: Vec<ScalarParam>,
    pub vector_params: Vec<VectorParam>,
    pub texture_params: Vec<TextureParam>,
    pub uniform_expression_names: Vec<String>,
    pub referenced_textures: Vec<String>,
    pub shader_map_hash_hex: Option<String>,
    #[serde(skip)]
    pub shader_map_hash: Option<[u8; 20]>,
}

#[derive(Clone, Debug)]
struct PropertyTag {
    name: String,
    prop_type: String,
    size: usize,
    extra_name: String,
    bool_val: bool,
    value_offset: usize,
    next_offset: usize,
}

/// Converts a UE4 package path (`/Game/...` or `/Engine/...`) into its Pak VFS virtual path.
pub fn game_path_to_virtual_path(game_path: &str) -> String {
    assert!(!game_path.is_empty());
    assert!(game_path.len() <= 4096);
    if let Some(rest) = game_path.strip_prefix("/Game/") {
        format!("UE_game/Content/{rest}")
    } else if let Some(rest) = game_path.strip_prefix("/Engine/") {
        format!("Engine/Content/{rest}")
    } else {
        game_path.trim_start_matches('/').to_string()
    }
}

fn read_fname_at(buf: &[u8], pos: usize, names: &[String]) -> Option<(String, usize)> {
    assert!(names.len() <= 65_536);
    let slice = buf.get(pos..pos + 8)?;
    let idx = LittleEndian::read_u32(&slice[0..4]) as usize;
    let num = LittleEndian::read_u32(&slice[4..8]);
    let base = names.get(idx)?;
    let formatted = if num == 0 {
        base.clone()
    } else {
        format!("{base}_{}", num - 1)
    };
    assert!(pos + 8 <= buf.len());
    Some((formatted, pos + 8))
}

fn read_property_tag(buf: &[u8], start: usize, names: &[String]) -> Option<Option<PropertyTag>> {
    assert!(start <= buf.len());
    assert!(names.len() <= 65_536);
    let (name, mut pos) = read_fname_at(buf, start, names)?;
    if name == "None" {
        return Some(None);
    }
    let (prop_type, next_pos) = read_fname_at(buf, pos, names)?;
    pos = next_pos;
    let hdr = buf.get(pos..pos + 8)?;
    let size = LittleEndian::read_i32(&hdr[0..4]);
    if !(0..=16 * 1024 * 1024).contains(&size) {
        return None;
    }
    let size = size as usize;
    pos += 8;

    let mut extra_name = String::new();
    let mut bool_val = false;
    match prop_type.as_str() {
        "StructProperty" => {
            let (sname, p2) = read_fname_at(buf, pos, names)?;
            extra_name = sname;
            let has_guid = *buf.get(p2 + 16)?;
            pos = p2 + 17 + if has_guid != 0 { 16 } else { 0 };
        }
        "BoolProperty" => {
            bool_val = *buf.get(pos)? != 0;
            let has_guid = *buf.get(pos + 1)?;
            pos += 2 + if has_guid != 0 { 16 } else { 0 };
        }
        "ByteProperty" | "EnumProperty" | "ArrayProperty" | "SetProperty" => {
            let (ename, p2) = read_fname_at(buf, pos, names)?;
            extra_name = ename;
            let has_guid = *buf.get(p2)?;
            pos = p2 + 1 + if has_guid != 0 { 16 } else { 0 };
        }
        "MapProperty" => {
            let (_, p2) = read_fname_at(buf, pos, names)?;
            let (_, p3) = read_fname_at(buf, p2, names)?;
            let has_guid = *buf.get(p3)?;
            pos = p3 + 1 + if has_guid != 0 { 16 } else { 0 };
        }
        _ => {
            let has_guid = *buf.get(pos)?;
            pos += 1 + if has_guid != 0 { 16 } else { 0 };
        }
    }
    let next_offset = pos.checked_add(size)?;
    if next_offset > buf.len() {
        return None;
    }
    Some(Some(PropertyTag {
        name,
        prop_type,
        size,
        extra_name,
        bool_val,
        value_offset: pos,
        next_offset,
    }))
}

fn extract_parameter_info_name(buf: &[u8], start: usize, end: usize, names: &[String]) -> String {
    assert!(start <= end && end <= buf.len());
    let mut pos = start;
    for _ in 0..16 {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        if tag.name == "Name"
            && tag.prop_type == "NameProperty"
            && tag.size == 8
            && let Some((pname, _)) = read_fname_at(buf, tag.value_offset, names)
        {
            return pname;
        }
        pos = tag.next_offset;
    }
    String::new()
}

fn parse_single_param_struct(
    buf: &[u8],
    start: usize,
    end: usize,
    names: &[String],
    imports: &[UassetImportEntry],
    info: &mut MaterialInfo,
    struct_type: &str,
) -> Option<usize> {
    assert!(start <= end && end <= buf.len());
    assert!(!struct_type.is_empty());
    let mut pos = start;
    let mut param_name = String::new();
    let mut f_val = 0.0f32;
    let mut v_val = [0.0f32; 4];
    let mut t_val = String::new();
    let mut sw_val = false;
    let mut mask = (false, false, false, false);

    for _ in 0..32 {
        let tag_opt = read_property_tag(buf, pos, names)?;
        let Some(tag) = tag_opt else {
            pos += 8;
            break;
        };
        match (tag.name.as_str(), tag.prop_type.as_str()) {
            ("ParameterInfo", "StructProperty") => {
                param_name =
                    extract_parameter_info_name(buf, tag.value_offset, tag.next_offset, names);
            }
            ("ParameterValue", "FloatProperty") if tag.size == 4 => {
                f_val = LittleEndian::read_f32(&buf[tag.value_offset..tag.value_offset + 4]);
            }
            ("ParameterValue", "StructProperty") if tag.size == 16 => {
                for (i, dst) in v_val.iter_mut().enumerate() {
                    let o = tag.value_offset + i * 4;
                    *dst = LittleEndian::read_f32(&buf[o..o + 4]);
                }
            }
            ("ParameterValue", "ObjectProperty") if tag.size == 4 => {
                let pidx = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4]);
                t_val = resolve_package_index(imports, pidx).unwrap_or_default();
            }
            ("Value", "BoolProperty") => sw_val = tag.bool_val,
            ("R", "BoolProperty") => mask.0 = tag.bool_val,
            ("G", "BoolProperty") => mask.1 = tag.bool_val,
            ("B", "BoolProperty") => mask.2 = tag.bool_val,
            ("A", "BoolProperty") => mask.3 = tag.bool_val,
            _ => {}
        }
        pos = tag.next_offset;
    }

    if !param_name.is_empty() {
        match struct_type {
            "ScalarParameterValue" => info.scalar_params.push(ScalarParam {
                name: param_name,
                value: f_val,
            }),
            "VectorParameterValue" => info.vector_params.push(VectorParam {
                name: param_name,
                rgba: v_val,
            }),
            "TextureParameterValue" => info.texture_params.push(TextureParam {
                name: param_name,
                texture_path: t_val,
            }),
            "StaticSwitchParameter" => info.static_switches.push(StaticSwitchParam {
                name: param_name,
                value: sw_val,
            }),
            "StaticComponentMaskParameter" => {
                info.static_component_masks.push(StaticComponentMaskParam {
                    name: param_name,
                    r: mask.0,
                    g: mask.1,
                    b: mask.2,
                    a: mask.3,
                });
            }
            _ => {}
        }
    }
    Some(pos)
}

fn parse_struct_array_property(
    buf: &[u8],
    tag: &PropertyTag,
    names: &[String],
    imports: &[UassetImportEntry],
    info: &mut MaterialInfo,
) {
    assert!(tag.value_offset <= tag.next_offset);
    assert!(tag.next_offset <= buf.len());
    if tag.size < 4 {
        return;
    }
    let count = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4]);
    if !(1..=ARRAY_ELEMS_MAX as i32).contains(&count) {
        return;
    }
    let Some(Some(proto)) = read_property_tag(buf, tag.value_offset + 4, names) else {
        return;
    };
    let mut elem_pos = proto.value_offset;
    for _ in 0..(count as usize) {
        if elem_pos >= tag.next_offset {
            break;
        }
        let Some(next_pos) = parse_single_param_struct(
            buf,
            elem_pos,
            tag.next_offset,
            names,
            imports,
            info,
            &proto.extra_name,
        ) else {
            break;
        };
        elem_pos = next_pos;
    }
}

fn parse_nested_material_struct(
    buf: &[u8],
    start: usize,
    end: usize,
    names: &[String],
    imports: &[UassetImportEntry],
    info: &mut MaterialInfo,
) {
    assert!(start <= end && end <= buf.len());
    let mut pos = start;
    for _ in 0..64 {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        match (tag.name.as_str(), tag.prop_type.as_str()) {
            ("BlendMode", "ByteProperty" | "EnumProperty") if tag.size == 8 => {
                if let Some((bm, _)) = read_fname_at(buf, tag.value_offset, names) {
                    info.blend_mode = Some(bm);
                }
            }
            ("ShadingModel", "ByteProperty" | "EnumProperty") if tag.size == 8 => {
                if let Some((sm, _)) = read_fname_at(buf, tag.value_offset, names) {
                    info.shading_model = Some(sm);
                }
            }
            ("StaticSwitchParameters" | "StaticComponentMaskParameters", "ArrayProperty") => {
                parse_struct_array_property(buf, &tag, names, imports, info);
            }
            _ => {}
        }
        pos = tag.next_offset;
    }
}

fn parse_top_level_properties(
    uexp: &[u8],
    names: &[String],
    imports: &[UassetImportEntry],
    info: &mut MaterialInfo,
) -> usize {
    assert!(uexp.len() <= UEXP_BYTES_MAX);
    assert!(names.len() <= 65_536);
    let mut pos = 0usize;
    let limit = uexp.len().saturating_sub(4);

    for _ in 0..PROPS_ITER_MAX {
        if pos >= limit {
            break;
        }
        let Some(tag_opt) = read_property_tag(uexp, pos, names) else {
            break;
        };
        let Some(tag) = tag_opt else {
            pos += 8;
            break;
        };
        match (tag.name.as_str(), tag.prop_type.as_str()) {
            ("Parent", "ObjectProperty") if tag.size == 4 => {
                let pidx = LittleEndian::read_i32(&uexp[tag.value_offset..tag.value_offset + 4]);
                info.parent_path = resolve_package_index(imports, pidx);
            }
            ("bHasStaticPermutationResource", "BoolProperty") => {
                info.has_static_permutation_resource = tag.bool_val;
            }
            ("BlendMode", "ByteProperty" | "EnumProperty") if tag.size == 8 => {
                if let Some((bm, _)) = read_fname_at(uexp, tag.value_offset, names) {
                    info.blend_mode = Some(bm);
                }
            }
            ("ShadingModel", "ByteProperty" | "EnumProperty") if tag.size == 8 => {
                if let Some((sm, _)) = read_fname_at(uexp, tag.value_offset, names) {
                    info.shading_model = Some(sm);
                }
            }
            ("BasePropertyOverrides" | "StaticParameters", "StructProperty") => {
                parse_nested_material_struct(
                    uexp,
                    tag.value_offset,
                    tag.next_offset,
                    names,
                    imports,
                    info,
                );
            }
            (
                "ScalarParameterValues" | "VectorParameterValues" | "TextureParameterValues",
                "ArrayProperty",
            ) => {
                parse_struct_array_property(uexp, &tag, names, imports, info);
            }
            ("CachedReferencedTextures", "ArrayProperty") if tag.size >= 4 => {
                let count = LittleEndian::read_i32(&uexp[tag.value_offset..tag.value_offset + 4])
                    .clamp(0, ARRAY_ELEMS_MAX as i32) as usize;
                for k in 0..count {
                    let o = tag.value_offset + 4 + k * 4;
                    if let Some(s) = uexp.get(o..o + 4) {
                        let pidx = LittleEndian::read_i32(s);
                        if let Some(p) = resolve_package_index(imports, pidx)
                            && !info.referenced_textures.contains(&p)
                        {
                            info.referenced_textures.push(p);
                        }
                    }
                }
            }
            _ => {}
        }
        pos = tag.next_offset;
    }
    assert!(pos <= uexp.len());
    pos
}

fn parse_fstring_inline(cur: &mut Cursor<&[u8]>) -> Option<String> {
    assert!((cur.position() as usize) <= cur.get_ref().len());
    let len = cur.read_i32::<LittleEndian>().ok()?;
    if len == 0 {
        return Some(String::new());
    }
    let start = cur.position() as usize;
    let buf = *cur.get_ref();
    if len < 0 {
        let chars = (-len) as usize;
        if chars > 1024 {
            return None;
        }
        let byte_len = chars * 2;
        let slice = buf.get(start..start + byte_len)?;
        cur.set_position((start + byte_len) as u64);
        let mut u16s = vec![0u16; chars];
        LittleEndian::read_u16_into(slice, &mut u16s);
        Some(
            String::from_utf16_lossy(&u16s)
                .trim_end_matches('\0')
                .to_string(),
        )
    } else {
        let byte_len = len as usize;
        if byte_len > 1024 {
            return None;
        }
        let slice = buf.get(start..start + byte_len)?;
        cur.set_position((start + byte_len) as u64);
        Some(
            String::from_utf8_lossy(slice)
                .trim_end_matches('\0')
                .to_string(),
        )
    }
}

fn parse_material_resource_uniform_names(uexp: &[u8], after_props: usize) -> Vec<String> {
    assert!(after_props <= uexp.len());
    let search_end = (after_props + 256).min(uexp.len().saturating_sub(16));
    for off in after_props..search_end {
        let Some(hdr) = uexp.get(off..off + 12) else {
            break;
        };
        let strip_flags = LittleEndian::read_i32(&hdr[0..4]);
        let num_res = LittleEndian::read_i32(&hdr[4..8]);
        let name_count = LittleEndian::read_i32(&hdr[8..12]);
        if strip_flags == 0
            && (1..=4).contains(&num_res)
            && (1..=UNIFORM_NAMES_MAX as i32).contains(&name_count)
        {
            let mut cur = Cursor::new(uexp);
            cur.set_position((off + 12) as u64);
            let mut out = Vec::with_capacity(name_count as usize);
            let mut ok = true;
            for _ in 0..(name_count as usize) {
                if let Some(s) = parse_fstring_inline(&mut cur)
                    && !s.is_empty()
                    && (cur.position() as usize) + 4 <= uexp.len()
                {
                    cur.set_position(cur.position() + 4);
                    out.push(s);
                } else {
                    ok = false;
                    break;
                }
            }
            if ok && !out.is_empty() {
                assert_eq!(out.len(), name_count as usize);
                return out;
            }
        }
    }
    Vec::new()
}

fn extract_tail_shadermap_hash(uexp: &[u8]) -> Option<[u8; 20]> {
    assert!(uexp.len() <= UEXP_BYTES_MAX);
    if uexp.len() < 32 {
        return None;
    }
    let tag_off = uexp.len() - 4;
    if LittleEndian::read_u32(&uexp[tag_off..]) != PACKAGE_FILE_TAG {
        return None;
    }
    let b_valid = LittleEndian::read_u32(&uexp[uexp.len() - 28..uexp.len() - 24]);
    if b_valid != 1 {
        return None;
    }
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&uexp[uexp.len() - 24..uexp.len() - 4]);
    if hash.iter().all(|&b| b == 0) {
        return None;
    }
    assert_eq!(hash.len(), 20);
    Some(hash)
}

/// Parses a cooked UE 4.27 `UMaterial` or `UMaterialInstanceConstant` (`.uasset` + `.uexp`)
/// into structured static switches, parameter overrides, uniform names, and `ShaderMapHash`.
pub fn parse_material_asset(
    virtual_path: &str,
    uasset: &[u8],
    uexp: &[u8],
) -> Result<MaterialInfo, ExtractError> {
    assert!(uasset.len() <= UASSET_BYTES_MAX);
    assert!(uexp.len() <= UEXP_BYTES_MAX);
    if uasset.len() < 8 || LittleEndian::read_u32(uasset) != PACKAGE_FILE_TAG {
        return Err(ExtractError::UassetFormat(format!(
            "Invalid .uasset package tag in {virtual_path}"
        )));
    }
    let names = parse_uasset_names(uasset);
    if names.is_empty() {
        return Err(ExtractError::UassetFormat(format!(
            "Empty FName table in {virtual_path}"
        )));
    }
    let imports = parse_uasset_imports(uasset, &names);

    let mut info = MaterialInfo {
        virtual_path: virtual_path.to_string(),
        parent_path: None,
        parent_chain: Vec::new(),
        resolved_shader_source_path: virtual_path.to_string(),
        has_static_permutation_resource: false,
        blend_mode: None,
        shading_model: None,
        static_switches: Vec::new(),
        static_component_masks: Vec::new(),
        scalar_params: Vec::new(),
        vector_params: Vec::new(),
        texture_params: Vec::new(),
        uniform_expression_names: Vec::new(),
        referenced_textures: Vec::new(),
        shader_map_hash_hex: None,
        shader_map_hash: None,
    };

    let after_props = parse_top_level_properties(uexp, &names, &imports, &mut info);
    info.uniform_expression_names = parse_material_resource_uniform_names(uexp, after_props);

    for imp in &imports {
        if imp.class_name == "Texture2D" && !info.referenced_textures.contains(&imp.package_path) {
            info.referenced_textures.push(imp.package_path.clone());
        }
    }

    if let Some(hash) = extract_tail_shadermap_hash(uexp) {
        info.shader_map_hash_hex = Some(hex::encode(hash));
        info.shader_map_hash = Some(hash);
    }

    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_path_to_virtual_path() {
        assert_eq!(
            game_path_to_virtual_path("/Game/Art/UIV2/Common/BaseMaterials/UI_M_V2_Base"),
            "UE_game/Content/Art/UIV2/Common/BaseMaterials/UI_M_V2_Base"
        );
        assert_eq!(
            game_path_to_virtual_path("/Engine/EngineResources/DefaultTexture"),
            "Engine/Content/EngineResources/DefaultTexture"
        );
    }

    #[test]
    fn test_board_cell_material_permutations_if_game_installed() {
        let Some(game_dir) = crate::discover_game_dir() else {
            return;
        };
        let Ok(vfs) = crate::PakVfs::open_game_dir(&game_dir, crate::DEFAULT_AES_KEY) else {
            return;
        };
        let m120 = vfs
            .read_material_info(
                "UE_game/Content/Art/UIV2/S16Gameplay/UIEffect/S16BoardCell/Material/UI_MI_V2_S16BoardCell_120",
            )
            .expect("Cell 120 MaterialInfo");
        assert!(m120.has_static_permutation_resource);
        assert_eq!(
            m120.shader_map_hash_hex.as_deref(),
            Some("e84bcebd9e9a6acca18430d0e63a9841f49e1675")
        );
        assert!(!m120.static_switches.is_empty());
        assert!(!m120.uniform_expression_names.is_empty());

        let m117 = vfs
            .read_material_info(
                "UE_game/Content/Art/UIV2/S16Gameplay/UIEffect/S16BoardCell/Material/UI_MI_V2_S16BoardCell_117",
            )
            .expect("Cell 117 MaterialInfo with Parent fallback");
        assert!(!m117.has_static_permutation_resource);
        assert_eq!(
            m117.shader_map_hash_hex.as_deref(),
            Some("7628ad520fe22c1a65d1912e2ce0adc9905083d2")
        );
    }
}

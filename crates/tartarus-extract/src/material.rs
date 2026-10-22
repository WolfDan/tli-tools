//! Cooked Unreal Engine 4.27 `UMaterial` and `UMaterialInstanceConstant` (`.uasset` + `.uexp`)
//! static-switch permutation, parameter override, uniform expression, and `ShaderMapHash` parser.

use std::collections::BTreeMap;
use std::io::Cursor;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::error::ExtractError;
use crate::uasset::{
    PACKAGE_FILE_TAG, UassetImportEntry, format_fname_idx, parse_uasset_imports,
    parse_uasset_names, resolve_package_index,
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

/// Header (`8` bytes) for a single vector or scalar preshader in `FMaterialUniformExpressionSet`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreshaderHeader {
    pub opcode_offset: u32,
    pub opcode_len: u32,
}

/// Scalar parameter descriptor (`24` bytes) in `FMaterialUniformExpressionSet`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UniformScalarParamInfo {
    pub name: String,
    pub default_value: f32,
}

/// Vector parameter descriptor (`36` bytes) in `FMaterialUniformExpressionSet`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UniformVectorParamInfo {
    pub name: String,
    pub default_rgba: [f32; 4],
}

/// 2D texture parameter descriptor (`28` bytes) in `FMaterialUniformExpressionSet`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniformTexture2DParamInfo {
    pub name: String,
    pub tex_index: i32,
    pub sampler_source: u32,
}

/// Parsed frozen `FMaterialUniformExpressionSet` from a cooked UE 4.26/4.27 material resource.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MaterialUniformExpressionSet {
    pub vec_preshaders: Vec<PreshaderHeader>,
    pub scalar_preshaders: Vec<PreshaderHeader>,
    pub scalar_params: Vec<UniformScalarParamInfo>,
    pub vec_params: Vec<UniformVectorParamInfo>,
    pub tex2d_params: Vec<UniformTexture2DParamInfo>,
    #[serde(skip)]
    pub preshader_data: Vec<u8>,
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
    pub base_referenced_textures: Vec<String>,
    pub uniform_expression_set: Option<MaterialUniformExpressionSet>,
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
            ("ReferencedTextures" | "CachedReferencedTextures", "ArrayProperty")
                if tag.size >= 4 =>
            {
                let count = LittleEndian::read_i32(&uexp[tag.value_offset..tag.value_offset + 4])
                    .clamp(0, ARRAY_ELEMS_MAX as i32) as usize;
                let is_base_ref =
                    tag.name == "ReferencedTextures" || info.base_referenced_textures.is_empty();
                if is_base_ref {
                    info.base_referenced_textures.clear();
                }
                for k in 0..count {
                    let o = tag.value_offset + 4 + k * 4;
                    if let Some(s) = uexp.get(o..o + 4) {
                        let pidx = LittleEndian::read_i32(s);
                        let resolved = resolve_package_index(imports, pidx).unwrap_or_default();
                        if is_base_ref {
                            info.base_referenced_textures.push(resolved.clone());
                        }
                        if !resolved.is_empty() && !info.referenced_textures.contains(&resolved) {
                            info.referenced_textures.push(resolved);
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

fn read_frozen_tarray(frozen: &[u8], ptr_off: usize) -> Option<(usize, usize)> {
    assert!(frozen.len() <= UEXP_BYTES_MAX);
    let hdr = frozen.get(ptr_off..ptr_off + 16)?;
    let raw_ptr = LittleEndian::read_u64(&hdr[0..8]);
    let arr_num = LittleEndian::read_i32(&hdr[8..12]);
    let arr_max = LittleEndian::read_i32(&hdr[12..16]);
    if arr_num <= 0 || arr_max < arr_num || arr_num > ARRAY_ELEMS_MAX as i32 * 64 {
        return Some((0, 0));
    }
    let data_off = ptr_off.checked_add((raw_ptr >> 1) as usize)?;
    if data_off > frozen.len() {
        return None;
    }
    assert!(arr_num > 0);
    Some((data_off, arr_num as usize))
}

fn parse_script_names_patch_map(
    uexp: &[u8],
    p_script_names: usize,
    names: &[String],
) -> BTreeMap<usize, String> {
    assert!(uexp.len() <= UEXP_BYTES_MAX);
    assert!(names.len() <= 65_536);
    let mut patch_map = BTreeMap::new();
    let Some(hdr) = uexp.get(p_script_names..p_script_names + 12) else {
        return patch_map;
    };
    let n_script_names = LittleEndian::read_u32(&hdr[4..8]).min(UNIFORM_NAMES_MAX as u32) as usize;
    let mut p = p_script_names + 12;
    for _ in 0..n_script_names {
        let Some(entry) = uexp.get(p..p + 12) else {
            break;
        };
        let name_idx = LittleEndian::read_u32(&entry[0..4]);
        let name_num = LittleEndian::read_u32(&entry[4..8]);
        let n_patches = LittleEndian::read_u32(&entry[8..12]).min(256) as usize;
        p += 12;
        let fname = format_fname_idx(names, name_idx, name_num);
        for _ in 0..n_patches {
            let Some(ps) = uexp.get(p..p + 4) else {
                break;
            };
            let patch_off = LittleEndian::read_u32(ps) as usize;
            patch_map.insert(patch_off, fname.clone());
            p += 4;
        }
    }
    assert!(patch_map.len() <= UNIFORM_NAMES_MAX * 4);
    patch_map
}

fn decode_frozen_preshader_headers(frozen: &[u8], ptr_off: usize) -> Option<Vec<PreshaderHeader>> {
    assert!(frozen.len() <= UEXP_BYTES_MAX);
    let (off, cnt) = read_frozen_tarray(frozen, ptr_off)?;
    let mut out = Vec::with_capacity(cnt.min(ARRAY_ELEMS_MAX));
    for i in 0..cnt.min(ARRAY_ELEMS_MAX) {
        let s = frozen.get(off + i * 8..off + i * 8 + 8)?;
        out.push(PreshaderHeader {
            opcode_offset: LittleEndian::read_u32(&s[0..4]),
            opcode_len: LittleEndian::read_u32(&s[4..8]),
        });
    }
    assert!(out.len() <= ARRAY_ELEMS_MAX);
    Some(out)
}

fn decode_frozen_uniform_set(
    frozen: &[u8],
    patch_map: &BTreeMap<usize, String>,
) -> Option<MaterialUniformExpressionSet> {
    assert!(frozen.len() <= UEXP_BYTES_MAX);
    if frozen.len() < 0x130 {
        return None;
    }
    let vec_preshaders = decode_frozen_preshader_headers(frozen, 0x070)?;
    let scalar_preshaders = decode_frozen_preshader_headers(frozen, 0x080)?;
    let (sc_off, sc_cnt) = read_frozen_tarray(frozen, 0x090)?;
    let (vc_off, vc_cnt) = read_frozen_tarray(frozen, 0x0a0)?;
    let (tx_off, tx_cnt) = read_frozen_tarray(frozen, 0x0b0)?;
    let (pd_off, pd_cnt) = read_frozen_tarray(frozen, 0x120)?;

    let mut set = MaterialUniformExpressionSet {
        vec_preshaders,
        scalar_preshaders,
        ..Default::default()
    };
    for i in 0..sc_cnt.min(ARRAY_ELEMS_MAX) {
        let item_off = sc_off + i * 24;
        let s = frozen.get(item_off..item_off + 24)?;
        set.scalar_params.push(UniformScalarParamInfo {
            name: patch_map.get(&item_off).cloned().unwrap_or_default(),
            default_value: LittleEndian::read_f32(&s[20..24]),
        });
    }
    for i in 0..vc_cnt.min(ARRAY_ELEMS_MAX) {
        let item_off = vc_off + i * 36;
        let s = frozen.get(item_off..item_off + 36)?;
        set.vec_params.push(UniformVectorParamInfo {
            name: patch_map.get(&item_off).cloned().unwrap_or_default(),
            default_rgba: [
                LittleEndian::read_f32(&s[20..24]),
                LittleEndian::read_f32(&s[24..28]),
                LittleEndian::read_f32(&s[28..32]),
                LittleEndian::read_f32(&s[32..36]),
            ],
        });
    }
    for i in 0..tx_cnt.min(ARRAY_ELEMS_MAX) {
        let item_off = tx_off + i * 28;
        let s = frozen.get(item_off..item_off + 28)?;
        set.tex2d_params.push(UniformTexture2DParamInfo {
            name: patch_map.get(&item_off).cloned().unwrap_or_default(),
            tex_index: LittleEndian::read_i32(&s[20..24]),
            sampler_source: LittleEndian::read_u32(&s[24..28]),
        });
    }
    if pd_cnt > 0 {
        set.preshader_data = frozen.get(pd_off..pd_off + pd_cnt)?.to_vec();
    }
    assert!(set.vec_preshaders.len() <= ARRAY_ELEMS_MAX);
    Some(set)
}

fn parse_material_resource_uniforms(
    uexp: &[u8],
    after_props: usize,
    names: &[String],
) -> (Vec<String>, Option<MaterialUniformExpressionSet>) {
    assert!(after_props <= uexp.len());
    assert!(names.len() <= 65_536);
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
            if !ok {
                continue;
            }
            let pos = cur.position() as usize;
            if let Some(len_slice) = uexp.get(pos + 62..pos + 66) {
                let frozen_len = LittleEndian::read_u32(len_slice) as usize;
                let frozen_start = pos + 66;
                let frozen_end = frozen_start.saturating_add(frozen_len);
                if frozen_len >= 0x130
                    && let Some(frozen) = uexp.get(frozen_start..frozen_end)
                {
                    let patch_map = parse_script_names_patch_map(uexp, frozen_end, &out);
                    let uset = decode_frozen_uniform_set(frozen, &patch_map);
                    if uset.is_some() || !out.is_empty() {
                        return (out, uset);
                    }
                }
            }
            if !out.is_empty() {
                return (out, None);
            }
        }
    }
    (Vec::new(), None)
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

fn apply_preshader_binary_op(
    op: u8,
    dim_a: usize,
    a: [f32; 4],
    dim_b: usize,
    b: [f32; 4],
) -> (usize, [f32; 4]) {
    assert!((1..=4).contains(&dim_a));
    assert!((1..=4).contains(&dim_b));
    let out_dim = dim_a.max(dim_b);
    let mut out = [0.0f32; 4];
    for c in 0..out_dim {
        let va = if dim_a == 1 { a[0] } else { a[c] };
        let vb = if dim_b == 1 { b[0] } else { b[c] };
        out[c] = match op {
            0x05 => va + vb,
            0x06 => va - vb,
            0x07 => va * vb,
            0x08 => {
                if vb.abs() > 1e-8 {
                    va / vb
                } else {
                    0.0
                }
            }
            0x09 => {
                if vb.abs() > 1e-8 {
                    va % vb
                } else {
                    0.0
                }
            }
            0x0a => va.min(vb),
            0x0b => va.max(vb),
            0x13 => va.atan2(vb),
            _ => 0.0,
        };
    }
    (out_dim, out)
}

fn apply_preshader_unary_op(op: u8, dim: usize, v: [f32; 4]) -> (usize, [f32; 4]) {
    assert!((1..=4).contains(&dim));
    let mut out = [0.0f32; 4];
    for c in 0..dim {
        let x = v[c];
        out[c] = match op {
            0x0d => x.sin(),
            0x0e => x.cos(),
            0x0f => x.tan(),
            0x10 => x.clamp(-1.0, 1.0).asin(),
            0x11 => x.clamp(-1.0, 1.0).acos(),
            0x12 => x.atan(),
            0x14 => x.max(0.0).sqrt(),
            0x15 => {
                if x.abs() > 1e-8 {
                    1.0 / x
                } else {
                    0.0
                }
            }
            0x18 => x.clamp(0.0, 1.0),
            0x19 => x.abs(),
            0x1a => x.floor(),
            0x1b => x.ceil(),
            0x1c => x.round(),
            0x1d => x.trunc(),
            0x1e => {
                if x > 0.0 {
                    1.0
                } else if x < 0.0 {
                    -1.0
                } else {
                    0.0
                }
            }
            0x1f => x - x.floor(),
            _ => x,
        };
    }
    assert!(out_dim_valid(dim));
    (dim, out)
}

fn out_dim_valid(dim: usize) -> bool {
    (1..=4).contains(&dim)
}

fn apply_preshader_clamp(stack: &mut Vec<(usize, [f32; 4])>) {
    assert!(stack.len() <= 4096);
    let (dh, hi) = stack.pop().unwrap_or((1, [1.0; 4]));
    let (dl, lo) = stack.pop().unwrap_or((1, [0.0; 4]));
    let (dx, x) = stack.pop().unwrap_or((1, [0.0; 4]));
    let dim = dx.max(dl).max(dh);
    let mut out = [0.0f32; 4];
    for c in 0..dim {
        let vx = if dx == 1 { x[0] } else { x[c] };
        let vlo = if dl == 1 { lo[0] } else { lo[c] };
        let vhi = if dh == 1 { hi[0] } else { hi[c] };
        out[c] = vx.clamp(vlo, vhi);
    }
    assert!(out_dim_valid(dim));
    stack.push((dim, out));
}

fn eval_single_preshader(
    info: &MaterialInfo,
    uset: &MaterialUniformExpressionSet,
    hdr: PreshaderHeader,
    scalar_overrides: &[(&str, f32)],
    vector_overrides: &[(&str, [f32; 4])],
) -> [f32; 4] {
    let start = hdr.opcode_offset as usize;
    let end = start.saturating_add(hdr.opcode_len as usize);
    let Some(code) = uset.preshader_data.get(start..end) else {
        return [0.0; 4];
    };
    assert!(code.len() <= 65_536);
    let mut stack: Vec<(usize, [f32; 4])> = Vec::with_capacity(16);
    let mut pc = 0usize;
    for _ in 0..4096 {
        if pc >= code.len() {
            break;
        }
        let op = code[pc];
        pc += 1;
        match op {
            0x00 => {}
            0x01 => stack.push((1, [0.0; 4])),
            0x02 => {
                let dim = (*code.get(pc).unwrap_or(&1) as usize).clamp(1, 4);
                pc += 1;
                let mut val = [0.0f32; 4];
                for dst in val.iter_mut().take(dim) {
                    if let Some(s) = code.get(pc..pc + 4) {
                        *dst = LittleEndian::read_f32(s);
                    }
                    pc += 4;
                }
                stack.push((dim, val));
            }
            0x03 => {
                let idx = code.get(pc..pc + 2).map_or(0, LittleEndian::read_u16) as usize;
                pc += 2;
                let v = uset.scalar_params.get(idx).map_or(0.0, |p| {
                    info.resolve_scalar_param(&p.name, p.default_value, scalar_overrides)
                });
                stack.push((1, [v, 0.0, 0.0, 0.0]));
            }
            0x04 => {
                let idx = code.get(pc..pc + 2).map_or(0, LittleEndian::read_u16) as usize;
                pc += 2;
                let v = uset.vec_params.get(idx).map_or([0.0; 4], |p| {
                    info.resolve_vector_param(&p.name, p.default_rgba, vector_overrides)
                });
                stack.push((4, v));
            }
            0x05..=0x0b | 0x13 => {
                let (db, b) = stack.pop().unwrap_or((1, [0.0; 4]));
                let (da, a) = stack.pop().unwrap_or((1, [0.0; 4]));
                stack.push(apply_preshader_binary_op(op, da, a, db, b));
            }
            0x0c => apply_preshader_clamp(&mut stack),
            0x0d..=0x12 | 0x14 | 0x15 | 0x18..=0x1f => {
                let (d, v) = stack.pop().unwrap_or((1, [0.0; 4]));
                stack.push(apply_preshader_unary_op(op, d, v));
            }
            0x16 | 0x17 | 0x20 | 0x21 | 0x23 | 0x24 => {
                eval_preshader_vector_op(op, code, &mut pc, &mut stack);
            }
            _ => break,
        }
    }
    assert!(stack.len() <= 4096);
    stack.pop().map_or([0.0; 4], |(_, v)| v)
}

fn eval_preshader_vector_op(
    op: u8,
    code: &[u8],
    pc: &mut usize,
    stack: &mut Vec<(usize, [f32; 4])>,
) {
    assert!(*pc <= code.len());
    match op {
        0x16 => {
            let (d, v) = stack.pop().unwrap_or((1, [0.0; 4]));
            let len = v.iter().take(d).map(|x| x * x).sum::<f32>().sqrt();
            stack.push((1, [len, 0.0, 0.0, 0.0]));
        }
        0x17 => {
            let (d, v) = stack.pop().unwrap_or((1, [0.0; 4]));
            let len = v.iter().take(d).map(|x| x * x).sum::<f32>().sqrt();
            let mut out = [0.0f32; 4];
            if len > 1e-8 {
                for c in 0..d {
                    out[c] = v[c] / len;
                }
            }
            stack.push((d, out));
        }
        0x20 => {
            let (db, b) = stack.pop().unwrap_or((1, [0.0; 4]));
            let (da, a) = stack.pop().unwrap_or((1, [0.0; 4]));
            let dim = da.max(db);
            let dot: f32 = (0..dim).map(|c| a[c] * b[c]).sum();
            stack.push((1, [dot, 0.0, 0.0, 0.0]));
        }
        0x21 => {
            let (_, b) = stack.pop().unwrap_or((3, [0.0; 4]));
            let (_, a) = stack.pop().unwrap_or((3, [0.0; 4]));
            stack.push((
                3,
                [
                    a[1] * b[2] - a[2] * b[1],
                    a[2] * b[0] - a[0] * b[2],
                    a[0] * b[1] - a[1] * b[0],
                    0.0,
                ],
            ));
        }
        0x23 => {
            let num_comp = (*code.get(*pc).unwrap_or(&4) as usize).clamp(1, 4);
            let r = (*code.get(*pc + 1).unwrap_or(&0) as usize) & 3;
            let g = (*code.get(*pc + 2).unwrap_or(&0) as usize) & 3;
            let b = (*code.get(*pc + 3).unwrap_or(&0) as usize) & 3;
            let a = (*code.get(*pc + 4).unwrap_or(&0) as usize) & 3;
            *pc = (*pc + 5).min(code.len());
            let (_, v) = stack.pop().unwrap_or((4, [0.0; 4]));
            stack.push((num_comp, [v[r], v[g], v[b], v[a]]));
        }
        0x24 => {
            *pc = (*pc + 1).min(code.len());
            let (db, b) = stack.pop().unwrap_or((1, [0.0; 4]));
            let (da, a) = stack.pop().unwrap_or((1, [0.0; 4]));
            let mut out = [0.0f32; 4];
            let copy_a = da.min(4);
            out[..copy_a].copy_from_slice(&a[..copy_a]);
            for i in 0..db {
                if da + i < 4 {
                    out[da + i] = b[i];
                }
            }
            stack.push(((da + db).min(4), out));
        }
        _ => {}
    }
    assert!(stack.len() <= 4096);
}

impl MaterialInfo {
    /// Resolves a scalar parameter by checking `overrides`, then `self.scalar_params`, then `default_val`.
    pub fn resolve_scalar_param(
        &self,
        name: &str,
        default_val: f32,
        overrides: &[(&str, f32)],
    ) -> f32 {
        assert!(overrides.len() <= 1024);
        if let Some((_, val)) = overrides.iter().find(|(k, _)| *k == name) {
            return *val;
        }
        if let Some(p) = self.scalar_params.iter().find(|p| p.name == name) {
            return p.value;
        }
        default_val
    }

    /// Resolves a vector parameter by checking `overrides`, then `self.vector_params`, then `default_rgba`.
    pub fn resolve_vector_param(
        &self,
        name: &str,
        default_rgba: [f32; 4],
        overrides: &[(&str, [f32; 4])],
    ) -> [f32; 4] {
        assert!(overrides.len() <= 1024);
        if let Some((_, val)) = overrides.iter().find(|(k, _)| *k == name) {
            return *val;
        }
        if let Some(p) = self.vector_params.iter().find(|p| p.name == name) {
            return p.rgba;
        }
        default_rgba
    }

    /// Evaluates the material's `FMaterialUniformExpressionSet` preshaders into the HLSL `cb2`
    /// constant buffer (`Vec<[f32; 4]>`).
    pub fn evaluate_uniform_buffer(
        &self,
        scalar_overrides: &[(&str, f32)],
        vector_overrides: &[(&str, [f32; 4])],
    ) -> Vec<[f32; 4]> {
        assert!(scalar_overrides.len() <= 1024);
        assert!(vector_overrides.len() <= 1024);
        let Some(uset) = &self.uniform_expression_set else {
            return Vec::new();
        };
        let n_vec = uset.vec_preshaders.len();
        let n_sc = uset.scalar_preshaders.len();
        let total_vec4 = n_vec + n_sc.div_ceil(4);
        let mut cb2 = vec![[0.0f32; 4]; total_vec4];
        for (i, &hdr) in uset.vec_preshaders.iter().enumerate() {
            cb2[i] = eval_single_preshader(self, uset, hdr, scalar_overrides, vector_overrides);
        }
        for (j, &hdr) in uset.scalar_preshaders.iter().enumerate() {
            let val = eval_single_preshader(self, uset, hdr, scalar_overrides, vector_overrides)[0];
            cb2[n_vec + (j / 4)][j % 4] = val;
        }
        cb2
    }

    /// Resolves the ordered HLSL texture register slots (`t0..tK`) from `Tex2DParams`,
    /// applying any runtime `texture_overrides` (e.g. `("T1", monster_pic_path)`),
    /// then `self.texture_params`, then `self.base_referenced_textures[tex_index]`.
    pub fn resolve_texture_slots(&self, texture_overrides: &[(&str, &str)]) -> Vec<String> {
        assert!(texture_overrides.len() <= 256);
        let Some(uset) = &self.uniform_expression_set else {
            return Vec::new();
        };
        let mut slots = Vec::with_capacity(uset.tex2d_params.len());
        for param in &uset.tex2d_params {
            if let Some((_, ov_path)) = texture_overrides.iter().find(|(k, _)| *k == param.name) {
                slots.push(game_path_to_virtual_path(ov_path));
                continue;
            }
            if let Some(tp) = self.texture_params.iter().find(|p| p.name == param.name)
                && !tp.texture_path.is_empty()
            {
                slots.push(game_path_to_virtual_path(&tp.texture_path));
                continue;
            }
            let fallback = if param.tex_index >= 0 {
                self.base_referenced_textures
                    .get(param.tex_index as usize)
                    .or_else(|| self.referenced_textures.get(param.tex_index as usize))
                    .map(|s| game_path_to_virtual_path(s))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            slots.push(fallback);
        }
        assert_eq!(slots.len(), uset.tex2d_params.len());
        slots
    }

    /// Resolves per-slot `(clamp_u, clamp_v)` overrides for each HLSL texture register (`t0..tK`)
    /// from the material's `T1_UClamp`/`T1_VClamp`, `T2_UClamp`/`T2_VClamp`, and
    /// `OpacityUClamp`/`OpacityVClamp` scalar parameters.
    pub fn resolve_slot_uv_clamps(&self, scalar_overrides: &[(&str, f32)]) -> Vec<(bool, bool)> {
        assert!(scalar_overrides.len() <= 1024);
        let Some(uset) = &self.uniform_expression_set else {
            return Vec::new();
        };
        let mut clamps = Vec::with_capacity(uset.tex2d_params.len());
        for param in &uset.tex2d_params {
            let uv_clamp = match param.name.as_str() {
                "T1" => (
                    self.resolve_scalar_param("T1_UClamp", 0.0, scalar_overrides) > 0.5,
                    self.resolve_scalar_param("T1_VClamp", 0.0, scalar_overrides) > 0.5,
                ),
                "T2" => (
                    self.resolve_scalar_param("T2_UClamp", 0.0, scalar_overrides) > 0.5,
                    self.resolve_scalar_param("T2_VClamp", 0.0, scalar_overrides) > 0.5,
                ),
                "OpacityTexture" => (
                    self.resolve_scalar_param("OpacityUClamp", 0.0, scalar_overrides) > 0.5,
                    self.resolve_scalar_param("OpacityVClamp", 0.0, scalar_overrides) > 0.5,
                ),
                "DistortMask" => (true, true),
                _ => (false, false),
            };
            clamps.push(uv_clamp);
        }
        assert_eq!(clamps.len(), uset.tex2d_params.len());
        clamps
    }

    /// Merges inherited `ShaderMapHash`, `uniform_expression_set`, `base_referenced_textures`,
    /// blend/shading modes, and non-overridden parameter values from a parent material.
    pub fn merge_parent(&mut self, parent: MaterialInfo) {
        assert!(self.parent_chain.len() <= 16);
        assert!(!parent.virtual_path.is_empty());
        if self.shader_map_hash.is_none() && parent.shader_map_hash.is_some() {
            self.shader_map_hash = parent.shader_map_hash;
            self.shader_map_hash_hex = parent.shader_map_hash_hex;
            self.resolved_shader_source_path = parent.virtual_path;
        }
        if self.uniform_expression_names.is_empty() && !parent.uniform_expression_names.is_empty() {
            self.uniform_expression_names = parent.uniform_expression_names;
        }
        if self.uniform_expression_set.is_none() && parent.uniform_expression_set.is_some() {
            self.uniform_expression_set = parent.uniform_expression_set;
            if !parent.base_referenced_textures.is_empty() {
                self.base_referenced_textures = parent.base_referenced_textures;
            }
        } else if self.base_referenced_textures.is_empty()
            && !parent.base_referenced_textures.is_empty()
        {
            self.base_referenced_textures = parent.base_referenced_textures;
        }
        if self.blend_mode.is_none() {
            self.blend_mode = parent.blend_mode;
        }
        if self.shading_model.is_none() {
            self.shading_model = parent.shading_model;
        }
        for sp in parent.scalar_params {
            if !self.scalar_params.iter().any(|p| p.name == sp.name) {
                self.scalar_params.push(sp);
            }
        }
        for vp in parent.vector_params {
            if !self.vector_params.iter().any(|p| p.name == vp.name) {
                self.vector_params.push(vp);
            }
        }
        for tp in parent.texture_params {
            if !self.texture_params.iter().any(|p| p.name == tp.name) {
                self.texture_params.push(tp);
            }
        }
    }
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
        base_referenced_textures: Vec::new(),
        uniform_expression_set: None,
        shader_map_hash_hex: None,
        shader_map_hash: None,
    };

    let after_props = parse_top_level_properties(uexp, &names, &imports, &mut info);
    let (uniform_names, uniform_set) = parse_material_resource_uniforms(uexp, after_props, &names);
    info.uniform_expression_names = uniform_names;
    info.uniform_expression_set = uniform_set;

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
        assert!(m117.shader_map_hash_hex.is_some());
        assert!(m117.uniform_expression_set.is_some());
        let cb2_117 = m117.evaluate_uniform_buffer(&[], &[]);
        assert!(
            cb2_117.len() >= 3,
            "Expected evaluated Material uniform buffer for 117, got len {}",
            cb2_117.len()
        );
        let cb2_120 = m120.evaluate_uniform_buffer(&[], &[]);
        assert!(
            cb2_120.len() >= 10,
            "Expected evaluated Material uniform buffer for 120, got len {}",
            cb2_120.len()
        );

        let m046 = vfs
            .read_material_info(
                "UE_game/Content/Art/UIV2/S16Gameplay/UIEffect/S16BoardCell/Material/UI_MI_V2_S16BoardCell_046",
            )
            .expect("Cell 046 MaterialInfo");
        let slots_046 = m046.resolve_texture_slots(&[]);
        assert_eq!(slots_046.len(), 4);
        assert!(
            slots_046
                .iter()
                .any(|s| s.ends_with("UI_T_V2_S16BoardCell_049")),
            "Expected slots_046 to include MainTex UI_T_V2_S16BoardCell_049, got {slots_046:?}"
        );
        assert!(
            slots_046
                .iter()
                .any(|s| s.ends_with("UI_T_V2_S16BoardCell_023")),
            "Expected slots_046 to include MaskTex UI_T_V2_S16BoardCell_023, got {slots_046:?}"
        );
    }
}

//! Cooked Unreal Engine 4.27 UMG Widget Blueprint (`.uasset` + `.uexp`) layout,
//! widget hierarchy, custom `UUIImage.SoftObjectPath`, and `T = 0` animation extractor.

use std::collections::BTreeMap;

use byteorder::{ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};

use crate::error::ExtractError;
use crate::uasset::{
    PACKAGE_FILE_TAG, UassetExportEntry, UassetImportEntry, format_fname_idx, parse_uasset_exports,
    parse_uasset_imports, parse_uasset_names, resolve_package_index,
};

const UASSET_BYTES_MAX: usize = 256 * 1024 * 1024;
const UEXP_BYTES_MAX: usize = 256 * 1024 * 1024;
const SLOTS_COUNT_MAX: usize = 65_536;
const PROPS_ITER_MAX: usize = 512;

const _: () = {
    assert!(UASSET_BYTES_MAX == 268_435_456);
    assert!(std::mem::size_of::<UmgAnchorData>() == 40);
};

/// Canonical 1920x1080 UMG `FAnchorData` (`Offsets`, `Anchors`, `Alignment`) from a
/// cooked `UCanvasPanelSlot`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UmgAnchorData {
    pub offset_left: f32,
    pub offset_top: f32,
    pub offset_right: f32,
    pub offset_bottom: f32,
    pub anchor_min_x: f32,
    pub anchor_min_y: f32,
    pub anchor_max_x: f32,
    pub anchor_max_y: f32,
    pub align_x: f32,
    pub align_y: f32,
}

impl UmgAnchorData {
    /// Returns the default UE4 `FAnchorData` (`Offsets = (0, 0, 100, 30)`).
    pub const fn ue4_default() -> Self {
        Self {
            offset_left: 0.0,
            offset_top: 0.0,
            offset_right: 100.0,
            offset_bottom: 30.0,
            anchor_min_x: 0.0,
            anchor_min_y: 0.0,
            anchor_max_x: 0.0,
            anchor_max_y: 0.0,
            align_x: 0.0,
            align_y: 0.0,
        }
    }
}

/// Complete layout and visual properties for a widget export in a cooked UMG Blueprint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UmgWidgetNode {
    pub export_index: usize,
    pub name: String,
    pub class_name: String,
    pub parent_export_index: usize,
    pub z_order: i32,
    pub slot_order: usize,
    pub anchor: UmgAnchorData,
    pub auto_size: bool,
    pub has_canvas_slot: bool,
    pub visibility: String,
    pub render_opacity: f32,
    pub render_translation: [f32; 2],
    pub render_scale: [f32; 2],
    pub render_shear: [f32; 2],
    pub render_angle: f32,
    pub clipping: String,
    pub soft_object_path: Option<String>,
    pub particle_asset_path: Option<String>,
    pub brush_image_size: [f32; 2],
    pub color_and_opacity: [f32; 4],
}

impl Default for UmgWidgetNode {
    fn default() -> Self {
        Self {
            export_index: 0,
            name: String::new(),
            class_name: String::new(),
            parent_export_index: 0,
            z_order: 0,
            slot_order: 0,
            anchor: UmgAnchorData::ue4_default(),
            auto_size: false,
            has_canvas_slot: false,
            visibility: String::from("Visible"),
            render_opacity: 1.0,
            render_translation: [0.0, 0.0],
            render_scale: [1.0, 1.0],
            render_shear: [0.0, 0.0],
            render_angle: 0.0,
            clipping: String::from("Inherit"),
            soft_object_path: None,
            particle_asset_path: None,
            brush_image_size: [32.0, 32.0],
            color_and_opacity: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Evaluated `T = 0` track values from a `UWidgetAnimation` (`UMovieScene`) export.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UmgAnimationStateT0 {
    pub name: String,
    pub render_opacities: BTreeMap<String, f32>,
    pub material_scalars: BTreeMap<String, Vec<(String, f32)>>,
}

/// Extracted widget export metadata, `UCanvasPanelSlot` layout, full widget tree nodes,
/// and `T = 0` animation states from a UMG Blueprint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UmgBlueprint {
    pub widget_names: Vec<String>,
    pub referenced_assets: Vec<String>,
    pub anchor_slots: Vec<UmgAnchorData>,
    pub widgets: Vec<UmgWidgetNode>,
    pub animations_t0: Vec<UmgAnimationStateT0>,
}

#[derive(Clone, Debug)]
struct PropertyTag {
    name: String,
    prop_type: String,
    size: usize,
    bool_val: bool,
    value_offset: usize,
    next_offset: usize,
}

#[derive(Clone, Debug, Default)]
struct ParsedSlotLink {
    slot_export: usize,
    parent_export: usize,
    content_export: usize,
    z_order: i32,
    auto_size: bool,
    is_canvas_slot: bool,
    anchor: UmgAnchorData,
}

fn read_fname_at(buf: &[u8], pos: usize, names: &[String]) -> Option<(String, usize)> {
    assert!(names.len() <= 65_536);
    let slice = buf.get(pos..pos + 8)?;
    let idx = LittleEndian::read_u32(&slice[0..4]);
    let num = LittleEndian::read_u32(&slice[4..8]);
    if idx as usize >= names.len() || num > 65_536 {
        return None;
    }
    let formatted = format_fname_idx(names, idx, num);
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

    let mut bool_val = false;
    match prop_type.as_str() {
        "StructProperty" => {
            let (_, p2) = read_fname_at(buf, pos, names)?;
            let has_guid = *buf.get(p2 + 16)?;
            pos = p2 + 17 + if has_guid != 0 { 16 } else { 0 };
        }
        "BoolProperty" => {
            bool_val = *buf.get(pos)? != 0;
            let has_guid = *buf.get(pos + 1)?;
            pos += 2 + if has_guid != 0 { 16 } else { 0 };
        }
        "ByteProperty" | "EnumProperty" | "ArrayProperty" | "SetProperty" => {
            let (_, p2) = read_fname_at(buf, pos, names)?;
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
        bool_val,
        value_offset: pos,
        next_offset,
    }))
}

fn strip_subobject_suffix(asset_path: &str) -> String {
    assert!(asset_path.len() <= 4096);
    let trimmed = asset_path.trim_end_matches('\0');
    let last_seg = trimmed.rsplit('/').next().unwrap_or(trimmed);
    if let Some((pkg_stem, _)) = last_seg.rsplit_once('.') {
        let prefix_len = trimmed.len() - last_seg.len() + pkg_stem.len();
        assert!(prefix_len <= trimmed.len());
        return trimmed[..prefix_len].to_string();
    }
    trimmed.to_string()
}

fn parse_margin_struct(buf: &[u8], tag: &PropertyTag, names: &[String]) -> [f32; 4] {
    assert!(tag.value_offset <= tag.next_offset && tag.next_offset <= buf.len());
    let mut out = [0.0f32, 0.0, 100.0, 30.0];
    if let Some(first_opt) = read_property_tag(buf, tag.value_offset, names) {
        if first_opt.is_none() {
            return out;
        }
        if let Some(first) = first_opt
            && matches!(first.name.as_str(), "Left" | "Top" | "Right" | "Bottom")
            && first.prop_type == "FloatProperty"
        {
            let mut pos = tag.value_offset;
            for _ in 0..8 {
                if pos >= tag.next_offset {
                    break;
                }
                let Some(Some(sub)) = read_property_tag(buf, pos, names) else {
                    break;
                };
                if sub.prop_type == "FloatProperty" && sub.size == 4 {
                    let v = LittleEndian::read_f32(&buf[sub.value_offset..sub.value_offset + 4]);
                    match sub.name.as_str() {
                        "Left" => out[0] = v,
                        "Top" => out[1] = v,
                        "Right" => out[2] = v,
                        "Bottom" => out[3] = v,
                        _ => {}
                    }
                }
                pos = sub.next_offset;
            }
            return out;
        }
    }
    if tag.size == 16 && tag.value_offset + 16 <= buf.len() {
        for (i, dst) in out.iter_mut().enumerate() {
            let o = tag.value_offset + i * 4;
            *dst = LittleEndian::read_f32(&buf[o..o + 4]);
        }
    }
    assert!(out.len() == 4);
    out
}

fn parse_anchors_struct(buf: &[u8], tag: &PropertyTag, names: &[String]) -> [f32; 4] {
    assert!(tag.value_offset <= tag.next_offset && tag.next_offset <= buf.len());
    let mut out = [0.0f32; 4];
    let mut pos = tag.value_offset;
    for _ in 0..8 {
        if pos >= tag.next_offset {
            break;
        }
        let Some(Some(sub)) = read_property_tag(buf, pos, names) else {
            break;
        };
        if sub.prop_type == "StructProperty" && sub.size == 8 {
            let x = LittleEndian::read_f32(&buf[sub.value_offset..sub.value_offset + 4]);
            let y = LittleEndian::read_f32(&buf[sub.value_offset + 4..sub.value_offset + 8]);
            match sub.name.as_str() {
                "Minimum" => {
                    out[0] = x;
                    out[1] = y;
                }
                "Maximum" => {
                    out[2] = x;
                    out[3] = y;
                }
                _ => {}
            }
        }
        pos = sub.next_offset;
    }
    assert!(out.len() == 4);
    out
}

fn parse_anchor_data_struct(buf: &[u8], tag: &PropertyTag, names: &[String]) -> UmgAnchorData {
    assert!(tag.value_offset <= tag.next_offset && tag.next_offset <= buf.len());
    let mut anchor = UmgAnchorData::ue4_default();
    let mut pos = tag.value_offset;
    for _ in 0..16 {
        if pos >= tag.next_offset {
            break;
        }
        let Some(Some(sub)) = read_property_tag(buf, pos, names) else {
            break;
        };
        match (sub.name.as_str(), sub.prop_type.as_str()) {
            ("Offsets", "StructProperty") => {
                let m = parse_margin_struct(buf, &sub, names);
                anchor.offset_left = m[0];
                anchor.offset_top = m[1];
                anchor.offset_right = m[2];
                anchor.offset_bottom = m[3];
            }
            ("Anchors", "StructProperty") => {
                let a = parse_anchors_struct(buf, &sub, names);
                anchor.anchor_min_x = a[0];
                anchor.anchor_min_y = a[1];
                anchor.anchor_max_x = a[2];
                anchor.anchor_max_y = a[3];
            }
            ("Alignment", "StructProperty") if sub.size == 8 => {
                anchor.align_x =
                    LittleEndian::read_f32(&buf[sub.value_offset..sub.value_offset + 4]);
                anchor.align_y =
                    LittleEndian::read_f32(&buf[sub.value_offset + 4..sub.value_offset + 8]);
            }
            _ => {}
        }
        pos = sub.next_offset;
    }
    assert!(anchor.offset_right.is_finite());
    anchor
}

fn parse_slot_export(buf: &[u8], exp: &UassetExportEntry, names: &[String]) -> ParsedSlotLink {
    assert!(exp.uexp_offset <= buf.len());
    let end = (exp.uexp_offset + exp.serial_size).min(buf.len());
    let mut link = ParsedSlotLink {
        slot_export: exp.export_index,
        is_canvas_slot: exp.class_name == "CanvasPanelSlot",
        anchor: UmgAnchorData::ue4_default(),
        ..Default::default()
    };
    let mut pos = exp.uexp_offset;
    for _ in 0..64 {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        match (tag.name.as_str(), tag.prop_type.as_str()) {
            ("Parent", "ObjectProperty") if tag.size == 4 => {
                let idx = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4]);
                if idx > 0 {
                    link.parent_export = idx as usize;
                }
            }
            ("Content", "ObjectProperty") if tag.size == 4 => {
                let idx = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4]);
                if idx > 0 {
                    link.content_export = idx as usize;
                }
            }
            ("ZOrder", "IntProperty") if tag.size == 4 => {
                link.z_order = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4]);
            }
            ("bAutoSize", "BoolProperty") => {
                link.auto_size = tag.bool_val;
            }
            ("LayoutData", "StructProperty") => {
                link.anchor = parse_anchor_data_struct(buf, &tag, names);
            }
            _ => {}
        }
        pos = tag.next_offset;
    }
    assert!(pos <= end);
    link
}

fn parse_widget_transform_struct(
    buf: &[u8],
    tag: &PropertyTag,
    names: &[String],
    node: &mut UmgWidgetNode,
) {
    assert!(tag.value_offset <= tag.next_offset && tag.next_offset <= buf.len());
    let mut pos = tag.value_offset;
    for _ in 0..16 {
        if pos >= tag.next_offset {
            break;
        }
        let Some(Some(sub)) = read_property_tag(buf, pos, names) else {
            break;
        };
        match (sub.name.as_str(), sub.prop_type.as_str()) {
            ("Translation", "StructProperty") if sub.size == 8 => {
                node.render_translation[0] =
                    LittleEndian::read_f32(&buf[sub.value_offset..sub.value_offset + 4]);
                node.render_translation[1] =
                    LittleEndian::read_f32(&buf[sub.value_offset + 4..sub.value_offset + 8]);
            }
            ("Scale", "StructProperty") if sub.size == 8 => {
                node.render_scale[0] =
                    LittleEndian::read_f32(&buf[sub.value_offset..sub.value_offset + 4]);
                node.render_scale[1] =
                    LittleEndian::read_f32(&buf[sub.value_offset + 4..sub.value_offset + 8]);
            }
            ("Shear", "StructProperty") if sub.size == 8 => {
                node.render_shear[0] =
                    LittleEndian::read_f32(&buf[sub.value_offset..sub.value_offset + 4]);
                node.render_shear[1] =
                    LittleEndian::read_f32(&buf[sub.value_offset + 4..sub.value_offset + 8]);
            }
            ("Angle", "FloatProperty") if sub.size == 4 => {
                node.render_angle =
                    LittleEndian::read_f32(&buf[sub.value_offset..sub.value_offset + 4]);
            }
            _ => {}
        }
        pos = sub.next_offset;
    }
    assert!(node.render_scale[0].is_finite());
}

fn parse_slate_brush_struct(
    buf: &[u8],
    tag: &PropertyTag,
    names: &[String],
    imports: &[UassetImportEntry],
    node: &mut UmgWidgetNode,
) {
    assert!(tag.value_offset <= tag.next_offset && tag.next_offset <= buf.len());
    let mut pos = tag.value_offset;
    for _ in 0..32 {
        if pos >= tag.next_offset {
            break;
        }
        let Some(Some(sub)) = read_property_tag(buf, pos, names) else {
            break;
        };
        match (sub.name.as_str(), sub.prop_type.as_str()) {
            ("ImageSize", "StructProperty") if sub.size == 8 => {
                node.brush_image_size[0] =
                    LittleEndian::read_f32(&buf[sub.value_offset..sub.value_offset + 4]);
                node.brush_image_size[1] =
                    LittleEndian::read_f32(&buf[sub.value_offset + 4..sub.value_offset + 8]);
            }
            ("ResourceObject", "ObjectProperty") if sub.size == 4 => {
                let pidx = LittleEndian::read_i32(&buf[sub.value_offset..sub.value_offset + 4]);
                if node.soft_object_path.is_none()
                    && let Some(p) = resolve_package_index(imports, pidx)
                {
                    node.soft_object_path = Some(p);
                }
            }
            _ => {}
        }
        pos = sub.next_offset;
    }
    assert!(node.brush_image_size[0].is_finite());
}

fn parse_widget_export(
    buf: &[u8],
    exp: &UassetExportEntry,
    names: &[String],
    imports: &[UassetImportEntry],
) -> UmgWidgetNode {
    assert!(exp.uexp_offset <= buf.len());
    let end = (exp.uexp_offset + exp.serial_size).min(buf.len());
    let mut node = UmgWidgetNode {
        export_index: exp.export_index,
        name: exp.object_name.clone(),
        class_name: exp.class_name.clone(),
        ..Default::default()
    };
    let mut pos = exp.uexp_offset;
    for _ in 0..PROPS_ITER_MAX {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        match (tag.name.as_str(), tag.prop_type.as_str()) {
            ("Visibility", "EnumProperty" | "ByteProperty") if tag.size == 8 => {
                if let Some((v, _)) = read_fname_at(buf, tag.value_offset, names) {
                    node.visibility = v.rsplit("::").next().unwrap_or(&v).to_string();
                }
            }
            ("Clipping", "EnumProperty" | "ByteProperty") if tag.size == 8 => {
                if let Some((c, _)) = read_fname_at(buf, tag.value_offset, names) {
                    node.clipping = c.rsplit("::").next().unwrap_or(&c).to_string();
                }
            }
            ("RenderOpacity", "FloatProperty") if tag.size == 4 => {
                node.render_opacity =
                    LittleEndian::read_f32(&buf[tag.value_offset..tag.value_offset + 4]);
            }
            ("RenderTransform", "StructProperty") => {
                parse_widget_transform_struct(buf, &tag, names, &mut node);
            }
            ("ColorAndOpacity", "StructProperty") if tag.size == 16 => {
                for (i, dst) in node.color_and_opacity.iter_mut().enumerate() {
                    let o = tag.value_offset + i * 4;
                    *dst = LittleEndian::read_f32(&buf[o..o + 4]);
                }
            }
            ("Brush", "StructProperty") => {
                parse_slate_brush_struct(buf, &tag, names, imports, &mut node);
            }
            ("WidthOverride", "FloatProperty") if tag.size == 4 => {
                node.brush_image_size[0] =
                    LittleEndian::read_f32(&buf[tag.value_offset..tag.value_offset + 4]);
            }
            ("HeightOverride", "FloatProperty") if tag.size == 4 => {
                node.brush_image_size[1] =
                    LittleEndian::read_f32(&buf[tag.value_offset..tag.value_offset + 4]);
            }
            ("SoftObjectPath" | "ParticleAsset", "StructProperty") if tag.size >= 8 => {
                if let Some((asset_name, _)) = read_fname_at(buf, tag.value_offset, names)
                    && asset_name != "None"
                {
                    let stripped = strip_subobject_suffix(&asset_name);
                    if tag.name == "ParticleAsset" {
                        node.particle_asset_path = Some(stripped.clone());
                    }
                    node.soft_object_path = Some(stripped);
                }
            }
            ("Asset" | "ParticleAsset", "ObjectProperty") if tag.size == 4 => {
                let pidx = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4]);
                if let Some(p) = resolve_package_index(imports, pidx) {
                    node.particle_asset_path = Some(p.clone());
                    if node.soft_object_path.is_none() {
                        node.soft_object_path = Some(p);
                    }
                }
            }
            _ => {}
        }
        pos = tag.next_offset;
    }
    assert!(node.export_index > 0);
    node
}

fn parse_panel_slot_orders(
    buf: &[u8],
    exports: &[UassetExportEntry],
    names: &[String],
) -> BTreeMap<usize, usize> {
    assert!(buf.len() <= UEXP_BYTES_MAX);
    assert!(exports.len() <= 65_536);
    let mut slot_orders = BTreeMap::new();
    for exp in exports {
        if !is_widget_class(&exp.class_name) {
            continue;
        }
        let end = (exp.uexp_offset + exp.serial_size).min(buf.len());
        let mut pos = exp.uexp_offset;
        for _ in 0..PROPS_ITER_MAX {
            if pos >= end {
                break;
            }
            let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
                break;
            };
            if tag.name == "Slots" && tag.prop_type == "ArrayProperty" && tag.size >= 4 {
                let count = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4])
                    .clamp(0, 4096) as usize;
                for order_idx in 0..count {
                    let o = tag.value_offset + 4 + order_idx * 4;
                    if let Some(s) = buf.get(o..o + 4) {
                        let slot_pidx = LittleEndian::read_i32(s);
                        if slot_pidx > 0 {
                            slot_orders.insert(slot_pidx as usize, order_idx);
                        }
                    }
                }
            }
            pos = tag.next_offset;
        }
    }
    assert!(slot_orders.len() <= SLOTS_COUNT_MAX);
    slot_orders
}

fn eval_float_channel_t0(buf: &[u8], start: usize, end: usize, names: &[String]) -> Option<f32> {
    assert!(start <= end && end <= buf.len());
    let mut pos = start;
    let mut default_val = None;
    let mut first_key_val = None;
    for _ in 0..32 {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        match (tag.name.as_str(), tag.prop_type.as_str()) {
            ("DefaultValue", "FloatProperty") if tag.size == 4 => {
                let v = LittleEndian::read_f32(&buf[tag.value_offset..tag.value_offset + 4]);
                if v.is_finite() && v.abs() < 1e10 {
                    default_val = Some(v);
                }
            }
            ("Values", "ArrayProperty") if tag.size >= 8 => {
                let count = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4]);
                if count > 0
                    && let Some(Some(proto)) = read_property_tag(buf, tag.value_offset + 4, names)
                    && proto.value_offset + 4 <= tag.next_offset
                {
                    let v =
                        LittleEndian::read_f32(&buf[proto.value_offset..proto.value_offset + 4]);
                    if v.is_finite() {
                        first_key_val = Some(v);
                    }
                }
            }
            _ => {}
        }
        pos = tag.next_offset;
    }
    first_key_val.or(default_val)
}

fn parse_float_section_t0(buf: &[u8], exp: &UassetExportEntry, names: &[String]) -> Option<f32> {
    assert!(exp.uexp_offset <= buf.len());
    let end = (exp.uexp_offset + exp.serial_size).min(buf.len());
    let mut pos = exp.uexp_offset;
    for _ in 0..32 {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        if tag.name == "FloatCurve" && tag.prop_type == "StructProperty" {
            return eval_float_channel_t0(buf, tag.value_offset, tag.next_offset, names);
        }
        pos = tag.next_offset;
    }
    None
}

fn parse_param_section_t0(
    buf: &[u8],
    exp: &UassetExportEntry,
    names: &[String],
) -> Vec<(String, f32)> {
    assert!(exp.uexp_offset <= buf.len());
    let end = (exp.uexp_offset + exp.serial_size).min(buf.len());
    let mut out = Vec::new();
    let mut pos = exp.uexp_offset;
    for _ in 0..32 {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        if tag.name == "ScalarParameterNamesAndCurves"
            && tag.prop_type == "ArrayProperty"
            && tag.size >= 4
        {
            let count = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4])
                .clamp(0, 64) as usize;
            if let Some(Some(proto)) = read_property_tag(buf, tag.value_offset + 4, names) {
                let mut epos = proto.value_offset;
                for _ in 0..count {
                    let mut pname = String::new();
                    let mut pval = None;
                    for _ in 0..16 {
                        if epos >= tag.next_offset {
                            break;
                        }
                        let Some(tag_opt) = read_property_tag(buf, epos, names) else {
                            break;
                        };
                        let Some(sub) = tag_opt else {
                            epos += 8;
                            break;
                        };
                        if sub.name == "ParameterName" && sub.size == 8 {
                            if let Some((n, _)) = read_fname_at(buf, sub.value_offset, names) {
                                pname = n;
                            }
                        } else if sub.name == "ParameterCurve" {
                            pval = eval_float_channel_t0(
                                buf,
                                sub.value_offset,
                                sub.next_offset,
                                names,
                            );
                        }
                        epos = sub.next_offset;
                    }
                    if !pname.is_empty()
                        && let Some(v) = pval
                    {
                        out.push((pname, v));
                    }
                }
            }
        }
        pos = tag.next_offset;
    }
    assert!(out.len() <= 64);
    out
}

fn parse_track_sections(
    buf: &[u8],
    exp: &UassetExportEntry,
    names: &[String],
) -> (String, Vec<usize>) {
    assert!(exp.uexp_offset <= buf.len());
    let end = (exp.uexp_offset + exp.serial_size).min(buf.len());
    let mut prop_name = String::new();
    let mut sections = Vec::new();
    let mut pos = exp.uexp_offset;
    for _ in 0..32 {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        match (tag.name.as_str(), tag.prop_type.as_str()) {
            ("PropertyName", "NameProperty") if tag.size == 8 => {
                if let Some((n, _)) = read_fname_at(buf, tag.value_offset, names) {
                    prop_name = n;
                }
            }
            ("Sections", "ArrayProperty") if tag.size >= 4 => {
                let count = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4])
                    .clamp(0, 64) as usize;
                for k in 0..count {
                    let o = tag.value_offset + 4 + k * 4;
                    if let Some(s) = buf.get(o..o + 4) {
                        let idx = LittleEndian::read_i32(s);
                        if idx > 0 {
                            sections.push(idx as usize);
                        }
                    }
                }
            }
            _ => {}
        }
        pos = tag.next_offset;
    }
    assert!(sections.len() <= 64);
    (prop_name, sections)
}

fn parse_moviescene_bindings(
    buf: &[u8],
    exp: &UassetExportEntry,
    names: &[String],
) -> BTreeMap<[u8; 16], Vec<usize>> {
    assert!(exp.uexp_offset <= buf.len());
    let end = (exp.uexp_offset + exp.serial_size).min(buf.len());
    let mut map = BTreeMap::new();
    let mut pos = exp.uexp_offset;
    for _ in 0..64 {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        if tag.name == "ObjectBindings" && tag.prop_type == "ArrayProperty" && tag.size >= 4 {
            let count = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4])
                .clamp(0, 256) as usize;
            if let Some(Some(proto)) = read_property_tag(buf, tag.value_offset + 4, names) {
                let mut epos = proto.value_offset;
                for _ in 0..count {
                    let mut guid = [0u8; 16];
                    let mut tracks = Vec::new();
                    for _ in 0..16 {
                        if epos >= tag.next_offset {
                            break;
                        }
                        let Some(tag_opt) = read_property_tag(buf, epos, names) else {
                            break;
                        };
                        let Some(sub) = tag_opt else {
                            epos += 8;
                            break;
                        };
                        if sub.name == "ObjectGuid" && sub.size == 16 {
                            guid.copy_from_slice(&buf[sub.value_offset..sub.value_offset + 16]);
                        } else if sub.name == "Tracks" && sub.size >= 4 {
                            let tc = LittleEndian::read_i32(
                                &buf[sub.value_offset..sub.value_offset + 4],
                            )
                            .clamp(0, 64) as usize;
                            for k in 0..tc {
                                let o = sub.value_offset + 4 + k * 4;
                                if let Some(s) = buf.get(o..o + 4) {
                                    let idx = LittleEndian::read_i32(s);
                                    if idx > 0 {
                                        tracks.push(idx as usize);
                                    }
                                }
                            }
                        }
                        epos = sub.next_offset;
                    }
                    map.insert(guid, tracks);
                }
            }
        }
        pos = tag.next_offset;
    }
    assert!(map.len() <= 256);
    map
}

fn parse_widget_animation_bindings(
    buf: &[u8],
    exp: &UassetExportEntry,
    names: &[String],
) -> (usize, BTreeMap<[u8; 16], String>) {
    assert!(exp.uexp_offset <= buf.len());
    let end = (exp.uexp_offset + exp.serial_size).min(buf.len());
    let mut movie_scene_idx = 0usize;
    let mut guid_to_widget = BTreeMap::new();
    let mut pos = exp.uexp_offset;
    for _ in 0..32 {
        if pos >= end {
            break;
        }
        let Some(Some(tag)) = read_property_tag(buf, pos, names) else {
            break;
        };
        match (tag.name.as_str(), tag.prop_type.as_str()) {
            ("MovieScene", "ObjectProperty") if tag.size == 4 => {
                let idx = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4]);
                if idx > 0 {
                    movie_scene_idx = idx as usize;
                }
            }
            ("AnimationBindings", "ArrayProperty") if tag.size >= 4 => {
                let count = LittleEndian::read_i32(&buf[tag.value_offset..tag.value_offset + 4])
                    .clamp(0, 256) as usize;
                if let Some(Some(proto)) = read_property_tag(buf, tag.value_offset + 4, names) {
                    let mut epos = proto.value_offset;
                    for _ in 0..count {
                        let mut wname = String::new();
                        let mut guid = [0u8; 16];
                        for _ in 0..16 {
                            if epos >= tag.next_offset {
                                break;
                            }
                            let Some(tag_opt) = read_property_tag(buf, epos, names) else {
                                break;
                            };
                            let Some(sub) = tag_opt else {
                                epos += 8;
                                break;
                            };
                            if sub.name == "WidgetName" && sub.size == 8 {
                                if let Some((n, _)) = read_fname_at(buf, sub.value_offset, names) {
                                    wname = n;
                                }
                            } else if sub.name == "AnimationGuid" && sub.size == 16 {
                                guid.copy_from_slice(&buf[sub.value_offset..sub.value_offset + 16]);
                            }
                            epos = sub.next_offset;
                        }
                        if !wname.is_empty() {
                            guid_to_widget.insert(guid, wname);
                        }
                    }
                }
            }
            _ => {}
        }
        pos = tag.next_offset;
    }
    assert!(guid_to_widget.len() <= 256);
    (movie_scene_idx, guid_to_widget)
}

fn evaluate_widget_animations_t0(
    uexp: &[u8],
    exports: &[UassetExportEntry],
    names: &[String],
) -> Vec<UmgAnimationStateT0> {
    assert!(uexp.len() <= UEXP_BYTES_MAX);
    assert!(exports.len() <= 65_536);
    let mut out = Vec::new();
    for exp in exports {
        if exp.class_name != "WidgetAnimation" {
            continue;
        }
        let (ms_idx, guid_to_widget) = parse_widget_animation_bindings(uexp, exp, names);
        let Some(ms_exp) = ms_idx.checked_sub(1).and_then(|i| exports.get(i)) else {
            continue;
        };
        let obj_bindings = parse_moviescene_bindings(uexp, ms_exp, names);
        let mut anim = UmgAnimationStateT0 {
            name: exp.object_name.clone(),
            ..Default::default()
        };
        for (guid, widget_name) in &guid_to_widget {
            let Some(track_indices) = obj_bindings.get(guid) else {
                continue;
            };
            for &t_idx in track_indices {
                let Some(t_exp) = t_idx.checked_sub(1).and_then(|i| exports.get(i)) else {
                    continue;
                };
                let (prop_name, sec_indices) = parse_track_sections(uexp, t_exp, names);
                for s_idx in sec_indices {
                    let Some(s_exp) = s_idx.checked_sub(1).and_then(|i| exports.get(i)) else {
                        continue;
                    };
                    if t_exp.class_name == "MovieSceneFloatTrack" && prop_name == "RenderOpacity" {
                        if let Some(val) = parse_float_section_t0(uexp, s_exp, names) {
                            anim.render_opacities.insert(widget_name.clone(), val);
                        }
                    } else if t_exp.class_name == "MovieSceneWidgetMaterialTrack" {
                        let params = parse_param_section_t0(uexp, s_exp, names);
                        if !params.is_empty() {
                            anim.material_scalars
                                .entry(widget_name.clone())
                                .or_default()
                                .extend(params);
                        }
                    }
                }
            }
        }
        out.push(anim);
    }
    assert!(out.len() <= exports.len());
    out
}

fn is_widget_class(class_name: &str) -> bool {
    assert!(class_name.len() <= 256);
    !class_name.is_empty()
        && !class_name.ends_with("Slot")
        && !class_name.starts_with("MovieScene")
        && !matches!(
            class_name,
            "WidgetBlueprintGeneratedClass"
                | "WidgetTree"
                | "WidgetAnimation"
                | "ComponentDelegateBinding"
        )
}

/// Parses a cooked UE 4.27 UMG Widget Blueprint (`.uasset` + `.uexp`) into its widget names,
/// referenced `/Game/...` asset paths, `UCanvasPanelSlot` anchor/offset slots, full widget
/// hierarchy (`UmgWidgetNode`), and `T = 0` `UWidgetAnimation` states.
pub fn parse_umg_blueprint(uasset: &[u8], uexp: &[u8]) -> Result<UmgBlueprint, ExtractError> {
    assert!(uasset.len() <= UASSET_BYTES_MAX);
    assert!(uexp.len() <= UEXP_BYTES_MAX);
    if uasset.len() < 8 || LittleEndian::read_u32(uasset) != PACKAGE_FILE_TAG {
        return Err(ExtractError::UassetFormat(
            "Invalid UMG .uasset package tag".to_string(),
        ));
    }
    let names = parse_uasset_names(uasset);
    if names.is_empty() {
        return Err(ExtractError::UassetFormat(
            "Empty FName table in UMG .uasset".to_string(),
        ));
    }
    let imports = parse_uasset_imports(uasset, &names);
    let exports = parse_uasset_exports(uasset, &names, &imports);

    let slot_orders = parse_panel_slot_orders(uexp, &exports, &names);
    let mut slot_by_content: BTreeMap<usize, ParsedSlotLink> = BTreeMap::new();
    let mut anchor_slots = Vec::new();
    for exp in &exports {
        if exp.class_name.ends_with("Slot") {
            let link = parse_slot_export(uexp, exp, &names);
            if link.is_canvas_slot {
                anchor_slots.push(link.anchor);
            }
            if link.content_export > 0 {
                slot_by_content.insert(link.content_export, link);
            }
        }
    }

    let mut widgets = Vec::new();
    for exp in &exports {
        if is_widget_class(&exp.class_name) {
            let mut node = parse_widget_export(uexp, exp, &names, &imports);
            node.slot_order = exp.export_index;
            if let Some(link) = slot_by_content.get(&exp.export_index) {
                node.parent_export_index = link.parent_export;
                node.z_order = link.z_order;
                node.slot_order = slot_orders
                    .get(&link.slot_export)
                    .copied()
                    .unwrap_or(link.slot_export);
                node.anchor = link.anchor;
                node.auto_size = link.auto_size;
                node.has_canvas_slot = link.is_canvas_slot;
            }
            widgets.push(node);
        }
    }

    for i in 0..widgets.len() {
        if widgets[i].class_name.ends_with("SizeBox") && widgets[i].brush_image_size == [32.0, 32.0]
        {
            let exp_idx = widgets[i].export_index;
            if let Some(child_size) = widgets
                .iter()
                .find(|c| c.parent_export_index == exp_idx)
                .map(|c| c.brush_image_size)
            {
                widgets[i].brush_image_size = child_size;
            }
        }
    }

    let mut widget_names = Vec::new();
    let mut referenced_assets = Vec::new();
    for name in &names {
        if name.starts_with("/Game/") {
            referenced_assets.push(name.clone());
        } else if name.starts_with("S16")
            || name.starts_with("Img_")
            || name.starts_with("Txt_")
            || name.starts_with("Btn_")
            || name.starts_with("Panel_")
            || name.starts_with("List_")
        {
            widget_names.push(name.clone());
        }
    }
    let animations_t0 = evaluate_widget_animations_t0(uexp, &exports, &names);
    assert!(anchor_slots.len() <= SLOTS_COUNT_MAX);
    Ok(UmgBlueprint {
        widget_names,
        referenced_assets,
        anchor_slots,
        widgets,
        animations_t0,
    })
}

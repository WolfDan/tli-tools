//! Cooked Unreal Engine 4.27 UMG Widget Blueprint (`.uasset` + `.uexp`) layout extractor.

use byteorder::{ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};

use crate::error::ExtractError;
use crate::uasset::{PACKAGE_FILE_TAG, parse_uasset_names};

const UASSET_BYTES_MAX: usize = 256 * 1024 * 1024;
const UEXP_BYTES_MAX: usize = 256 * 1024 * 1024;
const SLOTS_COUNT_MAX: usize = 4096;

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

/// Extracted widget export metadata and `UCanvasPanelSlot` layout from a UMG Blueprint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UmgBlueprint {
    pub widget_names: Vec<String>,
    pub referenced_assets: Vec<String>,
    pub anchor_slots: Vec<UmgAnchorData>,
}

fn find_struct_prop_indices(names: &[String]) -> Option<(i32, i32, i32, i32, i32, i32, i32)> {
    assert!(names.len() <= 65_536);
    let struct_idx = names.iter().position(|n| n == "StructProperty")? as i32;
    let offsets_idx = names
        .iter()
        .position(|n| n == "Offsets")
        .map_or(-1, |i| i as i32);
    let margin_idx = names
        .iter()
        .position(|n| n == "Margin")
        .map_or(-1, |i| i as i32);
    let idx_min = names
        .iter()
        .position(|n| n == "Minimum")
        .map_or(-1, |i| i as i32);
    let idx_max = names
        .iter()
        .position(|n| n == "Maximum")
        .map_or(-1, |i| i as i32);
    let align_idx = names
        .iter()
        .position(|n| n == "Alignment")
        .map_or(-1, |i| i as i32);
    let vec2d_idx = names
        .iter()
        .position(|n| n == "Vector2D")
        .map_or(-1, |i| i as i32);
    assert!(struct_idx >= 0);
    Some((
        struct_idx,
        offsets_idx,
        margin_idx,
        idx_min,
        idx_max,
        align_idx,
        vec2d_idx,
    ))
}

fn read_vec2d_in_window(
    window: &[u8],
    prop_idx: i32,
    struct_idx: i32,
    vec2d_idx: i32,
) -> Option<(f32, f32)> {
    assert!(window.len() <= UEXP_BYTES_MAX);
    if prop_idx < 0 || vec2d_idx < 0 {
        return None;
    }
    let mut pat = [0u8; 32];
    LittleEndian::write_i32(&mut pat[0..4], prop_idx);
    LittleEndian::write_i32(&mut pat[8..12], struct_idx);
    LittleEndian::write_i32(&mut pat[16..20], 8);
    LittleEndian::write_i32(&mut pat[24..28], vec2d_idx);
    let pos = memchr::memmem::find(window, &pat)?;
    let val_off = pos + 32 + 17;
    if val_off + 8 > window.len() {
        return None;
    }
    let x = LittleEndian::read_f32(&window[val_off..val_off + 4]);
    let y = LittleEndian::read_f32(&window[val_off + 4..val_off + 8]);
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    assert!(x.is_finite() && y.is_finite());
    Some((x, y))
}

fn scan_anchor_slots_in_uexp(uexp: &[u8], names: &[String]) -> Vec<UmgAnchorData> {
    assert!(uexp.len() <= UEXP_BYTES_MAX);
    assert!(names.len() <= 65_536);
    let Some((struct_idx, offsets_idx, margin_idx, idx_min, idx_max, align_idx, vec2d_idx)) =
        find_struct_prop_indices(names)
    else {
        return Vec::new();
    };
    if offsets_idx < 0 || margin_idx < 0 {
        return Vec::new();
    }

    let mut pat = [0u8; 32];
    LittleEndian::write_i32(&mut pat[0..4], offsets_idx);
    LittleEndian::write_i32(&mut pat[8..12], struct_idx);
    LittleEndian::write_i32(&mut pat[16..20], 16);
    LittleEndian::write_i32(&mut pat[24..28], margin_idx);

    let mut slots = Vec::new();
    let mut iterations = 0usize;
    for pos in memchr::memmem::find_iter(uexp, &pat) {
        iterations += 1;
        assert!(iterations <= SLOTS_COUNT_MAX);
        if slots.len() >= SLOTS_COUNT_MAX {
            break;
        }
        let val_off = pos + 32 + 17;
        if val_off + 16 > uexp.len() {
            continue;
        }
        let left = LittleEndian::read_f32(&uexp[val_off..val_off + 4]);
        let top = LittleEndian::read_f32(&uexp[val_off + 4..val_off + 8]);
        let right = LittleEndian::read_f32(&uexp[val_off + 8..val_off + 12]);
        let bottom = LittleEndian::read_f32(&uexp[val_off + 12..val_off + 16]);
        if !left.is_finite() || !top.is_finite() || !right.is_finite() || !bottom.is_finite() {
            continue;
        }
        let win_end = (val_off + 512).min(uexp.len());
        let window = &uexp[val_off..win_end];
        let (anchor_min_x, anchor_min_y) =
            read_vec2d_in_window(window, idx_min, struct_idx, vec2d_idx).unwrap_or((0.0, 0.0));
        let (anchor_max_x, anchor_max_y) =
            read_vec2d_in_window(window, idx_max, struct_idx, vec2d_idx).unwrap_or((0.0, 0.0));
        let (align_x, align_y) =
            read_vec2d_in_window(window, align_idx, struct_idx, vec2d_idx).unwrap_or((0.0, 0.0));
        slots.push(UmgAnchorData {
            offset_left: left,
            offset_top: top,
            offset_right: right,
            offset_bottom: bottom,
            anchor_min_x,
            anchor_min_y,
            anchor_max_x,
            anchor_max_y,
            align_x,
            align_y,
        });
    }
    assert!(slots.len() <= SLOTS_COUNT_MAX);
    slots
}

/// Parses a cooked UE 4.27 UMG Widget Blueprint (`.uasset` + `.uexp`) into its widget names,
/// referenced `/Game/...` asset paths, and `UCanvasPanelSlot` anchor/offset slots.
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
    let anchor_slots = scan_anchor_slots_in_uexp(uexp, &names);
    assert!(anchor_slots.len() <= SLOTS_COUNT_MAX);
    Ok(UmgBlueprint {
        widget_names,
        referenced_assets,
        anchor_slots,
    })
}

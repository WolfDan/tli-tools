//! Static `.pak`-synthesized UMG slot geometries and `#[repr(C, align(64))]` SIMD templates.

use glam::{DMat3, DVec2, DVec3};
use tartarus_core::{GiftId, ToolId};

pub const H0: [f64; 9] = [
    119.44981,
    -9.9567366,
    358.07993,
    -0.26960321,
    90.497559,
    136.75498,
    -0.00012281694,
    -0.015743494,
    1.0,
];

pub const PITCH0: f64 = 119.45;

pub const RING: [(u8, u8); 16] = [
    (0, 0),
    (1, 0),
    (2, 0),
    (3, 0),
    (4, 0),
    (5, 0),
    (5, 1),
    (5, 2),
    (5, 3),
    (4, 3),
    (3, 3),
    (2, 3),
    (1, 3),
    (0, 3),
    (0, 2),
    (0, 1),
];

pub const ARROW: [(f64, f64); 16] = [
    (0.56, 0.2),
    (1.0, 0.25),
    (2.0, 0.25),
    (3.0, 0.25),
    (4.0, 0.25),
    (4.44, 0.2),
    (4.42, 0.57),
    (4.42, 1.57),
    (4.45, 1.95),
    (4.0, 1.9),
    (3.0, 1.9),
    (2.0, 1.9),
    (1.0, 1.9),
    (0.55, 1.95),
    (0.58, 1.57),
    (0.58, 0.57),
];

pub const FACE_GX: usize = 16;
pub const FACE_GY: usize = 16;
pub const FACE_X0: f64 = -16.0;
pub const FACE_X1: f64 = 16.0;
pub const FACE_Y0: f64 = -58.0;
pub const FACE_Y1: f64 = -20.0;

pub const GLYPH_W: usize = 10;
pub const GLYPH_H: usize = 16;
pub const PREFIX_W: usize = 40;
pub const PREFIX_H: usize = 10;
pub const PREFIX_END: f64 = 5.46;
pub const PREFIX_NUM: f64 = 5.66;

pub const CARD_FRAME: f64 = 0.02;
pub const CARD_LIFT: f64 = 0.045;

include!("../ui_data.rs");

const _: () = {
    assert!(std::mem::align_of::<SimdVec160>() == 64);
    assert!(std::mem::align_of::<SimdVec256>() == 64);
    assert!(std::mem::align_of::<SimdVec400>() == 64);
    assert!(std::mem::align_of::<SimdVec576>() == 64);
    assert!(std::mem::align_of::<RefToolEntry>() == 64);
    assert!(std::mem::align_of::<RefPickToolEntry>() == 64);
    assert!(std::mem::align_of::<RefGiftEntry>() == 64);
    assert!(RING.len() == 16);
    assert!(ARROW.len() == 16);
    assert!(FACE_GX * FACE_GY == 256);
    assert!(GLYPH_W * GLYPH_H == 160);
    assert!(PREFIX_W * PREFIX_H == 400);
};

#[inline]
pub(crate) fn invert_3x3(h: &[f64; 9]) -> [f64; 9] {
    debug_assert_eq!(h.len(), 9);
    let mat = DMat3::from_cols_array(h).transpose();
    let inv = mat.inverse();
    let out = inv.transpose().to_cols_array();
    debug_assert!(out[0].is_finite());
    out
}

#[inline]
pub(crate) fn project_3x3(h: &[f64; 9], u: f64, v: f64) -> (f64, f64) {
    debug_assert!(u.is_finite());
    debug_assert!(v.is_finite());
    let mat = DMat3::from_cols_array(h).transpose();
    let p = mat * DVec3::new(u, v, 1.0);
    let res = DVec2::new(p.x / p.z, p.y / p.z);
    (res.x, res.y)
}

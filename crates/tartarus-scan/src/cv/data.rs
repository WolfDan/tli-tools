//! Reference templates, geometry constants, and precomputed computer vision tables.

use tartarus_core::ToolId;

include!("../cv_data.rs");

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

pub const PITCH0: f64 = 119.45;

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

pub const FACE_GX: usize = 18;
pub const FACE_GY: usize = 22;
pub const FACE_X0: f64 = -16.0;
pub const FACE_X1: f64 = 16.0;
pub const FACE_Y0: f64 = -58.0;
pub const FACE_Y1: f64 = -20.0;

pub const GLYPH_W: usize = 10;
pub const GLYPH_H: usize = 14;

pub const PREFIX_W: usize = 40;
pub const PREFIX_H: usize = 10;
pub const PREFIX_END: f64 = 5.46;
pub const PREFIX_NUM: f64 = 5.66;

pub const CARD_FRAME: f64 = 0.02;
pub const CARD_LIFT: f64 = 0.045;

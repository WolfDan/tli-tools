//! TrueType (`.ufont`) parser and anti-aliased glyph rasterizer via `ab_glyph`.

use crate::error::DataError;

pub struct TtfFont {
    pub font: ab_glyph::FontVec,
    pub units_per_em: u16,
}

pub struct RasterizedGlyph {
    pub cw: usize,
    pub ch_px: usize,
    pub blur: Vec<f32>,
    pub bx0: f64,
    pub by0: f64,
    pub bw: f64,
    pub bh: f64,
}

pub type BoardPoint = (i32, i32, f32);
pub type RoomBox = (u16, u16, u16, u16);
pub type DigitTemplates = [Vec<[f32; 140]>; 10];
pub type PrefixTemplates = Vec<[f32; 400]>;

fn box_blur_3x3(raw: &[f32], cw: usize, ch_px: usize) -> Vec<f32> {
    let mut blur = vec![30.0f32; cw * ch_px];
    for y in 0..ch_px {
        for x in 0..cw {
            let mut s = 0.0f32;
            for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    let yy = (y as isize + dy).clamp(0, ch_px as isize - 1) as usize;
                    let xx = (x as isize + dx).clamp(0, cw as isize - 1) as usize;
                    s += raw[yy * cw + xx];
                }
            }
            blur[y * cw + x] = s / 9.0;
        }
    }
    blur
}

impl TtfFont {
    pub fn parse(data: Vec<u8>) -> Result<Self, DataError> {
        use ab_glyph::Font;
        let font = ab_glyph::FontVec::try_from_vec(data)
            .map_err(|e| DataError::Font(format!("Invalid TrueType font: {e}")))?;
        let units_per_em = font.units_per_em().unwrap_or(1000.0).round() as u16;
        Ok(Self { font, units_per_em })
    }

    /// Rasterizes a single glyph into a padded anti-aliased + `3x3`-box-blurred grayscale buffer,
    /// returning `RasterizedGlyph`.
    pub fn rasterize_char(&self, ch: char, target_h: usize, pad: usize) -> Option<RasterizedGlyph> {
        use ab_glyph::Font;
        let gid = self.font.glyph_id(ch);
        if gid.0 == 0 {
            return None;
        }
        let ref_outlined = self.font.outline_glyph(gid.with_scale(100.0))?;
        let ref_bounds = ref_outlined.px_bounds();
        let ref_h = ref_bounds.height().max(1.0);
        let px_scale = 100.0 * (target_h as f32) / ref_h;
        let outlined = self.font.outline_glyph(gid.with_scale(px_scale))?;
        let bounds = outlined.px_bounds();
        let gw = (bounds.width().ceil() as usize).max(1);
        let gh = target_h;
        let cw = gw + 2 * pad;
        let ch_px = gh + 2 * pad;
        let mut raw = vec![30.0f32; cw * ch_px];

        outlined.draw(|px, py, cov| {
            let x = px as usize + pad;
            let y = py as usize + pad;
            if x < cw && y < ch_px {
                raw[y * cw + x] = 30.0 + 215.0 * cov;
            }
        });
        let blur = box_blur_3x3(&raw, cw, ch_px);
        Some(RasterizedGlyph {
            cw,
            ch_px,
            blur,
            bx0: pad as f64,
            by0: pad as f64,
            bw: bounds.width() as f64,
            bh: gh as f64,
        })
    }

    /// Rasterizes a multi-character text string (such as `"Go to Cell"`) into a padded blurred canvas.
    pub fn rasterize_text(&self, text: &str, scale: f32, pad: usize) -> Option<RasterizedGlyph> {
        use ab_glyph::{Font, ScaleFont, point};
        let px_scale = scale * self.font.height_unscaled();
        let scaled = self.font.as_scaled(px_scale);
        let mut pen_x = 0.0f32;
        let mut outlined_glyphs = Vec::new();
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for ch in text.chars() {
            let gid = scaled.glyph_id(ch);
            let glyph = gid.with_scale_and_position(px_scale, point(pen_x, 0.0));
            if let Some(outlined) = self.font.outline_glyph(glyph) {
                let b = outlined.px_bounds();
                min_x = min_x.min(b.min.x);
                max_x = max_x.max(b.max.x);
                min_y = min_y.min(b.min.y);
                max_y = max_y.max(b.max.y);
                outlined_glyphs.push(outlined);
            }
            pen_x += scaled.h_advance(gid);
        }
        if outlined_glyphs.is_empty() {
            return None;
        }
        let pw = (max_x - min_x).ceil().max(1.0) as usize;
        let ph = (max_y - min_y).ceil().max(1.0) as usize;
        let cw = pw + 2 * pad;
        let ch_px = ph + 2 * pad;
        let mut raw = vec![30.0f32; cw * ch_px];

        for outlined in &outlined_glyphs {
            let b = outlined.px_bounds();
            let ox = (b.min.x - min_x).round() as isize + pad as isize;
            let oy = (b.min.y - min_y).round() as isize + pad as isize;
            outlined.draw(|px, py, cov| {
                let x = ox + px as isize;
                let y = oy + py as isize;
                if x >= 0 && (x as usize) < cw && y >= 0 && (y as usize) < ch_px {
                    let idx = (y as usize) * cw + (x as usize);
                    raw[idx] = raw[idx].max(30.0 + 215.0 * cov);
                }
            });
        }
        let blur = box_blur_3x3(&raw, cw, ch_px);
        Some(RasterizedGlyph {
            cw,
            ch_px,
            blur,
            bx0: pad as f64,
            by0: pad as f64,
            bw: (max_x - min_x) as f64,
            bh: (max_y - min_y) as f64,
        })
    }
}

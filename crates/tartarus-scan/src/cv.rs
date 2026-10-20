use tartarus_core::{Beast, FixedVec, ObjectiveId, StartEventId, ToolId};

include!("cv_data.rs");

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

#[inline]
pub fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    use std::simd::prelude::*;
    use std::simd::StdFloat;
    let n = a.len().min(b.len());
    let mut acc0 = f32x8::splat(0.0);
    let mut acc1 = f32x8::splat(0.0);
    let mut i = 0usize;
    while i + 16 <= n {
        let va0 = f32x8::from_slice(&a[i..i + 8]);
        let vb0 = f32x8::from_slice(&b[i..i + 8]);
        let va1 = f32x8::from_slice(&a[i + 8..i + 16]);
        let vb1 = f32x8::from_slice(&b[i + 8..i + 16]);
        acc0 = va0.mul_add(vb0, acc0);
        acc1 = va1.mul_add(vb1, acc1);
        i += 16;
    }
    if i + 8 <= n {
        let va = f32x8::from_slice(&a[i..i + 8]);
        let vb = f32x8::from_slice(&b[i..i + 8]);
        acc0 = va.mul_add(vb, acc0);
        i += 8;
    }
    let mut sum = (acc0 + acc1).reduce_sum();
    while i < n {
        sum += a[i] * b[i];
        i += 1;
    }
    sum
}

#[inline]
fn normalize_zero_mean_in_place(v: &mut [f32], mean: f32) {
    use std::simd::prelude::*;
    use std::simd::StdFloat;
    let n = v.len();
    let vmean = f32x8::splat(mean);
    let mut nacc0 = f32x8::splat(0.0);
    let mut nacc1 = f32x8::splat(0.0);
    let mut i = 0usize;
    while i + 16 <= n {
        let x0 = f32x8::from_slice(&v[i..i + 8]) - vmean;
        let x1 = f32x8::from_slice(&v[i + 8..i + 16]) - vmean;
        x0.copy_to_slice(&mut v[i..i + 8]);
        x1.copy_to_slice(&mut v[i + 8..i + 16]);
        nacc0 = x0.mul_add(x0, nacc0);
        nacc1 = x1.mul_add(x1, nacc1);
        i += 16;
    }
    let mut norm = (nacc0 + nacc1).reduce_sum();
    while i < n {
        let x = v[i] - mean;
        v[i] = x;
        norm += x * x;
        i += 1;
    }
    let inv_norm = 1.0_f32 / norm.sqrt().max(1.0);
    let vinv = f32x8::splat(inv_norm);
    let mut j = 0usize;
    while j + 16 <= n {
        let x0 = f32x8::from_slice(&v[j..j + 8]) * vinv;
        let x1 = f32x8::from_slice(&v[j + 8..j + 16]) * vinv;
        x0.copy_to_slice(&mut v[j..j + 8]);
        x1.copy_to_slice(&mut v[j + 8..j + 16]);
        j += 16;
    }
    while j < n {
        v[j] *= inv_norm;
        j += 1;
    }
}

#[inline]
fn cosine_simd_impl(a: &[f32], b: &[f32]) -> f32 {
    use std::simd::prelude::*;
    use std::simd::StdFloat;
    let n = a.len().min(b.len());
    let mut v_ab = f32x8::splat(0.0);
    let mut v_aa = f32x8::splat(0.0);
    let mut v_bb = f32x8::splat(0.0);
    let mut i = 0usize;
    while i + 8 <= n {
        let va = f32x8::from_slice(&a[i..i + 8]);
        let vb = f32x8::from_slice(&b[i..i + 8]);
        v_ab = va.mul_add(vb, v_ab);
        v_aa = va.mul_add(va, v_aa);
        v_bb = vb.mul_add(vb, v_bb);
        i += 8;
    }
    let mut ab = v_ab.reduce_sum();
    let mut aa = v_aa.reduce_sum();
    let mut bb = v_bb.reduce_sum();
    while i < n {
        let x = a[i];
        let y = b[i];
        ab += x * y;
        aa += x * x;
        bb += y * y;
        i += 1;
    }
    if aa > 0.0 && bb > 0.0 {
        ab / (aa * bb).sqrt()
    } else {
        0.0
    }
}

#[inline]
pub fn project(h: &[f64; 9], u: f64, v: f64) -> (f64, f64) {
    let w = h[6] * u + h[7] * v + h[8];
    ((h[0] * u + h[1] * v + h[2]) / w, (h[3] * u + h[4] * v + h[5]) / w)
}

pub fn invert(m: &[f64; 9]) -> [f64; 9] {
    let [a, b, c, d, e, f, g, h, i] = *m;
    let a_co = e * i - f * h;
    let b_co = -(d * i - f * g);
    let c_co = d * h - e * g;
    let det = a * a_co + b * b_co + c * c_co;
    [
        a_co / det,
        -(b * i - c * h) / det,
        (b * f - c * e) / det,
        b_co / det,
        (a * i - c * g) / det,
        -(a * f - c * d) / det,
        c_co / det,
        -(a * h - b * g) / det,
        (a * e - b * d) / det,
    ]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardLoc {
    pub s: f64,
    pub tx: f64,
    pub ty: f64,
    pub score: f64,
    pub w: usize,
    pub h: usize,
    pub by_labels: bool,
    pub cells_matched: usize,
}

impl BoardLoc {
    #[inline]
    pub fn at(&self, u: f64, v: f64) -> (f64, f64) {
        let (x, y) = project(&H0, u, v);
        (self.s * x + self.tx, self.s * y + self.ty)
    }

    #[inline]
    pub fn grid_of(&self, x: f64, y: f64) -> (f64, f64) {
        let h_inv = invert(&H0);
        project(&h_inv, (x - self.tx) / self.s, (y - self.ty) / self.s)
    }

    #[inline]
    pub fn scale_at(&self, u: f64, v: f64) -> f64 {
        let (x0, y0) = project(&H0, u - 0.5, v);
        let (x1, y1) = project(&H0, u + 0.5, v);
        (self.s * (x1 - x0).hypot(y1 - y0)) / PITCH0
    }
}

pub struct ImageView<'a> {
    pub width: usize,
    pub height: usize,
    pub data: &'a [u8], // RGBA
}

impl<'a> ImageView<'a> {
    #[inline]
    pub fn rgb_at(&self, x: isize, y: isize) -> (u8, u8, u8) {
        let xi = x.clamp(0, self.width as isize - 1) as usize;
        let yi = y.clamp(0, self.height as isize - 1) as usize;
        let o = (yi * self.width + xi) * 4;
        (self.data[o], self.data[o + 1], self.data[o + 2])
    }

    #[inline]
    pub fn hsv_at(&self, x: f64, y: f64) -> (f32, f32, f32) {
        let (r, g, b) = self.rgb_at(x.round() as isize, y.round() as isize);
        hsv(r as f32, g as f32, b as f32)
    }

    #[inline]
    pub fn gray_px(&self, x: isize, y: isize) -> f32 {
        let (r, g, b) = self.rgb_at(x, y);
        0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32)
    }

    #[inline]
    pub fn sharp_px(&self, x: isize, y: isize) -> f32 {
        let wi = self.width as isize;
        let hi = self.height as isize;
        if x > 0 && x + 1 < wi && y > 0 && y + 1 < hi {
            let w = self.width;
            let ux = (x as usize) - 1;
            let uy = (y as usize) - 1;
            let mut sr = 0u32;
            let mut sg = 0u32;
            let mut sb = 0u32;
            for dy in 0..3 {
                let base = ((uy + dy) * w + ux) * 4;
                let row = &self.data[base..base + 12];
                sr += (row[0] as u32) + (row[4] as u32) + (row[8] as u32);
                sg += (row[1] as u32) + (row[5] as u32) + (row[9] as u32);
                sb += (row[2] as u32) + (row[6] as u32) + (row[10] as u32);
            }
            (0.299 * (sr as f32) + 0.587 * (sg as f32) + 0.114 * (sb as f32)) * (1.0 / 9.0)
        } else {
            let mut acc = 0.0_f32;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    acc += self.gray_px(x + dx, y + dy);
                }
            }
            acc * (1.0 / 9.0)
        }
    }

    #[inline]
    pub fn sample_sharp(&self, mut x: f64, mut y: f64) -> f32 {
        let w_max = (self.width as f64) - 1.001;
        let h_max = (self.height as f64) - 1.001;
        if x < 0.0 {
            x = 0.0;
        } else if x > w_max {
            x = w_max;
        }
        if y < 0.0 {
            y = 0.0;
        } else if y > h_max {
            y = h_max;
        }
        let x0 = x as usize;
        let y0 = y as usize;
        let fx = (x - (x0 as f64)) as f32;
        let fy = (y - (y0 as f64)) as f32;
        if x0 >= 1 && x0 + 2 < self.width && y0 >= 1 && y0 + 2 < self.height {
            let w = self.width;
            let ux = x0 - 1;
            let uy = y0 - 1;
            let wx0 = 1.0 - fx;
            let wx3 = fx;
            let wy0 = 1.0 - fy;
            let wy3 = fy;
            let mut r_tot = 0.0_f32;
            let mut g_tot = 0.0_f32;
            let mut b_tot = 0.0_f32;
            for (dy, wy) in [(0usize, wy0), (1, 1.0), (2, 1.0), (3, wy3)] {
                let base = ((uy + dy) * w + ux) * 4;
                let row = &self.data[base..base + 16];
                let rr = wx0 * (row[0] as f32)
                    + ((row[4] as u32 + row[8] as u32) as f32)
                    + wx3 * (row[12] as f32);
                let gg = wx0 * (row[1] as f32)
                    + ((row[5] as u32 + row[9] as u32) as f32)
                    + wx3 * (row[13] as f32);
                let bb = wx0 * (row[2] as f32)
                    + ((row[6] as u32 + row[10] as u32) as f32)
                    + wx3 * (row[14] as f32);
                r_tot += wy * rr;
                g_tot += wy * gg;
                b_tot += wy * bb;
            }
            (0.299 * r_tot + 0.587 * g_tot + 0.114 * b_tot) * (1.0 / 9.0)
        } else {
            let ix0 = x0 as isize;
            let iy0 = y0 as isize;
            let s00 = self.sharp_px(ix0, iy0);
            let s10 = self.sharp_px(ix0 + 1, iy0);
            let s01 = self.sharp_px(ix0, iy0 + 1);
            let s11 = self.sharp_px(ix0 + 1, iy0 + 1);
            (s00 * (1.0 - fx) + s10 * fx) * (1.0 - fy)
                + (s01 * (1.0 - fx) + s11 * fx) * fy
        }
    }
}

#[inline]
pub fn hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let d = mx - mn;
    let mut h = 0.0_f32;
    if d > 0.0 {
        if (mx - r).abs() < 1e-5 {
            h = (((g - b) / d) % 6.0 + 6.0) % 6.0;
        } else if (mx - g).abs() < 1e-5 {
            h = (b - r) / d + 2.0;
        } else {
            h = (r - g) / d + 4.0;
        }
    }
    (h * 60.0, if mx > 0.0 { d / mx } else { 0.0 }, mx / 255.0)
}

pub fn half_gray_of(img: &ImageView<'_>) -> (Vec<f32>, usize, usize) {
    use rayon::prelude::*;
    let w = img.width;
    let w2 = w >> 1;
    let h2 = img.height >> 1;
    let mut out = vec![0.0_f32; w2 * h2];
    if w2 * h2 >= 25_000 {
        out.par_chunks_mut(w2)
            .enumerate()
            .for_each(|(y2, row_out)| {
                let r0 = (2 * y2) * w * 4;
                let r1 = (2 * y2 + 1) * w * 4;
                for x2 in 0..w2 {
                    let c0 = r0 + (2 * x2) * 4;
                    let c1 = c0 + 4;
                    let c2 = r1 + (2 * x2) * 4;
                    let c3 = c2 + 4;
                    let r = (img.data[c0] as u32
                        + img.data[c1] as u32
                        + img.data[c2] as u32
                        + img.data[c3] as u32) as f32;
                    let g = (img.data[c0 + 1] as u32
                        + img.data[c1 + 1] as u32
                        + img.data[c2 + 1] as u32
                        + img.data[c3 + 1] as u32) as f32;
                    let b = (img.data[c0 + 2] as u32
                        + img.data[c1 + 2] as u32
                        + img.data[c2 + 2] as u32
                        + img.data[c3 + 2] as u32) as f32;
                    row_out[x2] = 0.25 * (0.299 * r + 0.587 * g + 0.114 * b);
                }
            });
    } else {
        for y2 in 0..h2 {
            let r0 = (2 * y2) * w * 4;
            let r1 = (2 * y2 + 1) * w * 4;
            let row_out = &mut out[y2 * w2..(y2 + 1) * w2];
            for x2 in 0..w2 {
                let c0 = r0 + (2 * x2) * 4;
                let c1 = c0 + 4;
                let c2 = r1 + (2 * x2) * 4;
                let c3 = c2 + 4;
                let r = (img.data[c0] as u32
                    + img.data[c1] as u32
                    + img.data[c2] as u32
                    + img.data[c3] as u32) as f32;
                let g = (img.data[c0 + 1] as u32
                    + img.data[c1 + 1] as u32
                    + img.data[c2 + 1] as u32
                    + img.data[c3 + 1] as u32) as f32;
                let b = (img.data[c0 + 2] as u32
                    + img.data[c1 + 2] as u32
                    + img.data[c2 + 2] as u32
                    + img.data[c3 + 2] as u32) as f32;
                row_out[x2] = 0.25 * (0.299 * r + 0.587 * g + 0.114 * b);
            }
        }
    }
    (out, w2, h2)
}

pub fn half(g: &[f32], w: usize, h: usize) -> (Vec<f32>, usize, usize) {
    let w2 = w >> 1;
    let h2 = h >> 1;
    let mut out = vec![0.0_f32; w2 * h2];
    for y in 0..h2 {
        for x in 0..w2 {
            let i = 2 * y * w + 2 * x;
            out[y * w2 + x] = 0.25 * (g[i] + g[i + 1] + g[i + w] + g[i + w + 1]);
        }
    }
    (out, w2, h2)
}

#[inline]
pub fn sample(g: &[f32], w: usize, h: usize, mut x: f64, mut y: f64) -> f32 {
    if x < 0.0 {
        x = 0.0;
    } else if x > (w as f64) - 1.001 {
        x = (w as f64) - 1.001;
    }
    if y < 0.0 {
        y = 0.0;
    } else if y > (h as f64) - 1.001 {
        y = (h as f64) - 1.001;
    }
    let x0 = x as usize;
    let y0 = y as usize;
    let fx = (x - (x0 as f64)) as f32;
    let fy = (y - (y0 as f64)) as f32;
    let i = y0 * w + x0;
    (g[i] * (1.0 - fx) + g[i + 1] * fx) * (1.0 - fy)
        + (g[i + w] * (1.0 - fx) + g[i + w + 1] * fx) * fy
}

#[derive(Clone, Copy, Debug)]
pub struct Blob {
    pub n: usize,
    pub cx: f64,
    pub cy: f64,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub w: i32,
    pub h: i32,
}

/// Pre-allocated scratch slot for zero-allocation OCR and connected-components scanning.
#[derive(Default)]
pub struct ScanScratchSlot {
    pub sat: Vec<u32>,
    pub elig_coords: Vec<(u16, u16, u16)>,
    pub hit_coords: Vec<(u16, u16)>,
    pub mask: Vec<u8>,
    pub stack: Vec<(u32, u32)>,
}

fn alloc_pre_faulted<T: Copy>(cap: usize, non_zero_val: T) -> Vec<T> {
    let mut v = vec![non_zero_val; cap];
    std::hint::black_box(&mut v);
    v.clear();
    v
}

impl ScanScratchSlot {
    pub fn with_capacity() -> Self {
        Self {
            sat: alloc_pre_faulted(1024 * 1024, 0u32),                 // 4 MB
            elig_coords: alloc_pre_faulted(256 * 1024, (0u16, 0u16, 0u16)), // 1.5 MB
            hit_coords: alloc_pre_faulted(32 * 1024, (0u16, 0u16)),    // 128 KB
            mask: alloc_pre_faulted(3 * 1024 * 1024, 0u8),             // 3 MB
            stack: alloc_pre_faulted(8 * 1024, (0u32, 0u32)),          // 64 KB
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.elig_coords.clear();
        if self.elig_coords.capacity() > 1024 * 1024 {
            self.elig_coords.shrink_to(256 * 1024);
        }
        self.hit_coords.clear();
        if self.hit_coords.capacity() > 128 * 1024 {
            self.hit_coords.shrink_to(32 * 1024);
        }
        self.stack.clear();
        if self.stack.capacity() > 32 * 1024 {
            self.stack.shrink_to(8 * 1024);
        }
        if self.sat.capacity() > 4 * 1024 * 1024 {
            self.sat.clear();
            self.sat.shrink_to(1024 * 1024);
        }
        if self.mask.capacity() > 8 * 1024 * 1024 {
            self.mask.clear();
            self.mask.shrink_to(3 * 1024 * 1024);
        }
    }
}

/// Pre-allocated per-cell blur scratch buffers (`row_buf` + `patch`) for `read_cell_with_slot`.
pub struct CellScratchSlot {
    pub row_buf: Vec<f32>,
    pub patch: Vec<f32>,
}

impl CellScratchSlot {
    pub fn with_capacity() -> Self {
        Self {
            row_buf: alloc_pre_faulted(130 * 112, 0.0_f32),
            patch: alloc_pre_faulted(128 * 112, 0.0_f32),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        if self.row_buf.capacity() > 256 * 128 {
            self.row_buf.clear();
            self.row_buf.shrink_to(130 * 112);
        }
        if self.patch.capacity() > 256 * 128 {
            self.patch.clear();
            self.patch.shrink_to(128 * 112);
        }
    }
}

/// Reusable, bounded-memory arena for parallel board & pick scanning.
/// Eliminates all per-frame heap allocations and thread-local buffer retention.
pub struct ScanArena {
    pub slots: [ScanScratchSlot; 6],
    pub cell_slots: [CellScratchSlot; 16],
}

impl Default for ScanArena {
    fn default() -> Self {
        Self::new()
    }
}

impl ScanArena {
    pub fn new() -> Self {
        let _ = board_template_refs();
        Self {
            slots: std::array::from_fn(|_| ScanScratchSlot::with_capacity()),
            cell_slots: std::array::from_fn(|_| CellScratchSlot::with_capacity()),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        for slot in &mut self.slots {
            slot.reset();
        }
        for slot in &mut self.cell_slots {
            slot.reset();
        }
    }
}

fn blobs_from_mask_hits(
    mask: &mut [u8],
    hit_coords: &[(u16, u16)],
    stack: &mut Vec<(u32, u32)>,
    w: usize,
    h: usize,
    bx0: usize,
    by0: usize,
) -> Vec<Blob> {
    let mut out = Vec::new();
    stack.clear();
    for &(sx0_u16, sy0_u16) in hit_coords {
        let sx0 = sx0_u16 as usize;
        let sy0 = sy0_u16 as usize;
        let i = sy0 * w + sx0;
        if mask[i] != 1 {
            continue;
        }
        mask[i] = 0;
        stack.push((sx0 as u32, sy0 as u32));
        let mut n = 0usize;
        let mut sx = 0usize;
        let mut sy = 0usize;
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = -1i32;
        let mut max_y = -1i32;
        while let Some((xu, yu)) = stack.pop() {
            let x = xu as usize;
            let y = yu as usize;
            let j = y * w + x;
            n += 1;
            sx += x;
            sy += y;
            let xi = xu as i32;
            let yi = yu as i32;
            if xi < min_x {
                min_x = xi;
            }
            if xi > max_x {
                max_x = xi;
            }
            if yi < min_y {
                min_y = yi;
            }
            if yi > max_y {
                max_y = yi;
            }
            if x > 0 && mask[j - 1] == 1 {
                mask[j - 1] = 0;
                stack.push((xu - 1, yu));
            }
            if x + 1 < w && mask[j + 1] == 1 {
                mask[j + 1] = 0;
                stack.push((xu + 1, yu));
            }
            if y > 0 && mask[j - w] == 1 {
                mask[j - w] = 0;
                stack.push((xu, yu - 1));
            }
            if y + 1 < h && mask[j + w] == 1 {
                mask[j + w] = 0;
                stack.push((xu, yu + 1));
            }
        }
        out.push(Blob {
            n,
            cx: (bx0 as f64) + (sx as f64) / (n as f64),
            cy: (by0 as f64) + (sy as f64) / (n as f64),
            x0: (bx0 as i32) + min_x,
            y0: (by0 as i32) + min_y,
            x1: (bx0 as i32) + max_x,
            y1: (by0 as i32) + max_y,
            w: max_x - min_x + 1,
            h: max_y - min_y + 1,
        });
    }
    out
}

pub fn blobs_with_slot<F>(
    img: &ImageView<'_>,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    slot: &mut ScanScratchSlot,
    test: F,
) -> Vec<Blob>
where
    F: Fn(usize, usize) -> bool + Sync,
{
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1.ceil() as isize).min(img.width as isize - 1);
    let by1 = (y1.ceil() as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return Vec::new();
    }
    let bx1 = bx1 as usize;
    let by1 = by1 as usize;
    let w = bx1 - bx0 + 1;
    let h = by1 - by0 + 1;
    let n = w * h;

    let ScanScratchSlot {
        hit_coords,
        mask,
        stack,
        ..
    } = slot;
    if mask.len() < n {
        mask.resize(n, 0);
    }
    hit_coords.clear();
    for y in 0..h {
        let row_off = y * w;
        let py = by0 + y;
        for x in 0..w {
            if test(bx0 + x, py) {
                mask[row_off + x] = 1;
                hit_coords.push((x as u16, y as u16));
            }
        }
    }
    if hit_coords.is_empty() {
        return Vec::new();
    }
    blobs_from_mask_hits(&mut mask[..n], hit_coords, stack, w, h, bx0, by0)
}

#[inline]
fn blobs_rgb_strided_with_slot<F>(
    img: &ImageView<'_>,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    skip_rect: Option<[usize; 4]>,
    step_y: usize,
    slot: &mut ScanScratchSlot,
    test: F,
) -> Vec<Blob>
where
    F: Fn(u8, u8, u8) -> bool,
{
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1.ceil() as isize).min(img.width as isize - 1);
    let by1 = (y1.ceil() as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return Vec::new();
    }
    let bx1 = bx1 as usize;
    let by1 = by1 as usize;
    let w = bx1 - bx0 + 1;
    let h = by1 - by0 + 1;
    let n_px = w * h;
    let step_y = step_y.max(1);

    let ScanScratchSlot {
        hit_coords,
        mask,
        stack,
        ..
    } = slot;
    if mask.len() < n_px {
        mask.resize(n_px, 0);
    }
    hit_coords.clear();
    stack.clear();

    let mut out = Vec::new();
    let in_skip = |px: usize, py: usize| -> bool {
        if let Some([sx0, sy0, sx1, sy1]) = skip_rect {
            py >= sy0 && py <= sy1 && px >= sx0 && px < sx1
        } else {
            false
        }
    };

    for sy0 in (0..h).step_by(step_y) {
        let py0 = by0 + sy0;
        let row_off = sy0 * w;
        let row_base = (py0 * img.width + bx0) * 4;
        let row_data = &img.data[row_base..row_base + w * 4];
        for sx0 in 0..w {
            let px0 = bx0 + sx0;
            if in_skip(px0, py0) {
                continue;
            }
            let o = sx0 * 4;
            if !test(row_data[o], row_data[o + 1], row_data[o + 2]) {
                continue;
            }
            let i0 = row_off + sx0;
            if mask[i0] != 0 {
                continue;
            }
            mask[i0] = 1;
            hit_coords.push((sx0 as u16, sy0 as u16));
            stack.push((sx0 as u32, sy0 as u32));
            let mut n = 0usize;
            let mut sx = 0usize;
            let mut sy = 0usize;
            let mut min_x = i32::MAX;
            let mut min_y = i32::MAX;
            let mut max_x = -1i32;
            let mut max_y = -1i32;
            while let Some((xu, yu)) = stack.pop() {
                let x = xu as usize;
                let y = yu as usize;
                let j = y * w + x;
                n += 1;
                sx += x;
                sy += y;
                let xi = xu as i32;
                let yi = yu as i32;
                if xi < min_x {
                    min_x = xi;
                }
                if xi > max_x {
                    max_x = xi;
                }
                if yi < min_y {
                    min_y = yi;
                }
                if yi > max_y {
                    max_y = yi;
                }
                if x > 0 {
                    let ni = j - 1;
                    if mask[ni] == 0 {
                        let nx = x - 1;
                        let ny = y;
                        if !in_skip(bx0 + nx, by0 + ny) {
                            let no = ((by0 + ny) * img.width + (bx0 + nx)) * 4;
                            if test(img.data[no], img.data[no + 1], img.data[no + 2]) {
                                mask[ni] = 1;
                                hit_coords.push((nx as u16, ny as u16));
                                stack.push((xu - 1, yu));
                            } else {
                                mask[ni] = 2;
                                hit_coords.push((nx as u16, ny as u16));
                            }
                        }
                    }
                }
                if x + 1 < w {
                    let ni = j + 1;
                    if mask[ni] == 0 {
                        let nx = x + 1;
                        let ny = y;
                        if !in_skip(bx0 + nx, by0 + ny) {
                            let no = ((by0 + ny) * img.width + (bx0 + nx)) * 4;
                            if test(img.data[no], img.data[no + 1], img.data[no + 2]) {
                                mask[ni] = 1;
                                hit_coords.push((nx as u16, ny as u16));
                                stack.push((xu + 1, yu));
                            } else {
                                mask[ni] = 2;
                                hit_coords.push((nx as u16, ny as u16));
                            }
                        }
                    }
                }
                if y > 0 {
                    let ni = j - w;
                    if mask[ni] == 0 {
                        let nx = x;
                        let ny = y - 1;
                        if !in_skip(bx0 + nx, by0 + ny) {
                            let no = ((by0 + ny) * img.width + (bx0 + nx)) * 4;
                            if test(img.data[no], img.data[no + 1], img.data[no + 2]) {
                                mask[ni] = 1;
                                hit_coords.push((nx as u16, ny as u16));
                                stack.push((xu, yu - 1));
                            } else {
                                mask[ni] = 2;
                                hit_coords.push((nx as u16, ny as u16));
                            }
                        }
                    }
                }
                if y + 1 < h {
                    let ni = j + w;
                    if mask[ni] == 0 {
                        let nx = x;
                        let ny = y + 1;
                        if !in_skip(bx0 + nx, by0 + ny) {
                            let no = ((by0 + ny) * img.width + (bx0 + nx)) * 4;
                            if test(img.data[no], img.data[no + 1], img.data[no + 2]) {
                                mask[ni] = 1;
                                hit_coords.push((nx as u16, ny as u16));
                                stack.push((xu, yu + 1));
                            } else {
                                mask[ni] = 2;
                                hit_coords.push((nx as u16, ny as u16));
                            }
                        }
                    }
                }
            }
            out.push(Blob {
                n,
                cx: (bx0 as f64) + (sx as f64) / (n as f64),
                cy: (by0 as f64) + (sy as f64) / (n as f64),
                x0: (bx0 as i32) + min_x,
                y0: (by0 as i32) + min_y,
                x1: (bx0 as i32) + max_x,
                y1: (by0 as i32) + max_y,
                w: max_x - min_x + 1,
                h: max_y - min_y + 1,
            });
        }
    }

    for &(hx, hy) in hit_coords.iter() {
        mask[(hy as usize) * w + (hx as usize)] = 0;
    }
    hit_coords.clear();
    out
}

struct PyrLevel {
    g: Vec<f32>,
    w: usize,
    h: usize,
    f: f64,
}

#[inline(always)]
fn ncc_accum_simd_impl(vals: &[f32], tv: &[f32]) -> (f32, f32, f32) {
    use std::simd::prelude::*;
    use std::simd::StdFloat;
    let n = vals.len().min(tv.len());
    let mut v_sx = f32x8::splat(0.0);
    let mut v_sxx = f32x8::splat(0.0);
    let mut v_sxt = f32x8::splat(0.0);
    let mut i = 0usize;
    while i + 8 <= n {
        let v = f32x8::from_slice(&vals[i..i + 8]);
        let t = f32x8::from_slice(&tv[i..i + 8]);
        v_sx += v;
        v_sxx = v.mul_add(v, v_sxx);
        v_sxt = v.mul_add(t, v_sxt);
        i += 8;
    }
    let mut sx = v_sx.reduce_sum();
    let mut sxx = v_sxx.reduce_sum();
    let mut sxt = v_sxt.reduce_sum();
    while i < n {
        let v = vals[i];
        sx += v;
        sxx += v * v;
        sxt += v * tv[i];
        i += 1;
    }
    (sx, sxx, sxt)
}

#[inline]
fn ncc_score_slice(vals: &[f32], tv: &[f32]) -> f64 {
    let m = vals.len().min(tv.len());
    let (sx, sxx, sxt) = ncc_accum_simd_impl(vals, tv);
    let sx64 = sx as f64;
    let varx = (sxx as f64) - (sx64 * sx64) / (m as f64);
    if varx > 1e-6 {
        (sxt as f64) / varx.sqrt()
    } else {
        -1.0
    }
}

struct BoardTemplateRefs {
    tv: Vec<f32>,
    c4_pts: Vec<(i32, i32, f32)>,
    c4_tv: Vec<f32>,
    c8_pts: Vec<(i32, i32, f32)>,
    c8_tv: Vec<f32>,
    c32_pts: Vec<(i32, i32, f32)>,
    c32_tv: Vec<f32>,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

fn board_template_refs() -> &'static BoardTemplateRefs {
    static REFS: std::sync::OnceLock<BoardTemplateRefs> = std::sync::OnceLock::new();
    REFS.get_or_init(|| {
        let pts = &REF_BOARD_POINTS;
        let n = pts.len();
        let mut mean = 0.0_f32;
        for p in pts {
            mean += p.2;
        }
        mean /= n as f32;
        let mut tv = vec![0.0_f32; n];
        let mut norm = 0.0_f32;
        for (i, p) in pts.iter().enumerate() {
            tv[i] = p.2 - mean;
            norm += tv[i] * tv[i];
        }
        norm = norm.sqrt().max(1e-6);
        for v in &mut tv {
            *v /= norm;
        }

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for p in pts {
            min_x = min_x.min(p.0 as f64);
            max_x = max_x.max(p.0 as f64);
            min_y = min_y.min(p.1 as f64);
            max_y = max_y.max(p.1 as f64);
        }

        let build_sub = |step: usize| -> (Vec<(i32, i32, f32)>, Vec<f32>) {
            let sub_pts: Vec<(i32, i32, f32)> = pts.iter().copied().step_by(step).collect();
            let cn = sub_pts.len();
            let mut cmean = 0.0_f32;
            for p in &sub_pts {
                cmean += p.2;
            }
            cmean /= cn as f32;
            let mut ctv = vec![0.0_f32; cn];
            let mut cnorm = 0.0_f32;
            for (j, p) in sub_pts.iter().enumerate() {
                ctv[j] = p.2 - cmean;
                cnorm += ctv[j] * ctv[j];
            }
            cnorm = cnorm.sqrt().max(1e-6);
            for v in &mut ctv {
                *v /= cnorm;
            }
            (sub_pts, ctv)
        };

        let (c4_pts, c4_tv) = build_sub(4);
        let (c8_pts, c8_tv) = build_sub(8);
        let (c32_pts, c32_tv) = build_sub(32);

        BoardTemplateRefs {
            tv,
            c4_pts,
            c4_tv,
            c8_pts,
            c8_tv,
            c32_pts,
            c32_tv,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    })
}

pub fn locate_board(img: &ImageView<'_>, hint: Option<BoardLoc>) -> Option<BoardLoc> {
    use rayon::prelude::*;

    let w = img.width;
    let h = img.height;
    if w < 64 || h < 64 {
        return None;
    }
    let pts = &REF_BOARD_POINTS;
    let n = pts.len();
    let t_refs = board_template_refs();
    let tv = &t_refs.tv;
    let c4_pts = &t_refs.c4_pts;
    let c4_tv = &t_refs.c4_tv;
    let c8_pts = &t_refs.c8_pts;
    let c8_tv = &t_refs.c8_tv;
    let c32_pts = &t_refs.c32_pts;
    let c32_tv = &t_refs.c32_tv;
    let min_x = t_refs.min_x;
    let max_x = t_refs.max_x;
    let min_y = t_refs.min_y;
    let max_y = t_refs.max_y;
    let bw = max_x - min_x;
    let bh = max_y - min_y;
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;

    let score_direct_gray = |sub_pts: &[(i32, i32, f32)], sub_tv: &[f32], s: f64, ox: f64, oy: f64| -> f64 {
        let m = sub_pts.len();
        let wi = w as isize;
        let hi = h as isize;
        let mut buf = [0.0_f32; 450];
        for i in 0..m {
            let xi = (ox + (sub_pts[i].0 as f64) * s) as isize;
            let yi = (oy + (sub_pts[i].1 as f64) * s) as isize;
            buf[i] = img.gray_px(xi.clamp(0, wi - 1), yi.clamp(0, hi - 1));
        }
        ncc_score_slice(&buf[..m], sub_tv)
    };

    let score_direct_sharp_sub = |sub_pts: &[(i32, i32, f32)], sub_tv: &[f32], s: f64, ox: f64, oy: f64| -> f64 {
        let m = sub_pts.len();
        let mut buf = [0.0_f32; 1800];
        for i in 0..m {
            buf[i] = img.sample_sharp(
                ox + (sub_pts[i].0 as f64) * s,
                oy + (sub_pts[i].1 as f64) * s,
            );
        }
        ncc_score_slice(&buf[..m], sub_tv)
    };

    // Zero-allocation refinement around a candidate (s, tx, ty):
    // 1. 225-point gate via img.gray_px (~0.4us)
    // 2. Fast exact-center check when stationary (~4us)
    // 3. 3-scale 1.0px integer grid search with 450 points + 0.5px peak refinement on winning scale
    let refine_near_direct = |cand_s: f64, cand_tx: f64, cand_ty: f64, min_accept: f64| -> Option<BoardLoc> {
        let coarse_check = score_direct_gray(c8_pts, c8_tv, cand_s, cand_tx, cand_ty);
        if coarse_check < (min_accept - 0.16).max(0.46) {
            return None;
        }
        if coarse_check >= (min_accept - 0.08).max(0.68) {
            let exact = score_direct_sharp_sub(pts, tv, cand_s, cand_tx, cand_ty);
            if exact >= min_accept.max(0.78) {
                return Some(BoardLoc {
                    s: cand_s,
                    tx: cand_tx,
                    ty: cand_ty,
                    score: exact,
                    w,
                    h,
                    by_labels: false,
                    cells_matched: 0,
                });
            }
        }
        let mut best_1px = (cand_s, cand_tx, cand_ty, -2.0_f64);
        for i in -1isize..=1 {
            let sc = cand_s * (1.0 + (i as f64) * 0.0025);
            let bx = cand_tx - (sc - cand_s) * cx;
            let by = cand_ty - (sc - cand_s) * cy;
            for dy in -2isize..=2 {
                for dx in -2isize..=2 {
                    let tx = bx + (dx as f64);
                    let ty = by + (dy as f64);
                    let v = score_direct_gray(c4_pts, c4_tv, sc, tx, ty);
                    if v > best_1px.3 {
                        best_1px = (sc, tx, ty, v);
                    }
                }
            }
        }
        if best_1px.3 < (min_accept - 0.08).max(0.50) {
            return None;
        }
        let (sc, cx_tx, cy_ty, _) = best_1px;
        let mut best_fine = (sc, cx_tx, cy_ty, -2.0_f64);
        for hdy in -1isize..=1 {
            let ty = cy_ty + (hdy as f64) * 0.5;
            for hdx in -1isize..=1 {
                let tx = cx_tx + (hdx as f64) * 0.5;
                let v = score_direct_sharp_sub(pts, tv, sc, tx, ty);
                if v > best_fine.3 {
                    best_fine = (sc, tx, ty, v);
                }
            }
        }

        if best_fine.3 >= min_accept {
            Some(BoardLoc {
                s: best_fine.0,
                tx: best_fine.1,
                ty: best_fine.2,
                score: best_fine.3,
                w,
                h,
                by_labels: false,
                cells_matched: 0,
            })
        } else {
            None
        }
    };

    if let Some(hloc) = hint {
        if hloc.w == w && hloc.h == h {
            if let Some(near) = refine_near_direct(hloc.s, hloc.tx, hloc.ty, 0.65_f64.max(hloc.score - 0.08)) {
                return Some(near);
            }
        }
    }

    // Standard 16:9 game window board position (reference frame is 1344x756)
    let s_win = ((w as f64) / 1344.0).min((h as f64) / 756.0);
    let tx_win = ((w as f64) - 1344.0 * s_win) * 0.5;
    let ty_win = ((h as f64) - 756.0 * s_win) * 0.5;
    if let Some(near) = refine_near_direct(s_win, tx_win, ty_win, 0.75) {
        return Some(near);
    }
    let s_w = (w as f64) / 1344.0;
    for &(cand_s, cand_tx, cand_ty) in &[
        (s_w, (w as f64) * 0.5 - 658.0 * s_w, (h as f64) * 0.5 - 370.5 * s_w),
        (s_w, (w as f64) * 0.5 - 658.0 * s_w, 0.0),
        (s_w, ((w as f64) - 1344.0 * s_w) * 0.5, ((h as f64) - 756.0 * s_w) * 0.5),
        (s_win, (w as f64) * 0.5 - 658.0 * s_win, (h as f64) * 0.5 - 370.5 * s_win),
    ] {
        if let Some(near) = refine_near_direct(cand_s, cand_tx, cand_ty, 0.75) {
            return Some(near);
        }
    }

    // Fast 9-scale window neighborhood pre-check using 56 points (~0.025ms when no board is present).
    let cn32 = c32_pts.len();
    let wi = w as isize;
    let hi = h as isize;
    let mut best_coarse: Option<(f64, isize, isize, f64)> = None;
    let mut dxs32 = [0isize; 64];
    let mut dys32 = [0isize; 64];
    let mut row_bases32 = [0usize; 64];
    let mut vals32 = [0.0_f32; 64];
    let dt = 3.5 * s_win;

    for i in -4isize..=4 {
        let s = s_win * (1.0 + (i as f64) * 0.015);
        let bx = tx_win - (s - s_win) * cx;
        let by = ty_win - (s - s_win) * cy;
        for j in 0..cn32 {
            dxs32[j] = ((c32_pts[j].0 as f64) * s).round() as isize;
            dys32[j] = ((c32_pts[j].1 as f64) * s).round() as isize;
        }
        for dy_i in (-6isize..=12).step_by(2) {
            let ty = by + (dy_i as f64) * dt;
            let oy = ty.round() as isize;
            for j in 0..cn32 {
                let yi = (oy + dys32[j]).clamp(0, hi - 1) as usize;
                row_bases32[j] = yi * w * 4;
            }
            for dx_i in (-6isize..=6).step_by(2) {
                let tx = bx + (dx_i as f64) * dt;
                let ox = tx.round() as isize;
                for j in 0..cn32 {
                    let xi = (ox + dxs32[j]).clamp(0, wi - 1) as usize;
                    let o = row_bases32[j] + xi * 4;
                    vals32[j] = 0.299 * (img.data[o] as f32)
                        + 0.587 * (img.data[o + 1] as f32)
                        + 0.114 * (img.data[o + 2] as f32);
                }
                let sc32 = ncc_score_slice(&vals32[..cn32], c32_tv);
                if best_coarse.map_or(true, |b| sc32 > b.3) {
                    best_coarse = Some((s, dx_i, dy_i, sc32));
                }
            }
        }
    }

    if let Some((cs, cdx, cdy, csc32)) = best_coarse {
        let mut best_win = (
            cs,
            tx_win - (cs - s_win) * cx + (cdx as f64) * dt,
            ty_win - (cs - s_win) * cy + (cdy as f64) * dt,
            csc32,
        );
        if csc32 >= 0.36 {
            let bx = tx_win - (cs - s_win) * cx;
            let by = ty_win - (cs - s_win) * cy;
            best_win.3 = -2.0;
            for fdy in (cdy - 1).max(-8)..=(cdy + 1).min(14) {
                for fdx in (cdx - 1).max(-8)..=(cdx + 1).min(8) {
                    let ftx = bx + (fdx as f64) * dt;
                    let fty = by + (fdy as f64) * dt;
                    let sc8 = score_direct_gray(c8_pts, c8_tv, cs, ftx, fty);
                    if sc8 > best_win.3 {
                        best_win = (cs, ftx, fty, sc8);
                    }
                }
            }
        }

        let (ws, wtx, wty, wsc) = best_win;
        if wsc >= 0.48 {
            let dt_mid = 1.2 * s_win;
            if let Some((ms, mtx, mty, msc)) = (-3isize..=3)
                .into_par_iter()
                .map(|i| {
                    let s = ws * (1.0 + (i as f64) * 0.005);
                    let bx = wtx - (s - ws) * cx;
                    let by = wty - (s - ws) * cy;
                    let mut best = (s, bx, by, -2.0_f64);
                    for dy in -3isize..=3 {
                        for dx in -3isize..=3 {
                            let tx = bx + (dx as f64) * dt_mid;
                            let ty = by + (dy as f64) * dt_mid;
                            let sc = score_direct_gray(c4_pts, c4_tv, s, tx, ty);
                            if sc > best.3 {
                                best = (s, tx, ty, sc);
                            }
                        }
                    }
                    best
                })
                .reduce_with(|a, b| if b.3 > a.3 { b } else { a })
            {
                if msc >= 0.52 {
                    if let Some(near) = refine_near_direct(ms, mtx, mty, 0.65) {
                        return Some(near);
                    }
                }
            }
        }

        let aspect = (w as f64) / (h as f64);
        if (1.45..=2.5).contains(&aspect) {
            return Some(BoardLoc {
                s: ws,
                tx: wtx,
                ty: wty,
                score: wsc.min(0.60),
                w,
                h,
                by_labels: false,
                cells_matched: 0,
            });
        }
    }

    let score_slice = |g: &[f32], lw_u: usize, lh_u: usize, pxs: &[f64], pys: &[f64], ox: f64, oy: f64, nearest: bool| -> f64 {
        let mut sx = 0.0_f32;
        let mut sxx = 0.0_f32;
        let mut sxt = 0.0_f32;
        if nearest {
            let lw = lw_u as isize;
            let lh = lh_u as isize;
            for i in 0..n {
                let xi = ((ox + pxs[i]) as isize).clamp(0, lw - 1) as usize;
                let yi = ((oy + pys[i]) as isize).clamp(0, lh - 1) as usize;
                let v = g[yi * lw_u + xi];
                sx += v;
                sxx += v * v;
                sxt += v * tv[i];
            }
        } else {
            for i in 0..n {
                let v = sample(g, lw_u, lh_u, ox + pxs[i], oy + pys[i]);
                sx += v;
                sxx += v * v;
                sxt += v * tv[i];
            }
        }
        let sx64 = sx as f64;
        let varx = (sxx as f64) - (sx64 * sx64) / (n as f64);
        if varx > 1e-6 {
            (sxt as f64) / varx.sqrt()
        } else {
            -1.0
        }
    };

    let score_pre = |lv: &PyrLevel, pxs: &[f64], pys: &[f64], ox: f64, oy: f64, nearest: bool| -> f64 {
        score_slice(&lv.g, lv.w, lv.h, pxs, pys, ox, oy, nearest)
    };

    // Fallback multi-scale pyramid for non-standard/cropped images
    let (g2, w2, h2) = half_gray_of(img);
    let mut levels = vec![PyrLevel {
        g: g2,
        w: w2,
        h: h2,
        f: 2.0,
    }];

    while levels.last().unwrap().w > 120 && levels.last().unwrap().h > 60 {
        let last = levels.last().unwrap();
        let (dg, dw, dh) = half(&last.g, last.w, last.h);
        let nf = last.f * 2.0;
        levels.push(PyrLevel {
            g: dg,
            w: dw,
            h: dh,
            f: nf,
        });
    }

    let cn = c4_pts.len();
    let s_min = 0.12_f64.max((0.3 * (w as f64)) / bw);
    let s_max = 8.0_f64.min(1.1 * ((w as f64) / bw).max((h as f64) / bh));
    let mut scales = Vec::new();
    let mut s = s_min;
    while s <= s_max {
        scales.push(s);
        s *= 1.05;
    }

    let mut top: Vec<(f64, f64, f64, f64, f64)> = scales
        .par_iter()
        .filter_map(|&s| {
            let mut l_idx = 0;
            while l_idx + 1 < levels.len() && s / levels[l_idx + 1].f >= 0.14 {
                l_idx += 1;
            }
            let lv = &levels[l_idx];
            let k = s / lv.f;
            let ox0 = (-min_x * k - 0.06 * bw * k).ceil() as isize;
            let ox1 = ((lv.w as f64) - 1.0 - max_x * k + 0.06 * bw * k).floor() as isize;
            let oy0 = (-min_y * k - 0.08 * bh * k).ceil() as isize;
            let oy1 = ((lv.h as f64) - 1.0 - max_y * k + 0.08 * bh * k).floor() as isize;
            if ox1 < ox0 || oy1 < oy0 {
                return None;
            }
            let step = if (ox1 - ox0 + 1) * (oy1 - oy0 + 1) > 12_000 {
                2usize
            } else {
                1usize
            } ;
            let lw = lv.w as isize;
            let lh = lv.h as isize;
            let dxs: Vec<isize> = c4_pts
                .iter()
                .map(|p| ((p.0 as f64) * k) as isize)
                .collect();
            let dys: Vec<isize> = c4_pts
                .iter()
                .map(|p| ((p.1 as f64) * k) as isize)
                .collect();
            let mut row_bases = vec![0usize; cn];
            let mut best: Option<(isize, isize, f64)> = None;
            for oy in (oy0..=oy1).step_by(step) {
                for j in 0..cn {
                    let yi = (oy + dys[j]).clamp(0, lh - 1) as usize;
                    row_bases[j] = yi * lv.w;
                }
                for ox in (ox0..=ox1).step_by(step) {
                    let mut sx = 0.0_f32;
                    let mut sxx = 0.0_f32;
                    let mut sxt = 0.0_f32;
                    for j in 0..cn {
                        let xi = (ox + dxs[j]).clamp(0, lw - 1) as usize;
                        let v = lv.g[row_bases[j] + xi];
                        sx += v;
                        sxx += v * v;
                        sxt += v * c4_tv[j];
                    }
                    let sx64 = sx as f64;
                    let varx = (sxx as f64) - (sx64 * sx64) / (cn as f64);
                    let sc = if varx > 1e-6 {
                        (sxt as f64) / varx.sqrt()
                    } else {
                        -1.0
                    };
                    if best.map_or(true, |b| sc > b.2) {
                        best = Some((ox, oy, sc));
                    }
                }
            }
            best.map(|(bx, by, bsc)| (s, (bx as f64) * lv.f, (by as f64) * lv.f, lv.f, bsc))
        })
        .collect();

    if top.is_empty() {
        return None;
    }
    top.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap());

    if top[0].4 < 0.45 {
        return Some(BoardLoc {
            s: top[0].0,
            tx: top[0].1,
            ty: top[0].2,
            score: top[0].4,
            w,
            h,
            by_labels: false,
            cells_matched: 0,
        });
    }

    let grid_lv = |c_s: f64,
                   c_tx: f64,
                   c_ty: f64,
                   c_score: f64,
                   lv: &PyrLevel,
                   ds: f64,
                   steps: isize,
                   dt: f64,
                   reach: isize|
     -> (f64, f64, f64, f64) {
        let mut best = (c_s, c_tx, c_ty, c_score);
        let mut pxs = vec![0.0_f64; n];
        let mut pys = vec![0.0_f64; n];
        let nearest = lv.f > 1.0;
        for i in -steps..=steps {
            let s = c_s * (1.0 + (i as f64) * ds);
            let k = s / lv.f;
            for (idx, p) in pts.iter().enumerate() {
                pxs[idx] = (p.0 as f64) * k;
                pys[idx] = (p.1 as f64) * k;
            }
            let bx = c_tx - (s - c_s) * cx;
            let by = c_ty - (s - c_s) * cy;
            for dy in -reach..=reach {
                for dx in -reach..=reach {
                    let tx = bx + (dx as f64) * dt;
                    let ty = by + (dy as f64) * dt;
                    let sc = score_pre(lv, &pxs, &pys, tx / lv.f, ty / lv.f, nearest);
                    if sc > best.3 {
                        best = (s, tx, ty, sc);
                    }
                }
            }
        }
        best
    };

    let mid_candidates: Vec<(f64, f64, f64, f64)> = top
        .par_iter()
        .take(5)
        .filter(|c| c.4 >= 0.45)
        .map(|&(cs, ctx, cty, cf, _)| {
            let mut l_idx = 0;
            while l_idx + 1 < levels.len() && cs / levels[l_idx + 1].f >= 0.5 {
                l_idx += 1;
            }
            let mid = &levels[l_idx];
            grid_lv(cs, ctx, cty, -2.0, mid, 0.01, 4, 1.0_f64.max(cf / 2.0), 3)
        })
        .collect();

    let best_mid = mid_candidates
        .iter()
        .copied()
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap())?;

    if best_mid.3 < 0.52 {
        return Some(BoardLoc {
            s: best_mid.0,
            tx: best_mid.1,
            ty: best_mid.2,
            score: best_mid.3,
            w,
            h,
            by_labels: false,
            cells_matched: 0,
        });
    }

    let best_final = mid_candidates
        .par_iter()
        .filter(|c| c.3 >= 0.52)
        .map(|&(c_s, c_tx, c_ty, _)| {
            let mut best = (c_s, c_tx, c_ty, -2.0_f64);
            for i in -3..=3 {
                let s = c_s * (1.0 + (i as f64) * 0.0025);
                let bx = c_tx - (s - c_s) * cx;
                let by = c_ty - (s - c_s) * cy;
                for dy in -3..=3 {
                    for dx in -3..=3 {
                        let tx = bx + (dx as f64) * 0.5;
                        let ty = by + (dy as f64) * 0.5;
                        let sc = score_direct_sharp_sub(pts, tv, s, tx, ty);
                        if sc > best.3 {
                            best = (s, tx, ty, sc);
                        }
                    }
                }
            }
            best
        })
        .reduce_with(|a, b| if b.3 > a.3 { b } else { a })?;

    Some(BoardLoc {
        s: best_final.0,
        tx: best_final.1,
        ty: best_final.2,
        score: best_final.3,
        w,
        h,
        by_labels: false,
        cells_matched: 0,
    })
}

pub fn padlock_at(img: &ImageView<'_>, x: f64, y: f64, k: f64) -> f32 {
    let x0 = x - 13.0 * k;
    let y0 = y - 11.0 * k;
    let x1 = x + 13.0 * k;
    let y1 = y + 11.0 * k;
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1.ceil() as isize).min(img.width as isize - 1);
    let by1 = (y1.ceil() as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return 0.0;
    }
    let bx1 = bx1 as usize;
    let by1 = by1 as usize;
    let w = bx1 - bx0 + 1;
    let h = by1 - by0 + 1;
    let n_px = w * h;

    let test = |px: usize, py: usize| -> bool {
        let (r, g, b) = img.rgb_at(px as isize, py as isize);
        let r = r as i32;
        let g = g as i32;
        let b = b as i32;
        let l = (r + g + b) / 3;
        l > 112
            && r >= g
            && g >= b - 4
            && r - b >= 6
            && (r - b <= 64 || (r - b <= 96 && g >= 95 && b >= 75))
    };

    let found = if n_px <= 4096 {
        let mut mask = [0u8; 4096];
        let mut hit_coords = [(0u16, 0u16); 4096];
        let mut hit_len = 0usize;
        for yy in 0..h {
            let row_off = yy * w;
            let py = by0 + yy;
            for xx in 0..w {
                if test(bx0 + xx, py) {
                    mask[row_off + xx] = 1;
                    hit_coords[hit_len] = (xx as u16, yy as u16);
                    hit_len += 1;
                }
            }
        }
        if hit_len == 0 {
            Vec::new()
        } else {
            let mut stack = Vec::with_capacity(64);
            blobs_from_mask_hits(&mut mask[..n_px], &hit_coords[..hit_len], &mut stack, w, h, bx0, by0)
        }
    } else {
        let mut slot = ScanScratchSlot::default();
        blobs_with_slot(img, x0, y0, x1, y1, &mut slot, test)
    };

    let mut best_fit: Option<f64> = None;
    for b in found {
        let area = (b.n as f64) / (k * k);
        let bw = (b.w as f64) / k;
        let bh = (b.h as f64) / k;
        let off = (b.cx - x).hypot(b.cy - y) / k;
        if !(35.0..=150.0).contains(&area)
            || !(7.0..=18.0).contains(&bw)
            || !(5.0..=15.0).contains(&bh)
            || off > 7.0
        {
            continue;
        }
        let fit = 1.0 - (area - 78.0).abs() / 78.0 - off / 14.0;
        if best_fit.map_or(true, |bf| fit > bf) {
            best_fit = Some(fit);
        }
    }
    best_fit.map_or(0.0, |fit| (0.55 + fit / 2.0).clamp(0.05, 1.0) as f32)
}

pub fn face_vector(gray: &[f32], w: usize, h: usize, x: f64, y: f64, k: f64) -> ([f32; 396], f32) {
    let mut v = [0.0_f32; 396];
    let mut mean = 0.0_f32;
    for j in 0..FACE_GY {
        for i in 0..FACE_GX {
            let px = x + k * (FACE_X0 + ((FACE_X1 - FACE_X0) * ((i as f64) + 0.5)) / (FACE_GX as f64));
            let py = y + k * (FACE_Y0 + ((FACE_Y1 - FACE_Y0) * ((j as f64) + 0.5)) / (FACE_GY as f64));
            let val = sample(gray, w, h, px, py);
            v[j * FACE_GX + i] = val;
            mean += val;
        }
    }
    mean /= 396.0;
    normalize_zero_mean_in_place(&mut v, mean);
    (v, mean)
}

pub struct FaceMatch {
    pub beast: Beast,
    pub score: f32,
    pub second: f32,
    pub brightness: f32,
}

pub fn face_match(gray: &[f32], w: usize, h: usize, x: f64, y: f64, k: f64) -> FaceMatch {
    const CENTER_INDICES: [usize; 6] = [4, 13, 22, 31, 40, 49];
    let mut scores = [f32::NEG_INFINITY; 4];
    let (fv0, brightness) = face_vector(gray, w, h, x, y, k);
    for (idx, refs) in [
        (0usize, &FACES_MAMMOTH[..]),
        (1usize, &FACES_LION[..]),
        (2usize, &FACES_WOLF[..]),
        (3usize, &FACES_RAT[..]),
    ] {
        for r in refs {
            let dot = dot_f32(r, &fv0);
            if dot > scores[idx] {
                scores[idx] = dot;
            }
        }
    }
    let max0 = scores[0].max(scores[1]).max(scores[2]).max(scores[3]);
    if max0 >= 0.25 {
        let mut dy = -3.0_f64;
        while dy <= 3.001 {
            let mut dx = -3.0_f64;
            while dx <= 3.001 {
                if dx.abs() > 1e-5 || dy.abs() > 1e-5 {
                    let (fv, _) = face_vector(gray, w, h, x + dx * k, y + dy * k, k);
                    for (idx, refs) in [
                        (0usize, &FACES_MAMMOTH[..]),
                        (1usize, &FACES_LION[..]),
                        (2usize, &FACES_WOLF[..]),
                        (3usize, &FACES_RAT[..]),
                    ] {
                        for &ci in &CENTER_INDICES {
                            let dot = dot_f32(&refs[ci], &fv);
                            if dot > scores[idx] {
                                scores[idx] = dot;
                            }
                        }
                    }
                }
                dx += 1.5;
            }
            dy += 1.5;
        }
    }

    let mut ranked = [
        (Beast::Mammoth, scores[0]),
        (Beast::Lion, scores[1]),
        (Beast::Wolf, scores[2]),
        (Beast::Rat, scores[3]),
    ];
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    FaceMatch {
        beast: ranked[0].0,
        score: ranked[0].1,
        second: ranked[1].1,
        brightness,
    }
}

pub struct FrameRarity {
    pub tier: u8,
    pub conf: f32,
    pub colored: f32,
}

pub fn frame_rarity(img: &ImageView<'_>, x: f64, y: f64, k: f64) -> FrameRarity {
    let tier_by_hue: [(f32, f32, usize); 5] = [
        (0.0, 14.0, 4),
        (14.0, 62.0, 3),
        (175.0, 242.0, 1),
        (242.0, 305.0, 2),
        (330.0, 360.0, 4),
    ];
    let mut bins = [0.0_f32; 5];
    let mut samples = 0usize;
    let mut bright = 0usize;
    let mut visit = |px: f64, py: f64| {
        let (hh, ss, vv) = img.hsv_at(px, py);
        samples += 1;
        if vv > 0.62 && ss < 0.25 {
            bright += 1;
        }
        if vv < 0.45 || ss < 0.32 {
            return;
        }
        let weight = ((ss - 0.28) * 1.6).clamp(0.5, 1.25);
        for &(lo, hi, t) in &tier_by_hue {
            if hh >= lo && hh < hi {
                bins[t] += weight;
            } else if (hh - lo).abs() < 3.5 || (hh - hi).abs() < 3.5 {
                bins[t] += 0.35 * weight;
            }
        }
    };
    let step_px = k.max(1.0);
    let mut py = y - 60.0 * k;
    while py <= y - 12.0 * k + 1e-5 {
        let mut px = x - 29.0 * k;
        while px <= x - 17.0 * k + 1e-5 {
            visit(px, py);
            px += step_px;
        }
        let mut px = x + 17.0 * k;
        while px <= x + 29.0 * k + 1e-5 {
            visit(px, py);
            px += step_px;
        }
        py += step_px;
    }
    let mut py = y - 14.0 * k;
    while py <= y - 4.0 * k + 1e-5 {
        let mut px = x - 9.0 * k;
        while px <= x + 9.0 * k + 1e-5 {
            visit(px, py);
            px += step_px;
        }
        py += step_px;
    }
    let cf: f32 = bins.iter().sum();
    let sf = samples.max(1) as f32;
    if cf < 0.025 * sf {
        let conf = if cf < 0.008 * sf {
            0.85
        } else if (bright as f32) > 0.04 * sf {
            0.80
        } else {
            0.55
        };
        return FrameRarity {
            tier: 0,
            conf,
            colored: cf / sf,
        };
    }
    let mut tier = 1usize;
    for t in 2..5 {
        if bins[t] > bins[tier] {
            tier = t;
        }
    }
    FrameRarity {
        tier: tier as u8,
        conf: (0.5 + bins[tier] / cf.max(1e-5) / 2.0).min(1.0),
        colored: cf / sf,
    }
}

pub fn color_share<F>(img: &ImageView<'_>, x0: f64, y0: f64, x1: f64, y1: f64, mut test: F) -> f32
where
    F: FnMut(f32, f32, f32) -> bool,
{
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1 as isize).min(img.width as isize - 1);
    let by1 = (y1 as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return 0.0;
    }
    let mut n = 0usize;
    let mut hit = 0usize;
    for y in by0..=(by1 as usize) {
        for x in bx0..=(bx1 as usize) {
            n += 1;
            let (r, g, b) = img.rgb_at(x as isize, y as isize);
            let (hh, ss, vv) = hsv(r as f32, g as f32, b as f32);
            if test(hh, ss, vv) {
                hit += 1;
            }
        }
    }
    if n > 0 {
        (hit as f32) / (n as f32)
    } else {
        0.0
    }
}

#[inline]
pub fn color_share_rgb<F>(
    img: &ImageView<'_>,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    step: usize,
    test: F,
) -> f32
where
    F: Fn(u8, u8, u8) -> bool,
{
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1 as isize).min(img.width as isize - 1);
    let by1 = (y1 as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return 0.0;
    }
    let bx1 = bx1 as usize;
    let by1 = by1 as usize;
    let step = step.max(1);
    let mut n = 0usize;
    let mut hit = 0usize;
    for y in (by0..=by1).step_by(step) {
        let row_base = y * img.width * 4;
        for x in (bx0..=bx1).step_by(step) {
            n += 1;
            let o = row_base + x * 4;
            if test(img.data[o], img.data[o + 1], img.data[o + 2]) {
                hit += 1;
            }
        }
    }
    if n > 0 {
        (hit as f32) / (n as f32)
    } else {
        0.0
    }
}

#[inline]
pub fn is_jaws_rgb(r: u8, g: u8, b: u8) -> bool {
    if r < 141 || r < g || r < b {
        return false;
    }
    let ru = r as u32;
    let mn = g.min(b) as u32;
    let d = ru - mn;
    if 5 * d <= 2 * ru {
        return false;
    }
    if g >= b {
        15 * ((g - b) as u32) < 4 * d
    } else {
        3 * ((b - g) as u32) < d
    }
}

#[inline]
pub fn is_nest_rgb(r: u8, g: u8, b: u8) -> bool {
    if r < 115 || r <= g || g < b {
        return false;
    }
    let ru = r as u32;
    let d = (r - b) as u32;
    let dg = (g - b) as u32;
    2 * d > ru && 3 * d <= 10 * dg && 30 * dg <= 23 * d
}

#[inline]
pub fn is_banner_red_rgb(r: u8, g: u8, b: u8) -> bool {
    if r < 103 || r < g || r < b {
        return false;
    }
    let ru = r as u32;
    let mn = g.min(b) as u32;
    let d = ru - mn;
    if 20 * d <= 9 * ru {
        return false;
    }
    if g >= b {
        15 * ((g - b) as u32) < 4 * d
    } else {
        3 * ((b - g) as u32) < d
    }
}

#[inline]
pub fn is_jaws(h: f32, s: f32, v: f32) -> bool {
    (h < 16.0 || h > 340.0) && s > 0.40 && v > 0.55
}

#[inline]
pub fn is_orange(h: f32, s: f32, v: f32) -> bool {
    (18.0..=46.0).contains(&h) && s > 0.60 && v > 0.55
}

#[inline]
pub fn is_gold(h: f32, s: f32, v: f32) -> bool {
    (32.0..=56.0).contains(&h) && s > 0.38 && s < 0.90 && v > 0.50
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScannedCellKind {
    Office,
    Locked,
    Inmate,
    #[default]
    Empty,
}

#[derive(Clone, Copy, Debug)]
pub struct ScannedCell {
    pub n: u8,
    pub kind: ScannedCellKind,
    pub beast: Beast,
    pub tier: u8,
    pub conf: f32,
    pub x: f64,
    pub y: f64,
    pub k: f64,
    pub nest: bool,
    pub jaws: bool,
    pub ghost: bool,
    pub event: bool,
}

impl Default for ScannedCell {
    fn default() -> Self {
        Self {
            n: 0,
            kind: ScannedCellKind::Empty,
            beast: Beast::Mammoth,
            tier: 0,
            conf: 0.0,
            x: 0.0,
            y: 0.0,
            k: 0.0,
            nest: false,
            jaws: false,
            ghost: false,
            event: false,
        }
    }
}

pub fn read_cell(img: &ImageView<'_>, loc: &BoardLoc, n: u8) -> ScannedCell {
    let mut slot = CellScratchSlot::with_capacity();
    read_cell_with_slot(img, loc, n, &mut slot)
}

pub fn read_cell_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    n: u8,
    slot: &mut CellScratchSlot,
) -> ScannedCell {
    if n == 0 {
        let (ox, oy) = loc.at(0.0, 0.0);
        return ScannedCell {
            n: 0,
            kind: ScannedCellKind::Office,
            beast: Beast::Mammoth,
            tier: 0,
            conf: 1.0,
            x: ox,
            y: oy,
            k: loc.scale_at(0.0, 0.0),
            nest: false,
            jaws: false,
            ghost: false,
            event: false,
        };
    }
    let (u, v) = RING[n as usize];
    let uf = u as f64;
    let vf = v as f64;
    let (x, y) = loc.at(uf, vf);
    let k = loc.scale_at(uf, vf);
    let lock = padlock_at(img, x, y, k);
    let nest = color_share_rgb(
        img,
        x - 15.0 * k,
        y - 28.0 * k,
        x + 15.0 * k,
        y - 6.0 * k,
        1,
        is_nest_rgb,
    ) > 0.07;

    // Fast path only for high-confidence padlocks; borderline padlocks are cross-checked
    // against face template score and card frame rarity below.
    if lock > 0.68 {
        return ScannedCell {
            n,
            kind: ScannedCellKind::Locked,
            beast: Beast::Mammoth,
            tier: 0,
            conf: lock,
            x,
            y,
            k,
            nest,
            jaws: false,
            ghost: false,
            event: false,
        };
    }

    let wi = img.width as isize;
    let hi = img.height as isize;
    let bx0 = (((x + k * (-3.0 + FACE_X0)).floor() as isize) - 1).clamp(0, wi - 1) as usize;
    let bx1 = (((x + k * (3.0 + FACE_X1)).ceil() as isize) + 2).clamp(0, wi - 1) as usize;
    let by0 = (((y + k * (-3.0 + FACE_Y0)).floor() as isize) - 1).clamp(0, hi - 1) as usize;
    let by1 = (((y + k * (3.0 + FACE_Y1)).ceil() as isize) + 2).clamp(0, hi - 1) as usize;
    let pw = (bx1 - bx0 + 1).max(2);
    let ph = (by1 - by0 + 1).max(2);
    let row_buf_len = (ph + 2) * pw;
    let patch_len = pw * ph;

    let CellScratchSlot { row_buf, patch } = slot;
    if row_buf.len() < row_buf_len {
        row_buf.resize(row_buf_len, 0.0);
    }
    if patch.len() < patch_len {
        patch.resize(patch_len, 0.0);
    }
    let row_buf = &mut row_buf[..row_buf_len];
    let patch = &mut patch[..patch_len];

    let w = img.width;
    for ry in 0..(ph + 2) {
        let cy = ((by0 as isize) + (ry as isize) - 1).clamp(0, hi - 1) as usize;
        let row_base = cy * w * 4;
        let dst = &mut row_buf[ry * pw..(ry + 1) * pw];
        let x_m1 = ((bx0 as isize) - 1).max(0) as usize;
        let o_m1 = row_base + x_m1 * 4;
        let mut l0 = 0.299 * (img.data[o_m1] as f32)
            + 0.587 * (img.data[o_m1 + 1] as f32)
            + 0.114 * (img.data[o_m1 + 2] as f32);
        let o_0 = row_base + bx0 * 4;
        let mut l1 = 0.299 * (img.data[o_0] as f32)
            + 0.587 * (img.data[o_0 + 1] as f32)
            + 0.114 * (img.data[o_0 + 2] as f32);
        for px in 0..pw {
            let cx2 = (bx0 + px + 1).min(w - 1);
            let o2 = row_base + cx2 * 4;
            let l2 = 0.299 * (img.data[o2] as f32)
                + 0.587 * (img.data[o2 + 1] as f32)
                + 0.114 * (img.data[o2 + 2] as f32);
            dst[px] = l0 + l1 + l2;
            l0 = l1;
            l1 = l2;
        }
    }
    const INV9: f32 = 1.0 / 9.0;
    for py in 0..ph {
        let r0 = &row_buf[py * pw..(py + 1) * pw];
        let r1 = &row_buf[(py + 1) * pw..(py + 2) * pw];
        let r2 = &row_buf[(py + 2) * pw..(py + 3) * pw];
        let out = &mut patch[py * pw..(py + 1) * pw];
        for px in 0..pw {
            out[px] = (r0[px] + r1[px] + r2[px]) * INV9;
        }
    }
    let face = face_match(patch, pw, ph, x - (bx0 as f64), y - (by0 as f64), k);
    let frame = frame_rarity(img, x, y, k);

    // If a borderline padlock was found, only override it if the face template & frame clearly show an inmate
    if lock > 0.40 && !(face.score > 0.62 && (frame.colored > 0.08 || (face.score - face.second) > 0.12)) {
        return ScannedCell {
            n,
            kind: ScannedCellKind::Locked,
            beast: Beast::Mammoth,
            tier: 0,
            conf: lock,
            x,
            y,
            k,
            nest,
            jaws: false,
            ghost: false,
            event: false,
        };
    }

    let jaws = color_share_rgb(
        img,
        x - 12.0 * k,
        y - 70.0 * k,
        x + 12.0 * k,
        y - 50.0 * k,
        1,
        is_jaws_rgb,
    ) > 0.14
        && color_share_rgb(
            img,
            x - 10.0 * k,
            y - 55.0 * k,
            x + 10.0 * k,
            y - 48.0 * k,
            1,
            is_jaws_rgb,
        ) > 0.18;
    let ghost = face.brightness > 175.0 && face.score < 0.55;

    if face.score > 0.42 || ghost || frame.colored > 0.12 {
        let margin = face.score - face.second;
        let c1 = (face.score - 0.30) * 1.6;
        let c2 = 0.40 + margin * 3.0;
        let raw_conf = c1.min(c2).min(frame.conf).clamp(0.05, 1.0);
        let conf = raw_conf * if ghost { 0.5 } else { 1.0 };
        return ScannedCell {
            n,
            kind: ScannedCellKind::Inmate,
            beast: face.beast,
            tier: frame.tier,
            conf,
            x,
            y,
            k,
            nest,
            jaws,
            ghost,
            event: false,
        };
    }

    ScannedCell {
        n,
        kind: ScannedCellKind::Empty,
        beast: Beast::Mammoth,
        tier: 0,
        conf: 0.40,
        x,
        y,
        k,
        nest,
        jaws,
        ghost,
        event: false,
    }
}

#[inline]
pub fn is_gold_rgb(r: u8, g: u8, b: u8) -> bool {
    if r < 128 || r <= g || g < b {
        return false;
    }
    let ru = r as u32;
    let d = (r - b) as u32;
    let dg = (g - b) as u32;
    d * 50 > 19 * ru && d * 10 < 9 * ru && 8 * d <= 15 * dg && 15 * dg <= 14 * d
}

#[inline]
pub fn is_orange_rgb(r: u8, g: u8, b: u8) -> bool {
    if r < 141 || r <= g || g < b {
        return false;
    }
    let ru = r as u32;
    let d = (r - b) as u32;
    let dg = (g - b) as u32;
    5 * d > 3 * ru && 3 * d <= 10 * dg && 30 * dg <= 23 * d
}

#[derive(Clone, Copy, Debug)]
pub struct WardenRead {
    pub cell: u8,
    pub conf: f32,
    pub box_coords: [i32; 4],
}

pub fn find_warden_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    slot: &mut ScanScratchSlot,
) -> Option<WardenRead> {
    let corners = [(0.15, -0.15), (4.85, -0.15), (4.85, 2.3), (0.15, 2.3)]
        .map(|(u, v)| loc.at(u, v));
    let min_x = corners.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
    let max_x = corners.iter().map(|c| c.0).fold(f64::NEG_INFINITY, f64::max);
    let min_y = corners.iter().map(|c| c.1).fold(f64::INFINITY, f64::min);
    let max_y = corners.iter().map(|c| c.1).fold(f64::NEG_INFINITY, f64::max);

    // Central courtyard (u in 1.25..3.75, v in 0.72..1.40) is > 0.55 grid units from all ARROW points
    let (ix0, iy0) = loc.at(1.25, 0.72);
    let (ix1, iy1) = loc.at(3.75, 1.40);
    let skip_rect = if ix1 > ix0 + 8.0 && iy1 > iy0 + 8.0 {
        Some([
            ix0.ceil().max(0.0) as usize,
            iy0.ceil().max(0.0) as usize,
            ix1.floor().max(0.0) as usize,
            iy1.floor().max(0.0) as usize,
        ])
    } else {
        None
    };

    let k = loc.s;
    let step_y = ((3.0 * k) as usize).clamp(1, 4);
    let found = blobs_rgb_strided_with_slot(
        img,
        min_x,
        min_y,
        max_x,
        max_y,
        skip_rect,
        step_y,
        slot,
        is_gold_rgb,
    );
    let mut groups: Vec<(f64, f64, usize, [i32; 4])> = Vec::new();
    for b in found {
        if (b.n as f64) < 6.0 * k * k {
            continue;
        }
        if let Some(g) = groups
            .iter_mut()
            .find(|q| (q.0 - b.cx).hypot(q.1 - b.cy) < 22.0 * k)
        {
            let tot = (g.2 + b.n) as f64;
            g.0 = (g.0 * (g.2 as f64) + b.cx * (b.n as f64)) / tot;
            g.1 = (g.1 * (g.2 as f64) + b.cy * (b.n as f64)) / tot;
            g.2 += b.n;
            g.3 = [
                g.3[0].min(b.x0),
                g.3[1].min(b.y0),
                g.3[2].max(b.x1),
                g.3[3].max(b.y1),
            ];
        } else {
            groups.push((b.cx, b.cy, b.n, [b.x0, b.y0, b.x1, b.y1]));
        }
    }

    let mut best: Option<(u8, f64, f64, [i32; 4])> = None;
    for (cx, cy, n, bx) in groups {
        let area = (n as f64) / (k * k);
        if !(70.0..=1600.0).contains(&area) {
            continue;
        }
        let (gu, gv) = loc.grid_of(cx, cy);
        let mut cell = 0u8;
        let mut dist = f64::INFINITY;
        for (idx, &(au, av)) in ARROW.iter().enumerate() {
            let d = (au - gu).hypot(av - gv);
            if d < dist {
                dist = d;
                cell = idx as u8;
            }
        }
        if dist > 0.55 {
            continue;
        }
        let fit = area / 500.0 - dist;
        if best.map_or(true, |b| fit > b.1) {
            best = Some((cell, fit, dist, bx));
        }
    }

    let blob_res = best.map(|(cell, _, dist, box_coords)| WardenRead {
        cell,
        conf: (1.0 - dist).clamp(0.2, 1.0) as f32,
        box_coords,
    });

    if blob_res.as_ref().is_some_and(|w| w.conf >= 0.58) {
        return blob_res;
    }

    // Fallback: direct anchor evaluation across all 16 ARROW positions
    let mut best_anchor: Option<(u8, f32, [i32; 4])> = None;
    for (idx, &(au, av)) in ARROW.iter().enumerate() {
        let (ax, ay) = loc.at(au, av);
        let rx = 18.0 * k;
        let ry = 20.0 * k;
        let share = color_share(img, ax - rx, ay - ry, ax + rx, ay + ry, is_gold);
        if share > 0.08 && best_anchor.map_or(true, |b| share > b.1) {
            best_anchor = Some((
                idx as u8,
                share,
                [
                    (ax - rx).round() as i32,
                    (ay - ry).round() as i32,
                    (ax + rx).round() as i32,
                    (ay + ry).round() as i32,
                ],
            ));
        }
    }

    if let Some((cell, share, box_coords)) = best_anchor {
        let anchor_conf = (share * 3.2).clamp(0.25, 0.85);
        if blob_res.as_ref().map_or(true, |w| anchor_conf > w.conf) {
            return Some(WardenRead {
                cell,
                conf: anchor_conf,
                box_coords,
            });
        }
    }

    blob_res
}

fn event_band_rects(loc: &BoardLoc) -> [[f64; 4]; 4] {
    let c1 = loc.at(1.0, 0.0);
    let c5 = loc.at(5.0, 0.0);
    let k_top = loc.scale_at(1.0, 0.0).max(loc.scale_at(5.0, 0.0));
    let top = [
        c1.0.min(c5.0) - 115.0 * k_top,
        c1.1.min(c5.1) - 155.0 * k_top,
        c1.0.max(c5.0) + 115.0 * k_top,
        c1.1.max(c5.1) + 80.0 * k_top,
    ];

    let c13 = loc.at(0.0, 3.0);
    let c8 = loc.at(5.0, 3.0);
    let k_bot = loc.scale_at(0.0, 3.0).max(loc.scale_at(5.0, 3.0));
    let bot = [
        c13.0.min(c8.0) - 115.0 * k_bot,
        c13.1.min(c8.1) - 155.0 * k_bot,
        c13.0.max(c8.0) + 115.0 * k_bot,
        c13.1.max(c8.1) + 80.0 * k_bot,
    ];

    let c15 = loc.at(0.0, 1.0);
    let c14 = loc.at(0.0, 2.0);
    let k_left = loc.scale_at(0.0, 1.0).max(loc.scale_at(0.0, 2.0));
    let left = [
        c14.0.min(c15.0) - 115.0 * k_left,
        c15.1.min(c14.1) - 155.0 * k_left,
        c14.0.max(c15.0) + 115.0 * k_left,
        c15.1.max(c14.1) + 80.0 * k_left,
    ];

    let c6 = loc.at(5.0, 1.0);
    let c7 = loc.at(5.0, 2.0);
    let k_right = loc.scale_at(5.0, 1.0).max(loc.scale_at(5.0, 2.0));
    let right = [
        c6.0.min(c7.0) - 115.0 * k_right,
        c6.1.min(c7.1) - 155.0 * k_right,
        c6.0.max(c7.0) + 115.0 * k_right,
        c6.1.max(c7.1) + 80.0 * k_right,
    ];

    [top, bot, left, right]
}

fn find_event_blobs_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    slot: &mut ScanScratchSlot,
) -> Vec<Blob> {
    let bands = event_band_rects(loc);
    let step_y = ((6.0 * loc.s) as usize).clamp(1, 8);
    let mut out = Vec::with_capacity(32);
    for [x0, y0, x1, y1] in bands {
        out.extend(blobs_rgb_strided_with_slot(
            img,
            x0,
            y0,
            x1,
            y1,
            None,
            step_y,
            slot,
            is_orange_rgb,
        ));
    }
    out
}

fn match_events_from_blobs(
    found: &[Blob],
    loc: &BoardLoc,
    cells: &[ScannedCell; 16],
    warden: Option<&WardenRead>,
) -> u16 {
    let k = loc.s;
    let mut out = 0u16;
    for b in found {
        let bh = b.h as f64;
        let bw = b.w as f64;
        if bh < 12.0 * k || bh > 36.0 * k || bh < 1.8 * bw {
            continue;
        }
        if let Some(w) = warden {
            let ab = w.box_coords;
            if b.cx > (ab[0] as f64) - 4.0 * k
                && b.cx < (ab[2] as f64) + 4.0 * k
                && b.cy > (ab[1] as f64) - 4.0 * k
                && b.cy < (ab[3] as f64) + 4.0 * k
            {
                continue;
            }
        }
        let mut nearest = 0u8;
        let mut dist = f64::INFINITY;
        for c in cells.iter().skip(1) {
            let d = (b.cx - c.x).hypot(b.cy - (c.y - 38.0 * c.k)) / c.k;
            if d < dist {
                dist = d;
                nearest = c.n;
            }
        }
        if dist > 35.0 && dist < 95.0 {
            out |= 1u16 << nearest;
        }
    }
    out
}

pub fn find_events_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    cells: &[ScannedCell; 16],
    warden: Option<&WardenRead>,
    slot: &mut ScanScratchSlot,
) -> u16 {
    let found = find_event_blobs_with_slot(img, loc, slot);
    match_events_from_blobs(&found, loc, cells, warden)
}

#[derive(Clone, Debug)]
pub struct Word {
    pub glyphs: Vec<Blob>,
    pub x0: i32,
    pub x1: i32,
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub words: Vec<Word>,
    pub glyphs: Vec<Blob>,
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub mid: f64,
    pub h: i32,
    pub glyphs: Vec<Blob>,
    pub pieces: Vec<Piece>,
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
}

fn group_blobs_into_lines(mut found: Vec<Blob>, min_h: f64, max_h: f64) -> Vec<Line> {
    found.retain(|b| {
        (b.h as f64) >= min_h * 0.45
            && (b.h as f64) <= max_h
            && (b.w as f64) <= max_h * 1.2
            && b.n >= 4
    });

    found.sort_by(|a, b| {
        let ma = (a.y0 + a.y1) as f64 * 0.5;
        let mb = (b.y0 + b.y1) as f64 * 0.5;
        ma.partial_cmp(&mb).unwrap()
    });

    let mut lines: Vec<Line> = Vec::new();
    for g in found {
        let mid = (g.y0 + g.y1) as f64 * 0.5;
        if let Some(line) = lines
            .iter_mut()
            .find(|l| (l.mid - mid).abs() < 4.0_f64.max((l.h as f64) * 0.55))
        {
            line.glyphs.push(g);
            line.h = line.h.max(g.h);
        } else {
            lines.push(Line {
                mid,
                h: g.h,
                glyphs: vec![g],
                pieces: Vec::new(),
                x0: g.x0,
                x1: g.x1,
                y0: g.y0,
                y1: g.y1,
            });
        }
    }

    let mut out = Vec::new();
    for mut line in lines {
        line.glyphs.sort_by_key(|g| g.x0);
        let mut tall: Vec<i32> = line.glyphs.iter().map(|g| g.h).collect();
        tall.sort_unstable();
        if let Some(&h75) = tall.get((tall.len() * 3) / 4) {
            line.h = h75;
        }
        let lh = line.h as f64;
        line.glyphs
            .retain(|g| !((g.h as f64) < 0.32 * lh && (g.n as f64) < 0.06 * lh * lh));
        if line.glyphs.is_empty() {
            continue;
        }

        let mut merged: Vec<Blob> = Vec::new();
        for g in line.glyphs {
            if let Some(prev) = merged.last_mut() {
                let overlap = prev.x1.min(g.x1) - prev.x0.max(g.x0) + 1;
                if (overlap as f64) > 0.5 * (prev.w.min(g.w) as f64) {
                    prev.x0 = prev.x0.min(g.x0);
                    prev.x1 = prev.x1.max(g.x1);
                    prev.y0 = prev.y0.min(g.y0);
                    prev.y1 = prev.y1.max(g.y1);
                    prev.w = prev.x1 - prev.x0 + 1;
                    prev.h = prev.y1 - prev.y0 + 1;
                    prev.n += g.n;
                    continue;
                }
            }
            merged.push(g);
        }
        line.glyphs = merged;

        let mut pieces: Vec<Piece> = Vec::new();
        let mut last_x1: Option<i32> = None;
        for g in &line.glyphs {
            let gap = last_x1.map_or(f64::INFINITY, |lx| (g.x0 - lx) as f64);
            if pieces.is_empty() || gap > 2.0 * lh {
                pieces.push(Piece {
                    words: Vec::new(),
                    glyphs: Vec::new(),
                    x0: g.x0,
                    x1: g.x1,
                    y0: g.y0,
                    y1: g.y1,
                });
            }
            let piece = pieces.last_mut().unwrap();
            if piece.words.is_empty() || gap > lh * 0.36 {
                piece.words.push(Word {
                    glyphs: Vec::new(),
                    x0: g.x0,
                    x1: g.x1,
                });
            }
            let word = piece.words.last_mut().unwrap();
            word.glyphs.push(*g);
            word.x1 = word.x1.max(g.x1);
            piece.glyphs.push(*g);
            piece.x1 = piece.x1.max(g.x1);
            piece.y0 = piece.y0.min(g.y0);
            piece.y1 = piece.y1.max(g.y1);
            last_x1 = Some(g.x1);
        }
        line.x0 = line.glyphs[0].x0;
        line.x1 = line.glyphs.iter().map(|g| g.x1).max().unwrap();
        line.y0 = line.glyphs.iter().map(|g| g.y0).min().unwrap();
        line.y1 = line.glyphs.iter().map(|g| g.y1).max().unwrap();
        line.pieces = pieces;
        out.push(line);
    }
    out
}

pub fn text_lines_radii_with_slot(
    img: &ImageView<'_>,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    min_h: f64,
    max_h: f64,
    radii: &[usize],
    sat_limit: f32,
    slot: &mut ScanScratchSlot,
) -> Vec<Vec<Line>> {
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1.ceil() as isize).min(img.width as isize - 1);
    let by1 = (y1.ceil() as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return vec![Vec::new(); radii.len()];
    }
    let bx1 = bx1 as usize;
    let by1 = by1 as usize;
    let w = bx1 - bx0 + 1;
    let h = by1 - by0 + 1;
    if w < 4 || h < 4 || radii.is_empty() {
        return vec![Vec::new(); radii.len()];
    }

    let ScanScratchSlot {
        sat,
        elig_coords,
        hit_coords,
        mask,
        stack,
    } = slot;

    let r_max = radii.iter().copied().max().unwrap_or(1).max(1);
    let sw = w + 2 * r_max;
    let sh = h + 2 * r_max;
    let sat_stride = sw + 1;
    let sat_len = (sh + 1) * sat_stride;
    if sat.len() < sat_len {
        sat.resize(sat_len, 0);
    }
    sat[..sat_stride].fill(0);

    elig_coords.clear();
    let sat_mul = (sat_limit * 256.0).round() as u32;

    let fill_border_row = |sat: &mut [u32], sy: usize, py: usize| {
        let row_base = (py * img.width + bx0) * 4;
        let data = &img.data[row_base..row_base + w * 4];
        let prev_row = sy * sat_stride;
        let cur_row = (sy + 1) * sat_stride;
        sat[cur_row] = 0;
        let mut row_sum = 0u32;
        let l_q0 =
            (77 * (data[0] as u32) + 150 * (data[1] as u32) + 29 * (data[2] as u32)) >> 6;
        for sx in 0..r_max {
            row_sum += l_q0;
            sat[cur_row + sx + 1] = sat[prev_row + sx + 1] + row_sum;
        }
        for x in 0..w {
            let o = x * 4;
            let l_q = (77 * (data[o] as u32)
                + 150 * (data[o + 1] as u32)
                + 29 * (data[o + 2] as u32))
                >> 6;
            row_sum += l_q;
            sat[cur_row + r_max + x + 1] = sat[prev_row + r_max + x + 1] + row_sum;
        }
        let o_last = (w - 1) * 4;
        let l_qn = (77 * (data[o_last] as u32)
            + 150 * (data[o_last + 1] as u32)
            + 29 * (data[o_last + 2] as u32))
            >> 6;
        for sx in (r_max + w)..sw {
            row_sum += l_qn;
            sat[cur_row + sx + 1] = sat[prev_row + sx + 1] + row_sum;
        }
    };

    for sy in 0..r_max {
        fill_border_row(sat, sy, by0);
    }

    for y in 0..h {
        let sy = y + r_max;
        let py = by0 + y;
        let row_base = (py * img.width + bx0) * 4;
        let data = &img.data[row_base..row_base + w * 4];
        let prev_row = sy * sat_stride;
        let cur_row = (sy + 1) * sat_stride;
        sat[cur_row] = 0;
        let mut row_sum = 0u32;
        let l_q0 =
            (77 * (data[0] as u32) + 150 * (data[1] as u32) + 29 * (data[2] as u32)) >> 6;
        for sx in 0..r_max {
            row_sum += l_q0;
            sat[cur_row + sx + 1] = sat[prev_row + sx + 1] + row_sum;
        }
        for x in 0..w {
            let o = x * 4;
            let rr = data[o];
            let gg = data[o + 1];
            let bb = data[o + 2];
            let l_256 = 77 * (rr as u32) + 150 * (gg as u32) + 29 * (bb as u32);
            row_sum += l_256 >> 6;
            sat[cur_row + r_max + x + 1] = sat[prev_row + r_max + x + 1] + row_sum;
            let mx = rr.max(gg).max(bb);
            if mx >= 45 && l_256 >= 45 * 256 {
                let mn = rr.min(gg).min(bb);
                if ((mx - mn) as u32) * 256 < sat_mul * (mx as u32) {
                    elig_coords.push((x as u16, y as u16, l_256 as u16));
                }
            }
        }
        let o_last = (w - 1) * 4;
        let l_qn = (77 * (data[o_last] as u32)
            + 150 * (data[o_last + 1] as u32)
            + 29 * (data[o_last + 2] as u32))
            >> 6;
        for sx in (r_max + w)..sw {
            row_sum += l_qn;
            sat[cur_row + sx + 1] = sat[prev_row + sx + 1] + row_sum;
        }
    }

    if elig_coords.is_empty() {
        return vec![Vec::new(); radii.len()];
    }

    for sy in (r_max + h)..sh {
        fill_border_row(sat, sy, by1);
    }

    let n = w * h;
    if mask.len() < n {
        mask.resize(n, 0);
    }

    let mut results = Vec::with_capacity(radii.len());
    for &rad in radii {
        let r = rad.max(1);
        let span = 2 * r + 1;
        let inv_area_scaled = 0.25_f32 / ((span * span) as f32);
        hit_coords.clear();
        for &(ex_u16, ey_u16, l_256) in elig_coords.iter() {
            let ex = ex_u16 as usize;
            let ey = ey_u16 as usize;
            let cx = ex + r_max;
            let cy = ey + r_max;
            let y0s = (cy - r) * sat_stride;
            let y1s = (cy + r + 1) * sat_stride;
            let x0s = cx - r;
            let x1s = cx + r + 1;
            let box_sum =
                (sat[y1s + x1s] + sat[y0s + x0s]).saturating_sub(sat[y0s + x1s] + sat[y1s + x0s]);
            let around_val = (box_sum as f32) * inv_area_scaled;
            let l = (l_256 as f32) * (1.0 / 256.0);
            if l - around_val >= 16.0_f32.max(0.3 * around_val) {
                mask[ey * w + ex] = 1;
                hit_coords.push((ex_u16, ey_u16));
            }
        }
        let found = if hit_coords.is_empty() {
            Vec::new()
        } else {
            blobs_from_mask_hits(&mut mask[..n], hit_coords, stack, w, h, bx0, by0)
        };
        results.push(group_blobs_into_lines(found, min_h, max_h));
    }
    results
}

pub fn glyph_vector(img: &ImageView<'_>, g: &Blob, line_h: i32) -> [f32; 140] {
    let mut v = [0.0_f32; 140];
    let scale = (g.h as f64).max((line_h as f64) * 0.7) / (GLYPH_H as f64);
    let cx = (g.x0 + g.x1) as f64 * 0.5;
    let cy = (g.y0 + g.y1) as f64 * 0.5;
    let sub = (scale.round() as usize).clamp(1, 4);
    let mut mean = 0.0_f32;
    for j in 0..GLYPH_H {
        for i in 0..GLYPH_W {
            let mut val = 0.0_f32;
            for b in 0..sub {
                for a in 0..sub {
                    let sx = cx
                        + ((i as f64) + ((a as f64) + 0.5) / (sub as f64) - (GLYPH_W as f64) * 0.5)
                            * scale;
                    let sy = cy
                        + ((j as f64) + ((b as f64) + 0.5) / (sub as f64) - (GLYPH_H as f64) * 0.5)
                            * scale;
                    val += img.sample_sharp(sx, sy);
                }
            }
            val /= (sub * sub) as f32;
            v[j * GLYPH_W + i] = val;
            mean += val;
        }
    }
    mean /= 140.0;
    normalize_zero_mean_in_place(&mut v, mean);
    v
}

pub fn digit_scores(img: &ImageView<'_>, g: &Blob, line_h: i32) -> [f32; 10] {
    let v = glyph_vector(img, g, line_h);
    let mut out = [-1.0_f32; 10];
    for (d, refs) in DIGITS.iter().enumerate() {
        for r in *refs {
            let dot = dot_f32(r, &v);
            if dot > out[d] {
                out[d] = dot;
            }
        }
    }
    out
}

const ALLOWED_CELLS: [u32; 15] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
const ALLOWED_DAYS: [u32; 41] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
];

#[inline]
const fn u32_digits(v: u32) -> ([usize; 2], usize) {
    if v >= 10 {
        ([(v / 10) as usize, (v % 10) as usize], 2)
    } else {
        ([v as usize, 0], 1)
    }
}

pub fn read_number(
    img: &ImageView<'_>,
    glyphs: &[Blob],
    line_h: i32,
    allowed: &[u32],
) -> Vec<(u32, f32)> {
    let n_glyphs = glyphs.len();
    if n_glyphs == 0 || n_glyphs > 2 {
        return Vec::new();
    }
    let mut per = [[-1.0_f32; 10]; 2];
    for (i, g) in glyphs.iter().enumerate() {
        per[i] = digit_scores(img, g, line_h);
    }
    let mut out = Vec::with_capacity(allowed.len());
    for &value in allowed {
        let (digits, dlen) = u32_digits(value);
        if dlen != n_glyphs {
            continue;
        }
        let mut sc = 0.0_f32;
        for i in 0..dlen {
            sc += per[i][digits[i]];
        }
        out.push((value, sc / (dlen as f32)));
    }
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    out
}

pub fn locate_by_labels_with_slot(
    img: &ImageView<'_>,
    slot: &mut ScanScratchSlot,
) -> Option<BoardLoc> {
    let unit = BoardLoc {
        s: 1.0,
        tx: 0.0,
        ty: 0.0,
        score: 0.0,
        w: img.width,
        h: img.height,
        by_labels: true,
        cells_matched: 0,
    };
    let spot: [(f64, f64); 16] = std::array::from_fn(|n| {
        if n == 0 {
            return (0.0, 0.0);
        }
        let (u, v) = RING[n];
        let (x, y) = unit.at(u as f64, v as f64);
        let k = unit.scale_at(u as f64, v as f64);
        let (ox, oy) = if v == 0 {
            (0.0, -88.0)
        } else if v == 3 {
            (0.0, 22.0)
        } else if u == 5 {
            (63.0, -38.0)
        } else {
            (-62.0, -39.0)
        };
        (x + ox * k, y + oy * k)
    });
    let cell_w = unit.at(1.0, 0.0).0 - unit.at(0.0, 0.0).0;
    let mut seen: Vec<(usize, f64, f64)> = Vec::new();
    let rad = ((40.0_f64 / 3.0).round() as usize).max(3);

    for line in text_lines_radii_with_slot(
        img,
        0.0,
        0.0,
        (img.width - 1) as f64,
        (img.height - 1) as f64,
        6.0,
        40.0,
        &[rad],
        0.34,
        slot,
    )
    .into_iter()
    .next()
    .unwrap_or_default()
    {
        for p in &line.pieces {
            if p.words.len() != 1
                || p.words[0].glyphs.is_empty()
                || p.words[0].glyphs.len() > 2
            {
                continue;
            }
            let r = read_number(img, &p.words[0].glyphs, line.h, &ALLOWED_CELLS);
            if let Some(&(val, score)) = r.first() {
                if score > 0.70 {
                    seen.push((
                        (val - 1) as usize,
                        (p.x0 + p.x1) as f64 * 0.5,
                        (p.y0 + p.y1) as f64 * 0.5,
                    ));
                }
            }
        }
    }

    let mut best: Option<(f64, usize, Vec<(usize, f64, f64)>)> = None;
    for i in 0..seen.len() {
        for j in (i + 1)..seen.len() {
            let a = seen[i];
            let b = seen[j];
            if a.0 == b.0 {
                continue;
            }
            let dr = (spot[b.0].0 - spot[a.0].0).hypot(spot[b.0].1 - spot[a.0].1);
            if dr < cell_w * 0.9 {
                continue;
            }
            let s = (b.1 - a.1).hypot(b.2 - a.2) / dr;
            if !(0.2..=6.0).contains(&s) {
                continue;
            }
            let tx = a.1 - s * spot[a.0].0;
            let ty = a.2 - s * spot[a.0].1;
            let inl: Vec<(usize, f64, f64)> = seen
                .iter()
                .copied()
                .filter(|c| {
                    (s * spot[c.0].0 + tx - c.1).hypot(s * spot[c.0].1 + ty - c.2)
                        < 0.18 * cell_w * s
                })
                .collect();
            let mut uniq = 0u16;
            for c in &inl {
                uniq |= 1u16 << c.0;
            }
            let cells = uniq.count_ones() as usize;
            if best.as_ref().map_or(true, |b| cells > b.1) {
                best = Some((s, cells, inl));
            }
        }
    }

    let (best_s, cells_matched, inl) = best?;
    if cells_matched < 5 {
        return None;
    }
    let m = inl.len() as f64;
    let rx = inl.iter().map(|c| spot[c.0].0).sum::<f64>() / m;
    let ry = inl.iter().map(|c| spot[c.0].1).sum::<f64>() / m;
    let px = inl.iter().map(|c| c.1).sum::<f64>() / m;
    let py = inl.iter().map(|c| c.2).sum::<f64>() / m;
    let mut num = 0.0_f64;
    let mut den = 0.0_f64;
    for c in &inl {
        num += (spot[c.0].0 - rx) * (c.1 - px) + (spot[c.0].1 - ry) * (c.2 - py);
        den += (spot[c.0].0 - rx).powi(2) + (spot[c.0].1 - ry).powi(2);
    }
    let s = if den > 0.0 { num / den } else { best_s };
    Some(BoardLoc {
        s,
        tx: px - s * rx,
        ty: py - s * ry,
        score: ((cells_matched as f64) / 15.0).min(1.0),
        w: img.width,
        h: img.height,
        by_labels: true,
        cells_matched,
    })
}

#[derive(Clone, Copy, Debug)]
pub struct ChainRead {
    pub banner: f32,
    pub targets: FixedVec<u8, 16>,
}

#[inline]
fn chain_banner_score(img: &ImageView<'_>, loc: &BoardLoc) -> f32 {
    let step = ((2.0 * loc.s).round() as usize).clamp(1, 4);
    let mut banner = 0.0_f32;
    let mut v = 3.9_f64;
    while v <= 5.01 {
        let (x0, y0) = loc.at(1.0, v);
        let (x1, y1) = loc.at(4.0, v + 0.15);
        let sh = color_share_rgb(
            img,
            x0.min(x1),
            y0.min(y1),
            x0.max(x1),
            y0.max(y1),
            step,
            is_banner_red_rgb,
        );
        if sh > banner {
            banner = sh;
        }
        v += 0.15;
    }
    banner
}

pub fn read_chain_screen(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    cells: &[ScannedCell; 16],
    warden: Option<&WardenRead>,
) -> Option<ChainRead> {
    read_chain_screen_with_banner(img, loc, cells, warden, chain_banner_score(img, loc))
}

fn read_chain_screen_with_banner(
    img: &ImageView<'_>,
    _loc: &BoardLoc,
    cells: &[ScannedCell; 16],
    warden: Option<&WardenRead>,
    banner: f32,
) -> Option<ChainRead> {
    if banner < 0.20 {
        return None;
    }
    let mut targets = FixedVec::new();
    for c in cells.iter().skip(1) {
        if warden.map_or(false, |w| c.n == w.cell) {
            continue;
        }
        let k = c.k;
        let step = (k.round() as usize).clamp(1, 2);
        if color_share_rgb(
            img,
            c.x - 40.0 * k,
            c.y - 80.0 * k,
            c.x + 40.0 * k,
            c.y - 50.0 * k,
            step,
            is_banner_red_rgb,
        ) > 0.15
        {
            targets.push(c.n);
        }
    }
    if targets.is_empty() {
        None
    } else {
        Some(ChainRead { banner, targets })
    }
}

pub fn prefix_vector(
    img: &ImageView<'_>,
    piece: &Piece,
    end: f64,
    dy0: i32,
    dy1: i32,
) -> [f32; 400] {
    let g0 = &piece.glyphs[0];
    let x0 = g0.x0 as f64;
    let x1 = x0 + end * (g0.h as f64);
    let y0 = (g0.y0 + dy0) as f64;
    let y1 = (g0.y1 + dy1) as f64;
    let mut v = [0.0_f32; 400];
    let mut mean = 0.0_f32;
    for j in 0..PREFIX_H {
        for i in 0..PREFIX_W {
            let sx = x0 + ((x1 - x0) * ((i as f64) + 0.5)) / (PREFIX_W as f64);
            let sy = y0 + ((y1 - y0) * ((j as f64) + 0.5)) / (PREFIX_H as f64);
            let val = img.sample_sharp(sx, sy);
            v[j * PREFIX_W + i] = val;
            mean += val;
        }
    }
    mean /= 400.0;
    normalize_zero_mean_in_place(&mut v, mean);
    v
}

pub fn prefix_like(img: &ImageView<'_>, piece: &Piece) -> f32 {
    let pv0 = prefix_vector(img, piece, 5.46, 0, 0);
    let mut best = -1.0_f32;
    for r in &PREFIX_GLYPHS {
        let dot = dot_f32(r, &pv0);
        if dot > best {
            best = dot;
        }
    }
    if best < 0.22 || best >= 0.56 {
        return best;
    }
    const CENTER_PREFIX: [usize; 2] = [13, 40];
    for end in [5.3, 5.46, 5.62] {
        for dy0 in [-1, 0, 1] {
            for dy1 in [-1, 0, 1] {
                if end == 5.46 && dy0 == 0 && dy1 == 0 {
                    continue;
                }
                let pv = prefix_vector(img, piece, end, dy0, dy1);
                let mut shift_best = -1.0_f32;
                for &idx in &CENTER_PREFIX {
                    let dot = dot_f32(&PREFIX_GLYPHS[idx], &pv);
                    if dot > shift_best {
                        shift_best = dot;
                    }
                }
                if shift_best > best {
                    best = shift_best;
                }
                if shift_best >= 0.44 {
                    for r in &PREFIX_GLYPHS {
                        let dot = dot_f32(r, &pv);
                        if dot > best {
                            best = dot;
                        }
                    }
                }
            }
        }
    }
    best
}

#[inline]
pub fn text_radii(img: &ImageView<'_>) -> [usize; 4] {
    [0.006, 0.01, 0.016, 0.026].map(|f| ((img.height as f64 * f).round() as usize).max(3))
}

#[inline]
pub fn text_radii_for_scale(s: f64) -> [usize; 4] {
    let vh = 720.0 * s.max(0.5);
    [0.006, 0.01, 0.016, 0.026].map(|f| ((vh * f).round() as usize).max(3))
}

pub fn split_glyph(img: &ImageView<'_>, g: &Blob) -> Option<[Blob; 2]> {
    let mut cut = -1i32;
    let mut least = f32::INFINITY;
    let start_x = ((g.x0 as f64) + (g.w as f64) * 0.3).round() as i32;
    let end_x = ((g.x1 as f64) - (g.w as f64) * 0.3).floor() as i32;
    for x in start_x..=end_x {
        let mut ink = 0.0_f32;
        for y in g.y0..=g.y1 {
            ink += img.sharp_px(x as isize, y as isize);
        }
        if ink < least {
            least = ink;
            cut = x;
        }
    }
    if cut < 0 {
        return None;
    }
    let mut left = *g;
    left.x1 = cut - 1;
    left.w = cut - g.x0;
    let mut right = *g;
    right.x0 = cut + 1;
    right.w = g.x1 - cut;
    Some([left, right])
}

#[derive(Clone, Debug)]
pub struct DoorsRead {
    pub cells: FixedVec<u8, 4>,
    pub conf: f32,
    pub candidates: Vec<Vec<(u8, f32)>>,
    pub office_card: bool,
}

fn solve_door_assignment(
    candidates: &[Vec<(u8, f32)>],
    office_card: bool,
    warden_cell: Option<u8>,
) -> Option<(FixedVec<u8, 4>, f32)> {
    let k = candidates.len();
    if k == 0 {
        if office_card {
            let mut cells = FixedVec::new();
            cells.push(0);
            return Some((cells, 0.85));
        }
        return None;
    }

    let mut best_cells: FixedVec<u8, 4> = FixedVec::new();
    let mut best_adj_score = f32::NEG_INFINITY;
    let mut best_raw_score = 0.0_f32;
    let mut cur = [0u8; 3];

    fn search(
        idx: usize,
        k: usize,
        candidates: &[Vec<(u8, f32)>],
        warden_cell: Option<u8>,
        cur: &mut [u8; 3],
        adj_sum: f32,
        raw_sum: f32,
        best_adj: &mut f32,
        best_raw: &mut f32,
        best_cells: &mut FixedVec<u8, 4>,
    ) {
        if idx == k {
            if adj_sum > *best_adj {
                *best_adj = adj_sum;
                *best_raw = raw_sum;
                best_cells.clear();
                best_cells.extend_from_slice(&cur[..k]);
            }
            return;
        }
        for &(cell_idx, sc) in candidates[idx].iter().take(4) {
            if cur[..idx].contains(&cell_idx) {
                continue;
            }
            let penalty = if warden_cell == Some(cell_idx) {
                0.16
            } else {
                0.0
            };
            cur[idx] = cell_idx;
            search(
                idx + 1,
                k,
                candidates,
                warden_cell,
                cur,
                adj_sum + sc - penalty,
                raw_sum + sc,
                best_adj,
                best_raw,
                best_cells,
            );
        }
    }

    search(
        0,
        k.min(3),
        candidates,
        warden_cell,
        &mut cur,
        0.0,
        0.0,
        &mut best_adj_score,
        &mut best_raw_score,
        &mut best_cells,
    );

    if best_cells.is_empty() {
        for list in candidates {
            if let Some(&(c, sc)) = list.first() {
                if !best_cells.contains(&c) {
                    best_cells.push(c);
                }
                best_raw_score += sc;
            }
        }
    }

    if office_card && !best_cells.contains(&0) {
        best_cells.push(0);
    }
    let avg_score = best_raw_score / (k.max(1) as f32);
    let conf = ((avg_score - 0.30) * 1.6).clamp(0.1, 1.0);
    Some((best_cells, conf))
}

pub fn refine_doors_with_warden(doors: &mut Option<DoorsRead>, warden_cell: Option<u8>) {
    let Some(d) = doors else {
        return;
    };
    if d.candidates.is_empty() {
        return;
    }
    if let Some((cells, conf)) =
        solve_door_assignment(&d.candidates, d.office_card, warden_cell)
    {
        d.cells = cells;
        d.conf = conf;
    }
}

fn read_doors_from_lines(img: &ImageView<'_>, lines_by_r: &[Vec<Line>]) -> Option<DoorsRead> {
    let mut best_group: Vec<(Piece, i32, f64, Vec<Blob>, f32)> = Vec::new();
    let mut best_fit = f32::INFINITY;
    let mut all_pieces: Vec<Piece> = Vec::new();

    for lines in lines_by_r {
        let mut cards = Vec::new();
        for line in lines {
            for p in &line.pieces {
                all_pieces.push(p.clone());
                let cap = p.glyphs[0].h as f64;
                if cap < 6.0 || p.glyphs.len() < 4 {
                    continue;
                }
                let span = ((p.x1 - p.x0) as f64) / cap;
                if !(5.9..=7.8).contains(&span) {
                    continue;
                }
                let mut start = (p.x0 as f64) + PREFIX_NUM * cap;
                let mut gap_best = 0i32;
                for i in 0..(p.glyphs.len() - 1) {
                    let end = ((p.glyphs[i].x1 - p.x0) as f64) / cap;
                    let gap = p.glyphs[i + 1].x0 - p.glyphs[i].x1;
                    if (4.9..=6.1).contains(&end) && gap > gap_best {
                        gap_best = gap;
                        start = p.glyphs[i + 1].x0 as f64;
                    }
                }
                let num: Vec<Blob> = p
                    .glyphs
                    .iter()
                    .copied()
                    .filter(|g| (g.x0 as f64) >= start && (g.h as f64) >= 0.45 * cap)
                    .collect();
                if num.is_empty() || num.len() > 2 {
                    continue;
                }
                let like = prefix_like(img, p);
                if like < 0.50 {
                    continue;
                }
                let mid = (p.y0 + p.y1) as f64 * 0.5;
                cards.push((p.clone(), cap as i32, mid, num, 1.0 - like));
            }
        }
        for c in &cards {
            let group: Vec<_> = cards
                .iter()
                .filter(|d| {
                    (d.2 - c.2).abs() < 2.5 * (c.1 as f64)
                        && ((d.1 - c.1).abs() as f64) < 0.3 * (c.1 as f64)
                })
                .cloned()
                .collect();
            if group.len() > 3 {
                continue;
            }
            let fit: f32 = group.iter().map(|d| d.4).sum();
            if group.len() > best_group.len()
                || (group.len() == best_group.len() && fit < best_fit)
            {
                best_group = group;
                best_fit = fit;
            }
        }
    }

    let office_card = if !best_group.is_empty() {
        let h = best_group[0].1 as f64;
        all_pieces.iter().any(|p| {
            let cap = p.glyphs[0].h as f64;
            let mid = (p.y0 + p.y1) as f64 * 0.5;
            if (cap - h).abs() > 0.35 * h
                || !best_group.iter().any(|c| (c.2 - mid).abs() < 2.2 * h)
            {
                return false;
            }
            if best_group
                .iter()
                .any(|c| c.0.x0 < p.x1 && p.x0 < c.0.x1)
            {
                return false;
            }
            let span = ((p.x1 - p.x0) as f64) / cap;
            let glyph_fill: f64 = p
                .glyphs
                .iter()
                .map(|g| g.w.max(1) as f64)
                .sum::<f64>()
                / cap;
            (9.5..=20.0).contains(&span) && glyph_fill >= 5.5 && p.glyphs.len() >= 10
        })
    } else {
        false
    };

    if best_group.len() < 2 && !(best_group.len() == 1 && office_card) {
        return None;
    }
    best_group.sort_by_key(|c| c.0.x0);

    let mut candidates: Vec<Vec<(u8, f32)>> = Vec::with_capacity(best_group.len());
    for c in &best_group {
        let mut whole = read_number(img, &c.3, c.1, &ALLOWED_CELLS);
        if c.3.len() == 1 && (c.3[0].w as f64) > 0.72 * (c.1 as f64) {
            if let Some(parts) = split_glyph(img, &c.3[0]) {
                whole.extend(read_number(img, &parts, c.1, &ALLOWED_CELLS));
                whole.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            }
        }
        if whole.is_empty() {
            return None;
        }
        let mut card_cands: Vec<(u8, f32)> = Vec::with_capacity(whole.len());
        for (val, sc) in whole {
            let idx = (val - 1) as u8;
            if !card_cands.iter().any(|&(existing, _)| existing == idx) {
                card_cands.push((idx, sc));
            }
        }
        candidates.push(card_cands);
    }

    let (cells, conf) = solve_door_assignment(&candidates, office_card, None)?;
    Some(DoorsRead {
        cells,
        conf,
        candidates,
        office_card,
    })
}

fn extract_days_from_lines(img: &ImageView<'_>, lines: &[Line]) -> Option<(u8, f32)> {
    let tallest = lines
        .iter()
        .flat_map(|l| {
            l.glyphs
                .iter()
                .filter(|g| (g.w as f64) >= 0.25 * (g.h as f64))
                .map(|g| g.h)
        })
        .max()
        .unwrap_or(1)
        .max(1) as f64;

    let pair = |a: &Blob, b: &Blob| -> bool {
        ((b.h - a.h).abs() as f64) < 0.2 * (a.h as f64)
            && ((b.x0 - a.x1) as f64) < 0.7 * (a.h as f64)
            && (b.w as f64) < (b.h as f64) * 1.1
            && (a.w as f64) < (a.h as f64) * 1.1
    };

    let mut best: Option<(f64, Vec<Blob>, i32)> = None;
    for line in lines {
        let glyphs: Vec<Blob> = line
            .glyphs
            .iter()
            .copied()
            .filter(|g| {
                (g.w as f64) >= 0.25 * (g.h as f64)
                    && (g.h as f64) >= (img.height as f64) * 0.016
            })
            .collect();
        for i in 0..glyphs.len() {
            let a = glyphs[i];
            if a.h < 8
                || (a.h as f64) > (img.height as f64) * 0.09
                || (a.w as f64) > (a.h as f64) * 1.1
                || (i > 0 && pair(&glyphs[i - 1], &a))
            {
                continue;
            }
            let mut word = vec![a];
            if i + 1 < glyphs.len() && pair(&a, &glyphs[i + 1]) {
                word.push(glyphs[i + 1]);
            }
            let end = word.last().unwrap().x1;
            let after: Vec<&Blob> = glyphs
                .iter()
                .filter(|g| g.x0 > end && (g.x0 - end) < 6 * a.h)
                .collect();
            let smaller = after
                .iter()
                .filter(|g| (g.h as f64) < 0.85 * (a.h as f64))
                .count();
            if !after.is_empty() && smaller * 2 < after.len() {
                continue;
            }
            let cx = (a.x0 + end) as f64 * 0.5;
            let half_w = (img.width as f64) * 0.5;
            let sc = (smaller.min(8) as f64)
                + 6.0 * (1.0 - (cx - half_w).abs() / half_w)
                + 6.0 * ((a.h as f64) / tallest);
            if best.as_ref().map_or(true, |b| sc > b.0) {
                best = Some((sc, word, a.h));
            }
        }
    }

    let (_, word, h_val) = best?;
    let res = read_number(img, &word, h_val, &ALLOWED_DAYS);
    let &(days, score) = res.first()?;
    Some((days as u8, ((score - 0.30) * 1.6).clamp(0.1, 1.0)))
}

pub fn read_days_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    slot: &mut ScanScratchSlot,
) -> Option<(u8, f32)> {
    if loc.by_labels {
        return None;
    }
    let top = loc.at(0.0, 0.0).1.min(loc.at(5.0, 0.0).1) - 95.0 * loc.s;
    if top < 8.0 {
        return None;
    }
    let radii = text_radii_for_scale(loc.s);
    let max_h = (72.0 * loc.s).max(24.0);
    let y1 = top.min(115.0 * loc.s);
    let fast_lines: Vec<Line> = text_lines_radii_with_slot(
        img,
        (img.width as f64) * 0.35,
        0.0,
        (img.width as f64) * 0.65,
        y1,
        8.0,
        max_h,
        &radii[1..4],
        1.01,
        slot,
    )
    .into_iter()
    .flatten()
    .collect();
    let fast_res = extract_days_from_lines(img, &fast_lines);
    if let Some((days, conf)) = fast_res {
        if conf >= 0.45 {
            return Some((days, conf));
        }
    }
    if fast_lines.is_empty() {
        return None;
    }

    let rem_radii = [radii[0]];
    let lines: Vec<Line> = text_lines_radii_with_slot(
        img,
        (img.width as f64) * 0.32,
        0.0,
        (img.width as f64) * 0.68,
        top,
        8.0,
        max_h,
        &rem_radii,
        1.01,
        slot,
    )
    .into_iter()
    .flatten()
    .collect();
    match (fast_res, extract_days_from_lines(img, &lines)) {
        (Some(a), Some(b)) => Some(if b.1 > a.1 { b } else { a }),
        (Some(a), None) => Some(a),
        (None, b) => b,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LabelBox {
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
    pub h: i32,
}

pub fn find_label_with_slot(
    img: &ImageView<'_>,
    bx: [f64; 4],
    letters: isize,
    shape: f64,
    slot: &mut ScanScratchSlot,
) -> Option<LabelBox> {
    let mut best: Option<(f64, LabelBox)> = None;
    let radii = text_radii(img);
    let max_h = 40.0_f64.max((img.height as f64) * 0.06);
    let ordered_radii = [radii[1], radii[2], radii[0]];
    for lines in text_lines_radii_with_slot(
        img,
        bx[0],
        bx[1],
        bx[2],
        bx[3],
        6.0,
        max_h,
        &ordered_radii,
        0.50,
        slot,
    ) {
        for line in lines {
            for p in &line.pieces {
                let n: isize = p.words.iter().map(|w| w.glyphs.len() as isize).sum();
                let ratio = ((p.x1 - p.x0) as f64) / (line.h.max(1) as f64);
                if (n - letters).abs() > 6 || (ratio - shape).abs() > 2.5 {
                    continue;
                }
                let off = ((n - letters).abs() as f64) / 3.0 + (ratio - shape).abs();
                if best.as_ref().map_or(true, |b| off < b.0) {
                    best = Some((
                        off,
                        LabelBox {
                            x0: p.x0,
                            x1: p.x1,
                            y0: p.y0,
                            y1: p.y1,
                            h: line.h,
                        },
                    ));
                }
            }
        }
        if best.as_ref().is_some_and(|(off, _)| *off <= 1.5) {
            break;
        }
    }
    best.map(|b| b.1)
}

pub fn art_feature(img: &ImageView<'_>, x0: i32, y0: i32, x1: i32, y1: i32) -> ([f32; 72], [f32; 13]) {
    let gx = 4usize;
    let gy = 6usize;
    let mut grid = [0.0_f32; 72];
    let mut hist = [0.0_f32; 13];
    let cx0 = x0.clamp(0, img.width as i32 - 1);
    let cy0 = y0.clamp(0, img.height as i32 - 1);
    let cx1 = x1.clamp(cx0, img.width as i32 - 1);
    let cy1 = y1.clamp(cy0, img.height as i32 - 1);
    let w = (cx1 - cx0 + 1).max(1) as usize;
    let h = (cy1 - cy0 + 1).max(1) as usize;

    for j in 0..gy {
        let py0 = (cy0 as usize) + (j * h) / gy;
        let py1 = (cy0 as usize) + ((j + 1) * h) / gy;
        for i in 0..gx {
            let px0 = (cx0 as usize) + (i * w) / gx;
            let px1 = (cx0 as usize) + ((i + 1) * w) / gx;
            let mut r = 0.0_f32;
            let mut g = 0.0_f32;
            let mut b = 0.0_f32;
            let mut n = 0usize;
            for y in py0..py1 {
                let row_base = y * img.width * 4;
                for x in px0..px1 {
                    let o = row_base + x * 4;
                    let rr = img.data[o];
                    let gg = img.data[o + 1];
                    let bb = img.data[o + 2];
                    let rf = rr as f32;
                    let gf = gg as f32;
                    let bf = bb as f32;
                    r += rf;
                    g += gf;
                    b += bf;
                    n += 1;
                    let mx = rr.max(gg).max(bb);
                    let mn = rr.min(gg).min(bb);
                    if mx > 63 && ((mx - mn) as u32) * 4 > (mx as u32) {
                        let (hh, ss, vv) = hsv(rf, gf, bf);
                        hist[((hh / 30.0).floor() as usize) % 12] += ss * vv;
                    } else {
                        hist[12] += (mx as f32) * (0.3 / 255.0);
                    }
                }
            }
            let o = (j * gx + i) * 3;
            if n > 0 {
                let nf = n as f32;
                grid[o] = r / nf;
                grid[o + 1] = g / nf;
                grid[o + 2] = b / nf;
            }
        }
    }
    (grid, hist)
}

#[inline]
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    cosine_simd_impl(a, b)
}

#[inline]
pub fn art_similarity(a_grid: &[f32; 72], a_hist: &[f32; 13], b_grid: &[f32; 72], b_hist: &[f32; 13]) -> f32 {
    let am = a_grid.iter().sum::<f32>() / 72.0;
    let bm = b_grid.iter().sum::<f32>() / 72.0;
    let mut ac = [0.0_f32; 72];
    let mut bc = [0.0_f32; 72];
    for i in 0..72 {
        ac[i] = a_grid[i] - am;
        bc[i] = b_grid[i] - bm;
    }
    0.5 * cosine(a_hist, b_hist) + 0.5 * (0.5 + 0.5 * cosine(&ac, &bc))
}

#[derive(Clone, Copy, Debug)]
pub struct ScannedTool {
    pub id: &'static str,
    pub sim: ToolId,
    pub name: &'static str,
    pub score: f32,
    pub conf: f32,
}

impl Default for ScannedTool {
    fn default() -> Self {
        Self {
            id: "",
            sim: ToolId::Revenge,
            name: "",
            score: 0.0,
            conf: 0.0,
        }
    }
}

pub fn is_overlay_occluded(img: &ImageView<'_>, x0: f64, y0: f64, x1: f64, y1: f64) -> bool {
    let ix0 = (x0.round() as isize).clamp(0, img.width as isize - 1) as usize;
    let ix1 = (x1.round() as isize).clamp(ix0 as isize, img.width as isize - 1) as usize;
    let iy0 = (y0.round() as isize).clamp(0, img.height as isize - 1) as usize;
    let iy1 = (y1.round() as isize).clamp(iy0 as isize, img.height as isize - 1) as usize;
    let mut cool = 0usize;
    let mut total = 0usize;
    let mut y = iy0;
    while y <= iy1 {
        let mut x = ix0;
        while x <= ix1 {
            let (r, g, b) = img.rgb_at(x as isize, y as isize);
            total += 1;
            if b > r.saturating_add(3) && g >= r {
                cool += 1;
            }
            x += 2;
        }
        y += 2;
    }
    total > 0 && cool * 3 >= total
}

pub fn read_tools_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    _slot: &mut ScanScratchSlot,
) -> (FixedVec<ScannedTool, 6>, Option<LabelBox>) {
    let left = loc.at(0.0, 0.0).0.min(loc.at(0.0, 3.0).0) - 70.0 * loc.s;
    if left < 30.0 {
        return (FixedVec::new(), None);
    }
    let h = 11.17 * loc.s;
    let mean_v = |x0: f64, y0: f64, x1: f64, y1: f64| -> f64 {
        let mut sum = 0.0_f64;
        let mut n = 0usize;
        let iy0 = (y0.round() as isize).max(0) as usize;
        let iy1 = (y1 as isize).min(img.height as isize - 1);
        let ix0 = (x0.round() as isize).max(0) as usize;
        let ix1 = (x1 as isize).min(img.width as isize - 1);
        if iy1 < iy0 as isize || ix1 < ix0 as isize {
            return 0.0;
        }
        for y in iy0..=(iy1 as usize) {
            for x in ix0..=(ix1 as usize) {
                let (r, g, b) = img.rgb_at(x as isize, y as isize);
                sum += (r.max(g).max(b) as f64) / 255.0;
                n += 1;
            }
        }
        if n > 0 {
            sum / (n as f64)
        } else {
            0.0
        }
    };

    let card_frame_edge = |x0: f64, y0: f64, x1: f64, y1: f64| -> f64 {
        let w = (0.1 * h).round().max(2.0);
        let reach = (0.15 * h).round().max(2.0) as isize;
        let ya = y0 + 0.8 * h;
        let yb = y1 - 0.8 * h;
        let xa = x0 + 0.8 * h;
        let xb = x1 - 0.8 * h;
        let step = |f: &dyn Fn(f64) -> f64| -> f64 {
            let mut best = 0.0_f64;
            for o in -reach..=reach {
                best = best.max(f(o as f64).abs());
            }
            best
        };
        let mut edges = [
            step(&|o| mean_v(x0 + o - w, ya, x0 + o - 1.0, yb) - mean_v(x0 + o + 1.0, ya, x0 + o + w, yb)),
            step(&|o| mean_v(x1 + o - w, ya, x1 + o - 1.0, yb) - mean_v(x1 + o + 1.0, ya, x1 + o + w, yb)),
            step(&|o| mean_v(xa, y0 + o - w, xb, y0 + o - 1.0) - mean_v(xa, y0 + o + 1.0, xb, y0 + o + w)),
            step(&|o| mean_v(xa, y1 + o - w, xb, y1 + o - 1.0) - mean_v(xa, y1 + o + 1.0, xb, y1 + o + w)),
        ];
        edges.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (edges[1] + edges[2]) * 0.5
    };

    // Direct UMG left-panel anchor with sub-millisecond Slot-0 frame micro-alignment (zero OCR)
    let base_x0 = (54.0 * loc.s + if loc.tx < -10.0 * loc.s { loc.tx } else { 0.0 }).max(0.0);
    let base_y1 = loc.ty + 210.4 * loc.s;
    let mut best_dx = 0.0_f64;
    let mut best_dy = 0.0_f64;
    let mut best_edge = -1.0_f64;
    for dx_u in [-1.5, 0.0, 1.5] {
        let dx = dx_u * loc.s;
        for dy_u in [-2.0, 0.0, 2.0] {
            let dy = dy_u * loc.s;
            let sx0 = base_x0 + dx + 0.15 * h;
            let sx1 = sx0 + 4.45 * h;
            let sy0 = base_y1 + dy + 1.55 * h;
            let sy1 = sy0 + 7.45 * h;
            if sx1 <= left {
                let edge = card_frame_edge(sx0, sy0, sx1, sy1);
                if edge > best_edge {
                    best_edge = edge;
                    best_dx = dx;
                    best_dy = dy;
                }
            }
        }
    }
    let label = LabelBox {
        x0: (base_x0 + best_dx).round() as i32,
        x1: (base_x0 + best_dx + 12.2 * h).round() as i32,
        y0: (base_y1 + best_dy - 10.6 * loc.s).round() as i32,
        y1: (base_y1 + best_dy).round() as i32,
        h: h.round().max(1.0) as i32,
    };

    let mut out = FixedVec::new();
    for i in 0..6 {
        let x0 = (label.x0 as f64) + (0.15 + 6.05 * (i as f64)) * h;
        let x1 = x0 + 4.45 * h;
        let y0 = (label.y1 as f64) + 1.55 * h;
        let y1 = y0 + 7.45 * h;
        if x1 > left {
            break;
        }
        let mut sy0 = (y0 + 0.4 * h).round().clamp(0.0, img.height as f64 - 1.0) as i32;
        let sy1 = (y1 - 0.4 * h).floor().clamp(0.0, img.height as f64 - 1.0) as i32;
        let sx0 = (x0 + 0.4 * h).round().clamp(0.0, img.width as f64 - 1.0) as i32;
        let sx1 = (x1 - 0.4 * h).floor().clamp(0.0, img.width as f64 - 1.0) as i32;
        while sy0 < sy1 {
            let mut cool_px = 0usize;
            let mut tot_px = 0usize;
            for x in sx0..=sx1 {
                let (r, g, b) = img.rgb_at(x as isize, sy0 as isize);
                tot_px += 1;
                if b > r.saturating_add(2) && g >= r {
                    cool_px += 1;
                }
            }
            if tot_px > 0 && cool_px * 3 >= tot_px {
                sy0 += 1;
            } else {
                break;
            }
        }
        // If we skipped overlay rows at the top, skip 1 extra border row
        if sy0 > (y0 + 0.4 * h).round() as i32 && sy0 < sy1 {
            sy0 += 1;
        }
        let inside = mean_v(
            x0 + 0.5 * h,
            (sy0 as f64).max(y0 + 0.5 * h),
            x1 - 0.5 * h,
            y1 - 0.5 * h,
        );
        let wall = mean_v(x1 + 0.35 * h, (sy0 as f64).max(y0), x1 + 1.3 * h, y1);
        let thr = 0.16_f32.max((inside * 1.9) as f32);
        let mut ax0 = i32::MAX;
        let mut ay0 = i32::MAX;
        let mut ax1 = -1i32;
        let mut ay1 = -1i32;
        let mut art = 0usize;
        for y in sy0..=sy1 {
            let row_base = (y as usize) * img.width * 4;
            for x in sx0..=sx1 {
                let o = row_base + (x as usize) * 4;
                let r = img.data[o];
                let g = img.data[o + 1];
                let b = img.data[o + 2];
                let mx = r.max(g).max(b);
                let v = (mx as f32) * (1.0 / 255.0);
                if v > thr {
                    let mn = r.min(g).min(b);
                    if ((mx - mn) as u32) * 5 > (mx as u32) || v > thr * 1.3 {
                        art += 1;
                        ax0 = ax0.min(x);
                        ax1 = ax1.max(x);
                        ay0 = ay0.min(y);
                        ay1 = ay1.max(y);
                    }
                }
            }
        }
        if (art as f64) < 1.0 * h * h && !((art as f64) > 0.5 * h * h && inside > wall + 0.012) {
            break;
        }
        let bw = (ax1 - ax0 + 1) as f64;
        let bh = (ay1 - ay0 + 1) as f64;
        if (art as f64) / (bw * bh).max(1.0) < 0.065 || bw > 2.0 * bh {
            break;
        }
        if card_frame_edge(x0, y0, x1, y1) < CARD_FRAME && inside - wall < CARD_LIFT {
            break;
        }

        let (fg0, fh0) = art_feature(img, ax0, ay0, ax1, ay1);
        let mut best_entry = &REF_TOOLS[0];
        let mut best_sc = f32::NEG_INFINITY;
        let mut second_sc = f32::NEG_INFINITY;
        for t in &REF_TOOLS {
            let sc = art_similarity(&fg0, &fh0, &t.grid, &t.hist);
            if sc > best_sc {
                if t.id != best_entry.id {
                    second_sc = best_sc;
                }
                best_sc = sc;
                best_entry = t;
            } else if t.id != best_entry.id && sc > second_sc {
                second_sc = sc;
            }
        }
        if best_sc < 0.86 || (best_sc - second_sc) < 0.04 {
            let extra_feats: [([f32; 72], [f32; 13]); 4] = [
                art_feature(img, ax0 - 1, ay0, ax1 - 1, ay1),
                art_feature(img, ax0 + 1, ay0, ax1 + 1, ay1),
                art_feature(img, ax0, ay0 - 1, ax1, ay1 - 1),
                art_feature(img, ax0, ay0 + 1, ax1, ay1 + 1),
            ];
            best_entry = &REF_TOOLS[0];
            best_sc = f32::NEG_INFINITY;
            second_sc = f32::NEG_INFINITY;
            for t in &REF_TOOLS {
                let sc0 = art_similarity(&fg0, &fh0, &t.grid, &t.hist);
                let sc = extra_feats
                    .iter()
                    .map(|(fg, fh)| art_similarity(fg, fh, &t.grid, &t.hist))
                    .fold(sc0, f32::max);
                if sc > best_sc {
                    if t.id != best_entry.id {
                        second_sc = best_sc;
                    }
                    best_sc = sc;
                    best_entry = t;
                } else if t.id != best_entry.id && sc > second_sc {
                    second_sc = sc;
                }
            }
        }
        let second = if second_sc.is_finite() { second_sc } else { 0.0 };
        let margin = best_sc - second;
        let conf = ((best_sc - 0.60) * 2.5 + margin * 5.0).clamp(0.1, 1.0);
        out.push(ScannedTool {
            id: best_entry.id,
            sim: best_entry.sim,
            name: best_entry.name,
            score: best_sc,
            conf,
        });
    }
    (out, Some(label))
}

pub fn read_chromatic_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    slot: &mut ScanScratchSlot,
) -> Option<usize> {
    let right = loc.at(5.0, 0.0).0.max(loc.at(5.0, 3.0).0) + 60.0 * loc.s;
    if right > (img.width as f64) - 30.0 {
        return None;
    }
    let u = loc.s;
    // Direct UMG right-panel anchor (integral image 2x5 grid search handles ±20*u alignment directly in O(1))
    let cx = (loc.tx + 1259.0 * u).min((img.width as f64) - 21.0 * u);
    let cy = loc.ty + 96.65 * u;
    let x0 = ((cx - 110.0 * u).floor() as isize).max(0) as usize;
    let x1 = ((cx + 40.0 * u).ceil() as isize)
        .min(img.width as isize - 1)
        .max(0) as usize;
    let y0 = ((cy + 10.0 * u).floor() as isize).max(0) as usize;
    let y1 = ((cy + 410.0 * u).ceil() as isize)
        .min(img.height as isize - 1)
        .max(0) as usize;
    if x1 < x0 || y1 < y0 {
        return None;
    }
    let sw = x1 - x0 + 2;
    let sh = y1 - y0 + 2;
    if slot.sat.len() < sw * sh {
        slot.sat.resize(sw * sh, 0);
    }
    let sum = &mut slot.sat[..sw * sh];
    sum[..sw].fill(0);
    for y in y0..=y1 {
        let row_off = (y - y0 + 1) * sw;
        let prev_off = (y - y0) * sw;
        sum[row_off] = 0;
        let mut row = 0u32;
        for x in x0..=x1 {
            let (r, g, b) = img.rgb_at(x as isize, y as isize);
            if r.max(g).max(b) > 51 {
                row += 1;
            }
            sum[row_off + (x - x0 + 1)] = sum[prev_off + (x - x0 + 1)] + row;
        }
    }
    let light_share = |x: f64, y: f64| -> f64 {
        let a = ((x - 18.0 * u).round() as isize).max(x0 as isize) as usize;
        let b = ((x + 18.0 * u).round() as isize)
            .min(x1 as isize)
            .max(0) as usize;
        let c = ((y - 18.0 * u).round() as isize).max(y0 as isize) as usize;
        let d = ((y + 18.0 * u).round() as isize)
            .min(y1 as isize)
            .max(0) as usize;
        if b < a || d < c {
            return 0.0;
        }
        let at2 = |px: usize, py: usize| sum[(py - y0) * sw + (px - x0)] as f64;
        (at2(b + 1, d + 1) - at2(a, d + 1) - at2(b + 1, c) + at2(a, c))
            / (((b - a + 1) * (d - c + 1)) as f64)
    };
    let grid_total = |ox: f64, oy: f64| -> f64 {
        let mut total = 0.0_f64;
        for r in 0..5 {
            for dx in [-66.0, -6.0] {
                let x = cx + (dx + ox) * u;
                let y = cy + (54.0 + 68.5 * (r as f64) + oy) * u;
                total += light_share(x, y);
            }
        }
        total
    };
    let mut best_ox = 0.0_f64;
    let mut best_oy = 0.0_f64;
    let mut best_total = grid_total(0.0, 0.0);
    for oy_i in (-20..=20).step_by(2) {
        let oy = oy_i as f64;
        for ox_i in (-20..=20).step_by(2) {
            let ox = ox_i as f64;
            let total = grid_total(ox, oy);
            if total > best_total {
                best_total = total;
                best_ox = ox;
                best_oy = oy;
            }
        }
    }
    let mut count = 0usize;
    'outer: for r in 0..5 {
        for dx in [-66.0, -6.0] {
            let x = cx + (dx + best_ox) * u;
            let y = cy + (54.0 + 68.5 * (r as f64) + best_oy) * u;
            if light_share(x, y) > 0.35 {
                count += 1;
            } else {
                break 'outer;
            }
        }
    }
    Some(count)
}

#[derive(Clone, Copy, Debug)]
pub struct PickCardRead {
    pub sim: ToolId,
    pub score: f32,
    pub conf: f32,
}

#[derive(Clone, Debug)]
pub struct PickScreenRead {
    pub cards: [PickCardRead; 3],
    pub reroll: bool,
}

/// Fast (< 0.03ms) structural verification that the 3-card Instrument Pick modal (`S16BoardRelicPop`)
/// is actually rendered on screen before running histograms or OCR.
/// Checks that the 3 card lower-body panels (`k = -1, 0, 1` at `y = 560..760`) and the two
/// inter-card dimmed modal gutters (`x = w/2 ± 240s`) have the dark slate backdrop of the popup.
#[inline]
fn pick_modal_backdrop_ok(img: &ImageView<'_>) -> bool {
    let s = (img.height as f64) / 1080.0;
    let step = (4.0 * s).round().max(2.0) as usize;

    // 1. Check the 3 dark card description bodies
    for k in [-1i32, 0, 1] {
        let c = (img.width as f64) * 0.5 + (k as f64) * 480.0 * s;
        let x0 = ((c - 150.0 * s).round() as isize).max(0) as usize;
        let x1 = ((c + 150.0 * s).round() as isize).min(img.width as isize - 1) as usize;
        let y0 = ((560.0 * s).round() as isize).max(0) as usize;
        let y1 = ((760.0 * s).round() as isize).min(img.height as isize - 1) as usize;
        if x1 <= x0 || y1 <= y0 {
            return false;
        }
        let mut total = 0usize;
        let mut dark_slate = 0usize;
        let mut sum_v = 0u32;
        for y in (y0..=y1).step_by(step) {
            let row = y * img.width * 4;
            for x in (x0..=x1).step_by(step) {
                let o = row + x * 4;
                let r = img.data[o];
                let g = img.data[o + 1];
                let b = img.data[o + 2];
                let mx = r.max(g).max(b);
                let mn = r.min(g).min(b);
                total += 1;
                sum_v += mx as u32;
                // Dark slate card background: low-to-moderate brightness and low saturation
                if mx <= 105 && (mx - mn) <= 48 {
                    dark_slate += 1;
                }
            }
        }
        if total == 0 || dark_slate * 100 < total * 48 || (sum_v / (total as u32)) > 96 {
            return false;
        }
    }

    // 2. Check the two dimmed modal gutters between cards (-240s and +240s)
    for sign in [-1.0_f64, 1.0_f64] {
        let gx = (img.width as f64) * 0.5 + sign * 240.0 * s;
        let x0 = ((gx - 18.0 * s).round() as isize).max(0) as usize;
        let x1 = ((gx + 18.0 * s).round() as isize).min(img.width as isize - 1) as usize;
        let y0 = ((280.0 * s).round() as isize).max(0) as usize;
        let y1 = ((740.0 * s).round() as isize).min(img.height as isize - 1) as usize;
        if x1 <= x0 || y1 <= y0 {
            return false;
        }
        let mut total = 0usize;
        let mut sum_v = 0u32;
        for y in (y0..=y1).step_by(step) {
            let row = y * img.width * 4;
            for x in (x0..=x1).step_by(step) {
                let o = row + x * 4;
                let mx = img.data[o].max(img.data[o + 1]).max(img.data[o + 2]);
                total += 1;
                sum_v += mx as u32;
            }
        }
        if total == 0 || (sum_v / (total as u32)) > 110 {
            return false;
        }
    }

    true
}

pub fn pick_art_box(img: &ImageView<'_>, k: i32) -> Option<[i32; 4]> {
    let s = (img.height as f64) / 1080.0;
    let c = (img.width as f64) * 0.5 + (k as f64) * 480.0 * s;
    let x0 = ((c - 85.0 * s).round() as i32).max(0);
    let x1 = ((c + 85.0 * s).round() as i32).min(img.width as i32 - 1);
    let y0 = ((270.0 * s).round() as i32).max(0);
    let y1 = ((480.0 * s).round() as i32).min(img.height as i32 - 1);
    if x1 < x0 || y1 < y0 {
        return None;
    }
    let w = (x1 - x0 + 1) as usize;
    let h = (y1 - y0 + 1) as usize;
    if w > 512 || h > 512 {
        return None;
    }
    let mut x_hist = [0u32; 512];
    let mut y_hist = [0u32; 512];
    let mut count = 0usize;
    let ux0 = x0 as usize;
    let uy0 = y0 as usize;
    for dy in 0..h {
        let row_base = (uy0 + dy) * img.width * 4;
        let mut row_hits = 0u32;
        for dx in 0..w {
            let o = row_base + (ux0 + dx) * 4;
            let r = img.data[o];
            let g = img.data[o + 1];
            let b = img.data[o + 2];
            let mx = r.max(g).max(b);
            let matched = if mx >= 82 {
                true
            } else if mx <= 45 {
                false
            } else {
                let mn = r.min(g).min(b);
                if ((mx - mn) as u32) * 20 <= 9 * (mx as u32) {
                    false
                } else {
                    let (hue, sat, v) = hsv(r as f32, g as f32, b as f32);
                    v > 0.32 || (sat > 0.45 && v > 0.18 && !(15.0..=50.0).contains(&hue))
                }
            };
            if matched {
                x_hist[dx] += 1;
                row_hits += 1;
            }
        }
        y_hist[dy] = row_hits;
        count += row_hits as usize;
    }
    let total_area = w * h;
    // Reject if too few pixels matched OR if the entire search box is uniformly bright (>92% fill)
    if (count as f64) < 400.0 * s * s || count * 100 > total_area * 92 {
        return None;
    }
    let rank_lo = (0.01 * ((count - 1) as f64)).round() as u32;
    let rank_hi = (0.99 * ((count - 1) as f64)).round() as u32;
    let percentile_from_hist = |hist: &[u32], base: i32, rank: u32| -> i32 {
        let mut acc = 0u32;
        for (idx, &c) in hist.iter().enumerate() {
            acc += c;
            if acc > rank {
                return base + (idx as i32);
            }
        }
        base + (hist.len().saturating_sub(1) as i32)
    };
    Some([
        percentile_from_hist(&x_hist[..w], x0, rank_lo),
        percentile_from_hist(&y_hist[..h], y0, rank_lo),
        percentile_from_hist(&x_hist[..w], x0, rank_hi),
        percentile_from_hist(&y_hist[..h], y0, rank_hi),
    ])
}

#[inline]
fn count_neutral_light_thresh(
    img: &ImageView<'_>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    min_mx: u8,
) -> usize {
    let ux0 = x0.max(0) as usize;
    let uy0 = y0.max(0) as usize;
    let ux1 = x1.min(img.width as i32 - 1);
    let uy1 = y1.min(img.height as i32 - 1);
    if ux1 < ux0 as i32 || uy1 < uy0 as i32 {
        return 0;
    }
    let ux1 = ux1 as usize;
    let uy1 = uy1 as usize;
    let mut light = 0usize;
    for y in (uy0..=uy1).step_by(2) {
        let row_base = y * img.width * 4;
        for x in (ux0..=ux1).step_by(2) {
            let o = row_base + x * 4;
            let r = img.data[o];
            let g = img.data[o + 1];
            let b = img.data[o + 2];
            let mx = r.max(g).max(b);
            if mx > min_mx {
                let mn = r.min(g).min(b);
                if ((mx - mn) as u32) * 4 < (mx as u32) {
                    light += 1;
                }
            }
        }
    }
    light
}

#[inline]
fn count_neutral_light(img: &ImageView<'_>, x0: i32, y0: i32, x1: i32, y1: i32) -> usize {
    count_neutral_light_thresh(img, x0, y0, x1, y1, 140)
}

pub fn pick_reroll_shown(img: &ImageView<'_>) -> bool {
    let s = (img.height as f64) / 1080.0;
    let x0 = ((img.width as f64) * 0.5 - 330.0 * s).round() as i32;
    let x1 = ((img.width as f64) * 0.5 - 175.0 * s).round() as i32;
    let y0 = (940.0 * s).round() as i32;
    let y1 = (985.0 * s).round() as i32;
    (count_neutral_light(img, x0, y0, x1, y1) as f64) > 15.0 * s * s
}

pub fn pick_confirm_shown(img: &ImageView<'_>) -> bool {
    let s = (img.height as f64) / 1080.0;
    let x0 = ((img.width as f64) * 0.5 - 120.0 * s).round() as i32;
    let x1 = ((img.width as f64) * 0.5 + 330.0 * s).round() as i32;
    let y0 = (940.0 * s).round() as i32;
    let y1 = (985.0 * s).round() as i32;
    (count_neutral_light_thresh(img, x0, y0, x1, y1, 115) as f64) > 15.0 * s * s
}

pub fn scan_pick_with_slot(
    img: &ImageView<'_>,
    slot: &mut ScanScratchSlot,
) -> Option<PickScreenRead> {
    if !pick_confirm_shown(img) || !pick_modal_backdrop_ok(img) {
        return None;
    }
    let s = (img.height as f64) / 1080.0;
    for k in [-1, 0, 1] {
        let c = (img.width as f64) * 0.5 + (k as f64) * 480.0 * s;
        let tx0 = (c - 180.0 * s).round() as i32;
        let tx1 = (c + 180.0 * s).round() as i32;
        let ty0 = (420.0 * s).round() as i32;
        let ty1 = (550.0 * s).round() as i32;
        if (count_neutral_light(img, tx0, ty0, tx1, ty1) as f64) < 10.0 * s * s {
            return None;
        }
    }

    let art_boxes = [
        pick_art_box(img, -1)?,
        pick_art_box(img, 0)?,
        pick_art_box(img, 1)?,
    ];
    for bx in &art_boxes {
        if ((bx[2] - bx[0]) as f64) < 60.0 * s || ((bx[3] - bx[1]) as f64) < 60.0 * s {
            return None;
        }
    }

    let radii = text_radii(img);
    let title_radius = [radii[1]];
    let min_title_h = (15.0 * s).round().max(8.0) as i32;
    let mut cards = [PickCardRead {
        sim: ToolId::Revenge,
        score: 0.0,
        conf: 0.0,
    }; 3];
    for (idx, k) in [-1, 0, 1].into_iter().enumerate() {
        let c = (img.width as f64) * 0.5 + (k as f64) * 480.0 * s;
        let title_lines = text_lines_radii_with_slot(
            img,
            c - 185.0 * s,
            420.0 * s,
            c + 185.0 * s,
            550.0 * s,
            (8.0 * s).max(5.0),
            (45.0 * s).max(16.0),
            &title_radius,
            0.35,
            slot,
        );
        let mut observed_title_glyphs = 0usize;
        if let Some(lines) = title_lines.first() {
            for line in lines {
                let ly = (line.y0 + line.y1) as f64 * 0.5;
                if ly < 470.0 * s || ly > 548.0 * s || line.h < min_title_h {
                    continue;
                }
                for piece in &line.pieces {
                    let px = (piece.x0 + piece.x1) as f64 * 0.5;
                    if (px - c).abs() <= 65.0 * s && piece.glyphs.len() >= 3 {
                        observed_title_glyphs += piece.glyphs.len();
                    }
                }
            }
            if observed_title_glyphs < 4 {
                for line in lines {
                    if line.glyphs.len() >= 4 {
                        observed_title_glyphs = observed_title_glyphs.max(line.glyphs.len());
                    }
                }
            }
        }
        if observed_title_glyphs < 4 {
            return None;
        }
        let bx = art_boxes[idx];
        let feats: [([f32; 72], [f32; 13]); 5] = [
            art_feature(img, bx[0], bx[1], bx[2], bx[3]),
            art_feature(img, bx[0] - 2, bx[1], bx[2] - 2, bx[3]),
            art_feature(img, bx[0] + 2, bx[1], bx[2] + 2, bx[3]),
            art_feature(img, bx[0], bx[1] - 2, bx[2], bx[3] - 2),
            art_feature(img, bx[0], bx[1] + 2, bx[2], bx[3] + 2),
        ];
        let mut best_by_sim = [f32::NEG_INFINITY; 31];
        for r in &REF_PICK_TOOLS {
            let sc = feats
                .iter()
                .map(|(fg, fh)| art_similarity(fg, fh, &r.grid, &r.hist))
                .fold(f32::NEG_INFINITY, f32::max);
            let slot_sc = &mut best_by_sim[r.sim.idx()];
            if sc > *slot_sc {
                *slot_sc = sc;
            }
        }
        for r in &REF_TOOLS {
            if r.is_panel {
                continue;
            }
            let sc = feats
                .iter()
                .map(|(fg, fh)| art_similarity(fg, fh, &r.grid, &r.hist))
                .fold(f32::NEG_INFINITY, f32::max);
            let slot_sc = &mut best_by_sim[r.sim.idx()];
            if sc > *slot_sc {
                *slot_sc = sc;
            }
        }
        let mut top_sim = ToolId::Revenge;
        let mut top_raw_sc = f32::NEG_INFINITY;
        let mut top_sc = f32::NEG_INFINITY;
        let mut second_sc = f32::NEG_INFINITY;
        for (&sim_id, &raw_sc) in tartarus_core::ALL_TOOLS.iter().zip(best_by_sim.iter()) {
            if !raw_sc.is_finite() {
                continue;
            }
            let expected_chars = sim_id.name_alnum_len();
            let diff = observed_title_glyphs.abs_diff(expected_chars);
            let title_adj = if diff == 0 {
                0.035
            } else if diff <= 1 {
                0.025
            } else if diff <= 2 {
                0.012
            } else if diff <= 4 {
                0.0
            } else {
                -((diff as f32 - 4.0) * 0.006).min(0.06)
            };
            let sc = raw_sc + title_adj;
            if sc > top_sc {
                second_sc = top_sc;
                top_sc = sc;
                top_raw_sc = raw_sc;
                top_sim = sim_id;
            } else if sc > second_sc {
                second_sc = sc;
            }
        }
        // Require a real minimum template similarity so background textures never pass via margin alone
        if !top_raw_sc.is_finite() || top_raw_sc < 0.80 {
            return None;
        }
        let second = if second_sc.is_finite() { second_sc } else { 0.0 };
        let margin = top_sc - second;
        let eff_sc = top_sc.max(top_raw_sc);
        let conf = if eff_sc < 0.82 {
            0.0
        } else {
            ((eff_sc - 0.85) * 5.0 + margin * 8.0).clamp(0.0, 1.0)
        };
        cards[idx] = PickCardRead {
            sim: top_sim,
            score: eff_sc,
            conf,
        };
    }
    // Enforce game-rule invariant: the 3 offered instruments in a pick are always distinct
    if cards[0].sim == cards[1].sim
        || cards[1].sim == cards[2].sim
        || cards[0].sim == cards[2].sim
    {
        return None;
    }
    if cards.iter().all(|c| c.conf < 0.20) {
        return None;
    }
    Some(PickScreenRead {
        cards,
        reroll: pick_reroll_shown(img),
    })
}

#[derive(Clone, Copy, Debug)]
pub enum TitleRead {
    Objective(ObjectiveId),
    StartEvent(StartEventId),
}

fn is_opulence_popup(img: &ImageView<'_>) -> bool {
    let w = img.width as usize;
    let h = img.height as usize;
    if w == 0 || h == 0 {
        return false;
    }
    let mut crimson = 0usize;
    let mut purple = 0usize;
    let ex0 = ((w as f64) * 0.46) as usize;
    let ex1 = (((w as f64) * 0.54) as usize).min(w);
    let ey0 = ((h as f64) * 0.16) as usize;
    let ey1 = (((h as f64) * 0.27) as usize).min(h);
    for y in ey0..ey1 {
        for x in ex0..ex1 {
            let idx = (y * w + x) * 4;
            let r = img.data[idx] as i32;
            let g = img.data[idx + 1] as i32;
            let b = img.data[idx + 2] as i32;
            if r > 110 && r > b + 45 && r > g + 35 {
                crimson += 1;
            } else if b > 70 && r > 50 && b > g + 15 && r > g + 10 && r <= b + 30 {
                purple += 1;
            }
        }
    }

    let mut cyan_line1 = 0usize;
    let lx0 = ((w as f64) * 0.45) as usize;
    let lx1 = (((w as f64) * 0.65) as usize).min(w);
    let ly0 = ((h as f64) * 0.365) as usize;
    let ly1 = (((h as f64) * 0.402) as usize).min(h);
    for y in ly0..ly1 {
        for x in lx0..lx1 {
            let idx = (y * w + x) * 4;
            let r = img.data[idx] as i32;
            let g = img.data[idx + 1] as i32;
            let b = img.data[idx + 2] as i32;
            if b > 130 && g > 120 && b > r + 40 && g > r + 30 {
                cyan_line1 += 1;
            }
        }
    }

    crimson > purple.saturating_mul(2).max(10) && cyan_line1 < 5
}

pub fn read_objective_title_with_slot(
    img: &ImageView<'_>,
    slot: &mut ScanScratchSlot,
) -> Option<TitleRead> {
    let single_word_start = if is_opulence_popup(img) {
        StartEventId::Opulence
    } else {
        StartEventId::Tyranny
    };
    let patterns: &[(TitleRead, &[isize])] = &[
        (TitleRead::Objective(ObjectiveId::Consume), &[8, 2, 3, 7]),
        (TitleRead::Objective(ObjectiveId::Consume), &[8, 5, 7]),
        (TitleRead::Objective(ObjectiveId::Doors), &[6, 4, 5]),
        (TitleRead::Objective(ObjectiveId::Incident), &[13]),
        (TitleRead::Objective(ObjectiveId::King), &[4, 2, 3, 5]),
        (TitleRead::Objective(ObjectiveId::King), &[4, 5, 5]),
        (TitleRead::Objective(ObjectiveId::Walk), &[6, 4]),
        (TitleRead::StartEvent(StartEventId::CageFight), &[4, 5]),
        (TitleRead::StartEvent(StartEventId::CageFight), &[4, 4]),
        (TitleRead::StartEvent(StartEventId::IronLaw), &[4, 3]),
        (TitleRead::StartEvent(single_word_start), &[7]),
        (TitleRead::StartEvent(single_word_start), &[8]),
    ];

    let mut best_match: Option<(TitleRead, i32, isize)> = None;
    let radii = text_radii(img);
    for lines in text_lines_radii_with_slot(
        img,
        (img.width as f64) * 0.25,
        (img.height as f64) * 0.12,
        (img.width as f64) * 0.75,
        (img.height as f64) * 0.40,
        6.0,
        48.0_f64.max((img.height as f64) * 0.07),
        &radii,
        0.34,
        slot,
    ) {
        for line in lines {
            for p in &line.pieces {
                let words: Vec<isize> = p.words.iter().map(|w| w.glyphs.len() as isize).collect();
                if words.is_empty() {
                    continue;
                }
                let has_obj_prefix = words[0] >= 8 && words[0] <= 11 && words.len() > 1;
                let name = if has_obj_prefix {
                    &words[1..]
                } else {
                    &words[..]
                };
                let mid_x = ((p.x0 + p.x1) as f64) * 0.5;
                let centered = (mid_x - (img.width as f64) * 0.5).abs() <= (img.width as f64) * 0.12;
                for &(id, want) in patterns {
                    if has_obj_prefix && matches!(id, TitleRead::StartEvent(_)) {
                        continue;
                    }
                    if want.len() == 1 && matches!(id, TitleRead::StartEvent(_)) && !centered {
                        continue;
                    }
                    if want.len() != name.len() {
                        continue;
                    }
                    if want
                        .iter()
                        .zip(name.iter())
                        .any(|(&a, &b)| (a - b).abs() > if a >= 8 { 2 } else { 1 })
                    {
                        continue;
                    }
                    let off: isize = want
                        .iter()
                        .zip(name.iter())
                        .map(|(a, b)| (a - b).abs())
                        .sum();
                    let total: isize = want.iter().sum();
                    if (off as f64) <= 1.0 + (total as f64) * 0.15 {
                        let better = best_match.map_or(true, |(_, bh, boff)| {
                            let same_tier = (line.h - bh).abs() <= bh.max(line.h) / 5;
                            if same_tier {
                                off < boff || (off == boff && line.h > bh)
                            } else {
                                line.h > bh
                            }
                        });
                        if better {
                            best_match = Some((id, line.h, off));
                        }
                    }
                }
            }
        }
    }
    best_match.map(|m| m.0)
}

pub fn read_hud_objective_with_slot(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    slot: &mut ScanScratchSlot,
) -> Option<ObjectiveId> {
    let patterns: &[(ObjectiveId, &[isize], bool)] = &[
        // Full single-line titles (with or without "Objective:")
        (ObjectiveId::Consume, &[8, 2, 3, 7], false),
        (ObjectiveId::Consume, &[8, 5, 7], false),
        (ObjectiveId::Doors, &[6, 4, 5], false),
        (ObjectiveId::Incident, &[13], true),
        (ObjectiveId::King, &[4, 2, 3, 5], false),
        (ObjectiveId::King, &[4, 5, 5], false),
        (ObjectiveId::Walk, &[6, 4], true),
        // Line-wrapped HUD titles ("Objective: Survival of the", "Objective: King of the")
        (ObjectiveId::Consume, &[8, 2, 3], true),
        (ObjectiveId::King, &[4, 2, 3], true),
        // Distinct HUD requirement lines below title
        (ObjectiveId::King, &[2, 5, 1, 7, 6], false),
        (ObjectiveId::King, &[7, 8], false),
        (ObjectiveId::Consume, &[7, 1, 5, 3], false),
        (ObjectiveId::Incident, &[7, 6, 8, 1, 5, 3], false),
        (ObjectiveId::Incident, &[7, 6, 8], false),
    ];
    let x1 = (loc.at(0.0, 0.0).0.min(loc.at(0.0, 3.0).0) - 40.0 * loc.s)
        .min((img.width as f64) * 0.30);
    let mut y0 = (loc.ty + 15.0 * loc.s).max(0.0);
    let y1 = (loc.ty + 185.0 * loc.s).min((img.height as f64) * 0.30);
    let probe_x0 = (20.0 * loc.s).max(0.0);
    let probe_x1 = x1.min(160.0 * loc.s);
    while y0 + 8.0 < y1 && is_overlay_occluded(img, probe_x0, y0, probe_x1, y0 + 8.0) {
        y0 += 6.0;
    }
    if x1 < 40.0 || y1 <= y0 + 14.0 {
        return None;
    }
    let radii = text_radii(img);
    let mut best_match: Option<(ObjectiveId, isize)> = None;
    for lines in text_lines_radii_with_slot(
        img,
        0.0,
        y0,
        x1,
        y1,
        5.0,
        36.0_f64.max((img.height as f64) * 0.05),
        &radii[..3],
        0.32,
        slot,
    ) {
        for line in lines {
            for p in &line.pieces {
                let words: Vec<isize> = p.words.iter().map(|w| w.glyphs.len() as isize).collect();
                if words.is_empty() {
                    continue;
                }
                let has_obj_prefix = words[0] >= 8 && words[0] <= 11 && words.len() > 1;
                let name = if has_obj_prefix {
                    &words[1..]
                } else {
                    &words[..]
                };
                for &(id, want, need_prefix) in patterns {
                    if need_prefix && !has_obj_prefix {
                        continue;
                    }
                    if want.len() != name.len() {
                        continue;
                    }
                    if want
                        .iter()
                        .zip(name.iter())
                        .any(|(&a, &b)| (a - b).abs() > 1)
                    {
                        continue;
                    }
                    let off: isize = want
                        .iter()
                        .zip(name.iter())
                        .map(|(a, b)| (a - b).abs())
                        .sum();
                    if off <= 1 && best_match.map_or(true, |(_, boff)| off < boff) {
                        best_match = Some((id, off));
                    }
                }
                // Disambiguate HUD requirement lines ("Move 18 tiles ...", "Open 10 cells ...",
                // "Consume 4 times ...", "Trigger Prison Incident ...") when title is covered
                if !has_obj_prefix && words.len() >= 3 {
                    if (3..=5).contains(&words[0])
                        && (1..=3).contains(&words[1])
                        && (4..=6).contains(&words[2])
                    {
                        let nums = read_number(img, &p.words[1].glyphs, line.h, &[10, 18]);
                        if let Some(&(n, score)) = nums.first() {
                            if score >= 0.30 {
                                let id = if n == 18 {
                                    ObjectiveId::Walk
                                } else {
                                    ObjectiveId::Doors
                                };
                                if best_match.map_or(true, |(_, boff)| boff > 0) {
                                    best_match = Some((id, 0));
                                }
                            }
                        }
                    } else if (6..=8).contains(&words[0])
                        && (1..=2).contains(&words[1])
                        && (4..=6).contains(&words[2])
                    {
                        if best_match.is_none() {
                            best_match = Some((ObjectiveId::Consume, 1));
                        }
                    } else if (6..=8).contains(&words[0])
                        && (5..=7).contains(&words[1])
                        && (7..=9).contains(&words[2])
                    {
                        if best_match.is_none() {
                            best_match = Some((ObjectiveId::Incident, 1));
                        }
                    }
                }
            }
        }
    }
    best_match.map(|(id, _)| id)
}

#[derive(Clone, Copy, Debug)]
pub struct DecisionRead {
    pub ratway: bool,
    pub office: bool,
    pub replace: bool,
    pub consume: bool,
    pub hunt: bool,
    pub end: bool,
    pub title: bool,
}

fn read_decision_from_lines(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    lines_by_r: &[Vec<Line>],
) -> Option<DecisionRead> {
    let bottom = loc.at(0.0, 3.0).1.max(loc.at(5.0, 3.0).1) + 30.0 * loc.s;
    if bottom >= (img.height as f64) - 20.0 {
        return None;
    }
    let x0 = (loc.at(0.0, 0.0).0.min(loc.at(0.0, 3.0).0) - 40.0 * loc.s).max(0.0);
    let x1 = (loc.at(5.0, 0.0).0.max(loc.at(5.0, 3.0).0) + 40.0 * loc.s)
        .min((img.width - 1) as f64);

    let mut title = false;
    let mut pick_cell = false;
    let mut replace = false;
    let mut hunt = false;
    let mut end = false;
    let mut labels: Vec<(i32, i32, f64, i32)> = Vec::new();
    let mut hunts: Vec<(i32, i32, f64, i32)> = Vec::new();
    let mut single: Vec<(isize, f64, i32, i32, f64, i32)> = Vec::new();
    let mut ends: Vec<(i32, f64, i32)> = Vec::new();
    let mut lefts: Vec<(i32, f64)> = Vec::new();

    let near = |w: &[isize], want: &[isize]| -> bool {
        if w.len() < want.len() {
            return false;
        }
        let tail = &w[w.len() - want.len()..];
        tail.iter()
            .zip(want.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<isize>()
            <= 2
    };

    for lines in lines_by_r {
        for line in lines {
            for p in &line.pieces {
                let n: isize = p.words.iter().map(|w| w.glyphs.len() as isize).sum();
                let span = ((p.x1 - p.x0) as f64) / (line.h.max(1) as f64);
                if p.words.len() <= 2 && (5..=11).contains(&n) {
                    single.push((n, span, p.x0, p.x1, line.mid, line.h));
                }
                if (14..=20).contains(&n) && span > 7.0 && span < 15.0 {
                    let words: Vec<isize> =
                        p.words.iter().map(|w| w.glyphs.len() as isize).collect();
                    if near(&words, &[6, 2, 9]) && !near(&words, &[4, 4, 8]) {
                        pick_cell = true;
                    } else if near(&words, &[4, 4, 8]) {
                        title = true;
                    }
                } else if p.words.len() <= 2
                    && (6..=8).contains(&n)
                    && span > 2.8
                    && span < 6.8
                {
                    labels.push((p.x0, p.x1, line.mid, line.h));
                } else if p.words.len() == 1
                    && (4..=5).contains(&n)
                    && span > 1.8
                    && span < 4.2
                {
                    hunts.push((p.x0, p.x1, line.mid, line.h));
                } else if n == 3 && span > 1.5 && span < 3.2 {
                    end = true;
                    if p.words.len() == 1 {
                        ends.push((p.x0, line.mid, line.h));
                    }
                }
                if (6..=13).contains(&n) && span > 3.0 && span < 8.0 {
                    lefts.push((p.x1, line.mid));
                }
            }
        }
    }

    let red_ratio = |rx0: usize, ry0: usize, rx1: usize, ry1: usize| -> f32 {
        let mut n = 0usize;
        let mut hit = 0usize;
        let mut y = ry0;
        while y < ry1 {
            let mut x = rx0;
            while x < rx1 {
                let (r, g, b) = img.rgb_at(x as isize, y as isize);
                n += 1;
                if r >= 76 && r > g && r > b {
                    let (h, s, v) = hsv(r as f32, g as f32, b as f32);
                    if (h < 12.0 || h > 348.0) && s > 0.55 && v > 0.30 {
                        hit += 1;
                    }
                }
                x += 2;
            }
            y += 2;
        }
        if n > 0 {
            (hit as f32) / (n as f32)
        } else {
            0.0
        }
    };

    let strip = ((100.0 * loc.s).round() as usize).max(8);
    let mut red = 0.0_f32;
    let mut sx = x0.round() as usize;
    let sx_end = x1 as usize;
    let by_start = bottom.round() as usize;
    let by_end = img.height.min(by_start + (260.0 * loc.s) as usize);
    while sx + strip <= sx_end {
        red = red.max(red_ratio(sx, by_start, sx + strip, by_end));
        sx += strip;
    }
    let consume = red > 0.01;

    let red_by = |lx0: i32, lx1: i32, lmid: f64, lh: i32| -> f32 {
        let w = lx1 - lx0;
        let y0 = ((lmid - 5.0 * (lh as f64)).round() as isize).max(0) as usize;
        let y1 = ((lmid + (lh as f64)) as usize).min(img.height);
        let rx0 = (lx0 - w / 2).max(0) as usize;
        let rx1 = ((lx1 + w / 2) as usize).min(img.width);
        red_ratio(rx0, y0, rx1, y1)
    };

    for &(lx0, lx1, lmid, lh) in &labels {
        if red_by(lx0, lx1, lmid, lh) < 0.01 {
            replace = true;
        }
    }
    for &(hx0, hx1, hmid, hh) in &hunts {
        let same_row = ends
            .iter()
            .any(|&(ex0, emid, eh)| hx1 < ex0 && (hmid - emid).abs() < (eh as f64) * 0.6)
            || labels
                .iter()
                .any(|&(lx0, _, lmid, lh)| (hx0 - lx0).abs() > 2 * lh && (hmid - lmid).abs() < (lh as f64) * 0.6);
        if same_row && red_by(hx0, hx1, hmid, hh) < 0.01 {
            hunt = true;
        }
    }

    let office = single.iter().any(|&(an, aspan, ax0, ax1, amid, ah)| {
        (7..=11).contains(&an)
            && aspan > 3.8
            && aspan < 7.0
            && red_by(ax0, ax1, amid, ah) < 0.01
            && single.iter().any(|&(bn, bspan, bx0, bx1, bmid, bh)| {
                (5..=7).contains(&bn)
                    && bspan > 3.0
                    && bspan < 5.8
                    && bx0 > ax1
                    && (bmid - amid).abs() < (ah as f64) * 0.6
                    && red_by(bx0, bx1, bmid, bh) < 0.01
            })
    });

    if pick_cell && !title {
        return Some(DecisionRead {
            ratway: true,
            office: false,
            replace: false,
            consume: false,
            hunt: false,
            end: false,
            title: false,
        });
    }
    let row = ends.iter().any(|&(ex0, emid, eh)| {
        lefts.iter().any(|&(lx1, lmid)| {
            (lmid - emid).abs() < (eh as f64) * 0.5
                && lx1 < ex0
                && (ex0 - lx1) < 30 * eh
        })
    });
    if !title && !(row && (consume || replace)) {
        return None;
    }
    if office {
        return Some(DecisionRead {
            ratway: false,
            office: true,
            replace: false,
            consume: false,
            hunt: false,
            end: false,
            title,
        });
    }
    if !replace && !consume && !end {
        return None;
    }
    Some(DecisionRead {
        ratway: false,
        office: false,
        replace,
        consume,
        hunt,
        end,
        title,
    })
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ScanProfile {
    pub loc_ms: f64,
    pub cells_ms: f64,
    pub ocr_ms: f64,
    pub pick_ms: f64,
    pub total_ms: f64,
}

#[derive(Clone, Debug)]
pub struct ScanResult {
    pub ok: bool,
    pub loc: Option<BoardLoc>,
    pub cells: Option<[ScannedCell; 16]>,
    pub warden: Option<WardenRead>,
    pub doors: Option<DoorsRead>,
    pub days: Option<(u8, f32)>,
    pub tools: FixedVec<ScannedTool, 6>,
    pub chromatic: Option<usize>,
    pub chain: Option<ChainRead>,
    pub decision: Option<DecisionRead>,
    pub pick: Option<PickScreenRead>,
    pub title: Option<TitleRead>,
    pub profile: ScanProfile,
}

fn has_devour_banner(img: &ImageView<'_>) -> bool {
    let x0 = (img.width * 3) / 10;
    let x1 = (img.width * 7) / 10;
    let y0 = (img.height * 45) / 100;
    let y1 = (img.height * 90) / 100;
    if x1 <= x0 || y1 <= y0 {
        return false;
    }
    let step_x = ((x1 - x0) / 32).max(2);
    let step_y = ((y1 - y0) / 24).max(2);
    for y in (y0..y1).step_by(step_y) {
        let mut hits = 0usize;
        let mut total = 0usize;
        for x in (x0..x1).step_by(step_x) {
            let (r, g, b) = img.rgb_at(x as isize, y as isize);
            total += 1;
            if r >= 102 && r > g && r > b {
                let (h, s, v) = hsv(r as f32, g as f32, b as f32);
                if (h < 16.0 || h > 340.0) && s > 0.45 && v > 0.40 {
                    hits += 1;
                }
            }
        }
        if total > 0 && hits * 6 >= total {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ScanOptions {
    pub skip_days: bool,
}

pub fn scan_image_with_arena(
    img: &ImageView<'_>,
    hint: Option<BoardLoc>,
    arena: &mut ScanArena,
) -> ScanResult {
    scan_image_with_options_and_arena(img, hint, ScanOptions::default(), arena)
}

pub fn scan_image_with_options_and_arena(
    img: &ImageView<'_>,
    hint: Option<BoardLoc>,
    opts: ScanOptions,
    arena: &mut ScanArena,
) -> ScanResult {
    use rayon::prelude::*;

    arena.reset();
    let t_total = std::time::Instant::now();
    let t_loc = std::time::Instant::now();
    let mut loc = locate_board(img, hint);
    if loc.map_or(false, |l| l.score >= 0.25 && l.score < 0.65) && has_devour_banner(img) {
        if let Some(by_labels) = locate_by_labels_with_slot(img, &mut arena.slots[0]) {
            if by_labels.cells_matched >= 8 {
                loc = Some(by_labels);
            }
        }
    }
    let loc_ms = t_loc.elapsed().as_secs_f64() * 1000.0;

    let valid_board = loc.map_or(false, |l| l.by_labels || l.score >= 0.65);
    if !valid_board {
        let total_ms = t_total.elapsed().as_secs_f64() * 1000.0;
        return ScanResult {
            ok: false,
            loc,
            cells: None,
            warden: None,
            doors: None,
            days: None,
            tools: FixedVec::new(),
            chromatic: None,
            chain: None,
            decision: None,
            pick: None,
            title: None,
            profile: ScanProfile {
                loc_ms,
                cells_ms: 0.0,
                ocr_ms: 0.0,
                pick_ms: 0.0,
                total_ms,
            },
        };
    }

    let loc = loc.unwrap();
    let radii = text_radii_for_scale(loc.s);
    let bottom = loc.at(0.0, 3.0).1.max(loc.at(5.0, 3.0).1) + 30.0 * loc.s;
    let board_cx = (loc.at(0.0, 1.5).0 + loc.at(5.0, 1.5).0) * 0.5;
    let bx0 = (board_cx - 420.0 * loc.s)
        .max(loc.at(0.0, 0.0).0.min(loc.at(0.0, 3.0).0) - 40.0 * loc.s)
        .max(0.0);
    let bx1 = (board_cx + 420.0 * loc.s)
        .min(loc.at(5.0, 0.0).0.max(loc.at(5.0, 3.0).0) + 40.0 * loc.s)
        .min((img.width - 1) as f64);
    let chain_banner = chain_banner_score(img, &loc);
    let has_chain = chain_banner >= 0.20;

    let ScanArena {
        slots: [slot0, slot1, slot2, slot3, slot4, slot5],
        cell_slots,
    } = arena;

    let mut cells = [ScannedCell::default(); 16];
    let ((warden, event_blobs, cells_ms), (((bottom_lines, mut doors), (days, (tools, chromatic))), ocr_ms)) =
        rayon::join(
            || {
                let t_cells = std::time::Instant::now();
                let (w, eb) = rayon::join(
                    || {
                        cells
                            .par_iter_mut()
                            .zip(cell_slots.par_iter_mut())
                            .enumerate()
                            .for_each(|(n, (out, cslot))| {
                                *out = read_cell_with_slot(img, &loc, n as u8, cslot);
                            });
                    },
                    || {
                        rayon::join(
                            || find_warden_with_slot(img, &loc, slot0),
                            || find_event_blobs_with_slot(img, &loc, slot5),
                        )
                    },
                )
                .1;
                (w, eb, t_cells.elapsed().as_secs_f64() * 1000.0)
            },
            || {
                if loc.by_labels {
                    (((Vec::new(), None), (None, (FixedVec::new(), None))), 0.0)
                } else {
                    let t_ocr = std::time::Instant::now();
                    let res = rayon::join(
                        || {
                            if !has_chain && bottom < (img.height as f64) - 20.0 {
                                let max_h = (50.0 * loc.s).max(24.0);
                                let by1 = (bottom + 215.0 * loc.s)
                                    .min((img.height as f64 - 16.0 * loc.s).max(bottom + 40.0))
                                    .min((img.height - 1) as f64);
                                let mut lines_by_r = text_lines_radii_with_slot(
                                    img,
                                    bx0,
                                    bottom,
                                    bx1,
                                    by1,
                                    6.0,
                                    max_h,
                                    &radii[1..3],
                                    0.34,
                                    slot1,
                                );
                                let fast_doors = read_doors_from_lines(img, &lines_by_r);
                                let fast_dec = if fast_doors.is_none() {
                                    read_decision_from_lines(img, &loc, &lines_by_r)
                                } else {
                                    None
                                };
                                let dec_complete = fast_dec.is_some_and(|d| {
                                    d.office || d.ratway || (d.end && (d.replace || d.consume))
                                });
                                if fast_doors.is_some()
                                    || dec_complete
                                    || lines_by_r.iter().all(|v| v.is_empty())
                                {
                                    (lines_by_r, fast_doors)
                                } else {
                                    let rem = [radii[0], radii[3]];
                                    let extra = text_lines_radii_with_slot(
                                        img,
                                        bx0,
                                        bottom,
                                        bx1,
                                        by1,
                                        6.0,
                                        max_h,
                                        &rem,
                                        0.34,
                                        slot1,
                                    );
                                    lines_by_r.extend(extra);
                                    let doors = read_doors_from_lines(img, &lines_by_r);
                                    (lines_by_r, doors)
                                }
                            } else {
                                (Vec::new(), None)
                            }
                        },
                        || {
                            rayon::join(
                                || {
                                    if opts.skip_days {
                                        None
                                    } else {
                                        read_days_with_slot(img, &loc, slot2)
                                    }
                                },
                                || {
                                    rayon::join(
                                        || read_tools_with_slot(img, &loc, slot3).0,
                                        || read_chromatic_with_slot(img, &loc, slot4),
                                    )
                                },
                            )
                        },
                    );
                    (res, t_ocr.elapsed().as_secs_f64() * 1000.0)
                }
            },
        );
    let events = match_events_from_blobs(&event_blobs, &loc, &cells, warden.as_ref());
    let chain = read_chain_screen_with_banner(
        img,
        &loc,
        &cells,
        warden.as_ref(),
        chain_banner,
    );

    refine_doors_with_warden(&mut doors, warden.map(|w| w.cell));

    let mut ev_bits = events;
    while ev_bits != 0 {
        let n = ev_bits.trailing_zeros() as usize;
        ev_bits &= ev_bits - 1;
        let c = &mut cells[n];
        if c.kind == ScannedCellKind::Locked || c.kind == ScannedCellKind::Inmate {
            c.event = true;
        }
    }

    let decision = if doors.is_some() || chain.is_some() || bottom_lines.is_empty() {
        None
    } else {
        read_decision_from_lines(img, &loc, &bottom_lines)
    };
    let total_ms = t_total.elapsed().as_secs_f64() * 1000.0;

    ScanResult {
        ok: true,
        loc: Some(loc),
        cells: Some(cells),
        warden,
        doors,
        days,
        tools,
        chromatic,
        chain,
        decision,
        pick: None,
        title: None,
        profile: ScanProfile {
            loc_ms,
            cells_ms,
            ocr_ms,
            pick_ms: 0.0,
            total_ms,
        },
    }
}

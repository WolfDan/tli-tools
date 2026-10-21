//! Pre-allocated scratch buffers (`ScanArena`, `ScanScratchSlot`, `CellScratchSlot`) and connected-component blob extraction.

use super::board::board_template_refs;
use super::image::ImageView;

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
            sat: alloc_pre_faulted(1024 * 1024, 0u32), // 4 MB
            elig_coords: alloc_pre_faulted(256 * 1024, (0u16, 0u16, 0u16)), // 1.5 MB
            hit_coords: alloc_pre_faulted(32 * 1024, (0u16, 0u16)), // 128 KB
            mask: alloc_pre_faulted(3 * 1024 * 1024, 0u8), // 3 MB
            stack: alloc_pre_faulted(8 * 1024, (0u32, 0u32)), // 64 KB
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

pub(crate) fn blobs_from_mask_hits(
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
pub(crate) fn blobs_rgb_strided_with_slot<F>(
    img: &ImageView<'_>,
    [x0, y0, x1, y1]: [f64; 4],
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

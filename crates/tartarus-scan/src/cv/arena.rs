//! Pre-allocated scratch buffers (`ScanArena`, `ScanScratchSlot`, `CellScratchSlot`)
//! and connected-component blob extraction.

use super::board::board_template_refs;
use super::image::ImageView;

const FLOOD_ITERATIONS_MAX: usize = 16 * 1024 * 1024;

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
#[repr(C, align(64))]
#[derive(Default)]
pub struct ScanScratchSlot {
    pub sat: Vec<u32>,
    pub elig_coords: Vec<(u16, u16, u16)>,
    pub hit_coords: Vec<(u16, u16)>,
    pub mask: Vec<u8>,
    pub stack: Vec<(u32, u32)>,
}

const _: () = {
    assert!(std::mem::align_of::<ScanScratchSlot>() == 64);
};

fn alloc_pre_faulted<T: Copy>(cap: usize, non_zero_val: T) -> Vec<T> {
    assert!(cap > 0);
    let mut v = vec![non_zero_val; cap];
    std::hint::black_box(&mut v);
    v.clear();
    assert!(v.capacity() >= cap);
    v
}

impl ScanScratchSlot {
    pub fn with_capacity() -> Self {
        let slot = Self {
            sat: alloc_pre_faulted(1024 * 1024, 0u32), // 4 MB
            elig_coords: alloc_pre_faulted(256 * 1024, (0u16, 0u16, 0u16)), // 1.5 MB
            hit_coords: alloc_pre_faulted(32 * 1024, (0u16, 0u16)), // 128 KB
            mask: alloc_pre_faulted(3 * 1024 * 1024, 0u8), // 3 MB
            stack: alloc_pre_faulted(8 * 1024, (0u32, 0u32)), // 64 KB
        };
        assert!(slot.sat.capacity() >= 1024 * 1024);
        assert!(slot.mask.capacity() >= 3 * 1024 * 1024);
        slot
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
        debug_assert!(self.hit_coords.is_empty());
        debug_assert!(self.stack.is_empty());
    }
}

/// Pre-allocated per-cell blur scratch buffers (`row_buf` + `patch`) for `read_cell_with_slot`.
pub struct CellScratchSlot {
    pub row_buf: Vec<f32>,
    pub patch: Vec<f32>,
}

impl CellScratchSlot {
    pub fn with_capacity() -> Self {
        let slot = Self {
            row_buf: alloc_pre_faulted(130 * 112, 0.0_f32),
            patch: alloc_pre_faulted(128 * 112, 0.0_f32),
        };
        assert!(slot.row_buf.capacity() >= 130 * 112);
        assert!(slot.patch.capacity() >= 128 * 112);
        slot
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
        debug_assert!(self.row_buf.capacity() <= 256 * 128);
        debug_assert!(self.patch.capacity() <= 256 * 128);
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
        let arena = Self {
            slots: std::array::from_fn(|_| ScanScratchSlot::with_capacity()),
            cell_slots: std::array::from_fn(|_| CellScratchSlot::with_capacity()),
        };
        assert_eq!(arena.slots.len(), 6);
        assert_eq!(arena.cell_slots.len(), 16);
        arena
    }

    #[inline]
    pub fn reset(&mut self) {
        assert_eq!(self.slots.len(), 6);
        assert_eq!(self.cell_slots.len(), 16);
        for slot in &mut self.slots {
            slot.reset();
        }
        for slot in &mut self.cell_slots {
            slot.reset();
        }
    }
}

#[derive(Clone, Copy)]
struct BlobAcc {
    n: usize,
    sx: usize,
    sy: usize,
    x_min: i32,
    y_min: i32,
    x_max: i32,
    y_max: i32,
}

impl BlobAcc {
    #[inline]
    fn new() -> Self {
        let acc = Self {
            n: 0,
            sx: 0,
            sy: 0,
            x_min: i32::MAX,
            y_min: i32::MAX,
            x_max: -1,
            y_max: -1,
        };
        debug_assert_eq!(acc.n, 0);
        debug_assert!(acc.x_max < acc.x_min);
        acc
    }

    #[inline]
    fn add(&mut self, xu: u32, yu: u32) {
        debug_assert!(xu <= i32::MAX as u32);
        debug_assert!(yu <= i32::MAX as u32);
        self.n += 1;
        self.sx += xu as usize;
        self.sy += yu as usize;
        let xi = xu as i32;
        let yi = yu as i32;
        self.x_min = self.x_min.min(xi);
        self.x_max = self.x_max.max(xi);
        self.y_min = self.y_min.min(yi);
        self.y_max = self.y_max.max(yi);
        debug_assert!(self.x_max >= self.x_min);
    }

    #[inline]
    fn into_blob(self, bx0: usize, by0: usize) -> Blob {
        debug_assert!(self.n > 0);
        debug_assert!(self.x_max >= self.x_min);
        debug_assert!(self.y_max >= self.y_min);
        Blob {
            n: self.n,
            cx: (bx0 as f64) + (self.sx as f64) / (self.n as f64),
            cy: (by0 as f64) + (self.sy as f64) / (self.n as f64),
            x0: (bx0 as i32) + self.x_min,
            y0: (by0 as i32) + self.y_min,
            x1: (bx0 as i32) + self.x_max,
            y1: (by0 as i32) + self.y_max,
            w: self.x_max - self.x_min + 1,
            h: self.y_max - self.y_min + 1,
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
    assert!(w > 0 && h > 0);
    assert!(mask.len() >= w * h);
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
        let mut acc = BlobAcc::new();
        let mut iterations = 0usize;
        while let Some((xu, yu)) = stack.pop() {
            iterations += 1;
            assert!(iterations < FLOOD_ITERATIONS_MAX);
            acc.add(xu, yu);
            let x = xu as usize;
            let y = yu as usize;
            let j = y * w + x;
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
        out.push(acc.into_blob(bx0, by0));
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
    assert!(img.width > 0);
    assert!(img.height > 0);
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
fn probe_rgb_neighbor<F>(
    img: &ImageView<'_>,
    [bx0, by0, w]: [usize; 3],
    [nx, ny]: [usize; 2],
    skip_rect: Option<[usize; 4]>,
    slot: &mut ScanScratchSlot,
    test: &F,
) where
    F: Fn(u8, u8, u8) -> bool,
{
    let ni = ny * w + nx;
    debug_assert!(ni < slot.mask.len());
    if slot.mask[ni] != 0 {
        return;
    }
    let px = bx0 + nx;
    let py = by0 + ny;
    if let Some([sx0, sy0, sx1, sy1]) = skip_rect
        && py >= sy0
        && py <= sy1
        && px >= sx0
        && px < sx1
    {
        return;
    }
    let no = (py * img.width + px) * 4;
    debug_assert!(no + 3 < img.data.len());
    slot.hit_coords.push((nx as u16, ny as u16));
    if test(img.data[no], img.data[no + 1], img.data[no + 2]) {
        slot.mask[ni] = 1;
        slot.stack.push((nx as u32, ny as u32));
    } else {
        slot.mask[ni] = 2;
    }
}

fn flood_rgb_component<F>(
    img: &ImageView<'_>,
    [bx0, by0, w, h]: [usize; 4],
    [sx0, sy0]: [usize; 2],
    skip_rect: Option<[usize; 4]>,
    slot: &mut ScanScratchSlot,
    test: &F,
) -> Blob
where
    F: Fn(u8, u8, u8) -> bool,
{
    assert!(w > 0 && h > 0);
    assert!(sx0 < w && sy0 < h);
    let i0 = sy0 * w + sx0;
    slot.mask[i0] = 1;
    slot.hit_coords.push((sx0 as u16, sy0 as u16));
    slot.stack.push((sx0 as u32, sy0 as u32));
    let mut acc = BlobAcc::new();
    let mut iterations = 0usize;
    while let Some((xu, yu)) = slot.stack.pop() {
        iterations += 1;
        assert!(iterations < FLOOD_ITERATIONS_MAX);
        acc.add(xu, yu);
        let x = xu as usize;
        let y = yu as usize;
        if x > 0 {
            probe_rgb_neighbor(img, [bx0, by0, w], [x - 1, y], skip_rect, slot, test);
        }
        if x + 1 < w {
            probe_rgb_neighbor(img, [bx0, by0, w], [x + 1, y], skip_rect, slot, test);
        }
        if y > 0 {
            probe_rgb_neighbor(img, [bx0, by0, w], [x, y - 1], skip_rect, slot, test);
        }
        if y + 1 < h {
            probe_rgb_neighbor(img, [bx0, by0, w], [x, y + 1], skip_rect, slot, test);
        }
    }
    acc.into_blob(bx0, by0)
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
    assert!(img.width > 0);
    assert!(img.height > 0);
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1.ceil() as isize).min(img.width as isize - 1);
    let by1 = (y1.ceil() as isize).min(img.height as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return Vec::new();
    }
    let w = (bx1 as usize) - bx0 + 1;
    let h = (by1 as usize) - by0 + 1;
    let n_px = w * h;
    let step_y = step_y.max(1);
    if slot.mask.len() < n_px {
        slot.mask.resize(n_px, 0);
    }
    slot.hit_coords.clear();
    slot.stack.clear();
    let mut out = Vec::new();
    for sy0 in (0..h).step_by(step_y) {
        let py0 = by0 + sy0;
        let row_off = sy0 * w;
        let row_base = (py0 * img.width + bx0) * 4;
        let row_data = &img.data[row_base..row_base + w * 4];
        for sx0 in 0..w {
            let px0 = bx0 + sx0;
            if let Some([sk_x0, sk_y0, sk_x1, sk_y1]) = skip_rect
                && py0 >= sk_y0
                && py0 <= sk_y1
                && px0 >= sk_x0
                && px0 < sk_x1
            {
                continue;
            }
            let o = sx0 * 4;
            if slot.mask[row_off + sx0] == 0 && test(row_data[o], row_data[o + 1], row_data[o + 2])
            {
                out.push(flood_rgb_component(
                    img,
                    [bx0, by0, w, h],
                    [sx0, sy0],
                    skip_rect,
                    slot,
                    &test,
                ));
            }
        }
    }
    for &(hx, hy) in slot.hit_coords.iter() {
        slot.mask[(hy as usize) * w + (hx as usize)] = 0;
    }
    slot.hit_coords.clear();
    out
}

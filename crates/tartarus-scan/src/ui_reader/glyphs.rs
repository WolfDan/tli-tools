//! Pre-allocated `#[repr(C, align(64))]` scratch buffers, connected-component blob extraction,
//! and `std::simd::f32x16` glyph & prefix correlation (`dot_vec160_simd`, `dot_vec400_simd`).

use super::data::{
    DIGITS, GLYPH_H, GLYPH_W, PREFIX_GLYPHS, PREFIX_H, PREFIX_W, SimdVec160, SimdVec400,
};
use super::simd::{FrameView, dot_vec160_simd, dot_vec400_simd, normalize_zero_mean_in_place_simd};

const FLOOD_ITERATIONS_MAX: usize = 16 * 1024 * 1024;

const _: () = {
    assert!(FLOOD_ITERATIONS_MAX == 16 * 1024 * 1024);
    assert!(GLYPH_W * GLYPH_H == 160);
    assert!(PREFIX_W * PREFIX_H == 400);
};

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

/// Pre-allocated cache-line-aligned (`64B`) scratch slot for zero-allocation ROI glyph reading.
#[repr(C, align(64))]
#[derive(Default)]
pub struct UiScratchSlot {
    pub sat: Vec<u32>,
    pub elig_coords: Vec<(u16, u16, u16)>,
    pub hit_coords: Vec<(u16, u16)>,
    pub mask: Vec<u8>,
    pub stack: Vec<(u32, u32)>,
}

/// Pre-allocated cache-line-aligned (`64B`) per-cell blur scratch slot (`row_buf` + `patch`).
#[repr(C, align(64))]
#[derive(Default)]
pub struct CellScratchSlot {
    pub row_buf: Vec<f32>,
    pub patch: Vec<f32>,
}

const _: () = {
    assert!(std::mem::align_of::<UiScratchSlot>() == 64);
    assert!(std::mem::align_of::<CellScratchSlot>() == 64);
};

fn alloc_pre_faulted<T: Copy>(cap: usize, non_zero_val: T) -> Vec<T> {
    assert!(cap > 0);
    let mut v = vec![non_zero_val; cap];
    std::hint::black_box(&mut v);
    v.clear();
    assert!(v.capacity() >= cap);
    v
}

impl UiScratchSlot {
    pub fn with_capacity() -> Self {
        let slot = Self {
            sat: alloc_pre_faulted(1024 * 1024, 0u32),
            elig_coords: alloc_pre_faulted(256 * 1024, (0u16, 0u16, 0u16)),
            hit_coords: alloc_pre_faulted(32 * 1024, (0u16, 0u16)),
            mask: alloc_pre_faulted(3 * 1024 * 1024, 0u8),
            stack: alloc_pre_faulted(8 * 1024, (0u32, 0u32)),
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

pub(crate) fn blobs_with_slot<F>(
    frame: &FrameView<'_>,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    slot: &mut UiScratchSlot,
    test: F,
) -> Vec<Blob>
where
    F: Fn(usize, usize) -> bool + Sync,
{
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1.ceil() as isize).min(frame.width() as isize - 1);
    let by1 = (y1.ceil() as isize).min(frame.height() as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return Vec::new();
    }
    let bx1 = bx1 as usize;
    let by1 = by1 as usize;
    let w = bx1 - bx0 + 1;
    let h = by1 - by0 + 1;
    let n = w * h;
    let UiScratchSlot {
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
    frame: &FrameView<'_>,
    [bx0, by0, w]: [usize; 3],
    [nx, ny]: [usize; 2],
    skip_rect: Option<[usize; 4]>,
    slot: &mut UiScratchSlot,
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
    let no = frame.row_offset(py) + px * 4;
    debug_assert!(no + 3 < frame.pixels_rgba8.len());
    slot.hit_coords.push((nx as u16, ny as u16));
    if test(
        frame.pixels_rgba8[no],
        frame.pixels_rgba8[no + 1],
        frame.pixels_rgba8[no + 2],
    ) {
        slot.mask[ni] = 1;
        slot.stack.push((nx as u32, ny as u32));
    } else {
        slot.mask[ni] = 2;
    }
}

fn flood_rgb_component<F>(
    frame: &FrameView<'_>,
    [bx0, by0, w, h]: [usize; 4],
    [sx0, sy0]: [usize; 2],
    skip_rect: Option<[usize; 4]>,
    slot: &mut UiScratchSlot,
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
            probe_rgb_neighbor(frame, [bx0, by0, w], [x - 1, y], skip_rect, slot, test);
        }
        if x + 1 < w {
            probe_rgb_neighbor(frame, [bx0, by0, w], [x + 1, y], skip_rect, slot, test);
        }
        if y > 0 {
            probe_rgb_neighbor(frame, [bx0, by0, w], [x, y - 1], skip_rect, slot, test);
        }
        if y + 1 < h {
            probe_rgb_neighbor(frame, [bx0, by0, w], [x, y + 1], skip_rect, slot, test);
        }
    }
    acc.into_blob(bx0, by0)
}

#[inline]
pub(crate) fn blobs_rgb_strided_with_slot<F>(
    frame: &FrameView<'_>,
    [x0, y0, x1, y1]: [f64; 4],
    skip_rect: Option<[usize; 4]>,
    step_y: usize,
    slot: &mut UiScratchSlot,
    test: F,
) -> Vec<Blob>
where
    F: Fn(u8, u8, u8) -> bool,
{
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1.ceil() as isize).min(frame.width() as isize - 1);
    let by1 = (y1.ceil() as isize).min(frame.height() as isize - 1);
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
        let row_base = frame.row_offset(py0) + bx0 * 4;
        let row_data = &frame.pixels_rgba8[row_base..row_base + w * 4];
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
                    frame,
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

fn cluster_blobs_by_row(mut found: Vec<Blob>, h_min: f64, h_max: f64) -> Vec<Line> {
    assert!(h_min > 0.0);
    assert!(h_max >= h_min);
    found.retain(|b| {
        (b.h as f64) >= h_min * 0.45
            && (b.h as f64) <= h_max
            && (b.w as f64) <= h_max * 1.2
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
    lines
}

fn merge_overlapping_glyphs(glyphs: Vec<Blob>) -> Vec<Blob> {
    debug_assert!(!glyphs.is_empty());
    let mut merged: Vec<Blob> = Vec::with_capacity(glyphs.len());
    for g in glyphs {
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
    debug_assert!(!merged.is_empty());
    merged
}

fn partition_line_pieces(line: &mut Line) {
    assert!(!line.glyphs.is_empty());
    assert!(line.h > 0);
    let lh = line.h as f64;
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
}

fn group_blobs_into_lines(found: Vec<Blob>, h_min: f64, h_max: f64) -> Vec<Line> {
    assert!(h_min > 0.0);
    assert!(h_max >= h_min);
    let lines = cluster_blobs_by_row(found, h_min, h_max);
    let mut out = Vec::with_capacity(lines.len());
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
        line.glyphs = merge_overlapping_glyphs(line.glyphs);
        partition_line_pieces(&mut line);
        out.push(line);
    }
    out
}

fn fill_sat_border_row(
    frame: &FrameView<'_>,
    sat: &mut [u32],
    dims: [usize; 5],
    sy: usize,
    py: usize,
) {
    let [bx0, w, r_max, sw, sat_stride] = dims;
    assert!(w > 0 && sw >= w + 2 * r_max);
    assert!((sy + 2) * sat_stride <= sat.len());
    let row_base = frame.row_offset(py) + bx0 * 4;
    let data = &frame.pixels_rgba8[row_base..row_base + w * 4];
    let prev_row = sy * sat_stride;
    let cur_row = (sy + 1) * sat_stride;
    sat[cur_row] = 0;
    let mut row_sum = 0u32;
    let l_q0 = (77 * (data[0] as u32) + 150 * (data[1] as u32) + 29 * (data[2] as u32)) >> 6;
    for sx in 0..r_max {
        row_sum += l_q0;
        sat[cur_row + sx + 1] = sat[prev_row + sx + 1] + row_sum;
    }
    for x in 0..w {
        let o = x * 4;
        let l_q =
            (77 * (data[o] as u32) + 150 * (data[o + 1] as u32) + 29 * (data[o + 2] as u32)) >> 6;
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
}

fn build_sat_and_eligible(
    frame: &FrameView<'_>,
    bounds: [usize; 6],
    sat_limit: f32,
    slot: &mut UiScratchSlot,
) {
    let [bx0, by0, by1, w, h, r_max] = bounds;
    assert!(w >= 4 && h >= 4);
    assert!(r_max >= 1);
    let sw = w + 2 * r_max;
    let sh = h + 2 * r_max;
    let sat_stride = sw + 1;
    let sat_len = (sh + 1) * sat_stride;
    if slot.sat.len() < sat_len {
        slot.sat.resize(sat_len, 0);
    }
    slot.sat[..sat_stride].fill(0);
    slot.elig_coords.clear();
    let sat_mul = (sat_limit * 256.0).round() as u32;
    let border_dims = [bx0, w, r_max, sw, sat_stride];
    for sy in 0..r_max {
        fill_sat_border_row(frame, &mut slot.sat, border_dims, sy, by0);
    }
    for y in 0..h {
        let sy = y + r_max;
        let py = by0 + y;
        let row_base = frame.row_offset(py) + bx0 * 4;
        let data = &frame.pixels_rgba8[row_base..row_base + w * 4];
        let prev_row = sy * sat_stride;
        let cur_row = (sy + 1) * sat_stride;
        slot.sat[cur_row] = 0;
        let mut row_sum = 0u32;
        let l_q0 = (77 * (data[0] as u32) + 150 * (data[1] as u32) + 29 * (data[2] as u32)) >> 6;
        for sx in 0..r_max {
            row_sum += l_q0;
            slot.sat[cur_row + sx + 1] = slot.sat[prev_row + sx + 1] + row_sum;
        }
        for x in 0..w {
            let o = x * 4;
            let (rr, gg, bb) = (data[o], data[o + 1], data[o + 2]);
            let l_256 = 77 * (rr as u32) + 150 * (gg as u32) + 29 * (bb as u32);
            row_sum += l_256 >> 6;
            slot.sat[cur_row + r_max + x + 1] = slot.sat[prev_row + r_max + x + 1] + row_sum;
            let mx = rr.max(gg).max(bb);
            if mx >= 56 && l_256 >= 56 * 256 {
                let mn = rr.min(gg).min(bb);
                if ((mx - mn) as u32) * 256 < sat_mul * (mx as u32) {
                    slot.elig_coords.push((x as u16, y as u16, l_256 as u16));
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
            slot.sat[cur_row + sx + 1] = slot.sat[prev_row + sx + 1] + row_sum;
        }
    }
    if !slot.elig_coords.is_empty() {
        for sy in (r_max + h)..sh {
            fill_sat_border_row(frame, &mut slot.sat, border_dims, sy, by1);
        }
    }
}

pub(crate) fn prepare_sat_region(
    frame: &FrameView<'_>,
    rect: [f64; 4],
    r_max: usize,
    sat_limit: f32,
    slot: &mut UiScratchSlot,
) -> Option<[usize; 5]> {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(sat_limit.is_finite() && r_max >= 1);
    let [x0, y0, x1, y1] = rect;
    let bx0 = (x0.floor() as isize).max(0) as usize;
    let by0 = (y0.floor() as isize).max(0) as usize;
    let bx1 = (x1.ceil() as isize).min(frame.width() as isize - 1);
    let by1 = (y1.ceil() as isize).min(frame.height() as isize - 1);
    if bx1 < bx0 as isize || by1 < by0 as isize {
        return None;
    }
    let bx1 = bx1 as usize;
    let by1 = by1 as usize;
    let w = bx1 - bx0 + 1;
    let h = by1 - by0 + 1;
    if w < 4 || h < 4 {
        return None;
    }
    build_sat_and_eligible(frame, [bx0, by0, by1, w, h, r_max], sat_limit, slot);
    (!slot.elig_coords.is_empty()).then_some([bx0, by0, w, h, r_max])
}

pub(crate) fn scan_radii_from_built_sat(
    dims: [usize; 5],
    h_bounds: (f64, f64),
    radii: &[usize],
    slot: &mut UiScratchSlot,
) -> Vec<Vec<Line>> {
    let [bx0, by0, w, h, r_max] = dims;
    assert!(w >= 4 && h >= 4);
    assert!(r_max >= 1);
    if slot.elig_coords.is_empty() || radii.is_empty() {
        return vec![Vec::new(); radii.len()];
    }
    let (h_min, h_max) = h_bounds;
    let sat_stride = w + 2 * r_max + 1;
    let n = w * h;
    if slot.mask.len() < n {
        slot.mask.resize(n, 0);
    }
    let mut results = Vec::with_capacity(radii.len());
    for &rad in radii {
        let r = rad.clamp(1, r_max);
        let span = 2 * r + 1;
        let inv_area_scaled = 0.25_f32 / ((span * span) as f32);
        slot.hit_coords.clear();
        for &(ex_u16, ey_u16, l_256) in slot.elig_coords.iter() {
            let (ex, ey) = (ex_u16 as usize, ey_u16 as usize);
            let (cx, cy) = (ex + r_max, ey + r_max);
            let (y0s, y1s) = ((cy - r) * sat_stride, (cy + r + 1) * sat_stride);
            let (x0s, x1s) = (cx - r, cx + r + 1);
            let box_sum = (slot.sat[y1s + x1s] + slot.sat[y0s + x0s])
                .saturating_sub(slot.sat[y0s + x1s] + slot.sat[y1s + x0s]);
            let around_val = (box_sum as f32) * inv_area_scaled;
            let l = (l_256 as f32) * (1.0 / 256.0);
            if l - around_val >= 16.0_f32.max(0.3 * around_val) {
                slot.mask[ey * w + ex] = 1;
                slot.hit_coords.push((ex_u16, ey_u16));
            }
        }
        let found = if slot.hit_coords.is_empty() {
            Vec::new()
        } else {
            blobs_from_mask_hits(
                &mut slot.mask[..n],
                &slot.hit_coords,
                &mut slot.stack,
                w,
                h,
                bx0,
                by0,
            )
        };
        results.push(group_blobs_into_lines(found, h_min, h_max));
    }
    results
}

pub(crate) fn text_lines_radii_with_slot(
    frame: &FrameView<'_>,
    rect: [f64; 4],
    h_bounds: (f64, f64),
    radii: &[usize],
    sat_limit: f32,
    slot: &mut UiScratchSlot,
) -> Vec<Vec<Line>> {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(sat_limit.is_finite());
    if radii.is_empty() {
        return Vec::new();
    }
    let r_max = radii.iter().copied().max().unwrap_or(1).max(1);
    let Some(dims) = prepare_sat_region(frame, rect, r_max, sat_limit, slot) else {
        return vec![Vec::new(); radii.len()];
    };
    scan_radii_from_built_sat(dims, h_bounds, radii, slot)
}

pub(crate) fn glyph_vector(frame: &FrameView<'_>, g: &Blob, line_h: i32) -> SimdVec160 {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(g.h > 0 && line_h > 0);
    let mut v = SimdVec160([0.0_f32; 160]);
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
                    val += frame.sample_sharp(sx, sy);
                }
            }
            val /= (sub * sub) as f32;
            v.0[j * GLYPH_W + i] = val;
            mean += val;
        }
    }
    mean /= 160.0;
    normalize_zero_mean_in_place_simd(&mut v.0, mean);
    v
}

pub(crate) fn digit_scores(frame: &FrameView<'_>, g: &Blob, line_h: i32) -> [f32; 10] {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(line_h > 0);
    let v = glyph_vector(frame, g, line_h);
    let mut out = [-1.0_f32; 10];
    for (d, refs) in DIGITS.iter().enumerate() {
        for r in *refs {
            let dot = dot_vec160_simd(&r.0, &v.0);
            if dot > out[d] {
                out[d] = dot;
            }
        }
    }
    out
}

pub(crate) const ALLOWED_CELLS: [u32; 15] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
pub(crate) const ALLOWED_DAYS: [u32; 41] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
];

#[inline]
const fn u32_digits(v: u32) -> ([usize; 2], usize) {
    assert!(v < 100);
    let res = if v >= 10 {
        ([(v / 10) as usize, (v % 10) as usize], 2)
    } else {
        ([v as usize, 0], 1)
    };
    assert!(res.1 >= 1 && res.1 <= 2);
    res
}

pub(crate) fn read_number(
    frame: &FrameView<'_>,
    glyphs: &[Blob],
    line_h: i32,
    allowed: &[u32],
) -> Vec<(u32, f32)> {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(line_h > 0);
    let n_glyphs = glyphs.len();
    if n_glyphs == 0 || n_glyphs > 2 {
        return Vec::new();
    }
    let mut per = [[-1.0_f32; 10]; 2];
    for (i, g) in glyphs.iter().enumerate() {
        per[i] = digit_scores(frame, g, line_h);
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

pub(crate) fn prefix_vector(
    frame: &FrameView<'_>,
    piece: &Piece,
    end: f64,
    dy0: i32,
    dy1: i32,
) -> SimdVec400 {
    assert!(!piece.glyphs.is_empty());
    assert!(end > 0.0 && end.is_finite());
    let g0 = &piece.glyphs[0];
    let x0 = g0.x0 as f64;
    let x1 = x0 + end * (g0.h as f64);
    let y0 = (g0.y0 + dy0) as f64;
    let y1 = (g0.y1 + dy1) as f64;
    let mut v = SimdVec400([0.0_f32; 400]);
    let mut mean = 0.0_f32;
    for j in 0..PREFIX_H {
        for i in 0..PREFIX_W {
            let sx = x0 + ((x1 - x0) * ((i as f64) + 0.5)) / (PREFIX_W as f64);
            let sy = y0 + ((y1 - y0) * ((j as f64) + 0.5)) / (PREFIX_H as f64);
            let val = frame.sample_sharp(sx, sy);
            v.0[j * PREFIX_W + i] = val;
            mean += val;
        }
    }
    mean /= 400.0;
    normalize_zero_mean_in_place_simd(&mut v.0, mean);
    v
}

pub(crate) fn prefix_like(frame: &FrameView<'_>, piece: &Piece) -> f32 {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(!piece.glyphs.is_empty());
    let pv0 = prefix_vector(frame, piece, 5.46, 0, 0);
    let mut best = -1.0_f32;
    for r in &PREFIX_GLYPHS {
        let dot = dot_vec400_simd(&r.0, &pv0.0);
        if dot > best {
            best = dot;
        }
    }
    if best < 0.36 || best >= 0.54 {
        return best;
    }
    const CENTER_PREFIX: [usize; 2] = [13, 40];
    const SHIFTS: [(f64, i32, i32); 8] = [
        (5.46, -1, -1),
        (5.46, 1, 1),
        (5.46, 0, -1),
        (5.46, 0, 1),
        (5.46, -1, 0),
        (5.46, 1, 0),
        (5.30, 0, 0),
        (5.62, 0, 0),
    ];
    for (end, dy0, dy1) in SHIFTS {
        let pv = prefix_vector(frame, piece, end, dy0, dy1);
        let mut shift_best = -1.0_f32;
        for &idx in &CENTER_PREFIX {
            let dot = dot_vec400_simd(&PREFIX_GLYPHS[idx].0, &pv.0);
            if dot > shift_best {
                shift_best = dot;
            }
        }
        if shift_best > best {
            best = shift_best;
        }
        if shift_best >= 0.44 {
            for r in &PREFIX_GLYPHS {
                let dot = dot_vec400_simd(&r.0, &pv.0);
                if dot > best {
                    best = dot;
                }
            }
        }
    }
    best
}

#[inline]
pub(crate) fn text_radii(frame: &FrameView<'_>) -> [usize; 4] {
    assert!(frame.width() > 0);
    assert!(frame.height() > 0);
    [0.006, 0.01, 0.016, 0.026].map(|f| ((frame.height() as f64 * f).round() as usize).max(3))
}

#[inline]
pub(crate) fn text_radii_for_scale(s: f64) -> [usize; 4] {
    assert!(s > 0.0);
    assert!(s.is_finite());
    let vh = 720.0 * s.max(0.5);
    [0.006, 0.01, 0.016, 0.026].map(|f| ((vh * f).round() as usize).max(3))
}

#[inline]
fn sharp_px(frame: &FrameView<'_>, x: isize, y: isize) -> f32 {
    debug_assert!(frame.width() > 0 && frame.height() > 0);
    let mut acc = 0.0_f32;
    for dy in -1..=1 {
        for dx in -1..=1 {
            acc += frame.gray_px(x + dx, y + dy);
        }
    }
    let out = acc * (1.0 / 9.0);
    debug_assert!(out.is_finite());
    out
}

pub(crate) fn split_glyph(frame: &FrameView<'_>, g: &Blob) -> Option<[Blob; 2]> {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(g.w > 0 && g.h > 0);
    let mut cut = -1i32;
    let mut least = f32::INFINITY;
    let start_x = ((g.x0 as f64) + (g.w as f64) * 0.3).round() as i32;
    let end_x = ((g.x1 as f64) - (g.w as f64) * 0.3).floor() as i32;
    for x in start_x..=end_x {
        let mut ink = 0.0_f32;
        for y in g.y0..=g.y1 {
            ink += sharp_px(frame, x as isize, y as isize);
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

#[derive(Clone, Copy, Debug)]
pub struct LabelBox {
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
    pub h: i32,
}

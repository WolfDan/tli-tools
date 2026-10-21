//! Window capture abstractions, SIMD BGRA->RGBA swizzle, and `StillnessTracker`.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::WindowCapturer;
#[cfg(target_os = "windows")]
pub use windows::WindowCapturer;

#[derive(Clone, Copy, Debug)]
pub struct CapturedFrame<'a> {
    pub width: usize,
    pub height: usize,
    pub rgba: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameWindowPlacement {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub monitor_index: Option<usize>,
    pub monitor_spatial_rank: Option<usize>,
    pub monitor: Option<MonitorRect>,
    pub total_monitors: usize,
}

#[inline]
fn luma_sad(a: &[f32; 64 * 36], b: &[f32; 64 * 36]) -> f32 {
    use std::simd::num::SimdFloat;
    use std::simd::prelude::*;
    let mut acc0 = f32x8::splat(0.0);
    let mut acc1 = f32x8::splat(0.0);
    let mut i = 0usize;
    while i < 64 * 36 {
        let va0 = f32x8::from_slice(&a[i..i + 8]);
        let vb0 = f32x8::from_slice(&b[i..i + 8]);
        let va1 = f32x8::from_slice(&a[i + 8..i + 16]);
        let vb1 = f32x8::from_slice(&b[i + 8..i + 16]);
        acc0 += (va0 - vb0).abs();
        acc1 += (va1 - vb1).abs();
        i += 16;
    }
    (acc0 + acc1).reduce_sum()
}

#[inline]
pub fn bgra_to_rgba_in_place(buf: &mut [u8]) {
    use std::simd::prelude::*;
    let alpha = u8x32::from_array([
        0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0,
        0, 255, 0, 0, 0, 255,
    ]);
    let n = buf.len();
    let mut i = 0usize;
    while i + 32 <= n {
        let v = u8x32::from_slice(&buf[i..i + 32]);
        let swapped = simd_swizzle!(
            v,
            [
                2, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15, 18, 17, 16, 19, 22, 21, 20,
                23, 26, 25, 24, 27, 30, 29, 28, 31
            ]
        ) | alpha;
        swapped.copy_to_slice(&mut buf[i..i + 32]);
        i += 32;
    }
    for px in buf[i..].as_chunks_mut::<4>().0 {
        px.swap(0, 2);
        px[3] = 255;
    }
}

/// Downsampled 64x36 luma buffer for sub-millisecond stillness checks.
pub struct StillnessTracker {
    prev_thumb: Option<[f32; 64 * 36]>,
    pub moved: bool,
    pub still_count: u8,
    pub last_diff: f32,
}

impl Default for StillnessTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl StillnessTracker {
    pub const W: usize = 64;
    pub const H: usize = 36;
    pub const STILL_DIFF: f32 = 1.0;
    pub const MOVE_DIFF: f32 = 2.0;
    pub const STILL_LOOKS: u8 = 7;

    pub fn new() -> Self {
        Self {
            prev_thumb: None,
            moved: false,
            still_count: 0,
            last_diff: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.prev_thumb = None;
        self.moved = false;
        self.still_count = 0;
        self.last_diff = 0.0;
    }

    pub fn compute_thumb(frame: &CapturedFrame<'_>) -> [f32; 64 * 36] {
        let mut out = [0.0_f32; 64 * 36];
        if frame.width == 0 || frame.height == 0 {
            return out;
        }
        for ty in 0..Self::H {
            let sy = (ty * frame.height) / Self::H;
            let row_off = sy * frame.width;
            for tx in 0..Self::W {
                let sx = (tx * frame.width) / Self::W;
                let idx = (row_off + sx) * 4;
                let r = frame.rgba[idx] as f32;
                let g = frame.rgba[idx + 1] as f32;
                let b = frame.rgba[idx + 2] as f32;
                out[ty * Self::W + tx] = 0.299 * r + 0.587 * g + 0.114 * b;
            }
        }
        out
    }

    /// Pushes a frame into the stillness tracker and updates `moved`, `still_count`, and `last_diff`.
    pub fn push(&mut self, frame: &CapturedFrame<'_>) {
        let cur = Self::compute_thumb(frame);
        if let Some(prev) = &self.prev_thumb {
            let sum = luma_sad(&cur, prev);
            let mean_diff = sum / ((Self::W * Self::H) as f32);
            let mut max_tile_diff = 0.0_f32;
            for ty in 0..6 {
                for tx in 0..8 {
                    let mut tile_sum = 0.0_f32;
                    for dy in 0..6 {
                        let row = (ty * 6 + dy) * Self::W + tx * 8;
                        for dx in 0..8 {
                            let idx = row + dx;
                            tile_sum += (cur[idx] - prev[idx]).abs();
                        }
                    }
                    let tile_mean = tile_sum / 48.0;
                    if tile_mean > max_tile_diff {
                        max_tile_diff = tile_mean;
                    }
                }
            }
            let diff = mean_diff.max(max_tile_diff * 0.25);
            self.last_diff = diff;
            if diff > Self::MOVE_DIFF {
                self.moved = true;
                self.still_count = 0;
            } else if diff <= Self::STILL_DIFF {
                self.still_count = self.still_count.saturating_add(1);
            } else {
                self.still_count = 0;
            }
        }
        self.prev_thumb = Some(cur);
    }

    /// Moved, then held still for at least `STILL_LOOKS` probes.
    pub fn settled(&self) -> bool {
        self.moved && self.still_count >= Self::STILL_LOOKS
    }

    /// Moving right now (`last_diff > MOVE_DIFF`).
    pub fn moving(&self) -> bool {
        self.prev_thumb.is_some() && self.last_diff > Self::MOVE_DIFF
    }
}

pub fn is_torchlight_title(title: &str) -> bool {
    let t = title.trim();
    t.eq_ignore_ascii_case("Torchlight: Infinite") || t.starts_with("Torchlight: Infinite")
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod platform {
    use super::{CapturedFrame, GameWindowPlacement};
    pub struct WindowCapturer;
    impl WindowCapturer {
        pub fn new() -> Self {
            Self
        }
        pub fn new_with_buffer() -> Self {
            Self
        }
        pub fn has_game_window(&mut self) -> bool {
            false
        }
        pub fn game_window_placement(&mut self) -> Option<GameWindowPlacement> {
            None
        }
        pub fn capture(&mut self) -> Option<CapturedFrame<'_>> {
            None
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub use platform::WindowCapturer;

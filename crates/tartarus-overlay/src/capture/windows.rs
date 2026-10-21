//! Windows Win32 `PrintWindow` GDI window capturer.

use super::{CapturedFrame, GameWindowPlacement, bgra_to_rgba_in_place, is_torchlight_title};
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SRCCOPY, SelectObject,
};
use windows_sys::Win32::Storage::Xps::{PW_RENDERFULLCONTENT, PrintWindow};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClientRect, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsIconic,
    IsWindowVisible,
};

pub struct WindowCapturer {
    cached_hwnd: HWND,
    frame_buf: Vec<u8>,
}

impl WindowCapturer {
    pub fn new() -> Self {
        Self {
            cached_hwnd: std::ptr::null_mut(),
            frame_buf: Vec::new(),
        }
    }

    pub fn new_with_buffer() -> Self {
        Self {
            cached_hwnd: std::ptr::null_mut(),
            frame_buf: vec![0u8; 2560 * 1440 * 4],
        }
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
                return 1;
            }
            let len = GetWindowTextLengthW(hwnd);
            if len <= 0 || len > 128 {
                return 1;
            }
            let mut buf = vec![0u16; (len + 1) as usize];
            let read = GetWindowTextW(hwnd, buf.as_mut_ptr(), len + 1);
            if read > 0 {
                let title = String::from_utf16_lossy(&buf[..read as usize]);
                if is_torchlight_title(&title) {
                    let out = lparam as *mut HWND;
                    *out = hwnd;
                    return 0;
                }
            }
            1
        }
    }

    pub fn has_game_window(&mut self) -> bool {
        unsafe {
            let mut hwnd = self.cached_hwnd;
            if hwnd.is_null() || IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
                hwnd = std::ptr::null_mut();
                EnumWindows(Some(Self::enum_proc), (&mut hwnd as *mut HWND) as LPARAM);
                self.cached_hwnd = hwnd;
            }
            !hwnd.is_null()
        }
    }

    pub fn game_window_placement(&mut self) -> Option<GameWindowPlacement> {
        unsafe {
            if !self.has_game_window() {
                return None;
            }
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetWindowRect(self.cached_hwnd, &mut rect) == 0 {
                return None;
            }
            let width = (rect.right - rect.left).max(0) as u32;
            let height = (rect.bottom - rect.top).max(0) as u32;
            if width < 320 || height < 200 {
                return None;
            }
            Some(GameWindowPlacement {
                x: rect.left,
                y: rect.top,
                width,
                height,
                monitor_index: None,
                monitor_spatial_rank: None,
                monitor: None,
                total_monitors: 0,
            })
        }
    }

    pub fn capture(&mut self) -> Option<CapturedFrame<'_>> {
        unsafe {
            if !self.has_game_window() {
                return None;
            }
            let hwnd = self.cached_hwnd;

            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetClientRect(hwnd, &mut rect) == 0 {
                self.cached_hwnd = std::ptr::null_mut();
                return None;
            }
            let w = (rect.right - rect.left) as usize;
            let h = (rect.bottom - rect.top) as usize;
            if w < 320 || h < 200 {
                return None;
            }

            let hdc_win = GetDC(hwnd);
            if hdc_win.is_null() {
                return None;
            }
            let hdc_mem = CreateCompatibleDC(hdc_win);
            let hbm = CreateCompatibleBitmap(hdc_win, w as i32, h as i32);
            let old = SelectObject(hdc_mem, hbm);

            let printed = PrintWindow(hwnd, hdc_mem, PW_RENDERFULLCONTENT);
            if printed == 0 {
                let _ = BitBlt(hdc_mem, 0, 0, w as i32, h as i32, hdc_win, 0, 0, SRCCOPY);
            }

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w as i32,
                    biHeight: -(h as i32), // top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [std::mem::zeroed()],
            };

            let need = w * h * 4;
            if self.frame_buf.len() < need {
                self.frame_buf.resize(need, 0);
            }
            let lines = GetDIBits(
                hdc_mem,
                hbm,
                0,
                h as u32,
                self.frame_buf.as_mut_ptr() as *mut _,
                &mut bmi,
                DIB_RGB_COLORS,
            );

            SelectObject(hdc_mem, old);
            DeleteObject(hbm);
            DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_win);

            if lines == 0 {
                return None;
            }

            bgra_to_rgba_in_place(&mut self.frame_buf[..need]);

            Some(CapturedFrame {
                width: w,
                height: h,
                rgba: &self.frame_buf[..need],
            })
        }
    }
}

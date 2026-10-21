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

const DEFAULT_CAP_BYTES: usize = 2560 * 1440 * 4;
const WIN_WIDTH_MIN: usize = 320;
const WIN_HEIGHT_MIN: usize = 200;

const _: () = {
    assert!(DEFAULT_CAP_BYTES == 14_745_600);
    assert!(WIN_WIDTH_MIN > 0 && WIN_HEIGHT_MIN > 0);
};

pub struct WindowCapturer {
    cached_hwnd: HWND,
    frame_buf: Vec<u8>,
}

impl WindowCapturer {
    pub fn new() -> Self {
        let cap = Self {
            cached_hwnd: std::ptr::null_mut(),
            frame_buf: Vec::new(),
        };
        assert!(cap.cached_hwnd.is_null());
        assert!(cap.frame_buf.is_empty());
        cap
    }

    pub fn new_with_buffer() -> Self {
        let cap = Self {
            cached_hwnd: std::ptr::null_mut(),
            frame_buf: vec![0u8; DEFAULT_CAP_BYTES],
        };
        assert!(cap.cached_hwnd.is_null());
        assert_eq!(cap.frame_buf.len(), DEFAULT_CAP_BYTES);
        cap
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            assert!(!hwnd.is_null());
            assert_ne!(lparam, 0);
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
                assert!(hwnd.is_null());
                EnumWindows(Some(Self::enum_proc), (&mut hwnd as *mut HWND) as LPARAM);
                self.cached_hwnd = hwnd;
            }
            let found = !hwnd.is_null();
            assert_eq!(found, !self.cached_hwnd.is_null());
            found
        }
    }

    pub fn game_window_placement(&mut self) -> Option<GameWindowPlacement> {
        unsafe {
            if !self.has_game_window() {
                return None;
            }
            assert!(!self.cached_hwnd.is_null());
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
            if (width as usize) < WIN_WIDTH_MIN || (height as usize) < WIN_HEIGHT_MIN {
                return None;
            }
            assert!(width >= WIN_WIDTH_MIN as u32 && height >= WIN_HEIGHT_MIN as u32);
            Some(GameWindowPlacement {
                x: rect.left,
                y: rect.top,
                width,
                height,
                monitor_index: None,
                monitor_spatial_rank: None,
                monitor: None,
                monitors_total: 0,
            })
        }
    }

    unsafe fn make_top_down_bmi(w: usize, h: usize) -> BITMAPINFO {
        assert!(w >= WIN_WIDTH_MIN && h >= WIN_HEIGHT_MIN);
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w as i32,
                biHeight: -(h as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [unsafe { std::mem::zeroed() }],
        };
        assert_eq!(bmi.bmiHeader.biBitCount, 32);
        bmi
    }

    unsafe fn read_window_bgra(&mut self, hwnd: HWND, w: usize, h: usize) -> bool {
        unsafe {
            assert!(!hwnd.is_null());
            assert!(w >= WIN_WIDTH_MIN && h >= WIN_HEIGHT_MIN);
            let hdc_win = GetDC(hwnd);
            if hdc_win.is_null() {
                return false;
            }
            let hdc_mem = CreateCompatibleDC(hdc_win);
            let hbm = CreateCompatibleBitmap(hdc_win, w as i32, h as i32);
            let old = SelectObject(hdc_mem, hbm);

            let printed = PrintWindow(hwnd, hdc_mem, PW_RENDERFULLCONTENT);
            if printed == 0 {
                let _ = BitBlt(hdc_mem, 0, 0, w as i32, h as i32, hdc_win, 0, 0, SRCCOPY);
            }

            let mut bmi = Self::make_top_down_bmi(w, h);
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

            lines != 0
        }
    }

    pub fn capture(&mut self) -> Option<CapturedFrame<'_>> {
        unsafe {
            if !self.has_game_window() {
                return None;
            }
            let hwnd = self.cached_hwnd;
            assert!(!hwnd.is_null());

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
            if w < WIN_WIDTH_MIN || h < WIN_HEIGHT_MIN {
                return None;
            }

            if !self.read_window_bgra(hwnd, w, h) {
                return None;
            }

            let need = w * h * 4;
            assert!(self.frame_buf.len() >= need);
            bgra_to_rgba_in_place(&mut self.frame_buf[..need]);

            Some(CapturedFrame {
                width: w,
                height: h,
                rgba: &self.frame_buf[..need],
            })
        }
    }
}

//! Linux X11 + MIT-SHM composite window capturer.

use super::{
    CapturedFrame, GameWindowPlacement, MonitorRect, bgra_to_rgba_in_place, is_torchlight_title,
};
use x11rb::connection::Connection;
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::protocol::shm::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{ConnectionExt, ImageFormat, Window};

const DEFAULT_CAP_BYTES: usize = 2560 * 1440 * 4;

struct ShmSegment {
    seg: shm::Seg,
    addr: *mut u8,
    cap: usize,
}

// Safety: ShmSegment is owned exclusively by WindowCapturer on a single thread.
unsafe impl Send for ShmSegment {}

impl ShmSegment {
    fn new(conn: &x11rb::rust_connection::RustConnection, cap: usize) -> Option<Self> {
        conn.shm_query_version().ok()?.reply().ok()?;
        let shmid = unsafe { libc::shmget(libc::IPC_PRIVATE, cap, libc::IPC_CREAT | 0o600) };
        if shmid < 0 {
            return None;
        }
        let addr = unsafe { libc::shmat(shmid, std::ptr::null(), 0).cast::<u8>() };
        if addr.is_null() || addr == (usize::MAX as *mut u8) {
            unsafe {
                libc::shmctl(shmid, libc::IPC_RMID, std::ptr::null_mut());
            }
            return None;
        }
        let seg = match conn.generate_id() {
            Ok(s) => s,
            Err(_) => {
                unsafe {
                    libc::shmdt(addr.cast::<libc::c_void>());
                    libc::shmctl(shmid, libc::IPC_RMID, std::ptr::null_mut());
                }
                return None;
            }
        };
        let attached = conn
            .shm_attach(seg, shmid as u32, false)
            .ok()
            .and_then(|c| c.check().ok())
            .is_some();
        unsafe {
            libc::shmctl(shmid, libc::IPC_RMID, std::ptr::null_mut());
        }
        if !attached {
            unsafe {
                libc::shmdt(addr.cast::<libc::c_void>());
            }
            return None;
        }
        let mut off = 0usize;
        while off < cap {
            unsafe {
                std::ptr::write_volatile(addr.add(off), 0);
            }
            off += 4096;
        }
        Some(Self { seg, addr, cap })
    }

    #[inline]
    fn as_slice(&self, len: usize) -> &[u8] {
        assert!(len <= self.cap);
        unsafe { std::slice::from_raw_parts(self.addr, len) }
    }

    #[inline]
    fn as_mut_slice(&mut self, len: usize) -> &mut [u8] {
        assert!(len <= self.cap);
        unsafe { std::slice::from_raw_parts_mut(self.addr, len) }
    }

    fn destroy(&mut self, conn: Option<&x11rb::rust_connection::RustConnection>) {
        if let Some(c) = conn {
            let _ = c.shm_detach(self.seg);
        }
        if !self.addr.is_null() {
            unsafe {
                libc::shmdt(self.addr.cast::<libc::c_void>());
            }
            self.addr = std::ptr::null_mut();
        }
    }
}

pub struct WindowCapturer {
    conn: Option<(x11rb::rust_connection::RustConnection, usize)>,
    cached_win: Option<Window>,
    cached_size: Option<(u16, u16)>,
    shm: Option<ShmSegment>,
    shm_disabled: bool,
    fallback_buf: Vec<u8>,
}

impl Drop for WindowCapturer {
    fn drop(&mut self) {
        if let Some(mut shm) = self.shm.take() {
            let conn_ref = self.conn.as_ref().map(|(c, _)| c);
            shm.destroy(conn_ref);
        }
    }
}

impl WindowCapturer {
    pub fn new() -> Self {
        let conn = x11rb::connect(None).ok();
        Self {
            conn,
            cached_win: None,
            cached_size: None,
            shm: None,
            shm_disabled: false,
            fallback_buf: Vec::new(),
        }
    }

    pub fn new_with_buffer() -> Self {
        let mut cap = Self::new();
        let _ = cap.has_game_window();
        let init_bytes = cap
            .cached_size
            .map(|(w, h)| (w as usize) * (h as usize) * 4)
            .unwrap_or(DEFAULT_CAP_BYTES)
            .max(DEFAULT_CAP_BYTES);
        if let Some((conn, _)) = cap.conn.as_ref() {
            cap.shm = ShmSegment::new(conn, init_bytes);
        }
        let _ = cap.capture();
        cap
    }

    fn viewable_window_size(
        conn: &x11rb::rust_connection::RustConnection,
        win: Window,
    ) -> Option<(u16, u16)> {
        let attr_cookie = conn.get_window_attributes(win).ok()?;
        let geom_cookie = conn.get_geometry(win).ok()?;
        let attr = attr_cookie.reply().ok()?;
        if attr.map_state != x11rb::protocol::xproto::MapState::VIEWABLE {
            return None;
        }
        let geom = geom_cookie.reply().ok()?;
        if geom.width >= 320 && geom.height >= 200 {
            Some((geom.width, geom.height))
        } else {
            None
        }
    }

    pub fn has_game_window(&mut self) -> bool {
        if self.conn.is_none() {
            self.conn = x11rb::connect(None).ok();
        }
        let Some((conn, screen_num)) = self.conn.as_ref() else {
            return false;
        };
        let root = conn.setup().roots[*screen_num].root;
        if let Some(win) = self.cached_win {
            if let Some((w, h)) = Self::viewable_window_size(conn, win) {
                self.cached_size = Some((w, h));
                return true;
            }
            self.cached_win = None;
            self.cached_size = None;
        }
        if let Some((win, w, h)) = Self::find_game_window(conn, root) {
            self.cached_win = Some(win);
            self.cached_size = Some((w, h));
            true
        } else {
            false
        }
    }

    fn find_game_window(
        conn: &x11rb::rust_connection::RustConnection,
        root: Window,
    ) -> Option<(Window, u16, u16)> {
        let c_net = conn.intern_atom(false, b"_NET_WM_NAME").ok()?;
        let c_utf8 = conn.intern_atom(false, b"UTF8_STRING").ok()?;
        let c_wm = conn.intern_atom(false, b"WM_NAME").ok()?;
        let net_wm_name = c_net.reply().ok()?.atom;
        let utf8_string = c_utf8.reply().ok()?.atom;
        let wm_name = c_wm.reply().ok()?.atom;

        let mut best: Option<(Window, u16, u16)> = None;
        let mut stack = vec![root];
        while let Some(win) = stack.pop() {
            let mut matched_title = false;
            if let Ok(cookie) = conn.get_property(false, win, net_wm_name, utf8_string, 0, 256)
                && let Ok(reply) = cookie.reply()
                && !reply.value.is_empty()
                && let Ok(title) = std::str::from_utf8(&reply.value)
                && is_torchlight_title(title)
            {
                matched_title = true;
            }
            if !matched_title
                && let Ok(cookie) = conn.get_property(false, win, wm_name, x11rb::NONE, 0, 256)
                && let Ok(reply) = cookie.reply()
                && !reply.value.is_empty()
                && let Ok(title) = std::str::from_utf8(&reply.value)
                && is_torchlight_title(title)
            {
                matched_title = true;
            }
            if matched_title && let Some((w, h)) = Self::viewable_window_size(conn, win) {
                let area = (w as u32) * (h as u32);
                let best_area = best
                    .map(|(_, bw, bh)| (bw as u32) * (bh as u32))
                    .unwrap_or(0);
                if area >= best_area {
                    best = Some((win, w, h));
                }
            }
            if let Ok(tree_cookie) = conn.query_tree(win)
                && let Ok(tree) = tree_cookie.reply()
            {
                for child in tree.children {
                    stack.push(child);
                }
            }
        }
        best
    }

    pub fn game_window_placement(&mut self) -> Option<GameWindowPlacement> {
        if !self.has_game_window() {
            return None;
        }
        let (conn, screen_num) = self.conn.as_ref()?;
        let root = conn.setup().roots[*screen_num].root;
        let win = self.cached_win?;

        let geom_cookie = conn.get_geometry(win).ok()?;
        let trans_cookie = conn.translate_coordinates(win, root, 0, 0).ok()?;
        let mon_cookie = conn.randr_get_monitors(root, true).ok();

        let geom = geom_cookie.reply().ok()?;
        let trans = trans_cookie.reply().ok()?;
        self.cached_size = Some((geom.width, geom.height));

        let wx = trans.dst_x as i32;
        let wy = trans.dst_y as i32;
        let ww = geom.width as u32;
        let wh = geom.height as u32;
        let wcx = wx + (ww as i32) / 2;
        let wcy = wy + (wh as i32) / 2;

        let mut monitors = Vec::new();
        if let Some(cookie) = mon_cookie
            && let Ok(reply) = cookie.reply()
        {
            for m in reply.monitors {
                if m.width > 0 && m.height > 0 {
                    monitors.push(MonitorRect {
                        x: m.x as i32,
                        y: m.y as i32,
                        width: m.width as u32,
                        height: m.height as u32,
                    });
                }
            }
        }

        let mut monitor_index = None;
        for (idx, m) in monitors.iter().enumerate() {
            if wcx >= m.x
                && wcx < m.x + (m.width as i32)
                && wcy >= m.y
                && wcy < m.y + (m.height as i32)
            {
                monitor_index = Some(idx);
                break;
            }
        }
        if monitor_index.is_none() && !monitors.is_empty() {
            monitor_index = monitors
                .iter()
                .enumerate()
                .min_by_key(|(_, m)| {
                    let mcx = m.x + (m.width as i32) / 2;
                    let mcy = m.y + (m.height as i32) / 2;
                    let dx = (wcx - mcx) as i64;
                    let dy = (wcy - mcy) as i64;
                    dx * dx + dy * dy
                })
                .map(|(idx, _)| idx);
        }

        let monitor_spatial_rank = monitor_index.map(|idx| {
            let mut order: Vec<usize> = (0..monitors.len()).collect();
            order.sort_by_key(|&i| (monitors[i].x, monitors[i].y));
            order.iter().position(|&i| i == idx).unwrap_or(idx)
        });

        let monitor = monitor_index.map(|idx| monitors[idx]);
        Some(GameWindowPlacement {
            x: wx,
            y: wy,
            width: ww,
            height: wh,
            monitor_index,
            monitor_spatial_rank,
            monitor,
            total_monitors: monitors.len(),
        })
    }

    pub fn capture(&mut self) -> Option<CapturedFrame<'_>> {
        if self.conn.is_none() {
            self.conn = x11rb::connect(None).ok();
        }
        let (conn, screen_num) = self.conn.as_ref()?;
        let root = conn.setup().roots[*screen_num].root;

        let mut target: Option<(Window, u16, u16)> = None;
        if let Some(win) = self.cached_win {
            if let Some((cw, ch)) = self.cached_size {
                if cw >= 320 && ch >= 200 {
                    target = Some((win, cw, ch));
                }
            } else if let Some((w, h)) = Self::viewable_window_size(conn, win) {
                self.cached_size = Some((w, h));
                target = Some((win, w, h));
            }
            if target.is_none() {
                self.cached_win = None;
                self.cached_size = None;
            }
        }
        if target.is_none() {
            let (win, w, h) = Self::find_game_window(conn, root)?;
            self.cached_win = Some(win);
            self.cached_size = Some((w, h));
            target = Some((win, w, h));
        }

        let (mut win, mut width, mut height) = target?;
        if !self.capture_into_buffers(win, width, height) {
            let rediscovered = self.conn.as_ref().and_then(|(conn, screen_num)| {
                if let Some((w, h)) = Self::viewable_window_size(conn, win) {
                    Some((win, w, h))
                } else {
                    let root = conn.setup().roots[*screen_num].root;
                    Self::find_game_window(conn, root)
                }
            });
            if let Some((new_win, new_w, new_h)) = rediscovered {
                win = new_win;
                width = new_w;
                height = new_h;
                self.cached_win = Some(win);
                self.cached_size = Some((width, height));
                if self.capture_into_buffers(win, width, height) {
                    let w = width as usize;
                    let h = height as usize;
                    let need = w * h * 4;
                    let slice = if let Some(shm) = self.shm.as_ref() {
                        shm.as_slice(need)
                    } else {
                        &self.fallback_buf[..need]
                    };
                    return Some(CapturedFrame {
                        width: w,
                        height: h,
                        rgba: slice,
                    });
                }
            }
            self.cached_win = None;
            self.cached_size = None;
            return None;
        }

        let w = width as usize;
        let h = height as usize;
        let need = w * h * 4;
        let slice = if let Some(shm) = self.shm.as_ref() {
            shm.as_slice(need)
        } else {
            &self.fallback_buf[..need]
        };
        Some(CapturedFrame {
            width: w,
            height: h,
            rgba: slice,
        })
    }

    fn capture_into_buffers(&mut self, win: Window, width: u16, height: u16) -> bool {
        let w = width as usize;
        let h = height as usize;
        let need = w * h * 4;
        let Some((conn, _)) = self.conn.as_ref() else {
            return false;
        };

        if !self.shm_disabled && self.shm.as_ref().is_none_or(|s| s.cap < need) {
            if let Some(mut old) = self.shm.take() {
                old.destroy(Some(conn));
            }
            self.shm = ShmSegment::new(conn, need.max(DEFAULT_CAP_BYTES));
            if self.shm.is_none() {
                self.shm_disabled = true;
            }
        }

        if let Some(shm) = self.shm.as_mut() {
            let ok = conn
                .shm_get_image(
                    win,
                    0,
                    0,
                    width,
                    height,
                    !0,
                    ImageFormat::Z_PIXMAP.into(),
                    shm.seg,
                    0,
                )
                .ok()
                .and_then(|c| c.reply().ok())
                .is_some();
            if ok {
                bgra_to_rgba_in_place(shm.as_mut_slice(need));
                return true;
            }
            if let Some(mut old) = self.shm.take() {
                old.destroy(Some(conn));
            }
            self.shm_disabled = true;
        }

        let Ok(cookie) = conn.get_image(ImageFormat::Z_PIXMAP, win, 0, 0, width, height, !0) else {
            return false;
        };
        let Ok(img) = cookie.reply() else {
            return false;
        };
        if img.data.len() < need {
            return false;
        }
        self.fallback_buf.resize(need, 0);
        self.fallback_buf.copy_from_slice(&img.data[..need]);
        bgra_to_rgba_in_place(&mut self.fallback_buf[..need]);
        true
    }
}

impl Default for WindowCapturer {
    fn default() -> Self {
        Self::new()
    }
}

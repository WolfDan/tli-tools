use std::sync::mpsc::{self, Receiver, Sender};

#[cfg(target_os = "windows")]
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

use crate::state::UiToWorker;

const POLL_COMMANDS_MAX: usize = 256;
const DEFAULT_KEYCODE_O: u8 = 32;
const DEFAULT_KEYCODE_A: u8 = 38;
const DEFAULT_KEYCODE_P: u8 = 33;

const _: () = {
    assert!(POLL_COMMANDS_MAX > 0);
    assert!(DEFAULT_KEYCODE_O != DEFAULT_KEYCODE_A);
    assert!(DEFAULT_KEYCODE_A != DEFAULT_KEYCODE_P);
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlCommand {
    ToggleOverlay,
    ToggleInspector,
    CaptureNow,
    ToggleFold,
    Quit,
}

pub struct ControlHub {
    pub rx: Receiver<ControlCommand>,
    pub tx: Sender<ControlCommand>,
    #[cfg(target_os = "windows")]
    worker_tx: Sender<UiToWorker>,
    #[cfg(target_os = "windows")]
    _hotkey_manager: Option<GlobalHotKeyManager>,
    #[cfg(target_os = "windows")]
    hotkey_ids: Option<(u32, u32, u32)>,
}

impl ControlHub {
    pub fn init(worker_tx: Sender<UiToWorker>) -> Self {
        assert!(std::mem::size_of_val(&worker_tx) > 0);
        let (tx, rx) = mpsc::channel();

        // 1a. Linux: Spawn pure-Rust X11 hotkey listener (non-fatal if X_GrabKey
        //     returns BadAccess) and StatusNotifierItem system tray via ksni.
        #[cfg(target_os = "linux")]
        {
            let hk_tx = tx.clone();
            let hk_worker_tx = worker_tx.clone();
            std::thread::Builder::new()
                .name("tartarus-hotkeys".into())
                .spawn(move || {
                    run_linux_x11_hotkeys(hk_tx, hk_worker_tx);
                })
                .ok();

            let tray_tx = tx.clone();
            let tray_worker_tx = worker_tx;
            let service = ksni::TrayService::new(LinuxTray {
                tx: tray_tx,
                worker_tx: tray_worker_tx,
            });
            service.spawn();
        }

        // 1b. Windows: Register global hotkeys via global-hotkey
        #[cfg(target_os = "windows")]
        let (manager_opt, ids_opt) = {
            let mut m_opt = None;
            let mut i_opt = None;
            if let Ok(manager) = GlobalHotKeyManager::new() {
                let mods = Modifiers::CONTROL | Modifiers::ALT;
                let hk_o = HotKey::new(Some(mods), Code::KeyO);
                let hk_a = HotKey::new(Some(mods), Code::KeyA);
                let hk_p = HotKey::new(Some(mods), Code::KeyP);
                let _ = manager.register(hk_o);
                let _ = manager.register(hk_a);
                let _ = manager.register(hk_p);
                i_opt = Some((hk_o.id(), hk_a.id(), hk_p.id()));
                m_opt = Some(manager);
            }
            (m_opt, i_opt)
        };

        let hub = Self {
            rx,
            tx,
            #[cfg(target_os = "windows")]
            worker_tx,
            #[cfg(target_os = "windows")]
            _hotkey_manager: manager_opt,
            #[cfg(target_os = "windows")]
            hotkey_ids: ids_opt,
        };
        assert!(std::mem::size_of_val(&hub) > 0);
        hub
    }

    pub fn poll(&self) -> Vec<ControlCommand> {
        let mut out = Vec::new();
        assert!(out.is_empty());
        #[cfg(target_os = "windows")]
        if let Some((id_o, id_a, id_p)) = self.hotkey_ids {
            let mut hk_iters = 0usize;
            while hk_iters < POLL_COMMANDS_MAX {
                let Ok(ev) = GlobalHotKeyEvent::receiver().try_recv() else {
                    break;
                };
                hk_iters += 1;
                if ev.state == HotKeyState::Pressed {
                    if ev.id == id_o {
                        out.push(ControlCommand::ToggleOverlay);
                    } else if ev.id == id_a {
                        out.push(ControlCommand::ToggleInspector);
                    } else if ev.id == id_p {
                        let _ = self.worker_tx.send(UiToWorker::CaptureNow);
                        out.push(ControlCommand::CaptureNow);
                    }
                }
            }
        }
        let mut cmd_iters = 0usize;
        while cmd_iters < POLL_COMMANDS_MAX {
            let Ok(cmd) = self.rx.try_recv() else {
                break;
            };
            cmd_iters += 1;
            out.push(cmd);
        }
        assert!(out.len() <= POLL_COMMANDS_MAX * 2);
        out
    }
}

#[cfg(target_os = "linux")]
fn resolve_x11_keycodes(conn: &impl x11rb::connection::Connection) -> [u8; 3] {
    use x11rb::protocol::xproto::ConnectionExt;

    let setup = conn.setup();
    let mut code_o: u8 = DEFAULT_KEYCODE_O;
    let mut code_a: u8 = DEFAULT_KEYCODE_A;
    let mut code_p: u8 = DEFAULT_KEYCODE_P;

    let kc_min = setup.min_keycode;
    let kc_max = setup.max_keycode;
    if kc_max >= kc_min
        && let Ok(cookie) = conn.get_keyboard_mapping(kc_min, kc_max - kc_min + 1)
        && let Ok(reply) = cookie.reply()
    {
        let per = reply.keysyms_per_keycode as usize;
        if per > 0 {
            for kc in kc_min..=kc_max {
                let start = (kc - kc_min) as usize * per;
                let end = (start + per).min(reply.keysyms.len());
                for &sym in &reply.keysyms[start..end] {
                    match sym {
                        0x006f | 0x004f => code_o = kc,
                        0x0061 | 0x0041 => code_a = kc,
                        0x0070 | 0x0050 => code_p = kc,
                        _ => {}
                    }
                }
            }
        }
    }
    assert!(code_o > 0 && code_a > 0 && code_p > 0);
    assert_ne!(code_o, code_a);
    [code_o, code_a, code_p]
}

#[cfg(target_os = "linux")]
fn grab_x11_hotkeys(conn: &impl x11rb::connection::Connection, root: u32, codes: [u8; 3]) -> bool {
    use x11rb::protocol::xproto::{ConnectionExt, GrabMode, ModMask};

    assert!(root > 0);
    assert_eq!(codes.len(), 3);
    let base_mods = u16::from(ModMask::CONTROL | ModMask::M1);
    let mod_variants = [
        base_mods,
        base_mods | u16::from(ModMask::M2),
        base_mods | u16::from(ModMask::LOCK),
        base_mods | u16::from(ModMask::M2 | ModMask::LOCK),
    ];

    let mut any_grabbed = false;
    for kc in codes {
        for &m in &mod_variants {
            if let Ok(cookie) = conn.grab_key(
                false,
                root,
                ModMask::from(m),
                kc,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            ) && cookie.check().is_ok()
            {
                any_grabbed = true;
            }
        }
    }
    any_grabbed
}

#[cfg(target_os = "linux")]
fn run_linux_x11_hotkeys(tx: Sender<ControlCommand>, worker_tx: Sender<UiToWorker>) {
    use x11rb::connection::Connection;
    use x11rb::protocol::Event;

    let Ok((conn, screen_num)) = x11rb::connect(None) else {
        return;
    };
    let setup = conn.setup();
    let Some(screen) = setup.roots.get(screen_num) else {
        return;
    };
    let root = screen.root;
    assert!(root > 0);

    let [code_o, code_a, code_p] = resolve_x11_keycodes(&conn);
    assert!(code_o > 0 && code_a > 0 && code_p > 0);
    if !grab_x11_hotkeys(&conn, root, [code_o, code_a, code_p]) {
        return;
    }

    while let Ok(event) = conn.wait_for_event() {
        if let Event::KeyPress(kp) = event {
            if kp.detail == code_o {
                let _ = tx.send(ControlCommand::ToggleOverlay);
            } else if kp.detail == code_a {
                let _ = tx.send(ControlCommand::ToggleInspector);
            } else if kp.detail == code_p {
                let _ = worker_tx.send(UiToWorker::CaptureNow);
                let _ = tx.send(ControlCommand::CaptureNow);
            }
        }
    }
}

#[cfg(target_os = "linux")]
struct LinuxTray {
    tx: Sender<ControlCommand>,
    worker_tx: Sender<UiToWorker>,
}

#[cfg(target_os = "linux")]
impl ksni::Tray for LinuxTray {
    fn id(&self) -> String {
        let id = String::from("tartarus-overlay");
        assert!(!id.is_empty());
        assert!(id.starts_with("tartarus"));
        id
    }

    fn title(&self) -> String {
        let title = String::from("Tartarus Architect Overlay");
        assert!(!title.is_empty());
        assert!(title.starts_with("Tartarus"));
        title
    }

    fn icon_name(&self) -> String {
        let icon = String::from("applications-games");
        assert!(!icon.is_empty());
        assert!(icon.contains('-'));
        icon
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        assert!(std::mem::size_of_val(&self.tx) > 0);
        assert!(std::mem::size_of_val(&self.worker_tx) > 0);
        let _ = self.tx.send(ControlCommand::ToggleOverlay);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let items: Vec<ksni::MenuItem<Self>> = vec![
            StandardItem {
                label: "Show / Hide Overlay (Ctrl+Alt+O)".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(ControlCommand::ToggleOverlay);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "16-Cell Board Inspector (Ctrl+Alt+A)".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(ControlCommand::ToggleInspector);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Capture Picture Now (Ctrl+Alt+P)".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.worker_tx.send(UiToWorker::CaptureNow);
                    let _ = this.tx.send(ControlCommand::CaptureNow);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Fold / Unfold Overlay".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(ControlCommand::ToggleFold);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit Tartarus Overlay".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(ControlCommand::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ];
        assert_eq!(items.len(), 6);
        assert!(!items.is_empty());
        items
    }
}

use std::sync::mpsc::{self, Receiver, Sender};

#[cfg(target_os = "windows")]
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

use crate::state::UiToWorker;

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
        let (tx, rx) = mpsc::channel();

        // 1a. Linux: Spawn pure-Rust X11 hotkey listener (non-fatal if X_GrabKey returns BadAccess)
        //     and StatusNotifierItem system tray via ksni
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

        Self {
            rx,
            tx,
            #[cfg(target_os = "windows")]
            worker_tx,
            #[cfg(target_os = "windows")]
            _hotkey_manager: manager_opt,
            #[cfg(target_os = "windows")]
            hotkey_ids: ids_opt,
        }
    }

    pub fn poll(&self) -> Vec<ControlCommand> {
        let mut out = Vec::new();
        #[cfg(target_os = "windows")]
        if let Some((id_o, id_a, id_p)) = self.hotkey_ids {
            while let Ok(ev) = GlobalHotKeyEvent::receiver().try_recv() {
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
        while let Ok(cmd) = self.rx.try_recv() {
            out.push(cmd);
        }
        out
    }
}

#[cfg(target_os = "linux")]
fn run_linux_x11_hotkeys(tx: Sender<ControlCommand>, worker_tx: Sender<UiToWorker>) {
    use x11rb::connection::Connection;
    use x11rb::protocol::Event;
    use x11rb::protocol::xproto::{ConnectionExt, GrabMode, ModMask};

    let Ok((conn, screen_num)) = x11rb::connect(None) else {
        return;
    };
    let setup = conn.setup();
    let Some(screen) = setup.roots.get(screen_num) else {
        return;
    };
    let root = screen.root;

    // Default evdev keycodes on Linux X11: 'o' = 32, 'a' = 38, 'p' = 33
    let mut code_o: u8 = 32;
    let mut code_a: u8 = 38;
    let mut code_p: u8 = 33;

    let min_kc = setup.min_keycode;
    let max_kc = setup.max_keycode;
    if max_kc >= min_kc {
        if let Ok(cookie) = conn.get_keyboard_mapping(min_kc, max_kc - min_kc + 1) {
            if let Ok(reply) = cookie.reply() {
                let per = reply.keysyms_per_keycode as usize;
                if per > 0 {
                    for kc in min_kc..=max_kc {
                        let start = (kc - min_kc) as usize * per;
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
        }
    }

    let base_mods = u16::from(ModMask::CONTROL | ModMask::M1);
    let mod_variants = [
        base_mods,
        base_mods | u16::from(ModMask::M2),
        base_mods | u16::from(ModMask::LOCK),
        base_mods | u16::from(ModMask::M2 | ModMask::LOCK),
    ];

    let mut any_grabbed = false;
    for &kc in &[code_o, code_a, code_p] {
        for &m in &mod_variants {
            if let Ok(cookie) = conn.grab_key(
                false,
                root,
                ModMask::from(m),
                kc,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            ) {
                if cookie.check().is_ok() {
                    any_grabbed = true;
                }
            }
        }
    }

    if !any_grabbed {
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
        "tartarus-overlay".into()
    }

    fn title(&self) -> String {
        "Tartarus Architect Overlay".into()
    }

    fn icon_name(&self) -> String {
        "applications-games".into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(ControlCommand::ToggleOverlay);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
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
        ]
    }
}

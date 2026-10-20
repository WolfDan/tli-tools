use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use gpui::DisplayId;
use tartarus_core::{AdviceSnapshot, BoardModel, RunPhase};
use tartarus_scan::ScanProfile;

use crate::capture::GameWindowPlacement;
use crate::tray::ControlHub;

pub fn read_process_rss_mb() -> f64 {
    use std::io::Read;
    if let Ok(mut file) = std::fs::File::open("/proc/self/statm") {
        let mut buf = [0u8; 128];
        if let Ok(n) = file.read(&mut buf) {
            if let Ok(statm) = std::str::from_utf8(&buf[..n]) {
                let mut parts = statm.split_whitespace();
                let _size = parts.next();
                if let (Some(res_str), Some(shared_str)) = (parts.next(), parts.next()) {
                    if let (Ok(resident), Ok(shared)) =
                        (res_str.parse::<u64>(), shared_str.parse::<u64>())
                    {
                        let private_pages = resident.saturating_sub(shared);
                        return (private_pages as f64) * (4096.0 / (1024.0 * 1024.0));
                    }
                }
            }
        }
    }
    0.0
}

#[derive(Clone, Copy, Debug)]
pub enum UiToWorker {
    CaptureNow,
    ConfirmTools,
    AckTreasureNow,
    UpdateModelAndRecheck(BoardModel),
    LoadExample,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum PanelLayout {
    /// Standard compact recommendation card.
    #[default]
    Compact,
    /// Expanded with the 4x4 board inspector (and optional tools drawer).
    Inspector { tools_drawer: bool },
    /// Manually folded by the user to a single header bar.
    FoldedManual,
    /// Automatically folded during a Riot cutscene; remembers which layout to restore when Riot ends.
    FoldedForRiot {
        restore_inspector: bool,
        restore_tools: bool,
    },
}

impl PanelLayout {
    #[inline]
    pub const fn is_folded(self) -> bool {
        matches!(self, Self::FoldedManual | Self::FoldedForRiot { .. })
    }

    #[inline]
    pub const fn is_inspector_open(self) -> bool {
        matches!(self, Self::Inspector { .. })
    }

    #[inline]
    pub const fn is_tools_drawer_open(self) -> bool {
        matches!(self, Self::Inspector { tools_drawer: true })
    }

    pub fn toggle_fold(&mut self) {
        *self = if self.is_folded() {
            Self::Compact
        } else {
            Self::FoldedManual
        };
    }

    pub fn toggle_inspector(&mut self) {
        *self = match *self {
            Self::Inspector { .. } => Self::Compact,
            _ => Self::Inspector {
                tools_drawer: false,
            },
        };
    }

    pub fn open_inspector(&mut self, with_tools_drawer: bool) {
        let cur_tools = match *self {
            Self::Inspector { tools_drawer } => tools_drawer,
            _ => false,
        };
        *self = Self::Inspector {
            tools_drawer: cur_tools || with_tools_drawer,
        };
    }

    pub fn toggle_tools_drawer(&mut self) {
        if let Self::Inspector { tools_drawer } = self {
            *tools_drawer = !*tools_drawer;
        } else {
            *self = Self::Inspector { tools_drawer: true };
        }
    }

    pub fn apply_riot(&mut self, riot_on: bool) -> bool {
        if riot_on {
            match *self {
                Self::Compact => {
                    *self = Self::FoldedForRiot {
                        restore_inspector: false,
                        restore_tools: false,
                    };
                    true
                }
                Self::Inspector { tools_drawer } => {
                    *self = Self::FoldedForRiot {
                        restore_inspector: true,
                        restore_tools: tools_drawer,
                    };
                    true
                }
                Self::FoldedManual | Self::FoldedForRiot { .. } => false,
            }
        } else if let Self::FoldedForRiot {
            restore_inspector,
            restore_tools,
        } = *self
        {
            *self = if restore_inspector {
                Self::Inspector {
                    tools_drawer: restore_tools,
                }
            } else {
                Self::Compact
            };
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DisplayTarget {
    Auto(Option<DisplayId>),
    Manual(DisplayId),
}

impl DisplayTarget {
    #[inline]
    pub const fn id(self) -> Option<DisplayId> {
        match self {
            Self::Auto(id) => id,
            Self::Manual(id) => Some(id),
        }
    }

    #[inline]
    pub fn ensure_default(&mut self, fallback: Option<DisplayId>) {
        if matches!(self, Self::Auto(None)) {
            *self = Self::Auto(fallback);
        }
    }
}

#[derive(Clone, Debug)]
pub struct OverlaySnapshot {
    pub model: BoardModel,
    pub advice: AdviceSnapshot,
    pub phase: RunPhase,
    pub riot: bool,
    pub notes: Vec<String>,
    pub game_connected: bool,
    pub log_connected: bool,
    pub cap_ms: f64,
    pub scan_ms: f64,
    pub scan_profile: ScanProfile,
    pub solve_ms: f64,
    pub rss_mb: f64,
    pub status_text: String,
}

pub fn initial_snapshot() -> OverlaySnapshot {
    OverlaySnapshot {
        model: BoardModel::blank(),
        advice: AdviceSnapshot::default(),
        phase: RunPhase::WaitingForRun,
        riot: false,
        notes: Vec::new(),
        game_connected: false,
        log_connected: false,
        cap_ms: 0.0,
        scan_ms: 0.0,
        scan_profile: ScanProfile::default(),
        solve_ms: 0.0,
        rss_mb: read_process_rss_mb(),
        status_text: "Waiting for the next move".into(),
    }
}

#[derive(Clone, Debug)]
pub enum WorkerToUi {
    Snapshot(OverlaySnapshot),
    RiotState(bool),
    GamePlacement(GameWindowPlacement),
}

pub struct SharedWorkerChannels {
    pub worker_rx: Receiver<WorkerToUi>,
    pub control_hub: ControlHub,
}

#[derive(Clone)]
pub struct OverlayWindowState {
    pub worker_tx: Sender<UiToWorker>,
    pub shared_rx: Arc<Mutex<SharedWorkerChannels>>,
    pub snap: OverlaySnapshot,
    pub visible: bool,
    pub layout: PanelLayout,
    pub selected_cell: u8,
    pub display_target: DisplayTarget,
    pub is_layer_shell: bool,
    pub margin_top: f32,
    pub margin_left: f32,
}

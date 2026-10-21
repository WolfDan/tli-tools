use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use gpui::DisplayId;
use tartarus_core::{AdviceSnapshot, BoardModel, RunPhase};
use tartarus_scan::UiReadProfile;

use crate::capture::GameWindowPlacement;
use crate::tray::ControlHub;

const _: () = {
    assert!(std::mem::size_of::<PanelLayout>() <= 4);
    assert!(std::mem::size_of::<UiToWorker>() <= 24);
};

pub fn read_process_rss_mb() -> f64 {
    let rss_mb = memory_stats::memory_stats()
        .map(|s| (s.physical_mem as f64) / (1024.0 * 1024.0))
        .unwrap_or(0.0);
    assert!(rss_mb.is_finite());
    assert!(rss_mb >= 0.0);
    rss_mb
}

#[derive(Clone, Debug)]
pub enum UiToWorker {
    CaptureNow,
    ConfirmTools,
    AckTreasureNow,
    UpdateModelAndRecheck(Box<BoardModel>),
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
    /// Automatically folded during a Riot cutscene; remembers which layout
    /// to restore when Riot ends.
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
        let was_folded = self.is_folded();
        *self = if was_folded {
            Self::Compact
        } else {
            Self::FoldedManual
        };
        assert_ne!(self.is_folded(), was_folded);
        assert!(!self.is_inspector_open());
    }

    pub fn toggle_inspector(&mut self) {
        let was_open = self.is_inspector_open();
        *self = match *self {
            Self::Inspector { .. } => Self::Compact,
            _ => Self::Inspector {
                tools_drawer: false,
            },
        };
        assert_ne!(self.is_inspector_open(), was_open);
        assert!(!self.is_folded());
    }

    pub fn open_inspector(&mut self, with_tools_drawer: bool) {
        let cur_tools = match *self {
            Self::Inspector { tools_drawer } => tools_drawer,
            _ => false,
        };
        *self = Self::Inspector {
            tools_drawer: cur_tools || with_tools_drawer,
        };
        assert!(self.is_inspector_open());
        assert!(!self.is_folded());
        assert!(!with_tools_drawer || self.is_tools_drawer_open());
    }

    pub fn toggle_tools_drawer(&mut self) {
        let prev_tools = self.is_tools_drawer_open();
        if let Self::Inspector { tools_drawer } = self {
            *tools_drawer = !*tools_drawer;
        } else {
            *self = Self::Inspector { tools_drawer: true };
        }
        assert!(self.is_inspector_open());
        assert_ne!(self.is_tools_drawer_open(), prev_tools);
    }

    pub fn apply_riot(&mut self, riot_on: bool) -> bool {
        let changed = if riot_on {
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
        };
        assert!(!riot_on || self.is_folded());
        assert!(riot_on || !matches!(*self, Self::FoldedForRiot { .. }));
        changed
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
        assert!(fallback.is_none() || self.id().is_some());
        assert!(matches!(self, Self::Auto(_) | Self::Manual(_)));
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
    pub scan_profile: UiReadProfile,
    pub solve_ms: f64,
    pub rss_mb: f64,
    pub status_text: String,
}

pub fn initial_snapshot() -> OverlaySnapshot {
    let snap = OverlaySnapshot {
        model: BoardModel::blank(),
        advice: AdviceSnapshot::default(),
        phase: RunPhase::WaitingForRun,
        riot: false,
        notes: Vec::new(),
        game_connected: false,
        log_connected: false,
        cap_ms: 0.0,
        scan_ms: 0.0,
        scan_profile: UiReadProfile::default(),
        solve_ms: 0.0,
        rss_mb: read_process_rss_mb(),
        status_text: "Waiting for the next move".into(),
    };
    assert!(!snap.status_text.is_empty());
    assert!(snap.rss_mb >= 0.0);
    snap
}

#[derive(Clone, Debug)]
pub enum WorkerToUi {
    Snapshot(Box<OverlaySnapshot>),
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

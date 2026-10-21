#![feature(portable_simd)]
#![forbid(unsafe_code)]

pub mod log_watcher;
pub mod tracker;
pub mod ui_reader;

pub use log_watcher::{ActiveUiView, LogEvent, LogWatcher};
pub use tracker::{CaptureTrigger, RunTracker, ScanReading};
pub use ui_reader::{
    BoardRead, CellKindRead, CellRead, ConvoyChainRead, DecisionRead, DoorCandidates, DoorsRead,
    FrameView, GiftCardRead, GiftModalRead, ReadBoardOptions, RelicCardRead, RelicModalRead,
    TitleModalRead, ToolSlotRead, UiReadProfile, UiReadResult, UiReader, UmgViewport, WardenRead,
    locate_board, read_cell, read_convoy_chain, read_gift_modal, refine_doors_with_warden,
};

//! Concrete UMG + `std::simd::f32x16` UI widget reader (`UiReader`) for S16 Tartarus.
//!
//! Replaces legacy global screen-scanning CV with `.pak`-driven UMG layout projection
//! (`UmgViewport`), 64-byte cache-line-aligned SIMD templates (`ui_data.rs`), and explicit
//! view-specific widget readers that eliminate overlap with `UE_game.log`.

pub mod board;
pub mod data;
pub mod glyphs;
pub mod hud;
pub mod modals;
pub mod simd;
pub mod viewport;

use rayon::prelude::*;
use tartarus_core::{FixedVec, ObjectiveId, StartEventId};

pub use board::{CellKindRead, CellRead, WardenRead, read_cell};
pub use hud::{DecisionRead, DoorCandidates, DoorsRead, ToolSlotRead, refine_doors_with_warden};
pub use modals::{
    ConvoyChainRead, GiftCardRead, GiftModalRead, RelicCardRead, RelicModalRead, TitleModalRead,
    read_convoy_chain, read_gift_modal,
};
pub use simd::FrameView;
pub use viewport::{UmgViewport, locate_board};

use board::{
    find_event_blobs_with_slot, find_warden_with_slot, match_events_from_blobs, read_cell_with_slot,
};
use glyphs::{
    Blob, CellScratchSlot, Line, UiScratchSlot, prepare_sat_region, scan_radii_from_built_sat,
    text_radii_for_scale,
};
use hud::{
    read_chromatic_with_slot, read_days_with_slot, read_decision_from_lines, read_doors_from_lines,
    read_hud_objective_with_slot, read_tools_with_slot,
};
use modals::{
    chain_banner_score, has_devour_banner, read_convoy_chain_with_banner,
    read_relic_modal_with_slot, read_title_modal_with_slot,
};
use viewport::{board_template_refs, locate_by_labels_with_slot};

/// Timing breakdown (in milliseconds) for a `UiReader` operation.
#[derive(Clone, Copy, Debug, Default)]
pub struct UiReadProfile {
    pub viewport_ms: f64,
    pub board_ms: f64,
    pub hud_ms: f64,
    pub modal_ms: f64,
    pub ms_total: f64,
}

/// Options controlling which HUD sub-widgets `UiReader::read_board` reads.
#[derive(Clone, Copy, Debug)]
pub struct ReadBoardOptions {
    /// When `false` (default when `UE_game.log` is active), skips reading the top-center
    /// day counter because `[S16LastDay]` from the log is 100% authoritative.
    pub read_days: bool,
}

impl Default for ReadBoardOptions {
    fn default() -> Self {
        Self { read_days: true }
    }
}

impl ReadBoardOptions {
    #[inline]
    pub const fn without_days() -> Self {
        Self { read_days: false }
    }
}

/// Complete typed result of reading `S16BoardMainView` (`UiReader::read_board`).
#[derive(Clone, Debug)]
pub struct BoardRead {
    pub ok: bool,
    pub viewport: Option<UmgViewport>,
    pub cells: Option<[CellRead; 16]>,
    pub warden: Option<WardenRead>,
    pub doors: Option<DoorsRead>,
    pub days: Option<(u8, f32)>,
    pub tools: FixedVec<ToolSlotRead, 6>,
    pub chromatic: Option<usize>,
    pub chain: Option<ConvoyChainRead>,
    pub decision: Option<DecisionRead>,
    pub profile: UiReadProfile,
}

/// Result of `UiReader::read_auto` when no `UE_game.log` view state is available.
#[derive(Clone, Debug)]
pub enum UiReadResult {
    Board(Box<BoardRead>),
    RelicModal(RelicModalRead),
    GiftModal(GiftModalRead),
    ObjectiveModal(ObjectiveId),
    ThemeModal(StartEventId),
    TitleModal(TitleModalRead),
    Idle,
}

/// Stateful, zero-allocation UMG widget reader with pre-allocated `#[repr(C, align(64))]`
/// scratch slots.
#[repr(C, align(64))]
pub struct UiReader {
    pub(crate) slots: [UiScratchSlot; 6],
    pub(crate) cell_slots: [CellScratchSlot; 16],
}

const _: () = {
    assert!(std::mem::align_of::<UiReader>() == 64);
};

impl Default for UiReader {
    fn default() -> Self {
        Self::new()
    }
}

fn resolve_viewport_with_fallback(
    frame: &FrameView<'_>,
    hint: Option<UmgViewport>,
    slot0: &mut UiScratchSlot,
) -> (Option<UmgViewport>, f64) {
    assert!(frame.width() > 0 && frame.height() > 0);
    let t_vp = std::time::Instant::now();
    let mut vp = locate_board(frame, hint);
    if vp.is_some_and(|l| l.score >= 0.25 && l.score < 0.65)
        && has_devour_banner(frame)
        && let Some(by_labels) = locate_by_labels_with_slot(frame, slot0)
        && by_labels.cells_matched >= 8
    {
        vp = Some(by_labels);
    }
    let viewport_ms = t_vp.elapsed().as_secs_f64() * 1000.0;
    assert!(viewport_ms.is_finite());
    (vp, viewport_ms)
}

fn read_bottom_hud_lines(
    frame: &FrameView<'_>,
    viewport: &UmgViewport,
    has_chain: bool,
    slot1: &mut UiScratchSlot,
) -> (Vec<Vec<Line>>, Option<DoorsRead>) {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(viewport.s > 0.0);
    let bottom = viewport.at(0.0, 3.0).1.max(viewport.at(5.0, 3.0).1) + 30.0 * viewport.s;
    if has_chain || bottom >= (frame.height() as f64) - 20.0 {
        return (Vec::new(), None);
    }
    let radii = text_radii_for_scale(viewport.s);
    let board_cx = (viewport.at(0.0, 1.5).0 + viewport.at(5.0, 1.5).0) * 0.5;
    let bx0 = (board_cx - 420.0 * viewport.s)
        .max(viewport.at(0.0, 0.0).0.min(viewport.at(0.0, 3.0).0) - 40.0 * viewport.s)
        .max(0.0);
    let bx1 = (board_cx + 420.0 * viewport.s)
        .min(viewport.at(5.0, 0.0).0.max(viewport.at(5.0, 3.0).0) + 40.0 * viewport.s)
        .min((frame.width() - 1) as f64);
    let h_max = (50.0 * viewport.s).max(24.0);
    let by1 = (bottom + 215.0 * viewport.s)
        .min((frame.height() as f64 - 16.0 * viewport.s).max(bottom + 40.0))
        .min((frame.height() - 1) as f64);
    let r_max = radii[3].max(1);
    let Some(dims) = prepare_sat_region(frame, [bx0, bottom, bx1, by1], r_max, 0.34, slot1) else {
        return (Vec::new(), None);
    };
    let mut lines_by_r = scan_radii_from_built_sat(dims, (6.0, h_max), &radii[1..3], slot1);
    let fast_doors = read_doors_from_lines(frame, &lines_by_r);
    let fast_dec = if fast_doors.is_none() {
        read_decision_from_lines(frame, viewport, &lines_by_r)
    } else {
        None
    };
    let dec_complete = fast_dec
        .is_some_and(|d| d.office || d.ratway || (d.end && (d.title || d.replace || d.consume)));
    if fast_doors.as_ref().is_some_and(|d| d.cells.len() >= 2)
        || dec_complete
        || lines_by_r.iter().all(|v| v.is_empty())
    {
        return (lines_by_r, fast_doors);
    }
    let extra = scan_radii_from_built_sat(dims, (6.0, h_max), &[radii[0], radii[3]], slot1);
    let extra_doors = read_doors_from_lines(frame, &extra);
    lines_by_r.extend(extra);
    let doors = match (fast_doors, extra_doors) {
        (Some(a), Some(b)) => {
            if b.cells.len() > a.cells.len() || (b.cells.len() == a.cells.len() && b.conf > a.conf)
            {
                Some(b)
            } else {
                Some(a)
            }
        }
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    (lines_by_r, doors)
}

fn read_cells_and_markers(
    frame: &FrameView<'_>,
    viewport: &UmgViewport,
    cells: &mut [CellRead; 16],
    cell_slots: &mut [CellScratchSlot; 16],
    slot0: &mut UiScratchSlot,
    slot5: &mut UiScratchSlot,
) -> (Option<WardenRead>, Vec<Blob>, f64) {
    assert_eq!(cells.len(), 16);
    assert_eq!(cell_slots.len(), 16);
    let t_board = std::time::Instant::now();
    let (w, eb) = rayon::join(
        || {
            cells
                .par_iter_mut()
                .zip(cell_slots.par_iter_mut())
                .enumerate()
                .for_each(|(n, (out, cslot))| {
                    *out = read_cell_with_slot(frame, viewport, n as u8, cslot);
                });
        },
        || {
            rayon::join(
                || find_warden_with_slot(frame, viewport, slot0),
                || find_event_blobs_with_slot(frame, viewport, slot5),
            )
        },
    )
    .1;
    (w, eb, t_board.elapsed().as_secs_f64() * 1000.0)
}

fn apply_event_bits(cells: &mut [CellRead; 16], mut ev_bits: u16) {
    assert_eq!(cells.len(), 16);
    let mut iterations = 0usize;
    while ev_bits != 0 {
        iterations += 1;
        assert!(iterations <= 16);
        let n = ev_bits.trailing_zeros() as usize;
        ev_bits &= ev_bits - 1;
        let c = &mut cells[n];
        if c.kind == CellKindRead::Locked || c.kind == CellKindRead::Inmate {
            c.event = true;
        }
    }
}

fn empty_board_read(viewport: Option<UmgViewport>, viewport_ms: f64, ms_total: f64) -> BoardRead {
    assert!(viewport_ms.is_finite());
    assert!(ms_total.is_finite());
    BoardRead {
        ok: false,
        viewport,
        cells: None,
        warden: None,
        doors: None,
        days: None,
        tools: FixedVec::new(),
        chromatic: None,
        chain: None,
        decision: None,
        profile: UiReadProfile {
            viewport_ms,
            board_ms: 0.0,
            hud_ms: 0.0,
            modal_ms: 0.0,
            ms_total,
        },
    }
}

type HudScanTuple = (
    (Vec<Vec<Line>>, Option<DoorsRead>),
    (
        Option<(u8, f32)>,
        (FixedVec<ToolSlotRead, 6>, Option<usize>),
    ),
);

fn read_hud_panels(
    frame: &FrameView<'_>,
    viewport: &UmgViewport,
    has_chain: bool,
    opts: ReadBoardOptions,
    slots: (
        &mut UiScratchSlot,
        &mut UiScratchSlot,
        &mut UiScratchSlot,
        &mut UiScratchSlot,
    ),
) -> (HudScanTuple, f64) {
    assert!(frame.width() > 0 && frame.height() > 0);
    assert!(viewport.s > 0.0);
    if viewport.by_labels {
        return (((Vec::new(), None), (None, (FixedVec::new(), None))), 0.0);
    }
    let (slot1, slot2, slot3, slot4) = slots;
    let t_hud = std::time::Instant::now();
    let res = rayon::join(
        || read_bottom_hud_lines(frame, viewport, has_chain, slot1),
        || {
            rayon::join(
                || {
                    opts.read_days
                        .then(|| read_days_with_slot(frame, viewport, slot2))
                        .flatten()
                },
                || {
                    rayon::join(
                        || read_tools_with_slot(frame, viewport, slot3).0,
                        || read_chromatic_with_slot(frame, viewport, slot4),
                    )
                },
            )
        },
    );
    (res, t_hud.elapsed().as_secs_f64() * 1000.0)
}

impl UiReader {
    /// Creates a pre-faulted, cache-line-aligned `UiReader` ready for zero-allocation frame reads.
    pub fn new() -> Self {
        let _ = board_template_refs();
        let reader = Self {
            slots: std::array::from_fn(|_| UiScratchSlot::with_capacity()),
            cell_slots: std::array::from_fn(|_| CellScratchSlot::with_capacity()),
        };
        assert_eq!(reader.slots.len(), 6);
        assert_eq!(reader.cell_slots.len(), 16);
        reader
    }

    #[inline]
    pub fn reset(&mut self) {
        assert_eq!(self.slots.len(), 6);
        assert_eq!(self.cell_slots.len(), 16);
        for slot in &mut self.slots {
            slot.reset();
        }
        for slot in &mut self.cell_slots {
            slot.reset();
        }
    }

    /// Resolves the `UmgViewport` for `frame`, optionally using a previous `hint` viewport.
    pub fn resolve_viewport(
        &mut self,
        frame: &FrameView<'_>,
        hint: Option<UmgViewport>,
    ) -> Option<UmgViewport> {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        self.slots[0].reset();
        resolve_viewport_with_fallback(frame, hint, &mut self.slots[0]).0
    }

    /// Reads `S16BoardMainView` (16 cells, warden, event markers, bottom route/decision cards,
    /// torture tools panel, chromatic vault, and optional day counter).
    pub fn read_board(
        &mut self,
        frame: &FrameView<'_>,
        hint: Option<UmgViewport>,
        opts: ReadBoardOptions,
    ) -> BoardRead {
        assert!(frame.width() > 0 && frame.height() > 0);
        self.reset();
        let t_total = std::time::Instant::now();
        let (vp_opt, viewport_ms) = resolve_viewport_with_fallback(frame, hint, &mut self.slots[0]);
        if !vp_opt.is_some_and(|l| l.by_labels || l.score >= 0.65) {
            let ms = t_total.elapsed().as_secs_f64() * 1000.0;
            return empty_board_read(vp_opt, viewport_ms, ms);
        }
        let vp = vp_opt.unwrap();
        let chain_banner = chain_banner_score(frame, &vp);
        let has_chain = chain_banner >= 0.20;
        let Self {
            slots: [s0, s1, s2, s3, s4, s5],
            cell_slots,
        } = self;
        let mut cells = [CellRead::default(); 16];
        let (
            (warden, event_blobs, board_ms),
            (((bottom_lines, mut doors), (days, (tools, chromatic))), hud_ms),
        ) = rayon::join(
            || read_cells_and_markers(frame, &vp, &mut cells, cell_slots, s0, s5),
            || read_hud_panels(frame, &vp, has_chain, opts, (s1, s2, s3, s4)),
        );
        let events = match_events_from_blobs(&event_blobs, &vp, &cells, warden.as_ref());
        let chain =
            read_convoy_chain_with_banner(frame, &vp, &cells, warden.as_ref(), chain_banner);
        refine_doors_with_warden(&mut doors, warden.map(|w| w.cell));
        apply_event_bits(&mut cells, events);
        let decision = (doors.is_none() && chain.is_none() && !bottom_lines.is_empty())
            .then(|| read_decision_from_lines(frame, &vp, &bottom_lines))
            .flatten();
        let ms_total = t_total.elapsed().as_secs_f64() * 1000.0;
        assert!(ms_total.is_finite());
        BoardRead {
            ok: true,
            viewport: Some(vp),
            cells: Some(cells),
            warden,
            doors,
            days,
            tools,
            chromatic,
            chain,
            decision,
            profile: UiReadProfile {
                viewport_ms,
                board_ms,
                hud_ms,
                modal_ms: 0.0,
                ms_total,
            },
        }
    }

    /// Reads the 3-card Instrument Pick modal (`S16BoardRelicPop`).
    pub fn read_relic_modal(&mut self, frame: &FrameView<'_>) -> Option<RelicModalRead> {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        self.slots[0].reset();
        read_relic_modal_with_slot(frame, &mut self.slots[0])
    }

    /// Reads the 3-card Colored Reward modal (`S16BoardColoredPop`).
    pub fn read_gift_modal(&mut self, frame: &FrameView<'_>) -> Option<GiftModalRead> {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        read_gift_modal(frame)
    }

    /// Reads the scroll title modal (`S16BoardTaskPop` or `S16BoardThemePop`).
    pub fn read_title_modal(&mut self, frame: &FrameView<'_>) -> Option<TitleModalRead> {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        self.slots[0].reset();
        read_title_modal_with_slot(frame, &mut self.slots[0])
    }

    /// Reads the Objective scroll modal (`S16BoardTaskPop`).
    pub fn read_objective_modal(&mut self, frame: &FrameView<'_>) -> Option<ObjectiveId> {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        match self.read_title_modal(frame)? {
            TitleModalRead::Objective(id) => Some(id),
            TitleModalRead::StartEvent(_) => None,
        }
    }

    /// Reads the Start-Event / Theme scroll modal (`S16BoardThemePop`).
    pub fn read_theme_modal(&mut self, frame: &FrameView<'_>) -> Option<StartEventId> {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        match self.read_title_modal(frame)? {
            TitleModalRead::StartEvent(id) => Some(id),
            TitleModalRead::Objective(_) => None,
        }
    }

    /// Reads the top-left HUD objective widget on `S16BoardMainView`.
    pub fn read_hud_objective(
        &mut self,
        frame: &FrameView<'_>,
        viewport: &UmgViewport,
    ) -> Option<ObjectiveId> {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        self.slots[0].reset();
        read_hud_objective_with_slot(frame, viewport, &mut self.slots[0])
    }

    /// Reads the Consume target banner and red target tags on `S16BoardMainView`.
    pub fn read_convoy_chain(
        &mut self,
        frame: &FrameView<'_>,
        viewport: &UmgViewport,
        cells: &[CellRead; 16],
        warden: Option<&WardenRead>,
    ) -> Option<ConvoyChainRead> {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        read_convoy_chain(frame, viewport, cells, warden)
    }

    /// Fallback automatic reader when `UE_game.log` is unavailable (`ActiveUiView::Unknown`).
    pub fn read_auto(
        &mut self,
        frame: &FrameView<'_>,
        hint: Option<UmgViewport>,
        opts: ReadBoardOptions,
    ) -> UiReadResult {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        let board = self.read_board(frame, hint, opts);
        if board.ok {
            return UiReadResult::Board(Box::new(board));
        }
        if let Some(relic) = self.read_relic_modal(frame) {
            return UiReadResult::RelicModal(relic);
        }
        if let Some(title) = self.read_title_modal(frame) {
            return UiReadResult::TitleModal(title);
        }
        if let Some(gift) = self.read_gift_modal(frame) {
            return UiReadResult::GiftModal(gift);
        }
        UiReadResult::Idle
    }
}

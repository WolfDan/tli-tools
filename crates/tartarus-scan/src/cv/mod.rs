//! Computer vision pipeline for scanning the 4x4 prison board, HUD panels, and modal screens.

pub mod arena;
pub mod board;
pub mod cell;
pub mod data;
pub mod hud;
pub mod image;
pub mod math;
pub mod modals;
pub mod ocr;

use rayon::prelude::*;
use tartarus_core::FixedVec;

pub use arena::{Blob, CellScratchSlot, ScanArena, ScanScratchSlot, blobs_with_slot};
pub use board::{locate_board, locate_by_labels_with_slot};
pub use cell::{
    FaceMatch, FrameRarity, ScannedCell, ScannedCellKind, WardenRead, face_match, face_vector,
    find_events_with_slot, find_warden_with_slot, frame_rarity, padlock_at, read_cell,
    read_cell_with_slot,
};
pub use data::{
    ARROW, CARD_FRAME, CARD_LIFT, CARD_MEDALLIONS, DIGITS, FACE_GX, FACE_GY, FACE_X0, FACE_X1,
    FACE_Y0, FACE_Y1, FACES_LION, FACES_MAMMOTH, FACES_RAT, FACES_WOLF, GLYPH_H, GLYPH_W, H0,
    PITCH0, PREFIX_END, PREFIX_GLYPHS, PREFIX_H, PREFIX_NUM, PREFIX_W, REF_BOARD_HEIGHT,
    REF_BOARD_POINTS, REF_BOARD_WIDTH, REF_PICK_TOOLS, REF_TOOLS, RING, ROOM_BG_BOXES,
    RefPickToolEntry, RefToolEntry, S16_RELIC_META, S16RelicMeta,
};
pub use hud::{
    DecisionRead, DoorsRead, ScannedTool, art_feature, art_similarity, is_overlay_occluded,
    read_chromatic_with_slot, read_days_with_slot, read_hud_objective_with_slot,
    read_tools_with_slot, refine_doors_with_warden,
};
pub use image::{
    ImageView, color_share, color_share_rgb, half, half_gray_of, hsv, is_banner_red_rgb, is_gold,
    is_gold_rgb, is_jaws, is_jaws_rgb, is_nest_rgb, is_orange, is_orange_rgb, sample,
};
pub use math::{BoardLoc, cosine, dot_f32, invert, project};
pub use modals::{
    ChainRead, PickCardRead, PickScreenRead, TitleRead, pick_art_box, pick_confirm_shown,
    pick_reroll_shown, read_chain_screen, read_objective_title_with_slot, scan_pick_with_slot,
};
pub use ocr::{
    LabelBox, Line, Piece, Word, digit_scores, find_label_with_slot, glyph_vector, prefix_like,
    prefix_vector, read_number, split_glyph, text_lines_radii_with_slot, text_radii,
    text_radii_for_scale,
};

use cell::{find_event_blobs_with_slot, match_events_from_blobs};
use hud::{read_decision_from_lines, read_doors_from_lines};
use modals::{chain_banner_score, has_devour_banner, read_chain_screen_with_banner};

#[derive(Clone, Copy, Debug, Default)]
pub struct ScanProfile {
    pub loc_ms: f64,
    pub cells_ms: f64,
    pub ocr_ms: f64,
    pub pick_ms: f64,
    pub ms_total: f64,
}

#[derive(Clone, Debug)]
pub struct ScanResult {
    pub ok: bool,
    pub loc: Option<BoardLoc>,
    pub cells: Option<[ScannedCell; 16]>,
    pub warden: Option<WardenRead>,
    pub doors: Option<DoorsRead>,
    pub days: Option<(u8, f32)>,
    pub tools: FixedVec<ScannedTool, 6>,
    pub chromatic: Option<usize>,
    pub chain: Option<ChainRead>,
    pub decision: Option<DecisionRead>,
    pub pick: Option<PickScreenRead>,
    pub title: Option<TitleRead>,
    pub profile: ScanProfile,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ScanOptions {
    pub skip_days: bool,
}

pub fn scan_image_with_arena(
    img: &ImageView<'_>,
    hint: Option<BoardLoc>,
    arena: &mut ScanArena,
) -> ScanResult {
    assert!(img.width > 0);
    assert!(img.height > 0);
    scan_image_with_options_and_arena(img, hint, ScanOptions::default(), arena)
}

fn locate_board_with_fallback(
    img: &ImageView<'_>,
    hint: Option<BoardLoc>,
    slot0: &mut ScanScratchSlot,
) -> (Option<BoardLoc>, f64) {
    assert!(img.width > 0 && img.height > 0);
    let t_loc = std::time::Instant::now();
    let mut loc = locate_board(img, hint);
    if loc.is_some_and(|l| l.score >= 0.25 && l.score < 0.65)
        && has_devour_banner(img)
        && let Some(by_labels) = locate_by_labels_with_slot(img, slot0)
        && by_labels.cells_matched >= 8
    {
        loc = Some(by_labels);
    }
    let loc_ms = t_loc.elapsed().as_secs_f64() * 1000.0;
    assert!(loc_ms.is_finite());
    (loc, loc_ms)
}

fn scan_bottom_hud_lines(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    has_chain: bool,
    slot1: &mut ScanScratchSlot,
) -> (Vec<Vec<Line>>, Option<DoorsRead>) {
    assert!(img.width > 0 && img.height > 0);
    assert!(loc.s > 0.0);
    let bottom = loc.at(0.0, 3.0).1.max(loc.at(5.0, 3.0).1) + 30.0 * loc.s;
    if has_chain || bottom >= (img.height as f64) - 20.0 {
        return (Vec::new(), None);
    }
    let radii = text_radii_for_scale(loc.s);
    let board_cx = (loc.at(0.0, 1.5).0 + loc.at(5.0, 1.5).0) * 0.5;
    let bx0 = (board_cx - 420.0 * loc.s)
        .max(loc.at(0.0, 0.0).0.min(loc.at(0.0, 3.0).0) - 40.0 * loc.s)
        .max(0.0);
    let bx1 = (board_cx + 420.0 * loc.s)
        .min(loc.at(5.0, 0.0).0.max(loc.at(5.0, 3.0).0) + 40.0 * loc.s)
        .min((img.width - 1) as f64);
    let h_max = (50.0 * loc.s).max(24.0);
    let by1 = (bottom + 215.0 * loc.s)
        .min((img.height as f64 - 16.0 * loc.s).max(bottom + 40.0))
        .min((img.height - 1) as f64);
    let mut lines_by_r = text_lines_radii_with_slot(
        img,
        [bx0, bottom, bx1, by1],
        (6.0, h_max),
        &radii[1..3],
        0.34,
        slot1,
    );
    let fast_doors = read_doors_from_lines(img, &lines_by_r);
    let fast_dec = if fast_doors.is_none() {
        read_decision_from_lines(img, loc, &lines_by_r)
    } else {
        None
    };
    let dec_complete =
        fast_dec.is_some_and(|d| d.office || d.ratway || (d.end && (d.replace || d.consume)));
    if fast_doors.is_some() || dec_complete || lines_by_r.iter().all(|v| v.is_empty()) {
        return (lines_by_r, fast_doors);
    }
    let rem = [radii[0], radii[3]];
    let extra = text_lines_radii_with_slot(
        img,
        [bx0, bottom, bx1, by1],
        (6.0, h_max),
        &rem,
        0.34,
        slot1,
    );
    lines_by_r.extend(extra);
    let doors = read_doors_from_lines(img, &lines_by_r);
    (lines_by_r, doors)
}

fn scan_cells_and_markers(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    cells: &mut [ScannedCell; 16],
    cell_slots: &mut [CellScratchSlot; 16],
    slot0: &mut ScanScratchSlot,
    slot5: &mut ScanScratchSlot,
) -> (Option<WardenRead>, Vec<Blob>, f64) {
    assert_eq!(cells.len(), 16);
    assert_eq!(cell_slots.len(), 16);
    let t_cells = std::time::Instant::now();
    let (w, eb) = rayon::join(
        || {
            cells
                .par_iter_mut()
                .zip(cell_slots.par_iter_mut())
                .enumerate()
                .for_each(|(n, (out, cslot))| {
                    *out = read_cell_with_slot(img, loc, n as u8, cslot);
                });
        },
        || {
            rayon::join(
                || find_warden_with_slot(img, loc, slot0),
                || find_event_blobs_with_slot(img, loc, slot5),
            )
        },
    )
    .1;
    (w, eb, t_cells.elapsed().as_secs_f64() * 1000.0)
}

fn apply_event_bits(cells: &mut [ScannedCell; 16], mut ev_bits: u16) {
    assert_eq!(cells.len(), 16);
    let mut iterations = 0usize;
    while ev_bits != 0 {
        iterations += 1;
        assert!(iterations <= 16);
        let n = ev_bits.trailing_zeros() as usize;
        ev_bits &= ev_bits - 1;
        let c = &mut cells[n];
        if c.kind == ScannedCellKind::Locked || c.kind == ScannedCellKind::Inmate {
            c.event = true;
        }
    }
}

fn empty_scan_result(loc: Option<BoardLoc>, loc_ms: f64, ms_total: f64) -> ScanResult {
    assert!(loc_ms.is_finite());
    assert!(ms_total.is_finite());
    ScanResult {
        ok: false,
        loc,
        cells: None,
        warden: None,
        doors: None,
        days: None,
        tools: FixedVec::new(),
        chromatic: None,
        chain: None,
        decision: None,
        pick: None,
        title: None,
        profile: ScanProfile {
            loc_ms,
            cells_ms: 0.0,
            ocr_ms: 0.0,
            pick_ms: 0.0,
            ms_total,
        },
    }
}

type OcrScanTuple = (
    (Vec<Vec<Line>>, Option<DoorsRead>),
    (Option<(u8, f32)>, (FixedVec<ScannedTool, 6>, Option<usize>)),
);

fn scan_ocr_and_hud_panels(
    img: &ImageView<'_>,
    loc: &BoardLoc,
    has_chain: bool,
    opts: ScanOptions,
    slots: (
        &mut ScanScratchSlot,
        &mut ScanScratchSlot,
        &mut ScanScratchSlot,
        &mut ScanScratchSlot,
    ),
) -> (OcrScanTuple, f64) {
    assert!(img.width > 0 && img.height > 0);
    assert!(loc.s > 0.0);
    if loc.by_labels {
        return (((Vec::new(), None), (None, (FixedVec::new(), None))), 0.0);
    }
    let (slot1, slot2, slot3, slot4) = slots;
    let t_ocr = std::time::Instant::now();
    let res = rayon::join(
        || scan_bottom_hud_lines(img, loc, has_chain, slot1),
        || {
            rayon::join(
                || {
                    (!opts.skip_days)
                        .then(|| read_days_with_slot(img, loc, slot2))
                        .flatten()
                },
                || {
                    rayon::join(
                        || read_tools_with_slot(img, loc, slot3).0,
                        || read_chromatic_with_slot(img, loc, slot4),
                    )
                },
            )
        },
    );
    (res, t_ocr.elapsed().as_secs_f64() * 1000.0)
}

pub fn scan_image_with_options_and_arena(
    img: &ImageView<'_>,
    hint: Option<BoardLoc>,
    opts: ScanOptions,
    arena: &mut ScanArena,
) -> ScanResult {
    assert!(img.width > 0);
    assert!(img.height > 0);
    arena.reset();
    let t_total = std::time::Instant::now();
    let (loc_opt, loc_ms) = locate_board_with_fallback(img, hint, &mut arena.slots[0]);
    if !loc_opt.is_some_and(|l| l.by_labels || l.score >= 0.65) {
        return empty_scan_result(loc_opt, loc_ms, t_total.elapsed().as_secs_f64() * 1000.0);
    }
    let loc = loc_opt.unwrap();
    let chain_banner = chain_banner_score(img, &loc);
    let has_chain = chain_banner >= 0.20;
    let ScanArena {
        slots: [slot0, slot1, slot2, slot3, slot4, slot5],
        cell_slots,
    } = arena;
    let mut cells = [ScannedCell::default(); 16];
    let (
        (warden, event_blobs, cells_ms),
        (((bottom_lines, mut doors), (days, (tools, chromatic))), ocr_ms),
    ) = rayon::join(
        || scan_cells_and_markers(img, &loc, &mut cells, cell_slots, slot0, slot5),
        || scan_ocr_and_hud_panels(img, &loc, has_chain, opts, (slot1, slot2, slot3, slot4)),
    );
    let events = match_events_from_blobs(&event_blobs, &loc, &cells, warden.as_ref());
    let chain = read_chain_screen_with_banner(img, &loc, &cells, warden.as_ref(), chain_banner);
    refine_doors_with_warden(&mut doors, warden.map(|w| w.cell));
    apply_event_bits(&mut cells, events);
    let decision = if doors.is_some() || chain.is_some() || bottom_lines.is_empty() {
        None
    } else {
        read_decision_from_lines(img, &loc, &bottom_lines)
    };
    let ms_total = t_total.elapsed().as_secs_f64() * 1000.0;
    ScanResult {
        ok: true,
        loc: Some(loc),
        cells: Some(cells),
        warden,
        doors,
        days,
        tools,
        chromatic,
        chain,
        decision,
        pick: None,
        title: None,
        profile: ScanProfile {
            loc_ms,
            cells_ms,
            ocr_ms,
            pick_ms: 0.0,
            ms_total,
        },
    }
}

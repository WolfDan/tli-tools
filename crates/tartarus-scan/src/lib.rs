#![feature(portable_simd)]
#![forbid(unsafe_code)]

pub mod cv;
pub mod tracker;

pub use cv::{
    ARROW, Blob, BoardLoc, CARD_FRAME, CARD_LIFT, CARD_MEDALLIONS, CellScratchSlot, ChainRead,
    DIGITS, DecisionRead, DoorsRead, FACE_GX, FACE_GY, FACE_X0, FACE_X1, FACE_Y0, FACE_Y1,
    FACES_LION, FACES_MAMMOTH, FACES_RAT, FACES_WOLF, FaceMatch, FrameRarity, GLYPH_H, GLYPH_W, H0,
    ImageView, LabelBox, Line, PITCH0, PREFIX_END, PREFIX_GLYPHS, PREFIX_H, PREFIX_NUM, PREFIX_W,
    PickCardRead, PickScreenRead, Piece, REF_BOARD_HEIGHT, REF_BOARD_POINTS, REF_BOARD_WIDTH,
    REF_PICK_TOOLS, REF_TOOLS, RING, ROOM_BG_BOXES, RefPickToolEntry, RefToolEntry, S16_RELIC_META,
    S16RelicMeta, ScanArena, ScanOptions, ScanProfile, ScanResult, ScanScratchSlot, ScannedCell,
    ScannedCellKind, ScannedTool, TitleRead, WardenRead, Word, art_feature, art_similarity,
    blobs_with_slot, color_share, color_share_rgb, cosine, digit_scores, dot_f32, face_match,
    face_vector, find_events_with_slot, find_label_with_slot, find_warden_with_slot, frame_rarity,
    glyph_vector, half, half_gray_of, hsv, invert, is_banner_red_rgb, is_gold, is_gold_rgb,
    is_jaws, is_jaws_rgb, is_nest_rgb, is_orange, is_orange_rgb, is_overlay_occluded, locate_board,
    locate_by_labels_with_slot, padlock_at, pick_art_box, pick_confirm_shown, pick_reroll_shown,
    prefix_like, prefix_vector, project, read_cell, read_cell_with_slot, read_chain_screen,
    read_chromatic_with_slot, read_days_with_slot, read_hud_objective_with_slot, read_number,
    read_objective_title_with_slot, read_tools_with_slot, refine_doors_with_warden, sample,
    scan_image_with_arena, scan_image_with_options_and_arena, scan_pick_with_slot, split_glyph,
    text_lines_radii_with_slot, text_radii, text_radii_for_scale,
};
pub use tracker::{CaptureTrigger, RunTracker, ScanReading};

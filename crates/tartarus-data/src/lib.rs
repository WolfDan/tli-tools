//! Authoritative S16 Tartarus game database, visual asset catalog renderer,
//! and static Rust CV table generator (`crates/tartarus-scan/src/cv_data.rs`).

pub mod assets;
pub mod catalog;
pub mod codegen;
pub mod db;
pub mod error;
pub mod font;

pub use assets::{
    DecodedAssetImage, alpha_bbox, alpha_bbox_thresh, compute_art_feature_region, hsv,
    load_s16_textures_by_name, pick_art_bbox_on_rgba, synthesize_icon_feature,
};
pub use catalog::{
    render_catalog_beasts_and_cells_png, render_catalog_board_layout_png,
    render_catalog_gifts_themes_events_png, render_catalog_relics_png,
};
pub use codegen::{
    generate_cv_data_rs, synthesize_beast_face_templates, synthesize_board_points_and_rooms,
    synthesize_card_medallions, synthesize_digits_and_prefix,
};
pub use db::{
    S16BeastEntry, S16EventEntry, S16GameDatabase, S16GiftEntry, S16RelicEntry, S16TaskEntry,
    canonical_beasts, extract_s16_game_database, map_event_id_to_sim, map_gift_id_to_sim,
    map_relic_id_to_sim, map_task_id_to_sim,
};
pub use error::DataError;
pub use font::{BoardPoint, DigitTemplates, PrefixTemplates, RasterizedGlyph, RoomBox, TtfFont};

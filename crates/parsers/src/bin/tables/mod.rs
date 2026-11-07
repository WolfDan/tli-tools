//! Strongly-typed `.bin` configuration tables, compound `FConfig*` sub-structs,
//! `AffixOp3Code` schemas, and `ActionData` variants.
pub mod action_data;
pub mod affix;
pub mod affix_ops;
pub mod attribute;
pub mod audio;
pub mod buff;
pub mod char;
pub mod drop;
pub mod equip;
pub mod hero;
pub mod item;
pub mod level;
pub mod map;
pub mod map_instance;
pub mod mercenary;
pub mod modifier;
pub mod monster;
pub mod npc;
pub mod resource;
pub mod season;
pub mod servant;
pub mod skill;
pub mod sp;
pub mod substructs;
pub mod system;
pub mod talent;
pub mod ui;
pub use action_data::*;
pub use affix::*;
pub use affix_ops::*;
pub use attribute::*;
pub use audio::*;
pub use buff::*;
pub use char::*;
pub use drop::*;
pub use equip::*;
pub use hero::*;
pub use item::*;
pub use level::*;
pub use map::*;
pub use map_instance::*;
pub use mercenary::*;
pub use modifier::*;
pub use monster::*;
pub use npc::*;
pub use resource::*;
pub use season::*;
pub use servant::*;
pub use skill::*;
pub use sp::*;
pub use substructs::*;
pub use system::*;
pub use talent::*;
pub use ui::*;
use crate::bin::container::{BinScratch, normalize_chunk_bin_stem};
use crate::bin::error::BinError;
use crate::bin::types::BinLimits;
type ChunkDomainVerifier = fn(
    &str,
    &[u8],
    &mut BinScratch,
) -> Option<Result<u32, BinError>>;
const CHUNK_DOMAIN_VERIFIERS: [ChunkDomainVerifier; 23] = [
    affix::verify_affix_bin_by_stem,
    attribute::verify_attribute_bin_by_stem,
    audio::verify_audio_bin_by_stem,
    buff::verify_buff_bin_by_stem,
    char::verify_char_bin_by_stem,
    drop::verify_drop_bin_by_stem,
    equip::verify_equip_bin_by_stem,
    hero::verify_hero_bin_by_stem,
    item::verify_item_bin_by_stem,
    level::verify_level_bin_by_stem,
    map::verify_map_bin_by_stem,
    mercenary::verify_mercenary_bin_by_stem,
    modifier::verify_modifier_bin_by_stem,
    monster::verify_monster_bin_by_stem,
    npc::verify_npc_bin_by_stem,
    resource::verify_resource_bin_by_stem,
    season::verify_season_bin_by_stem,
    servant::verify_servant_bin_by_stem,
    skill::verify_skill_bin_by_stem,
    sp::verify_sp_bin_by_stem,
    system::verify_system_bin_by_stem,
    talent::verify_talent_bin_by_stem,
    ui::verify_ui_bin_by_stem,
];
/// Parses a `*_ChunkA0.bin` buffer by file stem (`config_*`) and returns its row count.
pub fn verify_all_chunk_bin_by_stem(
    raw_stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<u32, BinError> {
    let stem = normalize_chunk_bin_stem(raw_stem);
    for verifier in CHUNK_DOMAIN_VERIFIERS {
        if let Some(res) = verifier(stem, buf, scratch) {
            return res;
        }
    }
    Err(BinError::UnknownTableStem {
        stem: stem.into(),
    })
}
/// Parses an uncompressed `Maps/<Map>/config_*.bin` buffer by file stem and returns its row count.
pub fn verify_all_map_bin_by_stem(
    stem: &str,
    buf: &[u8],
    limits: BinLimits,
) -> Result<u32, BinError> {
    if let Some(res) = map_instance::verify_map_instance_bin_by_stem(stem, buf, limits) {
        return res;
    }
    Err(BinError::UnknownTableStem {
        stem: stem.into(),
    })
}

//! Strongly-typed `DataCfg` and `LuaPreLoadTable` parsers grouped into 23 domain modules.

pub mod achievement;
pub mod activity;
pub mod auction;
pub mod chat;
pub mod contract;
pub mod drop;
pub mod equip;
pub mod fashion;
pub mod forge;
pub mod gacha;
pub mod grow_up;
pub mod hero;
pub mod item;
pub mod map;
pub mod modifier;
pub mod preload;
pub mod season;
pub mod servant;
pub mod shop;
pub mod skill;
pub mod sp;
pub mod system;
pub mod talent;

pub use achievement::*;
pub use activity::*;
pub use auction::*;
pub use chat::*;
pub use contract::*;
pub use drop::*;
pub use equip::*;
pub use fashion::*;
pub use forge::*;
pub use gacha::*;
pub use grow_up::*;
pub use hero::*;
pub use item::*;
pub use map::*;
pub use modifier::*;
pub use preload::*;
pub use season::*;
pub use servant::*;
pub use shop::*;
pub use skill::*;
pub use sp::*;
pub use system::*;
pub use talent::*;

use crate::luac::datacfg::DataCfgScratch;
use crate::luac::error::{DataCfgError, LuacError};

type DomainVerifier = fn(&str, &mut [u8], &mut DataCfgScratch) -> Option<Result<u32, LuacError>>;

const DOMAIN_VERIFIERS: [DomainVerifier; 23] = [
    achievement::verify_achievement_datacfg_by_stem,
    activity::verify_activity_datacfg_by_stem,
    auction::verify_auction_datacfg_by_stem,
    chat::verify_chat_datacfg_by_stem,
    contract::verify_contract_datacfg_by_stem,
    drop::verify_drop_datacfg_by_stem,
    equip::verify_equip_datacfg_by_stem,
    fashion::verify_fashion_datacfg_by_stem,
    forge::verify_forge_datacfg_by_stem,
    gacha::verify_gacha_datacfg_by_stem,
    grow_up::verify_grow_up_datacfg_by_stem,
    hero::verify_hero_datacfg_by_stem,
    item::verify_item_datacfg_by_stem,
    map::verify_map_datacfg_by_stem,
    modifier::verify_modifier_datacfg_by_stem,
    preload::verify_preload_datacfg_by_stem,
    season::verify_season_datacfg_by_stem,
    servant::verify_servant_datacfg_by_stem,
    shop::verify_shop_datacfg_by_stem,
    skill::verify_skill_datacfg_by_stem,
    sp::verify_sp_datacfg_by_stem,
    system::verify_system_datacfg_by_stem,
    talent::verify_talent_datacfg_by_stem,
];

/// Evaluates a `DataCfg` `.out` buffer by file stem (`config_*`) and returns its row/entry count.
pub fn verify_all_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<u32, LuacError> {
    for verifier in DOMAIN_VERIFIERS {
        if let Some(res) = verifier(stem, buf, scratch) {
            return res;
        }
    }
    Err(DataCfgError::UnknownField {
        table: "<datacfg_corpus>",
        field: stem.into(),
    }
    .into())
}

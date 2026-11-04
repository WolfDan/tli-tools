//! Auto-generated `LuaPreLoadTable/DataCfg` preload table types and parsers.

use crate::bounded_len_u32;
use crate::luac::datacfg::{
    DataCfgScratch, PreloadTable, parse_preload_list, parse_preload_map, parse_preload_map_2d,
};
use crate::luac::error::LuacError;
use crate::luac::types::IntOrIntList;

/// Typed preload table for `config_preload_attributes_show_type.out`.
pub type ConfigPreloadAttributesShowTypeTable = PreloadTable<Box<str>, Box<[u32]>>;

/// Parses `config_preload_attributes_show_type.out` into a [`ConfigPreloadAttributesShowTypeTable`].
pub fn parse_config_preload_attributes_show_type(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadAttributesShowTypeTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_attributes_show_type")
}

/// Typed preload table for `config_preload_fashion_active_item.out`.
pub type ConfigPreloadFashionActiveItemTable = PreloadTable<Box<str>, Box<[u32]>>;

/// Parses `config_preload_fashion_active_item.out` into a [`ConfigPreloadFashionActiveItemTable`].
pub fn parse_config_preload_fashion_active_item(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadFashionActiveItemTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_fashion_active_item")
}

/// Typed preload table for `config_preload_fashion_type.out`.
pub type ConfigPreloadFashionTypeTable = PreloadTable<Box<str>, Box<[u32]>>;

/// Parses `config_preload_fashion_type.out` into a [`ConfigPreloadFashionTypeTable`].
pub fn parse_config_preload_fashion_type(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadFashionTypeTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_fashion_type")
}

/// Typed preload table for `config_preload_item_base.out`.
pub type ConfigPreloadItemBaseTable = PreloadTable<Box<str>, Box<[u32]>>;

/// Parses `config_preload_item_base.out` into a [`ConfigPreloadItemBaseTable`].
pub fn parse_config_preload_item_base(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadItemBaseTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_item_base")
}

/// Typed preload table for `config_preload_item_base_deal_currency.out`.
pub type ConfigPreloadItemBaseDealCurrencyTable = PreloadTable<u8, Box<[u32]>>;

/// Parses `config_preload_item_base_deal_currency.out` into a [`ConfigPreloadItemBaseDealCurrencyTable`].
pub fn parse_config_preload_item_base_deal_currency(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadItemBaseDealCurrencyTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_item_base_deal_currency")
}

/// Typed preload table for `config_preload_item_gold.out`.
pub type ConfigPreloadItemGoldTable = PreloadTable<u32, Box<[u32]>>;

/// Parses `config_preload_item_gold.out` into a [`ConfigPreloadItemGoldTable`].
pub fn parse_config_preload_item_gold(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadItemGoldTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_item_gold")
}

/// Typed preload table for `config_preload_item_gold_base_id.out`.
pub type ConfigPreloadItemGoldBaseIdTable = PreloadTable<Box<str>, Box<[u32]>>;

/// Parses `config_preload_item_gold_base_id.out` into a [`ConfigPreloadItemGoldBaseIdTable`].
pub fn parse_config_preload_item_gold_base_id(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadItemGoldBaseIdTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_item_gold_base_id")
}

/// Typed preload table for `config_preload_map_modifier_group.out`.
pub type ConfigPreloadMapModifierGroupTable = PreloadTable<u16, Box<[u32]>>;

/// Parses `config_preload_map_modifier_group.out` into a [`ConfigPreloadMapModifierGroupTable`].
pub fn parse_config_preload_map_modifier_group(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadMapModifierGroupTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_map_modifier_group")
}

/// Typed preload table for `config_preload_modifier.out`.
pub type ConfigPreloadModifierTable = PreloadTable<Box<str>, Box<[u32]>>;

/// Parses `config_preload_modifier.out` into a [`ConfigPreloadModifierTable`].
pub fn parse_config_preload_modifier(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadModifierTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_modifier")
}

/// Typed preload table for `config_preload_modifier_affix_key.out`.
pub type ConfigPreloadModifierAffixKeyTable = PreloadTable<Box<str>, u32>;

/// Parses `config_preload_modifier_affix_key.out` into a [`ConfigPreloadModifierAffixKeyTable`].
pub fn parse_config_preload_modifier_affix_key(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadModifierAffixKeyTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_modifier_affix_key")
}

/// Typed preload table for `config_preload_modifier_group.out`.
pub type ConfigPreloadModifierGroupTable = PreloadTable<u32, Box<[u32]>>;

/// Parses `config_preload_modifier_group.out` into a [`ConfigPreloadModifierGroupTable`].
pub fn parse_config_preload_modifier_group(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadModifierGroupTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_modifier_group")
}

/// Typed 2D preload table for `config_preload_modifier_mag.out`.
pub type ConfigPreloadModifierMagTable = PreloadTable<(u8, u32), Box<[u32]>>;

/// Parses `config_preload_modifier_mag.out` into a [`ConfigPreloadModifierMagTable`].
pub fn parse_config_preload_modifier_mag(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadModifierMagTable, LuacError> {
    parse_preload_map_2d(buf, scratch, "config_preload_modifier_mag")
}

/// Typed preload list for `config_preload_servant.out`.
pub type ConfigPreloadServantTable = Box<[u16]>;

/// Parses `config_preload_servant.out` into a [`ConfigPreloadServantTable`].
pub fn parse_config_preload_servant(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadServantTable, LuacError> {
    parse_preload_list(buf, scratch, "config_preload_servant")
}

/// Typed 2D preload table for `config_preload_sp_map_v2_area.out`.
pub type ConfigPreloadSpMapV2AreaTable = PreloadTable<(u16, i8), Box<[u32]>>;

/// Parses `config_preload_sp_map_v2_area.out` into a [`ConfigPreloadSpMapV2AreaTable`].
pub fn parse_config_preload_sp_map_v2_area(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadSpMapV2AreaTable, LuacError> {
    parse_preload_map_2d(buf, scratch, "config_preload_sp_map_v2_area")
}

/// Typed preload table for `config_preload_sp_map_v2_level_id.out`.
pub type ConfigPreloadSpMapV2LevelIdTable = PreloadTable<Box<str>, IntOrIntList<u32>>;

/// Parses `config_preload_sp_map_v2_level_id.out` into a [`ConfigPreloadSpMapV2LevelIdTable`].
pub fn parse_config_preload_sp_map_v2_level_id(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadSpMapV2LevelIdTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_sp_map_v2_level_id")
}

/// Typed preload table for `config_preload_sp_map_v2_talentpoint.out`.
pub type ConfigPreloadSpMapV2TalentpointTable = PreloadTable<Box<str>, Box<[u32]>>;

/// Parses `config_preload_sp_map_v2_talentpoint.out` into a [`ConfigPreloadSpMapV2TalentpointTable`].
pub fn parse_config_preload_sp_map_v2_talentpoint(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadSpMapV2TalentpointTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_sp_map_v2_talentpoint")
}

/// Typed preload table for `config_preload_talent_god.out`.
pub type ConfigPreloadTalentGodTable = PreloadTable<Box<str>, Box<[u32]>>;

/// Parses `config_preload_talent_god.out` into a [`ConfigPreloadTalentGodTable`].
pub fn parse_config_preload_talent_god(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadTalentGodTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_talent_god")
}

/// Typed preload list for `config_preload_talent_sorted_id.out`.
pub type ConfigPreloadTalentSortedIdTable = Box<[u32]>;

/// Parses `config_preload_talent_sorted_id.out` into a [`ConfigPreloadTalentSortedIdTable`].
pub fn parse_config_preload_talent_sorted_id(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadTalentSortedIdTable, LuacError> {
    parse_preload_list(buf, scratch, "config_preload_talent_sorted_id")
}

/// Typed preload table for `config_preload_talent_type.out`.
pub type ConfigPreloadTalentTypeTable = PreloadTable<Box<str>, Box<[u32]>>;

/// Parses `config_preload_talent_type.out` into a [`ConfigPreloadTalentTypeTable`].
pub fn parse_config_preload_talent_type(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPreloadTalentTypeTable, LuacError> {
    parse_preload_map(buf, scratch, "config_preload_talent_type")
}

pub(crate) fn verify_preload_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_preload_attributes_show_type" => {
            Some(parse_config_preload_attributes_show_type(buf, scratch).map(|t| t.len()))
        }
        "config_preload_fashion_active_item" => {
            Some(parse_config_preload_fashion_active_item(buf, scratch).map(|t| t.len()))
        }
        "config_preload_fashion_type" => {
            Some(parse_config_preload_fashion_type(buf, scratch).map(|t| t.len()))
        }
        "config_preload_item_base" => {
            Some(parse_config_preload_item_base(buf, scratch).map(|t| t.len()))
        }
        "config_preload_item_base_deal_currency" => {
            Some(parse_config_preload_item_base_deal_currency(buf, scratch).map(|t| t.len()))
        }
        "config_preload_item_gold" => {
            Some(parse_config_preload_item_gold(buf, scratch).map(|t| t.len()))
        }
        "config_preload_item_gold_base_id" => {
            Some(parse_config_preload_item_gold_base_id(buf, scratch).map(|t| t.len()))
        }
        "config_preload_map_modifier_group" => {
            Some(parse_config_preload_map_modifier_group(buf, scratch).map(|t| t.len()))
        }
        "config_preload_modifier" => {
            Some(parse_config_preload_modifier(buf, scratch).map(|t| t.len()))
        }
        "config_preload_modifier_affix_key" => {
            Some(parse_config_preload_modifier_affix_key(buf, scratch).map(|t| t.len()))
        }
        "config_preload_modifier_group" => {
            Some(parse_config_preload_modifier_group(buf, scratch).map(|t| t.len()))
        }
        "config_preload_modifier_mag" => {
            Some(parse_config_preload_modifier_mag(buf, scratch).map(|t| t.len()))
        }
        "config_preload_servant" => {
            Some(parse_config_preload_servant(buf, scratch).map(|t| bounded_len_u32(t.len())))
        }
        "config_preload_sp_map_v2_area" => {
            Some(parse_config_preload_sp_map_v2_area(buf, scratch).map(|t| t.len()))
        }
        "config_preload_sp_map_v2_level_id" => {
            Some(parse_config_preload_sp_map_v2_level_id(buf, scratch).map(|t| t.len()))
        }
        "config_preload_sp_map_v2_talentpoint" => {
            Some(parse_config_preload_sp_map_v2_talentpoint(buf, scratch).map(|t| t.len()))
        }
        "config_preload_talent_god" => {
            Some(parse_config_preload_talent_god(buf, scratch).map(|t| t.len()))
        }
        "config_preload_talent_sorted_id" => Some(
            parse_config_preload_talent_sorted_id(buf, scratch).map(|t| bounded_len_u32(t.len())),
        ),
        "config_preload_talent_type" => {
            Some(parse_config_preload_talent_type(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

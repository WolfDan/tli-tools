//! Auto-generated `.bin` configuration table types and parsers for the `modifier` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
/// Row record for `config_effect_source_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEffectSourceRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigEffectSourceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_effect_source_ChunkA0.bin`.
pub type ConfigEffectSourceTable = BinTable<i32, ConfigEffectSourceRow>;
/// Parses `config_effect_source_ChunkA0.bin` into a [`ConfigEffectSourceTable`].
pub fn parse_config_effect_source(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigEffectSourceTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_modifier_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigModifierRow {
    pub affix: Box<[FConfigAffix]>,
    pub server_affix: Box<str>,
    pub dynmc_params: Box<[Box<[f64]>]>,
    pub affix_key: Box<str>,
    pub tags: Box<[i32]>,
    pub param_marker: Box<[i32]>,
    pub slot_num: Box<[i32]>,
    pub description: Box<str>,
    pub merc_mod_tag: Box<[i32]>,
    pub id: i32,
    pub group: i32,
    pub tier: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
    pub modifier_type: i32,
    pub modifier_level: i32,
    pub level: i32,
    pub weight: i32,
    pub exclusive_group: i32,
    pub drop_level: i32,
    pub drop_weight: i32,
    pub forge_level: i32,
    pub forge_weight: i32,
    pub forge_precise_level: i32,
    pub forge_precise_weight: i32,
    pub forge_reroll_level: i32,
    pub forge_reroll_weight: i32,
    pub ashes: i32,
    pub ashes_consume: i32,
    pub exclusive: i32,
    pub source: i32,
    pub drop_num: i32,
    pub max_drop_level: i32,
    pub legendary_drop_level: i32,
    pub legendary_drop_weight: i32,
    pub belong_affix_library: i32,
    pub extra_tier: i32,
    pub is_valid: bool,
}
impl ConfigModifierRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.group = cur.read_i32()?;
        self.tier = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        self.modifier_type = cur.read_i32()?;
        self.modifier_level = cur.read_i32()?;
        self.level = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        self.exclusive_group = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.drop_level = cur.read_i32()?;
        self.drop_weight = cur.read_i32()?;
        self.forge_level = cur.read_i32()?;
        self.forge_weight = cur.read_i32()?;
        self.forge_precise_level = cur.read_i32()?;
        self.forge_precise_weight = cur.read_i32()?;
        self.forge_reroll_level = cur.read_i32()?;
        self.forge_reroll_weight = cur.read_i32()?;
        self.affix = FConfigAffix::read_array(cur)?;
        self.server_affix = cur.read_str()?;
        self.dynmc_params = cur.read_array_2d_f64()?;
        self.ashes = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.ashes_consume = cur.read_i32()?;
        self.exclusive = cur.read_i32()?;
        self.source = cur.read_i32()?;
        self.drop_num = cur.read_i32()?;
        self.affix_key = cur.read_str()?;
        self.is_valid = cur.read_bool()?;
        self.tags = cur.read_array_i32()?;
        self.max_drop_level = cur.read_i32()?;
        self.legendary_drop_level = cur.read_i32()?;
        self.legendary_drop_weight = cur.read_i32()?;
        self.belong_affix_library = cur.read_i32()?;
        self.extra_tier = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.param_marker = cur.read_array_i32()?;
        self.slot_num = cur.read_array_i32()?;
        self.description = cur.read_str()?;
        self.merc_mod_tag = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigModifierRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_modifier_ChunkA0.bin`.
pub type ConfigModifierTable = BinTable<i32, ConfigModifierRow>;
/// Parses `config_modifier_ChunkA0.bin` into a [`ConfigModifierTable`].
pub fn parse_config_modifier(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigModifierTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_modifier_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_effect_source" => {
            Some(parse_config_effect_source(buf, scratch).map(|t| t.len()))
        }
        "config_modifier" => Some(parse_config_modifier(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

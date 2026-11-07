//! Auto-generated `.bin` configuration table types and parsers for the `attribute` domain.
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{FConfigI32ArrayStr, FConfigI32I32};
use crate::bin::types::{BinTable, VecDim};
use crate::bin::wire::WireCursor;
/// Row record for `config_attributes_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigAttributesRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub other_name: Box<str>,
    pub initial_value: f64,
    pub limit_range_value: VecDim,
    pub limit_min_incr_ext_value: f64,
    pub inherit_group: Box<[i32]>,
    pub show: Box<str>,
    pub id: i32,
    pub showtype: i32,
    pub show_format: FConfigI32I32,
    pub implemented: bool,
    pub allow_base: bool,
    pub allow_floating: bool,
    pub allow_increase: bool,
    pub allow_incr_ext: bool,
    pub allow_fixed: bool,
    pub limit_range: bool,
    pub limit_min_incr_ext: bool,
    pub allow_percent: bool,
    pub allow_negetive: bool,
    pub afl_inherit_banlist: bool,
}
impl ConfigAttributesRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.other_name = cur.read_str()?;
        self.initial_value = cur.read_f64()?;
        self.showtype = cur.read_i32()?;
        self.show_format = FConfigI32I32::read(cur)?;
        self.implemented = cur.read_bool()?;
        self.allow_base = cur.read_bool()?;
        self.allow_floating = cur.read_bool()?;
        self.allow_increase = cur.read_bool()?;
        self.allow_incr_ext = cur.read_bool()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.allow_fixed = cur.read_bool()?;
        self.limit_range = cur.read_bool()?;
        self.limit_range_value = cur.read_vec_dim()?;
        self.limit_min_incr_ext = cur.read_bool()?;
        self.limit_min_incr_ext_value = cur.read_f64()?;
        self.allow_percent = cur.read_bool()?;
        self.allow_negetive = cur.read_bool()?;
        self.inherit_group = cur.read_array_i32()?;
        self.afl_inherit_banlist = cur.read_bool()?;
        self.show = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigAttributesRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_attributes_ChunkA0.bin`.
pub type ConfigAttributesTable = BinTable<i32, ConfigAttributesRow>;
/// Parses `config_attributes_ChunkA0.bin` into a [`ConfigAttributesTable`].
pub fn parse_config_attributes(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAttributesTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_attributes_effect_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigAttributesEffectRow {
    pub to_tags: Box<[i32]>,
    pub mod_rate: f64,
    pub id: i32,
    pub effect_type: i32,
    pub from_id: i32,
    pub to_id: i32,
    pub mod_type: i32,
}
impl BinRow for ConfigAttributesEffectRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.effect_type = cur.read_i32()?;
        self.from_id = cur.read_i32()?;
        self.to_id = cur.read_i32()?;
        self.to_tags = cur.read_array_i32()?;
        self.mod_type = cur.read_i32()?;
        self.mod_rate = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_attributes_effect_ChunkA0.bin`.
pub type ConfigAttributesEffectTable = BinTable<i32, ConfigAttributesEffectRow>;
/// Parses `config_attributes_effect_ChunkA0.bin` into a [`ConfigAttributesEffectTable`].
pub fn parse_config_attributes_effect(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAttributesEffectTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_attributes_show_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAttributesShowRow {
    pub name: Box<str>,
    pub detail: Box<str>,
    pub tag_and: Box<[i32]>,
    pub tag: Box<[i32]>,
    pub tag_not: Box<[i32]>,
    pub tag_cal: Box<[Box<str>]>,
    pub show_sources: Box<[i32]>,
    pub formula: Box<str>,
    pub attr_detail: Box<[FConfigI32ArrayStr]>,
    pub appearance: Box<str>,
    pub hidden_condition: Box<str>,
    pub alter_appearance: Box<str>,
    pub alter_condition: Box<str>,
    pub id: i32,
    pub r#type: i32,
    pub sequence: i32,
    pub isoffhand: i32,
    pub enable_less_tag: i32,
    pub bind_skill: i32,
    pub is_abbr: bool,
}
impl ConfigAttributesShowRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.detail = cur.read_str()?;
        self.r#type = cur.read_i32()?;
        self.sequence = cur.read_i32()?;
        self.tag_and = cur.read_array_i32()?;
        self.tag = cur.read_array_i32()?;
        self.tag_not = cur.read_array_i32()?;
        self.tag_cal = cur.read_array_str()?;
        self.isoffhand = cur.read_i32()?;
        self.enable_less_tag = cur.read_i32()?;
        self.show_sources = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.bind_skill = cur.read_i32()?;
        self.formula = cur.read_str()?;
        self.attr_detail = FConfigI32ArrayStr::read_array(cur)?;
        self.appearance = cur.read_str()?;
        self.hidden_condition = cur.read_str()?;
        self.alter_appearance = cur.read_str()?;
        self.alter_condition = cur.read_str()?;
        self.is_abbr = cur.read_bool()?;
        Ok(())
    }
}
impl BinRow for ConfigAttributesShowRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_attributes_show_ChunkA0.bin`.
pub type ConfigAttributesShowTable = BinTable<i32, ConfigAttributesShowRow>;
/// Parses `config_attributes_show_ChunkA0.bin` into a [`ConfigAttributesShowTable`].
pub fn parse_config_attributes_show(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAttributesShowTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_attributes_show_index_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAttributesShowIndexRow {
    pub id: i32,
    pub attrshow: i32,
}
impl BinRow for ConfigAttributesShowIndexRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.attrshow = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_attributes_show_index_ChunkA0.bin`.
pub type ConfigAttributesShowIndexTable = BinTable<i32, ConfigAttributesShowIndexRow>;
/// Parses `config_attributes_show_index_ChunkA0.bin` into a [`ConfigAttributesShowIndexTable`].
pub fn parse_config_attributes_show_index(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAttributesShowIndexTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_damage_impulse_dir_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDamageImpulseDirRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigDamageImpulseDirRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_damage_impulse_dir_ChunkA0.bin`.
pub type ConfigDamageImpulseDirTable = BinTable<i32, ConfigDamageImpulseDirRow>;
/// Parses `config_damage_impulse_dir_ChunkA0.bin` into a [`ConfigDamageImpulseDirTable`].
pub fn parse_config_damage_impulse_dir(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDamageImpulseDirTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_attribute_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_attributes" => {
            Some(parse_config_attributes(buf, scratch).map(|t| t.len()))
        }
        "config_attributes_effect" => {
            Some(parse_config_attributes_effect(buf, scratch).map(|t| t.len()))
        }
        "config_attributes_show" => {
            Some(parse_config_attributes_show(buf, scratch).map(|t| t.len()))
        }
        "config_attributes_show_index" => {
            Some(parse_config_attributes_show_index(buf, scratch).map(|t| t.len()))
        }
        "config_damage_impulse_dir" => {
            Some(parse_config_damage_impulse_dir(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

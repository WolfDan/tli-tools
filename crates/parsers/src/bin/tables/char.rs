//! Auto-generated `.bin` configuration table types and parsers for the `char` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::FConfigI32F64;
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_anim_float_text_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigAnimFloatTextRow {
    pub des: Box<str>,
    pub res_cn: Box<str>,
    pub res_en: Box<str>,
    pub pos: Box<[f64]>,
    pub scale: f64,
    pub anim_name: Box<str>,
    pub id: i32,
    pub anchor_type: i32,
}
impl BinRow for ConfigAnimFloatTextRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.res_cn = cur.read_str()?;
        self.res_en = cur.read_str()?;
        self.pos = cur.read_array_f64()?;
        self.scale = cur.read_f64()?;
        self.anchor_type = cur.read_i32()?;
        self.anim_name = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_anim_float_text_ChunkA0.bin`.
pub type ConfigAnimFloatTextTable = BinTable<i32, ConfigAnimFloatTextRow>;
/// Parses `config_anim_float_text_ChunkA0.bin` into a [`ConfigAnimFloatTextTable`].
pub fn parse_config_anim_float_text(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAnimFloatTextTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_char_field_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigCharFieldRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub initial_value: f64,
    pub inherit_group: Box<[i32]>,
    pub min_value: f64,
    pub max_value: f64,
    pub id: i32,
    pub allow_percent: bool,
    pub afl_inherit_banlist: bool,
    pub has_min: bool,
    pub has_max: bool,
}
impl BinRow for ConfigCharFieldRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.initial_value = cur.read_f64()?;
        self.allow_percent = cur.read_bool()?;
        self.inherit_group = cur.read_array_i32()?;
        self.afl_inherit_banlist = cur.read_bool()?;
        self.has_min = cur.read_bool()?;
        self.min_value = cur.read_f64()?;
        self.has_max = cur.read_bool()?;
        self.max_value = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_char_field_ChunkA0.bin`.
pub type ConfigCharFieldTable = BinTable<i32, ConfigCharFieldRow>;
/// Parses `config_char_field_ChunkA0.bin` into a [`ConfigCharFieldTable`].
pub fn parse_config_char_field(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCharFieldTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_char_label_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCharLabelRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub inherit_group: Box<[i32]>,
    pub id: i32,
    pub afl_inherit_banlist: bool,
}
impl BinRow for ConfigCharLabelRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.inherit_group = cur.read_array_i32()?;
        self.afl_inherit_banlist = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_char_label_ChunkA0.bin`.
pub type ConfigCharLabelTable = BinTable<i32, ConfigCharLabelRow>;
/// Parses `config_char_label_ChunkA0.bin` into a [`ConfigCharLabelTable`].
pub fn parse_config_char_label(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCharLabelTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_char_state_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCharStateRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub category: Box<[i32]>,
    pub vfx: FxHashMap<Box<str>, Box<[i32]>>,
    pub affix_action: FxHashMap<i32, i32>,
    pub anim_suffix: Box<str>,
    pub id: i32,
    pub resource_set_index: i32,
    pub remove_vfx_on_dead: bool,
}
impl BinRow for ConfigCharStateRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.category = cur.read_array_i32()?;
        self.remove_vfx_on_dead = cur.read_bool()?;
        self.vfx = cur.read_map_str_array_i32()?;
        self.affix_action = cur.read_map_i32_i32()?;
        self.anim_suffix = cur.read_str()?;
        self.resource_set_index = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_char_state_ChunkA0.bin`.
pub type ConfigCharStateTable = BinTable<i32, ConfigCharStateRow>;
/// Parses `config_char_state_ChunkA0.bin` into a [`ConfigCharStateTable`].
pub fn parse_config_char_state(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCharStateTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_char_state_category_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigCharStateCategoryRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub affix_action: FConfigAffix,
    pub id: i32,
}
impl BinRow for ConfigCharStateCategoryRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.affix_action = FConfigAffix::read(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_char_state_category_ChunkA0.bin`.
pub type ConfigCharStateCategoryTable = BinTable<i32, ConfigCharStateCategoryRow>;
/// Parses `config_char_state_category_ChunkA0.bin` into a [`ConfigCharStateCategoryTable`].
pub fn parse_config_char_state_category(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCharStateCategoryTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_profession_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigProfessionRow {
    pub name: Box<str>,
    pub des: Box<str>,
    pub talent_id: Box<[i32]>,
    pub talent_num: Box<[f64]>,
    pub icon: Box<str>,
    pub background: Box<str>,
    pub attr: Box<[FConfigI32F64]>,
    pub unlock_condition: Box<[i32]>,
    pub icon_top: Box<str>,
    pub id: i32,
    pub before_id: i32,
}
impl BinRow for ConfigProfessionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.before_id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.des = cur.read_str()?;
        self.talent_id = cur.read_array_i32()?;
        self.talent_num = cur.read_array_f64()?;
        self.icon = cur.read_str()?;
        self.background = cur.read_str()?;
        self.attr = FConfigI32F64::read_array(cur)?;
        self.unlock_condition = cur.read_array_i32()?;
        self.icon_top = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_profession_ChunkA0.bin`.
pub type ConfigProfessionTable = BinTable<i32, ConfigProfessionRow>;
/// Parses `config_profession_ChunkA0.bin` into a [`ConfigProfessionTable`].
pub fn parse_config_profession(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigProfessionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_char_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_anim_float_text" => {
            Some(parse_config_anim_float_text(buf, scratch).map(|t| t.len()))
        }
        "config_char_field" => {
            Some(parse_config_char_field(buf, scratch).map(|t| t.len()))
        }
        "config_char_label" => {
            Some(parse_config_char_label(buf, scratch).map(|t| t.len()))
        }
        "config_char_state" => {
            Some(parse_config_char_state(buf, scratch).map(|t| t.len()))
        }
        "config_char_state_category" => {
            Some(parse_config_char_state_category(buf, scratch).map(|t| t.len()))
        }
        "config_profession" => {
            Some(parse_config_profession(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

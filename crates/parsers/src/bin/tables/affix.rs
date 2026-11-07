//! Auto-generated `.bin` configuration table types and parsers for the `affix` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
/// Row record for `config_affix_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAffixRow {
    pub t1: Box<[i32]>,
    pub t2: Box<[i32]>,
    pub t3: Box<[i32]>,
    pub t4: Box<[i32]>,
    pub t5: Box<[i32]>,
    pub t6: Box<[i32]>,
    pub t7: Box<[i32]>,
    pub t8: Box<[i32]>,
    pub t9: Box<[i32]>,
    pub t10: Box<[i32]>,
    pub t11: Box<[i32]>,
    pub t12: Box<[i32]>,
    pub t13: Box<[i32]>,
    pub t1_name: Box<str>,
    pub t2_name: Box<str>,
    pub t3_name: Box<str>,
    pub t4_name: Box<str>,
    pub t5_name: Box<str>,
    pub t6_name: Box<str>,
    pub t7_name: Box<str>,
    pub t8_name: Box<str>,
    pub t9_name: Box<str>,
    pub t10_name: Box<str>,
    pub t11_name: Box<str>,
    pub t12_name: Box<str>,
    pub t13_name: Box<str>,
    pub id: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
    pub affix_type: i32,
    pub tier: i32,
    pub tag: i32,
    pub if_exclusive: i32,
    pub source: i32,
}
impl ConfigAffixRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        self.affix_type = cur.read_i32()?;
        self.tier = cur.read_i32()?;
        self.t1 = cur.read_array_i32()?;
        self.t2 = cur.read_array_i32()?;
        self.t3 = cur.read_array_i32()?;
        self.t4 = cur.read_array_i32()?;
        self.t5 = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.t6 = cur.read_array_i32()?;
        self.t7 = cur.read_array_i32()?;
        self.t8 = cur.read_array_i32()?;
        self.t9 = cur.read_array_i32()?;
        self.t10 = cur.read_array_i32()?;
        self.t11 = cur.read_array_i32()?;
        self.t12 = cur.read_array_i32()?;
        self.t13 = cur.read_array_i32()?;
        self.t1_name = cur.read_str()?;
        self.t2_name = cur.read_str()?;
        self.t3_name = cur.read_str()?;
        self.t4_name = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.t5_name = cur.read_str()?;
        self.t6_name = cur.read_str()?;
        self.t7_name = cur.read_str()?;
        self.t8_name = cur.read_str()?;
        self.t9_name = cur.read_str()?;
        self.t10_name = cur.read_str()?;
        self.t11_name = cur.read_str()?;
        self.t12_name = cur.read_str()?;
        self.t13_name = cur.read_str()?;
        self.tag = cur.read_i32()?;
        self.if_exclusive = cur.read_i32()?;
        self.source = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigAffixRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_affix_ChunkA0.bin`.
pub type ConfigAffixTable = BinTable<i32, ConfigAffixRow>;
/// Parses `config_affix_ChunkA0.bin` into a [`ConfigAffixTable`].
pub fn parse_config_affix(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAffixTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_affix_blackboard_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAffixBlackboardTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigAffixBlackboardTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_affix_blackboard_type_ChunkA0.bin`.
pub type ConfigAffixBlackboardTypeTable = BinTable<i32, ConfigAffixBlackboardTypeRow>;
/// Parses `config_affix_blackboard_type_ChunkA0.bin` into a [`ConfigAffixBlackboardTypeTable`].
pub fn parse_config_affix_blackboard_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAffixBlackboardTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_affix_class_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAffixClassRow {
    pub affix_effects: Box<[i32]>,
    pub description: Box<str>,
    pub detail_description: Box<str>,
    pub parameter_abstract: Box<[Box<str>]>,
    pub parameter_extra: Box<str>,
    pub id: i32,
    pub scope: i32,
    pub combinable_parameter_index: i32,
    pub drop_limit_per_fight: i32,
    pub drop_limit_per_entity: i32,
    pub drop_limit_per_behavior: i32,
    pub source_type: i32,
    pub drop_carrier_check: i32,
}
impl ConfigAffixClassRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.affix_effects = cur.read_array_i32()?;
        self.scope = cur.read_i32()?;
        self.description = cur.read_str()?;
        self.detail_description = cur.read_str()?;
        self.combinable_parameter_index = cur.read_i32()?;
        self.parameter_abstract = cur.read_array_str()?;
        self.parameter_extra = cur.read_str()?;
        self.drop_limit_per_fight = cur.read_i32()?;
        self.drop_limit_per_entity = cur.read_i32()?;
        self.drop_limit_per_behavior = cur.read_i32()?;
        self.source_type = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.drop_carrier_check = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigAffixClassRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_affix_class_ChunkA0.bin`.
pub type ConfigAffixClassTable = BinTable<i32, ConfigAffixClassRow>;
/// Parses `config_affix_class_ChunkA0.bin` into a [`ConfigAffixClassTable`].
pub fn parse_config_affix_class(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAffixClassTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_affix_mapping_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigAffixMappingRow {
    pub affix: FConfigAffix,
    pub dync_params: Box<[Box<[f64]>]>,
    pub id: i32,
}
impl BinRow for ConfigAffixMappingRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.affix = FConfigAffix::read(cur)?;
        self.dync_params = cur.read_array_2d_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_affix_mapping_ChunkA0.bin`.
pub type ConfigAffixMappingTable = BinTable<i32, ConfigAffixMappingRow>;
/// Parses `config_affix_mapping_ChunkA0.bin` into a [`ConfigAffixMappingTable`].
pub fn parse_config_affix_mapping(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAffixMappingTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_affix_source_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAffixSourceRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub display_name: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigAffixSourceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.display_name = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_affix_source_ChunkA0.bin`.
pub type ConfigAffixSourceTable = BinTable<i32, ConfigAffixSourceRow>;
/// Parses `config_affix_source_ChunkA0.bin` into a [`ConfigAffixSourceTable`].
pub fn parse_config_affix_source(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAffixSourceTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_affix_tag_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAffixTagRow {
    pub text: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigAffixTagRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.text = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_affix_tag_ChunkA0.bin`.
pub type ConfigAffixTagTable = BinTable<i32, ConfigAffixTagRow>;
/// Parses `config_affix_tag_ChunkA0.bin` into a [`ConfigAffixTagTable`].
pub fn parse_config_affix_tag(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAffixTagTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_affix_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_affix" => Some(parse_config_affix(buf, scratch).map(|t| t.len())),
        "config_affix_blackboard_type" => {
            Some(parse_config_affix_blackboard_type(buf, scratch).map(|t| t.len()))
        }
        "config_affix_class" => {
            Some(parse_config_affix_class(buf, scratch).map(|t| t.len()))
        }
        "config_affix_mapping" => {
            Some(parse_config_affix_mapping(buf, scratch).map(|t| t.len()))
        }
        "config_affix_source" => {
            Some(parse_config_affix_source(buf, scratch).map(|t| t.len()))
        }
        "config_affix_tag" => Some(parse_config_affix_tag(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

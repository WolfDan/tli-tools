//! Auto-generated `.bin` configuration table types and parsers for the `talent` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
/// Row record for `config_talent_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigTalentRow {
    pub before_id: Box<[i32]>,
    pub position: Box<[f64]>,
    pub attr: Box<[FConfigAffix]>,
    pub icon: Box<str>,
    pub name: Box<str>,
    pub des_type2: Box<str>,
    pub des: Box<str>,
    pub tags: Box<[i32]>,
    pub server_affix: Box<str>,
    pub affix_key: Box<str>,
    pub id: i32,
    pub r#type: i32,
    pub accumulate_effect: i32,
    pub level_up_time: i32,
    pub need_points: i32,
    pub drop_weight: i32,
    pub drop_level: i32,
    pub group: i32,
    pub god: i32,
    pub build_weight: i32,
    pub enhance_level: i32,
    pub used: i32,
    pub hero_talent: i32,
    pub hero_id: i32,
}
impl ConfigTalentRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.accumulate_effect = cur.read_i32()?;
        self.level_up_time = cur.read_i32()?;
        self.need_points = cur.read_i32()?;
        self.before_id = cur.read_array_i32()?;
        self.position = cur.read_array_f64()?;
        self.attr = FConfigAffix::read_array(cur)?;
        self.icon = cur.read_str()?;
        self.name = cur.read_str()?;
        self.des_type2 = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.drop_weight = cur.read_i32()?;
        self.drop_level = cur.read_i32()?;
        self.group = cur.read_i32()?;
        self.god = cur.read_i32()?;
        self.tags = cur.read_array_i32()?;
        self.build_weight = cur.read_i32()?;
        self.enhance_level = cur.read_i32()?;
        self.server_affix = cur.read_str()?;
        self.used = cur.read_i32()?;
        self.hero_talent = cur.read_i32()?;
        self.hero_id = cur.read_i32()?;
        self.affix_key = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigTalentRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_talent_ChunkA0.bin`.
pub type ConfigTalentTable = BinTable<i32, ConfigTalentRow>;
/// Parses `config_talent_ChunkA0.bin` into a [`ConfigTalentTable`].
pub fn parse_config_talent(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTalentTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_talent_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_talent" => Some(parse_config_talent(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

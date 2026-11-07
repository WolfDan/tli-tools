//! Auto-generated `.bin` configuration table types and parsers for the `servant` domain.
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{FConfigI32I32, FConfigStrI32};
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
/// Row record for `config_servant_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantRow {
    pub name: Box<str>,
    pub des_promotion: Box<str>,
    pub extrapoint: Box<[FConfigI32I32]>,
    pub replaced_contract_point_ui: Box<[i32]>,
    pub fight_limit_id: Box<[i32]>,
    pub appear_pos: Box<[i32]>,
    pub icon_big: Box<str>,
    pub icon_pick_result: Box<str>,
    pub des_effect: Box<str>,
    pub promotion_types: Box<str>,
    pub id: i32,
    pub r#type: i32,
    pub install_category: i32,
    pub install_category_subtype: i32,
    pub rarity: i32,
    pub point: i32,
    pub recycle: FConfigI32I32,
    pub resource_id: i32,
    pub npc_id: i32,
    pub order: i32,
}
impl ConfigServantRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.r#type = cur.read_i32()?;
        self.install_category = cur.read_i32()?;
        self.install_category_subtype = cur.read_i32()?;
        self.rarity = cur.read_i32()?;
        self.point = cur.read_i32()?;
        self.des_promotion = cur.read_str()?;
        self.extrapoint = FConfigI32I32::read_array(cur)?;
        self.replaced_contract_point_ui = cur.read_array_i32()?;
        self.recycle = FConfigI32I32::read(cur)?;
        self.resource_id = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.npc_id = cur.read_i32()?;
        self.fight_limit_id = cur.read_array_i32()?;
        self.appear_pos = cur.read_array_i32()?;
        self.order = cur.read_i32()?;
        self.icon_big = cur.read_str()?;
        self.icon_pick_result = cur.read_str()?;
        self.des_effect = cur.read_str()?;
        self.promotion_types = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigServantRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_servant_ChunkA0.bin`.
pub type ConfigServantTable = BinTable<i32, ConfigServantRow>;
/// Parses `config_servant_ChunkA0.bin` into a [`ConfigServantTable`].
pub fn parse_config_servant(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigServantTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_servant_collect_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantCollectRow {
    pub item_type: Box<[i32]>,
    pub rarity: Box<[i32]>,
    pub id: i32,
    pub open_type: i32,
    pub collect_type: i32,
}
impl BinRow for ConfigServantCollectRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.open_type = cur.read_i32()?;
        self.collect_type = cur.read_i32()?;
        self.item_type = cur.read_array_i32()?;
        self.rarity = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_servant_collect_ChunkA0.bin`.
pub type ConfigServantCollectTable = BinTable<i32, ConfigServantCollectRow>;
/// Parses `config_servant_collect_ChunkA0.bin` into a [`ConfigServantCollectTable`].
pub fn parse_config_servant_collect(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigServantCollectTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_servant_fight_limit_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantFightLimitRow {
    pub params: Box<[i32]>,
    pub id: i32,
    pub r#type: i32,
}
impl BinRow for ConfigServantFightLimitRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.params = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_servant_fight_limit_ChunkA0.bin`.
pub type ConfigServantFightLimitTable = BinTable<i32, ConfigServantFightLimitRow>;
/// Parses `config_servant_fight_limit_ChunkA0.bin` into a [`ConfigServantFightLimitTable`].
pub fn parse_config_servant_fight_limit(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigServantFightLimitTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_servant_promotion_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantPromotionRow {
    pub cost: Box<[FConfigStrI32]>,
    pub id: i32,
    pub r#type: i32,
    pub star: i32,
}
impl BinRow for ConfigServantPromotionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.star = cur.read_i32()?;
        self.cost = FConfigStrI32::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_servant_promotion_ChunkA0.bin`.
pub type ConfigServantPromotionTable = BinTable<i32, ConfigServantPromotionRow>;
/// Parses `config_servant_promotion_ChunkA0.bin` into a [`ConfigServantPromotionTable`].
pub fn parse_config_servant_promotion(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigServantPromotionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_servant_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_servant" => Some(parse_config_servant(buf, scratch).map(|t| t.len())),
        "config_servant_collect" => {
            Some(parse_config_servant_collect(buf, scratch).map(|t| t.len()))
        }
        "config_servant_fight_limit" => {
            Some(parse_config_servant_fight_limit(buf, scratch).map(|t| t.len()))
        }
        "config_servant_promotion" => {
            Some(parse_config_servant_promotion(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

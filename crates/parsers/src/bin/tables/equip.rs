//! Auto-generated `.bin` configuration table types and parsers for the `equip` domain.
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_equip_slot_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEquipSlotRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigEquipSlotRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_equip_slot_ChunkA0.bin`.
pub type ConfigEquipSlotTable = BinTable<i32, ConfigEquipSlotRow>;
/// Parses `config_equip_slot_ChunkA0.bin` into a [`ConfigEquipSlotTable`].
pub fn parse_config_equip_slot(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigEquipSlotTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_forge_vague_config_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigForgeVagueConfigRow {
    pub type4: Box<[i32]>,
    pub item_level: Box<[i32]>,
    pub durability_interval: Box<[i32]>,
    pub item_consume: FxHashMap<i32, i32>,
    pub affix_num: FxHashMap<i32, i32>,
    pub rarity: Box<[i32]>,
    pub item_gold_rarity: Box<[i32]>,
    pub legendary_slot_num: Box<[i32]>,
    pub id: i32,
    pub lock_max: i32,
    pub lock_min: i32,
    pub lock_num: i32,
    pub ash_consume: i32,
    pub durability_consume: i32,
}
impl ConfigForgeVagueConfigRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.type4 = cur.read_array_i32()?;
        self.item_level = cur.read_array_i32()?;
        self.durability_interval = cur.read_array_i32()?;
        self.lock_max = cur.read_i32()?;
        self.lock_min = cur.read_i32()?;
        self.lock_num = cur.read_i32()?;
        self.item_consume = cur.read_map_i32_i32()?;
        self.ash_consume = cur.read_i32()?;
        self.durability_consume = cur.read_i32()?;
        self.affix_num = cur.read_map_i32_i32()?;
        self.rarity = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.item_gold_rarity = cur.read_array_i32()?;
        self.legendary_slot_num = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigForgeVagueConfigRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_forge_vague_config_ChunkA0.bin`.
pub type ConfigForgeVagueConfigTable = BinTable<i32, ConfigForgeVagueConfigRow>;
/// Parses `config_forge_vague_config_ChunkA0.bin` into a [`ConfigForgeVagueConfigTable`].
pub fn parse_config_forge_vague_config(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigForgeVagueConfigTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_weapon_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigWeaponRow {
    pub des: Box<str>,
    pub tags: Box<[i32]>,
    pub deputy_equip: Box<[i32]>,
    pub effect_type: Box<[i32]>,
    pub default_sockets: FxHashMap<i32, Box<str>>,
    pub group_name: Box<str>,
    pub id: i32,
    pub hand: i32,
    pub deputy: i32,
    pub melee: i32,
    pub ui_project_type2: i32,
    pub group: i32,
}
impl BinRow for ConfigWeaponRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.hand = cur.read_i32()?;
        self.deputy = cur.read_i32()?;
        self.melee = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.tags = cur.read_array_i32()?;
        self.deputy_equip = cur.read_array_i32()?;
        self.effect_type = cur.read_array_i32()?;
        self.default_sockets = cur.read_map_i32_str()?;
        self.ui_project_type2 = cur.read_i32()?;
        self.group_name = cur.read_str()?;
        self.group = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_weapon_ChunkA0.bin`.
pub type ConfigWeaponTable = BinTable<i32, ConfigWeaponRow>;
/// Parses `config_weapon_ChunkA0.bin` into a [`ConfigWeaponTable`].
pub fn parse_config_weapon(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigWeaponTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_equip_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_equip_slot" => {
            Some(parse_config_equip_slot(buf, scratch).map(|t| t.len()))
        }
        "config_forge_vague_config" => {
            Some(parse_config_forge_vague_config(buf, scratch).map(|t| t.len()))
        }
        "config_weapon" => Some(parse_config_weapon(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

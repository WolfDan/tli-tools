//! Auto-generated `.bin` configuration table types and parsers for the `mercenary` domain.
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::FConfigI32I32;
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_mercenary_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMercenaryRow {
    pub title: Box<str>,
    pub name: Box<str>,
    pub desc: Box<str>,
    pub init_skill: Box<[FConfigI32I32]>,
    pub active_skill_allow_tag: Box<[Box<[i32]>]>,
    pub active_skill_excluded_tag: Box<[Box<[i32]>]>,
    pub init_weapon: Box<[i32]>,
    pub id: i32,
    pub r#type: i32,
    pub rarity: i32,
    pub lv: i32,
    pub resource_id: i32,
    pub npc_id: i32,
    pub attr_id: i32,
    pub good_mod_num: i32,
    pub bad_mod_num: i32,
    pub init_skill_lv: i32,
    pub active_skill_slot_num: i32,
}
impl ConfigMercenaryRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.rarity = cur.read_i32()?;
        self.title = cur.read_str()?;
        self.lv = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.resource_id = cur.read_i32()?;
        self.npc_id = cur.read_i32()?;
        self.attr_id = cur.read_i32()?;
        self.good_mod_num = cur.read_i32()?;
        self.bad_mod_num = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.init_skill = FConfigI32I32::read_array(cur)?;
        self.init_skill_lv = cur.read_i32()?;
        self.active_skill_slot_num = cur.read_i32()?;
        self.active_skill_allow_tag = cur.read_array_2d_i32()?;
        self.active_skill_excluded_tag = cur.read_array_2d_i32()?;
        self.init_weapon = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMercenaryRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_mercenary_ChunkA0.bin`.
pub type ConfigMercenaryTable = BinTable<i32, ConfigMercenaryRow>;
/// Parses `config_mercenary_ChunkA0.bin` into a [`ConfigMercenaryTable`].
pub fn parse_config_mercenary(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMercenaryTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_mercenary_attr_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMercenaryAttrRow {
    pub name: Box<str>,
    pub hp: f64,
    pub mana: f64,
    pub shield: f64,
    pub damage: f64,
    pub armour: f64,
    pub hit: f64,
    pub dodge: f64,
    pub attack_block: f64,
    pub spell_block: f64,
    pub fire_res: f64,
    pub ice_res: f64,
    pub light_res: f64,
    pub chaos_res: f64,
    pub cri_rate: f64,
    pub cri_dmg: f64,
    pub hp_recovery: f64,
    pub mana_recovery: f64,
    pub move_speed: f64,
    pub defend_range: f64,
    pub teleport_range: f64,
    pub attributes: FxHashMap<i32, Box<[f64]>>,
    pub id: i32,
}
impl ConfigMercenaryAttrRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.hp = cur.read_f64()?;
        self.mana = cur.read_f64()?;
        self.shield = cur.read_f64()?;
        self.damage = cur.read_f64()?;
        self.armour = cur.read_f64()?;
        self.hit = cur.read_f64()?;
        self.dodge = cur.read_f64()?;
        self.attack_block = cur.read_f64()?;
        self.spell_block = cur.read_f64()?;
        self.fire_res = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.ice_res = cur.read_f64()?;
        self.light_res = cur.read_f64()?;
        self.chaos_res = cur.read_f64()?;
        self.cri_rate = cur.read_f64()?;
        self.cri_dmg = cur.read_f64()?;
        self.hp_recovery = cur.read_f64()?;
        self.mana_recovery = cur.read_f64()?;
        self.move_speed = cur.read_f64()?;
        self.defend_range = cur.read_f64()?;
        self.teleport_range = cur.read_f64()?;
        self.attributes = cur.read_map_i32_array_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigMercenaryAttrRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_mercenary_attr_ChunkA0.bin`.
pub type ConfigMercenaryAttrTable = BinTable<i32, ConfigMercenaryAttrRow>;
/// Parses `config_mercenary_attr_ChunkA0.bin` into a [`ConfigMercenaryAttrTable`].
pub fn parse_config_mercenary_attr(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMercenaryAttrTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_mercenary_resource_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMercenaryResourceRow {
    pub icon: Box<str>,
    pub init_weapon: Box<[i32]>,
    pub r#type: i32,
    pub index: i32,
    pub gameplay_show_resources_id: i32,
    pub npc_id: i32,
}
impl BinRow for ConfigMercenaryResourceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.r#type = cur.read_i32()?;
        self.index = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.gameplay_show_resources_id = cur.read_i32()?;
        self.npc_id = cur.read_i32()?;
        self.init_weapon = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_mercenary_resource_ChunkA0.bin`.
pub type ConfigMercenaryResourceTable = BinTable<(i32, i32), ConfigMercenaryResourceRow>;
/// Parses `config_mercenary_resource_ChunkA0.bin` into a [`ConfigMercenaryResourceTable`].
pub fn parse_config_mercenary_resource(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMercenaryResourceTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_mercenary_skill_config_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMercenarySkillConfigRow {
    pub modifier_tag_create: Box<[i32]>,
    pub forge_cost: FxHashMap<i32, i32>,
    pub modifier_tag_forge: Box<[i32]>,
    pub id: i32,
    pub modifier_num_create: i32,
    pub modifier_level_create: i32,
    pub forge_win_chance: i32,
    pub modifier_num_forge: i32,
    pub modifier_level_forge: i32,
}
impl BinRow for ConfigMercenarySkillConfigRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.modifier_num_create = cur.read_i32()?;
        self.modifier_tag_create = cur.read_array_i32()?;
        self.modifier_level_create = cur.read_i32()?;
        self.forge_cost = cur.read_map_i32_i32()?;
        self.forge_win_chance = cur.read_i32()?;
        self.modifier_num_forge = cur.read_i32()?;
        self.modifier_tag_forge = cur.read_array_i32()?;
        self.modifier_level_forge = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_mercenary_skill_config_ChunkA0.bin`.
pub type ConfigMercenarySkillConfigTable = BinTable<i32, ConfigMercenarySkillConfigRow>;
/// Parses `config_mercenary_skill_config_ChunkA0.bin` into a [`ConfigMercenarySkillConfigTable`].
pub fn parse_config_mercenary_skill_config(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMercenarySkillConfigTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_mercenary_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_mercenary" => Some(parse_config_mercenary(buf, scratch).map(|t| t.len())),
        "config_mercenary_attr" => {
            Some(parse_config_mercenary_attr(buf, scratch).map(|t| t.len()))
        }
        "config_mercenary_resource" => {
            Some(parse_config_mercenary_resource(buf, scratch).map(|t| t.len()))
        }
        "config_mercenary_skill_config" => {
            Some(parse_config_mercenary_skill_config(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

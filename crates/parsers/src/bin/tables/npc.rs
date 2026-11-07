//! Auto-generated `.bin` configuration table types and parsers for the `npc` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{
    FConfigI32ArrayI32, FConfigI32I32, FConfigStrArrayI32,
};
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_interactive_icon_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigInteractiveIconRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub name: Box<str>,
    pub tale_icon: Box<str>,
    pub othertale_icon: Box<str>,
    pub interactive_icon: Box<str>,
    pub head_icon: Box<str>,
    pub name_offset: Box<[i32]>,
    pub id: i32,
    pub minimap_id: i32,
}
impl BinRow for ConfigInteractiveIconRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.name = cur.read_str()?;
        self.tale_icon = cur.read_str()?;
        self.othertale_icon = cur.read_str()?;
        self.interactive_icon = cur.read_str()?;
        self.head_icon = cur.read_str()?;
        self.minimap_id = cur.read_i32()?;
        self.name_offset = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_interactive_icon_ChunkA0.bin`.
pub type ConfigInteractiveIconTable = BinTable<i32, ConfigInteractiveIconRow>;
/// Parses `config_interactive_icon_ChunkA0.bin` into a [`ConfigInteractiveIconTable`].
pub fn parse_config_interactive_icon(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigInteractiveIconTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_npc_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigNpcRow {
    pub name: Box<str>,
    pub res_weapon_list: Box<[FConfigI32ArrayI32]>,
    pub vfx: Box<[FConfigStrArrayI32]>,
    pub skill: Box<[i32]>,
    pub affix: Box<[FConfigAffix]>,
    pub ai_brain: Box<str>,
    pub death_effect: Box<[i32]>,
    pub ai_normal_brain: Box<str>,
    pub note: Box<str>,
    pub audio_switch: Box<str>,
    pub drop_delay: f64,
    pub life_timer: f64,
    pub enemy_icon: Box<str>,
    pub drop_affix: Box<[FConfigAffix]>,
    pub attributes: FxHashMap<i32, Box<[f64]>>,
    pub id: i32,
    pub resource_id: i32,
    pub res_material_set_id: i32,
    pub radius: i32,
    pub hit_radius: i32,
    pub is_summon: i32,
    pub r#type: i32,
    pub move_speed: i32,
    pub ai_config_id: i32,
    pub defend_range: i32,
    pub chase_range: i32,
    pub npc_attr_id: i32,
    pub enter_scene_type: i32,
    pub victim_snd_type: i32,
    pub range_type: i32,
    pub minimap_icon_id: i32,
    pub appear_skill_stone_id: i32,
    pub enable_boss_bgm: i32,
    pub life_timer_skill: i32,
    pub skill_season_vfx: i32,
    pub col_36: i32,
    pub servant_scheme_type: i32,
    pub show_minimap_icon: bool,
    pub infinite_hp: bool,
}
impl ConfigNpcRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.resource_id = cur.read_i32()?;
        self.res_material_set_id = cur.read_i32()?;
        self.res_weapon_list = FConfigI32ArrayI32::read_array(cur)?;
        self.vfx = FConfigStrArrayI32::read_array(cur)?;
        self.radius = cur.read_i32()?;
        self.hit_radius = cur.read_i32()?;
        self.is_summon = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.move_speed = cur.read_i32()?;
        self.skill = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.affix = FConfigAffix::read_array(cur)?;
        self.ai_brain = cur.read_str()?;
        self.death_effect = cur.read_array_i32()?;
        self.ai_normal_brain = cur.read_str()?;
        self.ai_config_id = cur.read_i32()?;
        self.defend_range = cur.read_i32()?;
        self.chase_range = cur.read_i32()?;
        self.npc_attr_id = cur.read_i32()?;
        self.note = cur.read_str()?;
        self.enter_scene_type = cur.read_i32()?;
        self.victim_snd_type = cur.read_i32()?;
        self.range_type = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.audio_switch = cur.read_str()?;
        self.minimap_icon_id = cur.read_i32()?;
        self.appear_skill_stone_id = cur.read_i32()?;
        self.enable_boss_bgm = cur.read_i32()?;
        self.drop_delay = cur.read_f64()?;
        self.life_timer = cur.read_f64()?;
        self.life_timer_skill = cur.read_i32()?;
        self.enemy_icon = cur.read_str()?;
        self.show_minimap_icon = cur.read_bool()?;
        self.infinite_hp = cur.read_bool()?;
        self.skill_season_vfx = cur.read_i32()?;
        self.drop_affix = FConfigAffix::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.col_36 = cur.read_i32()?;
        self.servant_scheme_type = cur.read_i32()?;
        self.attributes = cur.read_map_i32_array_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigNpcRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_npc_ChunkA0.bin`.
pub type ConfigNpcTable = BinTable<i32, ConfigNpcRow>;
/// Parses `config_npc_ChunkA0.bin` into a [`ConfigNpcTable`].
pub fn parse_config_npc(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigNpcTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_npc_affix_whitelist_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigNpcAffixWhitelistRow {
    pub affixes: Box<[i32]>,
    pub id: i32,
}
impl BinRow for ConfigNpcAffixWhitelistRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.affixes = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_npc_affix_whitelist_ChunkA0.bin`.
pub type ConfigNpcAffixWhitelistTable = BinTable<i32, ConfigNpcAffixWhitelistRow>;
/// Parses `config_npc_affix_whitelist_ChunkA0.bin` into a [`ConfigNpcAffixWhitelistTable`].
pub fn parse_config_npc_affix_whitelist(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigNpcAffixWhitelistTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_npc_amplification_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigNpcAmplificationRow {
    pub hp: f64,
    pub damage: f64,
    pub hit: f64,
    pub dodge: f64,
    pub hp_rarity: Box<[f64]>,
    pub damage_rarity: Box<[f64]>,
    pub shield: f64,
    pub armour: f64,
    pub cri_rate: f64,
    pub attack_block: f64,
    pub spell_block: f64,
    pub fire_res: f64,
    pub ice_res: f64,
    pub light_res: f64,
    pub chaos_res: f64,
    pub mana: f64,
    pub attributes: FxHashMap<i32, Box<[f64]>>,
    pub id: i32,
}
impl ConfigNpcAmplificationRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.hp = cur.read_f64()?;
        self.damage = cur.read_f64()?;
        self.hit = cur.read_f64()?;
        self.dodge = cur.read_f64()?;
        self.hp_rarity = cur.read_array_f64()?;
        self.damage_rarity = cur.read_array_f64()?;
        self.shield = cur.read_f64()?;
        self.armour = cur.read_f64()?;
        self.cri_rate = cur.read_f64()?;
        self.attack_block = cur.read_f64()?;
        self.spell_block = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.fire_res = cur.read_f64()?;
        self.ice_res = cur.read_f64()?;
        self.light_res = cur.read_f64()?;
        self.chaos_res = cur.read_f64()?;
        self.mana = cur.read_f64()?;
        self.attributes = cur.read_map_i32_array_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigNpcAmplificationRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_npc_amplification_ChunkA0.bin`.
pub type ConfigNpcAmplificationTable = BinTable<i32, ConfigNpcAmplificationRow>;
/// Parses `config_npc_amplification_ChunkA0.bin` into a [`ConfigNpcAmplificationTable`].
pub fn parse_config_npc_amplification(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigNpcAmplificationTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_npc_attr_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigNpcAttrRow {
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
    pub attributes: FxHashMap<i32, Box<[f64]>>,
    pub id: i32,
    pub maxdrop: i32,
    pub npc_drop_id: i32,
}
impl ConfigNpcAttrRow {
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
        self.maxdrop = cur.read_i32()?;
        self.npc_drop_id = cur.read_i32()?;
        self.attributes = cur.read_map_i32_array_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigNpcAttrRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_npc_attr_ChunkA0.bin`.
pub type ConfigNpcAttrTable = BinTable<i32, ConfigNpcAttrRow>;
/// Parses `config_npc_attr_ChunkA0.bin` into a [`ConfigNpcAttrTable`].
pub fn parse_config_npc_attr(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigNpcAttrTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_npc_bigmap_difficulty_amplification_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigNpcBigmapDifficultyAmplificationRow {
    pub hp: f64,
    pub damage: f64,
    pub hit: f64,
    pub dodge: f64,
    pub hp_rarity: Box<[f64]>,
    pub damage_rarity: Box<[f64]>,
    pub shield: f64,
    pub armour: f64,
    pub cri_rate: f64,
    pub attack_block: f64,
    pub spell_block: f64,
    pub fire_res: f64,
    pub ice_res: f64,
    pub light_res: f64,
    pub chaos_res: f64,
    pub mana: f64,
    pub attributes: FxHashMap<i32, Box<[f64]>>,
    pub id: i32,
}
impl ConfigNpcBigmapDifficultyAmplificationRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.hp = cur.read_f64()?;
        self.damage = cur.read_f64()?;
        self.hit = cur.read_f64()?;
        self.dodge = cur.read_f64()?;
        self.hp_rarity = cur.read_array_f64()?;
        self.damage_rarity = cur.read_array_f64()?;
        self.shield = cur.read_f64()?;
        self.armour = cur.read_f64()?;
        self.cri_rate = cur.read_f64()?;
        self.attack_block = cur.read_f64()?;
        self.spell_block = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.fire_res = cur.read_f64()?;
        self.ice_res = cur.read_f64()?;
        self.light_res = cur.read_f64()?;
        self.chaos_res = cur.read_f64()?;
        self.mana = cur.read_f64()?;
        self.attributes = cur.read_map_i32_array_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigNpcBigmapDifficultyAmplificationRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_npc_bigmap_difficulty_amplification_ChunkA0.bin`.
pub type ConfigNpcBigmapDifficultyAmplificationTable = BinTable<
    i32,
    ConfigNpcBigmapDifficultyAmplificationRow,
>;
/// Parses `config_npc_bigmap_difficulty_amplification_ChunkA0.bin` into a [`ConfigNpcBigmapDifficultyAmplificationTable`].
pub fn parse_config_npc_bigmap_difficulty_amplification(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigNpcBigmapDifficultyAmplificationTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_npc_drop_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigNpcDropRow {
    pub level: Box<[i32]>,
    pub classification: Box<[FConfigI32I32]>,
    pub id: i32,
    pub npc_drop_id: i32,
    pub r#type: i32,
}
impl BinRow for ConfigNpcDropRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.npc_drop_id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.level = cur.read_array_i32()?;
        self.classification = FConfigI32I32::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_npc_drop_ChunkA0.bin`.
pub type ConfigNpcDropTable = BinTable<i32, ConfigNpcDropRow>;
/// Parses `config_npc_drop_ChunkA0.bin` into a [`ConfigNpcDropTable`].
pub fn parse_config_npc_drop(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigNpcDropTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_npc_lv_attr_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigNpcLvAttrRow {
    pub attributes: FxHashMap<i32, Box<[f64]>>,
    pub level: i32,
    pub hit: i32,
    pub dodge: i32,
    pub physical_damage: i32,
}
impl BinRow for ConfigNpcLvAttrRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.level = cur.read_i32()?;
        self.hit = cur.read_i32()?;
        self.dodge = cur.read_i32()?;
        self.physical_damage = cur.read_i32()?;
        self.attributes = cur.read_map_i32_array_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_npc_lv_attr_ChunkA0.bin`.
pub type ConfigNpcLvAttrTable = BinTable<i32, ConfigNpcLvAttrRow>;
/// Parses `config_npc_lv_attr_ChunkA0.bin` into a [`ConfigNpcLvAttrTable`].
pub fn parse_config_npc_lv_attr(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigNpcLvAttrTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_npc_resource_set_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigNpcResourceSetRow {
    pub resource_id_list: Box<[i32]>,
    pub npc_id: i32,
    pub set_id: i32,
}
impl BinRow for ConfigNpcResourceSetRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.npc_id = cur.read_i32()?;
        self.set_id = cur.read_i32()?;
        self.resource_id_list = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_npc_resource_set_ChunkA0.bin`.
pub type ConfigNpcResourceSetTable = BinTable<(i32, i32), ConfigNpcResourceSetRow>;
/// Parses `config_npc_resource_set_ChunkA0.bin` into a [`ConfigNpcResourceSetTable`].
pub fn parse_config_npc_resource_set(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigNpcResourceSetTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_npc_view_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigNpcViewRow {
    pub name: Box<str>,
    pub title: Box<str>,
    pub size: Box<[i32]>,
    pub res_weapon_list: Box<[FConfigI32ArrayI32]>,
    pub npc_tool_des: Box<str>,
    pub id: i32,
    pub res_id: i32,
    pub name_height: i32,
    pub hide_blood_bar: i32,
    pub invincible: i32,
    pub tale_id: i32,
}
impl BinRow for ConfigNpcViewRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.res_id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.title = cur.read_str()?;
        self.name_height = cur.read_i32()?;
        self.size = cur.read_array_i32()?;
        self.hide_blood_bar = cur.read_i32()?;
        self.invincible = cur.read_i32()?;
        self.res_weapon_list = FConfigI32ArrayI32::read_array(cur)?;
        self.tale_id = cur.read_i32()?;
        self.npc_tool_des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_npc_view_ChunkA0.bin`.
pub type ConfigNpcViewTable = BinTable<i32, ConfigNpcViewRow>;
/// Parses `config_npc_view_ChunkA0.bin` into a [`ConfigNpcViewTable`].
pub fn parse_config_npc_view(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigNpcViewTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_npc_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_interactive_icon" => {
            Some(parse_config_interactive_icon(buf, scratch).map(|t| t.len()))
        }
        "config_npc" => Some(parse_config_npc(buf, scratch).map(|t| t.len())),
        "config_npc_affix_whitelist" => {
            Some(parse_config_npc_affix_whitelist(buf, scratch).map(|t| t.len()))
        }
        "config_npc_amplification" => {
            Some(parse_config_npc_amplification(buf, scratch).map(|t| t.len()))
        }
        "config_npc_attr" => Some(parse_config_npc_attr(buf, scratch).map(|t| t.len())),
        "config_npc_bigmap_difficulty_amplification" => {
            Some(
                parse_config_npc_bigmap_difficulty_amplification(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_npc_drop" => Some(parse_config_npc_drop(buf, scratch).map(|t| t.len())),
        "config_npc_lv_attr" => {
            Some(parse_config_npc_lv_attr(buf, scratch).map(|t| t.len()))
        }
        "config_npc_resource_set" => {
            Some(parse_config_npc_resource_set(buf, scratch).map(|t| t.len()))
        }
        "config_npc_view" => Some(parse_config_npc_view(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

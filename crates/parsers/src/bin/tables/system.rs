//! Auto-generated `.bin` configuration table types and parsers for the `system` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{
    FConfigI32Array2dI32I32, FConfigI32ArrayStr, FConfigI32I32, FConfigI32Str,
    FConfigStrI32,
};
use crate::bin::types::{BinTable, VecDim};
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_Steam_Friend_RichText_Map_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSteamFriendRichTextMapRow {
    pub desc: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigSteamFriendRichTextMapRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_Steam_Friend_RichText_Map_ChunkA0.bin`.
pub type ConfigSteamFriendRichTextMapTable = BinTable<
    i32,
    ConfigSteamFriendRichTextMapRow,
>;
/// Parses `config_Steam_Friend_RichText_Map_ChunkA0.bin` into a [`ConfigSteamFriendRichTextMapTable`].
pub fn parse_config_steam_friend_rich_text_map(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSteamFriendRichTextMapTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_accumulated_value_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigAccumulatedValueRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub change_interval: f64,
    pub reduce_interval: f64,
    pub max_reached_duration: f64,
    pub value_show_vfx_id: Box<[FConfigI32I32]>,
    pub base_affix: Box<[FConfigAffix]>,
    pub id: i32,
    pub reduce_type: i32,
    pub max_value: i32,
}
impl BinRow for ConfigAccumulatedValueRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.change_interval = cur.read_f64()?;
        self.reduce_type = cur.read_i32()?;
        self.reduce_interval = cur.read_f64()?;
        self.max_value = cur.read_i32()?;
        self.max_reached_duration = cur.read_f64()?;
        self.value_show_vfx_id = FConfigI32I32::read_array(cur)?;
        self.base_affix = FConfigAffix::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_accumulated_value_ChunkA0.bin`.
pub type ConfigAccumulatedValueTable = BinTable<i32, ConfigAccumulatedValueRow>;
/// Parses `config_accumulated_value_ChunkA0.bin` into a [`ConfigAccumulatedValueTable`].
pub fn parse_config_accumulated_value(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAccumulatedValueTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_attr_tag_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAttrTagRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigAttrTagRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_attr_tag_ChunkA0.bin`.
pub type ConfigAttrTagTable = BinTable<i32, ConfigAttrTagRow>;
/// Parses `config_attr_tag_ChunkA0.bin` into a [`ConfigAttrTagTable`].
pub fn parse_config_attr_tag(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAttrTagTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_c_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCRow {
    pub r#type: Box<str>,
    pub param: Box<str>,
    pub sys: Box<str>,
    pub intro: Box<str>,
    pub intro7: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigCRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_str()?;
        self.param = cur.read_str()?;
        self.sys = cur.read_str()?;
        self.intro = cur.read_str()?;
        self.intro7 = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_c_ChunkA0.bin`.
pub type ConfigCTable = BinTable<i32, ConfigCRow>;
/// Parses `config_c_ChunkA0.bin` into a [`ConfigCTable`].
pub fn parse_config_c(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_climbtower_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigClimbtowerRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigClimbtowerRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_climbtower_ChunkA0.bin`.
pub type ConfigClimbtowerTable = BinTable<i32, ConfigClimbtowerRow>;
/// Parses `config_climbtower_ChunkA0.bin` into a [`ConfigClimbtowerTable`].
pub fn parse_config_climbtower(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigClimbtowerTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_collector_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigCollectorRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub max_add_value: f64,
    pub id: i32,
}
impl BinRow for ConfigCollectorRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.max_add_value = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_collector_ChunkA0.bin`.
pub type ConfigCollectorTable = BinTable<i32, ConfigCollectorRow>;
/// Parses `config_collector_ChunkA0.bin` into a [`ConfigCollectorTable`].
pub fn parse_config_collector(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCollectorTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_collectorTask_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCollectorTaskRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigCollectorTaskRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_collectorTask_ChunkA0.bin`.
pub type ConfigCollectorTaskTable = BinTable<i32, ConfigCollectorTaskRow>;
/// Parses `config_collectorTask_ChunkA0.bin` into a [`ConfigCollectorTaskTable`].
pub fn parse_config_collector_task(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCollectorTaskTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_cond_event_task_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCondEventTaskRow {
    pub args: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigCondEventTaskRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.args = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_cond_event_task_ChunkA0.bin`.
pub type ConfigCondEventTaskTable = BinTable<i32, ConfigCondEventTaskRow>;
/// Parses `config_cond_event_task_ChunkA0.bin` into a [`ConfigCondEventTaskTable`].
pub fn parse_config_cond_event_task(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCondEventTaskTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_contractPoint_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigContractPointRow {
    pub affix: FConfigAffix,
    pub id: i32,
    pub prev_id: i32,
    pub area: i32,
    pub r#type: i32,
    pub position: i32,
    pub order_in_area: i32,
    pub cost: i32,
    pub contract_point_ui: i32,
    pub drop_type_hide: bool,
}
impl BinRow for ConfigContractPointRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.prev_id = cur.read_i32()?;
        self.area = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.position = cur.read_i32()?;
        self.order_in_area = cur.read_i32()?;
        self.cost = cur.read_i32()?;
        self.affix = FConfigAffix::read(cur)?;
        self.contract_point_ui = cur.read_i32()?;
        self.drop_type_hide = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_contractPoint_ChunkA0.bin`.
pub type ConfigContractPointTable = BinTable<i32, ConfigContractPointRow>;
/// Parses `config_contractPoint_ChunkA0.bin` into a [`ConfigContractPointTable`].
pub fn parse_config_contract_point(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigContractPointTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_cost_level_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCostLevelRow {
    pub id: i32,
    pub cost: i32,
}
impl BinRow for ConfigCostLevelRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.cost = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_cost_level_ChunkA0.bin`.
pub type ConfigCostLevelTable = BinTable<i32, ConfigCostLevelRow>;
/// Parses `config_cost_level_ChunkA0.bin` into a [`ConfigCostLevelTable`].
pub fn parse_config_cost_level(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCostLevelTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_customized_item_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCustomizedItemRow {
    pub level: Box<[i32]>,
    pub cost: Box<[i32]>,
    pub customized_mods: Box<[i32]>,
    pub filter: FxHashMap<Box<str>, Box<[i32]>>,
    pub legendary_slot_num: Box<[FConfigI32I32]>,
    pub legendary_random_affix_num: Box<[FConfigI32I32]>,
    pub customized_talent_drop: Box<[i32]>,
    pub filter_talent: FxHashMap<Box<str>, Box<[i32]>>,
    pub talent_block_base_talent_rule: Box<[FConfigI32I32]>,
    pub talent_block_slot_rule: Box<[FConfigI32I32]>,
    pub mods_sequence: Box<[i32]>,
    pub customized_base_mods: Box<[i32]>,
    pub id: i32,
    pub base_id: i32,
    pub identified: i32,
    pub rarity: i32,
    pub mods_num: i32,
    pub corrosion_type: i32,
    pub slot_num: i32,
    pub full_roll: i32,
    pub upper_customized: i32,
    pub deal: i32,
    pub base_mod_num: i32,
    pub col_23: i32,
    pub col_24: i32,
}
impl ConfigCustomizedItemRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.base_id = cur.read_i32()?;
        self.level = cur.read_array_i32()?;
        self.cost = cur.read_array_i32()?;
        self.identified = cur.read_i32()?;
        self.rarity = cur.read_i32()?;
        self.mods_num = cur.read_i32()?;
        self.customized_mods = cur.read_array_i32()?;
        self.filter = cur.read_map_str_array_i32()?;
        self.legendary_slot_num = FConfigI32I32::read_array(cur)?;
        self.legendary_random_affix_num = FConfigI32I32::read_array(cur)?;
        self.corrosion_type = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.customized_talent_drop = cur.read_array_i32()?;
        self.filter_talent = cur.read_map_str_array_i32()?;
        self.talent_block_base_talent_rule = FConfigI32I32::read_array(cur)?;
        self.talent_block_slot_rule = FConfigI32I32::read_array(cur)?;
        self.slot_num = cur.read_i32()?;
        self.mods_sequence = cur.read_array_i32()?;
        self.full_roll = cur.read_i32()?;
        self.upper_customized = cur.read_i32()?;
        self.deal = cur.read_i32()?;
        self.base_mod_num = cur.read_i32()?;
        self.customized_base_mods = cur.read_array_i32()?;
        self.col_23 = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.col_24 = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigCustomizedItemRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_customized_item_ChunkA0.bin`.
pub type ConfigCustomizedItemTable = BinTable<i32, ConfigCustomizedItemRow>;
/// Parses `config_customized_item_ChunkA0.bin` into a [`ConfigCustomizedItemTable`].
pub fn parse_config_customized_item(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCustomizedItemTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_death_anim_drop_delay_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigDeathAnimDropDelayRow {
    pub drop_delay_death_anim_play_time_ratio: FxHashMap<Box<str>, f64>,
    pub id: i32,
}
impl BinRow for ConfigDeathAnimDropDelayRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.drop_delay_death_anim_play_time_ratio = cur.read_map_str_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_death_anim_drop_delay_ChunkA0.bin`.
pub type ConfigDeathAnimDropDelayTable = BinTable<i32, ConfigDeathAnimDropDelayRow>;
/// Parses `config_death_anim_drop_delay_ChunkA0.bin` into a [`ConfigDeathAnimDropDelayTable`].
pub fn parse_config_death_anim_drop_delay(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDeathAnimDropDelayTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_death_anim_vfx_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigDeathAnimVfxRow {
    pub anim_bp_res: Box<[FConfigStrI32]>,
    pub vfx: Box<[i32]>,
    pub vfx_on_stop: Box<[i32]>,
    pub delay_freeze_pose_on_anim_done: f64,
    pub default_drop_delay: f64,
    pub stop_res_vfxs_on_killed: Box<str>,
    pub stop_res_vfxs_on_anim_done: Box<str>,
    pub hide_mesh_on_stop_delay: f64,
    pub id: i32,
    pub ragdoll: i32,
    pub hide_mesh: i32,
    pub hide_mesh_on_stop: i32,
    pub sink_into_ground: i32,
    pub turn_toward_causer: bool,
    pub align_stopvfx_anim: bool,
}
impl ConfigDeathAnimVfxRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.anim_bp_res = FConfigStrI32::read_array(cur)?;
        self.turn_toward_causer = cur.read_bool()?;
        self.vfx = cur.read_array_i32()?;
        self.vfx_on_stop = cur.read_array_i32()?;
        self.align_stopvfx_anim = cur.read_bool()?;
        self.ragdoll = cur.read_i32()?;
        self.hide_mesh = cur.read_i32()?;
        self.hide_mesh_on_stop = cur.read_i32()?;
        self.sink_into_ground = cur.read_i32()?;
        self.delay_freeze_pose_on_anim_done = cur.read_f64()?;
        self.default_drop_delay = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.stop_res_vfxs_on_killed = cur.read_str()?;
        self.stop_res_vfxs_on_anim_done = cur.read_str()?;
        self.hide_mesh_on_stop_delay = cur.read_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigDeathAnimVfxRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_death_anim_vfx_ChunkA0.bin`.
pub type ConfigDeathAnimVfxTable = BinTable<i32, ConfigDeathAnimVfxRow>;
/// Parses `config_death_anim_vfx_ChunkA0.bin` into a [`ConfigDeathAnimVfxTable`].
pub fn parse_config_death_anim_vfx(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDeathAnimVfxTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_death_cause_filter_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDeathCauseFilterRow {
    pub desc: Box<str>,
    pub damage_type_cond: Box<[i32]>,
    pub char_state_cond: Box<[i32]>,
    pub kill_tag_cond: Box<[i32]>,
    pub death_skill_tag_cond: Box<[i32]>,
    pub enum_kill_impulse_type_cond: Box<[i32]>,
    pub id: i32,
    pub crit_cond: i32,
    pub explode_on_death_cond: i32,
}
impl BinRow for ConfigDeathCauseFilterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.crit_cond = cur.read_i32()?;
        self.damage_type_cond = cur.read_array_i32()?;
        self.char_state_cond = cur.read_array_i32()?;
        self.kill_tag_cond = cur.read_array_i32()?;
        self.death_skill_tag_cond = cur.read_array_i32()?;
        self.enum_kill_impulse_type_cond = cur.read_array_i32()?;
        self.explode_on_death_cond = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_death_cause_filter_ChunkA0.bin`.
pub type ConfigDeathCauseFilterTable = BinTable<i32, ConfigDeathCauseFilterRow>;
/// Parses `config_death_cause_filter_ChunkA0.bin` into a [`ConfigDeathCauseFilterTable`].
pub fn parse_config_death_cause_filter(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDeathCauseFilterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_death_effect_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDeathEffectRow {
    pub death_anim_vfx: Box<[FConfigI32Array2dI32I32]>,
    pub id: i32,
}
impl BinRow for ConfigDeathEffectRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.death_anim_vfx = FConfigI32Array2dI32I32::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_death_effect_ChunkA0.bin`.
pub type ConfigDeathEffectTable = BinTable<i32, ConfigDeathEffectRow>;
/// Parses `config_death_effect_ChunkA0.bin` into a [`ConfigDeathEffectTable`].
pub fn parse_config_death_effect(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDeathEffectTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_death_info_show_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDeathInfoShowRow {
    pub args: Box<str>,
    pub desc: Box<str>,
    pub id: i32,
    pub true_redirect: i32,
    pub false_redirect: i32,
}
impl BinRow for ConfigDeathInfoShowRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.args = cur.read_str()?;
        self.true_redirect = cur.read_i32()?;
        self.false_redirect = cur.read_i32()?;
        self.desc = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_death_info_show_ChunkA0.bin`.
pub type ConfigDeathInfoShowTable = BinTable<i32, ConfigDeathInfoShowRow>;
/// Parses `config_death_info_show_ChunkA0.bin` into a [`ConfigDeathInfoShowTable`].
pub fn parse_config_death_info_show(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDeathInfoShowTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ember_formula_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEmberFormulaRow {
    pub formula: Box<str>,
    pub name: Box<str>,
    pub position: Box<str>,
    pub prefix1: Box<[i32]>,
    pub prefix2: Box<[i32]>,
    pub prefix3: Box<[i32]>,
    pub suffix1: Box<[i32]>,
    pub suffix2: Box<[i32]>,
    pub suffix3: Box<[i32]>,
    pub icon: Box<str>,
    pub id: i32,
    pub level: i32,
    pub unlock: FConfigI32I32,
}
impl ConfigEmberFormulaRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.formula = cur.read_str()?;
        self.name = cur.read_str()?;
        self.position = cur.read_str()?;
        self.level = cur.read_i32()?;
        self.unlock = FConfigI32I32::read(cur)?;
        self.prefix1 = cur.read_array_i32()?;
        self.prefix2 = cur.read_array_i32()?;
        self.prefix3 = cur.read_array_i32()?;
        self.suffix1 = cur.read_array_i32()?;
        self.suffix2 = cur.read_array_i32()?;
        self.suffix3 = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.icon = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigEmberFormulaRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ember_formula_ChunkA0.bin`.
pub type ConfigEmberFormulaTable = BinTable<i32, ConfigEmberFormulaRow>;
/// Parses `config_ember_formula_ChunkA0.bin` into a [`ConfigEmberFormulaTable`].
pub fn parse_config_ember_formula(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigEmberFormulaTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ember_formula_main_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEmberFormulaMainRow {
    pub name: Box<str>,
    pub position: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigEmberFormulaMainRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.position = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ember_formula_main_ChunkA0.bin`.
pub type ConfigEmberFormulaMainTable = BinTable<i32, ConfigEmberFormulaMainRow>;
/// Parses `config_ember_formula_main_ChunkA0.bin` into a [`ConfigEmberFormulaMainTable`].
pub fn parse_config_ember_formula_main(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigEmberFormulaMainTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ember_formula_position_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEmberFormulaPositionRow {
    pub name: Box<str>,
    pub position: Box<str>,
    pub icon: Box<str>,
    pub icon_type: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigEmberFormulaPositionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.position = cur.read_str()?;
        self.icon = cur.read_str()?;
        self.icon_type = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ember_formula_position_ChunkA0.bin`.
pub type ConfigEmberFormulaPositionTable = BinTable<i32, ConfigEmberFormulaPositionRow>;
/// Parses `config_ember_formula_position_ChunkA0.bin` into a [`ConfigEmberFormulaPositionTable`].
pub fn parse_config_ember_formula_position(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigEmberFormulaPositionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_fight_resource_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigFightResourceRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub show_buff_id: Box<[i32]>,
    pub limit_range_value: VecDim,
    pub id: i32,
    pub hide_icon_zero: bool,
    pub limit_range: bool,
}
impl BinRow for ConfigFightResourceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.show_buff_id = cur.read_array_i32()?;
        self.hide_icon_zero = cur.read_bool()?;
        self.limit_range = cur.read_bool()?;
        self.limit_range_value = cur.read_vec_dim()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_fight_resource_ChunkA0.bin`.
pub type ConfigFightResourceTable = BinTable<i32, ConfigFightResourceRow>;
/// Parses `config_fight_resource_ChunkA0.bin` into a [`ConfigFightResourceTable`].
pub fn parse_config_fight_resource(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigFightResourceTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_filter_target_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFilterTargetRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigFilterTargetRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_filter_target_ChunkA0.bin`.
pub type ConfigFilterTargetTable = BinTable<i32, ConfigFilterTargetRow>;
/// Parses `config_filter_target_ChunkA0.bin` into a [`ConfigFilterTargetTable`].
pub fn parse_config_filter_target(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigFilterTargetTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_formation_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigFormationRow {
    pub des: Box<str>,
    pub base_rotation: f64,
    pub ideal_distance_from_leader: f64,
    pub ideal_angle_first_member_from_center_axis: f64,
    pub ideal_angle_between_members: f64,
    pub default_member_radius: f64,
    pub min_dist_between_members_soft: f64,
    pub min_dist_from_leader_soft: f64,
    pub id: i32,
    pub capacity_each_layer: i32,
    pub increment_unit: i32,
}
impl BinRow for ConfigFormationRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.base_rotation = cur.read_f64()?;
        self.ideal_distance_from_leader = cur.read_f64()?;
        self.ideal_angle_first_member_from_center_axis = cur.read_f64()?;
        self.ideal_angle_between_members = cur.read_f64()?;
        self.default_member_radius = cur.read_f64()?;
        self.min_dist_between_members_soft = cur.read_f64()?;
        self.min_dist_from_leader_soft = cur.read_f64()?;
        self.capacity_each_layer = cur.read_i32()?;
        self.increment_unit = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_formation_ChunkA0.bin`.
pub type ConfigFormationTable = BinTable<i32, ConfigFormationRow>;
/// Parses `config_formation_ChunkA0.bin` into a [`ConfigFormationTable`].
pub fn parse_config_formation(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigFormationTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_function_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFunctionRow {
    pub name: Box<str>,
    pub season_id: Box<[i32]>,
    pub unhide_condition: Box<str>,
    pub icon: Box<str>,
    pub unlock_icon: Box<str>,
    pub des: Box<str>,
    pub function_id: Box<str>,
    pub show_location: Box<str>,
    pub show_language: Box<str>,
    pub free_icon: Box<str>,
    pub season_type: Box<[i32]>,
    pub channel: Box<str>,
    pub weblink: Box<str>,
    pub id: i32,
    pub condition: i32,
    pub level: i32,
    pub task_id: i32,
    pub con_event: i32,
    pub pre_function: i32,
    pub show: i32,
    pub show_order: i32,
    pub npc_show_order: i32,
    pub hint_type: i32,
    pub interactive_icon_id: i32,
    pub system_help: i32,
    pub show_gamepad_entrance: i32,
    pub gamepad_entrance_default: i32,
    pub gamepad_entrance_priority: i32,
    pub function_modifier_id: i32,
    pub color_back: i32,
    pub color_front: i32,
    pub free_location: i32,
    pub attention: bool,
    pub show_npc: bool,
}
impl ConfigFunctionRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.condition = cur.read_i32()?;
        self.level = cur.read_i32()?;
        self.task_id = cur.read_i32()?;
        self.con_event = cur.read_i32()?;
        self.pre_function = cur.read_i32()?;
        self.season_id = cur.read_array_i32()?;
        self.show = cur.read_i32()?;
        self.unhide_condition = cur.read_str()?;
        self.show_order = cur.read_i32()?;
        self.npc_show_order = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.hint_type = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.unlock_icon = cur.read_str()?;
        self.interactive_icon_id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.function_id = cur.read_str()?;
        self.attention = cur.read_bool()?;
        self.system_help = cur.read_i32()?;
        self.show_location = cur.read_str()?;
        self.show_language = cur.read_str()?;
        self.show_gamepad_entrance = cur.read_i32()?;
        self.gamepad_entrance_default = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.gamepad_entrance_priority = cur.read_i32()?;
        self.function_modifier_id = cur.read_i32()?;
        self.show_npc = cur.read_bool()?;
        self.free_icon = cur.read_str()?;
        self.color_back = cur.read_i32()?;
        self.color_front = cur.read_i32()?;
        self.free_location = cur.read_i32()?;
        self.season_type = cur.read_array_i32()?;
        self.channel = cur.read_str()?;
        self.weblink = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigFunctionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_function_ChunkA0.bin`.
pub type ConfigFunctionTable = BinTable<i32, ConfigFunctionRow>;
/// Parses `config_function_ChunkA0.bin` into a [`ConfigFunctionTable`].
pub fn parse_config_function(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigFunctionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_function_modifier_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFunctionModifierRow {
    pub sp1: Box<str>,
    pub sp2: Box<str>,
    pub sp3: Box<str>,
    pub sp4: Box<str>,
    pub sp5: Box<str>,
    pub id: i32,
    pub modifier: i32,
    pub function: i32,
    pub p1: i32,
    pub p2: i32,
    pub p3: i32,
    pub p4: i32,
    pub p5: i32,
    pub p6: i32,
    pub desc_id: i32,
    pub tier: i32,
    pub show_ui: bool,
}
impl ConfigFunctionModifierRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.modifier = cur.read_i32()?;
        self.function = cur.read_i32()?;
        self.p1 = cur.read_i32()?;
        self.p2 = cur.read_i32()?;
        self.p3 = cur.read_i32()?;
        self.p4 = cur.read_i32()?;
        self.p5 = cur.read_i32()?;
        self.p6 = cur.read_i32()?;
        self.sp1 = cur.read_str()?;
        self.sp2 = cur.read_str()?;
        self.sp3 = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.sp4 = cur.read_str()?;
        self.sp5 = cur.read_str()?;
        self.show_ui = cur.read_bool()?;
        self.desc_id = cur.read_i32()?;
        self.tier = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigFunctionModifierRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_function_modifier_ChunkA0.bin`.
pub type ConfigFunctionModifierTable = BinTable<i32, ConfigFunctionModifierRow>;
/// Parses `config_function_modifier_ChunkA0.bin` into a [`ConfigFunctionModifierTable`].
pub fn parse_config_function_modifier(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigFunctionModifierTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_function_modifier_desc_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigFunctionModifierDescRow {
    pub des: Box<str>,
    pub des_param: Box<str>,
    pub danger: Box<[i32]>,
    pub combine_parameter: Box<str>,
    pub param_multiplier: Box<[f64]>,
    pub id: i32,
}
impl BinRow for ConfigFunctionModifierDescRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.des_param = cur.read_str()?;
        self.danger = cur.read_array_i32()?;
        self.combine_parameter = cur.read_str()?;
        self.param_multiplier = cur.read_array_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_function_modifier_desc_ChunkA0.bin`.
pub type ConfigFunctionModifierDescTable = BinTable<i32, ConfigFunctionModifierDescRow>;
/// Parses `config_function_modifier_desc_ChunkA0.bin` into a [`ConfigFunctionModifierDescTable`].
pub fn parse_config_function_modifier_desc(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigFunctionModifierDescTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_game_platform_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGamePlatformRow {
    pub channel: Box<str>,
    pub platform: Box<str>,
    pub userdefine: FxHashMap<Box<str>, i32>,
    pub id: i32,
    pub client_cl: i32,
    pub auto_login: bool,
    pub guest_login: bool,
    pub taptap_login: bool,
    pub google_login: bool,
    pub apple_login: bool,
    pub steam_login: bool,
    pub region: bool,
    pub real_name: bool,
    pub tap_moment: bool,
    pub open_charge: bool,
    pub test_qualification: bool,
    pub phone_login: bool,
    pub test_antiaddiction: bool,
}
impl ConfigGamePlatformRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.channel = cur.read_str()?;
        self.platform = cur.read_str()?;
        self.client_cl = cur.read_i32()?;
        self.auto_login = cur.read_bool()?;
        self.guest_login = cur.read_bool()?;
        self.taptap_login = cur.read_bool()?;
        self.google_login = cur.read_bool()?;
        self.apple_login = cur.read_bool()?;
        self.steam_login = cur.read_bool()?;
        self.region = cur.read_bool()?;
        self.real_name = cur.read_bool()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.tap_moment = cur.read_bool()?;
        self.userdefine = cur.read_map_str_i32()?;
        self.open_charge = cur.read_bool()?;
        self.test_qualification = cur.read_bool()?;
        self.phone_login = cur.read_bool()?;
        self.test_antiaddiction = cur.read_bool()?;
        Ok(())
    }
}
impl BinRow for ConfigGamePlatformRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_game_platform_ChunkA0.bin`.
pub type ConfigGamePlatformTable = BinTable<i32, ConfigGamePlatformRow>;
/// Parses `config_game_platform_ChunkA0.bin` into a [`ConfigGamePlatformTable`].
pub fn parse_config_game_platform(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGamePlatformTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gen_contract_servant_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigGenContractServantRow {
    pub contract_point_other_affix: Box<[FConfigAffix]>,
    pub servant_affix: Box<[FConfigAffix]>,
    pub servant_skill_affix: Box<[FConfigAffix]>,
    pub other_affix_level: Box<[i32]>,
    pub servant_id: i32,
    pub star_level: i32,
    pub unified_pos: i32,
    pub affix_level: i32,
}
impl BinRow for ConfigGenContractServantRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.servant_id = cur.read_i32()?;
        self.star_level = cur.read_i32()?;
        self.unified_pos = cur.read_i32()?;
        self.contract_point_other_affix = FConfigAffix::read_array(cur)?;
        self.servant_affix = FConfigAffix::read_array(cur)?;
        self.affix_level = cur.read_i32()?;
        self.servant_skill_affix = FConfigAffix::read_array(cur)?;
        self.other_affix_level = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gen_contract_servant_ChunkA0.bin`.
pub type ConfigGenContractServantTable = BinTable<
    (i32, i32, i32),
    ConfigGenContractServantRow,
>;
/// Parses `config_gen_contract_servant_ChunkA0.bin` into a [`ConfigGenContractServantTable`].
pub fn parse_config_gen_contract_servant(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGenContractServantTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ghost_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGhostRow {
    pub skill_link_pos: Box<[i32]>,
    pub filter_tag_and: Box<[i32]>,
    pub filter_tag_or: Box<[i32]>,
    pub filter_tag_not: Box<[i32]>,
    pub tags_to_skill: Box<[i32]>,
    pub attr_inherit_group: Box<[i32]>,
    pub id: i32,
    pub monster_id: i32,
    pub category: i32,
    pub r#type: i32,
    pub teleport_into_boss_area_with_player: i32,
    pub skill_link: bool,
    pub as_owner: bool,
    pub skill_cost: bool,
    pub can_hurt: bool,
    pub use_weapon: bool,
    pub show_weapon: bool,
    pub self_skill_affixevent: bool,
    pub owner_skill_affixevent: bool,
    pub attr_mode: bool,
    pub use_afl_inherit_banlist: bool,
    pub ext_skill_mode: bool,
    pub owner_magic_energy: bool,
    pub enable_extra_dir_move: bool,
    pub owner_ammo: bool,
}
impl ConfigGhostRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.monster_id = cur.read_i32()?;
        self.category = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.skill_link = cur.read_bool()?;
        self.skill_link_pos = cur.read_array_i32()?;
        self.filter_tag_and = cur.read_array_i32()?;
        self.filter_tag_or = cur.read_array_i32()?;
        self.filter_tag_not = cur.read_array_i32()?;
        self.as_owner = cur.read_bool()?;
        self.skill_cost = cur.read_bool()?;
        self.tags_to_skill = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.can_hurt = cur.read_bool()?;
        self.use_weapon = cur.read_bool()?;
        self.show_weapon = cur.read_bool()?;
        self.self_skill_affixevent = cur.read_bool()?;
        self.owner_skill_affixevent = cur.read_bool()?;
        self.attr_mode = cur.read_bool()?;
        self.attr_inherit_group = cur.read_array_i32()?;
        self.use_afl_inherit_banlist = cur.read_bool()?;
        self.teleport_into_boss_area_with_player = cur.read_i32()?;
        self.ext_skill_mode = cur.read_bool()?;
        self.owner_magic_energy = cur.read_bool()?;
        self.enable_extra_dir_move = cur.read_bool()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.owner_ammo = cur.read_bool()?;
        Ok(())
    }
}
impl BinRow for ConfigGhostRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ghost_ChunkA0.bin`.
pub type ConfigGhostTable = BinTable<i32, ConfigGhostRow>;
/// Parses `config_ghost_ChunkA0.bin` into a [`ConfigGhostTable`].
pub fn parse_config_ghost(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGhostTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ghost_attr_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigGhostAttrRow {
    pub hp: f64,
    pub mana: f64,
    pub shield: f64,
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
    pub chase_range: f64,
    pub attributes: FxHashMap<i32, Box<[f64]>>,
    pub ghost_id: i32,
    pub skill_level: i32,
}
impl ConfigGhostAttrRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.ghost_id = cur.read_i32()?;
        self.skill_level = cur.read_i32()?;
        self.hp = cur.read_f64()?;
        self.mana = cur.read_f64()?;
        self.shield = cur.read_f64()?;
        self.armour = cur.read_f64()?;
        self.hit = cur.read_f64()?;
        self.dodge = cur.read_f64()?;
        self.attack_block = cur.read_f64()?;
        self.spell_block = cur.read_f64()?;
        self.fire_res = cur.read_f64()?;
        self.ice_res = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.light_res = cur.read_f64()?;
        self.chaos_res = cur.read_f64()?;
        self.cri_rate = cur.read_f64()?;
        self.cri_dmg = cur.read_f64()?;
        self.hp_recovery = cur.read_f64()?;
        self.mana_recovery = cur.read_f64()?;
        self.move_speed = cur.read_f64()?;
        self.defend_range = cur.read_f64()?;
        self.chase_range = cur.read_f64()?;
        self.attributes = cur.read_map_i32_array_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigGhostAttrRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ghost_attr_ChunkA0.bin`.
pub type ConfigGhostAttrTable = BinTable<(i32, i32), ConfigGhostAttrRow>;
/// Parses `config_ghost_attr_ChunkA0.bin` into a [`ConfigGhostAttrTable`].
pub fn parse_config_ghost_attr(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGhostAttrTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gm_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGmRow {
    pub des: Box<str>,
    pub currency: FxHashMap<i32, i32>,
    pub item: Box<[i32]>,
    pub item_gold: Box<[i32]>,
    pub item_test: Box<[i32]>,
    pub skill: Box<[i32]>,
    pub id: i32,
    pub item_level: i32,
}
impl BinRow for ConfigGmRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.currency = cur.read_map_i32_i32()?;
        self.item = cur.read_array_i32()?;
        self.item_gold = cur.read_array_i32()?;
        self.item_test = cur.read_array_i32()?;
        self.skill = cur.read_array_i32()?;
        self.item_level = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gm_ChunkA0.bin`.
pub type ConfigGmTable = BinTable<i32, ConfigGmRow>;
/// Parses `config_gm_ChunkA0.bin` into a [`ConfigGmTable`].
pub fn parse_config_gm(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGmTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gmlist_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGmlistRow {
    pub desc: Box<str>,
    pub param1: Box<str>,
    pub param2: Box<str>,
    pub param3: Box<str>,
    pub param4: Box<str>,
    pub param5: Box<str>,
    pub param6: Box<str>,
    pub param7: Box<str>,
    pub param8: Box<str>,
    pub param9: Box<str>,
    pub param10: Box<str>,
    pub param11: Box<str>,
    pub param12: Box<str>,
    pub param13: Box<str>,
    pub param14: Box<str>,
    pub param15: Box<str>,
    pub param16: Box<str>,
    pub param17: Box<str>,
    pub param18: Box<str>,
    pub param19: Box<str>,
    pub param20: Box<str>,
    pub id: i32,
    pub order: i32,
}
impl ConfigGmlistRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.param1 = cur.read_str()?;
        self.param2 = cur.read_str()?;
        self.param3 = cur.read_str()?;
        self.param4 = cur.read_str()?;
        self.param5 = cur.read_str()?;
        self.param6 = cur.read_str()?;
        self.param7 = cur.read_str()?;
        self.param8 = cur.read_str()?;
        self.param9 = cur.read_str()?;
        self.param10 = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.param11 = cur.read_str()?;
        self.param12 = cur.read_str()?;
        self.param13 = cur.read_str()?;
        self.param14 = cur.read_str()?;
        self.param15 = cur.read_str()?;
        self.param16 = cur.read_str()?;
        self.param17 = cur.read_str()?;
        self.param18 = cur.read_str()?;
        self.param19 = cur.read_str()?;
        self.param20 = cur.read_str()?;
        self.order = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigGmlistRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gmlist_ChunkA0.bin`.
pub type ConfigGmlistTable = BinTable<i32, ConfigGmlistRow>;
/// Parses `config_gmlist_ChunkA0.bin` into a [`ConfigGmlistTable`].
pub fn parse_config_gmlist(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGmlistTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hdui_image_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHduiImageRow {
    pub id: Box<str>,
    pub value: Box<str>,
    pub width: i32,
    pub height: i32,
}
impl BinRow for ConfigHduiImageRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_str()?;
        self.value = cur.read_str()?;
        self.width = cur.read_i32()?;
        self.height = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hdui_image_ChunkA0.bin`.
pub type ConfigHduiImageTable = BinTable<Box<str>, ConfigHduiImageRow>;
/// Parses `config_hdui_image_ChunkA0.bin` into a [`ConfigHduiImageTable`].
pub fn parse_config_hdui_image(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHduiImageTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hyperlink_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHyperlinkRow {
    pub name: Box<str>,
    pub des: Box<str>,
    pub des_param: Box<[Box<str>]>,
    pub custom_param: Box<str>,
    pub des_quote: Box<[i32]>,
    pub des_pic: Box<str>,
    pub id: i32,
    pub manual_rule_id: i32,
    pub des_type: i32,
}
impl BinRow for ConfigHyperlinkRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.des = cur.read_str()?;
        self.des_param = cur.read_array_str()?;
        self.manual_rule_id = cur.read_i32()?;
        self.custom_param = cur.read_str()?;
        self.des_quote = cur.read_array_i32()?;
        self.des_type = cur.read_i32()?;
        self.des_pic = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hyperlink_ChunkA0.bin`.
pub type ConfigHyperlinkTable = BinTable<i32, ConfigHyperlinkRow>;
/// Parses `config_hyperlink_ChunkA0.bin` into a [`ConfigHyperlinkTable`].
pub fn parse_config_hyperlink(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHyperlinkTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_impulse_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigImpulseRow {
    pub impulse: VecDim,
    pub random_percent_min: VecDim,
    pub random_percent_max: VecDim,
    pub id: i32,
}
impl BinRow for ConfigImpulseRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.impulse = cur.read_vec_dim()?;
        self.random_percent_min = cur.read_vec_dim()?;
        self.random_percent_max = cur.read_vec_dim()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_impulse_ChunkA0.bin`.
pub type ConfigImpulseTable = BinTable<i32, ConfigImpulseRow>;
/// Parses `config_impulse_ChunkA0.bin` into a [`ConfigImpulseTable`].
pub fn parse_config_impulse(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigImpulseTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_in_game_download_path_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigInGameDownloadPathRow {
    pub chunk_path: Box<[Box<str>]>,
    pub chunk_id: i32,
}
impl BinRow for ConfigInGameDownloadPathRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.chunk_id = cur.read_i32()?;
        self.chunk_path = cur.read_array_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_in_game_download_path_ChunkA0.bin`.
pub type ConfigInGameDownloadPathTable = BinTable<i32, ConfigInGameDownloadPathRow>;
/// Parses `config_in_game_download_path_ChunkA0.bin` into a [`ConfigInGameDownloadPathTable`].
pub fn parse_config_in_game_download_path(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigInGameDownloadPathTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_jigsaw_globalmap_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigJigsawGlobalmapRow {
    pub born_points: FxHashMap<i32, Box<[i32]>>,
    pub difficulty_levels: Box<[i32]>,
    pub id: i32,
}
impl BinRow for ConfigJigsawGlobalmapRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.born_points = cur.read_map_i32_array_i32()?;
        self.difficulty_levels = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_jigsaw_globalmap_ChunkA0.bin`.
pub type ConfigJigsawGlobalmapTable = BinTable<i32, ConfigJigsawGlobalmapRow>;
/// Parses `config_jigsaw_globalmap_ChunkA0.bin` into a [`ConfigJigsawGlobalmapTable`].
pub fn parse_config_jigsaw_globalmap(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigJigsawGlobalmapTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_jigsaw_localmap_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigJigsawLocalmapRow {
    pub difficulty_levels: Box<[i32]>,
    pub pieces: FxHashMap<i32, i32>,
    pub map_id: i32,
    pub piece_number: i32,
    pub r#type: i32,
}
impl BinRow for ConfigJigsawLocalmapRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.piece_number = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.difficulty_levels = cur.read_array_i32()?;
        self.pieces = cur.read_map_i32_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_jigsaw_localmap_ChunkA0.bin`.
pub type ConfigJigsawLocalmapTable = BinTable<i32, ConfigJigsawLocalmapRow>;
/// Parses `config_jigsaw_localmap_ChunkA0.bin` into a [`ConfigJigsawLocalmapTable`].
pub fn parse_config_jigsaw_localmap(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigJigsawLocalmapTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_key_bind_list_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigKeyBindListRow {
    pub icon: Box<str>,
    pub iconl: Box<str>,
    pub name: Box<str>,
    pub dsicon: Box<str>,
    pub sdicon: Box<str>,
    pub id: i32,
    pub r#use: i32,
}
impl BinRow for ConfigKeyBindListRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.iconl = cur.read_str()?;
        self.name = cur.read_str()?;
        self.r#use = cur.read_i32()?;
        self.dsicon = cur.read_str()?;
        self.sdicon = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_key_bind_list_ChunkA0.bin`.
pub type ConfigKeyBindListTable = BinTable<i32, ConfigKeyBindListRow>;
/// Parses `config_key_bind_list_ChunkA0.bin` into a [`ConfigKeyBindListTable`].
pub fn parse_config_key_bind_list(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigKeyBindListTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_keyboard_bind_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigKeyboardBindRow {
    pub des: Box<str>,
    pub name: Box<str>,
    pub forbiddenkey: Box<[i32]>,
    pub id: i32,
    pub func: i32,
    pub defaultkey: i32,
    pub show: i32,
    pub change: i32,
    pub clear: i32,
    pub setting_display_order: i32,
    pub gpdefaultkey: i32,
    pub gpshow: i32,
    pub gpchange: i32,
    pub gpclear: i32,
    pub show_placeholder: bool,
}
impl ConfigKeyboardBindRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.name = cur.read_str()?;
        self.func = cur.read_i32()?;
        self.defaultkey = cur.read_i32()?;
        self.show = cur.read_i32()?;
        self.change = cur.read_i32()?;
        self.clear = cur.read_i32()?;
        self.forbiddenkey = cur.read_array_i32()?;
        self.setting_display_order = cur.read_i32()?;
        self.gpdefaultkey = cur.read_i32()?;
        self.gpshow = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.gpchange = cur.read_i32()?;
        self.gpclear = cur.read_i32()?;
        self.show_placeholder = cur.read_bool()?;
        Ok(())
    }
}
impl BinRow for ConfigKeyboardBindRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_keyboard_bind_ChunkA0.bin`.
pub type ConfigKeyboardBindTable = BinTable<i32, ConfigKeyboardBindRow>;
/// Parses `config_keyboard_bind_ChunkA0.bin` into a [`ConfigKeyboardBindTable`].
pub fn parse_config_keyboard_bind(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigKeyboardBindTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_keyboard_layout_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigKeyboardLayoutRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub display_name: Box<str>,
    pub remap_default_action_key: FxHashMap<Box<str>, Box<str>>,
    pub remap_key_event: FxHashMap<i32, Box<[i32]>>,
    pub auto_detect_lang_id: Box<[i32]>,
    pub id: i32,
    pub auto_detect: i32,
    pub is_default: i32,
}
impl BinRow for ConfigKeyboardLayoutRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.display_name = cur.read_str()?;
        self.remap_default_action_key = cur.read_map_str_str()?;
        self.remap_key_event = cur.read_map_i32_array_i32()?;
        self.auto_detect = cur.read_i32()?;
        self.auto_detect_lang_id = cur.read_array_i32()?;
        self.is_default = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_keyboard_layout_ChunkA0.bin`.
pub type ConfigKeyboardLayoutTable = BinTable<i32, ConfigKeyboardLayoutRow>;
/// Parses `config_keyboard_layout_ChunkA0.bin` into a [`ConfigKeyboardLayoutTable`].
pub fn parse_config_keyboard_layout(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigKeyboardLayoutTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_kill_impulse_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigKillImpulseTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigKillImpulseTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_kill_impulse_type_ChunkA0.bin`.
pub type ConfigKillImpulseTypeTable = BinTable<i32, ConfigKillImpulseTypeRow>;
/// Parses `config_kill_impulse_type_ChunkA0.bin` into a [`ConfigKillImpulseTypeTable`].
pub fn parse_config_kill_impulse_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigKillImpulseTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_kill_reason_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigKillReasonRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub reason: Box<str>,
    pub id: i32,
    pub has_drop: bool,
}
impl BinRow for ConfigKillReasonRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.reason = cur.read_str()?;
        self.has_drop = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_kill_reason_ChunkA0.bin`.
pub type ConfigKillReasonTable = BinTable<i32, ConfigKillReasonRow>;
/// Parses `config_kill_reason_ChunkA0.bin` into a [`ConfigKillReasonTable`].
pub fn parse_config_kill_reason(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigKillReasonTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_loadingPic_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLoadingPicRow {
    pub pic_name: Box<str>,
    pub static_pic: Box<str>,
    pub activity_name: Box<str>,
    pub activity_desc: Box<str>,
    pub id: i32,
    pub weight: i32,
    pub is_show: bool,
    pub anim_type: bool,
    pub is_activity: bool,
}
impl BinRow for ConfigLoadingPicRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.pic_name = cur.read_str()?;
        self.weight = cur.read_i32()?;
        self.is_show = cur.read_bool()?;
        self.anim_type = cur.read_bool()?;
        self.static_pic = cur.read_str()?;
        self.is_activity = cur.read_bool()?;
        self.activity_name = cur.read_str()?;
        self.activity_desc = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_loadingPic_ChunkA0.bin`.
pub type ConfigLoadingPicTable = BinTable<i32, ConfigLoadingPicRow>;
/// Parses `config_loadingPic_ChunkA0.bin` into a [`ConfigLoadingPicTable`].
pub fn parse_config_loading_pic(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLoadingPicTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_loading_tips_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLoadingTipsRow {
    pub tips: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigLoadingTipsRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.tips = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_loading_tips_ChunkA0.bin`.
pub type ConfigLoadingTipsTable = BinTable<i32, ConfigLoadingTipsRow>;
/// Parses `config_loading_tips_ChunkA0.bin` into a [`ConfigLoadingTipsTable`].
pub fn parse_config_loading_tips(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLoadingTipsTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_location_server_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLocationServerRow {
    pub name: Box<str>,
    pub server_list: Box<[i32]>,
    pub client: Box<[Box<str>]>,
    pub taptap_region: Box<[Box<str>]>,
    pub appstore_region: Box<[Box<str>]>,
    pub id: i32,
}
impl BinRow for ConfigLocationServerRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.server_list = cur.read_array_i32()?;
        self.client = cur.read_array_str()?;
        self.taptap_region = cur.read_array_str()?;
        self.appstore_region = cur.read_array_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_location_server_ChunkA0.bin`.
pub type ConfigLocationServerTable = BinTable<i32, ConfigLocationServerRow>;
/// Parses `config_location_server_ChunkA0.bin` into a [`ConfigLocationServerTable`].
pub fn parse_config_location_server(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLocationServerTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_main_story_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMainStoryRow {
    pub name: Box<str>,
    pub icon: Box<str>,
    pub icon_mini: Box<str>,
    pub describe: Box<str>,
    pub place: Box<str>,
    pub icon_login: VecDim,
    pub before_id: Box<[i32]>,
    pub after_id: Box<[i32]>,
    pub shield_function: Box<[i32]>,
    pub unlock_condition: Box<str>,
    pub replace_condition: Box<str>,
    pub npc_tool_des: Box<str>,
    pub id: i32,
    pub enable: i32,
    pub main_city: i32,
    pub chapter: i32,
    pub level_id: i32,
    pub r#type: i32,
    pub login_x: i32,
    pub login_y: i32,
    pub login_towards_z: i32,
    pub revive_x: i32,
    pub revive_y: i32,
    pub revive_towards_z: i32,
    pub replace_id: i32,
    pub gm_order: i32,
    pub portal_num: i32,
    pub pass_drop: i32,
    pub dungeon: i32,
}
impl ConfigMainStoryRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.icon = cur.read_str()?;
        self.icon_mini = cur.read_str()?;
        self.enable = cur.read_i32()?;
        self.describe = cur.read_str()?;
        self.main_city = cur.read_i32()?;
        self.chapter = cur.read_i32()?;
        self.place = cur.read_str()?;
        self.icon_login = cur.read_vec_dim()?;
        self.level_id = cur.read_i32()?;
        self.before_id = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.after_id = cur.read_array_i32()?;
        self.r#type = cur.read_i32()?;
        self.shield_function = cur.read_array_i32()?;
        self.login_x = cur.read_i32()?;
        self.login_y = cur.read_i32()?;
        self.login_towards_z = cur.read_i32()?;
        self.revive_x = cur.read_i32()?;
        self.revive_y = cur.read_i32()?;
        self.revive_towards_z = cur.read_i32()?;
        self.unlock_condition = cur.read_str()?;
        self.replace_id = cur.read_i32()?;
        self.replace_condition = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.npc_tool_des = cur.read_str()?;
        self.gm_order = cur.read_i32()?;
        self.portal_num = cur.read_i32()?;
        self.pass_drop = cur.read_i32()?;
        self.dungeon = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMainStoryRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_main_story_ChunkA0.bin`.
pub type ConfigMainStoryTable = BinTable<i32, ConfigMainStoryRow>;
/// Parses `config_main_story_ChunkA0.bin` into a [`ConfigMainStoryTable`].
pub fn parse_config_main_story(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMainStoryTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_minimap_scale_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMinimapScaleRow {
    pub scale: f64,
    pub id: i32,
    pub fog: i32,
    pub route: bool,
}
impl BinRow for ConfigMinimapScaleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.scale = cur.read_f64()?;
        self.fog = cur.read_i32()?;
        self.route = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_minimap_scale_ChunkA0.bin`.
pub type ConfigMinimapScaleTable = BinTable<i32, ConfigMinimapScaleRow>;
/// Parses `config_minimap_scale_ChunkA0.bin` into a [`ConfigMinimapScaleTable`].
pub fn parse_config_minimap_scale(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMinimapScaleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_minion_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMinionRow {
    pub id: i32,
    pub monster_id: i32,
    pub category: i32,
    pub minion_type: i32,
    pub attr_type: i32,
    pub threat_type: i32,
    pub use_afl_inherit_banlist: bool,
}
impl BinRow for ConfigMinionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.monster_id = cur.read_i32()?;
        self.category = cur.read_i32()?;
        self.minion_type = cur.read_i32()?;
        self.attr_type = cur.read_i32()?;
        self.use_afl_inherit_banlist = cur.read_bool()?;
        self.threat_type = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_minion_ChunkA0.bin`.
pub type ConfigMinionTable = BinTable<i32, ConfigMinionRow>;
/// Parses `config_minion_ChunkA0.bin` into a [`ConfigMinionTable`].
pub fn parse_config_minion(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMinionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_minion_attr_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMinionAttrRow {
    pub hp: f64,
    pub mana: f64,
    pub shield: f64,
    pub damage: f64,
    pub ad: f64,
    pub cd: f64,
    pub fd: f64,
    pub ld: f64,
    pub td: f64,
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
    pub minion_id: i32,
    pub skill_level: i32,
}
impl ConfigMinionAttrRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.minion_id = cur.read_i32()?;
        self.skill_level = cur.read_i32()?;
        self.hp = cur.read_f64()?;
        self.mana = cur.read_f64()?;
        self.shield = cur.read_f64()?;
        self.damage = cur.read_f64()?;
        self.ad = cur.read_f64()?;
        self.cd = cur.read_f64()?;
        self.fd = cur.read_f64()?;
        self.ld = cur.read_f64()?;
        self.td = cur.read_f64()?;
        self.armour = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.hit = cur.read_f64()?;
        self.dodge = cur.read_f64()?;
        self.attack_block = cur.read_f64()?;
        self.spell_block = cur.read_f64()?;
        self.fire_res = cur.read_f64()?;
        self.ice_res = cur.read_f64()?;
        self.light_res = cur.read_f64()?;
        self.chaos_res = cur.read_f64()?;
        self.cri_rate = cur.read_f64()?;
        self.cri_dmg = cur.read_f64()?;
        self.hp_recovery = cur.read_f64()?;
        self.mana_recovery = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.move_speed = cur.read_f64()?;
        self.defend_range = cur.read_f64()?;
        self.teleport_range = cur.read_f64()?;
        self.attributes = cur.read_map_i32_array_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigMinionAttrRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_minion_attr_ChunkA0.bin`.
pub type ConfigMinionAttrTable = BinTable<(i32, i32), ConfigMinionAttrRow>;
/// Parses `config_minion_attr_ChunkA0.bin` into a [`ConfigMinionAttrTable`].
pub fn parse_config_minion_attr(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMinionAttrTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_mobileui_image_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMobileuiImageRow {
    pub id: Box<str>,
    pub value: Box<str>,
    pub width: i32,
    pub height: i32,
}
impl BinRow for ConfigMobileuiImageRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_str()?;
        self.value = cur.read_str()?;
        self.width = cur.read_i32()?;
        self.height = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_mobileui_image_ChunkA0.bin`.
pub type ConfigMobileuiImageTable = BinTable<Box<str>, ConfigMobileuiImageRow>;
/// Parses `config_mobileui_image_ChunkA0.bin` into a [`ConfigMobileuiImageTable`].
pub fn parse_config_mobileui_image(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMobileuiImageTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_monologue_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMonologueRow {
    pub r#enum: Box<str>,
    pub level: Box<[i32]>,
    pub monster_id: Box<[i32]>,
    pub mono: Box<[i32]>,
    pub hero_audio_id: Box<str>,
    pub special_mono: Box<[i32]>,
    pub des: Box<str>,
    pub id: i32,
    pub ratio: i32,
    pub priority: i32,
    pub cd: i32,
    pub interrupt: i32,
}
impl BinRow for ConfigMonologueRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.ratio = cur.read_i32()?;
        self.level = cur.read_array_i32()?;
        self.monster_id = cur.read_array_i32()?;
        self.mono = cur.read_array_i32()?;
        self.priority = cur.read_i32()?;
        self.cd = cur.read_i32()?;
        self.interrupt = cur.read_i32()?;
        self.hero_audio_id = cur.read_str()?;
        self.special_mono = cur.read_array_i32()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_monologue_ChunkA0.bin`.
pub type ConfigMonologueTable = BinTable<i32, ConfigMonologueRow>;
/// Parses `config_monologue_ChunkA0.bin` into a [`ConfigMonologueTable`].
pub fn parse_config_monologue(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMonologueTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_multi_language_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMultiLanguageRow {
    pub col_0: Box<str>,
    pub col_1: Box<str>,
    pub col_2: Box<str>,
}
impl BinRow for ConfigMultiLanguageRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.col_0 = cur.read_str()?;
        self.col_1 = cur.read_str()?;
        self.col_2 = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_multi_language_ChunkA0.bin`.
pub type ConfigMultiLanguageTable = BinTable<Box<str>, ConfigMultiLanguageRow>;
/// Parses `config_multi_language_ChunkA0.bin` into a [`ConfigMultiLanguageTable`].
pub fn parse_config_multi_language(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMultiLanguageTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_project_levels_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigProjectLevelsRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub level_path: Box<str>,
    pub param_ids: Box<[i32]>,
    pub id: i32,
}
impl BinRow for ConfigProjectLevelsRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.level_path = cur.read_str()?;
        self.param_ids = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_project_levels_ChunkA0.bin`.
pub type ConfigProjectLevelsTable = BinTable<i32, ConfigProjectLevelsRow>;
/// Parses `config_project_levels_ChunkA0.bin` into a [`ConfigProjectLevelsTable`].
pub fn parse_config_project_levels(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigProjectLevelsTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_rarity_equipment_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRarityEquipmentRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub tags: Box<[i32]>,
    pub id: i32,
}
impl BinRow for ConfigRarityEquipmentRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.tags = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_rarity_equipment_ChunkA0.bin`.
pub type ConfigRarityEquipmentTable = BinTable<i32, ConfigRarityEquipmentRow>;
/// Parses `config_rarity_equipment_ChunkA0.bin` into a [`ConfigRarityEquipmentTable`].
pub fn parse_config_rarity_equipment(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigRarityEquipmentTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_rarity_monster_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigRarityMonsterRow {
    pub experience: Box<str>,
    pub hp: f64,
    pub damage: f64,
    pub movespeed: f64,
    pub attackspeed: f64,
    pub bodysize: f64,
    pub level: Box<[i32]>,
    pub id: i32,
    pub rarity: i32,
    pub extra_level: i32,
    pub minimap_icon_id: i32,
}
impl BinRow for ConfigRarityMonsterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.rarity = cur.read_i32()?;
        self.experience = cur.read_str()?;
        self.extra_level = cur.read_i32()?;
        self.hp = cur.read_f64()?;
        self.damage = cur.read_f64()?;
        self.movespeed = cur.read_f64()?;
        self.attackspeed = cur.read_f64()?;
        self.bodysize = cur.read_f64()?;
        self.level = cur.read_array_i32()?;
        self.minimap_icon_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_rarity_monster_ChunkA0.bin`.
pub type ConfigRarityMonsterTable = BinTable<i32, ConfigRarityMonsterRow>;
/// Parses `config_rarity_monster_ChunkA0.bin` into a [`ConfigRarityMonsterTable`].
pub fn parse_config_rarity_monster(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigRarityMonsterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_rarity_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRarityTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub tags: Box<[i32]>,
    pub id: i32,
}
impl BinRow for ConfigRarityTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.tags = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_rarity_type_ChunkA0.bin`.
pub type ConfigRarityTypeTable = BinTable<i32, ConfigRarityTypeRow>;
/// Parses `config_rarity_type_ChunkA0.bin` into a [`ConfigRarityTypeTable`].
pub fn parse_config_rarity_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigRarityTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_recharge_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigRechargeRow {
    pub product_id: Box<str>,
    pub product_name: Box<str>,
    pub icon: Box<str>,
    pub reward: Box<str>,
    pub extra_reward: Box<str>,
    pub game_server_id: Box<str>,
    pub usd_price: f64,
    pub award_special: Box<str>,
    pub id: i32,
    pub rmb_price: i32,
    pub award_time: i32,
    pub discount: i32,
    pub have_limit: i32,
    pub order: i32,
    pub refund_mail_child_type: i32,
}
impl ConfigRechargeRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.product_id = cur.read_str()?;
        self.product_name = cur.read_str()?;
        self.icon = cur.read_str()?;
        self.reward = cur.read_str()?;
        self.extra_reward = cur.read_str()?;
        self.rmb_price = cur.read_i32()?;
        self.game_server_id = cur.read_str()?;
        self.usd_price = cur.read_f64()?;
        self.award_time = cur.read_i32()?;
        self.award_special = cur.read_str()?;
        self.discount = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.have_limit = cur.read_i32()?;
        self.order = cur.read_i32()?;
        self.refund_mail_child_type = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigRechargeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_recharge_ChunkA0.bin`.
pub type ConfigRechargeTable = BinTable<i32, ConfigRechargeRow>;
/// Parses `config_recharge_ChunkA0.bin` into a [`ConfigRechargeTable`].
pub fn parse_config_recharge(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigRechargeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_recharge_price_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigRechargePriceRow {
    pub sku_id: Box<str>,
    pub country: Box<str>,
    pub currency: Box<str>,
    pub price: f64,
    pub id: i32,
    pub steam_price: i32,
}
impl BinRow for ConfigRechargePriceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.sku_id = cur.read_str()?;
        self.country = cur.read_str()?;
        self.currency = cur.read_str()?;
        self.price = cur.read_f64()?;
        self.steam_price = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_recharge_price_ChunkA0.bin`.
pub type ConfigRechargePriceTable = BinTable<i32, ConfigRechargePriceRow>;
/// Parses `config_recharge_price_ChunkA0.bin` into a [`ConfigRechargePriceTable`].
pub fn parse_config_recharge_price(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigRechargePriceTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_recharge_price_steam_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRechargePriceSteamRow {
    pub sku_id: Box<str>,
    pub country: Box<str>,
    pub currency: Box<str>,
    pub id: i32,
    pub steam_price: i32,
}
impl BinRow for ConfigRechargePriceSteamRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.sku_id = cur.read_str()?;
        self.country = cur.read_str()?;
        self.currency = cur.read_str()?;
        self.steam_price = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_recharge_price_steam_ChunkA0.bin`.
pub type ConfigRechargePriceSteamTable = BinTable<i32, ConfigRechargePriceSteamRow>;
/// Parses `config_recharge_price_steam_ChunkA0.bin` into a [`ConfigRechargePriceSteamTable`].
pub fn parse_config_recharge_price_steam(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigRechargePriceSteamTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_recipe_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRecipeRow {
    pub des: Box<str>,
    pub des_in: Box<str>,
    pub des_out: Box<str>,
    pub recipe: Box<str>,
    pub product: Box<str>,
    pub preview: Box<str>,
    pub id: i32,
    pub priority: i32,
}
impl BinRow for ConfigRecipeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.des_in = cur.read_str()?;
        self.des_out = cur.read_str()?;
        self.priority = cur.read_i32()?;
        self.recipe = cur.read_str()?;
        self.product = cur.read_str()?;
        self.preview = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_recipe_ChunkA0.bin`.
pub type ConfigRecipeTable = BinTable<i32, ConfigRecipeRow>;
/// Parses `config_recipe_ChunkA0.bin` into a [`ConfigRecipeTable`].
pub fn parse_config_recipe(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigRecipeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_recipe_condition_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRecipeConditionRow {
    pub r#enum: Box<str>,
    pub default_import: Box<str>,
    pub default_output: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigRecipeConditionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.default_import = cur.read_str()?;
        self.default_output = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_recipe_condition_ChunkA0.bin`.
pub type ConfigRecipeConditionTable = BinTable<i32, ConfigRecipeConditionRow>;
/// Parses `config_recipe_condition_ChunkA0.bin` into a [`ConfigRecipeConditionTable`].
pub fn parse_config_recipe_condition(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigRecipeConditionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_red_dot_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRedDotRow {
    pub module_des: Box<str>,
    pub node_des: Box<str>,
    pub remove_type: Box<str>,
    pub resources: Box<str>,
    pub id: i32,
    pub real_id: i32,
    pub module_enum_value: i32,
    pub node_enum_value: i32,
    pub parent_id: i32,
    pub is_dynamic: bool,
    pub need_to_cache: bool,
}
impl BinRow for ConfigRedDotRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.real_id = cur.read_i32()?;
        self.module_enum_value = cur.read_i32()?;
        self.module_des = cur.read_str()?;
        self.node_enum_value = cur.read_i32()?;
        self.node_des = cur.read_str()?;
        self.parent_id = cur.read_i32()?;
        self.is_dynamic = cur.read_bool()?;
        self.remove_type = cur.read_str()?;
        self.resources = cur.read_str()?;
        self.need_to_cache = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_red_dot_ChunkA0.bin`.
pub type ConfigRedDotTable = BinTable<i32, ConfigRedDotRow>;
/// Parses `config_red_dot_ChunkA0.bin` into a [`ConfigRedDotTable`].
pub fn parse_config_red_dot(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigRedDotTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_scene_ui_order_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSceneUiOrderRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
    pub z_order: i32,
}
impl BinRow for ConfigSceneUiOrderRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.z_order = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_scene_ui_order_ChunkA0.bin`.
pub type ConfigSceneUiOrderTable = BinTable<i32, ConfigSceneUiOrderRow>;
/// Parses `config_scene_ui_order_ChunkA0.bin` into a [`ConfigSceneUiOrderTable`].
pub fn parse_config_scene_ui_order(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSceneUiOrderTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_server_affix_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServerAffixRow {
    pub desc: Box<str>,
    pub id: i32,
    pub r#type: i32,
    pub desc_order: i32,
}
impl BinRow for ConfigServerAffixRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.desc_order = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_server_affix_ChunkA0.bin`.
pub type ConfigServerAffixTable = BinTable<i32, ConfigServerAffixRow>;
/// Parses `config_server_affix_ChunkA0.bin` into a [`ConfigServerAffixTable`].
pub fn parse_config_server_affix(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigServerAffixTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_server_trigger_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServerTriggerRow {
    pub params: Box<[i32]>,
    pub id: i32,
    pub map_type: i32,
    pub map_id: i32,
    pub trigger_type: i32,
}
impl BinRow for ConfigServerTriggerRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.map_type = cur.read_i32()?;
        self.map_id = cur.read_i32()?;
        self.trigger_type = cur.read_i32()?;
        self.params = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_server_trigger_ChunkA0.bin`.
pub type ConfigServerTriggerTable = BinTable<i32, ConfigServerTriggerRow>;
/// Parses `config_server_trigger_ChunkA0.bin` into a [`ConfigServerTriggerTable`].
pub fn parse_config_server_trigger(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigServerTriggerTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_set_items_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSetItemsRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub set_num_modifier: FxHashMap<i32, Box<[i32]>>,
    pub id: i32,
}
impl BinRow for ConfigSetItemsRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.set_num_modifier = cur.read_map_i32_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_set_items_ChunkA0.bin`.
pub type ConfigSetItemsTable = BinTable<i32, ConfigSetItemsRow>;
/// Parses `config_set_items_ChunkA0.bin` into a [`ConfigSetItemsTable`].
pub fn parse_config_set_items(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSetItemsTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_steam_achievement_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSteamAchievementRow {
    pub steam_key: Box<str>,
    pub name: Box<str>,
    pub desc: Box<str>,
    pub stat_key: Box<str>,
    pub id: i32,
    pub hide: i32,
    pub cond_event_id: i32,
    pub stat_min: i32,
    pub stat_max: i32,
    pub using_old_stat: i32,
}
impl BinRow for ConfigSteamAchievementRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.steam_key = cur.read_str()?;
        self.name = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.hide = cur.read_i32()?;
        self.cond_event_id = cur.read_i32()?;
        self.stat_key = cur.read_str()?;
        self.stat_min = cur.read_i32()?;
        self.stat_max = cur.read_i32()?;
        self.using_old_stat = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_steam_achievement_ChunkA0.bin`.
pub type ConfigSteamAchievementTable = BinTable<i32, ConfigSteamAchievementRow>;
/// Parses `config_steam_achievement_ChunkA0.bin` into a [`ConfigSteamAchievementTable`].
pub fn parse_config_steam_achievement(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSteamAchievementTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_steam_language_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSteamLanguageRow {
    pub api_language: Box<str>,
    pub web_api_language: Box<str>,
}
impl BinRow for ConfigSteamLanguageRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.api_language = cur.read_str()?;
        self.web_api_language = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_steam_language_ChunkA0.bin`.
pub type ConfigSteamLanguageTable = BinTable<Box<str>, ConfigSteamLanguageRow>;
/// Parses `config_steam_language_ChunkA0.bin` into a [`ConfigSteamLanguageTable`].
pub fn parse_config_steam_language(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSteamLanguageTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_steam_stat_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSteamStatRow {
    pub steam_key: Box<str>,
    pub name: Box<str>,
    pub id: i32,
    pub r#type: i32,
}
impl BinRow for ConfigSteamStatRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.steam_key = cur.read_str()?;
        self.r#type = cur.read_i32()?;
        self.name = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_steam_stat_ChunkA0.bin`.
pub type ConfigSteamStatTable = BinTable<i32, ConfigSteamStatRow>;
/// Parses `config_steam_stat_ChunkA0.bin` into a [`ConfigSteamStatTable`].
pub fn parse_config_steam_stat(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSteamStatTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_tale_hero_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTaleHeroRow {
    pub des: Box<str>,
    pub option_behavior: Box<[i32]>,
    pub title: Box<str>,
    pub select_title: Box<[FConfigI32Str]>,
    pub select_next: Box<[i32]>,
    pub voice: Box<str>,
    pub actor: Box<str>,
    pub id: i32,
    pub r#type: i32,
    pub bind_id: i32,
    pub r#loop: i32,
    pub priority: i32,
    pub duration: i32,
    pub head_type: i32,
    pub side: i32,
}
impl ConfigTaleHeroRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.bind_id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.option_behavior = cur.read_array_i32()?;
        self.title = cur.read_str()?;
        self.r#loop = cur.read_i32()?;
        self.select_title = FConfigI32Str::read_array(cur)?;
        self.select_next = cur.read_array_i32()?;
        self.voice = cur.read_str()?;
        self.priority = cur.read_i32()?;
        self.duration = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.actor = cur.read_str()?;
        self.head_type = cur.read_i32()?;
        self.side = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigTaleHeroRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_tale_hero_ChunkA0.bin`.
pub type ConfigTaleHeroTable = BinTable<(i32, i32), ConfigTaleHeroRow>;
/// Parses `config_tale_hero_ChunkA0.bin` into a [`ConfigTaleHeroTable`].
pub fn parse_config_tale_hero(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTaleHeroTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_tale_npc_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTaleNpcRow {
    pub des: Box<str>,
    pub option_behavior: Box<[i32]>,
    pub function_id: Box<[i32]>,
    pub title: Box<str>,
    pub select_title: Box<[FConfigI32Str]>,
    pub select_next: Box<[i32]>,
    pub voice: Box<str>,
    pub actor: Box<str>,
    pub id: i32,
    pub r#type: i32,
    pub bind_id: i32,
    pub r#loop: i32,
    pub priority: i32,
    pub duration: i32,
    pub head_type: i32,
    pub side: i32,
}
impl ConfigTaleNpcRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.bind_id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.option_behavior = cur.read_array_i32()?;
        self.function_id = cur.read_array_i32()?;
        self.title = cur.read_str()?;
        self.r#loop = cur.read_i32()?;
        self.select_title = FConfigI32Str::read_array(cur)?;
        self.select_next = cur.read_array_i32()?;
        self.voice = cur.read_str()?;
        self.priority = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.duration = cur.read_i32()?;
        self.actor = cur.read_str()?;
        self.head_type = cur.read_i32()?;
        self.side = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigTaleNpcRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_tale_npc_ChunkA0.bin`.
pub type ConfigTaleNpcTable = BinTable<i32, ConfigTaleNpcRow>;
/// Parses `config_tale_npc_ChunkA0.bin` into a [`ConfigTaleNpcTable`].
pub fn parse_config_tale_npc(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTaleNpcTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_tale_text_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTaleTextRow {
    pub des: Box<str>,
    pub voice: Box<str>,
    pub id: i32,
    pub index: i32,
    pub sub_index: i32,
    pub r#type: i32,
    pub style: i32,
    pub pos_type: i32,
    pub priority: i32,
    pub duration: i32,
}
impl BinRow for ConfigTaleTextRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.index = cur.read_i32()?;
        self.sub_index = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.style = cur.read_i32()?;
        self.pos_type = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.voice = cur.read_str()?;
        self.priority = cur.read_i32()?;
        self.duration = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_tale_text_ChunkA0.bin`.
pub type ConfigTaleTextTable = BinTable<(i32, i32, i32), ConfigTaleTextRow>;
/// Parses `config_tale_text_ChunkA0.bin` into a [`ConfigTaleTextTable`].
pub fn parse_config_tale_text(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTaleTextTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_test_fight_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTestFightRow {
    pub id: i32,
    pub weapon_id: i32,
    pub main_skill_id: i32,
    pub support_skill_id1: i32,
    pub support_skill_id2: i32,
    pub support_skill_id3: i32,
    pub support_skill_id4: i32,
    pub support_skill_id5: i32,
    pub support_skill_id6: i32,
    pub talent1: i32,
    pub talent2: i32,
    pub talent3: i32,
    pub hero_character1: i32,
    pub hero_character2: i32,
    pub hero_character3: i32,
    pub hero_character4: i32,
    pub hero_character5: i32,
    pub hero_character6: i32,
}
impl ConfigTestFightRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.weapon_id = cur.read_i32()?;
        self.main_skill_id = cur.read_i32()?;
        self.support_skill_id1 = cur.read_i32()?;
        self.support_skill_id2 = cur.read_i32()?;
        self.support_skill_id3 = cur.read_i32()?;
        self.support_skill_id4 = cur.read_i32()?;
        self.support_skill_id5 = cur.read_i32()?;
        self.support_skill_id6 = cur.read_i32()?;
        self.talent1 = cur.read_i32()?;
        self.talent2 = cur.read_i32()?;
        self.talent3 = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.hero_character1 = cur.read_i32()?;
        self.hero_character2 = cur.read_i32()?;
        self.hero_character3 = cur.read_i32()?;
        self.hero_character4 = cur.read_i32()?;
        self.hero_character5 = cur.read_i32()?;
        self.hero_character6 = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigTestFightRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_test_fight_ChunkA0.bin`.
pub type ConfigTestFightTable = BinTable<i32, ConfigTestFightRow>;
/// Parses `config_test_fight_ChunkA0.bin` into a [`ConfigTestFightTable`].
pub fn parse_config_test_fight(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTestFightTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_test_map_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTestMapRow {
    pub name: Box<str>,
    pub npc_tool_des: Box<str>,
    pub id: i32,
    pub level_id: i32,
    pub level_type: i32,
    pub gm_order: i32,
    pub map_type: i32,
}
impl BinRow for ConfigTestMapRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.level_id = cur.read_i32()?;
        self.level_type = cur.read_i32()?;
        self.npc_tool_des = cur.read_str()?;
        self.gm_order = cur.read_i32()?;
        self.map_type = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_test_map_ChunkA0.bin`.
pub type ConfigTestMapTable = BinTable<i32, ConfigTestMapRow>;
/// Parses `config_test_map_ChunkA0.bin` into a [`ConfigTestMapTable`].
pub fn parse_config_test_map(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTestMapTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_v2_ui_showcase_skeletal_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigV2UiShowcaseSkeletalRow {
    pub skeletal_path: Box<str>,
    pub anim_bp_path: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigV2UiShowcaseSkeletalRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.skeletal_path = cur.read_str()?;
        self.anim_bp_path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_v2_ui_showcase_skeletal_ChunkA0.bin`.
pub type ConfigV2UiShowcaseSkeletalTable = BinTable<i32, ConfigV2UiShowcaseSkeletalRow>;
/// Parses `config_v2_ui_showcase_skeletal_ChunkA0.bin` into a [`ConfigV2UiShowcaseSkeletalTable`].
pub fn parse_config_v2_ui_showcase_skeletal(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigV2UiShowcaseSkeletalTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_v2_ui_showcase_skeletal_anim_res_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigV2UiShowcaseSkeletalAnimResRow {
    pub weapon_anim_set_ids: Box<[i32]>,
    pub id: i32,
    pub skeletal_id: i32,
}
impl BinRow for ConfigV2UiShowcaseSkeletalAnimResRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.skeletal_id = cur.read_i32()?;
        self.weapon_anim_set_ids = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_v2_ui_showcase_skeletal_anim_res_ChunkA0.bin`.
pub type ConfigV2UiShowcaseSkeletalAnimResTable = BinTable<
    i32,
    ConfigV2UiShowcaseSkeletalAnimResRow,
>;
/// Parses `config_v2_ui_showcase_skeletal_anim_res_ChunkA0.bin` into a [`ConfigV2UiShowcaseSkeletalAnimResTable`].
pub fn parse_config_v2_ui_showcase_skeletal_anim_res(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigV2UiShowcaseSkeletalAnimResTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_v2_ui_showcase_skeletal_map_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigV2UiShowcaseSkeletalMapRow {
    pub id: i32,
    pub hero_purchase_id: i32,
    pub skin_fashion_id: i32,
    pub skeletal_id: i32,
}
impl BinRow for ConfigV2UiShowcaseSkeletalMapRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.hero_purchase_id = cur.read_i32()?;
        self.skin_fashion_id = cur.read_i32()?;
        self.skeletal_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_v2_ui_showcase_skeletal_map_ChunkA0.bin`.
pub type ConfigV2UiShowcaseSkeletalMapTable = BinTable<
    i32,
    ConfigV2UiShowcaseSkeletalMapRow,
>;
/// Parses `config_v2_ui_showcase_skeletal_map_ChunkA0.bin` into a [`ConfigV2UiShowcaseSkeletalMapTable`].
pub fn parse_config_v2_ui_showcase_skeletal_map(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigV2UiShowcaseSkeletalMapTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_vendor_npc_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigVendorNpcRow {
    pub vendor_name: Box<str>,
    pub vendor_array: Box<[FConfigI32ArrayStr]>,
    pub id: i32,
    pub showing_exchange: i32,
}
impl BinRow for ConfigVendorNpcRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.vendor_name = cur.read_str()?;
        self.vendor_array = FConfigI32ArrayStr::read_array(cur)?;
        self.showing_exchange = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_vendor_npc_ChunkA0.bin`.
pub type ConfigVendorNpcTable = BinTable<i32, ConfigVendorNpcRow>;
/// Parses `config_vendor_npc_ChunkA0.bin` into a [`ConfigVendorNpcTable`].
pub fn parse_config_vendor_npc(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigVendorNpcTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
#[inline(never)]
fn verify_system_chunk_0(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_Steam_Friend_RichText_Map" => {
            Some(parse_config_steam_friend_rich_text_map(buf, scratch).map(|t| t.len()))
        }
        "config_accumulated_value" => {
            Some(parse_config_accumulated_value(buf, scratch).map(|t| t.len()))
        }
        "config_attr_tag" => Some(parse_config_attr_tag(buf, scratch).map(|t| t.len())),
        "config_c" => Some(parse_config_c(buf, scratch).map(|t| t.len())),
        "config_climbtower" => {
            Some(parse_config_climbtower(buf, scratch).map(|t| t.len()))
        }
        "config_collector" => Some(parse_config_collector(buf, scratch).map(|t| t.len())),
        "config_collectorTask" => {
            Some(parse_config_collector_task(buf, scratch).map(|t| t.len()))
        }
        "config_cond_event_task" => {
            Some(parse_config_cond_event_task(buf, scratch).map(|t| t.len()))
        }
        "config_contractPoint" => {
            Some(parse_config_contract_point(buf, scratch).map(|t| t.len()))
        }
        "config_cost_level" => {
            Some(parse_config_cost_level(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_system_chunk_1(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_customized_item" => {
            Some(parse_config_customized_item(buf, scratch).map(|t| t.len()))
        }
        "config_death_anim_drop_delay" => {
            Some(parse_config_death_anim_drop_delay(buf, scratch).map(|t| t.len()))
        }
        "config_death_anim_vfx" => {
            Some(parse_config_death_anim_vfx(buf, scratch).map(|t| t.len()))
        }
        "config_death_cause_filter" => {
            Some(parse_config_death_cause_filter(buf, scratch).map(|t| t.len()))
        }
        "config_death_effect" => {
            Some(parse_config_death_effect(buf, scratch).map(|t| t.len()))
        }
        "config_death_info_show" => {
            Some(parse_config_death_info_show(buf, scratch).map(|t| t.len()))
        }
        "config_ember_formula" => {
            Some(parse_config_ember_formula(buf, scratch).map(|t| t.len()))
        }
        "config_ember_formula_main" => {
            Some(parse_config_ember_formula_main(buf, scratch).map(|t| t.len()))
        }
        "config_ember_formula_position" => {
            Some(parse_config_ember_formula_position(buf, scratch).map(|t| t.len()))
        }
        "config_fight_resource" => {
            Some(parse_config_fight_resource(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_system_chunk_2(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_filter_target" => {
            Some(parse_config_filter_target(buf, scratch).map(|t| t.len()))
        }
        "config_formation" => Some(parse_config_formation(buf, scratch).map(|t| t.len())),
        "config_function" => Some(parse_config_function(buf, scratch).map(|t| t.len())),
        "config_function_modifier" => {
            Some(parse_config_function_modifier(buf, scratch).map(|t| t.len()))
        }
        "config_function_modifier_desc" => {
            Some(parse_config_function_modifier_desc(buf, scratch).map(|t| t.len()))
        }
        "config_game_platform" => {
            Some(parse_config_game_platform(buf, scratch).map(|t| t.len()))
        }
        "config_gen_contract_servant" => {
            Some(parse_config_gen_contract_servant(buf, scratch).map(|t| t.len()))
        }
        "config_ghost" => Some(parse_config_ghost(buf, scratch).map(|t| t.len())),
        "config_ghost_attr" => {
            Some(parse_config_ghost_attr(buf, scratch).map(|t| t.len()))
        }
        "config_gm" => Some(parse_config_gm(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}
#[inline(never)]
fn verify_system_chunk_3(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_gmlist" => Some(parse_config_gmlist(buf, scratch).map(|t| t.len())),
        "config_hdui_image" => {
            Some(parse_config_hdui_image(buf, scratch).map(|t| t.len()))
        }
        "config_hyperlink" => Some(parse_config_hyperlink(buf, scratch).map(|t| t.len())),
        "config_impulse" => Some(parse_config_impulse(buf, scratch).map(|t| t.len())),
        "config_in_game_download_path" => {
            Some(parse_config_in_game_download_path(buf, scratch).map(|t| t.len()))
        }
        "config_jigsaw_globalmap" => {
            Some(parse_config_jigsaw_globalmap(buf, scratch).map(|t| t.len()))
        }
        "config_jigsaw_localmap" => {
            Some(parse_config_jigsaw_localmap(buf, scratch).map(|t| t.len()))
        }
        "config_key_bind_list" => {
            Some(parse_config_key_bind_list(buf, scratch).map(|t| t.len()))
        }
        "config_keyboard_bind" => {
            Some(parse_config_keyboard_bind(buf, scratch).map(|t| t.len()))
        }
        "config_keyboard_layout" => {
            Some(parse_config_keyboard_layout(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_system_chunk_4(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_kill_impulse_type" => {
            Some(parse_config_kill_impulse_type(buf, scratch).map(|t| t.len()))
        }
        "config_kill_reason" => {
            Some(parse_config_kill_reason(buf, scratch).map(|t| t.len()))
        }
        "config_loadingPic" => {
            Some(parse_config_loading_pic(buf, scratch).map(|t| t.len()))
        }
        "config_loading_tips" => {
            Some(parse_config_loading_tips(buf, scratch).map(|t| t.len()))
        }
        "config_location_server" => {
            Some(parse_config_location_server(buf, scratch).map(|t| t.len()))
        }
        "config_main_story" => {
            Some(parse_config_main_story(buf, scratch).map(|t| t.len()))
        }
        "config_minimap_scale" => {
            Some(parse_config_minimap_scale(buf, scratch).map(|t| t.len()))
        }
        "config_minion" => Some(parse_config_minion(buf, scratch).map(|t| t.len())),
        "config_minion_attr" => {
            Some(parse_config_minion_attr(buf, scratch).map(|t| t.len()))
        }
        "config_mobileui_image" => {
            Some(parse_config_mobileui_image(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_system_chunk_5(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_monologue" => Some(parse_config_monologue(buf, scratch).map(|t| t.len())),
        "config_multi_language" => {
            Some(parse_config_multi_language(buf, scratch).map(|t| t.len()))
        }
        "config_project_levels" => {
            Some(parse_config_project_levels(buf, scratch).map(|t| t.len()))
        }
        "config_rarity_equipment" => {
            Some(parse_config_rarity_equipment(buf, scratch).map(|t| t.len()))
        }
        "config_rarity_monster" => {
            Some(parse_config_rarity_monster(buf, scratch).map(|t| t.len()))
        }
        "config_rarity_type" => {
            Some(parse_config_rarity_type(buf, scratch).map(|t| t.len()))
        }
        "config_recharge" => Some(parse_config_recharge(buf, scratch).map(|t| t.len())),
        "config_recharge_price" => {
            Some(parse_config_recharge_price(buf, scratch).map(|t| t.len()))
        }
        "config_recharge_price_steam" => {
            Some(parse_config_recharge_price_steam(buf, scratch).map(|t| t.len()))
        }
        "config_recipe" => Some(parse_config_recipe(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}
#[inline(never)]
fn verify_system_chunk_6(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_recipe_condition" => {
            Some(parse_config_recipe_condition(buf, scratch).map(|t| t.len()))
        }
        "config_red_dot" => Some(parse_config_red_dot(buf, scratch).map(|t| t.len())),
        "config_scene_ui_order" => {
            Some(parse_config_scene_ui_order(buf, scratch).map(|t| t.len()))
        }
        "config_server_affix" => {
            Some(parse_config_server_affix(buf, scratch).map(|t| t.len()))
        }
        "config_server_trigger" => {
            Some(parse_config_server_trigger(buf, scratch).map(|t| t.len()))
        }
        "config_set_items" => Some(parse_config_set_items(buf, scratch).map(|t| t.len())),
        "config_steam_achievement" => {
            Some(parse_config_steam_achievement(buf, scratch).map(|t| t.len()))
        }
        "config_steam_language" => {
            Some(parse_config_steam_language(buf, scratch).map(|t| t.len()))
        }
        "config_steam_stat" => {
            Some(parse_config_steam_stat(buf, scratch).map(|t| t.len()))
        }
        "config_tale_hero" => Some(parse_config_tale_hero(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}
#[inline(never)]
fn verify_system_chunk_7(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_tale_npc" => Some(parse_config_tale_npc(buf, scratch).map(|t| t.len())),
        "config_tale_text" => Some(parse_config_tale_text(buf, scratch).map(|t| t.len())),
        "config_test_fight" => {
            Some(parse_config_test_fight(buf, scratch).map(|t| t.len()))
        }
        "config_test_map" => Some(parse_config_test_map(buf, scratch).map(|t| t.len())),
        "config_v2_ui_showcase_skeletal" => {
            Some(parse_config_v2_ui_showcase_skeletal(buf, scratch).map(|t| t.len()))
        }
        "config_v2_ui_showcase_skeletal_anim_res" => {
            Some(
                parse_config_v2_ui_showcase_skeletal_anim_res(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_v2_ui_showcase_skeletal_map" => {
            Some(parse_config_v2_ui_showcase_skeletal_map(buf, scratch).map(|t| t.len()))
        }
        "config_vendor_npc" => {
            Some(parse_config_vendor_npc(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
pub(crate) fn verify_system_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    if let Some(res) = verify_system_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_system_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_system_chunk_2(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_system_chunk_3(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_system_chunk_4(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_system_chunk_5(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_system_chunk_6(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_system_chunk_7(stem, buf, scratch) {
        return Some(res);
    }
    None
}

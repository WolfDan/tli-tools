//! Auto-generated `.bin` configuration table types and parsers for the `skill` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{
    FConfigBoolArrayI32ArrayI32, FConfigI32Array2dI32F64, FConfigI32ArrayI32F64x3,
    FConfigI32I32,
};
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_skill_anime_template_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillAnimeTemplateRow {
    pub col_1: Box<str>,
    pub col_2: Box<str>,
    pub col_0: i32,
}
impl BinRow for ConfigSkillAnimeTemplateRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.col_0 = cur.read_i32()?;
        self.col_1 = cur.read_str()?;
        self.col_2 = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_anime_template_ChunkA0.bin`.
pub type ConfigSkillAnimeTemplateTable = BinTable<i32, ConfigSkillAnimeTemplateRow>;
/// Parses `config_skill_anime_template_ChunkA0.bin` into a [`ConfigSkillAnimeTemplateTable`].
pub fn parse_config_skill_anime_template(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillAnimeTemplateTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_skill_anime_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillAnimeTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigSkillAnimeTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_anime_type_ChunkA0.bin`.
pub type ConfigSkillAnimeTypeTable = BinTable<i32, ConfigSkillAnimeTypeRow>;
/// Parses `config_skill_anime_type_ChunkA0.bin` into a [`ConfigSkillAnimeTypeTable`].
pub fn parse_config_skill_anime_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillAnimeTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_skill_ctrl_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigSkillCtrlTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub on_touch: Box<[i32]>,
    pub on_drag: Box<[i32]>,
    pub on_hold: Box<[i32]>,
    pub on_click_release: Box<[i32]>,
    pub on_hold_release: Box<[i32]>,
    pub on_drag_release: Box<[i32]>,
    pub on_aim: Box<[i32]>,
    pub on_aim_stop: Box<[i32]>,
    pub aim_cursor_by_position_range_radius_ratio: f64,
    pub aim_cursor_by_motion_world_unit_ratio: f64,
    pub aim_cursor_by_motion_range_radius_factor: f64,
    pub aim_cursor_init_pos_range_radius_factor: f64,
    pub aim_cursor_init_pos_aoe_range_radius_factor: f64,
    pub touch_time_click_to_hold_threshold: f64,
    pub drag_dist_touch_to_drag_threshold: f64,
    pub drag_dist_drag_to_hold_threshold: f64,
    pub ignore_block_move_pad_request_types: Box<[i32]>,
    pub on_click_mk: Box<[i32]>,
    pub on_click_release_mk: Box<[i32]>,
    pub on_aim_mk: Box<[i32]>,
    pub on_aim_stop_mk: Box<[i32]>,
    pub ignore_block_move_pad_request_types_mk: Box<[i32]>,
    pub block_move_types: Box<[i32]>,
    pub id: i32,
    pub command_type: i32,
    pub aim_cursor_cast_range: i32,
    pub aim_cursor_direction_arrow: i32,
    pub aim_cursor_aoe_range: i32,
    pub auto_aiming_auxiliary_line: i32,
    pub aim_cursor_drag_mode: i32,
    pub aim_cursor_init_pos_mode: i32,
    pub auto_aim: i32,
    pub can_aim_be_canceled: bool,
    pub aim_affects_dir: bool,
    pub aim_affects_dist: bool,
    pub auto_aim_radius_use_camera_distance_scale: bool,
    pub skip_single_operation_check: bool,
    pub skip_widget_feedback: bool,
    pub enable_view_block_move: bool,
    pub enable_view_block_move_mk: bool,
}
impl ConfigSkillCtrlTypeRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.command_type = cur.read_i32()?;
        self.on_touch = cur.read_array_i32()?;
        self.on_drag = cur.read_array_i32()?;
        self.on_hold = cur.read_array_i32()?;
        self.on_click_release = cur.read_array_i32()?;
        self.on_hold_release = cur.read_array_i32()?;
        self.on_drag_release = cur.read_array_i32()?;
        self.on_aim = cur.read_array_i32()?;
        self.on_aim_stop = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.can_aim_be_canceled = cur.read_bool()?;
        self.aim_cursor_cast_range = cur.read_i32()?;
        self.aim_cursor_direction_arrow = cur.read_i32()?;
        self.aim_cursor_aoe_range = cur.read_i32()?;
        self.auto_aiming_auxiliary_line = cur.read_i32()?;
        self.aim_affects_dir = cur.read_bool()?;
        self.aim_affects_dist = cur.read_bool()?;
        self.aim_cursor_drag_mode = cur.read_i32()?;
        self.aim_cursor_by_position_range_radius_ratio = cur.read_f64()?;
        self.aim_cursor_by_motion_world_unit_ratio = cur.read_f64()?;
        self.aim_cursor_by_motion_range_radius_factor = cur.read_f64()?;
        self.aim_cursor_init_pos_mode = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.aim_cursor_init_pos_range_radius_factor = cur.read_f64()?;
        self.aim_cursor_init_pos_aoe_range_radius_factor = cur.read_f64()?;
        self.auto_aim = cur.read_i32()?;
        self.auto_aim_radius_use_camera_distance_scale = cur.read_bool()?;
        self.touch_time_click_to_hold_threshold = cur.read_f64()?;
        self.drag_dist_touch_to_drag_threshold = cur.read_f64()?;
        self.drag_dist_drag_to_hold_threshold = cur.read_f64()?;
        self.skip_single_operation_check = cur.read_bool()?;
        self.skip_widget_feedback = cur.read_bool()?;
        self.ignore_block_move_pad_request_types = cur.read_array_i32()?;
        self.on_click_mk = cur.read_array_i32()?;
        self.on_click_release_mk = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.on_aim_mk = cur.read_array_i32()?;
        self.on_aim_stop_mk = cur.read_array_i32()?;
        self.ignore_block_move_pad_request_types_mk = cur.read_array_i32()?;
        self.block_move_types = cur.read_array_i32()?;
        self.enable_view_block_move = cur.read_bool()?;
        self.enable_view_block_move_mk = cur.read_bool()?;
        Ok(())
    }
}
impl BinRow for ConfigSkillCtrlTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_ctrl_type_ChunkA0.bin`.
pub type ConfigSkillCtrlTypeTable = BinTable<i32, ConfigSkillCtrlTypeRow>;
/// Parses `config_skill_ctrl_type_ChunkA0.bin` into a [`ConfigSkillCtrlTypeTable`].
pub fn parse_config_skill_ctrl_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillCtrlTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_skill_damage_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigSkillDamageRow {
    pub damage_effective: f64,
    pub hit: Box<[FConfigI32ArrayI32F64x3]>,
    pub dot: Box<[FConfigI32ArrayI32F64x3]>,
    pub scondary: Box<[FConfigI32ArrayI32F64x3]>,
    pub true_damage: Box<[FConfigI32ArrayI32F64x3]>,
    pub affix: Box<[FConfigAffix]>,
    pub des: Box<str>,
    pub id: i32,
    pub level: i32,
    pub r#type: i32,
}
impl BinRow for ConfigSkillDamageRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.level = cur.read_i32()?;
        self.damage_effective = cur.read_f64()?;
        self.r#type = cur.read_i32()?;
        self.hit = FConfigI32ArrayI32F64x3::read_array(cur)?;
        self.dot = FConfigI32ArrayI32F64x3::read_array(cur)?;
        self.scondary = FConfigI32ArrayI32F64x3::read_array(cur)?;
        self.true_damage = FConfigI32ArrayI32F64x3::read_array(cur)?;
        self.affix = FConfigAffix::read_array(cur)?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_damage_ChunkA0.bin`.
pub type ConfigSkillDamageTable = BinTable<(i32, i32), ConfigSkillDamageRow>;
/// Parses `config_skill_damage_ChunkA0.bin` into a [`ConfigSkillDamageTable`].
pub fn parse_config_skill_damage(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillDamageTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_skill_experience_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillExperienceRow {
    pub remark: Box<str>,
    pub id: i32,
    pub experience: i32,
}
impl BinRow for ConfigSkillExperienceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.experience = cur.read_i32()?;
        self.remark = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_experience_ChunkA0.bin`.
pub type ConfigSkillExperienceTable = BinTable<i32, ConfigSkillExperienceRow>;
/// Parses `config_skill_experience_ChunkA0.bin` into a [`ConfigSkillExperienceTable`].
pub fn parse_config_skill_experience(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillExperienceTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_skill_pos_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillPosRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub skill_id: Box<[i32]>,
    pub id: i32,
    pub r#type: i32,
    pub group: i32,
    pub index: i32,
}
impl BinRow for ConfigSkillPosRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.r#type = cur.read_i32()?;
        self.group = cur.read_i32()?;
        self.index = cur.read_i32()?;
        self.skill_id = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_pos_ChunkA0.bin`.
pub type ConfigSkillPosTable = BinTable<i32, ConfigSkillPosRow>;
/// Parses `config_skill_pos_ChunkA0.bin` into a [`ConfigSkillPosTable`].
pub fn parse_config_skill_pos(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillPosTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_skill_season_vfx_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillSeasonVfxRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub suffix: Box<str>,
    pub id: i32,
    pub mode: i32,
}
impl BinRow for ConfigSkillSeasonVfxRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.mode = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.suffix = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_season_vfx_ChunkA0.bin`.
pub type ConfigSkillSeasonVfxTable = BinTable<i32, ConfigSkillSeasonVfxRow>;
/// Parses `config_skill_season_vfx_ChunkA0.bin` into a [`ConfigSkillSeasonVfxTable`].
pub fn parse_config_skill_season_vfx(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillSeasonVfxTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_skill_slots_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillSlotsRow {
    pub r#type: Box<[i32]>,
    pub icon: Box<str>,
    pub recommonded_skill_tag: Box<[i32]>,
    pub id: i32,
    pub skill_id: i32,
    pub slot_id: i32,
    pub unlock_level: i32,
    pub unlock_cost: i32,
    pub energy: i32,
    pub default_unlock: i32,
}
impl BinRow for ConfigSkillSlotsRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.skill_id = cur.read_i32()?;
        self.slot_id = cur.read_i32()?;
        self.r#type = cur.read_array_i32()?;
        self.icon = cur.read_str()?;
        self.unlock_level = cur.read_i32()?;
        self.unlock_cost = cur.read_i32()?;
        self.energy = cur.read_i32()?;
        self.default_unlock = cur.read_i32()?;
        self.recommonded_skill_tag = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_slots_ChunkA0.bin`.
pub type ConfigSkillSlotsTable = BinTable<i32, ConfigSkillSlotsRow>;
/// Parses `config_skill_slots_ChunkA0.bin` into a [`ConfigSkillSlotsTable`].
pub fn parse_config_skill_slots(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillSlotsTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_skill_stone_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigSkillStoneRow {
    pub name: Box<str>,
    pub slots_open_cost: Box<str>,
    pub skill_id: Box<[i32]>,
    pub mp_cost: Box<[FConfigI32Array2dI32F64]>,
    pub mp_cost_in_running: Box<[FConfigI32Array2dI32F64]>,
    pub mp_reserved: Box<[FConfigI32Array2dI32F64]>,
    pub mp_multiplier: Box<[FConfigI32I32]>,
    pub damage_effective: Box<[FConfigI32Array2dI32F64]>,
    pub tag: Box<[i32]>,
    pub added_tag: Box<[i32]>,
    pub removed_tag: Box<[i32]>,
    pub allow_tag: Box<[Box<[i32]>]>,
    pub excluded_tag: Box<[Box<[i32]>]>,
    pub use_level: Box<[FConfigI32I32]>,
    pub weapon: FConfigBoolArrayI32ArrayI32,
    pub weapon_restrict_description: Box<str>,
    pub affix: Box<[FConfigAffix]>,
    pub additional_affix: Box<[FConfigAffix]>,
    pub icon: Box<str>,
    pub fight_skillmode_icons: FxHashMap<i32, Box<str>>,
    pub ex_description: Box<str>,
    pub dmg_description: Box<[i32]>,
    pub buff_effect_description: Box<[i32]>,
    pub summon_npc_skill_id: Box<[i32]>,
    pub skill_energy: Box<[f64]>,
    pub energy_cost: FxHashMap<i32, Box<[f64]>>,
    pub simple_skill_des_id: Box<[i32]>,
    pub detail_skill_des_id: Box<[i32]>,
    pub recommonded_tag_group: Box<[i32]>,
    pub skillbag_tag_group: Box<[i32]>,
    pub skill_can_install_pos: Box<[i32]>,
    pub type6_add_modifier: Box<[FConfigAffix]>,
    pub id: i32,
    pub r#type: i32,
    pub reserved_type: i32,
    pub max_slots: i32,
    pub max_connection: i32,
    pub crit_value: i32,
    pub summon_npc_id: i32,
    pub novice_teaching: i32,
    pub skill_group: i32,
    pub type6_add_modifier_level: i32,
    pub type6_level_up_group: i32,
}
impl ConfigSkillStoneRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.r#type = cur.read_i32()?;
        self.reserved_type = cur.read_i32()?;
        self.max_slots = cur.read_i32()?;
        self.slots_open_cost = cur.read_str()?;
        self.max_connection = cur.read_i32()?;
        self.skill_id = cur.read_array_i32()?;
        self.mp_cost = FConfigI32Array2dI32F64::read_array(cur)?;
        self.mp_cost_in_running = FConfigI32Array2dI32F64::read_array(cur)?;
        self.mp_reserved = FConfigI32Array2dI32F64::read_array(cur)?;
        self.mp_multiplier = FConfigI32I32::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.damage_effective = FConfigI32Array2dI32F64::read_array(cur)?;
        self.tag = cur.read_array_i32()?;
        self.added_tag = cur.read_array_i32()?;
        self.removed_tag = cur.read_array_i32()?;
        self.allow_tag = cur.read_array_2d_i32()?;
        self.excluded_tag = cur.read_array_2d_i32()?;
        self.use_level = FConfigI32I32::read_array(cur)?;
        self.weapon = FConfigBoolArrayI32ArrayI32::read(cur)?;
        self.weapon_restrict_description = cur.read_str()?;
        self.crit_value = cur.read_i32()?;
        self.affix = FConfigAffix::read_array(cur)?;
        self.additional_affix = FConfigAffix::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.icon = cur.read_str()?;
        self.fight_skillmode_icons = cur.read_map_i32_str()?;
        self.ex_description = cur.read_str()?;
        self.dmg_description = cur.read_array_i32()?;
        self.buff_effect_description = cur.read_array_i32()?;
        self.summon_npc_id = cur.read_i32()?;
        self.summon_npc_skill_id = cur.read_array_i32()?;
        self.novice_teaching = cur.read_i32()?;
        self.skill_energy = cur.read_array_f64()?;
        self.energy_cost = cur.read_map_i32_array_f64()?;
        self.skill_group = cur.read_i32()?;
        self.simple_skill_des_id = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.detail_skill_des_id = cur.read_array_i32()?;
        self.recommonded_tag_group = cur.read_array_i32()?;
        self.skillbag_tag_group = cur.read_array_i32()?;
        self.skill_can_install_pos = cur.read_array_i32()?;
        self.type6_add_modifier_level = cur.read_i32()?;
        self.type6_add_modifier = FConfigAffix::read_array(cur)?;
        self.type6_level_up_group = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigSkillStoneRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_stone_ChunkA0.bin`.
pub type ConfigSkillStoneTable = BinTable<i32, ConfigSkillStoneRow>;
/// Parses `config_skill_stone_ChunkA0.bin` into a [`ConfigSkillStoneTable`].
pub fn parse_config_skill_stone(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillStoneTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_skill_stone_summon_npc_index_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillStoneSummonNpcIndexRow {
    pub ids: Box<[i32]>,
    pub summon_npc_id: i32,
}
impl BinRow for ConfigSkillStoneSummonNpcIndexRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.summon_npc_id = cur.read_i32()?;
        self.ids = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_stone_summon_npc_index_ChunkA0.bin`.
pub type ConfigSkillStoneSummonNpcIndexTable = BinTable<
    i32,
    ConfigSkillStoneSummonNpcIndexRow,
>;
/// Parses `config_skill_stone_summon_npc_index_ChunkA0.bin` into a [`ConfigSkillStoneSummonNpcIndexTable`].
pub fn parse_config_skill_stone_summon_npc_index(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillStoneSummonNpcIndexTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_skill_tag_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillTagRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub tag_type: Box<str>,
    pub des2: Box<str>,
    pub vfx: FxHashMap<Box<str>, Box<[i32]>>,
    pub id: i32,
    pub visible: i32,
    pub tag_classify: i32,
    pub tag_order: i32,
    pub tag_recommond_order: i32,
    pub to_ailment: bool,
    pub to_affix1011: bool,
    pub to_disable: bool,
}
impl ConfigSkillTagRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.to_ailment = cur.read_bool()?;
        self.to_affix1011 = cur.read_bool()?;
        self.to_disable = cur.read_bool()?;
        self.visible = cur.read_i32()?;
        self.tag_type = cur.read_str()?;
        self.tag_classify = cur.read_i32()?;
        self.tag_order = cur.read_i32()?;
        self.tag_recommond_order = cur.read_i32()?;
        self.des2 = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.vfx = cur.read_map_str_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigSkillTagRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_skill_tag_ChunkA0.bin`.
pub type ConfigSkillTagTable = BinTable<i32, ConfigSkillTagRow>;
/// Parses `config_skill_tag_ChunkA0.bin` into a [`ConfigSkillTagTable`].
pub fn parse_config_skill_tag(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSkillTagTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
#[inline(never)]
fn verify_skill_chunk_0(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_skill_anime_template" => {
            Some(parse_config_skill_anime_template(buf, scratch).map(|t| t.len()))
        }
        "config_skill_anime_type" => {
            Some(parse_config_skill_anime_type(buf, scratch).map(|t| t.len()))
        }
        "config_skill_ctrl_type" => {
            Some(parse_config_skill_ctrl_type(buf, scratch).map(|t| t.len()))
        }
        "config_skill_damage" => {
            Some(parse_config_skill_damage(buf, scratch).map(|t| t.len()))
        }
        "config_skill_experience" => {
            Some(parse_config_skill_experience(buf, scratch).map(|t| t.len()))
        }
        "config_skill_pos" => Some(parse_config_skill_pos(buf, scratch).map(|t| t.len())),
        "config_skill_season_vfx" => {
            Some(parse_config_skill_season_vfx(buf, scratch).map(|t| t.len()))
        }
        "config_skill_slots" => {
            Some(parse_config_skill_slots(buf, scratch).map(|t| t.len()))
        }
        "config_skill_stone" => {
            Some(parse_config_skill_stone(buf, scratch).map(|t| t.len()))
        }
        "config_skill_stone_summon_npc_index" => {
            Some(
                parse_config_skill_stone_summon_npc_index(buf, scratch).map(|t| t.len()),
            )
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_skill_chunk_1(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_skill_tag" => Some(parse_config_skill_tag(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}
pub(crate) fn verify_skill_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    if let Some(res) = verify_skill_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_skill_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    None
}

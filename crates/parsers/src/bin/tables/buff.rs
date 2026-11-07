//! Auto-generated `.bin` configuration table types and parsers for the `buff` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::action_data::ActionData;
use crate::bin::tables::substructs::{FConfigI32F64, FConfigI32I32, FConfigStrArrayI32};
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_char_buff_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigCharBuffRow {
    pub r#enum: Box<str>,
    pub name: Box<str>,
    pub des: Box<str>,
    pub buff_tag: Box<[i32]>,
    pub added_state: Box<[i32]>,
    pub tick_interval: f64,
    pub max_duration: f64,
    pub increase_caster_time: Box<[i32]>,
    pub increase_caster_effect: Box<[i32]>,
    pub increase_target_time: Box<[i32]>,
    pub increase_target_effect: Box<[i32]>,
    pub active_limit: Box<[FConfigI32I32]>,
    pub default_duration: Box<[FConfigI32F64]>,
    pub action_args: Box<str>,
    pub base_affix: Box<[FConfigAffix]>,
    pub view_icon: Box<str>,
    pub auto_show_tips: FConfigI32F64,
    pub sound: Box<[FConfigStrArrayI32]>,
    pub action_args_data: Box<str>,
    pub action_data: ActionData,
    pub hero_ids_only_play_local_player_fx: Box<[i32]>,
    pub description: Box<str>,
    pub id: i32,
    pub mode: i32,
    pub buff_type: i32,
    pub added_label: i32,
    pub layer_type: i32,
    pub active_limit_all_in_one: i32,
    pub active_limit_max: i32,
    pub active_limit_rule: i32,
    pub duration_rule: i32,
    pub duration_show_rule: i32,
    pub buff_group: i32,
    pub buff_group_priority: i32,
    pub action_id: i32,
    pub icon_order: i32,
    pub vfx: i32,
    pub skip_multiple: bool,
    pub skip_immune: bool,
    pub skip_affix_event: bool,
    pub skip_reflect: bool,
    pub skip_transfer: bool,
    pub skip_spread: bool,
    pub refresh_effect_scale: bool,
    pub active_stack_by_reuse: bool,
    pub effective_immediately: bool,
    pub propensity: bool,
    pub remove_vfx_on_dead: bool,
}
impl ConfigCharBuffRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.mode = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.name = cur.read_str()?;
        self.des = cur.read_str()?;
        self.skip_multiple = cur.read_bool()?;
        self.skip_immune = cur.read_bool()?;
        self.skip_affix_event = cur.read_bool()?;
        self.skip_reflect = cur.read_bool()?;
        self.skip_transfer = cur.read_bool()?;
        self.skip_spread = cur.read_bool()?;
        self.buff_type = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.buff_tag = cur.read_array_i32()?;
        self.added_label = cur.read_i32()?;
        self.added_state = cur.read_array_i32()?;
        self.tick_interval = cur.read_f64()?;
        self.max_duration = cur.read_f64()?;
        self.refresh_effect_scale = cur.read_bool()?;
        self.increase_caster_time = cur.read_array_i32()?;
        self.increase_caster_effect = cur.read_array_i32()?;
        self.increase_target_time = cur.read_array_i32()?;
        self.increase_target_effect = cur.read_array_i32()?;
        self.layer_type = cur.read_i32()?;
        self.active_limit_all_in_one = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.active_limit = FConfigI32I32::read_array(cur)?;
        self.active_limit_max = cur.read_i32()?;
        self.active_limit_rule = cur.read_i32()?;
        self.active_stack_by_reuse = cur.read_bool()?;
        self.duration_rule = cur.read_i32()?;
        self.duration_show_rule = cur.read_i32()?;
        self.buff_group = cur.read_i32()?;
        self.buff_group_priority = cur.read_i32()?;
        self.effective_immediately = cur.read_bool()?;
        self.default_duration = FConfigI32F64::read_array(cur)?;
        self.action_id = cur.read_i32()?;
        self.action_args = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.base_affix = FConfigAffix::read_array(cur)?;
        self.propensity = cur.read_bool()?;
        self.view_icon = cur.read_str()?;
        self.icon_order = cur.read_i32()?;
        self.auto_show_tips = FConfigI32F64::read(cur)?;
        self.sound = FConfigStrArrayI32::read_array(cur)?;
        self.remove_vfx_on_dead = cur.read_bool()?;
        self.vfx = cur.read_i32()?;
        self.action_args_data = cur.read_str()?;
        self.action_data = ActionData::read(cur)?;
        self.hero_ids_only_play_local_player_fx = cur.read_array_i32()?;
        self.description = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigCharBuffRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_char_buff_ChunkA0.bin`.
pub type ConfigCharBuffTable = BinTable<(i32, i32), ConfigCharBuffRow>;
/// Parses `config_char_buff_ChunkA0.bin` into a [`ConfigCharBuffTable`].
pub fn parse_config_char_buff(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCharBuffTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_char_buff_tag_index_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCharBuffTagIndexRow {
    pub ids: Box<[i32]>,
    pub modes: Box<[i32]>,
    pub buff_tag: i32,
}
impl BinRow for ConfigCharBuffTagIndexRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.buff_tag = cur.read_i32()?;
        self.ids = cur.read_array_i32()?;
        self.modes = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_char_buff_tag_index_ChunkA0.bin`.
pub type ConfigCharBuffTagIndexTable = BinTable<i32, ConfigCharBuffTagIndexRow>;
/// Parses `config_char_buff_tag_index_ChunkA0.bin` into a [`ConfigCharBuffTagIndexTable`].
pub fn parse_config_char_buff_tag_index(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCharBuffTagIndexTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_char_buff_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCharBuffTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
    pub max_count: i32,
    pub limit_rule: i32,
    pub limit_field: i32,
}
impl BinRow for ConfigCharBuffTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.max_count = cur.read_i32()?;
        self.limit_rule = cur.read_i32()?;
        self.limit_field = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_char_buff_type_ChunkA0.bin`.
pub type ConfigCharBuffTypeTable = BinTable<i32, ConfigCharBuffTypeRow>;
/// Parses `config_char_buff_type_ChunkA0.bin` into a [`ConfigCharBuffTypeTable`].
pub fn parse_config_char_buff_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCharBuffTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_char_buff_vfx_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigCharBuffVfxRow {
    pub begin: Box<[i32]>,
    pub begin_to_end: Box<[i32]>,
    pub end: Box<[i32]>,
    pub trigger: Box<[i32]>,
    pub trigger_interval: f64,
    pub max: Box<[i32]>,
    pub cost: Box<[i32]>,
    pub cost_max: Box<[i32]>,
    pub layer_params: FxHashMap<Box<str>, Box<[i32]>>,
    pub id: i32,
    pub hero_id: i32,
    pub stack_num: i32,
    pub view_group: i32,
    pub stackable: bool,
    pub repeat_begin: bool,
    pub force_begin: bool,
}
impl ConfigCharBuffVfxRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.hero_id = cur.read_i32()?;
        self.stack_num = cur.read_i32()?;
        self.view_group = cur.read_i32()?;
        self.stackable = cur.read_bool()?;
        self.repeat_begin = cur.read_bool()?;
        self.force_begin = cur.read_bool()?;
        self.begin = cur.read_array_i32()?;
        self.begin_to_end = cur.read_array_i32()?;
        self.end = cur.read_array_i32()?;
        self.trigger = cur.read_array_i32()?;
        self.trigger_interval = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.max = cur.read_array_i32()?;
        self.cost = cur.read_array_i32()?;
        self.cost_max = cur.read_array_i32()?;
        self.layer_params = cur.read_map_str_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigCharBuffVfxRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_char_buff_vfx_ChunkA0.bin`.
pub type ConfigCharBuffVfxTable = BinTable<(i32, i32, i32), ConfigCharBuffVfxRow>;
/// Parses `config_char_buff_vfx_ChunkA0.bin` into a [`ConfigCharBuffVfxTable`].
pub fn parse_config_char_buff_vfx(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCharBuffVfxTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_char_buff_view_group_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigCharBuffViewGroupRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub scale_num: Box<[f64]>,
    pub rotate_num: Box<[i32]>,
    pub init_rotation: Box<[i32]>,
    pub revolve_radius: Box<[i32]>,
    pub revolve_angle_speed: Box<[i32]>,
    pub revolve_plane_angle: Box<[i32]>,
    pub id: i32,
    pub max_count: i32,
    pub limit_rule: i32,
    pub revolve_vfx_num: i32,
    pub revolve: bool,
    pub revolve_single_track: bool,
    pub revolve_clockwise: bool,
    pub revolve_vfx_repeat: bool,
}
impl ConfigCharBuffViewGroupRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.max_count = cur.read_i32()?;
        self.limit_rule = cur.read_i32()?;
        self.scale_num = cur.read_array_f64()?;
        self.rotate_num = cur.read_array_i32()?;
        self.init_rotation = cur.read_array_i32()?;
        self.revolve = cur.read_bool()?;
        self.revolve_single_track = cur.read_bool()?;
        self.revolve_radius = cur.read_array_i32()?;
        self.revolve_clockwise = cur.read_bool()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.revolve_angle_speed = cur.read_array_i32()?;
        self.revolve_plane_angle = cur.read_array_i32()?;
        self.revolve_vfx_num = cur.read_i32()?;
        self.revolve_vfx_repeat = cur.read_bool()?;
        Ok(())
    }
}
impl BinRow for ConfigCharBuffViewGroupRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_char_buff_view_group_ChunkA0.bin`.
pub type ConfigCharBuffViewGroupTable = BinTable<i32, ConfigCharBuffViewGroupRow>;
/// Parses `config_char_buff_view_group_ChunkA0.bin` into a [`ConfigCharBuffViewGroupTable`].
pub fn parse_config_char_buff_view_group(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCharBuffViewGroupTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_buff_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_char_buff" => Some(parse_config_char_buff(buf, scratch).map(|t| t.len())),
        "config_char_buff_tag_index" => {
            Some(parse_config_char_buff_tag_index(buf, scratch).map(|t| t.len()))
        }
        "config_char_buff_type" => {
            Some(parse_config_char_buff_type(buf, scratch).map(|t| t.len()))
        }
        "config_char_buff_vfx" => {
            Some(parse_config_char_buff_vfx(buf, scratch).map(|t| t.len()))
        }
        "config_char_buff_view_group" => {
            Some(parse_config_char_buff_view_group(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

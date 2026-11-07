//! Auto-generated `.bin` configuration table types and parsers for the `map` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{
    FConfigI32ArrayI32, FConfigI32ArrayI32ArrayI32F64F64, FConfigI32F64, FConfigI32I32,
};
use crate::bin::types::{BinTable, VecDim};
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_map_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapRow {
    pub model: Box<str>,
    pub id: i32,
    pub test_level: i32,
}
impl BinRow for ConfigMapRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.model = cur.read_str()?;
        self.test_level = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_ChunkA0.bin`.
pub type ConfigMapTable = BinTable<i32, ConfigMapRow>;
/// Parses `config_map_ChunkA0.bin` into a [`ConfigMapTable`].
pub fn parse_config_map(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_affix_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapAffixRow {
    pub item_quantity: f64,
    pub item_rarity: f64,
    pub pack_size: f64,
    pub rarity: Box<[i32]>,
    pub id: i32,
    pub target: i32,
    pub danger: i32,
}
impl BinRow for ConfigMapAffixRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.item_quantity = cur.read_f64()?;
        self.item_rarity = cur.read_f64()?;
        self.pack_size = cur.read_f64()?;
        self.target = cur.read_i32()?;
        self.rarity = cur.read_array_i32()?;
        self.danger = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_affix_ChunkA0.bin`.
pub type ConfigMapAffixTable = BinTable<i32, ConfigMapAffixRow>;
/// Parses `config_map_affix_ChunkA0.bin` into a [`ConfigMapAffixTable`].
pub fn parse_config_map_affix(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapAffixTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_common_pos_cfg_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapCommonPosCfgRow {
    pub position: VecDim,
    pub id: i32,
    pub yaw: i32,
    pub r#type: i32,
    pub cfg_id: i32,
}
impl BinRow for ConfigMapCommonPosCfgRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.position = cur.read_vec_dim()?;
        self.yaw = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.cfg_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_common_pos_cfg_ChunkA0.bin`.
pub type ConfigMapCommonPosCfgTable = BinTable<i32, ConfigMapCommonPosCfgRow>;
/// Parses `config_map_common_pos_cfg_ChunkA0.bin` into a [`ConfigMapCommonPosCfgTable`].
pub fn parse_config_map_common_pos_cfg(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapCommonPosCfgTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_custom_trap_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapCustomTrapRow {
    pub name: Box<str>,
    pub des: Box<str>,
    pub shape: Box<[i32]>,
    pub status_loop: Box<[i32]>,
    pub id: i32,
    pub resource_id: i32,
    pub collision: i32,
    pub init_status: i32,
    pub max_action_times: i32,
    pub for_sequence_hide: i32,
    pub interaction_range: i32,
    pub interactive_icon_id: i32,
    pub trap_type: i32,
    pub performance_actor_id: i32,
    pub creator_interact_only: bool,
    pub show_name: bool,
    pub movable: bool,
}
impl ConfigMapCustomTrapRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.des = cur.read_str()?;
        self.resource_id = cur.read_i32()?;
        self.collision = cur.read_i32()?;
        self.shape = cur.read_array_i32()?;
        self.status_loop = cur.read_array_i32()?;
        self.init_status = cur.read_i32()?;
        self.max_action_times = cur.read_i32()?;
        self.for_sequence_hide = cur.read_i32()?;
        self.interaction_range = cur.read_i32()?;
        self.interactive_icon_id = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.trap_type = cur.read_i32()?;
        self.creator_interact_only = cur.read_bool()?;
        self.show_name = cur.read_bool()?;
        self.movable = cur.read_bool()?;
        self.performance_actor_id = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapCustomTrapRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_custom_trap_ChunkA0.bin`.
pub type ConfigMapCustomTrapTable = BinTable<i32, ConfigMapCustomTrapRow>;
/// Parses `config_map_custom_trap_ChunkA0.bin` into a [`ConfigMapCustomTrapTable`].
pub fn parse_config_map_custom_trap(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapCustomTrapTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_custom_trap_status_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapCustomTrapStatusRow {
    pub animations: FxHashMap<Box<str>, Box<[i32]>>,
    pub trap_attr: Box<[FConfigI32ArrayI32ArrayI32F64F64]>,
    pub trap_trigger: Box<[i32]>,
    pub trap_skill: Box<[i32]>,
    pub be_attack_recover_time: f64,
    pub interaction_time: f64,
    pub init_delay: f64,
    pub init_show: f64,
    pub action_interval: Box<[f64]>,
    pub gameplays: Box<[i32]>,
    pub ui_operate_param: Box<str>,
    pub interaction_cd: f64,
    pub trap_id: i32,
    pub status_id: i32,
    pub target: i32,
    pub effect: i32,
    pub change_mode: i32,
    pub next_status: i32,
    pub be_attack: i32,
    pub be_attach_recover_status: i32,
    pub interaction: i32,
    pub interaction_icon: i32,
    pub interaction_effect: i32,
    pub action_model: i32,
    pub side: i32,
    pub max_action_times: i32,
    pub skeleton: i32,
    pub interaction_open_ui: i32,
    pub max_interaction_count: i32,
    pub ai: i32,
    pub icon_visible: bool,
    pub head_icon_visible: bool,
}
impl ConfigMapCustomTrapStatusRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.trap_id = cur.read_i32()?;
        self.status_id = cur.read_i32()?;
        self.target = cur.read_i32()?;
        self.animations = cur.read_map_str_array_i32()?;
        self.effect = cur.read_i32()?;
        self.trap_attr = FConfigI32ArrayI32ArrayI32F64F64::read_array(cur)?;
        self.trap_trigger = cur.read_array_i32()?;
        self.trap_skill = cur.read_array_i32()?;
        self.change_mode = cur.read_i32()?;
        self.next_status = cur.read_i32()?;
        self.be_attack = cur.read_i32()?;
        self.be_attack_recover_time = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.be_attach_recover_status = cur.read_i32()?;
        self.interaction = cur.read_i32()?;
        self.interaction_icon = cur.read_i32()?;
        self.interaction_effect = cur.read_i32()?;
        self.interaction_time = cur.read_f64()?;
        self.init_delay = cur.read_f64()?;
        self.init_show = cur.read_f64()?;
        self.action_model = cur.read_i32()?;
        self.action_interval = cur.read_array_f64()?;
        self.side = cur.read_i32()?;
        self.max_action_times = cur.read_i32()?;
        self.icon_visible = cur.read_bool()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.skeleton = cur.read_i32()?;
        self.interaction_open_ui = cur.read_i32()?;
        self.gameplays = cur.read_array_i32()?;
        self.ui_operate_param = cur.read_str()?;
        self.head_icon_visible = cur.read_bool()?;
        self.interaction_cd = cur.read_f64()?;
        self.max_interaction_count = cur.read_i32()?;
        self.ai = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapCustomTrapStatusRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_custom_trap_status_ChunkA0.bin`.
pub type ConfigMapCustomTrapStatusTable = BinTable<
    (i32, i32),
    ConfigMapCustomTrapStatusRow,
>;
/// Parses `config_map_custom_trap_status_ChunkA0.bin` into a [`ConfigMapCustomTrapStatusTable`].
pub fn parse_config_map_custom_trap_status(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapCustomTrapStatusTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_destructible_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapDestructibleRow {
    pub des: Box<str>,
    pub shape: Box<[i32]>,
    pub resource_path: Box<str>,
    pub resource_material: Box<str>,
    pub sound_id: Box<str>,
    pub id: i32,
    pub shape_type: i32,
    pub collision_mode: i32,
    pub side: i32,
    pub durability: i32,
    pub drop_group: i32,
    pub vfx_id: i32,
    pub alive_vfx_id: i32,
    pub allow_walk: bool,
    pub allow_charge: bool,
    pub allow_jump: bool,
    pub allow_blink: bool,
    pub allow_bullet_normal: bool,
    pub allow_bullet_parabolic: bool,
    pub allow_bullet_floor: bool,
    pub allow_view: bool,
    pub resource_hide: bool,
}
impl ConfigMapDestructibleRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.shape_type = cur.read_i32()?;
        self.shape = cur.read_array_i32()?;
        self.collision_mode = cur.read_i32()?;
        self.side = cur.read_i32()?;
        self.allow_walk = cur.read_bool()?;
        self.allow_charge = cur.read_bool()?;
        self.allow_jump = cur.read_bool()?;
        self.allow_blink = cur.read_bool()?;
        self.allow_bullet_normal = cur.read_bool()?;
        self.allow_bullet_parabolic = cur.read_bool()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.allow_bullet_floor = cur.read_bool()?;
        self.allow_view = cur.read_bool()?;
        self.durability = cur.read_i32()?;
        self.drop_group = cur.read_i32()?;
        self.resource_path = cur.read_str()?;
        self.resource_hide = cur.read_bool()?;
        self.resource_material = cur.read_str()?;
        self.sound_id = cur.read_str()?;
        self.vfx_id = cur.read_i32()?;
        self.alive_vfx_id = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapDestructibleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_destructible_ChunkA0.bin`.
pub type ConfigMapDestructibleTable = BinTable<i32, ConfigMapDestructibleRow>;
/// Parses `config_map_destructible_ChunkA0.bin` into a [`ConfigMapDestructibleTable`].
pub fn parse_config_map_destructible(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapDestructibleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_destructible_avengers_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapDestructibleAvengersRow {
    pub des: Box<str>,
    pub trap_attr: Box<[FConfigI32ArrayI32ArrayI32F64F64]>,
    pub trap_skill: Box<[i32]>,
    pub id: i32,
    pub destructible_id: i32,
    pub target: i32,
}
impl BinRow for ConfigMapDestructibleAvengersRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.destructible_id = cur.read_i32()?;
        self.trap_attr = FConfigI32ArrayI32ArrayI32F64F64::read_array(cur)?;
        self.trap_skill = cur.read_array_i32()?;
        self.target = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_destructible_avengers_ChunkA0.bin`.
pub type ConfigMapDestructibleAvengersTable = BinTable<
    i32,
    ConfigMapDestructibleAvengersRow,
>;
/// Parses `config_map_destructible_avengers_ChunkA0.bin` into a [`ConfigMapDestructibleAvengersTable`].
pub fn parse_config_map_destructible_avengers(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapDestructibleAvengersTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_follower_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapFollowerRow {
    pub des: Box<str>,
    pub init_delay: Box<[f64]>,
    pub action_interval: Box<[f64]>,
    pub trap_attr: Box<[FConfigI32ArrayI32ArrayI32F64F64]>,
    pub weights: FxHashMap<i32, i32>,
    pub pack_interval: Box<[f64]>,
    pub trap_skill: Box<[FConfigI32I32]>,
    pub id: i32,
    pub init_way: i32,
    pub max_action_times: i32,
    pub pack_times: i32,
    pub pack_skill_count: i32,
    pub max_limit: i32,
}
impl ConfigMapFollowerRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.init_way = cur.read_i32()?;
        self.init_delay = cur.read_array_f64()?;
        self.max_action_times = cur.read_i32()?;
        self.action_interval = cur.read_array_f64()?;
        self.trap_attr = FConfigI32ArrayI32ArrayI32F64F64::read_array(cur)?;
        self.weights = cur.read_map_i32_i32()?;
        self.pack_times = cur.read_i32()?;
        self.pack_interval = cur.read_array_f64()?;
        self.pack_skill_count = cur.read_i32()?;
        self.trap_skill = FConfigI32I32::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.max_limit = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapFollowerRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_follower_ChunkA0.bin`.
pub type ConfigMapFollowerTable = BinTable<i32, ConfigMapFollowerRow>;
/// Parses `config_map_follower_ChunkA0.bin` into a [`ConfigMapFollowerTable`].
pub fn parse_config_map_follower(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapFollowerTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_actors_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplayActorsRow {
    pub name: Box<str>,
    pub gameplays: Box<[i32]>,
    pub desc: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigMapGameplayActorsRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.gameplays = cur.read_array_i32()?;
        self.desc = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_actors_ChunkA0.bin`.
pub type ConfigMapGameplayActorsTable = BinTable<i32, ConfigMapGameplayActorsRow>;
/// Parses `config_map_gameplay_actors_ChunkA0.bin` into a [`ConfigMapGameplayActorsTable`].
pub fn parse_config_map_gameplay_actors(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayActorsTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_god_of_darkness_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplayGodOfDarknessRow {
    pub prepare_time: f64,
    pub out_radius_time: f64,
    pub monster_weight: Box<[FConfigI32I32]>,
    pub skill: FxHashMap<i32, i32>,
    pub skill_open_lv: Box<[FConfigI32I32]>,
    pub buff: Box<[i32]>,
    pub lv_times: Box<[f64]>,
    pub lv_skill_nums: Box<[i32]>,
    pub lv_tales: Box<[FConfigI32I32]>,
    pub reward: Box<[FConfigI32I32]>,
    pub end_dialog_id: Box<[i32]>,
    pub drops: FxHashMap<i32, i32>,
    pub level_to_affix: Box<[FConfigAffix]>,
    pub id: i32,
    pub open_trap_id: i32,
    pub open_dialog_id: i32,
    pub camera_up_offset: i32,
    pub camera_up_time: i32,
    pub camera_down_offset: i32,
    pub camera_down_time: i32,
    pub area_radius: i32,
    pub area_effect_begin_vfx: i32,
    pub area_effect_loop_vfx: i32,
    pub area_effect_end_vfx: i32,
    pub monster_num: i32,
    pub monster_create_radius: i32,
    pub moneter_reborn_time: i32,
    pub base_lv: i32,
    pub content_id: i32,
}
impl ConfigMapGameplayGodOfDarknessRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.open_trap_id = cur.read_i32()?;
        self.open_dialog_id = cur.read_i32()?;
        self.prepare_time = cur.read_f64()?;
        self.camera_up_offset = cur.read_i32()?;
        self.camera_up_time = cur.read_i32()?;
        self.camera_down_offset = cur.read_i32()?;
        self.camera_down_time = cur.read_i32()?;
        self.area_radius = cur.read_i32()?;
        self.area_effect_begin_vfx = cur.read_i32()?;
        self.area_effect_loop_vfx = cur.read_i32()?;
        self.area_effect_end_vfx = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.out_radius_time = cur.read_f64()?;
        self.monster_weight = FConfigI32I32::read_array(cur)?;
        self.monster_num = cur.read_i32()?;
        self.monster_create_radius = cur.read_i32()?;
        self.moneter_reborn_time = cur.read_i32()?;
        self.skill = cur.read_map_i32_i32()?;
        self.skill_open_lv = FConfigI32I32::read_array(cur)?;
        self.buff = cur.read_array_i32()?;
        self.lv_times = cur.read_array_f64()?;
        self.lv_skill_nums = cur.read_array_i32()?;
        self.lv_tales = FConfigI32I32::read_array(cur)?;
        self.reward = FConfigI32I32::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.end_dialog_id = cur.read_array_i32()?;
        self.drops = cur.read_map_i32_i32()?;
        self.level_to_affix = FConfigAffix::read_array(cur)?;
        self.base_lv = cur.read_i32()?;
        self.content_id = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplayGodOfDarknessRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_god_of_darkness_ChunkA0.bin`.
pub type ConfigMapGameplayGodOfDarknessTable = BinTable<
    i32,
    ConfigMapGameplayGodOfDarknessRow,
>;
/// Parses `config_map_gameplay_god_of_darkness_ChunkA0.bin` into a [`ConfigMapGameplayGodOfDarknessTable`].
pub fn parse_config_map_gameplay_god_of_darkness(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayGodOfDarknessTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_god_of_darkness_king_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplayGodOfDarknessKingRow {
    pub boss_buff: Box<[i32]>,
    pub level_to_affix: Box<[FConfigAffix]>,
    pub ground_skill_interval: f64,
    pub id: i32,
    pub open_monster_id: i32,
    pub active_lv: i32,
}
impl BinRow for ConfigMapGameplayGodOfDarknessKingRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.open_monster_id = cur.read_i32()?;
        self.boss_buff = cur.read_array_i32()?;
        self.active_lv = cur.read_i32()?;
        self.level_to_affix = FConfigAffix::read_array(cur)?;
        self.ground_skill_interval = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_god_of_darkness_king_ChunkA0.bin`.
pub type ConfigMapGameplayGodOfDarknessKingTable = BinTable<
    i32,
    ConfigMapGameplayGodOfDarknessKingRow,
>;
/// Parses `config_map_gameplay_god_of_darkness_king_ChunkA0.bin` into a [`ConfigMapGameplayGodOfDarknessKingTable`].
pub fn parse_config_map_gameplay_god_of_darkness_king(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayGodOfDarknessKingTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_god_of_hunting_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplayGodOfHuntingRow {
    pub begin_dialog_id: Box<[i32]>,
    pub end_dialog_id: Box<[i32]>,
    pub buff: Box<[i32]>,
    pub buff_time: f64,
    pub reward: Box<[FConfigI32I32]>,
    pub col_11: Box<[FConfigAffix]>,
    pub drops: FxHashMap<i32, i32>,
    pub id: i32,
    pub level_range_min: i32,
    pub level_range_max: i32,
    pub interactive_id: i32,
    pub interactive_confirm_anim: i32,
    pub interactive_cancel_anim: i32,
    pub content_id: i32,
    pub random_reward: bool,
}
impl ConfigMapGameplayGodOfHuntingRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.level_range_min = cur.read_i32()?;
        self.level_range_max = cur.read_i32()?;
        self.interactive_id = cur.read_i32()?;
        self.interactive_confirm_anim = cur.read_i32()?;
        self.interactive_cancel_anim = cur.read_i32()?;
        self.begin_dialog_id = cur.read_array_i32()?;
        self.end_dialog_id = cur.read_array_i32()?;
        self.buff = cur.read_array_i32()?;
        self.buff_time = cur.read_f64()?;
        self.reward = FConfigI32I32::read_array(cur)?;
        self.col_11 = FConfigAffix::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.random_reward = cur.read_bool()?;
        self.drops = cur.read_map_i32_i32()?;
        self.content_id = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplayGodOfHuntingRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_god_of_hunting_ChunkA0.bin`.
pub type ConfigMapGameplayGodOfHuntingTable = BinTable<
    i32,
    ConfigMapGameplayGodOfHuntingRow,
>;
/// Parses `config_map_gameplay_god_of_hunting_ChunkA0.bin` into a [`ConfigMapGameplayGodOfHuntingTable`].
pub fn parse_config_map_gameplay_god_of_hunting(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayGodOfHuntingTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_god_of_hunting_affix_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplayGodOfHuntingAffixRow {
    pub affix: Box<[FConfigAffix]>,
    pub skill_affix: Box<[FConfigAffix]>,
    pub info: Box<str>,
    pub title: Box<str>,
    pub id: i32,
    pub weight: i32,
    pub affix_type: i32,
    pub limit: i32,
}
impl BinRow for ConfigMapGameplayGodOfHuntingAffixRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.affix = FConfigAffix::read_array(cur)?;
        self.skill_affix = FConfigAffix::read_array(cur)?;
        self.info = cur.read_str()?;
        self.title = cur.read_str()?;
        self.weight = cur.read_i32()?;
        self.affix_type = cur.read_i32()?;
        self.limit = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_god_of_hunting_affix_ChunkA0.bin`.
pub type ConfigMapGameplayGodOfHuntingAffixTable = BinTable<
    i32,
    ConfigMapGameplayGodOfHuntingAffixRow,
>;
/// Parses `config_map_gameplay_god_of_hunting_affix_ChunkA0.bin` into a [`ConfigMapGameplayGodOfHuntingAffixTable`].
pub fn parse_config_map_gameplay_god_of_hunting_affix(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayGodOfHuntingAffixTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_god_of_machine_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplayGodOfMachineRow {
    pub interactive_destroy_time: f64,
    pub round: FxHashMap<i32, Box<[i32]>>,
    pub twice_angle: FxHashMap<i32, i32>,
    pub twice_radius: FxHashMap<i32, i32>,
    pub round_score: FxHashMap<i32, i32>,
    pub time_offset: FxHashMap<i32, Box<[f64]>>,
    pub round_to_wave: FxHashMap<i32, Box<[i32]>>,
    pub wave_match: FxHashMap<i32, Box<[i32]>>,
    pub wave_monsters: FxHashMap<i32, Box<[i32]>>,
    pub monster_datas: FxHashMap<i32, Box<[i32]>>,
    pub monster_libs: FxHashMap<i32, Box<[i32]>>,
    pub monster_score: FxHashMap<i32, i32>,
    pub monster_damage: FxHashMap<i32, i32>,
    pub all_monster_affixs: FxHashMap<i32, Box<[i32]>>,
    pub sp_monster_fectch: FxHashMap<i32, Box<[i32]>>,
    pub sp_monster_affixs: FxHashMap<i32, Box<[i32]>>,
    pub rate_monster_fectch: FxHashMap<i32, Box<[i32]>>,
    pub drops: Box<[FConfigI32I32]>,
    pub drop_affixs_match: FxHashMap<i32, Box<[i32]>>,
    pub drop_round_affix: FConfigAffix,
    pub fx_duration: f64,
    pub round_time_limit: f64,
    pub sec_interactive_destroy_time: f64,
    pub sec_interactive_range: FxHashMap<i32, i32>,
    pub sec_interactive_num: FxHashMap<i32, Box<[i32]>>,
    pub sec_interactive_angle: FxHashMap<i32, i32>,
    pub id: i32,
    pub interactive_id: i32,
    pub interactive_anim_win: i32,
    pub interactive_anim_lose: i32,
    pub interactive_anim_end: i32,
    pub radius: i32,
    pub hp: i32,
    pub angle: i32,
    pub create_radius: i32,
    pub fx_born_monster: i32,
    pub content_id: i32,
    pub sec_interactive_id: i32,
    pub sec_interactive_anim_win: i32,
    pub sec_interactive_anim_lose: i32,
    pub sec_interactive_anim_end: i32,
    pub sec_interactive_radius: i32,
    pub sec_interactive_hp: i32,
    pub monster_through_percent: i32,
    pub sec_interactive_fail: i32,
}
impl ConfigMapGameplayGodOfMachineRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.interactive_id = cur.read_i32()?;
        self.interactive_anim_win = cur.read_i32()?;
        self.interactive_anim_lose = cur.read_i32()?;
        self.interactive_anim_end = cur.read_i32()?;
        self.interactive_destroy_time = cur.read_f64()?;
        self.radius = cur.read_i32()?;
        self.hp = cur.read_i32()?;
        self.angle = cur.read_i32()?;
        self.round = cur.read_map_i32_array_i32()?;
        self.twice_angle = cur.read_map_i32_i32()?;
        self.twice_radius = cur.read_map_i32_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.round_score = cur.read_map_i32_i32()?;
        self.create_radius = cur.read_i32()?;
        self.time_offset = cur.read_map_i32_array_f64()?;
        self.round_to_wave = cur.read_map_i32_array_i32()?;
        self.wave_match = cur.read_map_i32_array_i32()?;
        self.wave_monsters = cur.read_map_i32_array_i32()?;
        self.monster_datas = cur.read_map_i32_array_i32()?;
        self.monster_libs = cur.read_map_i32_array_i32()?;
        self.monster_score = cur.read_map_i32_i32()?;
        self.monster_damage = cur.read_map_i32_i32()?;
        self.all_monster_affixs = cur.read_map_i32_array_i32()?;
        self.sp_monster_fectch = cur.read_map_i32_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.sp_monster_affixs = cur.read_map_i32_array_i32()?;
        self.rate_monster_fectch = cur.read_map_i32_array_i32()?;
        self.drops = FConfigI32I32::read_array(cur)?;
        self.drop_affixs_match = cur.read_map_i32_array_i32()?;
        self.drop_round_affix = FConfigAffix::read(cur)?;
        self.fx_born_monster = cur.read_i32()?;
        self.fx_duration = cur.read_f64()?;
        self.round_time_limit = cur.read_f64()?;
        self.content_id = cur.read_i32()?;
        self.sec_interactive_id = cur.read_i32()?;
        self.sec_interactive_anim_win = cur.read_i32()?;
        self.sec_interactive_anim_lose = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.sec_interactive_anim_end = cur.read_i32()?;
        self.sec_interactive_destroy_time = cur.read_f64()?;
        self.sec_interactive_range = cur.read_map_i32_i32()?;
        self.sec_interactive_num = cur.read_map_i32_array_i32()?;
        self.sec_interactive_angle = cur.read_map_i32_i32()?;
        self.sec_interactive_radius = cur.read_i32()?;
        self.sec_interactive_hp = cur.read_i32()?;
        self.monster_through_percent = cur.read_i32()?;
        self.sec_interactive_fail = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplayGodOfMachineRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_god_of_machine_ChunkA0.bin`.
pub type ConfigMapGameplayGodOfMachineTable = BinTable<
    i32,
    ConfigMapGameplayGodOfMachineRow,
>;
/// Parses `config_map_gameplay_god_of_machine_ChunkA0.bin` into a [`ConfigMapGameplayGodOfMachineTable`].
pub fn parse_config_map_gameplay_god_of_machine(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayGodOfMachineTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_god_of_war_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplayGodOfWarRow {
    pub prepare_time: f64,
    pub rarity_score: FxHashMap<i32, i32>,
    pub lv_interval: Box<[i32]>,
    pub lv_times: Box<[f64]>,
    pub buff: Box<[i32]>,
    pub create_monster_num: Box<[i32]>,
    pub ui_remove_delay: f64,
    pub end_dialog_id: Box<[i32]>,
    pub reward: Box<[FConfigI32I32]>,
    pub map_add_score: Box<[FConfigI32I32]>,
    pub drops: FxHashMap<i32, i32>,
    pub level_to_affix: Box<[FConfigAffix]>,
    pub id: i32,
    pub open_trap_id: i32,
    pub open_dialog_id: i32,
    pub max_lv: i32,
    pub create_monster_prop: i32,
    pub create_monster_radius: i32,
    pub end_trap_id: i32,
    pub base_score: i32,
    pub content_id: i32,
}
impl ConfigMapGameplayGodOfWarRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.open_trap_id = cur.read_i32()?;
        self.open_dialog_id = cur.read_i32()?;
        self.prepare_time = cur.read_f64()?;
        self.rarity_score = cur.read_map_i32_i32()?;
        self.lv_interval = cur.read_array_i32()?;
        self.max_lv = cur.read_i32()?;
        self.lv_times = cur.read_array_f64()?;
        self.buff = cur.read_array_i32()?;
        self.create_monster_prop = cur.read_i32()?;
        self.create_monster_num = cur.read_array_i32()?;
        self.create_monster_radius = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.ui_remove_delay = cur.read_f64()?;
        self.end_dialog_id = cur.read_array_i32()?;
        self.reward = FConfigI32I32::read_array(cur)?;
        self.end_trap_id = cur.read_i32()?;
        self.base_score = cur.read_i32()?;
        self.map_add_score = FConfigI32I32::read_array(cur)?;
        self.drops = cur.read_map_i32_i32()?;
        self.level_to_affix = FConfigAffix::read_array(cur)?;
        self.content_id = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplayGodOfWarRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_god_of_war_ChunkA0.bin`.
pub type ConfigMapGameplayGodOfWarTable = BinTable<i32, ConfigMapGameplayGodOfWarRow>;
/// Parses `config_map_gameplay_god_of_war_ChunkA0.bin` into a [`ConfigMapGameplayGodOfWarTable`].
pub fn parse_config_map_gameplay_god_of_war(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayGodOfWarTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_god_of_war_hunt_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplayGodOfWarHuntRow {
    pub monster_weights: FxHashMap<i32, i32>,
    pub boss_spawn_chance: f64,
    pub convert_group_ratio: f64,
    pub boss_spawn_point_ids: Box<str>,
    pub escape_spawn_point_ids: Box<str>,
    pub initial_time: f64,
    pub rage_lv: Box<[i32]>,
    pub rage_buff: Box<[i32]>,
    pub rage_buff_layer_limit: Box<[i32]>,
    pub kill_add_time: Box<[f64]>,
    pub rarity_score: Box<[f64]>,
    pub drops: FxHashMap<i32, i32>,
    pub level_to_affix: Box<[FConfigAffix]>,
    pub prepare_time: f64,
    pub id: i32,
    pub escape_monster_id: i32,
    pub boss_monster_id: i32,
    pub convert_group_num_min: i32,
    pub convert_group_num_max: i32,
    pub lv_max: i32,
    pub open_trap_id: i32,
}
impl ConfigMapGameplayGodOfWarHuntRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.monster_weights = cur.read_map_i32_i32()?;
        self.escape_monster_id = cur.read_i32()?;
        self.boss_spawn_chance = cur.read_f64()?;
        self.boss_monster_id = cur.read_i32()?;
        self.convert_group_ratio = cur.read_f64()?;
        self.convert_group_num_min = cur.read_i32()?;
        self.convert_group_num_max = cur.read_i32()?;
        self.boss_spawn_point_ids = cur.read_str()?;
        self.escape_spawn_point_ids = cur.read_str()?;
        self.initial_time = cur.read_f64()?;
        self.rage_lv = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.rage_buff = cur.read_array_i32()?;
        self.rage_buff_layer_limit = cur.read_array_i32()?;
        self.kill_add_time = cur.read_array_f64()?;
        self.rarity_score = cur.read_array_f64()?;
        self.drops = cur.read_map_i32_i32()?;
        self.level_to_affix = FConfigAffix::read_array(cur)?;
        self.lv_max = cur.read_i32()?;
        self.open_trap_id = cur.read_i32()?;
        self.prepare_time = cur.read_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplayGodOfWarHuntRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_god_of_war_hunt_ChunkA0.bin`.
pub type ConfigMapGameplayGodOfWarHuntTable = BinTable<
    i32,
    ConfigMapGameplayGodOfWarHuntRow,
>;
/// Parses `config_map_gameplay_god_of_war_hunt_ChunkA0.bin` into a [`ConfigMapGameplayGodOfWarHuntTable`].
pub fn parse_config_map_gameplay_god_of_war_hunt(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayGodOfWarHuntTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_level_rank_config_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplayLevelRankConfigRow {
    pub level: Box<[i32]>,
    pub r#type: Box<str>,
    pub value: Box<str>,
    pub id: i32,
    pub group: i32,
}
impl BinRow for ConfigMapGameplayLevelRankConfigRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.group = cur.read_i32()?;
        self.level = cur.read_array_i32()?;
        self.r#type = cur.read_str()?;
        self.value = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_level_rank_config_ChunkA0.bin`.
pub type ConfigMapGameplayLevelRankConfigTable = BinTable<
    i32,
    ConfigMapGameplayLevelRankConfigRow,
>;
/// Parses `config_map_gameplay_level_rank_config_ChunkA0.bin` into a [`ConfigMapGameplayLevelRankConfigTable`].
pub fn parse_config_map_gameplay_level_rank_config(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayLevelRankConfigTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_realm_walker_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplayRealmWalkerRow {
    pub affix: FConfigAffix,
    pub id: i32,
    pub affix_seat: i32,
    pub weight: i32,
    pub mutex_affix: i32,
}
impl BinRow for ConfigMapGameplayRealmWalkerRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.affix_seat = cur.read_i32()?;
        self.affix = FConfigAffix::read(cur)?;
        self.weight = cur.read_i32()?;
        self.mutex_affix = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_realm_walker_ChunkA0.bin`.
pub type ConfigMapGameplayRealmWalkerTable = BinTable<
    (i32, i32),
    ConfigMapGameplayRealmWalkerRow,
>;
/// Parses `config_map_gameplay_realm_walker_ChunkA0.bin` into a [`ConfigMapGameplayRealmWalkerTable`].
pub fn parse_config_map_gameplay_realm_walker(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayRealmWalkerTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_altar_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplaySeasonAltarRow {
    pub drops: Box<[FConfigI32I32]>,
    pub failed_drop_affix: FxHashMap<i32, Box<[i32]>>,
    pub challenge_interval: Box<[f64]>,
    pub layers_match: Box<[Box<[i32]>]>,
    pub firstly_monster: FConfigI32ArrayI32,
    pub buff: Box<[i32]>,
    pub buff_time: f64,
    pub fight_bgm: Box<str>,
    pub id: i32,
    pub custom_trap_id: i32,
    pub level_num: i32,
    pub altar_num: FConfigI32I32,
    pub trap_state_open_ui: i32,
    pub trap_state_close_ui: i32,
    pub trap_state_confirm_ui: i32,
    pub need_boss: bool,
}
impl ConfigMapGameplaySeasonAltarRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.custom_trap_id = cur.read_i32()?;
        self.drops = FConfigI32I32::read_array(cur)?;
        self.level_num = cur.read_i32()?;
        self.failed_drop_affix = cur.read_map_i32_array_i32()?;
        self.challenge_interval = cur.read_array_f64()?;
        self.layers_match = cur.read_array_2d_i32()?;
        self.altar_num = FConfigI32I32::read(cur)?;
        self.need_boss = cur.read_bool()?;
        self.firstly_monster = FConfigI32ArrayI32::read(cur)?;
        self.buff = cur.read_array_i32()?;
        self.buff_time = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.trap_state_open_ui = cur.read_i32()?;
        self.trap_state_close_ui = cur.read_i32()?;
        self.trap_state_confirm_ui = cur.read_i32()?;
        self.fight_bgm = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplaySeasonAltarRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_altar_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonAltarTable = BinTable<
    i32,
    ConfigMapGameplaySeasonAltarRow,
>;
/// Parses `config_map_gameplay_season_altar_ChunkA0.bin` into a [`ConfigMapGameplaySeasonAltarTable`].
pub fn parse_config_map_gameplay_season_altar(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonAltarTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_altar_challenge_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplaySeasonAltarChallengeRow {
    pub trap_state: Box<[i32]>,
    pub monster_groups: Box<[FConfigI32ArrayI32]>,
    pub monster_refresh_rule: Box<[FConfigI32I32]>,
    pub time_limit: f64,
    pub monster_each_num: FxHashMap<i32, i32>,
    pub range_limit: FConfigI32F64,
    pub success_event: Box<[i32]>,
    pub fail_event: Box<[i32]>,
    pub player_affixs: Box<[FConfigAffix]>,
    pub monster_each_affixs: FxHashMap<i32, Box<[i32]>>,
    pub skill_stone_rule: Box<[FConfigI32ArrayI32]>,
    pub rule_info: Box<str>,
    pub timer_info: Box<str>,
    pub counter_info: Box<str>,
    pub id: i32,
    pub secion_id: i32,
    pub weight: i32,
    pub monster_target_num: i32,
}
impl ConfigMapGameplaySeasonAltarChallengeRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.secion_id = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        self.trap_state = cur.read_array_i32()?;
        self.monster_groups = FConfigI32ArrayI32::read_array(cur)?;
        self.monster_refresh_rule = FConfigI32I32::read_array(cur)?;
        self.time_limit = cur.read_f64()?;
        self.monster_target_num = cur.read_i32()?;
        self.monster_each_num = cur.read_map_i32_i32()?;
        self.range_limit = FConfigI32F64::read(cur)?;
        self.success_event = cur.read_array_i32()?;
        self.fail_event = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.player_affixs = FConfigAffix::read_array(cur)?;
        self.monster_each_affixs = cur.read_map_i32_array_i32()?;
        self.skill_stone_rule = FConfigI32ArrayI32::read_array(cur)?;
        self.rule_info = cur.read_str()?;
        self.timer_info = cur.read_str()?;
        self.counter_info = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplaySeasonAltarChallengeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_altar_challenge_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonAltarChallengeTable = BinTable<
    (i32, i32),
    ConfigMapGameplaySeasonAltarChallengeRow,
>;
/// Parses `config_map_gameplay_season_altar_challenge_ChunkA0.bin` into a [`ConfigMapGameplaySeasonAltarChallengeTable`].
pub fn parse_config_map_gameplay_season_altar_challenge(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonAltarChallengeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_altar_coord_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonAltarCoordRow {
    pub round_limit: Box<[i32]>,
    pub precoord_limit: Box<[i32]>,
    pub mutex_ids: Box<[i32]>,
    pub desc: Box<str>,
    pub effct_affix: Box<[i32]>,
    pub id: i32,
    pub group: i32,
    pub keyword: i32,
    pub r#type: i32,
    pub awaken_coord: i32,
    pub rarity: i32,
    pub vigilance: i32,
    pub moment_limit: FConfigI32I32,
    pub moment_weight: i32,
    pub is_awaken: bool,
}
impl ConfigMapGameplaySeasonAltarCoordRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.group = cur.read_i32()?;
        self.round_limit = cur.read_array_i32()?;
        self.precoord_limit = cur.read_array_i32()?;
        self.mutex_ids = cur.read_array_i32()?;
        self.keyword = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.is_awaken = cur.read_bool()?;
        self.awaken_coord = cur.read_i32()?;
        self.rarity = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.vigilance = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.moment_limit = FConfigI32I32::read(cur)?;
        self.moment_weight = cur.read_i32()?;
        self.effct_affix = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplaySeasonAltarCoordRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_altar_coord_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonAltarCoordTable = BinTable<
    i32,
    ConfigMapGameplaySeasonAltarCoordRow,
>;
/// Parses `config_map_gameplay_season_altar_coord_ChunkA0.bin` into a [`ConfigMapGameplaySeasonAltarCoordTable`].
pub fn parse_config_map_gameplay_season_altar_coord(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonAltarCoordTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_altar_group_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplaySeasonAltarGroupRow {
    pub monsters_lib: Box<[FConfigI32I32]>,
    pub added_affixs: Box<[FConfigAffix]>,
    pub id: i32,
    pub section_id: i32,
    pub rarity: i32,
    pub monster_num: FConfigI32I32,
    pub monster_side: i32,
    pub monster_radius: i32,
}
impl BinRow for ConfigMapGameplaySeasonAltarGroupRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.section_id = cur.read_i32()?;
        self.monsters_lib = FConfigI32I32::read_array(cur)?;
        self.rarity = cur.read_i32()?;
        self.monster_num = FConfigI32I32::read(cur)?;
        self.monster_side = cur.read_i32()?;
        self.added_affixs = FConfigAffix::read_array(cur)?;
        self.monster_radius = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_altar_group_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonAltarGroupTable = BinTable<
    (i32, i32),
    ConfigMapGameplaySeasonAltarGroupRow,
>;
/// Parses `config_map_gameplay_season_altar_group_ChunkA0.bin` into a [`ConfigMapGameplaySeasonAltarGroupTable`].
pub fn parse_config_map_gameplay_season_altar_group(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonAltarGroupTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_altar_item_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplaySeasonAltarItemRow {
    pub name: Box<str>,
    pub icon: Box<str>,
    pub desc: Box<str>,
    pub func_dropbox: Box<[i32]>,
    pub coord_ids: Box<[i32]>,
    pub affix: FConfigAffix,
    pub id: i32,
    pub group: i32,
    pub minimap_icon: i32,
    pub rarity: i32,
    pub r#type: i32,
    pub func_coord: i32,
    pub func_num: i32,
    pub func_rarity: i32,
    pub duration: i32,
    pub effect_limit: i32,
    pub moment_limit: FConfigI32I32,
    pub moment_weight: i32,
    pub can_drop: bool,
}
impl ConfigMapGameplaySeasonAltarItemRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.group = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.can_drop = cur.read_bool()?;
        self.icon = cur.read_str()?;
        self.minimap_icon = cur.read_i32()?;
        self.rarity = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.r#type = cur.read_i32()?;
        self.func_dropbox = cur.read_array_i32()?;
        self.func_coord = cur.read_i32()?;
        self.func_num = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.func_rarity = cur.read_i32()?;
        self.coord_ids = cur.read_array_i32()?;
        self.duration = cur.read_i32()?;
        self.effect_limit = cur.read_i32()?;
        self.moment_limit = FConfigI32I32::read(cur)?;
        self.moment_weight = cur.read_i32()?;
        self.affix = FConfigAffix::read(cur)?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplaySeasonAltarItemRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_altar_item_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonAltarItemTable = BinTable<
    i32,
    ConfigMapGameplaySeasonAltarItemRow,
>;
/// Parses `config_map_gameplay_season_altar_item_ChunkA0.bin` into a [`ConfigMapGameplaySeasonAltarItemTable`].
pub fn parse_config_map_gameplay_season_altar_item(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonAltarItemTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_altar_layer_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonAltarLayerRow {
    pub challenge_match: Box<[FConfigI32I32]>,
    pub id: i32,
    pub section_id: i32,
    pub trap_status: i32,
    pub is_exclude: bool,
}
impl BinRow for ConfigMapGameplaySeasonAltarLayerRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.section_id = cur.read_i32()?;
        self.trap_status = cur.read_i32()?;
        self.challenge_match = FConfigI32I32::read_array(cur)?;
        self.is_exclude = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_altar_layer_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonAltarLayerTable = BinTable<
    i32,
    ConfigMapGameplaySeasonAltarLayerRow,
>;
/// Parses `config_map_gameplay_season_altar_layer_ChunkA0.bin` into a [`ConfigMapGameplaySeasonAltarLayerTable`].
pub fn parse_config_map_gameplay_season_altar_layer(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonAltarLayerTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_corrupted_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplaySeasonCorruptedRow {
    pub monster_affix: Box<[FConfigAffix]>,
    pub monster_group_num: FxHashMap<i32, Box<[i32]>>,
    pub drop: Box<[i32]>,
    pub kill_monster_vfxx: Box<[i32]>,
    pub reward_type: Box<[i32]>,
    pub reward_weight: Box<[i32]>,
    pub reward_prop: Box<[i32]>,
    pub reward_monster: FxHashMap<i32, Box<[i32]>>,
    pub reward_drop_id: Box<[i32]>,
    pub reward_affix: Box<[FConfigAffix]>,
    pub monster_corrupted_point: Box<[i32]>,
    pub corrputed_portect_time: f64,
    pub corrupt_dec_interval: f64,
    pub skill_args: FxHashMap<i32, Box<[i32]>>,
    pub skill_kind: Box<[i32]>,
    pub skill_danger: Box<[i32]>,
    pub skill_affix: Box<[FConfigAffix]>,
    pub boss: Box<[i32]>,
    pub combo_reward: Box<[FConfigAffix]>,
    pub minimap: Box<[i32]>,
    pub active_cd_time: f64,
    pub id: i32,
    pub count: i32,
    pub max_corrupted_point: i32,
    pub corrputed_dec: i32,
    pub skill_group_num: i32,
    pub npc_enter_skill: i32,
}
impl ConfigMapGameplaySeasonCorruptedRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.count = cur.read_i32()?;
        self.monster_affix = FConfigAffix::read_array(cur)?;
        self.monster_group_num = cur.read_map_i32_array_i32()?;
        self.drop = cur.read_array_i32()?;
        self.kill_monster_vfxx = cur.read_array_i32()?;
        self.reward_type = cur.read_array_i32()?;
        self.reward_weight = cur.read_array_i32()?;
        self.reward_prop = cur.read_array_i32()?;
        self.reward_monster = cur.read_map_i32_array_i32()?;
        self.reward_drop_id = cur.read_array_i32()?;
        self.reward_affix = FConfigAffix::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.monster_corrupted_point = cur.read_array_i32()?;
        self.max_corrupted_point = cur.read_i32()?;
        self.corrputed_portect_time = cur.read_f64()?;
        self.corrputed_dec = cur.read_i32()?;
        self.corrupt_dec_interval = cur.read_f64()?;
        self.skill_args = cur.read_map_i32_array_i32()?;
        self.skill_kind = cur.read_array_i32()?;
        self.skill_danger = cur.read_array_i32()?;
        self.skill_group_num = cur.read_i32()?;
        self.skill_affix = FConfigAffix::read_array(cur)?;
        self.boss = cur.read_array_i32()?;
        self.npc_enter_skill = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.combo_reward = FConfigAffix::read_array(cur)?;
        self.minimap = cur.read_array_i32()?;
        self.active_cd_time = cur.read_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplaySeasonCorruptedRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_corrupted_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonCorruptedTable = BinTable<
    (i32, i32),
    ConfigMapGameplaySeasonCorruptedRow,
>;
/// Parses `config_map_gameplay_season_corrupted_ChunkA0.bin` into a [`ConfigMapGameplaySeasonCorruptedTable`].
pub fn parse_config_map_gameplay_season_corrupted(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonCorruptedTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_doll_drop_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplaySeasonDollDropRow {
    pub drop_affix: Box<[FConfigAffix]>,
    pub difficult: i32,
    pub rate: i32,
}
impl BinRow for ConfigMapGameplaySeasonDollDropRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.difficult = cur.read_i32()?;
        self.rate = cur.read_i32()?;
        self.drop_affix = FConfigAffix::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_doll_drop_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonDollDropTable = BinTable<
    (i32, i32),
    ConfigMapGameplaySeasonDollDropRow,
>;
/// Parses `config_map_gameplay_season_doll_drop_ChunkA0.bin` into a [`ConfigMapGameplaySeasonDollDropTable`].
pub fn parse_config_map_gameplay_season_doll_drop(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonDollDropTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_doll_drop_display_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonDollDropDisplayRow {
    pub icon: Box<str>,
    pub gear_display: Box<str>,
    pub rarity_type: i32,
    pub drop_type: i32,
}
impl BinRow for ConfigMapGameplaySeasonDollDropDisplayRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.rarity_type = cur.read_i32()?;
        self.drop_type = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.gear_display = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_doll_drop_display_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonDollDropDisplayTable = BinTable<
    (i32, i32),
    ConfigMapGameplaySeasonDollDropDisplayRow,
>;
/// Parses `config_map_gameplay_season_doll_drop_display_ChunkA0.bin` into a [`ConfigMapGameplaySeasonDollDropDisplayTable`].
pub fn parse_config_map_gameplay_season_doll_drop_display(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonDollDropDisplayTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_doll_modifier_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplaySeasonDollModifierRow {
    pub map_modifier: Box<[i32]>,
    pub gameplay_affix: Box<[FConfigAffix]>,
    pub function_modifier: Box<[i32]>,
    pub name: Box<str>,
    pub desc: Box<str>,
    pub icon: Box<str>,
    pub desc_param: Box<[i32]>,
    pub id: i32,
    pub event_type: i32,
    pub r#type: i32,
    pub weight: i32,
}
impl BinRow for ConfigMapGameplaySeasonDollModifierRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.event_type = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        self.map_modifier = cur.read_array_i32()?;
        self.gameplay_affix = FConfigAffix::read_array(cur)?;
        self.function_modifier = cur.read_array_i32()?;
        self.name = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.icon = cur.read_str()?;
        self.desc_param = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_doll_modifier_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonDollModifierTable = BinTable<
    i32,
    ConfigMapGameplaySeasonDollModifierRow,
>;
/// Parses `config_map_gameplay_season_doll_modifier_ChunkA0.bin` into a [`ConfigMapGameplaySeasonDollModifierTable`].
pub fn parse_config_map_gameplay_season_doll_modifier(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonDollModifierTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_overlap_monster_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplaySeasonOverlapMonsterRow {
    pub trigger: Box<[i32]>,
    pub born_num: Box<[i32]>,
    pub rare_num: Box<[FConfigI32ArrayI32]>,
    pub cutting_ratio: f64,
    pub born_num_time: Box<[f64]>,
    pub born_num_level: Box<[f64]>,
    pub outside_list: Box<[i32]>,
    pub rare_num_time: Box<[f64]>,
    pub rare_num_level: Box<[f64]>,
    pub map_id: i32,
    pub map_time_add: i32,
}
impl BinRow for ConfigMapGameplaySeasonOverlapMonsterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.trigger = cur.read_array_i32()?;
        self.born_num = cur.read_array_i32()?;
        self.rare_num = FConfigI32ArrayI32::read_array(cur)?;
        self.cutting_ratio = cur.read_f64()?;
        self.born_num_time = cur.read_array_f64()?;
        self.born_num_level = cur.read_array_f64()?;
        self.outside_list = cur.read_array_i32()?;
        self.map_time_add = cur.read_i32()?;
        self.rare_num_time = cur.read_array_f64()?;
        self.rare_num_level = cur.read_array_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_overlap_monster_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonOverlapMonsterTable = BinTable<
    i32,
    ConfigMapGameplaySeasonOverlapMonsterRow,
>;
/// Parses `config_map_gameplay_season_overlap_monster_ChunkA0.bin` into a [`ConfigMapGameplaySeasonOverlapMonsterTable`].
pub fn parse_config_map_gameplay_season_overlap_monster(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonOverlapMonsterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_plague_card_monster_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonPlagueCardMonsterRow {
    pub name: Box<str>,
    pub desc: Box<str>,
    pub icon: Box<str>,
    pub icon2: Box<str>,
    pub id: i32,
    pub species: i32,
    pub rarity: i32,
    pub npc_id: i32,
    pub base_score: i32,
    pub base_count: i32,
    pub weight: i32,
}
impl BinRow for ConfigMapGameplaySeasonPlagueCardMonsterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.species = cur.read_i32()?;
        self.rarity = cur.read_i32()?;
        self.npc_id = cur.read_i32()?;
        self.base_score = cur.read_i32()?;
        self.base_count = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.icon = cur.read_str()?;
        self.icon2 = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_plague_card_monster_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonPlagueCardMonsterTable = BinTable<
    i32,
    ConfigMapGameplaySeasonPlagueCardMonsterRow,
>;
/// Parses `config_map_gameplay_season_plague_card_monster_ChunkA0.bin` into a [`ConfigMapGameplaySeasonPlagueCardMonsterTable`].
pub fn parse_config_map_gameplay_season_plague_card_monster(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonPlagueCardMonsterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_plague_config_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonPlagueConfigRow {
    pub r#type: Box<str>,
    pub param: Box<str>,
    pub desc: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigMapGameplaySeasonPlagueConfigRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_str()?;
        self.param = cur.read_str()?;
        self.desc = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_plague_config_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonPlagueConfigTable = BinTable<
    i32,
    ConfigMapGameplaySeasonPlagueConfigRow,
>;
/// Parses `config_map_gameplay_season_plague_config_ChunkA0.bin` into a [`ConfigMapGameplaySeasonPlagueConfigTable`].
pub fn parse_config_map_gameplay_season_plague_config(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonPlagueConfigTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_roulette_gameplay_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonRouletteGameplayRow {
    pub unique_spawn_group_weight: Box<[FConfigI32ArrayI32]>,
    pub roll_spawn_group_weight: Box<[FConfigI32ArrayI32]>,
    pub story_danger_weight: Box<[FConfigI32I32]>,
    pub id: i32,
    pub common_spawn_kind: i32,
    pub common_spawn_total_num: i32,
    pub unique_spawn_kind: i32,
    pub roll_spawn_kind: i32,
    pub story_danger_prop: i32,
    pub only_appear_after_defeat_boss: bool,
}
impl BinRow for ConfigMapGameplaySeasonRouletteGameplayRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.common_spawn_kind = cur.read_i32()?;
        self.common_spawn_total_num = cur.read_i32()?;
        self.unique_spawn_kind = cur.read_i32()?;
        self.unique_spawn_group_weight = FConfigI32ArrayI32::read_array(cur)?;
        self.roll_spawn_kind = cur.read_i32()?;
        self.roll_spawn_group_weight = FConfigI32ArrayI32::read_array(cur)?;
        self.story_danger_prop = cur.read_i32()?;
        self.story_danger_weight = FConfigI32I32::read_array(cur)?;
        self.only_appear_after_defeat_boss = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_roulette_gameplay_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonRouletteGameplayTable = BinTable<
    i32,
    ConfigMapGameplaySeasonRouletteGameplayRow,
>;
/// Parses `config_map_gameplay_season_roulette_gameplay_ChunkA0.bin` into a [`ConfigMapGameplaySeasonRouletteGameplayTable`].
pub fn parse_config_map_gameplay_season_roulette_gameplay(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonRouletteGameplayTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_roulette_roulette_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonRouletteRouletteTypeRow {
    pub trap_states: FxHashMap<Box<str>, i32>,
    pub enter_map_dialog_ids: Box<[FConfigI32I32]>,
    pub big_prize_dialog_ids: Box<[FConfigI32I32]>,
    pub end_normal_dialog_ids: Box<[FConfigI32I32]>,
    pub end_destroy_dialog_ids: Box<[FConfigI32I32]>,
    pub id: i32,
    pub custom_trap_id: i32,
    pub appear_projectile_skill_stone: i32,
}
impl BinRow for ConfigMapGameplaySeasonRouletteRouletteTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.custom_trap_id = cur.read_i32()?;
        self.trap_states = cur.read_map_str_i32()?;
        self.enter_map_dialog_ids = FConfigI32I32::read_array(cur)?;
        self.big_prize_dialog_ids = FConfigI32I32::read_array(cur)?;
        self.end_normal_dialog_ids = FConfigI32I32::read_array(cur)?;
        self.end_destroy_dialog_ids = FConfigI32I32::read_array(cur)?;
        self.appear_projectile_skill_stone = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_roulette_roulette_type_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonRouletteRouletteTypeTable = BinTable<
    i32,
    ConfigMapGameplaySeasonRouletteRouletteTypeRow,
>;
/// Parses `config_map_gameplay_season_roulette_roulette_type_ChunkA0.bin` into a [`ConfigMapGameplaySeasonRouletteRouletteTypeTable`].
pub fn parse_config_map_gameplay_season_roulette_roulette_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonRouletteRouletteTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_season_roulette_talent_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonRouletteTalentRow {
    pub name: Box<str>,
    pub pic: Box<str>,
    pub line: Box<[i32]>,
    pub function_modifiers: Box<[i32]>,
    pub map_modifiers: Box<[i32]>,
    pub id: i32,
    pub function_modifier: i32,
    pub map_modifier: i32,
}
impl BinRow for ConfigMapGameplaySeasonRouletteTalentRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.pic = cur.read_str()?;
        self.line = cur.read_array_i32()?;
        self.function_modifier = cur.read_i32()?;
        self.map_modifier = cur.read_i32()?;
        self.function_modifiers = cur.read_array_i32()?;
        self.map_modifiers = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_season_roulette_talent_ChunkA0.bin`.
pub type ConfigMapGameplaySeasonRouletteTalentTable = BinTable<
    i32,
    ConfigMapGameplaySeasonRouletteTalentRow,
>;
/// Parses `config_map_gameplay_season_roulette_talent_ChunkA0.bin` into a [`ConfigMapGameplaySeasonRouletteTalentTable`].
pub fn parse_config_map_gameplay_season_roulette_talent(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySeasonRouletteTalentTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_speedrun_modifier_difficulty_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySpeedrunModifierDifficultyRow {
    pub modifier_group: Box<[FConfigI32I32]>,
    pub name: Box<str>,
    pub tip_title: Box<str>,
    pub id: i32,
    pub danger: i32,
}
impl BinRow for ConfigMapGameplaySpeedrunModifierDifficultyRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.modifier_group = FConfigI32I32::read_array(cur)?;
        self.name = cur.read_str()?;
        self.danger = cur.read_i32()?;
        self.tip_title = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_speedrun_modifier_difficulty_ChunkA0.bin`.
pub type ConfigMapGameplaySpeedrunModifierDifficultyTable = BinTable<
    i32,
    ConfigMapGameplaySpeedrunModifierDifficultyRow,
>;
/// Parses `config_map_gameplay_speedrun_modifier_difficulty_ChunkA0.bin` into a [`ConfigMapGameplaySpeedrunModifierDifficultyTable`].
pub fn parse_config_map_gameplay_speedrun_modifier_difficulty(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySpeedrunModifierDifficultyTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_speedrun_random_event_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySpeedrunRandomEventRow {
    pub name: Box<str>,
    pub desc: Box<str>,
    pub icon: Box<str>,
    pub id: i32,
    pub gameplay_type: i32,
}
impl BinRow for ConfigMapGameplaySpeedrunRandomEventRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.gameplay_type = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.icon = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_speedrun_random_event_ChunkA0.bin`.
pub type ConfigMapGameplaySpeedrunRandomEventTable = BinTable<
    i32,
    ConfigMapGameplaySpeedrunRandomEventRow,
>;
/// Parses `config_map_gameplay_speedrun_random_event_ChunkA0.bin` into a [`ConfigMapGameplaySpeedrunRandomEventTable`].
pub fn parse_config_map_gameplay_speedrun_random_event(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySpeedrunRandomEventTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_speedrun_season_effect_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySpeedrunSeasonEffectRow {
    pub name: Box<str>,
    pub desc: Box<str>,
    pub icon: Box<str>,
    pub id: i32,
    pub gameplay_type: i32,
    pub num_per_level: i32,
}
impl BinRow for ConfigMapGameplaySpeedrunSeasonEffectRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.gameplay_type = cur.read_i32()?;
        self.num_per_level = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.icon = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_speedrun_season_effect_ChunkA0.bin`.
pub type ConfigMapGameplaySpeedrunSeasonEffectTable = BinTable<
    i32,
    ConfigMapGameplaySpeedrunSeasonEffectRow,
>;
/// Parses `config_map_gameplay_speedrun_season_effect_ChunkA0.bin` into a [`ConfigMapGameplaySpeedrunSeasonEffectTable`].
pub fn parse_config_map_gameplay_speedrun_season_effect(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySpeedrunSeasonEffectTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_speedrun_tier_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySpeedrunTierRow {
    pub init_map_modifier: Box<[i32]>,
    pub random_level_list: Box<[FConfigI32I32]>,
    pub random_event_group_id: Box<[FConfigI32I32]>,
    pub random_event_level_weight: Box<[i32]>,
    pub standard_level_list: Box<[i32]>,
    pub standard_event_list: Box<[i32]>,
    pub monster_affix: Box<[FConfigI32I32]>,
    pub force_monster_affix: Box<[FConfigI32I32]>,
    pub map_random_modifier_group: Box<[i32]>,
    pub random_modifier_num: Box<[FConfigI32I32]>,
    pub sp_modifier_difficult: Box<[FConfigI32I32]>,
    pub time: Box<str>,
    pub id: i32,
    pub base_level: i32,
    pub season_effect_id: i32,
    pub random_event_num: i32,
    pub challenge_num: i32,
    pub time_limit: i32,
    pub point_num: i32,
    pub reward_group: i32,
    pub reward_group_season: i32,
    pub difficulty: i32,
    pub is_standard_tier: bool,
    pub is_damage_lock: bool,
}
impl ConfigMapGameplaySpeedrunTierRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.base_level = cur.read_i32()?;
        self.init_map_modifier = cur.read_array_i32()?;
        self.season_effect_id = cur.read_i32()?;
        self.random_level_list = FConfigI32I32::read_array(cur)?;
        self.random_event_group_id = FConfigI32I32::read_array(cur)?;
        self.random_event_num = cur.read_i32()?;
        self.random_event_level_weight = cur.read_array_i32()?;
        self.is_standard_tier = cur.read_bool()?;
        self.standard_level_list = cur.read_array_i32()?;
        self.standard_event_list = cur.read_array_i32()?;
        self.challenge_num = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.time_limit = cur.read_i32()?;
        self.point_num = cur.read_i32()?;
        self.is_damage_lock = cur.read_bool()?;
        self.monster_affix = FConfigI32I32::read_array(cur)?;
        self.force_monster_affix = FConfigI32I32::read_array(cur)?;
        self.map_random_modifier_group = cur.read_array_i32()?;
        self.random_modifier_num = FConfigI32I32::read_array(cur)?;
        self.sp_modifier_difficult = FConfigI32I32::read_array(cur)?;
        self.reward_group = cur.read_i32()?;
        self.reward_group_season = cur.read_i32()?;
        self.difficulty = cur.read_i32()?;
        self.time = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigMapGameplaySpeedrunTierRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_speedrun_tier_ChunkA0.bin`.
pub type ConfigMapGameplaySpeedrunTierTable = BinTable<
    i32,
    ConfigMapGameplaySpeedrunTierRow,
>;
/// Parses `config_map_gameplay_speedrun_tier_ChunkA0.bin` into a [`ConfigMapGameplaySpeedrunTierTable`].
pub fn parse_config_map_gameplay_speedrun_tier(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplaySpeedrunTierTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_treasure_chest_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplayTreasureChestRow {
    pub name: Box<str>,
    pub drop: Box<[FConfigI32I32]>,
    pub open_result: Box<[FConfigI32I32]>,
    pub id: i32,
    pub weight: i32,
    pub interactive_id: i32,
    pub openl_level_id: i32,
}
impl BinRow for ConfigMapGameplayTreasureChestRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.weight = cur.read_i32()?;
        self.interactive_id = cur.read_i32()?;
        self.openl_level_id = cur.read_i32()?;
        self.drop = FConfigI32I32::read_array(cur)?;
        self.open_result = FConfigI32I32::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_treasure_chest_ChunkA0.bin`.
pub type ConfigMapGameplayTreasureChestTable = BinTable<
    i32,
    ConfigMapGameplayTreasureChestRow,
>;
/// Parses `config_map_gameplay_treasure_chest_ChunkA0.bin` into a [`ConfigMapGameplayTreasureChestTable`].
pub fn parse_config_map_gameplay_treasure_chest(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayTreasureChestTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_gameplay_treasure_chest_result_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplayTreasureChestResultRow {
    pub name: Box<str>,
    pub create_iteractive_id: Box<[FConfigI32I32]>,
    pub create_iteractive_count: Box<[FConfigI32I32]>,
    pub create_interactive_dely: f64,
    pub id: i32,
    pub drop_way: i32,
    pub create_monster: i32,
}
impl BinRow for ConfigMapGameplayTreasureChestResultRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.drop_way = cur.read_i32()?;
        self.create_iteractive_id = FConfigI32I32::read_array(cur)?;
        self.create_iteractive_count = FConfigI32I32::read_array(cur)?;
        self.create_interactive_dely = cur.read_f64()?;
        self.create_monster = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_treasure_chest_result_ChunkA0.bin`.
pub type ConfigMapGameplayTreasureChestResultTable = BinTable<
    i32,
    ConfigMapGameplayTreasureChestResultRow,
>;
/// Parses `config_map_gameplay_treasure_chest_result_ChunkA0.bin` into a [`ConfigMapGameplayTreasureChestResultTable`].
pub fn parse_config_map_gameplay_treasure_chest_result(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapGameplayTreasureChestResultTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_icon_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapIconRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub res: Box<str>,
    pub id: i32,
    pub zorder: i32,
    pub show_in_filter: bool,
}
impl BinRow for ConfigMapIconRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.res = cur.read_str()?;
        self.zorder = cur.read_i32()?;
        self.show_in_filter = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_icon_ChunkA0.bin`.
pub type ConfigMapIconTable = BinTable<i32, ConfigMapIconRow>;
/// Parses `config_map_icon_ChunkA0.bin` into a [`ConfigMapIconTable`].
pub fn parse_config_map_icon(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapIconTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_icon_label_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapIconLabelRow {
    pub id: i32,
    pub label: i32,
    pub map_icon: i32,
    pub zorder: i32,
}
impl BinRow for ConfigMapIconLabelRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.label = cur.read_i32()?;
        self.map_icon = cur.read_i32()?;
        self.zorder = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_icon_label_ChunkA0.bin`.
pub type ConfigMapIconLabelTable = BinTable<i32, ConfigMapIconLabelRow>;
/// Parses `config_map_icon_label_ChunkA0.bin` into a [`ConfigMapIconLabelTable`].
pub fn parse_config_map_icon_label(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapIconLabelTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_interactive_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapInteractiveRow {
    pub desc: Box<str>,
    pub name: Box<str>,
    pub wait_time: f64,
    pub delay_remove: f64,
    pub delay_trigger_finish: f64,
    pub icon_offset: Box<[i32]>,
    pub interactive_text: Box<str>,
    pub id: i32,
    pub resource_id: i32,
    pub init_sequence_id: i32,
    pub finish_sequence_id: i32,
    pub idle_sequence_id: i32,
    pub wait_montage: i32,
    pub shape_radius: i32,
    pub active_radius: i32,
    pub icon_visible_type: i32,
    pub touch_count: i32,
    pub interactive_icon_style: i32,
    pub exit_montage_id: i32,
    pub active_montage_id: i32,
    pub hide_weapon: bool,
}
impl ConfigMapInteractiveRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.name = cur.read_str()?;
        self.resource_id = cur.read_i32()?;
        self.wait_time = cur.read_f64()?;
        self.init_sequence_id = cur.read_i32()?;
        self.finish_sequence_id = cur.read_i32()?;
        self.idle_sequence_id = cur.read_i32()?;
        self.delay_remove = cur.read_f64()?;
        self.delay_trigger_finish = cur.read_f64()?;
        self.wait_montage = cur.read_i32()?;
        self.shape_radius = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.active_radius = cur.read_i32()?;
        self.icon_offset = cur.read_array_i32()?;
        self.icon_visible_type = cur.read_i32()?;
        self.touch_count = cur.read_i32()?;
        self.hide_weapon = cur.read_bool()?;
        self.interactive_text = cur.read_str()?;
        self.interactive_icon_style = cur.read_i32()?;
        self.exit_montage_id = cur.read_i32()?;
        self.active_montage_id = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapInteractiveRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_interactive_ChunkA0.bin`.
pub type ConfigMapInteractiveTable = BinTable<i32, ConfigMapInteractiveRow>;
/// Parses `config_map_interactive_ChunkA0.bin` into a [`ConfigMapInteractiveTable`].
pub fn parse_config_map_interactive(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapInteractiveTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_modifier_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapModifierRow {
    pub item_quantity: f64,
    pub item_rarity: f64,
    pub pack_size: f64,
    pub rarity: Box<[i32]>,
    pub danger: Box<[i32]>,
    pub affix: Box<[FConfigAffix]>,
    pub dynmc_params: Box<[Box<[f64]>]>,
    pub drop_affix: Box<[FConfigAffix]>,
    pub gameplay_type: Box<[i32]>,
    pub description: Box<str>,
    pub monster_source: Box<[i32]>,
    pub custom_trap: Box<[i32]>,
    pub open_gameplay_type: Box<[i32]>,
    pub drop_carrier: Box<[i32]>,
    pub id: i32,
    pub affix_type: i32,
    pub map_tier: i32,
    pub map_type: i32,
    pub target: i32,
    pub r#type: i32,
    pub group: i32,
    pub hide: bool,
    pub runtime_modifier: bool,
}
impl ConfigMapModifierRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.affix_type = cur.read_i32()?;
        self.map_tier = cur.read_i32()?;
        self.map_type = cur.read_i32()?;
        self.item_quantity = cur.read_f64()?;
        self.item_rarity = cur.read_f64()?;
        self.pack_size = cur.read_f64()?;
        self.target = cur.read_i32()?;
        self.rarity = cur.read_array_i32()?;
        self.danger = cur.read_array_i32()?;
        self.affix = FConfigAffix::read_array(cur)?;
        self.dynmc_params = cur.read_array_2d_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.drop_affix = FConfigAffix::read_array(cur)?;
        self.r#type = cur.read_i32()?;
        self.hide = cur.read_bool()?;
        self.gameplay_type = cur.read_array_i32()?;
        self.description = cur.read_str()?;
        self.runtime_modifier = cur.read_bool()?;
        self.monster_source = cur.read_array_i32()?;
        self.custom_trap = cur.read_array_i32()?;
        self.open_gameplay_type = cur.read_array_i32()?;
        self.group = cur.read_i32()?;
        self.drop_carrier = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapModifierRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_modifier_ChunkA0.bin`.
pub type ConfigMapModifierTable = BinTable<i32, ConfigMapModifierRow>;
/// Parses `config_map_modifier_ChunkA0.bin` into a [`ConfigMapModifierTable`].
pub fn parse_config_map_modifier(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapModifierTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_obstacle_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapObstacleRow {
    pub shape: Box<str>,
    pub id: i32,
    pub resource: i32,
    pub collision: i32,
}
impl BinRow for ConfigMapObstacleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.resource = cur.read_i32()?;
        self.shape = cur.read_str()?;
        self.collision = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_obstacle_ChunkA0.bin`.
pub type ConfigMapObstacleTable = BinTable<i32, ConfigMapObstacleRow>;
/// Parses `config_map_obstacle_ChunkA0.bin` into a [`ConfigMapObstacleTable`].
pub fn parse_config_map_obstacle(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapObstacleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_performance_actor_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapPerformanceActorRow {
    pub name: Box<str>,
    pub bp_path: Box<str>,
    pub id: i32,
    pub interactive_icon_id: i32,
    pub createor_interact_only: i32,
    pub gameplaytype: i32,
    pub interaction: bool,
    pub show_name: bool,
}
impl BinRow for ConfigMapPerformanceActorRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.bp_path = cur.read_str()?;
        self.interaction = cur.read_bool()?;
        self.interactive_icon_id = cur.read_i32()?;
        self.createor_interact_only = cur.read_i32()?;
        self.show_name = cur.read_bool()?;
        self.gameplaytype = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_performance_actor_ChunkA0.bin`.
pub type ConfigMapPerformanceActorTable = BinTable<i32, ConfigMapPerformanceActorRow>;
/// Parses `config_map_performance_actor_ChunkA0.bin` into a [`ConfigMapPerformanceActorTable`].
pub fn parse_config_map_performance_actor(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapPerformanceActorTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_portal_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapPortalRow {
    pub name: Box<str>,
    pub target_level_id: Box<[i32]>,
    pub destination_pos: VecDim,
    pub delay_touch_time: f64,
    pub touch_audio_key: Box<str>,
    pub portal_id: i32,
    pub portal_type_id: i32,
    pub default_state: i32,
    pub mesh_id: i32,
    pub anim_active_id: i32,
    pub anim_idle_id: i32,
    pub unactivate_resource_id: i32,
    pub resource_id: i32,
    pub close_resource_id: i32,
    pub touch_resource_id: i32,
    pub untouch_resource_id: i32,
    pub name_height_offset: i32,
    pub appear_distance: i32,
    pub player_enter_vfx_id: i32,
    pub player_enter_montage_id: i32,
    pub player_exit_vfx_id: i32,
    pub player_exit_montage_id: i32,
    pub minimap_icon: i32,
    pub interactive_distance: i32,
    pub no_trans_anim: i32,
    pub for_sequence_hide: i32,
    pub fashion_type: i32,
    pub activate_distance: i32,
    pub transport_distance: i32,
    pub team_use: bool,
    pub face_to_camera: bool,
}
impl ConfigMapPortalRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.portal_id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.portal_type_id = cur.read_i32()?;
        self.target_level_id = cur.read_array_i32()?;
        self.destination_pos = cur.read_vec_dim()?;
        self.default_state = cur.read_i32()?;
        self.delay_touch_time = cur.read_f64()?;
        self.mesh_id = cur.read_i32()?;
        self.anim_active_id = cur.read_i32()?;
        self.anim_idle_id = cur.read_i32()?;
        self.unactivate_resource_id = cur.read_i32()?;
        self.resource_id = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.close_resource_id = cur.read_i32()?;
        self.touch_resource_id = cur.read_i32()?;
        self.untouch_resource_id = cur.read_i32()?;
        self.name_height_offset = cur.read_i32()?;
        self.appear_distance = cur.read_i32()?;
        self.player_enter_vfx_id = cur.read_i32()?;
        self.player_enter_montage_id = cur.read_i32()?;
        self.player_exit_vfx_id = cur.read_i32()?;
        self.player_exit_montage_id = cur.read_i32()?;
        self.minimap_icon = cur.read_i32()?;
        self.interactive_distance = cur.read_i32()?;
        self.no_trans_anim = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.for_sequence_hide = cur.read_i32()?;
        self.fashion_type = cur.read_i32()?;
        self.team_use = cur.read_bool()?;
        self.face_to_camera = cur.read_bool()?;
        self.activate_distance = cur.read_i32()?;
        self.transport_distance = cur.read_i32()?;
        self.touch_audio_key = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigMapPortalRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_portal_ChunkA0.bin`.
pub type ConfigMapPortalTable = BinTable<i32, ConfigMapPortalRow>;
/// Parses `config_map_portal_ChunkA0.bin` into a [`ConfigMapPortalTable`].
pub fn parse_config_map_portal(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapPortalTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_tale_actor_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapTaleActorRow {
    pub offset: Box<[i32]>,
    pub id: i32,
    pub bubble_distance: i32,
    pub dialog_distance: i32,
    pub bubble_tale_npc_id: i32,
    pub dialog_tale_npc_id: i32,
    pub delay_trigger: i32,
    pub idle_vfx: i32,
    pub trigger_vfx: i32,
}
impl BinRow for ConfigMapTaleActorRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.bubble_distance = cur.read_i32()?;
        self.dialog_distance = cur.read_i32()?;
        self.bubble_tale_npc_id = cur.read_i32()?;
        self.dialog_tale_npc_id = cur.read_i32()?;
        self.delay_trigger = cur.read_i32()?;
        self.offset = cur.read_array_i32()?;
        self.idle_vfx = cur.read_i32()?;
        self.trigger_vfx = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_tale_actor_ChunkA0.bin`.
pub type ConfigMapTaleActorTable = BinTable<i32, ConfigMapTaleActorRow>;
/// Parses `config_map_tale_actor_ChunkA0.bin` into a [`ConfigMapTaleActorTable`].
pub fn parse_config_map_tale_actor(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapTaleActorTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_transport_check_point_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapTransportCheckPointRow {
    pub des: Box<str>,
    pub transport_map_login: VecDim,
    pub unlock_condition: Box<str>,
    pub id: i32,
    pub mesh: i32,
    pub active_vfx: i32,
    pub defualt_active: i32,
    pub active_distance: i32,
    pub interactive_distance: i32,
    pub defualt_vfx: i32,
    pub near_vfx: i32,
    pub transport_map_id: i32,
    pub transport_map_type: i32,
    pub resource_id: i32,
    pub idle_montage_id: i32,
    pub active_montage_id: i32,
}
impl ConfigMapTransportCheckPointRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.mesh = cur.read_i32()?;
        self.active_vfx = cur.read_i32()?;
        self.defualt_active = cur.read_i32()?;
        self.active_distance = cur.read_i32()?;
        self.interactive_distance = cur.read_i32()?;
        self.defualt_vfx = cur.read_i32()?;
        self.near_vfx = cur.read_i32()?;
        self.transport_map_id = cur.read_i32()?;
        self.transport_map_type = cur.read_i32()?;
        self.transport_map_login = cur.read_vec_dim()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.unlock_condition = cur.read_str()?;
        self.resource_id = cur.read_i32()?;
        self.idle_montage_id = cur.read_i32()?;
        self.active_montage_id = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapTransportCheckPointRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_transport_check_point_ChunkA0.bin`.
pub type ConfigMapTransportCheckPointTable = BinTable<
    i32,
    ConfigMapTransportCheckPointRow,
>;
/// Parses `config_map_transport_check_point_ChunkA0.bin` into a [`ConfigMapTransportCheckPointTable`].
pub fn parse_config_map_transport_check_point(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapTransportCheckPointTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_map_vibe_actor_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapVibeActorRow {
    pub path: Box<str>,
    pub ref_res_path: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigMapVibeActorRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.path = cur.read_str()?;
        self.ref_res_path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_vibe_actor_ChunkA0.bin`.
pub type ConfigMapVibeActorTable = BinTable<i32, ConfigMapVibeActorRow>;
/// Parses `config_map_vibe_actor_ChunkA0.bin` into a [`ConfigMapVibeActorTable`].
pub fn parse_config_map_vibe_actor(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMapVibeActorTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_world_boss_attr_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigWorldBossAttrRow {
    pub stage_range: Box<[i32]>,
    pub life_point: f64,
    pub name_suffix: Box<str>,
    pub id: i32,
    pub npc_id: i32,
}
impl BinRow for ConfigWorldBossAttrRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.npc_id = cur.read_i32()?;
        self.stage_range = cur.read_array_i32()?;
        self.life_point = cur.read_f64()?;
        self.name_suffix = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_world_boss_attr_ChunkA0.bin`.
pub type ConfigWorldBossAttrTable = BinTable<i32, ConfigWorldBossAttrRow>;
/// Parses `config_world_boss_attr_ChunkA0.bin` into a [`ConfigWorldBossAttrTable`].
pub fn parse_config_world_boss_attr(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigWorldBossAttrTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_world_map_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigWorldMapRow {
    pub icon: Box<str>,
    pub place: Box<str>,
    pub npc_tool_des: Box<str>,
    pub id: i32,
    pub level_id: i32,
    pub before: i32,
    pub after: i32,
    pub item_base_id: i32,
    pub rarity: i32,
}
impl BinRow for ConfigWorldMapRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.level_id = cur.read_i32()?;
        self.before = cur.read_i32()?;
        self.after = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.place = cur.read_str()?;
        self.item_base_id = cur.read_i32()?;
        self.rarity = cur.read_i32()?;
        self.npc_tool_des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_world_map_ChunkA0.bin`.
pub type ConfigWorldMapTable = BinTable<i32, ConfigWorldMapRow>;
/// Parses `config_world_map_ChunkA0.bin` into a [`ConfigWorldMapTable`].
pub fn parse_config_world_map(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigWorldMapTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
#[inline(never)]
fn verify_map_chunk_0(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_map" => Some(parse_config_map(buf, scratch).map(|t| t.len())),
        "config_map_affix" => Some(parse_config_map_affix(buf, scratch).map(|t| t.len())),
        "config_map_common_pos_cfg" => {
            Some(parse_config_map_common_pos_cfg(buf, scratch).map(|t| t.len()))
        }
        "config_map_custom_trap" => {
            Some(parse_config_map_custom_trap(buf, scratch).map(|t| t.len()))
        }
        "config_map_custom_trap_status" => {
            Some(parse_config_map_custom_trap_status(buf, scratch).map(|t| t.len()))
        }
        "config_map_destructible" => {
            Some(parse_config_map_destructible(buf, scratch).map(|t| t.len()))
        }
        "config_map_destructible_avengers" => {
            Some(parse_config_map_destructible_avengers(buf, scratch).map(|t| t.len()))
        }
        "config_map_follower" => {
            Some(parse_config_map_follower(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_actors" => {
            Some(parse_config_map_gameplay_actors(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_god_of_darkness" => {
            Some(
                parse_config_map_gameplay_god_of_darkness(buf, scratch).map(|t| t.len()),
            )
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_map_chunk_1(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_map_gameplay_god_of_darkness_king" => {
            Some(
                parse_config_map_gameplay_god_of_darkness_king(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_god_of_hunting" => {
            Some(parse_config_map_gameplay_god_of_hunting(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_god_of_hunting_affix" => {
            Some(
                parse_config_map_gameplay_god_of_hunting_affix(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_god_of_machine" => {
            Some(parse_config_map_gameplay_god_of_machine(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_god_of_war" => {
            Some(parse_config_map_gameplay_god_of_war(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_god_of_war_hunt" => {
            Some(
                parse_config_map_gameplay_god_of_war_hunt(buf, scratch).map(|t| t.len()),
            )
        }
        "config_map_gameplay_level_rank_config" => {
            Some(
                parse_config_map_gameplay_level_rank_config(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_realm_walker" => {
            Some(parse_config_map_gameplay_realm_walker(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_season_altar" => {
            Some(parse_config_map_gameplay_season_altar(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_season_altar_challenge" => {
            Some(
                parse_config_map_gameplay_season_altar_challenge(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_map_chunk_2(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_map_gameplay_season_altar_coord" => {
            Some(
                parse_config_map_gameplay_season_altar_coord(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_altar_group" => {
            Some(
                parse_config_map_gameplay_season_altar_group(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_altar_item" => {
            Some(
                parse_config_map_gameplay_season_altar_item(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_altar_layer" => {
            Some(
                parse_config_map_gameplay_season_altar_layer(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_corrupted" => {
            Some(
                parse_config_map_gameplay_season_corrupted(buf, scratch).map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_doll_drop" => {
            Some(
                parse_config_map_gameplay_season_doll_drop(buf, scratch).map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_doll_drop_display" => {
            Some(
                parse_config_map_gameplay_season_doll_drop_display(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_doll_modifier" => {
            Some(
                parse_config_map_gameplay_season_doll_modifier(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_overlap_monster" => {
            Some(
                parse_config_map_gameplay_season_overlap_monster(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_plague_card_monster" => {
            Some(
                parse_config_map_gameplay_season_plague_card_monster(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_map_chunk_3(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_map_gameplay_season_plague_config" => {
            Some(
                parse_config_map_gameplay_season_plague_config(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_roulette_gameplay" => {
            Some(
                parse_config_map_gameplay_season_roulette_gameplay(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_roulette_roulette_type" => {
            Some(
                parse_config_map_gameplay_season_roulette_roulette_type(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_roulette_talent" => {
            Some(
                parse_config_map_gameplay_season_roulette_talent(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_speedrun_modifier_difficulty" => {
            Some(
                parse_config_map_gameplay_speedrun_modifier_difficulty(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_speedrun_random_event" => {
            Some(
                parse_config_map_gameplay_speedrun_random_event(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_speedrun_season_effect" => {
            Some(
                parse_config_map_gameplay_speedrun_season_effect(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_speedrun_tier" => {
            Some(parse_config_map_gameplay_speedrun_tier(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_treasure_chest" => {
            Some(parse_config_map_gameplay_treasure_chest(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_treasure_chest_result" => {
            Some(
                parse_config_map_gameplay_treasure_chest_result(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_map_chunk_4(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_map_icon" => Some(parse_config_map_icon(buf, scratch).map(|t| t.len())),
        "config_map_icon_label" => {
            Some(parse_config_map_icon_label(buf, scratch).map(|t| t.len()))
        }
        "config_map_interactive" => {
            Some(parse_config_map_interactive(buf, scratch).map(|t| t.len()))
        }
        "config_map_modifier" => {
            Some(parse_config_map_modifier(buf, scratch).map(|t| t.len()))
        }
        "config_map_obstacle" => {
            Some(parse_config_map_obstacle(buf, scratch).map(|t| t.len()))
        }
        "config_map_performance_actor" => {
            Some(parse_config_map_performance_actor(buf, scratch).map(|t| t.len()))
        }
        "config_map_portal" => {
            Some(parse_config_map_portal(buf, scratch).map(|t| t.len()))
        }
        "config_map_tale_actor" => {
            Some(parse_config_map_tale_actor(buf, scratch).map(|t| t.len()))
        }
        "config_map_transport_check_point" => {
            Some(parse_config_map_transport_check_point(buf, scratch).map(|t| t.len()))
        }
        "config_map_vibe_actor" => {
            Some(parse_config_map_vibe_actor(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_map_chunk_5(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_world_boss_attr" => {
            Some(parse_config_world_boss_attr(buf, scratch).map(|t| t.len()))
        }
        "config_world_map" => Some(parse_config_world_map(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}
pub(crate) fn verify_map_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    if let Some(res) = verify_map_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_map_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_map_chunk_2(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_map_chunk_3(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_map_chunk_4(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_map_chunk_5(stem, buf, scratch) {
        return Some(res);
    }
    None
}

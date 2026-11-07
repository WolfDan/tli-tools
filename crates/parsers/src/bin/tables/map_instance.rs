//! Auto-generated `.bin` configuration table types and parsers for the `map_instance` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, parse_map_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{
    FConfigI32I32, FConfigI32Str, FConfigMapMonsterPointPointInfo,
};
use crate::bin::tables::system::{ConfigTaleHeroTable, ConfigTaleNpcTable};
use crate::bin::types::{BinLimits, BinTable, VecDim};
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_level_area.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelAreaRow {
    pub points: Box<[i32]>,
    pub monster_kinds: Box<[FConfigI32I32]>,
    pub tags: Box<[i32]>,
    pub danger_weights: Box<[FConfigI32I32]>,
    pub model_id: i32,
    pub group_id: i32,
    pub area_id: i32,
    pub weight: i32,
    pub danger_percent: i32,
}
impl BinRow for ConfigLevelAreaRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.model_id = cur.read_i32()?;
        self.group_id = cur.read_i32()?;
        self.area_id = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        self.points = cur.read_array_i32()?;
        self.monster_kinds = FConfigI32I32::read_array(cur)?;
        self.tags = cur.read_array_i32()?;
        self.danger_percent = cur.read_i32()?;
        self.danger_weights = FConfigI32I32::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_area.bin`.
pub type ConfigLevelAreaTable = BinTable<(i32, i32), ConfigLevelAreaRow>;
/// Parses `Maps/<Map>/config_level_area.bin` into a [`ConfigLevelAreaTable`].
pub fn parse_config_level_area(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigLevelAreaTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_level_plot.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelPlotRow {
    pub tags: Box<[i32]>,
    pub position: VecDim,
    pub wall_position: Box<[VecDim]>,
    pub wall_rotation: Box<[f64]>,
    pub model_id: i32,
    pub id: i32,
    pub plot_map_id: i32,
    pub logic_point_id: i32,
    pub walk_direction: i32,
}
impl BinRow for ConfigLevelPlotRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.model_id = cur.read_i32()?;
        self.id = cur.read_i32()?;
        self.plot_map_id = cur.read_i32()?;
        self.tags = cur.read_array_i32()?;
        self.position = cur.read_vec_dim()?;
        self.wall_position = cur.read_array_vec_dim()?;
        self.wall_rotation = cur.read_array_f64()?;
        self.logic_point_id = cur.read_i32()?;
        self.walk_direction = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_plot.bin`.
pub type ConfigLevelPlotTable = BinTable<(i32, i32), ConfigLevelPlotRow>;
/// Parses `Maps/<Map>/config_level_plot.bin` into a [`ConfigLevelPlotTable`].
pub fn parse_config_level_plot(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigLevelPlotTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_level_plot_logic_point.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelPlotLogicPointRow {
    pub pos: VecDim,
    pub yaw: f64,
    pub model_id: i32,
    pub logic_id: i32,
    pub map_id: i32,
    pub point_type: i32,
    pub point_unique_id: i32,
    pub point_radius: i32,
}
impl BinRow for ConfigLevelPlotLogicPointRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.model_id = cur.read_i32()?;
        self.logic_id = cur.read_i32()?;
        self.map_id = cur.read_i32()?;
        self.point_type = cur.read_i32()?;
        self.point_unique_id = cur.read_i32()?;
        self.point_radius = cur.read_i32()?;
        self.pos = cur.read_vec_dim()?;
        self.yaw = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_plot_logic_point.bin`.
pub type ConfigLevelPlotLogicPointTable = BinTable<
    (i32, i32, i32, i32),
    ConfigLevelPlotLogicPointRow,
>;
/// Parses `Maps/<Map>/config_level_plot_logic_point.bin` into a [`ConfigLevelPlotLogicPointTable`].
pub fn parse_config_level_plot_logic_point(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigLevelPlotLogicPointTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_level_point_distance.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelPointDistanceRow {
    pub grid: FxHashMap<i32, Box<[i32]>>,
    pub points_index: FxHashMap<i32, i32>,
    pub model_id: i32,
    pub r#type: i32,
}
impl BinRow for ConfigLevelPointDistanceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.model_id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.grid = cur.read_map_i32_array_i32()?;
        self.points_index = cur.read_map_i32_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_point_distance.bin`.
pub type ConfigLevelPointDistanceTable = BinTable<i32, ConfigLevelPointDistanceRow>;
/// Parses `Maps/<Map>/config_level_point_distance.bin` into a [`ConfigLevelPointDistanceTable`].
pub fn parse_config_level_point_distance(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigLevelPointDistanceTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_level_preset_template.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelPresetTemplateRow {
    pub name: Box<str>,
    pub function: Box<[FConfigI32I32]>,
    pub points: Box<[i32]>,
    pub sleep_points: Box<[i32]>,
    pub model_id: i32,
    pub group_id: i32,
    pub id: i32,
}
impl BinRow for ConfigLevelPresetTemplateRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.model_id = cur.read_i32()?;
        self.group_id = cur.read_i32()?;
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.function = FConfigI32I32::read_array(cur)?;
        self.points = cur.read_array_i32()?;
        self.sleep_points = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_preset_template.bin`.
pub type ConfigLevelPresetTemplateTable = BinTable<
    (i32, i32),
    ConfigLevelPresetTemplateRow,
>;
/// Parses `Maps/<Map>/config_level_preset_template.bin` into a [`ConfigLevelPresetTemplateTable`].
pub fn parse_config_level_preset_template(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigLevelPresetTemplateTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_art_actor.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapArtActorRow {
    pub pos: Box<[f64]>,
    pub rot: Box<[f64]>,
    pub scale: Box<[f64]>,
    pub path: Box<str>,
    pub ref_res_path: Box<str>,
    pub material: Box<[FConfigI32Str]>,
    pub map_id: i32,
    pub group: i32,
    pub actor_type: i32,
    pub cfg_id: i32,
    pub unique_id: i32,
}
impl BinRow for ConfigMapArtActorRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.group = cur.read_i32()?;
        self.actor_type = cur.read_i32()?;
        self.pos = cur.read_array_f64()?;
        self.rot = cur.read_array_f64()?;
        self.scale = cur.read_array_f64()?;
        self.path = cur.read_str()?;
        self.cfg_id = cur.read_i32()?;
        self.unique_id = cur.read_i32()?;
        self.ref_res_path = cur.read_str()?;
        self.material = FConfigI32Str::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_art_actor.bin`.
pub type ConfigMapArtActorTable = BinTable<(i32, i32, i32), ConfigMapArtActorRow>;
/// Parses `Maps/<Map>/config_map_art_actor.bin` into a [`ConfigMapArtActorTable`].
pub fn parse_config_map_art_actor(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapArtActorTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_art_camera_actor.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapArtCameraActorRow {
    pub shape: Box<[i32]>,
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
    pub distance: f64,
    pub blend_time: f64,
    pub map_id: i32,
    pub group: i32,
    pub unique_id: i32,
    pub trigger_type: i32,
    pub fog_height: i32,
}
impl BinRow for ConfigMapArtCameraActorRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.group = cur.read_i32()?;
        self.unique_id = cur.read_i32()?;
        self.trigger_type = cur.read_i32()?;
        self.shape = cur.read_array_i32()?;
        self.roll = cur.read_f64()?;
        self.pitch = cur.read_f64()?;
        self.yaw = cur.read_f64()?;
        self.distance = cur.read_f64()?;
        self.blend_time = cur.read_f64()?;
        self.fog_height = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_art_camera_actor.bin`.
pub type ConfigMapArtCameraActorTable = BinTable<(i32, i32), ConfigMapArtCameraActorRow>;
/// Parses `Maps/<Map>/config_map_art_camera_actor.bin` into a [`ConfigMapArtCameraActorTable`].
pub fn parse_config_map_art_camera_actor(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapArtCameraActorTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_auto_gen_circle.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapAutoGenCircleRow {
    pub center: VecDim,
    pub radius: f64,
    pub points: Box<[i32]>,
    pub map_id: i32,
    pub scheme_id: i32,
    pub circle_id: i32,
    pub gameplay_id: i32,
    pub gameplay_type: i32,
}
impl BinRow for ConfigMapAutoGenCircleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.scheme_id = cur.read_i32()?;
        self.circle_id = cur.read_i32()?;
        self.center = cur.read_vec_dim()?;
        self.radius = cur.read_f64()?;
        self.gameplay_id = cur.read_i32()?;
        self.gameplay_type = cur.read_i32()?;
        self.points = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_auto_gen_circle.bin`.
pub type ConfigMapAutoGenCircleTable = BinTable<(i32, i32), ConfigMapAutoGenCircleRow>;
/// Parses `Maps/<Map>/config_map_auto_gen_circle.bin` into a [`ConfigMapAutoGenCircleTable`].
pub fn parse_config_map_auto_gen_circle(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapAutoGenCircleTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_auto_gen_inner_circle.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapAutoGenInnerCircleRow {
    pub center: VecDim,
    pub radius: f64,
    pub points: Box<[i32]>,
    pub map_id: i32,
    pub scheme_id: i32,
    pub circle_id: i32,
    pub inner_circle_id: i32,
}
impl BinRow for ConfigMapAutoGenInnerCircleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.scheme_id = cur.read_i32()?;
        self.circle_id = cur.read_i32()?;
        self.inner_circle_id = cur.read_i32()?;
        self.center = cur.read_vec_dim()?;
        self.radius = cur.read_f64()?;
        self.points = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_auto_gen_inner_circle.bin`.
pub type ConfigMapAutoGenInnerCircleTable = BinTable<
    (i32, i32, i32),
    ConfigMapAutoGenInnerCircleRow,
>;
/// Parses `Maps/<Map>/config_map_auto_gen_inner_circle.bin` into a [`ConfigMapAutoGenInnerCircleTable`].
pub fn parse_config_map_auto_gen_inner_circle(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapAutoGenInnerCircleTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_auto_gen_point.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapAutoGenPointRow {
    pub pos: VecDim,
    pub radius: f64,
    pub neighbors: Box<[i32]>,
    pub neighbors_interval: Box<[i32]>,
    pub map_id: i32,
    pub point_id: i32,
    pub circle_id: i32,
    pub scheme_id: i32,
}
impl BinRow for ConfigMapAutoGenPointRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.point_id = cur.read_i32()?;
        self.circle_id = cur.read_i32()?;
        self.pos = cur.read_vec_dim()?;
        self.radius = cur.read_f64()?;
        self.neighbors = cur.read_array_i32()?;
        self.neighbors_interval = cur.read_array_i32()?;
        self.scheme_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_auto_gen_point.bin`.
pub type ConfigMapAutoGenPointTable = BinTable<(i32, i32), ConfigMapAutoGenPointRow>;
/// Parses `Maps/<Map>/config_map_auto_gen_point.bin` into a [`ConfigMapAutoGenPointTable`].
pub fn parse_config_map_auto_gen_point(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapAutoGenPointTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_elevator.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapElevatorRow {
    pub des: Box<str>,
    pub size: VecDim,
    pub height: f64,
    pub time: f64,
    pub wait_time: f64,
    pub top_stop_blocks: Box<[i32]>,
    pub bottom_stop_blocks: Box<[i32]>,
    pub move_mesh: Box<str>,
    pub trigger_area: VecDim,
    pub block_pos: VecDim,
    pub front_back_block_size: VecDim,
    pub left_right_block_size: VecDim,
    pub map_id: i32,
    pub id: i32,
    pub start_pos_type: i32,
    pub start_anim: i32,
    pub run_anim: i32,
    pub end_anim: i32,
    pub trigger_range: i32,
    pub player_trigger: bool,
}
impl ConfigMapElevatorRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.size = cur.read_vec_dim()?;
        self.height = cur.read_f64()?;
        self.time = cur.read_f64()?;
        self.wait_time = cur.read_f64()?;
        self.start_pos_type = cur.read_i32()?;
        self.top_stop_blocks = cur.read_array_i32()?;
        self.bottom_stop_blocks = cur.read_array_i32()?;
        self.move_mesh = cur.read_str()?;
        self.player_trigger = cur.read_bool()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.trigger_area = cur.read_vec_dim()?;
        self.start_anim = cur.read_i32()?;
        self.run_anim = cur.read_i32()?;
        self.end_anim = cur.read_i32()?;
        self.block_pos = cur.read_vec_dim()?;
        self.front_back_block_size = cur.read_vec_dim()?;
        self.left_right_block_size = cur.read_vec_dim()?;
        self.trigger_range = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapElevatorRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_elevator.bin`.
pub type ConfigMapElevatorTable = BinTable<i32, ConfigMapElevatorRow>;
/// Parses `Maps/<Map>/config_map_elevator.bin` into a [`ConfigMapElevatorTable`].
pub fn parse_config_map_elevator(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapElevatorTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_gameplay_point.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplayPointRow {
    pub r#type: Box<[i32]>,
    pub pos: VecDim,
    pub yaw: f64,
    pub level_id: i32,
    pub unique_id: i32,
    pub area_id: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
    pub type5: i32,
    pub plot_id: i32,
}
impl BinRow for ConfigMapGameplayPointRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.level_id = cur.read_i32()?;
        self.unique_id = cur.read_i32()?;
        self.area_id = cur.read_i32()?;
        self.r#type = cur.read_array_i32()?;
        self.pos = cur.read_vec_dim()?;
        self.yaw = cur.read_f64()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        self.type5 = cur.read_i32()?;
        self.plot_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_gameplay_point.bin`.
pub type ConfigMapGameplayPointTable = BinTable<i32, ConfigMapGameplayPointRow>;
/// Parses `Maps/<Map>/config_map_gameplay_point.bin` into a [`ConfigMapGameplayPointTable`].
pub fn parse_config_map_gameplay_point(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapGameplayPointTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_level_pos_record.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapLevelPosRecordRow {
    pub name: Box<str>,
    pub pos: VecDim,
    pub map_id: i32,
    pub index: i32,
}
impl BinRow for ConfigMapLevelPosRecordRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.index = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.pos = cur.read_vec_dim()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_level_pos_record.bin`.
pub type ConfigMapLevelPosRecordTable = BinTable<i32, ConfigMapLevelPosRecordRow>;
/// Parses `Maps/<Map>/config_map_level_pos_record.bin` into a [`ConfigMapLevelPosRecordTable`].
pub fn parse_config_map_level_pos_record(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapLevelPosRecordTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_main_city_start_pos.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapMainCityStartPosRow {
    pub pos: VecDim,
    pub level_id: i32,
    pub unique_id: i32,
    pub r#type: i32,
}
impl BinRow for ConfigMapMainCityStartPosRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.level_id = cur.read_i32()?;
        self.unique_id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.pos = cur.read_vec_dim()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_main_city_start_pos.bin`.
pub type ConfigMapMainCityStartPosTable = BinTable<i32, ConfigMapMainCityStartPosRow>;
/// Parses `Maps/<Map>/config_map_main_city_start_pos.bin` into a [`ConfigMapMainCityStartPosTable`].
pub fn parse_config_map_main_city_start_pos(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapMainCityStartPosTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_monster_point.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapMonsterPointRow {
    pub point_info: FConfigMapMonsterPointPointInfo,
    pub affix: Box<[FConfigAffix]>,
    pub tags: Box<[i32]>,
    pub level_id: i32,
    pub monster_point_id: i32,
    pub speed: i32,
    pub defend_range: i32,
    pub chase_range: i32,
    pub template_id: i32,
    pub side: i32,
    pub rect_yaw: i32,
    pub rect_column: i32,
    pub source_type: i32,
    pub plot_id: i32,
}
impl ConfigMapMonsterPointRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.level_id = cur.read_i32()?;
        self.monster_point_id = cur.read_i32()?;
        self.point_info = FConfigMapMonsterPointPointInfo::read(cur)?;
        self.speed = cur.read_i32()?;
        self.affix = FConfigAffix::read_array(cur)?;
        self.defend_range = cur.read_i32()?;
        self.chase_range = cur.read_i32()?;
        self.template_id = cur.read_i32()?;
        self.side = cur.read_i32()?;
        self.rect_yaw = cur.read_i32()?;
        self.rect_column = cur.read_i32()?;
        self.tags = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.source_type = cur.read_i32()?;
        self.plot_id = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigMapMonsterPointRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_monster_point.bin`.
pub type ConfigMapMonsterPointTable = BinTable<i32, ConfigMapMonsterPointRow>;
/// Parses `Maps/<Map>/config_map_monster_point.bin` into a [`ConfigMapMonsterPointTable`].
pub fn parse_config_map_monster_point(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapMonsterPointTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_object.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapObjectRow {
    pub object_name: Box<str>,
    pub pos: Box<[VecDim]>,
    pub id: i32,
    pub map_id: i32,
    pub object_id: i32,
    pub speed: i32,
    pub collision_mode: i32,
    pub collision_through: i32,
    pub for_sequence_hide: i32,
}
impl BinRow for ConfigMapObjectRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.map_id = cur.read_i32()?;
        self.object_id = cur.read_i32()?;
        self.object_name = cur.read_str()?;
        self.speed = cur.read_i32()?;
        self.collision_mode = cur.read_i32()?;
        self.collision_through = cur.read_i32()?;
        self.pos = cur.read_array_vec_dim()?;
        self.for_sequence_hide = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_object.bin`.
pub type ConfigMapObjectTable = BinTable<i32, ConfigMapObjectRow>;
/// Parses `Maps/<Map>/config_map_object.bin` into a [`ConfigMapObjectTable`].
pub fn parse_config_map_object(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapObjectTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_revive.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapReviveRow {
    pub des: Box<str>,
    pub triggers: Box<str>,
    pub map_id: i32,
    pub id: i32,
    pub shape_id: i32,
    pub shape: i32,
}
impl BinRow for ConfigMapReviveRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.shape_id = cur.read_i32()?;
        self.shape = cur.read_i32()?;
        self.triggers = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_revive.bin`.
pub type ConfigMapReviveTable = BinTable<i32, ConfigMapReviveRow>;
/// Parses `Maps/<Map>/config_map_revive.bin` into a [`ConfigMapReviveTable`].
pub fn parse_config_map_revive(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapReviveTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_transform_point.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapTransformPointRow {
    pub pos: VecDim,
    pub yaw: f64,
    pub level_id: i32,
    pub unique_id: i32,
}
impl BinRow for ConfigMapTransformPointRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.level_id = cur.read_i32()?;
        self.unique_id = cur.read_i32()?;
        self.pos = cur.read_vec_dim()?;
        self.yaw = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_transform_point.bin`.
pub type ConfigMapTransformPointTable = BinTable<i32, ConfigMapTransformPointRow>;
/// Parses `Maps/<Map>/config_map_transform_point.bin` into a [`ConfigMapTransformPointTable`].
pub fn parse_config_map_transform_point(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapTransformPointTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_trigger_auto.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapTriggerAutoRow {
    pub events: Box<[FConfigI32Str]>,
    pub conditions: Box<[FConfigI32Str]>,
    pub actions: Box<[FConfigI32Str]>,
    pub tick_interval: f64,
    pub id: i32,
    pub level_id: i32,
    pub object_id: i32,
    pub auto_close: i32,
    pub condition_type: i32,
    pub condition_check_type: i32,
    pub enable: bool,
}
impl BinRow for ConfigMapTriggerAutoRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.level_id = cur.read_i32()?;
        self.object_id = cur.read_i32()?;
        self.enable = cur.read_bool()?;
        self.auto_close = cur.read_i32()?;
        self.events = FConfigI32Str::read_array(cur)?;
        self.conditions = FConfigI32Str::read_array(cur)?;
        self.actions = FConfigI32Str::read_array(cur)?;
        self.tick_interval = cur.read_f64()?;
        self.condition_type = cur.read_i32()?;
        self.condition_check_type = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_trigger_auto.bin`.
pub type ConfigMapTriggerAutoTable = BinTable<i32, ConfigMapTriggerAutoRow>;
/// Parses `Maps/<Map>/config_map_trigger_auto.bin` into a [`ConfigMapTriggerAutoTable`].
pub fn parse_config_map_trigger_auto(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapTriggerAutoTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Row record for `config_map_waypoint.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapWaypointRow {
    pub pos: VecDim,
    pub trans_info: Box<[FConfigI32I32]>,
    pub map_id: i32,
    pub group: i32,
    pub lane_id: i32,
    pub index: i32,
    pub jump_way_point: FConfigI32I32,
}
impl BinRow for ConfigMapWaypointRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_id = cur.read_i32()?;
        self.group = cur.read_i32()?;
        self.lane_id = cur.read_i32()?;
        self.index = cur.read_i32()?;
        self.pos = cur.read_vec_dim()?;
        self.trans_info = FConfigI32I32::read_array(cur)?;
        self.jump_way_point = FConfigI32I32::read(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_map_waypoint.bin`.
pub type ConfigMapWaypointTable = BinTable<(i32, i32, i32), ConfigMapWaypointRow>;
/// Parses `Maps/<Map>/config_map_waypoint.bin` into a [`ConfigMapWaypointTable`].
pub fn parse_config_map_waypoint(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigMapWaypointTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Parses `Maps/<Map>/config_tale_hero.bin` into a [`ConfigTaleHeroTable`].
pub fn parse_map_config_tale_hero(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigTaleHeroTable, BinError> {
    parse_map_bin_table(buf, limits)
}
/// Parses `Maps/<Map>/config_tale_npc.bin` into a [`ConfigTaleNpcTable`].
pub fn parse_map_config_tale_npc(
    buf: &[u8],
    limits: BinLimits,
) -> Result<ConfigTaleNpcTable, BinError> {
    parse_map_bin_table(buf, limits)
}
#[inline(never)]
fn verify_map_instance_chunk_0(
    stem: &str,
    buf: &[u8],
    limits: BinLimits,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_level_area" => {
            Some(parse_config_level_area(buf, limits).map(|t| t.len()))
        }
        "config_level_plot" => {
            Some(parse_config_level_plot(buf, limits).map(|t| t.len()))
        }
        "config_level_plot_logic_point" => {
            Some(parse_config_level_plot_logic_point(buf, limits).map(|t| t.len()))
        }
        "config_level_point_distance" => {
            Some(parse_config_level_point_distance(buf, limits).map(|t| t.len()))
        }
        "config_level_preset_template" => {
            Some(parse_config_level_preset_template(buf, limits).map(|t| t.len()))
        }
        "config_map_art_actor" => {
            Some(parse_config_map_art_actor(buf, limits).map(|t| t.len()))
        }
        "config_map_art_camera_actor" => {
            Some(parse_config_map_art_camera_actor(buf, limits).map(|t| t.len()))
        }
        "config_map_auto_gen_circle" => {
            Some(parse_config_map_auto_gen_circle(buf, limits).map(|t| t.len()))
        }
        "config_map_auto_gen_inner_circle" => {
            Some(parse_config_map_auto_gen_inner_circle(buf, limits).map(|t| t.len()))
        }
        "config_map_auto_gen_point" => {
            Some(parse_config_map_auto_gen_point(buf, limits).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_map_instance_chunk_1(
    stem: &str,
    buf: &[u8],
    limits: BinLimits,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_map_elevator" => {
            Some(parse_config_map_elevator(buf, limits).map(|t| t.len()))
        }
        "config_map_gameplay_point" => {
            Some(parse_config_map_gameplay_point(buf, limits).map(|t| t.len()))
        }
        "config_map_level_pos_record" => {
            Some(parse_config_map_level_pos_record(buf, limits).map(|t| t.len()))
        }
        "config_map_main_city_start_pos" => {
            Some(parse_config_map_main_city_start_pos(buf, limits).map(|t| t.len()))
        }
        "config_map_monster_point" => {
            Some(parse_config_map_monster_point(buf, limits).map(|t| t.len()))
        }
        "config_map_object" => {
            Some(parse_config_map_object(buf, limits).map(|t| t.len()))
        }
        "config_map_revive" => {
            Some(parse_config_map_revive(buf, limits).map(|t| t.len()))
        }
        "config_map_transform_point" => {
            Some(parse_config_map_transform_point(buf, limits).map(|t| t.len()))
        }
        "config_map_trigger_auto" => {
            Some(parse_config_map_trigger_auto(buf, limits).map(|t| t.len()))
        }
        "config_map_waypoint" => {
            Some(parse_config_map_waypoint(buf, limits).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_map_instance_chunk_2(
    stem: &str,
    buf: &[u8],
    limits: BinLimits,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_tale_hero" => {
            Some(parse_map_config_tale_hero(buf, limits).map(|t| t.len()))
        }
        "config_tale_npc" => {
            Some(parse_map_config_tale_npc(buf, limits).map(|t| t.len()))
        }
        _ => None,
    }
}
pub(crate) fn verify_map_instance_bin_by_stem(
    stem: &str,
    buf: &[u8],
    limits: BinLimits,
) -> Option<Result<u32, BinError>> {
    if let Some(res) = verify_map_instance_chunk_0(stem, buf, limits) {
        return Some(res);
    }
    if let Some(res) = verify_map_instance_chunk_1(stem, buf, limits) {
        return Some(res);
    }
    if let Some(res) = verify_map_instance_chunk_2(stem, buf, limits) {
        return Some(res);
    }
    None
}

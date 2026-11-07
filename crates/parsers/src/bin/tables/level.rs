//! Auto-generated `.bin` configuration table types and parsers for the `level` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{
    FConfigF64x4, FConfigI32ArrayI32, FConfigI32I32, FConfigStrArrayI32,
};
use crate::bin::types::{BinTable, VecDim};
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_Level_loading_pic_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelLoadingPicRow {
    pub loading_pic_id: Box<[i32]>,
    pub id: i32,
    pub level_type: i32,
}
impl BinRow for ConfigLevelLoadingPicRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.level_type = cur.read_i32()?;
        self.loading_pic_id = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_Level_loading_pic_ChunkA0.bin`.
pub type ConfigLevelLoadingPicTable = BinTable<i32, ConfigLevelLoadingPicRow>;
/// Parses `config_Level_loading_pic_ChunkA0.bin` into a [`ConfigLevelLoadingPicTable`].
pub fn parse_config_level_loading_pic(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelLoadingPicTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_chapter_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigChapterRow {
    pub name: Box<str>,
    pub vice_name: Box<str>,
    pub order: Box<str>,
    pub main_story_id: Box<[i32]>,
    pub main_city_transport: VecDim,
    pub chapter_map_resources: Box<str>,
    pub camera_coordinate: VecDim,
    pub camera_range: Box<[i32]>,
    pub camera_range_proportion: Box<[i32]>,
    pub camera_zoom_range: Box<[i32]>,
    pub ui_zoom_range: f64,
    pub camera_show_time: f64,
    pub camera_back_speed: Box<str>,
    pub change_chapter_resources: Box<str>,
    pub id: i32,
    pub main_city_id: i32,
    pub resources_rotate: i32,
    pub camera_speed: i32,
    pub camera_zoom_speed: i32,
    pub camera_show_level: i32,
    pub camera_show_deviation: i32,
    pub camera_level: i32,
}
impl ConfigChapterRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.vice_name = cur.read_str()?;
        self.order = cur.read_str()?;
        self.main_story_id = cur.read_array_i32()?;
        self.main_city_id = cur.read_i32()?;
        self.main_city_transport = cur.read_vec_dim()?;
        self.chapter_map_resources = cur.read_str()?;
        self.resources_rotate = cur.read_i32()?;
        self.camera_coordinate = cur.read_vec_dim()?;
        self.camera_range = cur.read_array_i32()?;
        self.camera_range_proportion = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.camera_speed = cur.read_i32()?;
        self.camera_zoom_range = cur.read_array_i32()?;
        self.camera_zoom_speed = cur.read_i32()?;
        self.camera_show_level = cur.read_i32()?;
        self.ui_zoom_range = cur.read_f64()?;
        self.camera_show_deviation = cur.read_i32()?;
        self.camera_show_time = cur.read_f64()?;
        self.camera_level = cur.read_i32()?;
        self.camera_back_speed = cur.read_str()?;
        self.change_chapter_resources = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigChapterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_chapter_ChunkA0.bin`.
pub type ConfigChapterTable = BinTable<i32, ConfigChapterRow>;
/// Parses `config_chapter_ChunkA0.bin` into a [`ConfigChapterTable`].
pub fn parse_config_chapter(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigChapterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_guide_condition_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuideConditionRow {
    pub cond_attr: FxHashMap<i32, i32>,
    pub id: i32,
    pub cond_page: i32,
}
impl BinRow for ConfigGuideConditionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.cond_page = cur.read_i32()?;
        self.cond_attr = cur.read_map_i32_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_guide_condition_ChunkA0.bin`.
pub type ConfigGuideConditionTable = BinTable<i32, ConfigGuideConditionRow>;
/// Parses `config_guide_condition_ChunkA0.bin` into a [`ConfigGuideConditionTable`].
pub fn parse_config_guide_condition(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGuideConditionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_guide_config_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuideConfigRow {
    pub guide_sequence: FxHashMap<i32, i32>,
    pub break_connect: Box<[FConfigI32ArrayI32]>,
    pub id: i32,
    pub trigger_condition: i32,
    pub execute_priority: i32,
}
impl BinRow for ConfigGuideConfigRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.guide_sequence = cur.read_map_i32_i32()?;
        self.trigger_condition = cur.read_i32()?;
        self.break_connect = FConfigI32ArrayI32::read_array(cur)?;
        self.execute_priority = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_guide_config_ChunkA0.bin`.
pub type ConfigGuideConfigTable = BinTable<i32, ConfigGuideConfigRow>;
/// Parses `config_guide_config_ChunkA0.bin` into a [`ConfigGuideConfigTable`].
pub fn parse_config_guide_config(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGuideConfigTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_guide_item_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuideItemRow {
    pub guide_tip: Box<str>,
    pub id: i32,
    pub guide_type: bool,
    pub guide_bone: bool,
}
impl BinRow for ConfigGuideItemRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.guide_type = cur.read_bool()?;
        self.guide_bone = cur.read_bool()?;
        self.guide_tip = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_guide_item_ChunkA0.bin`.
pub type ConfigGuideItemTable = BinTable<i32, ConfigGuideItemRow>;
/// Parses `config_guide_item_ChunkA0.bin` into a [`ConfigGuideItemTable`].
pub fn parse_config_guide_item(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGuideItemTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_guide_parameter_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuideParameterRow {
    pub parm_name: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigGuideParameterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.parm_name = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_guide_parameter_ChunkA0.bin`.
pub type ConfigGuideParameterTable = BinTable<i32, ConfigGuideParameterRow>;
/// Parses `config_guide_parameter_ChunkA0.bin` into a [`ConfigGuideParameterTable`].
pub fn parse_config_guide_parameter(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGuideParameterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelRow {
    pub name: Box<str>,
    pub start_pos: VecDim,
    pub start_rotate: VecDim,
    pub born_group: Box<[i32]>,
    pub trigger_group: Box<[i32]>,
    pub art_group: Box<[i32]>,
    pub bgm: Box<str>,
    pub camera_vfx: Box<[i32]>,
    pub born_num: Box<[i32]>,
    pub rare_num: Box<[FConfigI32ArrayI32]>,
    pub outside_list: Box<[i32]>,
    pub outside_rand_list: Box<[FConfigI32I32]>,
    pub outside_monster_affix: Box<[FConfigI32I32]>,
    pub ect_num: Box<[FConfigI32ArrayI32]>,
    pub template_limit: Box<[FConfigStrArrayI32]>,
    pub special_drop: FxHashMap<i32, Box<[i32]>>,
    pub init_map_modifier: Box<[i32]>,
    pub random_modifier_number: Box<[FConfigI32I32]>,
    pub random_map_modifier: Box<[FConfigI32I32]>,
    pub des: Box<str>,
    pub safebox_num: Box<[i32]>,
    pub drop_show: Box<[FConfigI32I32]>,
    pub gameplay_random_ids: Box<[i32]>,
    pub loading_pic: Box<[i32]>,
    pub s5_trigger_group: Box<[i32]>,
    pub force_outside_monster_affix: Box<[FConfigI32I32]>,
    pub id: i32,
    pub model: i32,
    pub init_direction: i32,
    pub waypoint_group: i32,
    pub base_level: i32,
    pub outside_diff_id: i32,
    pub disappernum: i32,
    pub r#type: i32,
    pub drop_type: i32,
    pub gameplay_random_id: i32,
    pub outside_rand_num: i32,
    pub plot_map_id: i32,
    pub close_performance_limit: bool,
    pub random_direction: bool,
    pub show_init_map_modifier: bool,
    pub force_outside_gen: bool,
    pub disable_sp_map_affix_base_weight: bool,
    pub disable_sp_modifier_group: bool,
}
impl ConfigLevelRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.model = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.close_performance_limit = cur.read_bool()?;
        self.start_pos = cur.read_vec_dim()?;
        self.start_rotate = cur.read_vec_dim()?;
        self.init_direction = cur.read_i32()?;
        self.random_direction = cur.read_bool()?;
        self.born_group = cur.read_array_i32()?;
        self.trigger_group = cur.read_array_i32()?;
        self.waypoint_group = cur.read_i32()?;
        self.art_group = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.base_level = cur.read_i32()?;
        self.outside_diff_id = cur.read_i32()?;
        self.bgm = cur.read_str()?;
        self.camera_vfx = cur.read_array_i32()?;
        self.disappernum = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.drop_type = cur.read_i32()?;
        self.born_num = cur.read_array_i32()?;
        self.rare_num = FConfigI32ArrayI32::read_array(cur)?;
        self.gameplay_random_id = cur.read_i32()?;
        self.outside_list = cur.read_array_i32()?;
        self.outside_rand_list = FConfigI32I32::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.outside_rand_num = cur.read_i32()?;
        self.outside_monster_affix = FConfigI32I32::read_array(cur)?;
        self.ect_num = FConfigI32ArrayI32::read_array(cur)?;
        self.template_limit = FConfigStrArrayI32::read_array(cur)?;
        self.special_drop = cur.read_map_i32_array_i32()?;
        self.show_init_map_modifier = cur.read_bool()?;
        self.init_map_modifier = cur.read_array_i32()?;
        self.random_modifier_number = FConfigI32I32::read_array(cur)?;
        self.random_map_modifier = FConfigI32I32::read_array(cur)?;
        self.des = cur.read_str()?;
        self.safebox_num = cur.read_array_i32()?;
        self.drop_show = FConfigI32I32::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.gameplay_random_ids = cur.read_array_i32()?;
        self.loading_pic = cur.read_array_i32()?;
        self.s5_trigger_group = cur.read_array_i32()?;
        self.force_outside_gen = cur.read_bool()?;
        self.force_outside_monster_affix = FConfigI32I32::read_array(cur)?;
        self.plot_map_id = cur.read_i32()?;
        self.disable_sp_map_affix_base_weight = cur.read_bool()?;
        self.disable_sp_modifier_group = cur.read_bool()?;
        Ok(())
    }
}
impl BinRow for ConfigLevelRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_ChunkA0.bin`.
pub type ConfigLevelTable = BinTable<i32, ConfigLevelRow>;
/// Parses `config_level_ChunkA0.bin` into a [`ConfigLevelTable`].
pub fn parse_config_level(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_effect_ground_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelEffectGroundRow {
    pub shape: Box<str>,
    pub pos_list: Box<[VecDim]>,
    pub speed: f64,
    pub life: f64,
    pub scale: f64,
    pub id: i32,
    pub buff_id: i32,
    pub buff_way: i32,
    pub vfx: i32,
}
impl BinRow for ConfigLevelEffectGroundRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.shape = cur.read_str()?;
        self.pos_list = cur.read_array_vec_dim()?;
        self.speed = cur.read_f64()?;
        self.buff_id = cur.read_i32()?;
        self.buff_way = cur.read_i32()?;
        self.vfx = cur.read_i32()?;
        self.life = cur.read_f64()?;
        self.scale = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_effect_ground_ChunkA0.bin`.
pub type ConfigLevelEffectGroundTable = BinTable<i32, ConfigLevelEffectGroundRow>;
/// Parses `config_level_effect_ground_ChunkA0.bin` into a [`ConfigLevelEffectGroundTable`].
pub fn parse_config_level_effect_ground(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelEffectGroundTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_effect_mpc_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelEffectMpcRow {
    pub s5_mask_ctrl: FConfigF64x4,
    pub s5_emissive_color: FConfigF64x4,
    pub s5_grass_leaf_color: FConfigF64x4,
    pub s5_grass_root_color: FConfigF64x4,
    pub light_intensity: f64,
    pub light_color: FConfigF64x4,
    pub fog_inscattering_color: FConfigF64x4,
    pub vignette_intensity: f64,
    pub die_jie_ctrl: FConfigF64x4,
    pub row_name: Box<str>,
    pub id: i32,
    pub merge_state: i32,
    pub switch_tex: i32,
}
impl ConfigLevelEffectMpcRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.merge_state = cur.read_i32()?;
        self.switch_tex = cur.read_i32()?;
        self.s5_mask_ctrl = FConfigF64x4::read(cur)?;
        self.s5_emissive_color = FConfigF64x4::read(cur)?;
        self.s5_grass_leaf_color = FConfigF64x4::read(cur)?;
        self.s5_grass_root_color = FConfigF64x4::read(cur)?;
        self.light_intensity = cur.read_f64()?;
        self.light_color = FConfigF64x4::read(cur)?;
        self.fog_inscattering_color = FConfigF64x4::read(cur)?;
        self.vignette_intensity = cur.read_f64()?;
        self.die_jie_ctrl = FConfigF64x4::read(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.row_name = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigLevelEffectMpcRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_effect_mpc_ChunkA0.bin`.
pub type ConfigLevelEffectMpcTable = BinTable<i32, ConfigLevelEffectMpcRow>;
/// Parses `config_level_effect_mpc_ChunkA0.bin` into a [`ConfigLevelEffectMpcTable`].
pub fn parse_config_level_effect_mpc(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelEffectMpcTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_effect_param_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelEffectParamRow {
    pub light_intensity: f64,
    pub fog_height_falloff: f64,
    pub fog_start_distance: f64,
    pub vignette_intensity: f64,
    pub saturation: f64,
    pub gain: f64,
    pub id: i32,
    pub fog_color: i32,
}
impl BinRow for ConfigLevelEffectParamRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.light_intensity = cur.read_f64()?;
        self.fog_color = cur.read_i32()?;
        self.fog_height_falloff = cur.read_f64()?;
        self.fog_start_distance = cur.read_f64()?;
        self.vignette_intensity = cur.read_f64()?;
        self.saturation = cur.read_f64()?;
        self.gain = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_effect_param_ChunkA0.bin`.
pub type ConfigLevelEffectParamTable = BinTable<i32, ConfigLevelEffectParamRow>;
/// Parses `config_level_effect_param_ChunkA0.bin` into a [`ConfigLevelEffectParamTable`].
pub fn parse_config_level_effect_param(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelEffectParamTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_monster_catalog_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelMonsterCatalogRow {
    pub name: Box<str>,
    pub groups: Box<[FConfigI32I32]>,
    pub id: i32,
    pub force_rare: i32,
}
impl BinRow for ConfigLevelMonsterCatalogRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.groups = FConfigI32I32::read_array(cur)?;
        self.force_rare = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_monster_catalog_ChunkA0.bin`.
pub type ConfigLevelMonsterCatalogTable = BinTable<i32, ConfigLevelMonsterCatalogRow>;
/// Parses `config_level_monster_catalog_ChunkA0.bin` into a [`ConfigLevelMonsterCatalogTable`].
pub fn parse_config_level_monster_catalog(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelMonsterCatalogTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_monster_catalog_content_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelMonsterCatalogContentRow {
    pub des: Box<str>,
    pub queue: Box<[i32]>,
    pub monster: Box<[FConfigI32I32]>,
    pub monster_num: Box<[i32]>,
    pub monster_affix: Box<[FConfigAffix]>,
    pub group_id: i32,
    pub seat: i32,
    pub magic: i32,
    pub rare: i32,
}
impl BinRow for ConfigLevelMonsterCatalogContentRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.group_id = cur.read_i32()?;
        self.seat = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.magic = cur.read_i32()?;
        self.rare = cur.read_i32()?;
        self.queue = cur.read_array_i32()?;
        self.monster = FConfigI32I32::read_array(cur)?;
        self.monster_num = cur.read_array_i32()?;
        self.monster_affix = FConfigAffix::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_monster_catalog_content_ChunkA0.bin`.
pub type ConfigLevelMonsterCatalogContentTable = BinTable<
    (i32, i32),
    ConfigLevelMonsterCatalogContentRow,
>;
/// Parses `config_level_monster_catalog_content_ChunkA0.bin` into a [`ConfigLevelMonsterCatalogContentTable`].
pub fn parse_config_level_monster_catalog_content(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelMonsterCatalogContentTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_monster_s5_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelMonsterS5Row {
    pub born_num: Box<[i32]>,
    pub rare_num: Box<[FConfigI32ArrayI32]>,
    pub outside_list: Box<[i32]>,
    pub outside_rand_list: Box<[FConfigI32I32]>,
    pub outside_monster_affix: Box<[FConfigI32I32]>,
    pub init_map_modifier: Box<[i32]>,
    pub random_modifier_number: Box<[FConfigI32I32]>,
    pub random_map_modifier: Box<[FConfigI32I32]>,
    pub id: i32,
    pub base_level: i32,
    pub outside_diff_id: i32,
    pub outside_rand_num: i32,
}
impl BinRow for ConfigLevelMonsterS5Row {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.base_level = cur.read_i32()?;
        self.outside_diff_id = cur.read_i32()?;
        self.born_num = cur.read_array_i32()?;
        self.rare_num = FConfigI32ArrayI32::read_array(cur)?;
        self.outside_list = cur.read_array_i32()?;
        self.outside_rand_list = FConfigI32I32::read_array(cur)?;
        self.outside_rand_num = cur.read_i32()?;
        self.outside_monster_affix = FConfigI32I32::read_array(cur)?;
        self.init_map_modifier = cur.read_array_i32()?;
        self.random_modifier_number = FConfigI32I32::read_array(cur)?;
        self.random_map_modifier = FConfigI32I32::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_monster_s5_ChunkA0.bin`.
pub type ConfigLevelMonsterS5Table = BinTable<i32, ConfigLevelMonsterS5Row>;
/// Parses `config_level_monster_s5_ChunkA0.bin` into a [`ConfigLevelMonsterS5Table`].
pub fn parse_config_level_monster_s5(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelMonsterS5Table, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_npc_info_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelNpcInfoRow {
    pub allow_level: Box<[FConfigI32I32]>,
    pub location: VecDim,
    pub scale: VecDim,
    pub rotation: f64,
    pub id: i32,
    pub map_id: i32,
    pub cfg_id: i32,
    pub monster_id: i32,
    pub any_level: bool,
}
impl BinRow for ConfigLevelNpcInfoRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.map_id = cur.read_i32()?;
        self.cfg_id = cur.read_i32()?;
        self.allow_level = FConfigI32I32::read_array(cur)?;
        self.location = cur.read_vec_dim()?;
        self.scale = cur.read_vec_dim()?;
        self.rotation = cur.read_f64()?;
        self.monster_id = cur.read_i32()?;
        self.any_level = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_npc_info_ChunkA0.bin`.
pub type ConfigLevelNpcInfoTable = BinTable<i32, ConfigLevelNpcInfoRow>;
/// Parses `config_level_npc_info_ChunkA0.bin` into a [`ConfigLevelNpcInfoTable`].
pub fn parse_config_level_npc_info(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelNpcInfoTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_outside_difficulty_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelOutsideDifficultyRow {
    pub outside_danger_weight: Box<[FConfigI32I32]>,
    pub init_map_modifier: Box<[i32]>,
    pub id: i32,
    pub outside_danger_prop: i32,
    pub add_level: i32,
}
impl BinRow for ConfigLevelOutsideDifficultyRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.outside_danger_prop = cur.read_i32()?;
        self.outside_danger_weight = FConfigI32I32::read_array(cur)?;
        self.add_level = cur.read_i32()?;
        self.init_map_modifier = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_outside_difficulty_ChunkA0.bin`.
pub type ConfigLevelOutsideDifficultyTable = BinTable<
    i32,
    ConfigLevelOutsideDifficultyRow,
>;
/// Parses `config_level_outside_difficulty_ChunkA0.bin` into a [`ConfigLevelOutsideDifficultyTable`].
pub fn parse_config_level_outside_difficulty(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelOutsideDifficultyTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_outside_modifier_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelOutsideModifierRow {
    pub affix: Box<[FConfigAffix]>,
    pub limit: Box<[i32]>,
    pub sort_info: Box<[i32]>,
    pub id: i32,
    pub group: i32,
    pub effect_zone: i32,
    pub hard: i32,
    pub blank: i32,
    pub magic: i32,
    pub rare: i32,
    pub level_limit: i32,
    pub ignore_limit: bool,
}
impl BinRow for ConfigLevelOutsideModifierRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.affix = FConfigAffix::read_array(cur)?;
        self.group = cur.read_i32()?;
        self.effect_zone = cur.read_i32()?;
        self.hard = cur.read_i32()?;
        self.blank = cur.read_i32()?;
        self.magic = cur.read_i32()?;
        self.rare = cur.read_i32()?;
        self.limit = cur.read_array_i32()?;
        self.ignore_limit = cur.read_bool()?;
        self.level_limit = cur.read_i32()?;
        self.sort_info = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_outside_modifier_ChunkA0.bin`.
pub type ConfigLevelOutsideModifierTable = BinTable<i32, ConfigLevelOutsideModifierRow>;
/// Parses `config_level_outside_modifier_ChunkA0.bin` into a [`ConfigLevelOutsideModifierTable`].
pub fn parse_config_level_outside_modifier(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelOutsideModifierTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_outside_monster_catalog_content_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelOutsideMonsterCatalogContentRow {
    pub des: Box<str>,
    pub monster: Box<[FConfigI32I32]>,
    pub monster_num: Box<[i32]>,
    pub monster_affix: Box<[FConfigAffix]>,
    pub affix_effect_zone: Box<[i32]>,
    pub exclude_outside_affix_id: Box<[i32]>,
    pub exclude_outside_affix_group: Box<[i32]>,
    pub kind: i32,
    pub danger_lv: i32,
    pub group_id: i32,
    pub seat: i32,
    pub magic: i32,
    pub rare: i32,
    pub force_rare: i32,
    pub monster_create_type: i32,
    pub monster_appear_id: i32,
    pub tag_catelog: i32,
}
impl ConfigLevelOutsideMonsterCatalogContentRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.kind = cur.read_i32()?;
        self.danger_lv = cur.read_i32()?;
        self.group_id = cur.read_i32()?;
        self.seat = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.magic = cur.read_i32()?;
        self.rare = cur.read_i32()?;
        self.force_rare = cur.read_i32()?;
        self.monster = FConfigI32I32::read_array(cur)?;
        self.monster_num = cur.read_array_i32()?;
        self.monster_affix = FConfigAffix::read_array(cur)?;
        self.monster_create_type = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.monster_appear_id = cur.read_i32()?;
        self.tag_catelog = cur.read_i32()?;
        self.affix_effect_zone = cur.read_array_i32()?;
        self.exclude_outside_affix_id = cur.read_array_i32()?;
        self.exclude_outside_affix_group = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigLevelOutsideMonsterCatalogContentRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_outside_monster_catalog_content_ChunkA0.bin`.
pub type ConfigLevelOutsideMonsterCatalogContentTable = BinTable<
    (i32, i32, i32, i32),
    ConfigLevelOutsideMonsterCatalogContentRow,
>;
/// Parses `config_level_outside_monster_catalog_content_ChunkA0.bin` into a [`ConfigLevelOutsideMonsterCatalogContentTable`].
pub fn parse_config_level_outside_monster_catalog_content(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelOutsideMonsterCatalogContentTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_overlap_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelOverlapRow {
    pub born_group: Box<[i32]>,
    pub id: i32,
}
impl BinRow for ConfigLevelOverlapRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.born_group = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_overlap_ChunkA0.bin`.
pub type ConfigLevelOverlapTable = BinTable<i32, ConfigLevelOverlapRow>;
/// Parses `config_level_overlap_ChunkA0.bin` into a [`ConfigLevelOverlapTable`].
pub fn parse_config_level_overlap(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelOverlapTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_sequence_set_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelSequenceSetRow {
    pub desc: Box<str>,
    pub teleport_position: VecDim,
    pub start_blend_time: f64,
    pub end_blend_time: f64,
    pub bind_track_name: Box<str>,
    pub sequence_transform: Box<[VecDim]>,
    pub id: i32,
    pub resource_id: i32,
    pub teleport_map_id: i32,
    pub post_new_bgm: i32,
    pub bind_npc_id: i32,
    pub enable_skip: bool,
    pub enable_teleport: bool,
    pub enable_invincible: bool,
    pub enable_hide_ui: bool,
    pub enable_hide_player: bool,
    pub enable_hide_portal: bool,
    pub enable_hide_custom_trap: bool,
    pub enable_hide_interactive_item: bool,
    pub enable_hide_map_object: bool,
    pub enable_hide_npc: bool,
    pub enable_pause_bgm: bool,
    pub enable_camera_controll: bool,
}
impl ConfigLevelSequenceSetRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.resource_id = cur.read_i32()?;
        self.enable_skip = cur.read_bool()?;
        self.enable_teleport = cur.read_bool()?;
        self.teleport_map_id = cur.read_i32()?;
        self.teleport_position = cur.read_vec_dim()?;
        self.enable_invincible = cur.read_bool()?;
        self.enable_hide_ui = cur.read_bool()?;
        self.enable_hide_player = cur.read_bool()?;
        self.enable_hide_portal = cur.read_bool()?;
        self.enable_hide_custom_trap = cur.read_bool()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.enable_hide_interactive_item = cur.read_bool()?;
        self.enable_hide_map_object = cur.read_bool()?;
        self.enable_hide_npc = cur.read_bool()?;
        self.enable_pause_bgm = cur.read_bool()?;
        self.post_new_bgm = cur.read_i32()?;
        self.start_blend_time = cur.read_f64()?;
        self.end_blend_time = cur.read_f64()?;
        self.bind_npc_id = cur.read_i32()?;
        self.bind_track_name = cur.read_str()?;
        self.sequence_transform = cur.read_array_vec_dim()?;
        self.enable_camera_controll = cur.read_bool()?;
        Ok(())
    }
}
impl BinRow for ConfigLevelSequenceSetRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_sequence_set_ChunkA0.bin`.
pub type ConfigLevelSequenceSetTable = BinTable<i32, ConfigLevelSequenceSetRow>;
/// Parses `config_level_sequence_set_ChunkA0.bin` into a [`ConfigLevelSequenceSetTable`].
pub fn parse_config_level_sequence_set(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelSequenceSetTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_sequence_subtitle_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigLevelSequenceSubtitleRow {
    pub start_time: f64,
    pub stay_time: f64,
    pub subtitle_text: Box<str>,
    pub id: i32,
    pub step: i32,
}
impl BinRow for ConfigLevelSequenceSubtitleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.step = cur.read_i32()?;
        self.start_time = cur.read_f64()?;
        self.stay_time = cur.read_f64()?;
        self.subtitle_text = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_sequence_subtitle_ChunkA0.bin`.
pub type ConfigLevelSequenceSubtitleTable = BinTable<
    (i32, i32),
    ConfigLevelSequenceSubtitleRow,
>;
/// Parses `config_level_sequence_subtitle_ChunkA0.bin` into a [`ConfigLevelSequenceSubtitleTable`].
pub fn parse_config_level_sequence_subtitle(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelSequenceSubtitleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_skill_level_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelSkillLevelRow {
    pub level: i32,
    pub skill_level: i32,
}
impl BinRow for ConfigLevelSkillLevelRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.level = cur.read_i32()?;
        self.skill_level = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_skill_level_ChunkA0.bin`.
pub type ConfigLevelSkillLevelTable = BinTable<i32, ConfigLevelSkillLevelRow>;
/// Parses `config_level_skill_level_ChunkA0.bin` into a [`ConfigLevelSkillLevelTable`].
pub fn parse_config_level_skill_level(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelSkillLevelTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_level_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
    pub cache_num: i32,
    pub max_frame: i32,
    pub always_new: i32,
}
impl BinRow for ConfigLevelTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.cache_num = cur.read_i32()?;
        self.max_frame = cur.read_i32()?;
        self.always_new = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_level_type_ChunkA0.bin`.
pub type ConfigLevelTypeTable = BinTable<i32, ConfigLevelTypeRow>;
/// Parses `config_level_type_ChunkA0.bin` into a [`ConfigLevelTypeTable`].
pub fn parse_config_level_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLevelTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_task_formal_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTaskFormalRow {
    pub level_id: Box<[i32]>,
    pub title: Box<str>,
    pub subtitle: Box<str>,
    pub des: Box<str>,
    pub circle_pos: Box<[Box<str>]>,
    pub create_portal: Box<str>,
    pub create_monster: Box<str>,
    pub args: Box<[FConfigStrArrayI32]>,
    pub client_extend_anime: Box<[i32]>,
    pub client_extend_dialog: Box<[FConfigI32I32]>,
    pub unlock_portal: Box<str>,
    pub aft_npc: Box<str>,
    pub next_id: Box<[i32]>,
    pub subtitle_guide: Box<str>,
    pub waypoint: Box<[Box<str>]>,
    pub gameplay: Box<str>,
    pub season_ids: Box<[i32]>,
    pub id: i32,
    pub weights: i32,
    pub r#type: i32,
    pub bar_weights: i32,
    pub follow: i32,
    pub event_id: i32,
    pub override_color: i32,
    pub task_chain_id: i32,
    pub task_data_level: i32,
}
impl ConfigTaskFormalRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.weights = cur.read_i32()?;
        self.level_id = cur.read_array_i32()?;
        self.title = cur.read_str()?;
        self.subtitle = cur.read_str()?;
        self.des = cur.read_str()?;
        self.r#type = cur.read_i32()?;
        self.bar_weights = cur.read_i32()?;
        self.follow = cur.read_i32()?;
        self.circle_pos = cur.read_array_str()?;
        self.create_portal = cur.read_str()?;
        self.create_monster = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.args = FConfigStrArrayI32::read_array(cur)?;
        self.client_extend_anime = cur.read_array_i32()?;
        self.client_extend_dialog = FConfigI32I32::read_array(cur)?;
        self.unlock_portal = cur.read_str()?;
        self.aft_npc = cur.read_str()?;
        self.next_id = cur.read_array_i32()?;
        self.subtitle_guide = cur.read_str()?;
        self.waypoint = cur.read_array_str()?;
        self.event_id = cur.read_i32()?;
        self.gameplay = cur.read_str()?;
        self.season_ids = cur.read_array_i32()?;
        self.override_color = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.task_chain_id = cur.read_i32()?;
        self.task_data_level = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigTaskFormalRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_task_formal_ChunkA0.bin`.
pub type ConfigTaskFormalTable = BinTable<i32, ConfigTaskFormalRow>;
/// Parses `config_task_formal_ChunkA0.bin` into a [`ConfigTaskFormalTable`].
pub fn parse_config_task_formal(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTaskFormalTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
#[inline(never)]
fn verify_level_chunk_0(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_Level_loading_pic" => {
            Some(parse_config_level_loading_pic(buf, scratch).map(|t| t.len()))
        }
        "config_chapter" => Some(parse_config_chapter(buf, scratch).map(|t| t.len())),
        "config_guide_condition" => {
            Some(parse_config_guide_condition(buf, scratch).map(|t| t.len()))
        }
        "config_guide_config" => {
            Some(parse_config_guide_config(buf, scratch).map(|t| t.len()))
        }
        "config_guide_item" => {
            Some(parse_config_guide_item(buf, scratch).map(|t| t.len()))
        }
        "config_guide_parameter" => {
            Some(parse_config_guide_parameter(buf, scratch).map(|t| t.len()))
        }
        "config_level" => Some(parse_config_level(buf, scratch).map(|t| t.len())),
        "config_level_effect_ground" => {
            Some(parse_config_level_effect_ground(buf, scratch).map(|t| t.len()))
        }
        "config_level_effect_mpc" => {
            Some(parse_config_level_effect_mpc(buf, scratch).map(|t| t.len()))
        }
        "config_level_effect_param" => {
            Some(parse_config_level_effect_param(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_level_chunk_1(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_level_monster_catalog" => {
            Some(parse_config_level_monster_catalog(buf, scratch).map(|t| t.len()))
        }
        "config_level_monster_catalog_content" => {
            Some(
                parse_config_level_monster_catalog_content(buf, scratch).map(|t| t.len()),
            )
        }
        "config_level_monster_s5" => {
            Some(parse_config_level_monster_s5(buf, scratch).map(|t| t.len()))
        }
        "config_level_npc_info" => {
            Some(parse_config_level_npc_info(buf, scratch).map(|t| t.len()))
        }
        "config_level_outside_difficulty" => {
            Some(parse_config_level_outside_difficulty(buf, scratch).map(|t| t.len()))
        }
        "config_level_outside_modifier" => {
            Some(parse_config_level_outside_modifier(buf, scratch).map(|t| t.len()))
        }
        "config_level_outside_monster_catalog_content" => {
            Some(
                parse_config_level_outside_monster_catalog_content(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_level_overlap" => {
            Some(parse_config_level_overlap(buf, scratch).map(|t| t.len()))
        }
        "config_level_sequence_set" => {
            Some(parse_config_level_sequence_set(buf, scratch).map(|t| t.len()))
        }
        "config_level_sequence_subtitle" => {
            Some(parse_config_level_sequence_subtitle(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_level_chunk_2(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_level_skill_level" => {
            Some(parse_config_level_skill_level(buf, scratch).map(|t| t.len()))
        }
        "config_level_type" => {
            Some(parse_config_level_type(buf, scratch).map(|t| t.len()))
        }
        "config_task_formal" => {
            Some(parse_config_task_formal(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
pub(crate) fn verify_level_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    if let Some(res) = verify_level_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_level_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_level_chunk_2(stem, buf, scratch) {
        return Some(res);
    }
    None
}

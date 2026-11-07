//! Auto-generated `.bin` configuration table types and parsers for the `ui` domain.
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::FConfigStrI32;
use crate::bin::types::{BinTable, VecDim};
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_TextTable_GameFunc_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTextTableGameFuncRow {
    pub id: Box<str>,
    pub value: Box<str>,
}
impl BinRow for ConfigTextTableGameFuncRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_str()?;
        self.value = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_TextTable_GameFunc_ChunkA0.bin`.
pub type ConfigTextTableGameFuncTable = BinTable<Box<str>, ConfigTextTableGameFuncRow>;
/// Parses `config_TextTable_GameFunc_ChunkA0.bin` into a [`ConfigTextTableGameFuncTable`].
pub fn parse_config_text_table_game_func(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTextTableGameFuncTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_TextTable_NetCode_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTextTableNetCodeRow {
    pub id: Box<str>,
    pub value: Box<str>,
    pub desc: Box<str>,
    pub name: Box<str>,
    pub server_error: Box<str>,
    pub notice_type: i32,
    pub show_logout: bool,
}
impl BinRow for ConfigTextTableNetCodeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_str()?;
        self.value = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.name = cur.read_str()?;
        self.server_error = cur.read_str()?;
        self.show_logout = cur.read_bool()?;
        self.notice_type = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_TextTable_NetCode_ChunkA0.bin`.
pub type ConfigTextTableNetCodeTable = BinTable<Box<str>, ConfigTextTableNetCodeRow>;
/// Parses `config_TextTable_NetCode_ChunkA0.bin` into a [`ConfigTextTableNetCodeTable`].
pub fn parse_config_text_table_net_code(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTextTableNetCodeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_UI_Project_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigUIProjectRow {
    pub mesh_place: Box<str>,
    pub actor_location: Box<str>,
    pub actor_rotate: Box<str>,
    pub actor_zoom: Box<str>,
    pub vidicon_location: Box<str>,
    pub vidicon_rotate: Box<str>,
    pub vidicon_zoom: Box<str>,
    pub vidicon_distance: f64,
    pub actor_map_location: Box<str>,
    pub actor_map_rotate: Box<str>,
    pub actor_map_zoom: Box<str>,
    pub camera_rotation: Box<[i32]>,
    pub target_gamma: f64,
    pub id: i32,
    pub type1: i32,
    pub type2: i32,
    pub rt_size: i32,
    pub post_process_rt: bool,
}
impl ConfigUIProjectRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.mesh_place = cur.read_str()?;
        self.actor_location = cur.read_str()?;
        self.actor_rotate = cur.read_str()?;
        self.actor_zoom = cur.read_str()?;
        self.vidicon_location = cur.read_str()?;
        self.vidicon_rotate = cur.read_str()?;
        self.vidicon_zoom = cur.read_str()?;
        self.vidicon_distance = cur.read_f64()?;
        self.actor_map_location = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.actor_map_rotate = cur.read_str()?;
        self.actor_map_zoom = cur.read_str()?;
        self.rt_size = cur.read_i32()?;
        self.camera_rotation = cur.read_array_i32()?;
        self.target_gamma = cur.read_f64()?;
        self.post_process_rt = cur.read_bool()?;
        Ok(())
    }
}
impl BinRow for ConfigUIProjectRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_UI_Project_ChunkA0.bin`.
pub type ConfigUIProjectTable = BinTable<i32, ConfigUIProjectRow>;
/// Parses `config_UI_Project_ChunkA0.bin` into a [`ConfigUIProjectTable`].
pub fn parse_config_ui_project(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUIProjectTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gamepad_forcefeedback_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGamepadForcefeedbackRow {
    pub name: Box<str>,
    pub id: i32,
    pub r#type: i32,
    pub order: i32,
}
impl BinRow for ConfigGamepadForcefeedbackRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.order = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gamepad_forcefeedback_ChunkA0.bin`.
pub type ConfigGamepadForcefeedbackTable = BinTable<i32, ConfigGamepadForcefeedbackRow>;
/// Parses `config_gamepad_forcefeedback_ChunkA0.bin` into a [`ConfigGamepadForcefeedbackTable`].
pub fn parse_config_gamepad_forcefeedback(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGamepadForcefeedbackTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_language_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLanguageRow {
    pub language_code: Box<str>,
    pub font_style: Box<str>,
    pub name: Box<str>,
    pub voice_name: Box<str>,
    pub main_font: Box<str>,
    pub ui2_bold_font: Box<str>,
    pub ui2_light_font: Box<str>,
    pub fix_font_size: FxHashMap<i32, i32>,
    pub language_id: i32,
    pub is_voice: i32,
    pub show_order: i32,
    pub min_font_size: i32,
    pub sub_font_size: i32,
    pub lineheight_per: i32,
}
impl ConfigLanguageRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.language_code = cur.read_str()?;
        self.font_style = cur.read_str()?;
        self.language_id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.is_voice = cur.read_i32()?;
        self.voice_name = cur.read_str()?;
        self.show_order = cur.read_i32()?;
        self.main_font = cur.read_str()?;
        self.min_font_size = cur.read_i32()?;
        self.sub_font_size = cur.read_i32()?;
        self.lineheight_per = cur.read_i32()?;
        self.ui2_bold_font = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.ui2_light_font = cur.read_str()?;
        self.fix_font_size = cur.read_map_i32_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigLanguageRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_language_ChunkA0.bin`.
pub type ConfigLanguageTable = BinTable<Box<str>, ConfigLanguageRow>;
/// Parses `config_language_ChunkA0.bin` into a [`ConfigLanguageTable`].
pub fn parse_config_language(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigLanguageTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_team_bonus_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigTeamBonusRow {
    pub exp_incress: f64,
    pub drop_incress: f64,
    pub monster_more_hp: f64,
    pub monster_more_damage: f64,
    pub id: i32,
}
impl BinRow for ConfigTeamBonusRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.exp_incress = cur.read_f64()?;
        self.drop_incress = cur.read_f64()?;
        self.monster_more_hp = cur.read_f64()?;
        self.monster_more_damage = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_team_bonus_ChunkA0.bin`.
pub type ConfigTeamBonusTable = BinTable<i32, ConfigTeamBonusRow>;
/// Parses `config_team_bonus_ChunkA0.bin` into a [`ConfigTeamBonusTable`].
pub fn parse_config_team_bonus(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTeamBonusTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_team_bonus_range_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTeamBonusRangeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
    pub limit_range: i32,
}
impl BinRow for ConfigTeamBonusRangeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.limit_range = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_team_bonus_range_ChunkA0.bin`.
pub type ConfigTeamBonusRangeTable = BinTable<i32, ConfigTeamBonusRangeRow>;
/// Parses `config_team_bonus_range_ChunkA0.bin` into a [`ConfigTeamBonusRangeTable`].
pub fn parse_config_team_bonus_range(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigTeamBonusRangeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_animation_curve_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiAnimationCurveRow {
    pub id: Box<str>,
    pub value: Box<str>,
}
impl BinRow for ConfigUiAnimationCurveRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_str()?;
        self.value = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_animation_curve_ChunkA0.bin`.
pub type ConfigUiAnimationCurveTable = BinTable<Box<str>, ConfigUiAnimationCurveRow>;
/// Parses `config_ui_animation_curve_ChunkA0.bin` into a [`ConfigUiAnimationCurveTable`].
pub fn parse_config_ui_animation_curve(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiAnimationCurveTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_button_style_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigUiButtonStyleRow {
    pub text_padding: Box<[f64]>,
    pub text_alignment: VecDim,
    pub icon_alignment: VecDim,
    pub icon_pos: VecDim,
    pub icon_padding: Box<[f64]>,
    pub normal_image: Box<str>,
    pub segmentation_image: Box<[f64]>,
    pub shading_image: Box<str>,
    pub shading_alignment: VecDim,
    pub shading_pos: VecDim,
    pub shading_size: VecDim,
    pub fix_scale: f64,
    pub id: i32,
    pub width: i32,
    pub height: i32,
    pub text_style_id: i32,
    pub text_decoration: bool,
}
impl ConfigUiButtonStyleRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.width = cur.read_i32()?;
        self.height = cur.read_i32()?;
        self.text_style_id = cur.read_i32()?;
        self.text_padding = cur.read_array_f64()?;
        self.text_alignment = cur.read_vec_dim()?;
        self.icon_alignment = cur.read_vec_dim()?;
        self.icon_pos = cur.read_vec_dim()?;
        self.icon_padding = cur.read_array_f64()?;
        self.normal_image = cur.read_str()?;
        self.segmentation_image = cur.read_array_f64()?;
        self.shading_image = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.shading_alignment = cur.read_vec_dim()?;
        self.shading_pos = cur.read_vec_dim()?;
        self.shading_size = cur.read_vec_dim()?;
        self.text_decoration = cur.read_bool()?;
        self.fix_scale = cur.read_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigUiButtonStyleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_button_style_ChunkA0.bin`.
pub type ConfigUiButtonStyleTable = BinTable<i32, ConfigUiButtonStyleRow>;
/// Parses `config_ui_button_style_ChunkA0.bin` into a [`ConfigUiButtonStyleTable`].
pub fn parse_config_ui_button_style(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiButtonStyleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_color_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiColorRow {
    pub id: i32,
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub a: i32,
}
impl BinRow for ConfigUiColorRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r = cur.read_i32()?;
        self.g = cur.read_i32()?;
        self.b = cur.read_i32()?;
        self.a = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_color_ChunkA0.bin`.
pub type ConfigUiColorTable = BinTable<i32, ConfigUiColorRow>;
/// Parses `config_ui_color_ChunkA0.bin` into a [`ConfigUiColorTable`].
pub fn parse_config_ui_color(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiColorTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_cursor_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiCursorRow {
    pub name: Box<str>,
    pub hotspot: Box<str>,
    pub name_mid: Box<str>,
    pub hotspot_mid: Box<str>,
    pub name_high: Box<str>,
    pub hotspot_high: Box<str>,
    pub name_ex: Box<str>,
    pub hotspot_ex: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigUiCursorRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.hotspot = cur.read_str()?;
        self.name_mid = cur.read_str()?;
        self.hotspot_mid = cur.read_str()?;
        self.name_high = cur.read_str()?;
        self.hotspot_high = cur.read_str()?;
        self.name_ex = cur.read_str()?;
        self.hotspot_ex = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_cursor_ChunkA0.bin`.
pub type ConfigUiCursorTable = BinTable<i32, ConfigUiCursorRow>;
/// Parses `config_ui_cursor_ChunkA0.bin` into a [`ConfigUiCursorTable`].
pub fn parse_config_ui_cursor(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiCursorTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_emotion_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiEmotionRow {
    pub resource: Box<str>,
    pub id: i32,
    pub width: i32,
    pub height: i32,
}
impl BinRow for ConfigUiEmotionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.resource = cur.read_str()?;
        self.width = cur.read_i32()?;
        self.height = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_emotion_ChunkA0.bin`.
pub type ConfigUiEmotionTable = BinTable<i32, ConfigUiEmotionRow>;
/// Parses `config_ui_emotion_ChunkA0.bin` into a [`ConfigUiEmotionTable`].
pub fn parse_config_ui_emotion(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiEmotionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_image_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiImageRow {
    pub id: Box<str>,
    pub value: Box<str>,
    pub season: Box<str>,
    pub col_3: Box<str>,
}
impl BinRow for ConfigUiImageRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_str()?;
        self.value = cur.read_str()?;
        self.season = cur.read_str()?;
        self.col_3 = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_image_ChunkA0.bin`.
pub type ConfigUiImageTable = BinTable<Box<str>, ConfigUiImageRow>;
/// Parses `config_ui_image_ChunkA0.bin` into a [`ConfigUiImageTable`].
pub fn parse_config_ui_image(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiImageTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_imageV2_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiImageV2Row {
    pub id: Box<str>,
    pub value: Box<str>,
    pub season: Box<str>,
    pub platform: Box<str>,
}
impl BinRow for ConfigUiImageV2Row {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_str()?;
        self.value = cur.read_str()?;
        self.season = cur.read_str()?;
        self.platform = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_imageV2_ChunkA0.bin`.
pub type ConfigUiImageV2Table = BinTable<Box<str>, ConfigUiImageV2Row>;
/// Parses `config_ui_imageV2_ChunkA0.bin` into a [`ConfigUiImageV2Table`].
pub fn parse_config_ui_image_v2(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiImageV2Table, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_image_atlas_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiImageAtlasRow {
    pub id: Box<str>,
    pub value: Box<str>,
    pub season: Box<str>,
    pub col_3: Box<str>,
}
impl BinRow for ConfigUiImageAtlasRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_str()?;
        self.value = cur.read_str()?;
        self.season = cur.read_str()?;
        self.col_3 = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_image_atlas_ChunkA0.bin`.
pub type ConfigUiImageAtlasTable = BinTable<Box<str>, ConfigUiImageAtlasRow>;
/// Parses `config_ui_image_atlas_ChunkA0.bin` into a [`ConfigUiImageAtlasTable`].
pub fn parse_config_ui_image_atlas(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiImageAtlasTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_item_quality_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiItemQualityRow {
    pub icon: Box<str>,
    pub id: i32,
    pub style_pick: i32,
    pub fx_type: i32,
    pub color: i32,
}
impl BinRow for ConfigUiItemQualityRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.style_pick = cur.read_i32()?;
        self.fx_type = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.color = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_item_quality_ChunkA0.bin`.
pub type ConfigUiItemQualityTable = BinTable<i32, ConfigUiItemQualityRow>;
/// Parses `config_ui_item_quality_ChunkA0.bin` into a [`ConfigUiItemQualityTable`].
pub fn parse_config_ui_item_quality(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiItemQualityTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_offset_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigUiOffsetRow {
    pub offset: Box<str>,
    pub fix_scale: f64,
    pub id: i32,
}
impl BinRow for ConfigUiOffsetRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.offset = cur.read_str()?;
        self.fix_scale = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_offset_ChunkA0.bin`.
pub type ConfigUiOffsetTable = BinTable<i32, ConfigUiOffsetRow>;
/// Parses `config_ui_offset_ChunkA0.bin` into a [`ConfigUiOffsetTable`].
pub fn parse_config_ui_offset(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiOffsetTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_png9_style_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigUiPng9StyleRow {
    pub image: Box<str>,
    pub segmentation_image: Box<[f64]>,
    pub fix_scale: f64,
    pub id: i32,
    pub width: i32,
    pub height: i32,
}
impl BinRow for ConfigUiPng9StyleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.width = cur.read_i32()?;
        self.height = cur.read_i32()?;
        self.image = cur.read_str()?;
        self.segmentation_image = cur.read_array_f64()?;
        self.fix_scale = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_png9_style_ChunkA0.bin`.
pub type ConfigUiPng9StyleTable = BinTable<i32, ConfigUiPng9StyleRow>;
/// Parses `config_ui_png9_style_ChunkA0.bin` into a [`ConfigUiPng9StyleTable`].
pub fn parse_config_ui_png9_style(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiPng9StyleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_safe_zone_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiSafeZoneRow {
    pub device_type: Box<str>,
    pub left_padding: i32,
    pub up_padding: i32,
    pub right_padding: i32,
    pub down_padding: i32,
}
impl BinRow for ConfigUiSafeZoneRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.device_type = cur.read_str()?;
        self.left_padding = cur.read_i32()?;
        self.up_padding = cur.read_i32()?;
        self.right_padding = cur.read_i32()?;
        self.down_padding = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_safe_zone_ChunkA0.bin`.
pub type ConfigUiSafeZoneTable = BinTable<Box<str>, ConfigUiSafeZoneRow>;
/// Parses `config_ui_safe_zone_ChunkA0.bin` into a [`ConfigUiSafeZoneTable`].
pub fn parse_config_ui_safe_zone(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiSafeZoneTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_scale_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigUiScaleRow {
    pub scale: f64,
    pub id: i32,
}
impl BinRow for ConfigUiScaleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.scale = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_scale_ChunkA0.bin`.
pub type ConfigUiScaleTable = BinTable<i32, ConfigUiScaleRow>;
/// Parses `config_ui_scale_ChunkA0.bin` into a [`ConfigUiScaleTable`].
pub fn parse_config_ui_scale(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiScaleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_showcase_animset_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiShowcaseAnimsetRow {
    pub anim_play_count: Box<[FConfigStrI32]>,
    pub sub_anim_set: FxHashMap<i32, Box<[i32]>>,
    pub sub_skeletal_mesh: Box<str>,
    pub sub_mesh_socket: Box<str>,
    pub anim_blueprint: Box<str>,
    pub id: i32,
    pub restart_index: i32,
}
impl BinRow for ConfigUiShowcaseAnimsetRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.anim_play_count = FConfigStrI32::read_array(cur)?;
        self.restart_index = cur.read_i32()?;
        self.sub_anim_set = cur.read_map_i32_array_i32()?;
        self.sub_skeletal_mesh = cur.read_str()?;
        self.sub_mesh_socket = cur.read_str()?;
        self.anim_blueprint = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_showcase_animset_ChunkA0.bin`.
pub type ConfigUiShowcaseAnimsetTable = BinTable<i32, ConfigUiShowcaseAnimsetRow>;
/// Parses `config_ui_showcase_animset_ChunkA0.bin` into a [`ConfigUiShowcaseAnimsetTable`].
pub fn parse_config_ui_showcase_animset(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiShowcaseAnimsetTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_style_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiStyleRow {
    pub font_name: Box<str>,
    pub typeface: Box<str>,
    pub font_mat: Box<str>,
    pub id: i32,
    pub font_size: i32,
    pub font_color: i32,
    pub outline_size: i32,
    pub outline_color: i32,
    pub shadow_size: i32,
    pub shadow_color: i32,
    pub shadow_angle: i32,
    pub italic_angle: i32,
    pub letter_spacing: i32,
    pub rtf_id: i32,
}
impl ConfigUiStyleRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.font_name = cur.read_str()?;
        self.typeface = cur.read_str()?;
        self.font_size = cur.read_i32()?;
        self.font_color = cur.read_i32()?;
        self.font_mat = cur.read_str()?;
        self.outline_size = cur.read_i32()?;
        self.outline_color = cur.read_i32()?;
        self.shadow_size = cur.read_i32()?;
        self.shadow_color = cur.read_i32()?;
        self.shadow_angle = cur.read_i32()?;
        self.italic_angle = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.letter_spacing = cur.read_i32()?;
        self.rtf_id = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigUiStyleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_style_ChunkA0.bin`.
pub type ConfigUiStyleTable = BinTable<i32, ConfigUiStyleRow>;
/// Parses `config_ui_style_ChunkA0.bin` into a [`ConfigUiStyleTable`].
pub fn parse_config_ui_style(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiStyleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_ui_style_v2_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiStyleV2Row {
    pub font_name: Box<str>,
    pub typeface: Box<str>,
    pub font_mat: Box<str>,
    pub id: i32,
    pub font_size: i32,
    pub font_color: i32,
    pub outline_size: i32,
    pub outline_color: i32,
    pub shadow_size: i32,
    pub shadow_color: i32,
    pub shadow_angle: i32,
    pub italic_angle: i32,
    pub letter_spacing: i32,
    pub rtf_id: i32,
}
impl ConfigUiStyleV2Row {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.font_name = cur.read_str()?;
        self.typeface = cur.read_str()?;
        self.font_size = cur.read_i32()?;
        self.font_color = cur.read_i32()?;
        self.font_mat = cur.read_str()?;
        self.outline_size = cur.read_i32()?;
        self.outline_color = cur.read_i32()?;
        self.shadow_size = cur.read_i32()?;
        self.shadow_color = cur.read_i32()?;
        self.shadow_angle = cur.read_i32()?;
        self.italic_angle = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.letter_spacing = cur.read_i32()?;
        self.rtf_id = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigUiStyleV2Row {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_ui_style_v2_ChunkA0.bin`.
pub type ConfigUiStyleV2Table = BinTable<i32, ConfigUiStyleV2Row>;
/// Parses `config_ui_style_v2_ChunkA0.bin` into a [`ConfigUiStyleV2Table`].
pub fn parse_config_ui_style_v2(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigUiStyleV2Table, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
#[inline(never)]
fn verify_ui_chunk_0(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_TextTable_GameFunc" => {
            Some(parse_config_text_table_game_func(buf, scratch).map(|t| t.len()))
        }
        "config_TextTable_NetCode" => {
            Some(parse_config_text_table_net_code(buf, scratch).map(|t| t.len()))
        }
        "config_UI_Project" => {
            Some(parse_config_ui_project(buf, scratch).map(|t| t.len()))
        }
        "config_gamepad_forcefeedback" => {
            Some(parse_config_gamepad_forcefeedback(buf, scratch).map(|t| t.len()))
        }
        "config_language" => Some(parse_config_language(buf, scratch).map(|t| t.len())),
        "config_team_bonus" => {
            Some(parse_config_team_bonus(buf, scratch).map(|t| t.len()))
        }
        "config_team_bonus_range" => {
            Some(parse_config_team_bonus_range(buf, scratch).map(|t| t.len()))
        }
        "config_ui_animation_curve" => {
            Some(parse_config_ui_animation_curve(buf, scratch).map(|t| t.len()))
        }
        "config_ui_button_style" => {
            Some(parse_config_ui_button_style(buf, scratch).map(|t| t.len()))
        }
        "config_ui_color" => Some(parse_config_ui_color(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}
#[inline(never)]
fn verify_ui_chunk_1(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_ui_cursor" => Some(parse_config_ui_cursor(buf, scratch).map(|t| t.len())),
        "config_ui_emotion" => {
            Some(parse_config_ui_emotion(buf, scratch).map(|t| t.len()))
        }
        "config_ui_image" => Some(parse_config_ui_image(buf, scratch).map(|t| t.len())),
        "config_ui_imageV2" => {
            Some(parse_config_ui_image_v2(buf, scratch).map(|t| t.len()))
        }
        "config_ui_image_atlas" => {
            Some(parse_config_ui_image_atlas(buf, scratch).map(|t| t.len()))
        }
        "config_ui_item_quality" => {
            Some(parse_config_ui_item_quality(buf, scratch).map(|t| t.len()))
        }
        "config_ui_offset" => Some(parse_config_ui_offset(buf, scratch).map(|t| t.len())),
        "config_ui_png9_style" => {
            Some(parse_config_ui_png9_style(buf, scratch).map(|t| t.len()))
        }
        "config_ui_safe_zone" => {
            Some(parse_config_ui_safe_zone(buf, scratch).map(|t| t.len()))
        }
        "config_ui_scale" => Some(parse_config_ui_scale(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}
#[inline(never)]
fn verify_ui_chunk_2(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_ui_showcase_animset" => {
            Some(parse_config_ui_showcase_animset(buf, scratch).map(|t| t.len()))
        }
        "config_ui_style" => Some(parse_config_ui_style(buf, scratch).map(|t| t.len())),
        "config_ui_style_v2" => {
            Some(parse_config_ui_style_v2(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
pub(crate) fn verify_ui_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    if let Some(res) = verify_ui_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_ui_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_ui_chunk_2(stem, buf, scratch) {
        return Some(res);
    }
    None
}

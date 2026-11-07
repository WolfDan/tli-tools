//! Auto-generated `.bin` configuration table types and parsers for the `sp` domain.
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{FConfigI32ArrayI32, FConfigI32I32};
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_sp_area_lvl_v2_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigSpAreaLvlV2Row {
    pub name: Box<str>,
    pub eye_num_normal: Box<[i32]>,
    pub eye_num_retry: Box<[i32]>,
    pub boss_title_prefix: Box<str>,
    pub boss_title: Box<str>,
    pub boss_name: Box<str>,
    pub comment: Box<str>,
    pub affix_num: Box<[i32]>,
    pub prefix: Box<str>,
    pub suffix: Box<str>,
    pub refresh_cost: Box<[i32]>,
    pub lvl_intro_txt: Box<str>,
    pub lvl_unlock_txt: Box<str>,
    pub lvl_up_txt_upgrade: Box<str>,
    pub lvl_up_txt_still: Box<str>,
    pub map_affix_refresh_cost_item: Box<[FConfigI32ArrayI32]>,
    pub map_affix_refresh_cost_num: Box<[FConfigI32ArrayI32]>,
    pub map_affix_add_cost_item: Box<[FConfigI32ArrayI32]>,
    pub map_affix_add_cost_num: Box<[FConfigI32ArrayI32]>,
    pub map_affix_base: Box<[i32]>,
    pub map_outside_modifier_group: Box<[FConfigI32I32]>,
    pub unlock_req: Box<[FConfigI32I32]>,
    pub talentpoint_chance: f64,
    pub map_affix_base_weight: Box<[FConfigI32I32]>,
    pub random_modifier_number: Box<[FConfigI32I32]>,
    pub map_default_mod_random: Box<[i32]>,
    pub map_affix_refresh_cost_item2: Box<[FConfigI32ArrayI32]>,
    pub map_affix_refresh_cost_num2: Box<[FConfigI32ArrayI32]>,
    pub map_affix_add_cost_item2: Box<[FConfigI32ArrayI32]>,
    pub map_affix_add_cost_num2: Box<[FConfigI32ArrayI32]>,
    pub id: i32,
    pub area_id: i32,
    pub area_lvl: i32,
    pub prefix_num: i32,
    pub affix_lvl: i32,
    pub lvl_display: i32,
    pub map_affix_init_num: i32,
    pub map_affix_max_num: i32,
    pub map_affix_refresh_limit: i32,
    pub map_affix_refresh_lock: i32,
    pub map_affix_add_lock: i32,
    pub spmode_unlock_level: i32,
    pub phase: i32,
    pub map_affix_refresh_limit2: i32,
    pub enable: bool,
}
impl ConfigSpAreaLvlV2Row {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.area_id = cur.read_i32()?;
        self.area_lvl = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.eye_num_normal = cur.read_array_i32()?;
        self.eye_num_retry = cur.read_array_i32()?;
        self.boss_title_prefix = cur.read_str()?;
        self.boss_title = cur.read_str()?;
        self.boss_name = cur.read_str()?;
        self.comment = cur.read_str()?;
        self.affix_num = cur.read_array_i32()?;
        self.prefix_num = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.prefix = cur.read_str()?;
        self.suffix = cur.read_str()?;
        self.refresh_cost = cur.read_array_i32()?;
        self.affix_lvl = cur.read_i32()?;
        self.lvl_intro_txt = cur.read_str()?;
        self.lvl_unlock_txt = cur.read_str()?;
        self.lvl_up_txt_upgrade = cur.read_str()?;
        self.lvl_up_txt_still = cur.read_str()?;
        self.lvl_display = cur.read_i32()?;
        self.map_affix_init_num = cur.read_i32()?;
        self.map_affix_max_num = cur.read_i32()?;
        self.map_affix_refresh_cost_item = FConfigI32ArrayI32::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.map_affix_refresh_cost_num = FConfigI32ArrayI32::read_array(cur)?;
        self.map_affix_add_cost_item = FConfigI32ArrayI32::read_array(cur)?;
        self.map_affix_add_cost_num = FConfigI32ArrayI32::read_array(cur)?;
        self.map_affix_refresh_limit = cur.read_i32()?;
        self.map_affix_refresh_lock = cur.read_i32()?;
        self.map_affix_add_lock = cur.read_i32()?;
        self.spmode_unlock_level = cur.read_i32()?;
        self.map_affix_base = cur.read_array_i32()?;
        self.map_outside_modifier_group = FConfigI32I32::read_array(cur)?;
        self.enable = cur.read_bool()?;
        self.unlock_req = FConfigI32I32::read_array(cur)?;
        self.talentpoint_chance = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.phase = cur.read_i32()?;
        self.map_affix_base_weight = FConfigI32I32::read_array(cur)?;
        self.random_modifier_number = FConfigI32I32::read_array(cur)?;
        self.map_default_mod_random = cur.read_array_i32()?;
        self.map_affix_refresh_limit2 = cur.read_i32()?;
        self.map_affix_refresh_cost_item2 = FConfigI32ArrayI32::read_array(cur)?;
        self.map_affix_refresh_cost_num2 = FConfigI32ArrayI32::read_array(cur)?;
        self.map_affix_add_cost_item2 = FConfigI32ArrayI32::read_array(cur)?;
        self.map_affix_add_cost_num2 = FConfigI32ArrayI32::read_array(cur)?;
        Ok(())
    }
}
impl BinRow for ConfigSpAreaLvlV2Row {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_sp_area_lvl_v2_ChunkA0.bin`.
pub type ConfigSpAreaLvlV2Table = BinTable<i32, ConfigSpAreaLvlV2Row>;
/// Parses `config_sp_area_lvl_v2_ChunkA0.bin` into a [`ConfigSpAreaLvlV2Table`].
pub fn parse_config_sp_area_lvl_v2(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSpAreaLvlV2Table, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_sp_map_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigSpMapRow {
    pub unlock_level: Box<[FConfigI32I32]>,
    pub is_unlock: f64,
    pub item_base_id: Box<[i32]>,
    pub refresh_cost: Box<[i32]>,
    pub icon: Box<str>,
    pub place: Box<[i32]>,
    pub block_ui: Box<[i32]>,
    pub npc_tool_des: Box<str>,
    pub id: i32,
    pub level_id: i32,
    pub map_type: i32,
    pub world_level: i32,
    pub aera_id: i32,
    pub process: i32,
}
impl ConfigSpMapRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.level_id = cur.read_i32()?;
        self.map_type = cur.read_i32()?;
        self.world_level = cur.read_i32()?;
        self.aera_id = cur.read_i32()?;
        self.process = cur.read_i32()?;
        self.unlock_level = FConfigI32I32::read_array(cur)?;
        self.is_unlock = cur.read_f64()?;
        self.item_base_id = cur.read_array_i32()?;
        self.refresh_cost = cur.read_array_i32()?;
        self.icon = cur.read_str()?;
        self.place = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.block_ui = cur.read_array_i32()?;
        self.npc_tool_des = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigSpMapRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_sp_map_ChunkA0.bin`.
pub type ConfigSpMapTable = BinTable<i32, ConfigSpMapRow>;
/// Parses `config_sp_map_ChunkA0.bin` into a [`ConfigSpMapTable`].
pub fn parse_config_sp_map(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSpMapTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_sp_map_v2_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpMapV2Row {
    pub prism_socket: Box<[i32]>,
    pub icon: Box<str>,
    pub place: Box<[i32]>,
    pub block_ui: Box<[i32]>,
    pub classification: Box<[i32]>,
    pub special_drop: Box<[i32]>,
    pub prism_sockets: FxHashMap<i32, Box<[i32]>>,
    pub prism2_sockets: FxHashMap<i32, Box<[i32]>>,
    pub prism3_sockets: FxHashMap<i32, Box<[i32]>>,
    pub illustration: Box<str>,
    pub desc: Box<str>,
    pub mechanic_desc: Box<str>,
    pub id: i32,
    pub area_lv: i32,
    pub map_type: i32,
    pub area_id: i32,
    pub group: i32,
    pub boss_id: i32,
    pub level_id: i32,
    pub level_id_repeat: i32,
    pub item_base_id: i32,
    pub item_num: i32,
    pub item_num_repeat: i32,
    pub boss_progress: i32,
    pub boss_progress_repeat: i32,
    pub flame1_drop: i32,
    pub extra_lv: i32,
    pub gameplay: i32,
    pub unknown_id: i32,
    pub enable: i32,
    pub talentpoint_normal: i32,
    pub talentpoint_core: i32,
    pub boss_progress_item: i32,
    pub money_id: i32,
    pub area_boss_difficulty: i32,
}
impl ConfigSpMapV2Row {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.area_lv = cur.read_i32()?;
        self.map_type = cur.read_i32()?;
        self.area_id = cur.read_i32()?;
        self.group = cur.read_i32()?;
        self.boss_id = cur.read_i32()?;
        self.level_id = cur.read_i32()?;
        self.level_id_repeat = cur.read_i32()?;
        self.item_base_id = cur.read_i32()?;
        self.item_num = cur.read_i32()?;
        self.item_num_repeat = cur.read_i32()?;
        self.boss_progress = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.boss_progress_repeat = cur.read_i32()?;
        self.flame1_drop = cur.read_i32()?;
        self.extra_lv = cur.read_i32()?;
        self.gameplay = cur.read_i32()?;
        self.prism_socket = cur.read_array_i32()?;
        self.icon = cur.read_str()?;
        self.place = cur.read_array_i32()?;
        self.block_ui = cur.read_array_i32()?;
        self.classification = cur.read_array_i32()?;
        self.unknown_id = cur.read_i32()?;
        self.special_drop = cur.read_array_i32()?;
        self.enable = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.prism_sockets = cur.read_map_i32_array_i32()?;
        self.prism2_sockets = cur.read_map_i32_array_i32()?;
        self.prism3_sockets = cur.read_map_i32_array_i32()?;
        self.talentpoint_normal = cur.read_i32()?;
        self.talentpoint_core = cur.read_i32()?;
        self.boss_progress_item = cur.read_i32()?;
        self.illustration = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.mechanic_desc = cur.read_str()?;
        self.money_id = cur.read_i32()?;
        self.area_boss_difficulty = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigSpMapV2Row {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_sp_map_v2_ChunkA0.bin`.
pub type ConfigSpMapV2Table = BinTable<i32, ConfigSpMapV2Row>;
/// Parses `config_sp_map_v2_ChunkA0.bin` into a [`ConfigSpMapV2Table`].
pub fn parse_config_sp_map_v2(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSpMapV2Table, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_sp_map_v2_level_index_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpMapV2LevelIndexRow {
    pub area_lvs: Box<[i32]>,
    pub level_id: i32,
}
impl BinRow for ConfigSpMapV2LevelIndexRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.level_id = cur.read_i32()?;
        self.area_lvs = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_sp_map_v2_level_index_ChunkA0.bin`.
pub type ConfigSpMapV2LevelIndexTable = BinTable<i32, ConfigSpMapV2LevelIndexRow>;
/// Parses `config_sp_map_v2_level_index_ChunkA0.bin` into a [`ConfigSpMapV2LevelIndexTable`].
pub fn parse_config_sp_map_v2_level_index(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSpMapV2LevelIndexTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_sp_rank_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpRankRow {
    pub major_rank_name: Box<str>,
    pub minor_rank_name: Box<str>,
    pub mode_lv: Box<str>,
    pub mode_dps: Box<str>,
    pub mode_survival: Box<str>,
    pub mode_name: Box<str>,
    pub mode_buff: Box<str>,
    pub mode_unlock_req_text: Box<str>,
    pub random_modifier_num: Box<str>,
    pub compass_num: Box<str>,
    pub item_quantity: Box<str>,
    pub area_boss_drop_rate: Box<str>,
    pub new_drop: Box<[i32]>,
    pub id: i32,
    pub major_rank: i32,
    pub minor_rank: i32,
    pub mode: i32,
    pub unlock_req_area: i32,
    pub unlock_req_boss: i32,
}
impl ConfigSpRankRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.major_rank = cur.read_i32()?;
        self.minor_rank = cur.read_i32()?;
        self.mode = cur.read_i32()?;
        self.major_rank_name = cur.read_str()?;
        self.minor_rank_name = cur.read_str()?;
        self.mode_lv = cur.read_str()?;
        self.mode_dps = cur.read_str()?;
        self.mode_survival = cur.read_str()?;
        self.mode_name = cur.read_str()?;
        self.mode_buff = cur.read_str()?;
        self.mode_unlock_req_text = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.unlock_req_area = cur.read_i32()?;
        self.unlock_req_boss = cur.read_i32()?;
        self.random_modifier_num = cur.read_str()?;
        self.compass_num = cur.read_str()?;
        self.item_quantity = cur.read_str()?;
        self.area_boss_drop_rate = cur.read_str()?;
        self.new_drop = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigSpRankRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_sp_rank_ChunkA0.bin`.
pub type ConfigSpRankTable = BinTable<i32, ConfigSpRankRow>;
/// Parses `config_sp_rank_ChunkA0.bin` into a [`ConfigSpRankTable`].
pub fn parse_config_sp_rank(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigSpRankTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_sp_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_sp_area_lvl_v2" => {
            Some(parse_config_sp_area_lvl_v2(buf, scratch).map(|t| t.len()))
        }
        "config_sp_map" => Some(parse_config_sp_map(buf, scratch).map(|t| t.len())),
        "config_sp_map_v2" => Some(parse_config_sp_map_v2(buf, scratch).map(|t| t.len())),
        "config_sp_map_v2_level_index" => {
            Some(parse_config_sp_map_v2_level_index(buf, scratch).map(|t| t.len()))
        }
        "config_sp_rank" => Some(parse_config_sp_rank(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

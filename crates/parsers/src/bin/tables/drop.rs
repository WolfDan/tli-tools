//! Auto-generated `.bin` configuration table types and parsers for the `drop` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::FConfigI32I32;
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_drop_audio_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropAudioRow {
    pub desc: Box<str>,
    pub audio_path: Box<str>,
    pub switch_key: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigDropAudioRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.audio_path = cur.read_str()?;
        self.switch_key = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_audio_ChunkA0.bin`.
pub type ConfigDropAudioTable = BinTable<i32, ConfigDropAudioRow>;
/// Parses `config_drop_audio_ChunkA0.bin` into a [`ConfigDropAudioTable`].
pub fn parse_config_drop_audio(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropAudioTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_carrier_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigDropCarrierRow {
    pub name: Box<str>,
    pub drop_affix: Box<[FConfigAffix]>,
    pub id: i32,
    pub action_model: i32,
    pub range: i32,
    pub drop_id: i32,
    pub drop_limit: i32,
    pub enum_monster_source_type: i32,
    pub view_actor_id: i32,
    pub servant_scheme_type: i32,
}
impl BinRow for ConfigDropCarrierRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.action_model = cur.read_i32()?;
        self.range = cur.read_i32()?;
        self.drop_id = cur.read_i32()?;
        self.drop_limit = cur.read_i32()?;
        self.enum_monster_source_type = cur.read_i32()?;
        self.view_actor_id = cur.read_i32()?;
        self.drop_affix = FConfigAffix::read_array(cur)?;
        self.servant_scheme_type = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_carrier_ChunkA0.bin`.
pub type ConfigDropCarrierTable = BinTable<i32, ConfigDropCarrierRow>;
/// Parses `config_drop_carrier_ChunkA0.bin` into a [`ConfigDropCarrierTable`].
pub fn parse_config_drop_carrier(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropCarrierTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_filter_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropFilterRow {
    pub rarity: Box<[i32]>,
    pub drop_lv: Box<[i32]>,
    pub item_lv: Box<[i32]>,
    pub cost: Box<[i32]>,
    pub skill_lv: Box<[i32]>,
    pub talent_node: Box<[FConfigI32I32]>,
    pub mod_id: Box<[i32]>,
    pub talent_node_id: Box<[i32]>,
    pub id: i32,
    pub priority: i32,
    pub item_id: i32,
    pub item_gold_id: i32,
    pub item_gold_talen_id: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
    pub is_identified: i32,
    pub is_eroded: i32,
    pub style: i32,
    pub mod_tier: FConfigI32I32,
}
impl ConfigDropFilterRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.priority = cur.read_i32()?;
        self.item_id = cur.read_i32()?;
        self.item_gold_id = cur.read_i32()?;
        self.item_gold_talen_id = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        self.rarity = cur.read_array_i32()?;
        self.drop_lv = cur.read_array_i32()?;
        self.item_lv = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.cost = cur.read_array_i32()?;
        self.is_identified = cur.read_i32()?;
        self.is_eroded = cur.read_i32()?;
        self.skill_lv = cur.read_array_i32()?;
        self.style = cur.read_i32()?;
        self.mod_tier = FConfigI32I32::read(cur)?;
        self.talent_node = FConfigI32I32::read_array(cur)?;
        self.mod_id = cur.read_array_i32()?;
        self.talent_node_id = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigDropFilterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_filter_ChunkA0.bin`.
pub type ConfigDropFilterTable = BinTable<i32, ConfigDropFilterRow>;
/// Parses `config_drop_filter_ChunkA0.bin` into a [`ConfigDropFilterTable`].
pub fn parse_config_drop_filter(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropFilterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_filter_1_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropFilter1Row {
    pub rarity: Box<[i32]>,
    pub drop_lv: Box<[i32]>,
    pub item_lv: Box<[i32]>,
    pub cost: Box<[i32]>,
    pub skill_lv: Box<[i32]>,
    pub talent_node: Box<[FConfigI32I32]>,
    pub mod_id: Box<[i32]>,
    pub talent_node_id: Box<[i32]>,
    pub id: i32,
    pub priority: i32,
    pub item_id: i32,
    pub item_gold_id: i32,
    pub item_gold_talen_id: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
    pub is_identified: i32,
    pub is_eroded: i32,
    pub style: i32,
    pub mod_tier: FConfigI32I32,
}
impl ConfigDropFilter1Row {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.priority = cur.read_i32()?;
        self.item_id = cur.read_i32()?;
        self.item_gold_id = cur.read_i32()?;
        self.item_gold_talen_id = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        self.rarity = cur.read_array_i32()?;
        self.drop_lv = cur.read_array_i32()?;
        self.item_lv = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.cost = cur.read_array_i32()?;
        self.is_identified = cur.read_i32()?;
        self.is_eroded = cur.read_i32()?;
        self.skill_lv = cur.read_array_i32()?;
        self.style = cur.read_i32()?;
        self.mod_tier = FConfigI32I32::read(cur)?;
        self.talent_node = FConfigI32I32::read_array(cur)?;
        self.mod_id = cur.read_array_i32()?;
        self.talent_node_id = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigDropFilter1Row {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_filter_1_ChunkA0.bin`.
pub type ConfigDropFilter1Table = BinTable<i32, ConfigDropFilter1Row>;
/// Parses `config_drop_filter_1_ChunkA0.bin` into a [`ConfigDropFilter1Table`].
pub fn parse_config_drop_filter_1(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropFilter1Table, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_filter_2_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropFilter2Row {
    pub rarity: Box<[i32]>,
    pub drop_lv: Box<[i32]>,
    pub item_lv: Box<[i32]>,
    pub cost: Box<[i32]>,
    pub skill_lv: Box<[i32]>,
    pub talent_node: Box<[FConfigI32I32]>,
    pub mod_id: Box<[i32]>,
    pub talent_node_id: Box<[i32]>,
    pub id: i32,
    pub priority: i32,
    pub item_id: i32,
    pub item_gold_id: i32,
    pub item_gold_talen_id: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
    pub is_identified: i32,
    pub is_eroded: i32,
    pub style: i32,
    pub mod_tier: FConfigI32I32,
}
impl ConfigDropFilter2Row {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.priority = cur.read_i32()?;
        self.item_id = cur.read_i32()?;
        self.item_gold_id = cur.read_i32()?;
        self.item_gold_talen_id = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        self.rarity = cur.read_array_i32()?;
        self.drop_lv = cur.read_array_i32()?;
        self.item_lv = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.cost = cur.read_array_i32()?;
        self.is_identified = cur.read_i32()?;
        self.is_eroded = cur.read_i32()?;
        self.skill_lv = cur.read_array_i32()?;
        self.style = cur.read_i32()?;
        self.mod_tier = FConfigI32I32::read(cur)?;
        self.talent_node = FConfigI32I32::read_array(cur)?;
        self.mod_id = cur.read_array_i32()?;
        self.talent_node_id = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigDropFilter2Row {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_filter_2_ChunkA0.bin`.
pub type ConfigDropFilter2Table = BinTable<i32, ConfigDropFilter2Row>;
/// Parses `config_drop_filter_2_ChunkA0.bin` into a [`ConfigDropFilter2Table`].
pub fn parse_config_drop_filter_2(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropFilter2Table, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_filter_3_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropFilter3Row {
    pub rarity: Box<[i32]>,
    pub drop_lv: Box<[i32]>,
    pub item_lv: Box<[i32]>,
    pub cost: Box<[i32]>,
    pub skill_lv: Box<[i32]>,
    pub talent_node: Box<[FConfigI32I32]>,
    pub mod_id: Box<[i32]>,
    pub talent_node_id: Box<[i32]>,
    pub id: i32,
    pub priority: i32,
    pub item_id: i32,
    pub item_gold_id: i32,
    pub item_gold_talen_id: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
    pub is_identified: i32,
    pub is_eroded: i32,
    pub style: i32,
    pub mod_tier: FConfigI32I32,
}
impl ConfigDropFilter3Row {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.priority = cur.read_i32()?;
        self.item_id = cur.read_i32()?;
        self.item_gold_id = cur.read_i32()?;
        self.item_gold_talen_id = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        self.rarity = cur.read_array_i32()?;
        self.drop_lv = cur.read_array_i32()?;
        self.item_lv = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.cost = cur.read_array_i32()?;
        self.is_identified = cur.read_i32()?;
        self.is_eroded = cur.read_i32()?;
        self.skill_lv = cur.read_array_i32()?;
        self.style = cur.read_i32()?;
        self.mod_tier = FConfigI32I32::read(cur)?;
        self.talent_node = FConfigI32I32::read_array(cur)?;
        self.mod_id = cur.read_array_i32()?;
        self.talent_node_id = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigDropFilter3Row {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_filter_3_ChunkA0.bin`.
pub type ConfigDropFilter3Table = BinTable<i32, ConfigDropFilter3Row>;
/// Parses `config_drop_filter_3_ChunkA0.bin` into a [`ConfigDropFilter3Table`].
pub fn parse_config_drop_filter_3(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropFilter3Table, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_filter_list_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropFilterListRow {
    pub filter: Box<str>,
    pub name: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigDropFilterListRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.filter = cur.read_str()?;
        self.name = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_filter_list_ChunkA0.bin`.
pub type ConfigDropFilterListTable = BinTable<i32, ConfigDropFilterListRow>;
/// Parses `config_drop_filter_list_ChunkA0.bin` into a [`ConfigDropFilterListTable`].
pub fn parse_config_drop_filter_list(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropFilterListTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_parament_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropParamentRow {
    pub r#enum: Box<str>,
    pub name: Box<str>,
    pub id: i32,
    pub modifier_extra: i32,
}
impl BinRow for ConfigDropParamentRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.name = cur.read_str()?;
        self.modifier_extra = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_parament_ChunkA0.bin`.
pub type ConfigDropParamentTable = BinTable<i32, ConfigDropParamentRow>;
/// Parses `config_drop_parament_ChunkA0.bin` into a [`ConfigDropParamentTable`].
pub fn parse_config_drop_parament(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropParamentTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_servant_scheme_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropServantSchemeTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
    pub servant_auto_apply_id: i32,
}
impl BinRow for ConfigDropServantSchemeTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.servant_auto_apply_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_servant_scheme_type_ChunkA0.bin`.
pub type ConfigDropServantSchemeTypeTable = BinTable<
    i32,
    ConfigDropServantSchemeTypeRow,
>;
/// Parses `config_drop_servant_scheme_type_ChunkA0.bin` into a [`ConfigDropServantSchemeTypeTable`].
pub fn parse_config_drop_servant_scheme_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropServantSchemeTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_simulate_map_modifier_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropSimulateMapModifierRow {
    pub modifier_id: i32,
    pub r#type: i32,
    pub group_id: i32,
    pub random_prop: i32,
    pub inc_percent: i32,
    pub more_percent: i32,
    pub source_type: i32,
}
impl BinRow for ConfigDropSimulateMapModifierRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.modifier_id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.group_id = cur.read_i32()?;
        self.random_prop = cur.read_i32()?;
        self.inc_percent = cur.read_i32()?;
        self.more_percent = cur.read_i32()?;
        self.source_type = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_simulate_map_modifier_ChunkA0.bin`.
pub type ConfigDropSimulateMapModifierTable = BinTable<
    (i32, i32),
    ConfigDropSimulateMapModifierRow,
>;
/// Parses `config_drop_simulate_map_modifier_ChunkA0.bin` into a [`ConfigDropSimulateMapModifierTable`].
pub fn parse_config_drop_simulate_map_modifier(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropSimulateMapModifierTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_style_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigDropStyleRow {
    pub text: Box<str>,
    pub text_pre: Box<str>,
    pub text_suf: Box<str>,
    pub icon: Box<str>,
    pub frame_size: f64,
    pub beam: Box<[i32]>,
    pub beam_appear: Box<[i32]>,
    pub pick_trace: Box<[i32]>,
    pub map_icon_size: f64,
    pub id: i32,
    pub text_style: i32,
    pub text_color: i32,
    pub icon_bd: i32,
    pub icon_bg: i32,
    pub text_bd: i32,
    pub text_bg: i32,
    pub pick_priority: i32,
    pub map_icon: i32,
    pub audio_level: i32,
    pub hide: bool,
    pub show_icon: bool,
    pub icon_fx: bool,
}
impl ConfigDropStyleRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.hide = cur.read_bool()?;
        self.text = cur.read_str()?;
        self.text_pre = cur.read_str()?;
        self.text_suf = cur.read_str()?;
        self.text_style = cur.read_i32()?;
        self.text_color = cur.read_i32()?;
        self.show_icon = cur.read_bool()?;
        self.icon_bd = cur.read_i32()?;
        self.icon_bg = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.icon_fx = cur.read_bool()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.text_bd = cur.read_i32()?;
        self.text_bg = cur.read_i32()?;
        self.frame_size = cur.read_f64()?;
        self.pick_priority = cur.read_i32()?;
        self.beam = cur.read_array_i32()?;
        self.beam_appear = cur.read_array_i32()?;
        self.pick_trace = cur.read_array_i32()?;
        self.map_icon = cur.read_i32()?;
        self.map_icon_size = cur.read_f64()?;
        self.audio_level = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigDropStyleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_style_ChunkA0.bin`.
pub type ConfigDropStyleTable = BinTable<i32, ConfigDropStyleRow>;
/// Parses `config_drop_style_ChunkA0.bin` into a [`ConfigDropStyleTable`].
pub fn parse_config_drop_style(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropStyleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_view_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigDropViewRow {
    pub protected_time: f64,
    pub show_delay_time: f64,
    pub vfx_args: FxHashMap<Box<str>, Box<[i32]>>,
    pub sound_args: FxHashMap<Box<str>, Box<str>>,
    pub id: i32,
    pub res_id: i32,
    pub pick_distance: i32,
    pub auto_pick: bool,
    pub show_info: bool,
}
impl BinRow for ConfigDropViewRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.res_id = cur.read_i32()?;
        self.auto_pick = cur.read_bool()?;
        self.protected_time = cur.read_f64()?;
        self.pick_distance = cur.read_i32()?;
        self.show_info = cur.read_bool()?;
        self.show_delay_time = cur.read_f64()?;
        self.vfx_args = cur.read_map_str_array_i32()?;
        self.sound_args = cur.read_map_str_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_view_ChunkA0.bin`.
pub type ConfigDropViewTable = BinTable<i32, ConfigDropViewRow>;
/// Parses `config_drop_view_ChunkA0.bin` into a [`ConfigDropViewTable`].
pub fn parse_config_drop_view(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropViewTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_virtual_entity_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigDropVirtualEntityRow {
    pub normal_drop: Box<[i32]>,
    pub map_drop: Box<[i32]>,
    pub unkonwnmap_drop: Box<[i32]>,
    pub special_drop: FxHashMap<i32, Box<[i32]>>,
    pub drop_affix: Box<[FConfigAffix]>,
    pub icon: Box<[Box<str>]>,
    pub icon_tips: Box<[Box<str>]>,
    pub id: i32,
    pub rairty: i32,
    pub drop_level: i32,
}
impl BinRow for ConfigDropVirtualEntityRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.rairty = cur.read_i32()?;
        self.drop_level = cur.read_i32()?;
        self.normal_drop = cur.read_array_i32()?;
        self.map_drop = cur.read_array_i32()?;
        self.unkonwnmap_drop = cur.read_array_i32()?;
        self.special_drop = cur.read_map_i32_array_i32()?;
        self.drop_affix = FConfigAffix::read_array(cur)?;
        self.icon = cur.read_array_str()?;
        self.icon_tips = cur.read_array_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_virtual_entity_ChunkA0.bin`.
pub type ConfigDropVirtualEntityTable = BinTable<i32, ConfigDropVirtualEntityRow>;
/// Parses `config_drop_virtual_entity_ChunkA0.bin` into a [`ConfigDropVirtualEntityTable`].
pub fn parse_config_drop_virtual_entity(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropVirtualEntityTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_drop_voice_filter_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropVoiceFilterRow {
    pub talent_node: Box<[FConfigI32I32]>,
    pub id: i32,
    pub priority: i32,
    pub item_id: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
    pub item_gold_rarity_level: i32,
    pub mod_tier: FConfigI32I32,
    pub num: i32,
    pub voice: i32,
    pub voice_cue: i32,
}
impl ConfigDropVoiceFilterRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.priority = cur.read_i32()?;
        self.item_id = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        self.item_gold_rarity_level = cur.read_i32()?;
        self.mod_tier = FConfigI32I32::read(cur)?;
        self.talent_node = FConfigI32I32::read_array(cur)?;
        self.num = cur.read_i32()?;
        self.voice = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.voice_cue = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigDropVoiceFilterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_drop_voice_filter_ChunkA0.bin`.
pub type ConfigDropVoiceFilterTable = BinTable<i32, ConfigDropVoiceFilterRow>;
/// Parses `config_drop_voice_filter_ChunkA0.bin` into a [`ConfigDropVoiceFilterTable`].
pub fn parse_config_drop_voice_filter(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigDropVoiceFilterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_reward_group_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRewardGroupRow {
    pub args: Box<str>,
    pub rewards: Box<str>,
    pub id: i32,
    pub r#type: i32,
}
impl BinRow for ConfigRewardGroupRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.args = cur.read_str()?;
        self.rewards = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_reward_group_ChunkA0.bin`.
pub type ConfigRewardGroupTable = BinTable<i32, ConfigRewardGroupRow>;
/// Parses `config_reward_group_ChunkA0.bin` into a [`ConfigRewardGroupTable`].
pub fn parse_config_reward_group(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigRewardGroupTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
#[inline(never)]
fn verify_drop_chunk_0(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_drop_audio" => {
            Some(parse_config_drop_audio(buf, scratch).map(|t| t.len()))
        }
        "config_drop_carrier" => {
            Some(parse_config_drop_carrier(buf, scratch).map(|t| t.len()))
        }
        "config_drop_filter" => {
            Some(parse_config_drop_filter(buf, scratch).map(|t| t.len()))
        }
        "config_drop_filter_1" => {
            Some(parse_config_drop_filter_1(buf, scratch).map(|t| t.len()))
        }
        "config_drop_filter_2" => {
            Some(parse_config_drop_filter_2(buf, scratch).map(|t| t.len()))
        }
        "config_drop_filter_3" => {
            Some(parse_config_drop_filter_3(buf, scratch).map(|t| t.len()))
        }
        "config_drop_filter_list" => {
            Some(parse_config_drop_filter_list(buf, scratch).map(|t| t.len()))
        }
        "config_drop_parament" => {
            Some(parse_config_drop_parament(buf, scratch).map(|t| t.len()))
        }
        "config_drop_servant_scheme_type" => {
            Some(parse_config_drop_servant_scheme_type(buf, scratch).map(|t| t.len()))
        }
        "config_drop_simulate_map_modifier" => {
            Some(parse_config_drop_simulate_map_modifier(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_drop_chunk_1(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_drop_style" => {
            Some(parse_config_drop_style(buf, scratch).map(|t| t.len()))
        }
        "config_drop_view" => Some(parse_config_drop_view(buf, scratch).map(|t| t.len())),
        "config_drop_virtual_entity" => {
            Some(parse_config_drop_virtual_entity(buf, scratch).map(|t| t.len()))
        }
        "config_drop_voice_filter" => {
            Some(parse_config_drop_voice_filter(buf, scratch).map(|t| t.len()))
        }
        "config_reward_group" => {
            Some(parse_config_reward_group(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
pub(crate) fn verify_drop_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    if let Some(res) = verify_drop_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_drop_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    None
}

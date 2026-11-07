//! Auto-generated `.bin` configuration table types and parsers for the `item` domain.
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{FConfigI32ArrayI32, FConfigI32I32, FConfigStrI32};
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_currency_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCurrencyRow {
    pub name: Box<str>,
    pub availableitemtype: Box<[i32]>,
    pub availableitemrarity: Box<[i32]>,
    pub r#type: Box<str>,
    pub weight: Box<str>,
    pub added: Box<str>,
    pub announcement: Box<str>,
    pub intro: Box<str>,
    pub id: i32,
    pub subid: i32,
    pub currencyid: i32,
    pub logicid: i32,
    pub parameter: i32,
    pub rarity: i32,
}
impl ConfigCurrencyRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.subid = cur.read_i32()?;
        self.currencyid = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.availableitemtype = cur.read_array_i32()?;
        self.availableitemrarity = cur.read_array_i32()?;
        self.logicid = cur.read_i32()?;
        self.r#type = cur.read_str()?;
        self.parameter = cur.read_i32()?;
        self.weight = cur.read_str()?;
        self.added = cur.read_str()?;
        self.announcement = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.rarity = cur.read_i32()?;
        self.intro = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigCurrencyRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_currency_ChunkA0.bin`.
pub type ConfigCurrencyTable = BinTable<i32, ConfigCurrencyRow>;
/// Parses `config_currency_ChunkA0.bin` into a [`ConfigCurrencyTable`].
pub fn parse_config_currency(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigCurrencyTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_fashion_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFashionRow {
    pub name: Box<str>,
    pub icon: Box<str>,
    pub fashion_item_icon: Box<str>,
    pub fashion_item_icon_effect: Box<str>,
    pub description: Box<str>,
    pub skin: Box<str>,
    pub behavior: Box<str>,
    pub sequence: Box<str>,
    pub level: Box<str>,
    pub show_media: Box<str>,
    pub color: Box<str>,
    pub active_item: Box<str>,
    pub skill: Box<[i32]>,
    pub rule: Box<[i32]>,
    pub recycle: FConfigStrI32,
    pub icon_pick_result: Box<str>,
    pub icon_pick_card_list_show: Box<str>,
    pub action_fashion_anim: FxHashMap<i32, Box<str>>,
    pub audio: Box<[i32]>,
    pub pick_card_cover_img: Box<str>,
    pub skin_v2: Box<str>,
    pub fashion_item_spine: Box<str>,
    pub id: i32,
    pub r#type: i32,
    pub effective: i32,
    pub hero: i32,
    pub weapon: i32,
    pub weapon_show: i32,
    pub show_resources_id: i32,
    pub offhand_mirror_axes: i32,
    pub order: i32,
    pub replaced_gender: i32,
    pub is_vaild: i32,
    pub invaild_compensation: i32,
}
impl ConfigFashionRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.r#type = cur.read_i32()?;
        self.effective = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.fashion_item_icon = cur.read_str()?;
        self.fashion_item_icon_effect = cur.read_str()?;
        self.description = cur.read_str()?;
        self.skin = cur.read_str()?;
        self.behavior = cur.read_str()?;
        self.sequence = cur.read_str()?;
        self.level = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.show_media = cur.read_str()?;
        self.color = cur.read_str()?;
        self.active_item = cur.read_str()?;
        self.hero = cur.read_i32()?;
        self.weapon = cur.read_i32()?;
        self.skill = cur.read_array_i32()?;
        self.rule = cur.read_array_i32()?;
        self.recycle = FConfigStrI32::read(cur)?;
        self.weapon_show = cur.read_i32()?;
        self.show_resources_id = cur.read_i32()?;
        self.icon_pick_result = cur.read_str()?;
        self.offhand_mirror_axes = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.icon_pick_card_list_show = cur.read_str()?;
        self.action_fashion_anim = cur.read_map_i32_str()?;
        self.order = cur.read_i32()?;
        self.audio = cur.read_array_i32()?;
        self.replaced_gender = cur.read_i32()?;
        self.pick_card_cover_img = cur.read_str()?;
        self.skin_v2 = cur.read_str()?;
        self.is_vaild = cur.read_i32()?;
        self.invaild_compensation = cur.read_i32()?;
        self.fashion_item_spine = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigFashionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_fashion_ChunkA0.bin`.
pub type ConfigFashionTable = BinTable<i32, ConfigFashionRow>;
/// Parses `config_fashion_ChunkA0.bin` into a [`ConfigFashionTable`].
pub fn parse_config_fashion(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigFashionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_fashion_audio_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigFashionAudioRow {
    pub desc: Box<str>,
    pub audio_path: Box<str>,
    pub cooldown: f64,
    pub id: i32,
    pub cue: i32,
}
impl BinRow for ConfigFashionAudioRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.audio_path = cur.read_str()?;
        self.cooldown = cur.read_f64()?;
        self.cue = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_fashion_audio_ChunkA0.bin`.
pub type ConfigFashionAudioTable = BinTable<i32, ConfigFashionAudioRow>;
/// Parses `config_fashion_audio_ChunkA0.bin` into a [`ConfigFashionAudioTable`].
pub fn parse_config_fashion_audio(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigFashionAudioTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_fashion_rule_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigFashionRuleRow {
    pub vfx: Box<[i32]>,
    pub vfx_showcase: Box<[i32]>,
    pub buff_vfx: Box<[FConfigI32I32]>,
    pub replace_weapon_res_main: FxHashMap<i32, i32>,
    pub replace_weapon_res_off: FxHashMap<i32, i32>,
    pub replace_portal_type: Box<[i32]>,
    pub body_vfx: Box<str>,
    pub body_slot: Box<str>,
    pub fashion_body_vfx: FxHashMap<Box<str>, Box<str>>,
    pub weapon_round_vfx: Box<str>,
    pub weapon_box_vfx: Box<str>,
    pub weapon_trail_vfx: Box<str>,
    pub weapon_round_dot_vfx: Box<str>,
    pub fashion_body_vfx_fight: Box<[i32]>,
    pub replace_summon_res: Box<[FConfigI32ArrayI32]>,
    pub hero_speciality_skill_stones: Box<[i32]>,
    pub replace_drop_vfx_beam: FxHashMap<i32, i32>,
    pub replace_drop_vfx_beam_appear: FxHashMap<i32, i32>,
    pub replace_drop_vfx_pick_trace: FxHashMap<i32, i32>,
    pub custom_param: Box<str>,
    pub skill_attachment_vfx_by_tag: FxHashMap<i32, i32>,
    pub replace_standalone_vfx_by_id: FxHashMap<i32, i32>,
    pub skill_attachment_delay: f64,
    pub skill_agent_res_id_by_tag: FxHashMap<i32, i32>,
    pub extra_kill_fx: Box<[i32]>,
    pub replace_damage_fx: Box<[i32]>,
    pub replace_vfx_event: FxHashMap<i32, i32>,
    pub portal_scale: Box<[f64]>,
    pub woodstakes_bodypart_res: FxHashMap<Box<str>, i32>,
    pub replace_juli: Box<[i32]>,
    pub hero_character_main_weapon: FxHashMap<i32, i32>,
    pub hero_character_offhand_weapon: FxHashMap<i32, i32>,
    pub hero_character_action: Box<str>,
    pub extra_vfx: Box<str>,
    pub extra_slot: Box<str>,
    pub id: i32,
    pub replace_char_res: i32,
    pub char_res_material_set_id: i32,
    pub replace_portal_resource_id: i32,
    pub replace_portal_mesh_id: i32,
    pub replace_portal_anim_active_id: i32,
    pub replace_portal_anim_idle_id: i32,
    pub replace_portal_vfx_id: i32,
    pub replace_portal_close_vfx_id: i32,
    pub replace_portal_touch_vfx_id: i32,
    pub replace_portal_untouchable_vfx_id: i32,
    pub replace_portal_enter_vfx_id: i32,
    pub replace_portal_enter_mtg_id: i32,
    pub replace_portal_exit_vfx_id: i32,
    pub replace_portal_exit_mtg_id: i32,
    pub footstep_vfx_id: i32,
    pub skill_repeat_vfx_id: i32,
    pub skill_repeat_ext_vfx_id: i32,
    pub body_vfx_fight: i32,
    pub weapon_round_vfx_fight: i32,
    pub weapon_box_vfx_fight: i32,
    pub weapon_trail_vfx_fight: i32,
    pub weapon_round_dot_vfx_fight: i32,
    pub replace_woodstakes_res: i32,
    pub six_god_vfx: i32,
    pub replace_hero_res_set: i32,
    pub custom_skill_repeat_vfx: bool,
}
impl ConfigFashionRuleRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.vfx = cur.read_array_i32()?;
        self.vfx_showcase = cur.read_array_i32()?;
        self.buff_vfx = FConfigI32I32::read_array(cur)?;
        self.replace_weapon_res_main = cur.read_map_i32_i32()?;
        self.replace_weapon_res_off = cur.read_map_i32_i32()?;
        self.replace_char_res = cur.read_i32()?;
        self.char_res_material_set_id = cur.read_i32()?;
        self.replace_portal_resource_id = cur.read_i32()?;
        self.replace_portal_type = cur.read_array_i32()?;
        self.replace_portal_mesh_id = cur.read_i32()?;
        self.replace_portal_anim_active_id = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.replace_portal_anim_idle_id = cur.read_i32()?;
        self.replace_portal_vfx_id = cur.read_i32()?;
        self.replace_portal_close_vfx_id = cur.read_i32()?;
        self.replace_portal_touch_vfx_id = cur.read_i32()?;
        self.replace_portal_untouchable_vfx_id = cur.read_i32()?;
        self.replace_portal_enter_vfx_id = cur.read_i32()?;
        self.replace_portal_enter_mtg_id = cur.read_i32()?;
        self.replace_portal_exit_vfx_id = cur.read_i32()?;
        self.replace_portal_exit_mtg_id = cur.read_i32()?;
        self.footstep_vfx_id = cur.read_i32()?;
        self.skill_repeat_vfx_id = cur.read_i32()?;
        self.skill_repeat_ext_vfx_id = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.custom_skill_repeat_vfx = cur.read_bool()?;
        self.body_vfx = cur.read_str()?;
        self.body_slot = cur.read_str()?;
        self.fashion_body_vfx = cur.read_map_str_str()?;
        self.weapon_round_vfx = cur.read_str()?;
        self.weapon_box_vfx = cur.read_str()?;
        self.weapon_trail_vfx = cur.read_str()?;
        self.weapon_round_dot_vfx = cur.read_str()?;
        self.body_vfx_fight = cur.read_i32()?;
        self.fashion_body_vfx_fight = cur.read_array_i32()?;
        self.weapon_round_vfx_fight = cur.read_i32()?;
        self.weapon_box_vfx_fight = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.weapon_trail_vfx_fight = cur.read_i32()?;
        self.weapon_round_dot_vfx_fight = cur.read_i32()?;
        self.replace_summon_res = FConfigI32ArrayI32::read_array(cur)?;
        self.replace_woodstakes_res = cur.read_i32()?;
        self.hero_speciality_skill_stones = cur.read_array_i32()?;
        self.replace_drop_vfx_beam = cur.read_map_i32_i32()?;
        self.replace_drop_vfx_beam_appear = cur.read_map_i32_i32()?;
        self.replace_drop_vfx_pick_trace = cur.read_map_i32_i32()?;
        self.custom_param = cur.read_str()?;
        self.skill_attachment_vfx_by_tag = cur.read_map_i32_i32()?;
        self.replace_standalone_vfx_by_id = cur.read_map_i32_i32()?;
        self.skill_attachment_delay = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_4(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.skill_agent_res_id_by_tag = cur.read_map_i32_i32()?;
        self.extra_kill_fx = cur.read_array_i32()?;
        self.replace_damage_fx = cur.read_array_i32()?;
        self.replace_vfx_event = cur.read_map_i32_i32()?;
        self.six_god_vfx = cur.read_i32()?;
        self.portal_scale = cur.read_array_f64()?;
        self.woodstakes_bodypart_res = cur.read_map_str_i32()?;
        self.replace_hero_res_set = cur.read_i32()?;
        self.replace_juli = cur.read_array_i32()?;
        self.hero_character_main_weapon = cur.read_map_i32_i32()?;
        self.hero_character_offhand_weapon = cur.read_map_i32_i32()?;
        self.hero_character_action = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_5(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.extra_vfx = cur.read_str()?;
        self.extra_slot = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigFashionRuleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        self.read_fields_4(cur)?;
        self.read_fields_5(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_fashion_rule_ChunkA0.bin`.
pub type ConfigFashionRuleTable = BinTable<i32, ConfigFashionRuleRow>;
/// Parses `config_fashion_rule_ChunkA0.bin` into a [`ConfigFashionRuleTable`].
pub fn parse_config_fashion_rule(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigFashionRuleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_item_affix_threshold_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemAffixThresholdRow {
    pub description: Box<str>,
    pub id: i32,
    pub type1: i32,
    pub type2: i32,
    pub behavior: i32,
    pub rarity: i32,
    pub minaffixnum: i32,
    pub maxaffixnum: i32,
    pub minprefixnum: i32,
    pub maxprefixnum: i32,
    pub minsuffixnum: i32,
    pub maxsuffixnum: i32,
}
impl BinRow for ConfigItemAffixThresholdRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.behavior = cur.read_i32()?;
        self.description = cur.read_str()?;
        self.rarity = cur.read_i32()?;
        self.minaffixnum = cur.read_i32()?;
        self.maxaffixnum = cur.read_i32()?;
        self.minprefixnum = cur.read_i32()?;
        self.maxprefixnum = cur.read_i32()?;
        self.minsuffixnum = cur.read_i32()?;
        self.maxsuffixnum = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_item_affix_threshold_ChunkA0.bin`.
pub type ConfigItemAffixThresholdTable = BinTable<i32, ConfigItemAffixThresholdRow>;
/// Parses `config_item_affix_threshold_ChunkA0.bin` into a [`ConfigItemAffixThresholdTable`].
pub fn parse_config_item_affix_threshold(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigItemAffixThresholdTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_item_base_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemBaseRow {
    pub name: Box<str>,
    pub icon: Box<str>,
    pub icon256: Box<str>,
    pub resource: Box<str>,
    pub description1: Box<str>,
    pub description2: Box<str>,
    pub item_usable: FxHashMap<i32, Box<str>>,
    pub deal_forbid_status: Box<[i32]>,
    pub base_modifier: Box<[i32]>,
    pub func_modifier: Box<[FConfigI32ArrayI32]>,
    pub map_modifier: Box<[i32]>,
    pub position: Box<[i32]>,
    pub need_attr: FxHashMap<i32, i32>,
    pub fragment: Box<str>,
    pub active: Box<str>,
    pub detail: Box<str>,
    pub cost: Box<[i32]>,
    pub icon_card: Box<str>,
    pub hero_purchase: Box<[i32]>,
    pub drop_desc: Box<[i32]>,
    pub use_details: Box<str>,
    pub base_attr_modifier_group: Box<[i32]>,
    pub id: i32,
    pub resource_equip: i32,
    pub drop_view: i32,
    pub show_rarity: i32,
    pub drop_hint_level: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
    pub reward_box: FConfigI32I32,
    pub max_num: i32,
    pub tier: i32,
    pub deal: i32,
    pub deal_currency: i32,
    pub skill: FConfigI32I32,
    pub score: i32,
    pub require_level: i32,
    pub hero: i32,
    pub use_mode: i32,
    pub drop_filter: i32,
    pub map_material_limit_num: i32,
    pub reward_box_alter: i32,
    pub formal: bool,
    pub bind_after_deal: bool,
}
impl ConfigItemBaseRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.icon = cur.read_str()?;
        self.icon256 = cur.read_str()?;
        self.resource = cur.read_str()?;
        self.resource_equip = cur.read_i32()?;
        self.drop_view = cur.read_i32()?;
        self.description1 = cur.read_str()?;
        self.description2 = cur.read_str()?;
        self.show_rarity = cur.read_i32()?;
        self.drop_hint_level = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        self.item_usable = cur.read_map_i32_str()?;
        self.reward_box = FConfigI32I32::read(cur)?;
        self.max_num = cur.read_i32()?;
        self.tier = cur.read_i32()?;
        self.deal = cur.read_i32()?;
        self.deal_currency = cur.read_i32()?;
        self.deal_forbid_status = cur.read_array_i32()?;
        self.skill = FConfigI32I32::read(cur)?;
        self.base_modifier = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.func_modifier = FConfigI32ArrayI32::read_array(cur)?;
        self.map_modifier = cur.read_array_i32()?;
        self.score = cur.read_i32()?;
        self.require_level = cur.read_i32()?;
        self.position = cur.read_array_i32()?;
        self.need_attr = cur.read_map_i32_i32()?;
        self.fragment = cur.read_str()?;
        self.hero = cur.read_i32()?;
        self.active = cur.read_str()?;
        self.detail = cur.read_str()?;
        self.cost = cur.read_array_i32()?;
        self.icon_card = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.hero_purchase = cur.read_array_i32()?;
        self.drop_desc = cur.read_array_i32()?;
        self.use_mode = cur.read_i32()?;
        self.use_details = cur.read_str()?;
        self.drop_filter = cur.read_i32()?;
        self.formal = cur.read_bool()?;
        self.base_attr_modifier_group = cur.read_array_i32()?;
        self.bind_after_deal = cur.read_bool()?;
        self.map_material_limit_num = cur.read_i32()?;
        self.reward_box_alter = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigItemBaseRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_item_base_ChunkA0.bin`.
pub type ConfigItemBaseTable = BinTable<i32, ConfigItemBaseRow>;
/// Parses `config_item_base_ChunkA0.bin` into a [`ConfigItemBaseTable`].
pub fn parse_config_item_base(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigItemBaseTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_item_base_skill_index_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemBaseSkillIndexRow {
    pub ids: Box<[i32]>,
    pub skill_key: i32,
}
impl BinRow for ConfigItemBaseSkillIndexRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.skill_key = cur.read_i32()?;
        self.ids = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_item_base_skill_index_ChunkA0.bin`.
pub type ConfigItemBaseSkillIndexTable = BinTable<i32, ConfigItemBaseSkillIndexRow>;
/// Parses `config_item_base_skill_index_ChunkA0.bin` into a [`ConfigItemBaseSkillIndexTable`].
pub fn parse_config_item_base_skill_index(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigItemBaseSkillIndexTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_item_base_type1_index_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemBaseType1IndexRow {
    pub ids: Box<[i32]>,
    pub skill_keys: Box<[i32]>,
    pub type1: i32,
}
impl BinRow for ConfigItemBaseType1IndexRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.type1 = cur.read_i32()?;
        self.ids = cur.read_array_i32()?;
        self.skill_keys = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_item_base_type1_index_ChunkA0.bin`.
pub type ConfigItemBaseType1IndexTable = BinTable<i32, ConfigItemBaseType1IndexRow>;
/// Parses `config_item_base_type1_index_ChunkA0.bin` into a [`ConfigItemBaseType1IndexTable`].
pub fn parse_config_item_base_type1_index(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigItemBaseType1IndexTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_item_cost_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemCostRow {
    pub type4: Box<[i32]>,
    pub level: Box<[i32]>,
    pub cost: Box<[i32]>,
    pub id: i32,
    pub type1: i32,
}
impl BinRow for ConfigItemCostRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.type1 = cur.read_i32()?;
        self.type4 = cur.read_array_i32()?;
        self.level = cur.read_array_i32()?;
        self.cost = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_item_cost_ChunkA0.bin`.
pub type ConfigItemCostTable = BinTable<i32, ConfigItemCostRow>;
/// Parses `config_item_cost_ChunkA0.bin` into a [`ConfigItemCostTable`].
pub fn parse_config_item_cost(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigItemCostTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_item_drop_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemDropRow {
    pub drop_desc: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigItemDropRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.drop_desc = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_item_drop_ChunkA0.bin`.
pub type ConfigItemDropTable = BinTable<i32, ConfigItemDropRow>;
/// Parses `config_item_drop_ChunkA0.bin` into a [`ConfigItemDropTable`].
pub fn parse_config_item_drop(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigItemDropTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_item_gold_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemGoldRow {
    pub deal_forbid_status: Box<[i32]>,
    pub name: Box<str>,
    pub icon: Box<str>,
    pub resource: Box<str>,
    pub description: Box<str>,
    pub prefix1: FxHashMap<i32, Box<[i32]>>,
    pub prefix1_des: Box<str>,
    pub prefix2: FxHashMap<i32, Box<[i32]>>,
    pub prefix2_des: Box<str>,
    pub prefix3: FxHashMap<i32, Box<[i32]>>,
    pub prefix3_des: Box<str>,
    pub prefix4: FxHashMap<i32, Box<[i32]>>,
    pub prefix4_des: Box<str>,
    pub suffix1: FxHashMap<i32, Box<[i32]>>,
    pub suffix1_des: Box<str>,
    pub suffix2: FxHashMap<i32, Box<[i32]>>,
    pub suffix2_des: Box<str>,
    pub suffix3: FxHashMap<i32, Box<[i32]>>,
    pub suffix3_des: Box<str>,
    pub suffix4: FxHashMap<i32, Box<[i32]>>,
    pub suffix4_des: Box<str>,
    pub hero_purchase: Box<[i32]>,
    pub drop_desc: Box<[i32]>,
    pub drop_level_interval: Box<[i32]>,
    pub id: i32,
    pub base_id: i32,
    pub deal: i32,
    pub resource_equip: i32,
    pub drop_view: i32,
    pub drop_hint_level: i32,
    pub require_level: i32,
    pub drop_level: i32,
    pub drop_type: i32,
    pub rarity_level: i32,
    pub drop_filter: i32,
    pub slot_type: i32,
    pub affix_no_repeat: i32,
    pub formal: bool,
    pub is_mercenary: bool,
}
impl ConfigItemGoldRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.base_id = cur.read_i32()?;
        self.deal = cur.read_i32()?;
        self.deal_forbid_status = cur.read_array_i32()?;
        self.name = cur.read_str()?;
        self.icon = cur.read_str()?;
        self.resource = cur.read_str()?;
        self.resource_equip = cur.read_i32()?;
        self.drop_view = cur.read_i32()?;
        self.drop_hint_level = cur.read_i32()?;
        self.description = cur.read_str()?;
        self.prefix1 = cur.read_map_i32_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.prefix1_des = cur.read_str()?;
        self.prefix2 = cur.read_map_i32_array_i32()?;
        self.prefix2_des = cur.read_str()?;
        self.prefix3 = cur.read_map_i32_array_i32()?;
        self.prefix3_des = cur.read_str()?;
        self.prefix4 = cur.read_map_i32_array_i32()?;
        self.prefix4_des = cur.read_str()?;
        self.suffix1 = cur.read_map_i32_array_i32()?;
        self.suffix1_des = cur.read_str()?;
        self.suffix2 = cur.read_map_i32_array_i32()?;
        self.suffix2_des = cur.read_str()?;
        self.suffix3 = cur.read_map_i32_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.suffix3_des = cur.read_str()?;
        self.suffix4 = cur.read_map_i32_array_i32()?;
        self.suffix4_des = cur.read_str()?;
        self.require_level = cur.read_i32()?;
        self.drop_level = cur.read_i32()?;
        self.drop_type = cur.read_i32()?;
        self.rarity_level = cur.read_i32()?;
        self.hero_purchase = cur.read_array_i32()?;
        self.drop_desc = cur.read_array_i32()?;
        self.drop_filter = cur.read_i32()?;
        self.formal = cur.read_bool()?;
        self.slot_type = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.drop_level_interval = cur.read_array_i32()?;
        self.affix_no_repeat = cur.read_i32()?;
        self.is_mercenary = cur.read_bool()?;
        Ok(())
    }
}
impl BinRow for ConfigItemGoldRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        self.read_fields_3(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_item_gold_ChunkA0.bin`.
pub type ConfigItemGoldTable = BinTable<i32, ConfigItemGoldRow>;
/// Parses `config_item_gold_ChunkA0.bin` into a [`ConfigItemGoldTable`].
pub fn parse_config_item_gold(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigItemGoldTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_item_icon_mapping_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemIconMappingRow {
    pub icon: Box<str>,
    pub rarity: Box<[i32]>,
    pub id: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
}
impl BinRow for ConfigItemIconMappingRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.rarity = cur.read_array_i32()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_item_icon_mapping_ChunkA0.bin`.
pub type ConfigItemIconMappingTable = BinTable<i32, ConfigItemIconMappingRow>;
/// Parses `config_item_icon_mapping_ChunkA0.bin` into a [`ConfigItemIconMappingTable`].
pub fn parse_config_item_icon_mapping(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigItemIconMappingTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_item_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigItemTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_item_type_ChunkA0.bin`.
pub type ConfigItemTypeTable = BinTable<i32, ConfigItemTypeRow>;
/// Parses `config_item_type_ChunkA0.bin` into a [`ConfigItemTypeTable`].
pub fn parse_config_item_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigItemTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
#[inline(never)]
fn verify_item_chunk_0(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_currency" => Some(parse_config_currency(buf, scratch).map(|t| t.len())),
        "config_fashion" => Some(parse_config_fashion(buf, scratch).map(|t| t.len())),
        "config_fashion_audio" => {
            Some(parse_config_fashion_audio(buf, scratch).map(|t| t.len()))
        }
        "config_fashion_rule" => {
            Some(parse_config_fashion_rule(buf, scratch).map(|t| t.len()))
        }
        "config_item_affix_threshold" => {
            Some(parse_config_item_affix_threshold(buf, scratch).map(|t| t.len()))
        }
        "config_item_base" => Some(parse_config_item_base(buf, scratch).map(|t| t.len())),
        "config_item_base_skill_index" => {
            Some(parse_config_item_base_skill_index(buf, scratch).map(|t| t.len()))
        }
        "config_item_base_type1_index" => {
            Some(parse_config_item_base_type1_index(buf, scratch).map(|t| t.len()))
        }
        "config_item_cost" => Some(parse_config_item_cost(buf, scratch).map(|t| t.len())),
        "config_item_drop" => Some(parse_config_item_drop(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}
#[inline(never)]
fn verify_item_chunk_1(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_item_gold" => Some(parse_config_item_gold(buf, scratch).map(|t| t.len())),
        "config_item_icon_mapping" => {
            Some(parse_config_item_icon_mapping(buf, scratch).map(|t| t.len()))
        }
        "config_item_type" => Some(parse_config_item_type(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}
pub(crate) fn verify_item_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    if let Some(res) = verify_item_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_item_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    None
}

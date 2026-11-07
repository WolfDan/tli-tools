//! Auto-generated `.bin` configuration table types and parsers for the `hero` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{FConfigI32I32, FConfigI32Str};
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_hero_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHeroRow {
    pub name: Box<str>,
    pub nickname: Box<str>,
    pub background_story: Box<str>,
    pub portrait1: Box<str>,
    pub portrait2: Box<str>,
    pub portrait3: Box<str>,
    pub portrait4: Box<str>,
    pub icon: Box<str>,
    pub pic: Box<str>,
    pub purchase_id: Box<[i32]>,
    pub character_bg: Box<str>,
    pub short_name: Box<str>,
    pub id: i32,
    pub order: i32,
    pub effective: i32,
    pub visible: i32,
    pub recommend: i32,
    pub hero_tale_id: i32,
    pub hero_audio_id: i32,
}
impl ConfigHeroRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.order = cur.read_i32()?;
        self.effective = cur.read_i32()?;
        self.visible = cur.read_i32()?;
        self.recommend = cur.read_i32()?;
        self.nickname = cur.read_str()?;
        self.background_story = cur.read_str()?;
        self.portrait1 = cur.read_str()?;
        self.portrait2 = cur.read_str()?;
        self.portrait3 = cur.read_str()?;
        self.portrait4 = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.icon = cur.read_str()?;
        self.hero_tale_id = cur.read_i32()?;
        self.hero_audio_id = cur.read_i32()?;
        self.pic = cur.read_str()?;
        self.purchase_id = cur.read_array_i32()?;
        self.character_bg = cur.read_str()?;
        self.short_name = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigHeroRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hero_ChunkA0.bin`.
pub type ConfigHeroTable = BinTable<i32, ConfigHeroRow>;
/// Parses `config_hero_ChunkA0.bin` into a [`ConfigHeroTable`].
pub fn parse_config_hero(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHeroTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hero_character_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigHeroCharacterRow {
    pub before_id: Box<[i32]>,
    pub name: Box<str>,
    pub affix: Box<[FConfigAffix]>,
    pub additional_affix: Box<[FConfigAffix]>,
    pub buff_effect_description: Box<[i32]>,
    pub icon: Box<str>,
    pub desc: Box<str>,
    pub slot_index: Box<[i32]>,
    pub slot_rule: Box<str>,
    pub slot_des: Box<str>,
    pub desc_max: Box<str>,
    pub tier_id_list: Box<[i32]>,
    pub position: Box<[i32]>,
    pub character_id: i32,
    pub sub_index: i32,
    pub tab_index: i32,
    pub hero_id: i32,
    pub hero_purchase_id: i32,
    pub level: i32,
    pub task_id: i32,
    pub hunter_trial: i32,
    pub slot: i32,
    pub slot_num: i32,
    pub group: i32,
    pub can_add_by_other_purchase: bool,
}
impl ConfigHeroCharacterRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.character_id = cur.read_i32()?;
        self.sub_index = cur.read_i32()?;
        self.tab_index = cur.read_i32()?;
        self.hero_id = cur.read_i32()?;
        self.hero_purchase_id = cur.read_i32()?;
        self.level = cur.read_i32()?;
        self.task_id = cur.read_i32()?;
        self.hunter_trial = cur.read_i32()?;
        self.before_id = cur.read_array_i32()?;
        self.name = cur.read_str()?;
        self.can_add_by_other_purchase = cur.read_bool()?;
        self.affix = FConfigAffix::read_array(cur)?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.additional_affix = FConfigAffix::read_array(cur)?;
        self.buff_effect_description = cur.read_array_i32()?;
        self.slot = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.slot_num = cur.read_i32()?;
        self.slot_index = cur.read_array_i32()?;
        self.slot_rule = cur.read_str()?;
        self.slot_des = cur.read_str()?;
        self.desc_max = cur.read_str()?;
        self.tier_id_list = cur.read_array_i32()?;
        self.group = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.position = cur.read_array_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigHeroCharacterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hero_character_ChunkA0.bin`.
pub type ConfigHeroCharacterTable = BinTable<i32, ConfigHeroCharacterRow>;
/// Parses `config_hero_character_ChunkA0.bin` into a [`ConfigHeroCharacterTable`].
pub fn parse_config_hero_character(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHeroCharacterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hero_character_additional_affix_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigHeroCharacterAdditionalAffixRow {
    pub affix2: Box<[FConfigAffix]>,
    pub id: i32,
    pub skill_stone_id: i32,
    pub character_tier: i32,
}
impl BinRow for ConfigHeroCharacterAdditionalAffixRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.skill_stone_id = cur.read_i32()?;
        self.character_tier = cur.read_i32()?;
        self.affix2 = FConfigAffix::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hero_character_additional_affix_ChunkA0.bin`.
pub type ConfigHeroCharacterAdditionalAffixTable = BinTable<
    i32,
    ConfigHeroCharacterAdditionalAffixRow,
>;
/// Parses `config_hero_character_additional_affix_ChunkA0.bin` into a [`ConfigHeroCharacterAdditionalAffixTable`].
pub fn parse_config_hero_character_additional_affix(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHeroCharacterAdditionalAffixTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hero_character_custom_slot_options_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigHeroCharacterCustomSlotOptionsRow {
    pub name: Box<str>,
    pub icon: Box<str>,
    pub affix: Box<[FConfigAffix]>,
    pub group_name: Box<str>,
    pub id: i32,
    pub character_tier_id: i32,
    pub level: i32,
    pub next_id: i32,
    pub preview_slot_id: i32,
    pub preview_options_index: i32,
}
impl BinRow for ConfigHeroCharacterCustomSlotOptionsRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.icon = cur.read_str()?;
        self.character_tier_id = cur.read_i32()?;
        self.affix = FConfigAffix::read_array(cur)?;
        self.level = cur.read_i32()?;
        self.group_name = cur.read_str()?;
        self.next_id = cur.read_i32()?;
        self.preview_slot_id = cur.read_i32()?;
        self.preview_options_index = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hero_character_custom_slot_options_ChunkA0.bin`.
pub type ConfigHeroCharacterCustomSlotOptionsTable = BinTable<
    i32,
    ConfigHeroCharacterCustomSlotOptionsRow,
>;
/// Parses `config_hero_character_custom_slot_options_ChunkA0.bin` into a [`ConfigHeroCharacterCustomSlotOptionsTable`].
pub fn parse_config_hero_character_custom_slot_options(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHeroCharacterCustomSlotOptionsTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hero_character_jewel_slot_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHeroCharacterJewelSlotRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigHeroCharacterJewelSlotRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hero_character_jewel_slot_ChunkA0.bin`.
pub type ConfigHeroCharacterJewelSlotTable = BinTable<
    i32,
    ConfigHeroCharacterJewelSlotRow,
>;
/// Parses `config_hero_character_jewel_slot_ChunkA0.bin` into a [`ConfigHeroCharacterJewelSlotTable`].
pub fn parse_config_hero_character_jewel_slot(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHeroCharacterJewelSlotTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hero_character_tier_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigHeroCharacterTierRow {
    pub affix: Box<[FConfigAffix]>,
    pub additional_affix: Box<[FConfigAffix]>,
    pub server_affix: Box<str>,
    pub character_tier_id: i32,
    pub character_id: i32,
}
impl BinRow for ConfigHeroCharacterTierRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.character_tier_id = cur.read_i32()?;
        self.character_id = cur.read_i32()?;
        self.affix = FConfigAffix::read_array(cur)?;
        self.additional_affix = FConfigAffix::read_array(cur)?;
        self.server_affix = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hero_character_tier_ChunkA0.bin`.
pub type ConfigHeroCharacterTierTable = BinTable<i32, ConfigHeroCharacterTierRow>;
/// Parses `config_hero_character_tier_ChunkA0.bin` into a [`ConfigHeroCharacterTierTable`].
pub fn parse_config_hero_character_tier(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHeroCharacterTierTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hero_purchase_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigHeroPurchaseRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub name: Box<str>,
    pub nickname: Box<str>,
    pub fashion_show_icon: Box<str>,
    pub create_role_small_icon: Box<str>,
    pub character_icon: Box<str>,
    pub rank_portrait_icon: Box<str>,
    pub show_character: Box<str>,
    pub default_fashions: Box<[i32]>,
    pub sub_character: Box<[i32]>,
    pub unlock_by_battlepass: Box<[i32]>,
    pub shop_need_hint_text: Box<str>,
    pub hp: f64,
    pub mana: f64,
    pub shield: f64,
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
    pub move_speed: f64,
    pub attack_speed: f64,
    pub hp_recovery: f64,
    pub mana_recovery: f64,
    pub skill: Box<[i32]>,
    pub death_effect: Box<[i32]>,
    pub equip: Box<[i32]>,
    pub affix: Box<[FConfigAffix]>,
    pub active: Box<str>,
    pub recommended_talent: Box<str>,
    pub equip_ratio: Box<[FConfigI32I32]>,
    pub server_affix: Box<str>,
    pub id: i32,
    pub gender: i32,
    pub hero_id: i32,
    pub effective: i32,
    pub order: i32,
    pub visible: i32,
    pub visible_new: i32,
    pub season_new: i32,
    pub fashion_default_head_id: i32,
    pub recommend: i32,
    pub hero_show_resources_id: i32,
    pub resource_id: i32,
    pub shop_id: i32,
    pub radius: i32,
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub skill_stone: i32,
    pub speciality: i32,
    pub drop_equip_ratio_id: i32,
    pub skill_repeat_vfx_id: i32,
    pub skill_repeat_ext_vfx_id: i32,
    pub hero_audio_id: i32,
    pub tree_mode: i32,
    pub free: bool,
}
impl ConfigHeroPurchaseRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.name = cur.read_str()?;
        self.gender = cur.read_i32()?;
        self.hero_id = cur.read_i32()?;
        self.effective = cur.read_i32()?;
        self.order = cur.read_i32()?;
        self.visible = cur.read_i32()?;
        self.visible_new = cur.read_i32()?;
        self.season_new = cur.read_i32()?;
        self.nickname = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.fashion_show_icon = cur.read_str()?;
        self.create_role_small_icon = cur.read_str()?;
        self.character_icon = cur.read_str()?;
        self.rank_portrait_icon = cur.read_str()?;
        self.fashion_default_head_id = cur.read_i32()?;
        self.recommend = cur.read_i32()?;
        self.show_character = cur.read_str()?;
        self.hero_show_resources_id = cur.read_i32()?;
        self.resource_id = cur.read_i32()?;
        self.default_fashions = cur.read_array_i32()?;
        self.sub_character = cur.read_array_i32()?;
        self.shop_id = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.unlock_by_battlepass = cur.read_array_i32()?;
        self.shop_need_hint_text = cur.read_str()?;
        self.free = cur.read_bool()?;
        self.radius = cur.read_i32()?;
        self.hp = cur.read_f64()?;
        self.mana = cur.read_f64()?;
        self.shield = cur.read_f64()?;
        self.ad = cur.read_f64()?;
        self.cd = cur.read_f64()?;
        self.fd = cur.read_f64()?;
        self.ld = cur.read_f64()?;
        self.td = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_3(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.armour = cur.read_f64()?;
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
        self.move_speed = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_4(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.attack_speed = cur.read_f64()?;
        self.hp_recovery = cur.read_f64()?;
        self.mana_recovery = cur.read_f64()?;
        self.strength = cur.read_i32()?;
        self.dexterity = cur.read_i32()?;
        self.intelligence = cur.read_i32()?;
        self.skill = cur.read_array_i32()?;
        self.skill_stone = cur.read_i32()?;
        self.death_effect = cur.read_array_i32()?;
        self.equip = cur.read_array_i32()?;
        self.affix = FConfigAffix::read_array(cur)?;
        self.speciality = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_5(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.active = cur.read_str()?;
        self.recommended_talent = cur.read_str()?;
        self.equip_ratio = FConfigI32I32::read_array(cur)?;
        self.drop_equip_ratio_id = cur.read_i32()?;
        self.skill_repeat_vfx_id = cur.read_i32()?;
        self.skill_repeat_ext_vfx_id = cur.read_i32()?;
        self.hero_audio_id = cur.read_i32()?;
        self.server_affix = cur.read_str()?;
        self.tree_mode = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigHeroPurchaseRow {
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
/// Typed `BinTable` for `config_hero_purchase_ChunkA0.bin`.
pub type ConfigHeroPurchaseTable = BinTable<i32, ConfigHeroPurchaseRow>;
/// Parses `config_hero_purchase_ChunkA0.bin` into a [`ConfigHeroPurchaseTable`].
pub fn parse_config_hero_purchase(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHeroPurchaseTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hero_resource_set_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHeroResourceSetRow {
    pub resource_id_list: Box<[i32]>,
    pub replaced_fashion_id: Box<[i32]>,
    pub hero_purchase_id: i32,
    pub set_id: i32,
    pub replaced_ui_showcase_animset: i32,
}
impl BinRow for ConfigHeroResourceSetRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.hero_purchase_id = cur.read_i32()?;
        self.set_id = cur.read_i32()?;
        self.resource_id_list = cur.read_array_i32()?;
        self.replaced_fashion_id = cur.read_array_i32()?;
        self.replaced_ui_showcase_animset = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hero_resource_set_ChunkA0.bin`.
pub type ConfigHeroResourceSetTable = BinTable<(i32, i32), ConfigHeroResourceSetRow>;
/// Parses `config_hero_resource_set_ChunkA0.bin` into a [`ConfigHeroResourceSetTable`].
pub fn parse_config_hero_resource_set(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHeroResourceSetTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hero_show_resources_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigHeroShowResourcesRow {
    pub anim_bp_res: Box<str>,
    pub anim_bp_sub_res: Box<str>,
    pub skeletal_res: Box<str>,
    pub role_icon: Box<str>,
    pub offset: f64,
    pub show_effects: Box<str>,
    pub show_action: Box<str>,
    pub sub_mesh_res: Box<[FConfigI32Str]>,
    pub show_action2: Box<str>,
    pub show_media: Box<str>,
    pub hero_transition: Box<str>,
    pub change_keyword: Box<str>,
    pub hero_character_action: Box<str>,
    pub hero_scene: Box<str>,
    pub hero_physics_res: Box<str>,
    pub id: i32,
    pub sequence: i32,
    pub main_resource_weapon: FConfigI32I32,
    pub offhand_resource_weapon: FConfigI32I32,
    pub hero_character_main_weapon: FConfigI32I32,
    pub hero_character_offhand_weapon: FConfigI32I32,
    pub hero_character_show_weapon: bool,
}
impl ConfigHeroShowResourcesRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.anim_bp_res = cur.read_str()?;
        self.anim_bp_sub_res = cur.read_str()?;
        self.skeletal_res = cur.read_str()?;
        self.sequence = cur.read_i32()?;
        self.role_icon = cur.read_str()?;
        self.main_resource_weapon = FConfigI32I32::read(cur)?;
        self.offhand_resource_weapon = FConfigI32I32::read(cur)?;
        self.hero_character_main_weapon = FConfigI32I32::read(cur)?;
        self.hero_character_offhand_weapon = FConfigI32I32::read(cur)?;
        self.offset = cur.read_f64()?;
        self.show_effects = cur.read_str()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.show_action = cur.read_str()?;
        self.sub_mesh_res = FConfigI32Str::read_array(cur)?;
        self.show_action2 = cur.read_str()?;
        self.show_media = cur.read_str()?;
        self.hero_transition = cur.read_str()?;
        self.change_keyword = cur.read_str()?;
        self.hero_character_action = cur.read_str()?;
        self.hero_character_show_weapon = cur.read_bool()?;
        self.hero_scene = cur.read_str()?;
        self.hero_physics_res = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigHeroShowResourcesRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hero_show_resources_ChunkA0.bin`.
pub type ConfigHeroShowResourcesTable = BinTable<i32, ConfigHeroShowResourcesRow>;
/// Parses `config_hero_show_resources_ChunkA0.bin` into a [`ConfigHeroShowResourcesTable`].
pub fn parse_config_hero_show_resources(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHeroShowResourcesTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_hero_speciality_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHeroSpecialityRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub name: Box<str>,
    pub desc: Box<str>,
    pub param: Box<str>,
    pub cast_ctrl_skills: FxHashMap<Box<str>, i32>,
    pub ui_bp_path: Box<str>,
    pub id: i32,
    pub ctrl_skill_prototype: i32,
    pub show_shortcut: i32,
}
impl BinRow for ConfigHeroSpecialityRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.name = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.param = cur.read_str()?;
        self.ctrl_skill_prototype = cur.read_i32()?;
        self.cast_ctrl_skills = cur.read_map_str_i32()?;
        self.show_shortcut = cur.read_i32()?;
        self.ui_bp_path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_hero_speciality_ChunkA0.bin`.
pub type ConfigHeroSpecialityTable = BinTable<i32, ConfigHeroSpecialityRow>;
/// Parses `config_hero_speciality_ChunkA0.bin` into a [`ConfigHeroSpecialityTable`].
pub fn parse_config_hero_speciality(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigHeroSpecialityTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_hero_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_hero" => Some(parse_config_hero(buf, scratch).map(|t| t.len())),
        "config_hero_character" => {
            Some(parse_config_hero_character(buf, scratch).map(|t| t.len()))
        }
        "config_hero_character_additional_affix" => {
            Some(
                parse_config_hero_character_additional_affix(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_hero_character_custom_slot_options" => {
            Some(
                parse_config_hero_character_custom_slot_options(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_hero_character_jewel_slot" => {
            Some(parse_config_hero_character_jewel_slot(buf, scratch).map(|t| t.len()))
        }
        "config_hero_character_tier" => {
            Some(parse_config_hero_character_tier(buf, scratch).map(|t| t.len()))
        }
        "config_hero_purchase" => {
            Some(parse_config_hero_purchase(buf, scratch).map(|t| t.len()))
        }
        "config_hero_resource_set" => {
            Some(parse_config_hero_resource_set(buf, scratch).map(|t| t.len()))
        }
        "config_hero_show_resources" => {
            Some(parse_config_hero_show_resources(buf, scratch).map(|t| t.len()))
        }
        "config_hero_speciality" => {
            Some(parse_config_hero_speciality(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

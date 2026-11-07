//! Auto-generated `LuaTable/DataCfg` types and parsers for the `hero` domain.
use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;
/// Row record for `config_bd_reference_tag.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigBdReferenceTagRow {
    pub title: Box<str>,
    pub _title: Box<str>,
    pub id: i8,
    pub order: u8,
}
impl DataCfgRow for ConfigBdReferenceTagRow {
    const TABLE_NAME: &'static str = "config_bd_reference_tag";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_bd_reference_tag.out`.
pub type ConfigBdReferenceTagTable = DataTable<u8, ConfigBdReferenceTagRow>;
/// Parses `config_bd_reference_tag.out` into a [`ConfigBdReferenceTagTable`].
pub fn parse_config_bd_reference_tag(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigBdReferenceTagTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_hero_character_custom_slot.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHeroCharacterCustomSlotRow {
    pub info_text: Box<str>,
    pub tag: Box<[u32]>,
    pub type_detail: Box<[u8]>,
    pub _info_text: Box<str>,
    pub hero_purchase_id: u16,
    pub addon_num: u8,
    pub id: i8,
    pub slot_id: u8,
    pub r#type: u8,
}
impl DataCfgRow for ConfigHeroCharacterCustomSlotRow {
    const TABLE_NAME: &'static str = "config_hero_character_custom_slot";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AddonNum" => {
                self.addon_num = ctx.decode_field(val, Self::TABLE_NAME, "AddonNum")?;
            }
            b"HeroPurchaseId" => {
                self.hero_purchase_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "HeroPurchaseId")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InfoText" => {
                self.info_text = ctx.decode_field(val, Self::TABLE_NAME, "InfoText")?;
            }
            b"SlotId" => {
                self.slot_id = ctx.decode_field(val, Self::TABLE_NAME, "SlotId")?;
            }
            b"Tag" => self.tag = ctx.decode_field(val, Self::TABLE_NAME, "Tag")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"TypeDetail" => {
                self.type_detail = ctx.decode_field(val, Self::TABLE_NAME, "TypeDetail")?;
            }
            b"_InfoText" => {
                self._info_text = ctx.decode_field(val, Self::TABLE_NAME, "_InfoText")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_hero_character_custom_slot.out`.
pub type ConfigHeroCharacterCustomSlotTable = DataTable<
    u8,
    ConfigHeroCharacterCustomSlotRow,
>;
/// Parses `config_hero_character_custom_slot.out` into a [`ConfigHeroCharacterCustomSlotTable`].
pub fn parse_config_hero_character_custom_slot(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigHeroCharacterCustomSlotTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_hero_character_tree_line.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigHeroCharacterTreeLineRow {
    pub angle: f64,
    pub img: Box<str>,
    pub mat: Box<str>,
    pub form_character_id: u32,
    pub to_character_id: u32,
    pub hero_purchase_id: u16,
    pub id: i8,
}
impl DataCfgRow for ConfigHeroCharacterTreeLineRow {
    const TABLE_NAME: &'static str = "config_hero_character_tree_line";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Angle" => self.angle = ctx.decode_field(val, Self::TABLE_NAME, "Angle")?,
            b"FormCharacterId" => {
                self.form_character_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "FormCharacterId")?;
            }
            b"HeroPurchaseId" => {
                self.hero_purchase_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "HeroPurchaseId")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Img" => self.img = ctx.decode_field(val, Self::TABLE_NAME, "Img")?,
            b"Mat" => self.mat = ctx.decode_field(val, Self::TABLE_NAME, "Mat")?,
            b"ToCharacterId" => {
                self.to_character_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "ToCharacterId")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_hero_character_tree_line.out`.
pub type ConfigHeroCharacterTreeLineTable = DataTable<
    u8,
    ConfigHeroCharacterTreeLineRow,
>;
/// Parses `config_hero_character_tree_line.out` into a [`ConfigHeroCharacterTreeLineTable`].
pub fn parse_config_hero_character_tree_line(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigHeroCharacterTreeLineTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_hero_item_base_attribute.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHeroItemBaseAttributeRow {
    pub attributes: Box<[u32]>,
    pub drop_level: Box<[u16]>,
    pub rarity: Box<[u8]>,
    pub type2: Box<[u8]>,
    pub id: i8,
}
impl DataCfgRow for ConfigHeroItemBaseAttributeRow {
    const TABLE_NAME: &'static str = "config_hero_item_base_attribute";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Attributes" => {
                self.attributes = ctx.decode_field(val, Self::TABLE_NAME, "Attributes")?;
            }
            b"DropLevel" => {
                self.drop_level = ctx.decode_field(val, Self::TABLE_NAME, "DropLevel")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_hero_item_base_attribute.out`.
pub type ConfigHeroItemBaseAttributeTable = DataTable<
    u8,
    ConfigHeroItemBaseAttributeRow,
>;
/// Parses `config_hero_item_base_attribute.out` into a [`ConfigHeroItemBaseAttributeTable`].
pub fn parse_config_hero_item_base_attribute(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigHeroItemBaseAttributeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_hero_item_enhance.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHeroItemEnhanceRow {
    pub desc: Box<str>,
    pub item_consume: FxHashMap<u16, u16>,
    pub item_level: Box<[u8]>,
    pub rarity: Box<[u8]>,
    pub type2: Box<[u8]>,
    pub _desc: Box<str>,
    pub id: i16,
    pub base_attr_tier: u8,
    pub character_level_mod: u8,
    pub current_enhance_level: u8,
    pub random_affix_mod: u8,
    pub target_enhance_level: u8,
}
impl DataCfgRow for ConfigHeroItemEnhanceRow {
    const TABLE_NAME: &'static str = "config_hero_item_enhance";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BaseAttrTier" => {
                self.base_attr_tier = ctx
                    .decode_field(val, Self::TABLE_NAME, "BaseAttrTier")?;
            }
            b"CharacterLevelMod" => {
                self.character_level_mod = ctx
                    .decode_field(val, Self::TABLE_NAME, "CharacterLevelMod")?;
            }
            b"CurrentEnhanceLevel" => {
                self.current_enhance_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "CurrentEnhanceLevel")?;
            }
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemConsume" => {
                self.item_consume = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemConsume")?;
            }
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?;
            }
            b"RandomAffixMod" => {
                self.random_affix_mod = ctx
                    .decode_field(val, Self::TABLE_NAME, "RandomAffixMod")?;
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"TargetEnhanceLevel" => {
                self.target_enhance_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetEnhanceLevel")?;
            }
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_hero_item_enhance.out`.
pub type ConfigHeroItemEnhanceTable = DataTable<u8, ConfigHeroItemEnhanceRow>;
/// Parses `config_hero_item_enhance.out` into a [`ConfigHeroItemEnhanceTable`].
pub fn parse_config_hero_item_enhance(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigHeroItemEnhanceTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Entry for `config_hero_item_reroll.AffixTier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeroItemRerollAffixTierEntry {
    pub weight: u16,
    pub tier: u8,
}
impl FromEvalVal for HeroItemRerollAffixTierEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (tier, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { weight, tier })
    }
}
/// Row record for `config_hero_item_reroll.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHeroItemRerollRow {
    pub affix_tier: Box<[HeroItemRerollAffixTierEntry]>,
    pub item_consume: FxHashMap<u16, u8>,
    pub item_consume_awaken: FxHashMap<u16, u8>,
    pub item_level: Box<[u8]>,
    pub rarity: Box<[u8]>,
    pub type2: Box<[u8]>,
    pub id: i8,
}
impl DataCfgRow for ConfigHeroItemRerollRow {
    const TABLE_NAME: &'static str = "config_hero_item_reroll";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AffixTier" => {
                self.affix_tier = ctx.decode_field(val, Self::TABLE_NAME, "AffixTier")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemConsume" => {
                self.item_consume = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemConsume")?;
            }
            b"ItemConsumeAwaken" => {
                self.item_consume_awaken = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemConsumeAwaken")?;
            }
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?;
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_hero_item_reroll.out`.
pub type ConfigHeroItemRerollTable = DataTable<u8, ConfigHeroItemRerollRow>;
/// Parses `config_hero_item_reroll.out` into a [`ConfigHeroItemRerollTable`].
pub fn parse_config_hero_item_reroll(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigHeroItemRerollTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_heroranking.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHerorankingRow {
    pub icon: Box<str>,
    pub name: Box<str>,
    pub _name: Box<str>,
    pub id: i8,
    pub max_num: u8,
    pub node_id: u8,
    pub show: bool,
}
impl DataCfgRow for ConfigHerorankingRow {
    const TABLE_NAME: &'static str = "config_heroranking";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MaxNum" => {
                self.max_num = ctx.decode_field(val, Self::TABLE_NAME, "MaxNum")?;
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"NodeId" => {
                self.node_id = ctx.decode_field(val, Self::TABLE_NAME, "NodeId")?;
            }
            b"Show" => self.show = ctx.decode_field(val, Self::TABLE_NAME, "Show")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_heroranking.out`.
pub type ConfigHerorankingTable = DataTable<u8, ConfigHerorankingRow>;
/// Parses `config_heroranking.out` into a [`ConfigHerorankingTable`].
pub fn parse_config_heroranking(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigHerorankingTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_heroranking_equip_tag.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHerorankingEquipTagRow {
    pub description: Box<str>,
    pub _description: Box<str>,
    pub id: i8,
    pub type1: u8,
    pub type2: u8,
    pub type3: u8,
    pub type4: u8,
}
impl DataCfgRow for ConfigHerorankingEquipTagRow {
    const TABLE_NAME: &'static str = "config_heroranking_equip_tag";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Description" => {
                self.description = ctx
                    .decode_field(val, Self::TABLE_NAME, "Description")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            b"_Description" => {
                self._description = ctx
                    .decode_field(val, Self::TABLE_NAME, "_Description")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_heroranking_equip_tag.out`.
pub type ConfigHerorankingEquipTagTable = DataTable<u8, ConfigHerorankingEquipTagRow>;
/// Parses `config_heroranking_equip_tag.out` into a [`ConfigHerorankingEquipTagTable`].
pub fn parse_config_heroranking_equip_tag(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigHerorankingEquipTagTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
pub(crate) fn verify_hero_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_bd_reference_tag" => {
            Some(parse_config_bd_reference_tag(buf, scratch).map(|t| t.len()))
        }
        "config_hero_character_custom_slot" => {
            Some(parse_config_hero_character_custom_slot(buf, scratch).map(|t| t.len()))
        }
        "config_hero_character_tree_line" => {
            Some(parse_config_hero_character_tree_line(buf, scratch).map(|t| t.len()))
        }
        "config_hero_item_base_attribute" => {
            Some(parse_config_hero_item_base_attribute(buf, scratch).map(|t| t.len()))
        }
        "config_hero_item_enhance" => {
            Some(parse_config_hero_item_enhance(buf, scratch).map(|t| t.len()))
        }
        "config_hero_item_reroll" => {
            Some(parse_config_hero_item_reroll(buf, scratch).map(|t| t.len()))
        }
        "config_heroranking" => {
            Some(parse_config_heroranking(buf, scratch).map(|t| t.len()))
        }
        "config_heroranking_equip_tag" => {
            Some(parse_config_heroranking_equip_tag(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

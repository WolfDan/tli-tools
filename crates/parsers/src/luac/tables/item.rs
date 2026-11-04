//! Auto-generated `LuaTable/DataCfg` types and parsers for the `item` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;

/// Row record for `config_confirm_buy_item_text.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigConfirmBuyItemTextRow {
    pub detail_text: Box<str>,
    pub id: i8,
    pub item: Box<str>,
    pub remain_text: Box<str>,
    pub tips_text: Box<str>,
    pub _detail_text: Box<str>,
    pub _remain_text: Box<str>,
    pub _tips_text: Box<str>,
}

impl DataCfgRow for ConfigConfirmBuyItemTextRow {
    const TABLE_NAME: &'static str = "config_confirm_buy_item_text";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DetailText" => {
                self.detail_text = ctx.decode_field(val, Self::TABLE_NAME, "DetailText")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Item" => self.item = ctx.decode_field(val, Self::TABLE_NAME, "Item")?,
            b"RemainText" => {
                self.remain_text = ctx.decode_field(val, Self::TABLE_NAME, "RemainText")?
            }
            b"TipsText" => self.tips_text = ctx.decode_field(val, Self::TABLE_NAME, "TipsText")?,
            b"_DetailText" => {
                self._detail_text = ctx.decode_field(val, Self::TABLE_NAME, "_DetailText")?
            }
            b"_RemainText" => {
                self._remain_text = ctx.decode_field(val, Self::TABLE_NAME, "_RemainText")?
            }
            b"_TipsText" => {
                self._tips_text = ctx.decode_field(val, Self::TABLE_NAME, "_TipsText")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_confirm_buy_item_text.out`.
pub type ConfigConfirmBuyItemTextTable = DataTable<u8, ConfigConfirmBuyItemTextRow>;

/// Parses `config_confirm_buy_item_text.out` into a [`ConfigConfirmBuyItemTextTable`].
pub fn parse_config_confirm_buy_item_text(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigConfirmBuyItemTextTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_consum.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigConsumRow {
    pub icon: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigConsumRow {
    const TABLE_NAME: &'static str = "config_consum";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_consum.out`.
pub type ConfigConsumTable = DataTable<u8, ConfigConsumRow>;

/// Parses `config_consum.out` into a [`ConfigConsumTable`].
pub fn parse_config_consum(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigConsumTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_convert.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigConvertRow {
    pub convert_id: u8,
    pub convert_number: u8,
    pub convert_type: u8,
    pub id: i8,
    pub ignore_confirm: u8,
    pub target_id: u8,
    pub target_number: u8,
    pub target_type: u8,
}

impl DataCfgRow for ConfigConvertRow {
    const TABLE_NAME: &'static str = "config_convert";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ConvertId" => {
                self.convert_id = ctx.decode_field(val, Self::TABLE_NAME, "ConvertId")?
            }
            b"ConvertNumber" => {
                self.convert_number = ctx.decode_field(val, Self::TABLE_NAME, "ConvertNumber")?
            }
            b"ConvertType" => {
                self.convert_type = ctx.decode_field(val, Self::TABLE_NAME, "ConvertType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IgnoreConfirm" => {
                self.ignore_confirm = ctx.decode_field(val, Self::TABLE_NAME, "IgnoreConfirm")?
            }
            b"TargetId" => self.target_id = ctx.decode_field(val, Self::TABLE_NAME, "TargetId")?,
            b"TargetNumber" => {
                self.target_number = ctx.decode_field(val, Self::TABLE_NAME, "TargetNumber")?
            }
            b"TargetType" => {
                self.target_type = ctx.decode_field(val, Self::TABLE_NAME, "TargetType")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_convert.out`.
pub type ConfigConvertTable = DataTable<u8, ConfigConvertRow>;

/// Parses `config_convert.out` into a [`ConfigConvertTable`].
pub fn parse_config_convert(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigConvertTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_experience.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigExperienceRow {
    pub exp: u64,
    pub experience_lost: u8,
    pub id: i8,
    pub revival_coin_cost: u8,
}

impl DataCfgRow for ConfigExperienceRow {
    const TABLE_NAME: &'static str = "config_experience";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Exp" => self.exp = ctx.decode_field(val, Self::TABLE_NAME, "Exp")?,
            b"ExperienceLost" => {
                self.experience_lost = ctx.decode_field(val, Self::TABLE_NAME, "ExperienceLost")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"RevivalCoinCost" => {
                self.revival_coin_cost =
                    ctx.decode_field(val, Self::TABLE_NAME, "RevivalCoinCost")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_experience.out`.
pub type ConfigExperienceTable = DataTable<u8, ConfigExperienceRow>;

/// Parses `config_experience.out` into a [`ConfigExperienceTable`].
pub fn parse_config_experience(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigExperienceTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_item_binding_mapping.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemBindingMappingRow {
    pub binding_item_id: u32,
    pub id: i8,
    pub item_id: u32,
}

impl DataCfgRow for ConfigItemBindingMappingRow {
    const TABLE_NAME: &'static str = "config_item_binding_mapping";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BindingItemId" => {
                self.binding_item_id = ctx.decode_field(val, Self::TABLE_NAME, "BindingItemId")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemId" => self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_item_binding_mapping.out`.
pub type ConfigItemBindingMappingTable = DataTable<u8, ConfigItemBindingMappingRow>;

/// Parses `config_item_binding_mapping.out` into a [`ConfigItemBindingMappingTable`].
pub fn parse_config_item_binding_mapping(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigItemBindingMappingTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_item_gold_exchange.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemGoldExchangeRow {
    pub hero_purchase: Box<[u16]>,
    pub id: i32,
    pub item_gold_id: u32,
    pub level: u8,
    pub name: Box<str>,
    pub num: u8,
    pub order: u16,
    pub valid: u8,
    pub wish_level: u16,
}

impl DataCfgRow for ConfigItemGoldExchangeRow {
    const TABLE_NAME: &'static str = "config_item_gold_exchange";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"HeroPurchase" => {
                self.hero_purchase = ctx.decode_field(val, Self::TABLE_NAME, "HeroPurchase")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemGoldId" => {
                self.item_gold_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemGoldId")?
            }
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Num" => self.num = ctx.decode_field(val, Self::TABLE_NAME, "Num")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"Valid" => self.valid = ctx.decode_field(val, Self::TABLE_NAME, "Valid")?,
            b"WishLevel" => {
                self.wish_level = ctx.decode_field(val, Self::TABLE_NAME, "WishLevel")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_item_gold_exchange.out`.
pub type ConfigItemGoldExchangeTable = DataTable<u32, ConfigItemGoldExchangeRow>;

/// Parses `config_item_gold_exchange.out` into a [`ConfigItemGoldExchangeTable`].
pub fn parse_config_item_gold_exchange(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigItemGoldExchangeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_item_gold_random_affix_config.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemGoldRandomAffixConfigRow {
    pub affix_num: FxHashMap<u8, Box<[u8]>>,
    pub id: i16,
    pub item_level: Box<[u16]>,
    pub rarity: u8,
    pub slot_num: FxHashMap<u8, u16>,
    pub slot_type: Box<[u8]>,
    pub type1: u8,
}

impl DataCfgRow for ConfigItemGoldRandomAffixConfigRow {
    const TABLE_NAME: &'static str = "config_item_gold_random_affix_config";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AffixNum" => self.affix_num = ctx.decode_field(val, Self::TABLE_NAME, "AffixNum")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"SlotNum" => self.slot_num = ctx.decode_field(val, Self::TABLE_NAME, "SlotNum")?,
            b"SlotType" => self.slot_type = ctx.decode_field(val, Self::TABLE_NAME, "SlotType")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_item_gold_random_affix_config.out`.
pub type ConfigItemGoldRandomAffixConfigTable = DataTable<u8, ConfigItemGoldRandomAffixConfigRow>;

/// Parses `config_item_gold_random_affix_config.out` into a [`ConfigItemGoldRandomAffixConfigTable`].
pub fn parse_config_item_gold_random_affix_config(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigItemGoldRandomAffixConfigTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_item_rarity_mapping.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemRarityMappingRow {
    pub id: i32,
    pub item_gold_slot_type: Box<[u8]>,
    pub rarity: u8,
    pub rarity_show: u8,
    pub type1: u8,
    pub type1_arr: Box<[u8]>,
}

impl DataCfgRow for ConfigItemRarityMappingRow {
    const TABLE_NAME: &'static str = "config_item_rarity_mapping";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemGoldSlotType" => {
                self.item_gold_slot_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "ItemGoldSlotType")?
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"RarityShow" => {
                self.rarity_show = ctx.decode_field(val, Self::TABLE_NAME, "RarityShow")?
            }
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type1Arr" => self.type1_arr = ctx.decode_field(val, Self::TABLE_NAME, "Type1Arr")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_item_rarity_mapping.out`.
pub type ConfigItemRarityMappingTable = DataTable<u32, ConfigItemRarityMappingRow>;

/// Parses `config_item_rarity_mapping.out` into a [`ConfigItemRarityMappingTable`].
pub fn parse_config_item_rarity_mapping(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigItemRarityMappingTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_item_s13_organ.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemS13OrganRow {
    pub affix_range: Box<[u8]>,
    pub available_item_level: Box<[u8]>,
    pub desc: Box<str>,
    pub id: i32,
    pub item_gold_id: u32,
    pub item_gold_option_number: u8,
    pub organ_type: u8,
    pub s13_equipment_type4: Box<[u8]>,
    pub show_order: u16,
    pub special_comsume: FxHashMap<u32, u8>,
    pub target_affix_group: Box<[u32]>,
    pub target_affix_library: u8,
    pub _desc: Box<str>,
}

impl ConfigItemS13OrganRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AffixRange" => {
                self.affix_range = ctx.decode_field(val, Self::TABLE_NAME, "AffixRange")?
            }
            b"AvailableItemLevel" => {
                self.available_item_level =
                    ctx.decode_field(val, Self::TABLE_NAME, "AvailableItemLevel")?
            }
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemGoldId" => {
                self.item_gold_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemGoldId")?
            }
            b"ItemGoldOptionNumber" => {
                self.item_gold_option_number =
                    ctx.decode_field(val, Self::TABLE_NAME, "ItemGoldOptionNumber")?
            }
            b"OrganType" => {
                self.organ_type = ctx.decode_field(val, Self::TABLE_NAME, "OrganType")?
            }
            b"S13EquipmentType4" => {
                self.s13_equipment_type4 =
                    ctx.decode_field(val, Self::TABLE_NAME, "S13EquipmentType4")?
            }
            b"ShowOrder" => {
                self.show_order = ctx.decode_field(val, Self::TABLE_NAME, "ShowOrder")?
            }
            b"SpecialComsume" => {
                self.special_comsume = ctx.decode_field(val, Self::TABLE_NAME, "SpecialComsume")?
            }
            b"TargetAffixGroup" => {
                self.target_affix_group =
                    ctx.decode_field(val, Self::TABLE_NAME, "TargetAffixGroup")?
            }
            b"TargetAffixLibrary" => {
                self.target_affix_library =
                    ctx.decode_field(val, Self::TABLE_NAME, "TargetAffixLibrary")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_field_1(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigItemS13OrganRow {
    const TABLE_NAME: &'static str = "config_item_s13_organ";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        if self.set_field_0(ctx, field, val)? || self.set_field_1(ctx, field, val)? {
            return Ok(());
        }
        Err(ctx.unknown_field(Self::TABLE_NAME, field))
    }
}

/// Typed `DataTable` for `config_item_s13_organ.out`.
pub type ConfigItemS13OrganTable = DataTable<u32, ConfigItemS13OrganRow>;

/// Parses `config_item_s13_organ.out` into a [`ConfigItemS13OrganTable`].
pub fn parse_config_item_s13_organ(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigItemS13OrganTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_item_stream_mode.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemStreamModeRow {
    pub icon_stream_mod: Box<str>,
    pub item_base_id: i32,
}

impl DataCfgRow for ConfigItemStreamModeRow {
    const TABLE_NAME: &'static str = "config_item_stream_mode";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"IconStreamMod" => {
                self.icon_stream_mod = ctx.decode_field(val, Self::TABLE_NAME, "IconStreamMod")?
            }
            b"ItemBaseId" => {
                self.item_base_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemBaseId")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_item_stream_mode.out`.
pub type ConfigItemStreamModeTable = DataTable<u32, ConfigItemStreamModeRow>;

/// Parses `config_item_stream_mode.out` into a [`ConfigItemStreamModeTable`].
pub fn parse_config_item_stream_mode(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigItemStreamModeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_item_talent_block_gold.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemTalentBlockGoldRow {
    pub id: i16,
    pub influence: FxHashMap<u16, u8>,
    pub influence_description: Box<str>,
    pub limit: u8,
    pub talent_block_shape_ids: FxHashMap<u16, u8>,
    pub talent_description1: Box<str>,
    pub talent_description2: Box<str>,
    pub talent_description3: Box<str>,
    pub talent_description4: Box<str>,
    pub talent_description5: Box<str>,
    pub talent_description6: Box<str>,
    pub talent_description7: Box<str>,
    pub talent_description8: Box<str>,
    pub talent_pool1: FxHashMap<u8, u8>,
    pub talent_pool2: FxHashMap<u8, u8>,
    pub talent_pool3: FxHashMap<u8, u8>,
    pub talent_pool4: FxHashMap<u8, u8>,
    pub talent_pool5: [u8; 0],
    pub talent_pool6: [u8; 0],
    pub talent_pool7: [u8; 0],
    pub talent_pool8: [u8; 0],
    pub _influence_description: Box<str>,
    pub _talent_description1: Box<str>,
    pub _talent_description2: Box<str>,
    pub _talent_description3: Box<str>,
    pub _talent_description4: Box<str>,
    pub _talent_description5: Box<str>,
    pub _talent_description6: Box<str>,
    pub _talent_description7: Box<str>,
    pub _talent_description8: Box<str>,
}

impl ConfigItemTalentBlockGoldRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Influence" => {
                self.influence = ctx.decode_field(val, Self::TABLE_NAME, "Influence")?
            }
            b"InfluenceDescription" => {
                self.influence_description =
                    ctx.decode_field(val, Self::TABLE_NAME, "InfluenceDescription")?
            }
            b"Limit" => self.limit = ctx.decode_field(val, Self::TABLE_NAME, "Limit")?,
            b"TalentBlockShapeIds" => {
                self.talent_block_shape_ids =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentBlockShapeIds")?
            }
            b"TalentDescription1" => {
                self.talent_description1 =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentDescription1")?
            }
            b"TalentDescription2" => {
                self.talent_description2 =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentDescription2")?
            }
            b"TalentDescription3" => {
                self.talent_description3 =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentDescription3")?
            }
            b"TalentDescription4" => {
                self.talent_description4 =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentDescription4")?
            }
            b"TalentDescription5" => {
                self.talent_description5 =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentDescription5")?
            }
            b"TalentDescription6" => {
                self.talent_description6 =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentDescription6")?
            }
            b"TalentDescription7" => {
                self.talent_description7 =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentDescription7")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_field_1(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"TalentDescription8" => {
                self.talent_description8 =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentDescription8")?
            }
            b"TalentPool1" => {
                self.talent_pool1 = ctx.decode_field(val, Self::TABLE_NAME, "TalentPool1")?
            }
            b"TalentPool2" => {
                self.talent_pool2 = ctx.decode_field(val, Self::TABLE_NAME, "TalentPool2")?
            }
            b"TalentPool3" => {
                self.talent_pool3 = ctx.decode_field(val, Self::TABLE_NAME, "TalentPool3")?
            }
            b"TalentPool4" => {
                self.talent_pool4 = ctx.decode_field(val, Self::TABLE_NAME, "TalentPool4")?
            }
            b"TalentPool5" => {
                self.talent_pool5 = ctx.decode_field(val, Self::TABLE_NAME, "TalentPool5")?
            }
            b"TalentPool6" => {
                self.talent_pool6 = ctx.decode_field(val, Self::TABLE_NAME, "TalentPool6")?
            }
            b"TalentPool7" => {
                self.talent_pool7 = ctx.decode_field(val, Self::TABLE_NAME, "TalentPool7")?
            }
            b"TalentPool8" => {
                self.talent_pool8 = ctx.decode_field(val, Self::TABLE_NAME, "TalentPool8")?
            }
            b"_InfluenceDescription" => {
                self._influence_description =
                    ctx.decode_field(val, Self::TABLE_NAME, "_InfluenceDescription")?
            }
            b"_TalentDescription1" => {
                self._talent_description1 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_TalentDescription1")?
            }
            b"_TalentDescription2" => {
                self._talent_description2 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_TalentDescription2")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_field_2(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"_TalentDescription3" => {
                self._talent_description3 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_TalentDescription3")?
            }
            b"_TalentDescription4" => {
                self._talent_description4 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_TalentDescription4")?
            }
            b"_TalentDescription5" => {
                self._talent_description5 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_TalentDescription5")?
            }
            b"_TalentDescription6" => {
                self._talent_description6 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_TalentDescription6")?
            }
            b"_TalentDescription7" => {
                self._talent_description7 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_TalentDescription7")?
            }
            b"_TalentDescription8" => {
                self._talent_description8 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_TalentDescription8")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigItemTalentBlockGoldRow {
    const TABLE_NAME: &'static str = "config_item_talent_block_gold";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        if self.set_field_0(ctx, field, val)?
            || self.set_field_1(ctx, field, val)?
            || self.set_field_2(ctx, field, val)?
        {
            return Ok(());
        }
        Err(ctx.unknown_field(Self::TABLE_NAME, field))
    }
}

/// Typed `DataTable` for `config_item_talent_block_gold.out`.
pub type ConfigItemTalentBlockGoldTable = DataTable<u16, ConfigItemTalentBlockGoldRow>;

/// Parses `config_item_talent_block_gold.out` into a [`ConfigItemTalentBlockGoldTable`].
pub fn parse_config_item_talent_block_gold(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigItemTalentBlockGoldTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_item_target_function.TargetFunction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemTargetFunctionTargetFunction {
    pub function_id: u16,
    pub enabled: u8,
}

impl FromEvalVal for ItemTargetFunctionTargetFunction {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (function_id, enabled) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            function_id,
            enabled,
        })
    }
}

/// Row record for `config_item_target_function.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemTargetFunctionRow {
    pub id: i32,
    pub target_function: ItemTargetFunctionTargetFunction,
}

impl DataCfgRow for ConfigItemTargetFunctionRow {
    const TABLE_NAME: &'static str = "config_item_target_function";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"TargetFunction" => {
                self.target_function = ctx.decode_field(val, Self::TABLE_NAME, "TargetFunction")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_item_target_function.out`.
pub type ConfigItemTargetFunctionTable = DataTable<u32, ConfigItemTargetFunctionRow>;

/// Parses `config_item_target_function.out` into a [`ConfigItemTargetFunctionTable`].
pub fn parse_config_item_target_function(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigItemTargetFunctionTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_item_tattoo.UpModifier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemTattooUpModifierEntry {
    pub slot: u8,
    pub modifier_id: u32,
}

impl FromEvalVal for ItemTattooUpModifierEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (slot, modifier_id) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { slot, modifier_id })
    }
}

/// Row record for `config_item_tattoo.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemTattooRow {
    pub active_num: u8,
    pub add_modifier: [u8; 0],
    pub add_modifier_base: [u8; 0],
    pub description_middle: Box<str>,
    pub description_small: Box<str>,
    pub extra_point_num: FxHashMap<u8, u16>,
    pub id: i32,
    pub modifier: Box<[u32]>,
    pub modifier_group: [u8; 0],
    pub modifier_num: [u8; 0],
    pub servant_cluster: Box<[Box<[u8]>]>,
    pub server_affix: Box<str>,
    pub tattoo_group: u8,
    pub tattoo_size: u8,
    pub tattoo_type: u16,
    pub up_modifier: Box<[ItemTattooUpModifierEntry]>,
    pub _description_middle: Box<str>,
    pub _description_small: Box<str>,
}

impl ConfigItemTattooRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"ActiveNum" => {
                self.active_num = ctx.decode_field(val, Self::TABLE_NAME, "ActiveNum")?
            }
            b"AddModifier" => {
                self.add_modifier = ctx.decode_field(val, Self::TABLE_NAME, "AddModifier")?
            }
            b"AddModifierBase" => {
                self.add_modifier_base =
                    ctx.decode_field(val, Self::TABLE_NAME, "AddModifierBase")?
            }
            b"DescriptionMiddle" => {
                self.description_middle =
                    ctx.decode_field(val, Self::TABLE_NAME, "DescriptionMiddle")?
            }
            b"DescriptionSmall" => {
                self.description_small =
                    ctx.decode_field(val, Self::TABLE_NAME, "DescriptionSmall")?
            }
            b"ExtraPointNum" => {
                self.extra_point_num = ctx.decode_field(val, Self::TABLE_NAME, "ExtraPointNum")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Modifier" => self.modifier = ctx.decode_field(val, Self::TABLE_NAME, "Modifier")?,
            b"ModifierGroup" => {
                self.modifier_group = ctx.decode_field(val, Self::TABLE_NAME, "ModifierGroup")?
            }
            b"ModifierNum" => {
                self.modifier_num = ctx.decode_field(val, Self::TABLE_NAME, "ModifierNum")?
            }
            b"ServantCluster" => {
                self.servant_cluster = ctx.decode_field(val, Self::TABLE_NAME, "ServantCluster")?
            }
            b"ServerAffix" => {
                self.server_affix = ctx.decode_field(val, Self::TABLE_NAME, "ServerAffix")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_field_1(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"TattooGroup" => {
                self.tattoo_group = ctx.decode_field(val, Self::TABLE_NAME, "TattooGroup")?
            }
            b"TattooSize" => {
                self.tattoo_size = ctx.decode_field(val, Self::TABLE_NAME, "TattooSize")?
            }
            b"TattooType" => {
                self.tattoo_type = ctx.decode_field(val, Self::TABLE_NAME, "TattooType")?
            }
            b"UpModifier" => {
                self.up_modifier = ctx.decode_field(val, Self::TABLE_NAME, "UpModifier")?
            }
            b"_DescriptionMiddle" => {
                self._description_middle =
                    ctx.decode_field(val, Self::TABLE_NAME, "_DescriptionMiddle")?
            }
            b"_DescriptionSmall" => {
                self._description_small =
                    ctx.decode_field(val, Self::TABLE_NAME, "_DescriptionSmall")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigItemTattooRow {
    const TABLE_NAME: &'static str = "config_item_tattoo";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        if self.set_field_0(ctx, field, val)? || self.set_field_1(ctx, field, val)? {
            return Ok(());
        }
        Err(ctx.unknown_field(Self::TABLE_NAME, field))
    }
}

/// Typed `DataTable` for `config_item_tattoo.out`.
pub type ConfigItemTattooTable = DataTable<u32, ConfigItemTattooRow>;

/// Parses `config_item_tattoo.out` into a [`ConfigItemTattooTable`].
pub fn parse_config_item_tattoo(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigItemTattooTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_item_type_list.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigItemTypeListRow {
    pub description02: Box<str>,
    pub icon: Box<str>,
    pub icon02: Box<str>,
    pub id: i32,
    pub islegend: u8,
    pub item_type: [u8; 0],
    pub menu_id: u8,
    pub name: Box<str>,
    pub node_id: u16,
    pub sort: u32,
    pub _description02: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigItemTypeListRow {
    const TABLE_NAME: &'static str = "config_item_type_list";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Description02" => {
                self.description02 = ctx.decode_field(val, Self::TABLE_NAME, "Description02")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Icon02" => self.icon02 = ctx.decode_field(val, Self::TABLE_NAME, "Icon02")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Islegend" => self.islegend = ctx.decode_field(val, Self::TABLE_NAME, "Islegend")?,
            b"ItemType" => self.item_type = ctx.decode_field(val, Self::TABLE_NAME, "ItemType")?,
            b"MenuId" => self.menu_id = ctx.decode_field(val, Self::TABLE_NAME, "MenuId")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"NodeId" => self.node_id = ctx.decode_field(val, Self::TABLE_NAME, "NodeId")?,
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"_Description02" => {
                self._description02 = ctx.decode_field(val, Self::TABLE_NAME, "_Description02")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_item_type_list.out`.
pub type ConfigItemTypeListTable = DataTable<u32, ConfigItemTypeListRow>;

/// Parses `config_item_type_list.out` into a [`ConfigItemTypeListTable`].
pub fn parse_config_item_type_list(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigItemTypeListTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_money.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMoneyRow {
    pub c_type: Box<str>,
    pub color: u8,
    pub data_scope: u8,
    pub description: Box<str>,
    pub detail: Box<str>,
    pub icon: Box<str>,
    pub icon_pick_card_list_show: Box<str>,
    pub icon_pick_card_result: Box<str>,
    pub icon_pick_card_show: Box<str>,
    pub icon_show: Box<str>,
    pub id: i32,
    pub name: Box<str>,
    pub plus_page: Box<str>,
    pub plus_sign: u8,
    pub show_inpets: u8,
    pub show_only_teamowner: u8,
    pub _description: Box<str>,
    pub _name: Box<str>,
}

impl ConfigMoneyRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"CType" => self.c_type = ctx.decode_field(val, Self::TABLE_NAME, "CType")?,
            b"Color" => self.color = ctx.decode_field(val, Self::TABLE_NAME, "Color")?,
            b"DataScope" => {
                self.data_scope = ctx.decode_field(val, Self::TABLE_NAME, "DataScope")?
            }
            b"Description" => {
                self.description = ctx.decode_field(val, Self::TABLE_NAME, "Description")?
            }
            b"Detail" => self.detail = ctx.decode_field(val, Self::TABLE_NAME, "Detail")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"IconPickCardListShow" => {
                self.icon_pick_card_list_show =
                    ctx.decode_field(val, Self::TABLE_NAME, "IconPickCardListShow")?
            }
            b"IconPickCardResult" => {
                self.icon_pick_card_result =
                    ctx.decode_field(val, Self::TABLE_NAME, "IconPickCardResult")?
            }
            b"IconPickCardShow" => {
                self.icon_pick_card_show =
                    ctx.decode_field(val, Self::TABLE_NAME, "IconPickCardShow")?
            }
            b"IconShow" => self.icon_show = ctx.decode_field(val, Self::TABLE_NAME, "IconShow")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_field_1(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"PlusPage" => self.plus_page = ctx.decode_field(val, Self::TABLE_NAME, "PlusPage")?,
            b"PlusSign" => self.plus_sign = ctx.decode_field(val, Self::TABLE_NAME, "PlusSign")?,
            b"ShowInpets" => {
                self.show_inpets = ctx.decode_field(val, Self::TABLE_NAME, "ShowInpets")?
            }
            b"ShowOnlyTeamowner" => {
                self.show_only_teamowner =
                    ctx.decode_field(val, Self::TABLE_NAME, "ShowOnlyTeamowner")?
            }
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigMoneyRow {
    const TABLE_NAME: &'static str = "config_money";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        if self.set_field_0(ctx, field, val)? || self.set_field_1(ctx, field, val)? {
            return Ok(());
        }
        Err(ctx.unknown_field(Self::TABLE_NAME, field))
    }
}

/// Typed `DataTable` for `config_money.out`.
pub type ConfigMoneyTable = DataTable<u32, ConfigMoneyRow>;

/// Parses `config_money.out` into a [`ConfigMoneyTable`].
pub fn parse_config_money(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMoneyTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_reward.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRewardRow {
    pub des: Box<str>,
    pub direct_get: bool,
    pub group_id: Box<[u32]>,
    pub hero_id: u16,
    pub id: i32,
    pub num: u8,
    pub requirement: Box<str>,
    pub reward: Box<str>,
    pub r#type: u8,
}

impl DataCfgRow for ConfigRewardRow {
    const TABLE_NAME: &'static str = "config_reward";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"DirectGet" => {
                self.direct_get = ctx.decode_field(val, Self::TABLE_NAME, "DirectGet")?
            }
            b"GroupId" => self.group_id = ctx.decode_field(val, Self::TABLE_NAME, "GroupId")?,
            b"HeroId" => self.hero_id = ctx.decode_field(val, Self::TABLE_NAME, "HeroId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Num" => self.num = ctx.decode_field(val, Self::TABLE_NAME, "Num")?,
            b"Requirement" => {
                self.requirement = ctx.decode_field(val, Self::TABLE_NAME, "Requirement")?
            }
            b"Reward" => self.reward = ctx.decode_field(val, Self::TABLE_NAME, "Reward")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_reward.out`.
pub type ConfigRewardTable = DataTable<u32, ConfigRewardRow>;

/// Parses `config_reward.out` into a [`ConfigRewardTable`].
pub fn parse_config_reward(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigRewardTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_specified_item.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpecifiedItemRow {
    pub cost: u16,
    pub id: i32,
    pub level: u8,
    pub prefix1: u32,
    pub prefix2: u32,
    pub prefix3: u32,
    pub prefix_hole: u8,
    pub rarity: u8,
    pub suffix1: u32,
    pub suffix2: u32,
    pub suffix3: u32,
    pub suffix_hole: u8,
}

impl DataCfgRow for ConfigSpecifiedItemRow {
    const TABLE_NAME: &'static str = "config_specified_item";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Prefix1" => self.prefix1 = ctx.decode_field(val, Self::TABLE_NAME, "Prefix1")?,
            b"Prefix2" => self.prefix2 = ctx.decode_field(val, Self::TABLE_NAME, "Prefix2")?,
            b"Prefix3" => self.prefix3 = ctx.decode_field(val, Self::TABLE_NAME, "Prefix3")?,
            b"PrefixHole" => {
                self.prefix_hole = ctx.decode_field(val, Self::TABLE_NAME, "PrefixHole")?
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Suffix1" => self.suffix1 = ctx.decode_field(val, Self::TABLE_NAME, "Suffix1")?,
            b"Suffix2" => self.suffix2 = ctx.decode_field(val, Self::TABLE_NAME, "Suffix2")?,
            b"Suffix3" => self.suffix3 = ctx.decode_field(val, Self::TABLE_NAME, "Suffix3")?,
            b"SuffixHole" => {
                self.suffix_hole = ctx.decode_field(val, Self::TABLE_NAME, "SuffixHole")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_specified_item.out`.
pub type ConfigSpecifiedItemTable = DataTable<u32, ConfigSpecifiedItemRow>;

/// Parses `config_specified_item.out` into a [`ConfigSpecifiedItemTable`].
pub fn parse_config_specified_item(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpecifiedItemTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

fn verify_item_chunk_0(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_confirm_buy_item_text" => {
            Some(parse_config_confirm_buy_item_text(buf, scratch).map(|t| t.len()))
        }
        "config_consum" => Some(parse_config_consum(buf, scratch).map(|t| t.len())),
        "config_convert" => Some(parse_config_convert(buf, scratch).map(|t| t.len())),
        "config_experience" => Some(parse_config_experience(buf, scratch).map(|t| t.len())),
        "config_item_binding_mapping" => {
            Some(parse_config_item_binding_mapping(buf, scratch).map(|t| t.len()))
        }
        "config_item_gold_exchange" => {
            Some(parse_config_item_gold_exchange(buf, scratch).map(|t| t.len()))
        }
        "config_item_gold_random_affix_config" => {
            Some(parse_config_item_gold_random_affix_config(buf, scratch).map(|t| t.len()))
        }
        "config_item_rarity_mapping" => {
            Some(parse_config_item_rarity_mapping(buf, scratch).map(|t| t.len()))
        }
        "config_item_s13_organ" => Some(parse_config_item_s13_organ(buf, scratch).map(|t| t.len())),
        "config_item_stream_mode" => {
            Some(parse_config_item_stream_mode(buf, scratch).map(|t| t.len()))
        }
        "config_item_talent_block_gold" => {
            Some(parse_config_item_talent_block_gold(buf, scratch).map(|t| t.len()))
        }
        "config_item_target_function" => {
            Some(parse_config_item_target_function(buf, scratch).map(|t| t.len()))
        }
        "config_item_tattoo" => Some(parse_config_item_tattoo(buf, scratch).map(|t| t.len())),
        "config_item_type_list" => Some(parse_config_item_type_list(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

fn verify_item_chunk_1(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_money" => Some(parse_config_money(buf, scratch).map(|t| t.len())),
        "config_reward" => Some(parse_config_reward(buf, scratch).map(|t| t.len())),
        "config_specified_item" => Some(parse_config_specified_item(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

pub(crate) fn verify_item_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    if let Some(res) = verify_item_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_item_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    None
}

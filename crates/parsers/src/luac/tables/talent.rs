//! Auto-generated `LuaTable/DataCfg` types and parsers for the `talent` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;

/// Row record for `config_devil_block_attribute_s15.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDevilBlockAttributeS15Row {
    pub affix_weight: u8,
    pub allow_position: Box<[u16]>,
    pub default_position: u16,
    pub grow_des: Box<str>,
    pub id: i8,
    pub influence_id: u16,
    pub item_id: u32,
    pub sever_affix: Box<str>,
    pub shape_group: u8,
    pub _grow_des: Box<str>,
}

impl DataCfgRow for ConfigDevilBlockAttributeS15Row {
    const TABLE_NAME: &'static str = "config_devil_block_attribute_s15";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AffixWeight" => {
                self.affix_weight = ctx.decode_field(val, Self::TABLE_NAME, "AffixWeight")?
            }
            b"AllowPosition" => {
                self.allow_position = ctx.decode_field(val, Self::TABLE_NAME, "AllowPosition")?
            }
            b"DefaultPosition" => {
                self.default_position =
                    ctx.decode_field(val, Self::TABLE_NAME, "DefaultPosition")?
            }
            b"GrowDes" => self.grow_des = ctx.decode_field(val, Self::TABLE_NAME, "GrowDes")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InfluenceId" => {
                self.influence_id = ctx.decode_field(val, Self::TABLE_NAME, "InfluenceId")?
            }
            b"ItemId" => self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?,
            b"SeverAffix" => {
                self.sever_affix = ctx.decode_field(val, Self::TABLE_NAME, "SeverAffix")?
            }
            b"ShapeGroup" => {
                self.shape_group = ctx.decode_field(val, Self::TABLE_NAME, "ShapeGroup")?
            }
            b"_GrowDes" => self._grow_des = ctx.decode_field(val, Self::TABLE_NAME, "_GrowDes")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_devil_block_attribute_s15.out`.
pub type ConfigDevilBlockAttributeS15Table = DataTable<u8, ConfigDevilBlockAttributeS15Row>;

/// Parses `config_devil_block_attribute_s15.out` into a [`ConfigDevilBlockAttributeS15Table`].
pub fn parse_config_devil_block_attribute_s15(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDevilBlockAttributeS15Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_devil_block_basetalent_drop_ratio.TalentRule`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DevilBlockBasetalentDropRatioTalentRuleEntry {
    pub rule_id: u16,
    pub weight: u16,
}

impl FromEvalVal for DevilBlockBasetalentDropRatioTalentRuleEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (rule_id, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { rule_id, weight })
    }
}

/// Row record for `config_devil_block_basetalent_drop_ratio.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDevilBlockBasetalentDropRatioRow {
    pub id: i8,
    pub level: Box<[u16]>,
    pub rarity: Box<[u8]>,
    pub talent_rule: Box<[DevilBlockBasetalentDropRatioTalentRuleEntry]>,
    pub type3: u8,
    pub type4: u8,
}

impl DataCfgRow for ConfigDevilBlockBasetalentDropRatioRow {
    const TABLE_NAME: &'static str = "config_devil_block_basetalent_drop_ratio";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"TalentRule" => {
                self.talent_rule = ctx.decode_field(val, Self::TABLE_NAME, "TalentRule")?
            }
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_devil_block_basetalent_drop_ratio.out`.
pub type ConfigDevilBlockBasetalentDropRatioTable =
    DataTable<u8, ConfigDevilBlockBasetalentDropRatioRow>;

/// Parses `config_devil_block_basetalent_drop_ratio.out` into a [`ConfigDevilBlockBasetalentDropRatioTable`].
pub fn parse_config_devil_block_basetalent_drop_ratio(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDevilBlockBasetalentDropRatioTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_devil_block_c_bonus.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDevilBlockCBonusRow {
    pub bounus_modifier: Box<[u32]>,
    pub group: u16,
    pub id: i32,
    pub mark_num: Box<[u8]>,
}

impl DataCfgRow for ConfigDevilBlockCBonusRow {
    const TABLE_NAME: &'static str = "config_devil_block_c_bonus";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BounusModifier" => {
                self.bounus_modifier = ctx.decode_field(val, Self::TABLE_NAME, "BounusModifier")?
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MarkNum" => self.mark_num = ctx.decode_field(val, Self::TABLE_NAME, "MarkNum")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_devil_block_c_bonus.out`.
pub type ConfigDevilBlockCBonusTable = DataTable<u32, ConfigDevilBlockCBonusRow>;

/// Parses `config_devil_block_c_bonus.out` into a [`ConfigDevilBlockCBonusTable`].
pub fn parse_config_devil_block_c_bonus(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDevilBlockCBonusTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_devil_block_enhance_s15.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDevilBlockEnhanceS15Row {
    pub allow_type3: Box<[u8]>,
    pub consume: FxHashMap<u32, u8>,
    pub consume_rarity: u8,
    pub desc: Box<str>,
    pub id: i16,
    pub item_level: Box<[u16]>,
    pub phase: u8,
    pub rarity: u8,
    pub recipe_id: Box<[u16]>,
    pub request_num: u8,
    pub slot: u8,
    pub target_talent_type: Box<[u8]>,
    pub type3: Box<[u8]>,
    pub type4: Box<[u8]>,
    pub _desc: Box<str>,
}

impl ConfigDevilBlockEnhanceS15Row {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AllowType3" => {
                self.allow_type3 = ctx.decode_field(val, Self::TABLE_NAME, "AllowType3")?
            }
            b"Consume" => self.consume = ctx.decode_field(val, Self::TABLE_NAME, "Consume")?,
            b"ConsumeRarity" => {
                self.consume_rarity = ctx.decode_field(val, Self::TABLE_NAME, "ConsumeRarity")?
            }
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?
            }
            b"Phase" => self.phase = ctx.decode_field(val, Self::TABLE_NAME, "Phase")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"RecipeId" => self.recipe_id = ctx.decode_field(val, Self::TABLE_NAME, "RecipeId")?,
            b"RequestNum" => {
                self.request_num = ctx.decode_field(val, Self::TABLE_NAME, "RequestNum")?
            }
            b"Slot" => self.slot = ctx.decode_field(val, Self::TABLE_NAME, "Slot")?,
            b"TargetTalentType" => {
                self.target_talent_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "TargetTalentType")?
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
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigDevilBlockEnhanceS15Row {
    const TABLE_NAME: &'static str = "config_devil_block_enhance_s15";

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

/// Typed `DataTable` for `config_devil_block_enhance_s15.out`.
pub type ConfigDevilBlockEnhanceS15Table = DataTable<u8, ConfigDevilBlockEnhanceS15Row>;

/// Parses `config_devil_block_enhance_s15.out` into a [`ConfigDevilBlockEnhanceS15Table`].
pub fn parse_config_devil_block_enhance_s15(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDevilBlockEnhanceS15Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_devil_block_shape_s15.Shape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DevilBlockShapeS15ShapeEntry {
    pub part_index: u8,
    pub offset_x: i8,
    pub offset_y: i8,
}

impl FromEvalVal for DevilBlockShapeS15ShapeEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (part_index, (offset_x, offset_y)) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            part_index,
            offset_x,
            offset_y,
        })
    }
}

/// Entry for `config_devil_block_shape_s15.Shape0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DevilBlockShapeS15Shape0Entry {
    pub part_index: u8,
    pub offset_x: i8,
    pub offset_y: i8,
}

impl FromEvalVal for DevilBlockShapeS15Shape0Entry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (part_index, (offset_x, offset_y)) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            part_index,
            offset_x,
            offset_y,
        })
    }
}

/// Row record for `config_devil_block_shape_s15.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigDevilBlockShapeS15Row {
    pub center_offset: Box<[f64]>,
    pub group: u8,
    pub grow_count: u8,
    pub icon_max: Box<str>,
    pub icon_min: Box<str>,
    pub id: i32,
    pub mask: Box<str>,
    pub mask_rotation: u16,
    pub position: u16,
    pub shape: Box<[DevilBlockShapeS15ShapeEntry]>,
    pub shape0: Box<[DevilBlockShapeS15Shape0Entry]>,
}

impl DataCfgRow for ConfigDevilBlockShapeS15Row {
    const TABLE_NAME: &'static str = "config_devil_block_shape_s15";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CenterOffset" => {
                self.center_offset = ctx.decode_field(val, Self::TABLE_NAME, "CenterOffset")?
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"GrowCount" => {
                self.grow_count = ctx.decode_field(val, Self::TABLE_NAME, "GrowCount")?
            }
            b"IconMax" => self.icon_max = ctx.decode_field(val, Self::TABLE_NAME, "IconMax")?,
            b"IconMin" => self.icon_min = ctx.decode_field(val, Self::TABLE_NAME, "IconMin")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Mask" => self.mask = ctx.decode_field(val, Self::TABLE_NAME, "Mask")?,
            b"MaskRotation" => {
                self.mask_rotation = ctx.decode_field(val, Self::TABLE_NAME, "MaskRotation")?
            }
            b"Position" => self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?,
            b"Shape" => self.shape = ctx.decode_field(val, Self::TABLE_NAME, "Shape")?,
            b"Shape0" => self.shape0 = ctx.decode_field(val, Self::TABLE_NAME, "Shape0")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_devil_block_shape_s15.out`.
pub type ConfigDevilBlockShapeS15Table = DataTable<u16, ConfigDevilBlockShapeS15Row>;

/// Parses `config_devil_block_shape_s15.out` into a [`ConfigDevilBlockShapeS15Table`].
pub fn parse_config_devil_block_shape_s15(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDevilBlockShapeS15Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_devil_block_slot_drop_ratio_s15.SlotRecipe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DevilBlockSlotDropRatioS15SlotRecipeEntry {
    pub recipe_id: u16,
    pub weight: u16,
}

impl FromEvalVal for DevilBlockSlotDropRatioS15SlotRecipeEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (recipe_id, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { recipe_id, weight })
    }
}

/// Row record for `config_devil_block_slot_drop_ratio_s15.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDevilBlockSlotDropRatioS15Row {
    pub base_talent_recipe: Box<[u16]>,
    pub id: i8,
    pub slot_recipe: Box<[DevilBlockSlotDropRatioS15SlotRecipeEntry]>,
    pub type3: u8,
    pub type4: u8,
}

impl DataCfgRow for ConfigDevilBlockSlotDropRatioS15Row {
    const TABLE_NAME: &'static str = "config_devil_block_slot_drop_ratio_s15";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BaseTalentRecipe" => {
                self.base_talent_recipe =
                    ctx.decode_field(val, Self::TABLE_NAME, "BaseTalentRecipe")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"SlotRecipe" => {
                self.slot_recipe = ctx.decode_field(val, Self::TABLE_NAME, "SlotRecipe")?
            }
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_devil_block_slot_drop_ratio_s15.out`.
pub type ConfigDevilBlockSlotDropRatioS15Table = DataTable<u8, ConfigDevilBlockSlotDropRatioS15Row>;

/// Parses `config_devil_block_slot_drop_ratio_s15.out` into a [`ConfigDevilBlockSlotDropRatioS15Table`].
pub fn parse_config_devil_block_slot_drop_ratio_s15(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDevilBlockSlotDropRatioS15Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_prism_attribute.InfluenceShape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrismAttributeInfluenceShapeEntry {
    pub shape_id: u8,
    pub weight: u8,
}

impl FromEvalVal for PrismAttributeInfluenceShapeEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (shape_id, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { shape_id, weight })
    }
}

/// Entry for `config_prism_attribute.InherentAffixNum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrismAttributeInherentAffixNumEntry {
    pub count: u8,
    pub weight: u8,
}

impl FromEvalVal for PrismAttributeInherentAffixNumEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (count, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { count, weight })
    }
}

/// Entry for `config_prism_attribute.SlotNum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrismAttributeSlotNumEntry {
    pub count: u8,
    pub weight: u8,
}

impl FromEvalVal for PrismAttributeSlotNumEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (count, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { count, weight })
    }
}

/// Row record for `config_prism_attribute.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigPrismAttributeRow {
    pub id: i8,
    pub influence_shape: Box<[PrismAttributeInfluenceShapeEntry]>,
    pub inherent_affix_num: Box<[PrismAttributeInherentAffixNumEntry]>,
    pub item_id: [u8; 0],
    pub level: Box<[u8]>,
    pub slot_num: Box<[PrismAttributeSlotNumEntry]>,
    pub type2: Box<[u8]>,
    pub type3: Box<[u8]>,
}

impl DataCfgRow for ConfigPrismAttributeRow {
    const TABLE_NAME: &'static str = "config_prism_attribute";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InfluenceShape" => {
                self.influence_shape = ctx.decode_field(val, Self::TABLE_NAME, "InfluenceShape")?
            }
            b"InherentAffixNum" => {
                self.inherent_affix_num =
                    ctx.decode_field(val, Self::TABLE_NAME, "InherentAffixNum")?
            }
            b"ItemId" => self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"SlotNum" => self.slot_num = ctx.decode_field(val, Self::TABLE_NAME, "SlotNum")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_prism_attribute.out`.
pub type ConfigPrismAttributeTable = DataTable<u8, ConfigPrismAttributeRow>;

/// Parses `config_prism_attribute.out` into a [`ConfigPrismAttributeTable`].
pub fn parse_config_prism_attribute(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPrismAttributeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_prism_enhance_s12.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigPrismEnhanceS12Row {
    pub consume: FxHashMap<u32, u8>,
    pub desc: Box<str>,
    pub enhance_type: u8,
    pub id: i16,
    pub level: Box<[u8]>,
    pub refresh_consume: FxHashMap<u32, u8>,
    pub slot: Box<[u8]>,
    pub slot_num: Box<[u8]>,
    pub tags: Box<[u32]>,
    pub type2: Box<[u8]>,
    pub type3: Box<[u8]>,
    pub type4: Box<[u8]>,
    pub _desc: Box<str>,
}

impl ConfigPrismEnhanceS12Row {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Consume" => self.consume = ctx.decode_field(val, Self::TABLE_NAME, "Consume")?,
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"EnhanceType" => {
                self.enhance_type = ctx.decode_field(val, Self::TABLE_NAME, "EnhanceType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"RefreshConsume" => {
                self.refresh_consume = ctx.decode_field(val, Self::TABLE_NAME, "RefreshConsume")?
            }
            b"Slot" => self.slot = ctx.decode_field(val, Self::TABLE_NAME, "Slot")?,
            b"SlotNum" => self.slot_num = ctx.decode_field(val, Self::TABLE_NAME, "SlotNum")?,
            b"Tags" => self.tags = ctx.decode_field(val, Self::TABLE_NAME, "Tags")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
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

impl DataCfgRow for ConfigPrismEnhanceS12Row {
    const TABLE_NAME: &'static str = "config_prism_enhance_s12";

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

/// Typed `DataTable` for `config_prism_enhance_s12.out`.
pub type ConfigPrismEnhanceS12Table = DataTable<u16, ConfigPrismEnhanceS12Row>;

/// Parses `config_prism_enhance_s12.out` into a [`ConfigPrismEnhanceS12Table`].
pub fn parse_config_prism_enhance_s12(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPrismEnhanceS12Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_prism_influence_shape.Shape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrismInfluenceShapeShapeEntry {
    pub part_index: u8,
    pub offset_x: i8,
    pub offset_y: i8,
}

impl FromEvalVal for PrismInfluenceShapeShapeEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (part_index, (offset_x, offset_y)) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            part_index,
            offset_x,
            offset_y,
        })
    }
}

/// Row record for `config_prism_influence_shape.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigPrismInfluenceShapeRow {
    pub desc: Box<str>,
    pub id: i8,
    pub shape: Box<[PrismInfluenceShapeShapeEntry]>,
    pub ui_appearance: Box<str>,
    pub _desc: Box<str>,
}

impl DataCfgRow for ConfigPrismInfluenceShapeRow {
    const TABLE_NAME: &'static str = "config_prism_influence_shape";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Shape" => self.shape = ctx.decode_field(val, Self::TABLE_NAME, "Shape")?,
            b"UiAppearance" => {
                self.ui_appearance = ctx.decode_field(val, Self::TABLE_NAME, "UiAppearance")?
            }
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_prism_influence_shape.out`.
pub type ConfigPrismInfluenceShapeTable = DataTable<u8, ConfigPrismInfluenceShapeRow>;

/// Parses `config_prism_influence_shape.out` into a [`ConfigPrismInfluenceShapeTable`].
pub fn parse_config_prism_influence_shape(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPrismInfluenceShapeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_prism_modifier_mapping.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigPrismModifierMappingRow {
    pub id: i8,
    pub level: Box<[u8]>,
    pub modifier_mapping: Box<[u32]>,
    pub type2: Box<[u8]>,
}

impl DataCfgRow for ConfigPrismModifierMappingRow {
    const TABLE_NAME: &'static str = "config_prism_modifier_mapping";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"ModifierMapping" => {
                self.modifier_mapping =
                    ctx.decode_field(val, Self::TABLE_NAME, "ModifierMapping")?
            }
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_prism_modifier_mapping.out`.
pub type ConfigPrismModifierMappingTable = DataTable<u8, ConfigPrismModifierMappingRow>;

/// Parses `config_prism_modifier_mapping.out` into a [`ConfigPrismModifierMappingTable`].
pub fn parse_config_prism_modifier_mapping(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigPrismModifierMappingTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_talent_block_board.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentBlockBoardRow {
    pub id: i16,
    pub item_cost: [u8; 0],
    pub level: u8,
    pub location: Box<[u8]>,
    pub task_desc: Box<str>,
    pub task_id: u8,
    pub _task_desc: Box<str>,
}

impl DataCfgRow for ConfigTalentBlockBoardRow {
    const TABLE_NAME: &'static str = "config_talent_block_board";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ID" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "ID")?,
            b"ItemCost" => self.item_cost = ctx.decode_field(val, Self::TABLE_NAME, "ItemCost")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Location" => self.location = ctx.decode_field(val, Self::TABLE_NAME, "Location")?,
            b"TaskDesc" => self.task_desc = ctx.decode_field(val, Self::TABLE_NAME, "TaskDesc")?,
            b"TaskId" => self.task_id = ctx.decode_field(val, Self::TABLE_NAME, "TaskId")?,
            b"_TaskDesc" => {
                self._task_desc = ctx.decode_field(val, Self::TABLE_NAME, "_TaskDesc")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_talent_block_board.out`.
pub type ConfigTalentBlockBoardTable = DataTable<u16, ConfigTalentBlockBoardRow>;

/// Parses `config_talent_block_board.out` into a [`ConfigTalentBlockBoardTable`].
pub fn parse_config_talent_block_board(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentBlockBoardTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_talent_block_detail_search.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentBlockDetailSearchRow {
    pub base_talent: Box<[u8]>,
    pub build_talent: Box<[u8]>,
    pub id: i8,
    pub type1: Box<[u8]>,
    pub type2: Box<[u8]>,
    pub type3: Box<[u8]>,
    pub type4: Box<[u8]>,
}

impl DataCfgRow for ConfigTalentBlockDetailSearchRow {
    const TABLE_NAME: &'static str = "config_talent_block_detail_search";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BaseTalent" => {
                self.base_talent = ctx.decode_field(val, Self::TABLE_NAME, "BaseTalent")?
            }
            b"BuildTalent" => {
                self.build_talent = ctx.decode_field(val, Self::TABLE_NAME, "BuildTalent")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_talent_block_detail_search.out`.
pub type ConfigTalentBlockDetailSearchTable = DataTable<u8, ConfigTalentBlockDetailSearchRow>;

/// Parses `config_talent_block_detail_search.out` into a [`ConfigTalentBlockDetailSearchTable`].
pub fn parse_config_talent_block_detail_search(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentBlockDetailSearchTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_talent_block_enhance.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentBlockEnhanceRow {
    pub aria: u8,
    pub enhance_success1: FxHashMap<u8, Box<[u16]>>,
    pub enhance_success2: FxHashMap<u8, Box<[u16]>>,
    pub enhance_type: u8,
    pub group: u16,
    pub id: i16,
    pub talent_limit: Box<str>,
    pub upper_limit: Box<str>,
}

impl DataCfgRow for ConfigTalentBlockEnhanceRow {
    const TABLE_NAME: &'static str = "config_talent_block_enhance";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Aria" => self.aria = ctx.decode_field(val, Self::TABLE_NAME, "Aria")?,
            b"EnhanceSuccess1" => {
                self.enhance_success1 =
                    ctx.decode_field(val, Self::TABLE_NAME, "EnhanceSuccess1")?
            }
            b"EnhanceSuccess2" => {
                self.enhance_success2 =
                    ctx.decode_field(val, Self::TABLE_NAME, "EnhanceSuccess2")?
            }
            b"EnhanceType" => {
                self.enhance_type = ctx.decode_field(val, Self::TABLE_NAME, "EnhanceType")?
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"TalentLimit" => {
                self.talent_limit = ctx.decode_field(val, Self::TABLE_NAME, "TalentLimit")?
            }
            b"UpperLimit" => {
                self.upper_limit = ctx.decode_field(val, Self::TABLE_NAME, "UpperLimit")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_talent_block_enhance.out`.
pub type ConfigTalentBlockEnhanceTable = DataTable<u16, ConfigTalentBlockEnhanceRow>;

/// Parses `config_talent_block_enhance.out` into a [`ConfigTalentBlockEnhanceTable`].
pub fn parse_config_talent_block_enhance(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentBlockEnhanceTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_talent_block_enhance_cost.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentBlockEnhanceCostRow {
    pub back_consume: Box<str>,
    pub back_consume_number: u8,
    pub consume: Box<str>,
    pub id: i16,
    pub key: u8,
    pub revert_consume: Box<str>,
    pub r#type: u8,
}

impl DataCfgRow for ConfigTalentBlockEnhanceCostRow {
    const TABLE_NAME: &'static str = "config_talent_block_enhance_cost";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BackConsume" => {
                self.back_consume = ctx.decode_field(val, Self::TABLE_NAME, "BackConsume")?
            }
            b"BackConsumeNumber" => {
                self.back_consume_number =
                    ctx.decode_field(val, Self::TABLE_NAME, "BackConsumeNumber")?
            }
            b"Consume" => self.consume = ctx.decode_field(val, Self::TABLE_NAME, "Consume")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Key" => self.key = ctx.decode_field(val, Self::TABLE_NAME, "Key")?,
            b"RevertConsume" => {
                self.revert_consume = ctx.decode_field(val, Self::TABLE_NAME, "RevertConsume")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_talent_block_enhance_cost.out`.
pub type ConfigTalentBlockEnhanceCostTable = DataTable<u16, ConfigTalentBlockEnhanceCostRow>;

/// Parses `config_talent_block_enhance_cost.out` into a [`ConfigTalentBlockEnhanceCostTable`].
pub fn parse_config_talent_block_enhance_cost(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentBlockEnhanceCostTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_talent_block_enhance_s7.Consume`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TalentBlockEnhanceS7ConsumeEntry {
    pub item_id: u16,
    pub count: u8,
}

impl FromEvalVal for TalentBlockEnhanceS7ConsumeEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (item_id, count) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { item_id, count })
    }
}

/// Entry for `config_talent_block_enhance_s7.ProfessionSpecifiedCost`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TalentBlockEnhanceS7ProfessionSpecifiedCostEntry {
    pub profession_id: u8,
    pub item_id: u16,
    pub count: u8,
}

impl FromEvalVal for TalentBlockEnhanceS7ProfessionSpecifiedCostEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (profession_id, (item_id, count)) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            profession_id,
            item_id,
            count,
        })
    }
}

/// Row record for `config_talent_block_enhance_s7.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentBlockEnhanceS7Row {
    pub consume: Box<[TalentBlockEnhanceS7ConsumeEntry]>,
    pub desc: Box<str>,
    pub forge_talent_type: Box<[u8]>,
    pub id: i16,
    pub item_level: Box<[u16]>,
    pub legendary_affix_num: Box<[u8]>,
    pub profession_specified: u8,
    pub profession_specified_cost: Box<[TalentBlockEnhanceS7ProfessionSpecifiedCostEntry]>,
    pub slot_num1: Box<[u8]>,
    pub slot_num2: Box<[u8]>,
    pub slot_num3: Box<[u8]>,
    pub slot_type: Box<[u8]>,
    pub target_talent_type: Box<[u8]>,
    pub r#type: u8,
    pub _desc: Box<str>,
}

impl ConfigTalentBlockEnhanceS7Row {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Consume" => self.consume = ctx.decode_field(val, Self::TABLE_NAME, "Consume")?,
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"ForgeTalentType" => {
                self.forge_talent_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "ForgeTalentType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?
            }
            b"LegendaryAffixNum" => {
                self.legendary_affix_num =
                    ctx.decode_field(val, Self::TABLE_NAME, "LegendaryAffixNum")?
            }
            b"ProfessionSpecified" => {
                self.profession_specified =
                    ctx.decode_field(val, Self::TABLE_NAME, "ProfessionSpecified")?
            }
            b"ProfessionSpecifiedCost" => {
                self.profession_specified_cost =
                    ctx.decode_field(val, Self::TABLE_NAME, "ProfessionSpecifiedCost")?
            }
            b"SlotNum1" => self.slot_num1 = ctx.decode_field(val, Self::TABLE_NAME, "SlotNum1")?,
            b"SlotNum2" => self.slot_num2 = ctx.decode_field(val, Self::TABLE_NAME, "SlotNum2")?,
            b"SlotNum3" => self.slot_num3 = ctx.decode_field(val, Self::TABLE_NAME, "SlotNum3")?,
            b"SlotType" => self.slot_type = ctx.decode_field(val, Self::TABLE_NAME, "SlotType")?,
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
            b"TargetTalentType" => {
                self.target_talent_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "TargetTalentType")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigTalentBlockEnhanceS7Row {
    const TABLE_NAME: &'static str = "config_talent_block_enhance_s7";

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

/// Typed `DataTable` for `config_talent_block_enhance_s7.out`.
pub type ConfigTalentBlockEnhanceS7Table = DataTable<u8, ConfigTalentBlockEnhanceS7Row>;

/// Parses `config_talent_block_enhance_s7.out` into a [`ConfigTalentBlockEnhanceS7Table`].
pub fn parse_config_talent_block_enhance_s7(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentBlockEnhanceS7Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_talent_block_filter.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentBlockFilterRow {
    pub desc: Box<str>,
    pub id: i8,
    pub item_id: Box<[u32]>,
    pub item_type: u8,
    pub _desc: Box<str>,
}

impl DataCfgRow for ConfigTalentBlockFilterRow {
    const TABLE_NAME: &'static str = "config_talent_block_filter";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemId" => self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?,
            b"ItemType" => self.item_type = ctx.decode_field(val, Self::TABLE_NAME, "ItemType")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_talent_block_filter.out`.
pub type ConfigTalentBlockFilterTable = DataTable<u8, ConfigTalentBlockFilterRow>;

/// Parses `config_talent_block_filter.out` into a [`ConfigTalentBlockFilterTable`].
pub fn parse_config_talent_block_filter(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentBlockFilterTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_talent_block_influence.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentBlockInfluenceRow {
    pub description: Box<str>,
    pub effect: Box<str>,
    pub id: i16,
    pub params: Box<[u8]>,
    pub title: Box<str>,
    pub _description: Box<str>,
    pub _title: Box<str>,
}

impl DataCfgRow for ConfigTalentBlockInfluenceRow {
    const TABLE_NAME: &'static str = "config_talent_block_influence";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Description" => {
                self.description = ctx.decode_field(val, Self::TABLE_NAME, "Description")?
            }
            b"Effect" => self.effect = ctx.decode_field(val, Self::TABLE_NAME, "Effect")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Params" => self.params = ctx.decode_field(val, Self::TABLE_NAME, "Params")?,
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_talent_block_influence.out`.
pub type ConfigTalentBlockInfluenceTable = DataTable<u16, ConfigTalentBlockInfluenceRow>;

/// Parses `config_talent_block_influence.out` into a [`ConfigTalentBlockInfluenceTable`].
pub fn parse_config_talent_block_influence(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentBlockInfluenceTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_talent_block_revert_s7.Consume`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TalentBlockRevertS7ConsumeEntry {
    pub item_id: u16,
    pub count: u8,
}

impl FromEvalVal for TalentBlockRevertS7ConsumeEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (item_id, count) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { item_id, count })
    }
}

/// Row record for `config_talent_block_revert_s7.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentBlockRevertS7Row {
    pub consume: Box<[TalentBlockRevertS7ConsumeEntry]>,
    pub id: i16,
    pub item_level: Box<[u16]>,
    pub revert_count: Box<[u32]>,
}

impl DataCfgRow for ConfigTalentBlockRevertS7Row {
    const TABLE_NAME: &'static str = "config_talent_block_revert_s7";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Consume" => self.consume = ctx.decode_field(val, Self::TABLE_NAME, "Consume")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?
            }
            b"RevertCount" => {
                self.revert_count = ctx.decode_field(val, Self::TABLE_NAME, "RevertCount")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_talent_block_revert_s7.out`.
pub type ConfigTalentBlockRevertS7Table = DataTable<u16, ConfigTalentBlockRevertS7Row>;

/// Parses `config_talent_block_revert_s7.out` into a [`ConfigTalentBlockRevertS7Table`].
pub fn parse_config_talent_block_revert_s7(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentBlockRevertS7Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_talent_block_shape.ReshapeConsume`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TalentBlockShapeReshapeConsumeEntry {
    pub step: u8,
    pub min_count: u8,
    pub max_count: u16,
    pub item_id: u16,
    pub count: u8,
}

impl FromEvalVal for TalentBlockShapeReshapeConsumeEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (step, (min_count, max_count, item_id, count)) =
            ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            step,
            min_count,
            max_count,
            item_id,
            count,
        })
    }
}

/// Entry for `config_talent_block_shape.RotationConsume`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TalentBlockShapeRotationConsumeEntry {
    pub step: u8,
    pub min_count: u8,
    pub max_count: u16,
    pub item_id: u16,
    pub count: u8,
}

impl FromEvalVal for TalentBlockShapeRotationConsumeEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (step, (min_count, max_count, item_id, count)) =
            ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            step,
            min_count,
            max_count,
            item_id,
            count,
        })
    }
}

/// Row record for `config_talent_block_shape.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigTalentBlockShapeRow {
    pub center_offset: Box<[f64]>,
    pub forbid_reshape: u8,
    pub forbid_rotate: u8,
    pub group: u16,
    pub id: i16,
    pub icon: Box<str>,
    pub is_mirror: u8,
    pub mask: Box<str>,
    pub material: Box<[Box<str>]>,
    pub name: Box<str>,
    pub reshape_consume: Box<[TalentBlockShapeReshapeConsumeEntry]>,
    pub rotation_consume: Box<[TalentBlockShapeRotationConsumeEntry]>,
    pub shape: Box<str>,
    pub talent_num: u8,
    pub _name: Box<str>,
}

impl ConfigTalentBlockShapeRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"CenterOffset" => {
                self.center_offset = ctx.decode_field(val, Self::TABLE_NAME, "CenterOffset")?
            }
            b"ForbidReshape" => {
                self.forbid_reshape = ctx.decode_field(val, Self::TABLE_NAME, "ForbidReshape")?
            }
            b"ForbidRotate" => {
                self.forbid_rotate = ctx.decode_field(val, Self::TABLE_NAME, "ForbidRotate")?
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"ID" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "ID")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"IsMirror" => self.is_mirror = ctx.decode_field(val, Self::TABLE_NAME, "IsMirror")?,
            b"Mask" => self.mask = ctx.decode_field(val, Self::TABLE_NAME, "Mask")?,
            b"Material" => self.material = ctx.decode_field(val, Self::TABLE_NAME, "Material")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"ReshapeConsume" => {
                self.reshape_consume = ctx.decode_field(val, Self::TABLE_NAME, "ReshapeConsume")?
            }
            b"RotationConsume" => {
                self.rotation_consume =
                    ctx.decode_field(val, Self::TABLE_NAME, "RotationConsume")?
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
            b"Shape" => self.shape = ctx.decode_field(val, Self::TABLE_NAME, "Shape")?,
            b"TalentNum" => {
                self.talent_num = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigTalentBlockShapeRow {
    const TABLE_NAME: &'static str = "config_talent_block_shape";

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

/// Typed `DataTable` for `config_talent_block_shape.out`.
pub type ConfigTalentBlockShapeTable = DataTable<u16, ConfigTalentBlockShapeRow>;

/// Parses `config_talent_block_shape.out` into a [`ConfigTalentBlockShapeTable`].
pub fn parse_config_talent_block_shape(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentBlockShapeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_talent_block_slot_recipe.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentBlockSlotRecipeRow {
    pub id: i16,
    pub recipe_type: u8,
    pub talent_num1: u8,
    pub talent_num2: u8,
    pub talent_num3: u8,
    pub talent_num4: u8,
    pub talent_num5: u8,
    pub talent_num6: u8,
    pub talent_num7: u8,
    pub talent_num7_a: u8,
    pub talent_num7_b: u8,
    pub talent_num7_c: u8,
    pub talent_num8: u8,
    pub talent_num8_a: u8,
    pub talent_num8_b: u8,
    pub talent_num8_c: u8,
}

impl ConfigTalentBlockSlotRecipeRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"RecipeType" => {
                self.recipe_type = ctx.decode_field(val, Self::TABLE_NAME, "RecipeType")?
            }
            b"TalentNum1" => {
                self.talent_num1 = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum1")?
            }
            b"TalentNum2" => {
                self.talent_num2 = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum2")?
            }
            b"TalentNum3" => {
                self.talent_num3 = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum3")?
            }
            b"TalentNum4" => {
                self.talent_num4 = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum4")?
            }
            b"TalentNum5" => {
                self.talent_num5 = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum5")?
            }
            b"TalentNum6" => {
                self.talent_num6 = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum6")?
            }
            b"TalentNum7" => {
                self.talent_num7 = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum7")?
            }
            b"TalentNum7A" => {
                self.talent_num7_a = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum7A")?
            }
            b"TalentNum7B" => {
                self.talent_num7_b = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum7B")?
            }
            b"TalentNum7C" => {
                self.talent_num7_c = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum7C")?
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
            b"TalentNum8" => {
                self.talent_num8 = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum8")?
            }
            b"TalentNum8A" => {
                self.talent_num8_a = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum8A")?
            }
            b"TalentNum8B" => {
                self.talent_num8_b = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum8B")?
            }
            b"TalentNum8C" => {
                self.talent_num8_c = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum8C")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigTalentBlockSlotRecipeRow {
    const TABLE_NAME: &'static str = "config_talent_block_slot_recipe";

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

/// Typed `DataTable` for `config_talent_block_slot_recipe.out`.
pub type ConfigTalentBlockSlotRecipeTable = DataTable<u16, ConfigTalentBlockSlotRecipeRow>;

/// Parses `config_talent_block_slot_recipe.out` into a [`ConfigTalentBlockSlotRecipeTable`].
pub fn parse_config_talent_block_slot_recipe(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentBlockSlotRecipeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_talent_filter.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentFilterRow {
    pub id: i8,
    pub talent_gods: Box<[u8]>,
    pub talent_ids: [u8; 0],
    pub talent_tags: Box<[u8]>,
    pub talent_types: Box<[u16]>,
}

impl DataCfgRow for ConfigTalentFilterRow {
    const TABLE_NAME: &'static str = "config_talent_filter";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"TalentGods" => {
                self.talent_gods = ctx.decode_field(val, Self::TABLE_NAME, "TalentGods")?
            }
            b"TalentIds" => {
                self.talent_ids = ctx.decode_field(val, Self::TABLE_NAME, "TalentIds")?
            }
            b"TalentTags" => {
                self.talent_tags = ctx.decode_field(val, Self::TABLE_NAME, "TalentTags")?
            }
            b"TalentTypes" => {
                self.talent_types = ctx.decode_field(val, Self::TABLE_NAME, "TalentTypes")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_talent_filter.out`.
pub type ConfigTalentFilterTable = DataTable<u8, ConfigTalentFilterRow>;

/// Parses `config_talent_filter.out` into a [`ConfigTalentFilterTable`].
pub fn parse_config_talent_filter(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentFilterTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_talent_group_moon.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTalentGroupMoonRow {
    pub hero_id: u32,
    pub id: i16,
    pub profession_id: u8,
    pub talent_group_moon: Box<[u32]>,
    pub talent_original_id: Box<[u32]>,
}

impl DataCfgRow for ConfigTalentGroupMoonRow {
    const TABLE_NAME: &'static str = "config_talent_group_moon";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"HeroId" => self.hero_id = ctx.decode_field(val, Self::TABLE_NAME, "HeroId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ProfessionId" => {
                self.profession_id = ctx.decode_field(val, Self::TABLE_NAME, "ProfessionId")?
            }
            b"TalentGroupMoon" => {
                self.talent_group_moon =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentGroupMoon")?
            }
            b"TalentOriginalId" => {
                self.talent_original_id =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentOriginalId")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_talent_group_moon.out`.
pub type ConfigTalentGroupMoonTable = DataTable<u16, ConfigTalentGroupMoonRow>;

/// Parses `config_talent_group_moon.out` into a [`ConfigTalentGroupMoonTable`].
pub fn parse_config_talent_group_moon(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTalentGroupMoonTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

fn verify_talent_chunk_0(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_devil_block_attribute_s15" => {
            Some(parse_config_devil_block_attribute_s15(buf, scratch).map(|t| t.len()))
        }
        "config_devil_block_basetalent_drop_ratio" => {
            Some(parse_config_devil_block_basetalent_drop_ratio(buf, scratch).map(|t| t.len()))
        }
        "config_devil_block_c_bonus" => {
            Some(parse_config_devil_block_c_bonus(buf, scratch).map(|t| t.len()))
        }
        "config_devil_block_enhance_s15" => {
            Some(parse_config_devil_block_enhance_s15(buf, scratch).map(|t| t.len()))
        }
        "config_devil_block_shape_s15" => {
            Some(parse_config_devil_block_shape_s15(buf, scratch).map(|t| t.len()))
        }
        "config_devil_block_slot_drop_ratio_s15" => {
            Some(parse_config_devil_block_slot_drop_ratio_s15(buf, scratch).map(|t| t.len()))
        }
        "config_prism_attribute" => {
            Some(parse_config_prism_attribute(buf, scratch).map(|t| t.len()))
        }
        "config_prism_enhance_s12" => {
            Some(parse_config_prism_enhance_s12(buf, scratch).map(|t| t.len()))
        }
        "config_prism_influence_shape" => {
            Some(parse_config_prism_influence_shape(buf, scratch).map(|t| t.len()))
        }
        "config_prism_modifier_mapping" => {
            Some(parse_config_prism_modifier_mapping(buf, scratch).map(|t| t.len()))
        }
        "config_talent_block_board" => {
            Some(parse_config_talent_block_board(buf, scratch).map(|t| t.len()))
        }
        "config_talent_block_detail_search" => {
            Some(parse_config_talent_block_detail_search(buf, scratch).map(|t| t.len()))
        }
        "config_talent_block_enhance" => {
            Some(parse_config_talent_block_enhance(buf, scratch).map(|t| t.len()))
        }
        "config_talent_block_enhance_cost" => {
            Some(parse_config_talent_block_enhance_cost(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

fn verify_talent_chunk_1(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_talent_block_enhance_s7" => {
            Some(parse_config_talent_block_enhance_s7(buf, scratch).map(|t| t.len()))
        }
        "config_talent_block_filter" => {
            Some(parse_config_talent_block_filter(buf, scratch).map(|t| t.len()))
        }
        "config_talent_block_influence" => {
            Some(parse_config_talent_block_influence(buf, scratch).map(|t| t.len()))
        }
        "config_talent_block_revert_s7" => {
            Some(parse_config_talent_block_revert_s7(buf, scratch).map(|t| t.len()))
        }
        "config_talent_block_shape" => {
            Some(parse_config_talent_block_shape(buf, scratch).map(|t| t.len()))
        }
        "config_talent_block_slot_recipe" => {
            Some(parse_config_talent_block_slot_recipe(buf, scratch).map(|t| t.len()))
        }
        "config_talent_filter" => Some(parse_config_talent_filter(buf, scratch).map(|t| t.len())),
        "config_talent_group_moon" => {
            Some(parse_config_talent_group_moon(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

pub(crate) fn verify_talent_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    if let Some(res) = verify_talent_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_talent_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    None
}

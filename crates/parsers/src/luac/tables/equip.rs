//! Auto-generated `LuaTable/DataCfg` types and parsers for the `equip` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;

/// Row record for `config_ashesconfig.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAshesconfigRow {
    pub ashes_id: u32,
    pub forge_expend: FxHashMap<u32, u8>,
    pub id: i16,
    pub increased_affix: u8,
    pub min: u8,
    pub num: u8,
}

impl DataCfgRow for ConfigAshesconfigRow {
    const TABLE_NAME: &'static str = "config_ashesconfig";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AshesID" => self.ashes_id = ctx.decode_field(val, Self::TABLE_NAME, "AshesID")?,
            b"ForgeExpend" => {
                self.forge_expend = ctx.decode_field(val, Self::TABLE_NAME, "ForgeExpend")?
            }
            b"ID" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "ID")?,
            b"IncreasedAffix" => {
                self.increased_affix = ctx.decode_field(val, Self::TABLE_NAME, "IncreasedAffix")?
            }
            b"Min" => self.min = ctx.decode_field(val, Self::TABLE_NAME, "Min")?,
            b"Num" => self.num = ctx.decode_field(val, Self::TABLE_NAME, "Num")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_ashesconfig.out`.
pub type ConfigAshesconfigTable = DataTable<u8, ConfigAshesconfigRow>;

/// Parses `config_ashesconfig.out` into a [`ConfigAshesconfigTable`].
pub fn parse_config_ashesconfig(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAshesconfigTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_candle_combine_cost.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCandleCombineCostRow {
    pub cost_num: u8,
    pub id: i8,
    pub resouce_lv: Box<[u16]>,
}

impl DataCfgRow for ConfigCandleCombineCostRow {
    const TABLE_NAME: &'static str = "config_candle_combine_cost";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CostNum" => self.cost_num = ctx.decode_field(val, Self::TABLE_NAME, "CostNum")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ResouceLv" => {
                self.resouce_lv = ctx.decode_field(val, Self::TABLE_NAME, "ResouceLv")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_candle_combine_cost.out`.
pub type ConfigCandleCombineCostTable = DataTable<u8, ConfigCandleCombineCostRow>;

/// Parses `config_candle_combine_cost.out` into a [`ConfigCandleCombineCostTable`].
pub fn parse_config_candle_combine_cost(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCandleCombineCostTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_comburent.Function`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ComburentFunction {
    pub kind: u8,
    pub value: u8,
}

impl FromEvalVal for ComburentFunction {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (kind, value) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { kind, value })
    }
}

/// Row record for `config_comburent.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigComburentRow {
    pub function: ComburentFunction,
    pub id: i32,
}

impl DataCfgRow for ConfigComburentRow {
    const TABLE_NAME: &'static str = "config_comburent";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"ID" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "ID")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_comburent.out`.
pub type ConfigComburentTable = DataTable<u32, ConfigComburentRow>;

/// Parses `config_comburent.out` into a [`ConfigComburentTable`].
pub fn parse_config_comburent(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigComburentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_corrosion_consume.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCorrosionConsumeRow {
    pub consume: FxHashMap<Box<str>, u8>,
    pub corrosion_type: u8,
    pub id: i16,
    pub item_level: u16,
}

impl DataCfgRow for ConfigCorrosionConsumeRow {
    const TABLE_NAME: &'static str = "config_corrosion_consume";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Consume" => self.consume = ctx.decode_field(val, Self::TABLE_NAME, "Consume")?,
            b"CorrosionType" => {
                self.corrosion_type = ctx.decode_field(val, Self::TABLE_NAME, "CorrosionType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_corrosion_consume.out`.
pub type ConfigCorrosionConsumeTable = DataTable<u16, ConfigCorrosionConsumeRow>;

/// Parses `config_corrosion_consume.out` into a [`ConfigCorrosionConsumeTable`].
pub fn parse_config_corrosion_consume(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCorrosionConsumeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_corrosion_effect.Parameter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CorrosionEffectParameterEntry {
    pub kind: u8,
    pub value: u8,
}

impl FromEvalVal for CorrosionEffectParameterEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (kind, value) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { kind, value })
    }
}

/// Row record for `config_corrosion_effect.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCorrosionEffectRow {
    pub bind: u8,
    pub content: u8,
    pub description: Box<str>,
    pub extra_param: u8,
    pub highlight: u8,
    pub icon: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub parameter: Box<[CorrosionEffectParameterEntry]>,
    pub sort: u8,
    pub _description: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigCorrosionEffectRow {
    const TABLE_NAME: &'static str = "config_corrosion_effect";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Bind" => self.bind = ctx.decode_field(val, Self::TABLE_NAME, "Bind")?,
            b"Content" => self.content = ctx.decode_field(val, Self::TABLE_NAME, "Content")?,
            b"Description" => {
                self.description = ctx.decode_field(val, Self::TABLE_NAME, "Description")?
            }
            b"ExtraParam" => {
                self.extra_param = ctx.decode_field(val, Self::TABLE_NAME, "ExtraParam")?
            }
            b"Highlight" => {
                self.highlight = ctx.decode_field(val, Self::TABLE_NAME, "Highlight")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Parameter" => {
                self.parameter = ctx.decode_field(val, Self::TABLE_NAME, "Parameter")?
            }
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_corrosion_effect.out`.
pub type ConfigCorrosionEffectTable = DataTable<u8, ConfigCorrosionEffectRow>;

/// Parses `config_corrosion_effect.out` into a [`ConfigCorrosionEffectTable`].
pub fn parse_config_corrosion_effect(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCorrosionEffectTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_corrosion_special.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCorrosionSpecialRow {
    pub corrosion_effect_new: u8,
    pub corrosion_effect_old: u8,
    pub group: u32,
    pub id: i8,
    pub tier: u8,
}

impl DataCfgRow for ConfigCorrosionSpecialRow {
    const TABLE_NAME: &'static str = "config_corrosion_special";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CorrosionEffectNew" => {
                self.corrosion_effect_new =
                    ctx.decode_field(val, Self::TABLE_NAME, "CorrosionEffectNew")?
            }
            b"CorrosionEffectOld" => {
                self.corrosion_effect_old =
                    ctx.decode_field(val, Self::TABLE_NAME, "CorrosionEffectOld")?
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Tier" => self.tier = ctx.decode_field(val, Self::TABLE_NAME, "Tier")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_corrosion_special.out`.
pub type ConfigCorrosionSpecialTable = DataTable<u8, ConfigCorrosionSpecialRow>;

/// Parses `config_corrosion_special.out` into a [`ConfigCorrosionSpecialTable`].
pub fn parse_config_corrosion_special(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCorrosionSpecialTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_corrosion_type.Effect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CorrosionTypeEffectEntry {
    pub kind: u8,
    pub value: u8,
}

impl FromEvalVal for CorrosionTypeEffectEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (kind, value) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { kind, value })
    }
}

/// Row record for `config_corrosion_type.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCorrosionTypeRow {
    pub effect: Box<[CorrosionTypeEffectEntry]>,
    pub extra_corrosion_group: Box<[u32]>,
    pub extra_corrosion_weight: Box<[u8]>,
    pub function: u16,
    pub id: i8,
    pub limited_itemlevel: u16,
    pub rarity: Box<[u8]>,
    pub r#type: u8,
}

impl DataCfgRow for ConfigCorrosionTypeRow {
    const TABLE_NAME: &'static str = "config_corrosion_type";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Effect" => self.effect = ctx.decode_field(val, Self::TABLE_NAME, "Effect")?,
            b"ExtraCorrosionGroup" => {
                self.extra_corrosion_group =
                    ctx.decode_field(val, Self::TABLE_NAME, "ExtraCorrosionGroup")?
            }
            b"ExtraCorrosionWeight" => {
                self.extra_corrosion_weight =
                    ctx.decode_field(val, Self::TABLE_NAME, "ExtraCorrosionWeight")?
            }
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LimitedItemlevel" => {
                self.limited_itemlevel =
                    ctx.decode_field(val, Self::TABLE_NAME, "LimitedItemlevel")?
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_corrosion_type.out`.
pub type ConfigCorrosionTypeTable = DataTable<u8, ConfigCorrosionTypeRow>;

/// Parses `config_corrosion_type.out` into a [`ConfigCorrosionTypeTable`].
pub fn parse_config_corrosion_type(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCorrosionTypeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_cost_forgel.Consume`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CostForgelConsume {
    pub item_id: u16,
    pub count: u8,
}

impl FromEvalVal for CostForgelConsume {
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

/// Entry for `config_cost_forgel.CostWeight`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CostForgelCostWeightEntry {
    pub level_range: Box<str>,
    pub weight: u8,
}

impl FromEvalVal for CostForgelCostWeightEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (level_range, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            level_range,
            weight,
        })
    }
}

/// Row record for `config_cost_forgel.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCostForgelRow {
    pub consume: CostForgelConsume,
    pub cost_weight: Box<[CostForgelCostWeightEntry]>,
    pub id: i16,
    pub item_level: Box<[u16]>,
}

impl DataCfgRow for ConfigCostForgelRow {
    const TABLE_NAME: &'static str = "config_cost_forgel";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Consume" => self.consume = ctx.decode_field(val, Self::TABLE_NAME, "Consume")?,
            b"CostWeight" => {
                self.cost_weight = ctx.decode_field(val, Self::TABLE_NAME, "CostWeight")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_cost_forgel.out`.
pub type ConfigCostForgelTable = DataTable<u16, ConfigCostForgelRow>;

/// Parses `config_cost_forgel.out` into a [`ConfigCostForgelTable`].
pub fn parse_config_cost_forgel(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCostForgelTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_enchantment_type.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEnchantmentTypeRow {
    pub id: i8,
    pub sort: u8,
    pub text: Box<str>,
    pub _text: Box<str>,
}

impl DataCfgRow for ConfigEnchantmentTypeRow {
    const TABLE_NAME: &'static str = "config_enchantment_type";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"Text" => self.text = ctx.decode_field(val, Self::TABLE_NAME, "Text")?,
            b"_Text" => self._text = ctx.decode_field(val, Self::TABLE_NAME, "_Text")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_enchantment_type.out`.
pub type ConfigEnchantmentTypeTable = DataTable<u8, ConfigEnchantmentTypeRow>;

/// Parses `config_enchantment_type.out` into a [`ConfigEnchantmentTypeTable`].
pub fn parse_config_enchantment_type(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEnchantmentTypeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_enchantmentconfig.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEnchantmentconfigRow {
    pub affix_type: u8,
    pub effect: u32,
    pub enchantment_expend: FxHashMap<u32, u8>,
    pub enchantment_id: i16,
    pub position: Box<[u8]>,
    pub quality: u8,
    pub r#type: u8,
    pub unlock_condition: FxHashMap<u8, u8>,
}

impl DataCfgRow for ConfigEnchantmentconfigRow {
    const TABLE_NAME: &'static str = "config_enchantmentconfig";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AffixType" => {
                self.affix_type = ctx.decode_field(val, Self::TABLE_NAME, "AffixType")?
            }
            b"Effect" => self.effect = ctx.decode_field(val, Self::TABLE_NAME, "Effect")?,
            b"EnchantmentExpend" => {
                self.enchantment_expend =
                    ctx.decode_field(val, Self::TABLE_NAME, "EnchantmentExpend")?
            }
            b"EnchantmentID" => {
                self.enchantment_id = ctx.decode_field(val, Self::TABLE_NAME, "EnchantmentID")?
            }
            b"Position" => self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?,
            b"Quality" => self.quality = ctx.decode_field(val, Self::TABLE_NAME, "Quality")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"UnlockCondition" => {
                self.unlock_condition =
                    ctx.decode_field(val, Self::TABLE_NAME, "UnlockCondition")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_enchantmentconfig.out`.
pub type ConfigEnchantmentconfigTable = DataTable<u16, ConfigEnchantmentconfigRow>;

/// Parses `config_enchantmentconfig.out` into a [`ConfigEnchantmentconfigTable`].
pub fn parse_config_enchantmentconfig(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEnchantmentconfigTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_equip_base_affix_consume.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEquipBaseAffixConsumeRow {
    pub consume: FxHashMap<u32, u8>,
    pub id: i8,
    pub type4: u8,
}

impl DataCfgRow for ConfigEquipBaseAffixConsumeRow {
    const TABLE_NAME: &'static str = "config_equip_base_affix_consume";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Consume" => self.consume = ctx.decode_field(val, Self::TABLE_NAME, "Consume")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_equip_base_affix_consume.out`.
pub type ConfigEquipBaseAffixConsumeTable = DataTable<u8, ConfigEquipBaseAffixConsumeRow>;

/// Parses `config_equip_base_affix_consume.out` into a [`ConfigEquipBaseAffixConsumeTable`].
pub fn parse_config_equip_base_affix_consume(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEquipBaseAffixConsumeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_equip_chip_list.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEquipChipListRow {
    pub icon_max: Box<str>,
    pub icon_min: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub tier: u8,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigEquipChipListRow {
    const TABLE_NAME: &'static str = "config_equip_chip_list";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"IconMax" => self.icon_max = ctx.decode_field(val, Self::TABLE_NAME, "IconMax")?,
            b"IconMin" => self.icon_min = ctx.decode_field(val, Self::TABLE_NAME, "IconMin")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Tier" => self.tier = ctx.decode_field(val, Self::TABLE_NAME, "Tier")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_equip_chip_list.out`.
pub type ConfigEquipChipListTable = DataTable<u8, ConfigEquipChipListRow>;

/// Parses `config_equip_chip_list.out` into a [`ConfigEquipChipListTable`].
pub fn parse_config_equip_chip_list(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEquipChipListTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_equip_chip_s11.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEquipChipS11Row {
    pub consume_high: FxHashMap<u32, u8>,
    pub consume_low: FxHashMap<u32, u8>,
    pub id: i8,
    pub level: Box<[u8]>,
    pub rarity: Box<[u8]>,
    pub type1: Box<[u8]>,
    pub type4: Box<[u8]>,
}

impl DataCfgRow for ConfigEquipChipS11Row {
    const TABLE_NAME: &'static str = "config_equip_chip_s11";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ConsumeHigh" => {
                self.consume_high = ctx.decode_field(val, Self::TABLE_NAME, "ConsumeHigh")?
            }
            b"ConsumeLow" => {
                self.consume_low = ctx.decode_field(val, Self::TABLE_NAME, "ConsumeLow")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_equip_chip_s11.out`.
pub type ConfigEquipChipS11Table = DataTable<u8, ConfigEquipChipS11Row>;

/// Parses `config_equip_chip_s11.out` into a [`ConfigEquipChipS11Table`].
pub fn parse_config_equip_chip_s11(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEquipChipS11Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_equip_chipmatching_s11.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEquipChipmatchingS11Row {
    pub affix_name: Box<str>,
    pub affix_tier: u8,
    pub chipmatch: Box<[u8]>,
    pub id: i16,
    pub modifier_type: u8,
    pub show: u8,
    pub tags: u16,
    pub _affix_name: Box<str>,
}

impl DataCfgRow for ConfigEquipChipmatchingS11Row {
    const TABLE_NAME: &'static str = "config_equip_chipmatching_s11";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AffixName" => {
                self.affix_name = ctx.decode_field(val, Self::TABLE_NAME, "AffixName")?
            }
            b"AffixTier" => {
                self.affix_tier = ctx.decode_field(val, Self::TABLE_NAME, "AffixTier")?
            }
            b"Chipmatch" => {
                self.chipmatch = ctx.decode_field(val, Self::TABLE_NAME, "Chipmatch")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ModifierType" => {
                self.modifier_type = ctx.decode_field(val, Self::TABLE_NAME, "ModifierType")?
            }
            b"Show" => self.show = ctx.decode_field(val, Self::TABLE_NAME, "Show")?,
            b"Tags" => self.tags = ctx.decode_field(val, Self::TABLE_NAME, "Tags")?,
            b"_AffixName" => {
                self._affix_name = ctx.decode_field(val, Self::TABLE_NAME, "_AffixName")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_equip_chipmatching_s11.out`.
pub type ConfigEquipChipmatchingS11Table = DataTable<u16, ConfigEquipChipmatchingS11Row>;

/// Parses `config_equip_chipmatching_s11.out` into a [`ConfigEquipChipmatchingS11Table`].
pub fn parse_config_equip_chipmatching_s11(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEquipChipmatchingS11Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_equip_chipmatching_type.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEquipChipmatchingTypeRow {
    pub chipmatch_num: Box<[u8]>,
    pub id: i8,
    pub match_num: u8,
}

impl DataCfgRow for ConfigEquipChipmatchingTypeRow {
    const TABLE_NAME: &'static str = "config_equip_chipmatching_type";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ChipmatchNum" => {
                self.chipmatch_num = ctx.decode_field(val, Self::TABLE_NAME, "ChipmatchNum")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MatchNum" => self.match_num = ctx.decode_field(val, Self::TABLE_NAME, "MatchNum")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_equip_chipmatching_type.out`.
pub type ConfigEquipChipmatchingTypeTable = DataTable<u8, ConfigEquipChipmatchingTypeRow>;

/// Parses `config_equip_chipmatching_type.out` into a [`ConfigEquipChipmatchingTypeTable`].
pub fn parse_config_equip_chipmatching_type(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEquipChipmatchingTypeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_equip_enchant_s10.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEquipEnchantS10Row {
    pub enchant_affix_tier: FxHashMap<u32, u8>,
    pub enchant_affix_type: u8,
    pub enchant_consume: FxHashMap<u32, u8>,
    pub enchant_desc: Box<str>,
    pub enchant_name: Box<str>,
    pub enchant_rarity: u8,
    pub id: i32,
    pub level: Box<[u8]>,
    pub rarity: Box<[u8]>,
    pub show: u8,
    pub type4: Box<[u8]>,
    pub unlock_event: u8,
    pub unlock_level: u8,
    pub _enchant_desc: Box<str>,
    pub _enchant_name: Box<str>,
}

impl ConfigEquipEnchantS10Row {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"EnchantAffixTier" => {
                self.enchant_affix_tier =
                    ctx.decode_field(val, Self::TABLE_NAME, "EnchantAffixTier")?
            }
            b"EnchantAffixType" => {
                self.enchant_affix_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "EnchantAffixType")?
            }
            b"EnchantConsume" => {
                self.enchant_consume = ctx.decode_field(val, Self::TABLE_NAME, "EnchantConsume")?
            }
            b"EnchantDesc" => {
                self.enchant_desc = ctx.decode_field(val, Self::TABLE_NAME, "EnchantDesc")?
            }
            b"EnchantName" => {
                self.enchant_name = ctx.decode_field(val, Self::TABLE_NAME, "EnchantName")?
            }
            b"EnchantRarity" => {
                self.enchant_rarity = ctx.decode_field(val, Self::TABLE_NAME, "EnchantRarity")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Show" => self.show = ctx.decode_field(val, Self::TABLE_NAME, "Show")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            b"UnlockEvent" => {
                self.unlock_event = ctx.decode_field(val, Self::TABLE_NAME, "UnlockEvent")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    #[inline(never)]
    fn set_field_1(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"UnlockLevel" => {
                self.unlock_level = ctx.decode_field(val, Self::TABLE_NAME, "UnlockLevel")?
            }
            b"_EnchantDesc" => {
                self._enchant_desc = ctx.decode_field(val, Self::TABLE_NAME, "_EnchantDesc")?
            }
            b"_EnchantName" => {
                self._enchant_name = ctx.decode_field(val, Self::TABLE_NAME, "_EnchantName")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigEquipEnchantS10Row {
    const TABLE_NAME: &'static str = "config_equip_enchant_s10";

    #[inline(never)]
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

/// Typed `DataTable` for `config_equip_enchant_s10.out`.
pub type ConfigEquipEnchantS10Table = DataTable<u32, ConfigEquipEnchantS10Row>;

/// Parses `config_equip_enchant_s10.out` into a [`ConfigEquipEnchantS10Table`].
pub fn parse_config_equip_enchant_s10(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEquipEnchantS10Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_equip_position_s13.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEquipPositionS13Row {
    pub id: i8,
    pub name: Box<str>,
    pub target_type3_list: Box<[u8]>,
    pub type3: u8,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigEquipPositionS13Row {
    const TABLE_NAME: &'static str = "config_equip_position_s13";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"TargetType3List" => {
                self.target_type3_list =
                    ctx.decode_field(val, Self::TABLE_NAME, "TargetType3List")?
            }
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_equip_position_s13.out`.
pub type ConfigEquipPositionS13Table = DataTable<u8, ConfigEquipPositionS13Row>;

/// Parses `config_equip_position_s13.out` into a [`ConfigEquipPositionS13Table`].
pub fn parse_config_equip_position_s13(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEquipPositionS13Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_install_gold.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigInstallGoldRow {
    pub gold_id: i16,
    pub limit_num: u8,
}

impl DataCfgRow for ConfigInstallGoldRow {
    const TABLE_NAME: &'static str = "config_install_gold";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"GoldId" => self.gold_id = ctx.decode_field(val, Self::TABLE_NAME, "GoldId")?,
            b"LimitNum" => self.limit_num = ctx.decode_field(val, Self::TABLE_NAME, "LimitNum")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_install_gold.out`.
pub type ConfigInstallGoldTable = DataTable<u16, ConfigInstallGoldRow>;

/// Parses `config_install_gold.out` into a [`ConfigInstallGoldTable`].
pub fn parse_config_install_gold(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigInstallGoldTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_memory_awaken_recipe_s14.AwakenPart2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MemoryAwakenRecipeS14AwakenPart2Entry {
    pub boost_id: u8,
    pub probability: u8,
}

impl FromEvalVal for MemoryAwakenRecipeS14AwakenPart2Entry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (boost_id, probability) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            boost_id,
            probability,
        })
    }
}

/// Entry for `config_memory_awaken_recipe_s14.CostItem`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MemoryAwakenRecipeS14CostItemEntry {
    pub item_id: u32,
    pub count: u8,
}

impl FromEvalVal for MemoryAwakenRecipeS14CostItemEntry {
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

/// Entry for `config_memory_awaken_recipe_s14.CostItemSp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MemoryAwakenRecipeS14CostItemSpEntry {
    pub item_id: u32,
    pub count: u8,
}

impl FromEvalVal for MemoryAwakenRecipeS14CostItemSpEntry {
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

/// Row record for `config_memory_awaken_recipe_s14.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMemoryAwakenRecipeS14Row {
    pub awaken_part1: u8,
    pub awaken_part2: Box<[MemoryAwakenRecipeS14AwakenPart2Entry]>,
    pub awaken_part3: Box<[u8]>,
    pub awaken_part4: FxHashMap<u8, u8>,
    pub awaken_part4_sp: FxHashMap<u16, u8>,
    pub cost_item: Box<[MemoryAwakenRecipeS14CostItemEntry]>,
    pub cost_item_new: FxHashMap<u8, Box<[u32]>>,
    pub cost_item_sp: Box<[MemoryAwakenRecipeS14CostItemSpEntry]>,
    pub cost_item_sp_new: FxHashMap<u8, Box<[u32]>>,
    pub id: i8,
    pub is_valid: bool,
    pub memory_level: Box<[u16]>,
    pub memory_rarity: u8,
    pub memory_type2: Box<[u8]>,
    pub order: u8,
    pub part4_isbind: u8,
    pub part4_sp_isbind: u8,
}

impl ConfigMemoryAwakenRecipeS14Row {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AwakenPart1" => {
                self.awaken_part1 = ctx.decode_field(val, Self::TABLE_NAME, "AwakenPart1")?
            }
            b"AwakenPart2" => {
                self.awaken_part2 = ctx.decode_field(val, Self::TABLE_NAME, "AwakenPart2")?
            }
            b"AwakenPart3" => {
                self.awaken_part3 = ctx.decode_field(val, Self::TABLE_NAME, "AwakenPart3")?
            }
            b"AwakenPart4" => {
                self.awaken_part4 = ctx.decode_field(val, Self::TABLE_NAME, "AwakenPart4")?
            }
            b"AwakenPart4Sp" => {
                self.awaken_part4_sp = ctx.decode_field(val, Self::TABLE_NAME, "AwakenPart4Sp")?
            }
            b"CostItem" => self.cost_item = ctx.decode_field(val, Self::TABLE_NAME, "CostItem")?,
            b"CostItemNew" => {
                self.cost_item_new = ctx.decode_field(val, Self::TABLE_NAME, "CostItemNew")?
            }
            b"CostItemSp" => {
                self.cost_item_sp = ctx.decode_field(val, Self::TABLE_NAME, "CostItemSp")?
            }
            b"CostItemSpNew" => {
                self.cost_item_sp_new = ctx.decode_field(val, Self::TABLE_NAME, "CostItemSpNew")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsValid" => self.is_valid = ctx.decode_field(val, Self::TABLE_NAME, "IsValid")?,
            b"MemoryLevel" => {
                self.memory_level = ctx.decode_field(val, Self::TABLE_NAME, "MemoryLevel")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    #[inline(never)]
    fn set_field_1(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"MemoryRarity" => {
                self.memory_rarity = ctx.decode_field(val, Self::TABLE_NAME, "MemoryRarity")?
            }
            b"MemoryType2" => {
                self.memory_type2 = ctx.decode_field(val, Self::TABLE_NAME, "MemoryType2")?
            }
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"Part4Isbind" => {
                self.part4_isbind = ctx.decode_field(val, Self::TABLE_NAME, "Part4Isbind")?
            }
            b"Part4SpIsbind" => {
                self.part4_sp_isbind = ctx.decode_field(val, Self::TABLE_NAME, "Part4SpIsbind")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigMemoryAwakenRecipeS14Row {
    const TABLE_NAME: &'static str = "config_memory_awaken_recipe_s14";

    #[inline(never)]
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

/// Typed `DataTable` for `config_memory_awaken_recipe_s14.out`.
pub type ConfigMemoryAwakenRecipeS14Table = DataTable<u8, ConfigMemoryAwakenRecipeS14Row>;

/// Parses `config_memory_awaken_recipe_s14.out` into a [`ConfigMemoryAwakenRecipeS14Table`].
pub fn parse_config_memory_awaken_recipe_s14(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMemoryAwakenRecipeS14Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_memory_awaken_times.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMemoryAwakenTimesRow {
    pub id: i8,
    pub max_awaken_cost: u8,
    pub max_awaken_times: i8,
    pub memory_level: Box<[u16]>,
    pub memory_rarity: u8,
    pub memory_type2: Box<[u8]>,
}

impl DataCfgRow for ConfigMemoryAwakenTimesRow {
    const TABLE_NAME: &'static str = "config_memory_awaken_times";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MaxAwakenCost" => {
                self.max_awaken_cost = ctx.decode_field(val, Self::TABLE_NAME, "MaxAwakenCost")?
            }
            b"MaxAwakenTimes" => {
                self.max_awaken_times = ctx.decode_field(val, Self::TABLE_NAME, "MaxAwakenTimes")?
            }
            b"MemoryLevel" => {
                self.memory_level = ctx.decode_field(val, Self::TABLE_NAME, "MemoryLevel")?
            }
            b"MemoryRarity" => {
                self.memory_rarity = ctx.decode_field(val, Self::TABLE_NAME, "MemoryRarity")?
            }
            b"MemoryType2" => {
                self.memory_type2 = ctx.decode_field(val, Self::TABLE_NAME, "MemoryType2")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_memory_awaken_times.out`.
pub type ConfigMemoryAwakenTimesTable = DataTable<u8, ConfigMemoryAwakenTimesRow>;

/// Parses `config_memory_awaken_times.out` into a [`ConfigMemoryAwakenTimesTable`].
pub fn parse_config_memory_awaken_times(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMemoryAwakenTimesTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_mercenary_equip_transform.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMercenaryEquipTransformRow {
    pub cost: FxHashMap<u32, u16>,
    pub id: i32,
    pub result_id: u32,
}

impl DataCfgRow for ConfigMercenaryEquipTransformRow {
    const TABLE_NAME: &'static str = "config_mercenary_equip_transform";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ResultId" => self.result_id = ctx.decode_field(val, Self::TABLE_NAME, "ResultId")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_mercenary_equip_transform.out`.
pub type ConfigMercenaryEquipTransformTable = DataTable<u32, ConfigMercenaryEquipTransformRow>;

/// Parses `config_mercenary_equip_transform.out` into a [`ConfigMercenaryEquipTransformTable`].
pub fn parse_config_mercenary_equip_transform(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMercenaryEquipTransformTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_rarity_equip.AffixNum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RarityEquipAffixNumEntry {
    pub count: u8,
    pub weight: u16,
}

impl FromEvalVal for RarityEquipAffixNumEntry {
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

/// Entry for `config_rarity_equip.AffixSlotNum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RarityEquipAffixSlotNumEntry {
    pub group: u8,
    pub min_slots: u8,
    pub max_slots: u8,
    pub weight: u8,
}

impl FromEvalVal for RarityEquipAffixSlotNumEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (group, (min_slots, max_slots, weight)) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            group,
            min_slots,
            max_slots,
            weight,
        })
    }
}

/// Entry for `config_rarity_equip.BaseAffixNum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RarityEquipBaseAffixNumEntry {
    pub count: u8,
    pub weight: u16,
}

impl FromEvalVal for RarityEquipBaseAffixNumEntry {
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

/// Entry for `config_rarity_equip.BaseTier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RarityEquipBaseTierEntry {
    pub tier: u8,
    pub weight: u16,
}

impl FromEvalVal for RarityEquipBaseTierEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (tier, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { tier, weight })
    }
}

/// Row record for `config_rarity_equip.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRarityEquipRow {
    pub affix_num: Box<[RarityEquipAffixNumEntry]>,
    pub affix_slot_num: Box<[RarityEquipAffixSlotNumEntry]>,
    pub base_affix_num: Box<[RarityEquipBaseAffixNumEntry]>,
    pub base_tier: Box<[RarityEquipBaseTierEntry]>,
    pub drop_affix_tier: [u8; 0],
    pub id: i8,
    pub level: Box<[u8]>,
    pub min_base_tier1_num: u8,
    pub rarity: u8,
    pub type1: u8,
}

impl DataCfgRow for ConfigRarityEquipRow {
    const TABLE_NAME: &'static str = "config_rarity_equip";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AffixNum" => self.affix_num = ctx.decode_field(val, Self::TABLE_NAME, "AffixNum")?,
            b"AffixSlotNum" => {
                self.affix_slot_num = ctx.decode_field(val, Self::TABLE_NAME, "AffixSlotNum")?
            }
            b"BaseAffixNum" => {
                self.base_affix_num = ctx.decode_field(val, Self::TABLE_NAME, "BaseAffixNum")?
            }
            b"BaseTier" => self.base_tier = ctx.decode_field(val, Self::TABLE_NAME, "BaseTier")?,
            b"DropAffixTier" => {
                self.drop_affix_tier = ctx.decode_field(val, Self::TABLE_NAME, "DropAffixTier")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"MinBaseTier1Num" => {
                self.min_base_tier1_num =
                    ctx.decode_field(val, Self::TABLE_NAME, "MinBaseTier1Num")?
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_rarity_equip.out`.
pub type ConfigRarityEquipTable = DataTable<u8, ConfigRarityEquipRow>;

/// Parses `config_rarity_equip.out` into a [`ConfigRarityEquipTable`].
pub fn parse_config_rarity_equip(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigRarityEquipTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_sift_affix.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSiftAffixRow {
    pub icon: Box<str>,
    pub id: i8,
    pub parameter: u32,
    pub text: Box<str>,
    pub r#type: u8,
    pub _text: Box<str>,
}

impl DataCfgRow for ConfigSiftAffixRow {
    const TABLE_NAME: &'static str = "config_sift_affix";

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
            b"Parameter" => {
                self.parameter = ctx.decode_field(val, Self::TABLE_NAME, "Parameter")?
            }
            b"Text" => self.text = ctx.decode_field(val, Self::TABLE_NAME, "Text")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Text" => self._text = ctx.decode_field(val, Self::TABLE_NAME, "_Text")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sift_affix.out`.
pub type ConfigSiftAffixTable = DataTable<u8, ConfigSiftAffixRow>;

/// Parses `config_sift_affix.out` into a [`ConfigSiftAffixTable`].
pub fn parse_config_sift_affix(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSiftAffixTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_sift_equip.Affix`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SiftEquipAffixEntry {
    pub affix_id: u32,
    pub weight: u8,
}

impl FromEvalVal for SiftEquipAffixEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (affix_id, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { affix_id, weight })
    }
}

/// Entry for `config_sift_equip.AshesAffixType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SiftEquipAshesAffixTypeEntry {
    pub affix_id: u32,
    pub affix_type: u8,
}

impl FromEvalVal for SiftEquipAshesAffixTypeEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (affix_id, affix_type) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            affix_id,
            affix_type,
        })
    }
}

/// Entry for `config_sift_equip.Enchantment`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SiftEquipEnchantmentEntry {
    pub enchantment_id: u16,
    pub weight: u8,
}

impl FromEvalVal for SiftEquipEnchantmentEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (enchantment_id, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            enchantment_id,
            weight,
        })
    }
}

/// Row record for `config_sift_equip.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSiftEquipRow {
    pub affix: Box<[SiftEquipAffixEntry]>,
    pub ashes_affix_type: Box<[SiftEquipAshesAffixTypeEntry]>,
    pub enchantment: Box<[SiftEquipEnchantmentEntry]>,
    pub id: i8,
}

impl DataCfgRow for ConfigSiftEquipRow {
    const TABLE_NAME: &'static str = "config_sift_equip";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Affix" => self.affix = ctx.decode_field(val, Self::TABLE_NAME, "Affix")?,
            b"AshesAffixType" => {
                self.ashes_affix_type = ctx.decode_field(val, Self::TABLE_NAME, "AshesAffixType")?
            }
            b"Enchantment" => {
                self.enchantment = ctx.decode_field(val, Self::TABLE_NAME, "Enchantment")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sift_equip.out`.
pub type ConfigSiftEquipTable = DataTable<u8, ConfigSiftEquipRow>;

/// Parses `config_sift_equip.out` into a [`ConfigSiftEquipTable`].
pub fn parse_config_sift_equip(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSiftEquipTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_sort_ashesconfig.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSortAshesconfigRow {
    pub ashes_id: u32,
    pub id: i8,
    pub sort: u8,
    pub r#type: u8,
}

impl DataCfgRow for ConfigSortAshesconfigRow {
    const TABLE_NAME: &'static str = "config_sort_ashesconfig";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AshesID" => self.ashes_id = ctx.decode_field(val, Self::TABLE_NAME, "AshesID")?,
            b"ID" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "ID")?,
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sort_ashesconfig.out`.
pub type ConfigSortAshesconfigTable = DataTable<u8, ConfigSortAshesconfigRow>;

/// Parses `config_sort_ashesconfig.out` into a [`ConfigSortAshesconfigTable`].
pub fn parse_config_sort_ashesconfig(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSortAshesconfigTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

fn verify_equip_chunk_0(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_ashesconfig" => Some(parse_config_ashesconfig(buf, scratch).map(|t| t.len())),
        "config_candle_combine_cost" => {
            Some(parse_config_candle_combine_cost(buf, scratch).map(|t| t.len()))
        }
        "config_comburent" => Some(parse_config_comburent(buf, scratch).map(|t| t.len())),
        "config_corrosion_consume" => {
            Some(parse_config_corrosion_consume(buf, scratch).map(|t| t.len()))
        }
        "config_corrosion_effect" => {
            Some(parse_config_corrosion_effect(buf, scratch).map(|t| t.len()))
        }
        "config_corrosion_special" => {
            Some(parse_config_corrosion_special(buf, scratch).map(|t| t.len()))
        }
        "config_corrosion_type" => Some(parse_config_corrosion_type(buf, scratch).map(|t| t.len())),
        "config_cost_forgel" => Some(parse_config_cost_forgel(buf, scratch).map(|t| t.len())),
        "config_enchantment_type" => {
            Some(parse_config_enchantment_type(buf, scratch).map(|t| t.len()))
        }
        "config_enchantmentconfig" => {
            Some(parse_config_enchantmentconfig(buf, scratch).map(|t| t.len()))
        }
        "config_equip_base_affix_consume" => {
            Some(parse_config_equip_base_affix_consume(buf, scratch).map(|t| t.len()))
        }
        "config_equip_chip_list" => {
            Some(parse_config_equip_chip_list(buf, scratch).map(|t| t.len()))
        }
        "config_equip_chip_s11" => Some(parse_config_equip_chip_s11(buf, scratch).map(|t| t.len())),
        "config_equip_chipmatching_s11" => {
            Some(parse_config_equip_chipmatching_s11(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

fn verify_equip_chunk_1(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_equip_chipmatching_type" => {
            Some(parse_config_equip_chipmatching_type(buf, scratch).map(|t| t.len()))
        }
        "config_equip_enchant_s10" => {
            Some(parse_config_equip_enchant_s10(buf, scratch).map(|t| t.len()))
        }
        "config_equip_position_s13" => {
            Some(parse_config_equip_position_s13(buf, scratch).map(|t| t.len()))
        }
        "config_install_gold" => Some(parse_config_install_gold(buf, scratch).map(|t| t.len())),
        "config_memory_awaken_recipe_s14" => {
            Some(parse_config_memory_awaken_recipe_s14(buf, scratch).map(|t| t.len()))
        }
        "config_memory_awaken_times" => {
            Some(parse_config_memory_awaken_times(buf, scratch).map(|t| t.len()))
        }
        "config_mercenary_equip_transform" => {
            Some(parse_config_mercenary_equip_transform(buf, scratch).map(|t| t.len()))
        }
        "config_rarity_equip" => Some(parse_config_rarity_equip(buf, scratch).map(|t| t.len())),
        "config_sift_affix" => Some(parse_config_sift_affix(buf, scratch).map(|t| t.len())),
        "config_sift_equip" => Some(parse_config_sift_equip(buf, scratch).map(|t| t.len())),
        "config_sort_ashesconfig" => {
            Some(parse_config_sort_ashesconfig(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

pub(crate) fn verify_equip_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    if let Some(res) = verify_equip_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_equip_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    None
}

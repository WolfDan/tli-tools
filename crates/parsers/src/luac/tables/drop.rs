//! Auto-generated `LuaTable/DataCfg` types and parsers for the `drop` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;

/// Row record for `config_drop_base.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropBaseRow {
    pub hero_purchase: Box<[u16]>,
    pub id: i32,
    pub item_id: u32,
    pub level: Box<[u16]>,
    pub num: u16,
    pub rarity: u8,
    pub skill_level: u8,
    pub weight: u8,
}

impl DataCfgRow for ConfigDropBaseRow {
    const TABLE_NAME: &'static str = "config_drop_base";

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
            b"ItemId" => self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Num" => self.num = ctx.decode_field(val, Self::TABLE_NAME, "Num")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"SkillLevel" => {
                self.skill_level = ctx.decode_field(val, Self::TABLE_NAME, "SkillLevel")?
            }
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_drop_base.out`.
pub type ConfigDropBaseTable = DataTable<u32, ConfigDropBaseRow>;

/// Parses `config_drop_base.out` into a [`ConfigDropBaseTable`].
pub fn parse_config_drop_base(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropBaseTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_drop_group.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropGroupRow {
    pub args: u8,
    pub base_id: Box<str>,
    pub des: Box<str>,
    pub group_id: i32,
    pub r#type: u8,
}

impl DataCfgRow for ConfigDropGroupRow {
    const TABLE_NAME: &'static str = "config_drop_group";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Args" => self.args = ctx.decode_field(val, Self::TABLE_NAME, "Args")?,
            b"BaseId" => self.base_id = ctx.decode_field(val, Self::TABLE_NAME, "BaseId")?,
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"GroupId" => self.group_id = ctx.decode_field(val, Self::TABLE_NAME, "GroupId")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_drop_group.out`.
pub type ConfigDropGroupTable = DataTable<u32, ConfigDropGroupRow>;

/// Parses `config_drop_group.out` into a [`ConfigDropGroupTable`].
pub fn parse_config_drop_group(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropGroupTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_drop_type_info.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropTypeInfoRow {
    pub des: Box<str>,
    pub icon: Box<str>,
    pub id: i8,
    pub istips: bool,
    pub level: Box<[u8]>,
    pub pic: Box<str>,
    pub rarity: Box<[u8]>,
    pub type1: u8,
    pub type2: u8,
    pub type3: u16,
    pub type4: u8,
    pub _des: Box<str>,
}

impl DataCfgRow for ConfigDropTypeInfoRow {
    const TABLE_NAME: &'static str = "config_drop_type_info";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Istips" => self.istips = ctx.decode_field(val, Self::TABLE_NAME, "Istips")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Pic" => self.pic = ctx.decode_field(val, Self::TABLE_NAME, "Pic")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_drop_type_info.out`.
pub type ConfigDropTypeInfoTable = DataTable<u8, ConfigDropTypeInfoRow>;

/// Parses `config_drop_type_info.out` into a [`ConfigDropTypeInfoTable`].
pub fn parse_config_drop_type_info(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropTypeInfoTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_dropfilter_sys_condition.Placeholder`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DropfilterSysConditionPlaceholderEntry {
    pub index: u8,
    pub select_data_type: u8,
}

impl FromEvalVal for DropfilterSysConditionPlaceholderEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (index, select_data_type) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            index,
            select_data_type,
        })
    }
}

/// Entry for `config_dropfilter_sys_condition.SelectionValue`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DropfilterSysConditionSelectionValueEntry {
    pub index: u8,
    pub values: Box<[i8]>,
}

impl FromEvalVal for DropfilterSysConditionSelectionValueEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (index, values) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { index, values })
    }
}

/// Row record for `config_dropfilter_sys_condition.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterSysConditionRow {
    pub condition_type: u8,
    pub deprecated: bool,
    pub description: Box<str>,
    pub id: i16,
    pub name: Box<str>,
    pub placeholder: Box<[DropfilterSysConditionPlaceholderEntry]>,
    pub rule_desc: Box<str>,
    pub rule_desc2: Box<str>,
    pub selection_content: Box<[u8]>,
    pub selection_des: Box<str>,
    pub selection_value: Box<[DropfilterSysConditionSelectionValueEntry]>,
    pub show_in_cg: i8,
    pub _description: Box<str>,
    pub _name: Box<str>,
    pub _rule_desc: Box<str>,
    pub _rule_desc2: Box<str>,
    pub _selection_des: Box<str>,
}

impl ConfigDropfilterSysConditionRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"ConditionType" => {
                self.condition_type = ctx.decode_field(val, Self::TABLE_NAME, "ConditionType")?
            }
            b"Deprecated" => {
                self.deprecated = ctx.decode_field(val, Self::TABLE_NAME, "Deprecated")?
            }
            b"Description" => {
                self.description = ctx.decode_field(val, Self::TABLE_NAME, "Description")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Placeholder" => {
                self.placeholder = ctx.decode_field(val, Self::TABLE_NAME, "Placeholder")?
            }
            b"RuleDesc" => self.rule_desc = ctx.decode_field(val, Self::TABLE_NAME, "RuleDesc")?,
            b"RuleDesc2" => {
                self.rule_desc2 = ctx.decode_field(val, Self::TABLE_NAME, "RuleDesc2")?
            }
            b"SelectionContent" => {
                self.selection_content =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionContent")?
            }
            b"SelectionDes" => {
                self.selection_des = ctx.decode_field(val, Self::TABLE_NAME, "SelectionDes")?
            }
            b"SelectionValue" => {
                self.selection_value = ctx.decode_field(val, Self::TABLE_NAME, "SelectionValue")?
            }
            b"ShowInCG" => self.show_in_cg = ctx.decode_field(val, Self::TABLE_NAME, "ShowInCG")?,
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
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_RuleDesc" => {
                self._rule_desc = ctx.decode_field(val, Self::TABLE_NAME, "_RuleDesc")?
            }
            b"_RuleDesc2" => {
                self._rule_desc2 = ctx.decode_field(val, Self::TABLE_NAME, "_RuleDesc2")?
            }
            b"_SelectionDes" => {
                self._selection_des = ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDes")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigDropfilterSysConditionRow {
    const TABLE_NAME: &'static str = "config_dropfilter_sys_condition";

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

/// Typed `DataTable` for `config_dropfilter_sys_condition.out`.
pub type ConfigDropfilterSysConditionTable = DataTable<u8, ConfigDropfilterSysConditionRow>;

/// Parses `config_dropfilter_sys_condition.out` into a [`ConfigDropfilterSysConditionTable`].
pub fn parse_config_dropfilter_sys_condition(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterSysConditionTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_sys_entrance.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterSysEntranceRow {
    pub condition_id: Box<[u8]>,
    pub default_condition_id: Box<[u8]>,
    pub id: i8,
    pub is_legend: u8,
    pub max_rule_num: u8,
    pub name: Box<str>,
    pub selector_item: u16,
    pub selector_merc_skill: u16,
    pub selector_mod: u16,
    pub selector_talent: u16,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigDropfilterSysEntranceRow {
    const TABLE_NAME: &'static str = "config_dropfilter_sys_entrance";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ConditionId" => {
                self.condition_id = ctx.decode_field(val, Self::TABLE_NAME, "ConditionId")?
            }
            b"DefaultConditionId" => {
                self.default_condition_id =
                    ctx.decode_field(val, Self::TABLE_NAME, "DefaultConditionId")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsLegend" => self.is_legend = ctx.decode_field(val, Self::TABLE_NAME, "IsLegend")?,
            b"MaxRuleNum" => {
                self.max_rule_num = ctx.decode_field(val, Self::TABLE_NAME, "MaxRuleNum")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"SelectorItem" => {
                self.selector_item = ctx.decode_field(val, Self::TABLE_NAME, "SelectorItem")?
            }
            b"SelectorMercSkill" => {
                self.selector_merc_skill =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectorMercSkill")?
            }
            b"SelectorMod" => {
                self.selector_mod = ctx.decode_field(val, Self::TABLE_NAME, "SelectorMod")?
            }
            b"SelectorTalent" => {
                self.selector_talent = ctx.decode_field(val, Self::TABLE_NAME, "SelectorTalent")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_dropfilter_sys_entrance.out`.
pub type ConfigDropfilterSysEntranceTable = DataTable<u8, ConfigDropfilterSysEntranceRow>;

/// Parses `config_dropfilter_sys_entrance.out` into a [`ConfigDropfilterSysEntranceTable`].
pub fn parse_config_dropfilter_sys_entrance(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterSysEntranceTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_sys_phase.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterSysPhaseRow {
    pub des: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub _des: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigDropfilterSysPhaseRow {
    const TABLE_NAME: &'static str = "config_dropfilter_sys_phase";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_dropfilter_sys_phase.out`.
pub type ConfigDropfilterSysPhaseTable = DataTable<u8, ConfigDropfilterSysPhaseRow>;

/// Parses `config_dropfilter_sys_phase.out` into a [`ConfigDropfilterSysPhaseTable`].
pub fn parse_config_dropfilter_sys_phase(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterSysPhaseTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_sys_selector_item.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterSysSelectorItemRow {
    pub id: i16,
    pub is_legend: u8,
    pub name: Box<str>,
    pub sub_ids: Box<[u16]>,
    pub type1: u8,
    pub type2: u8,
    pub type3: u8,
    pub type4: u16,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigDropfilterSysSelectorItemRow {
    const TABLE_NAME: &'static str = "config_dropfilter_sys_selector_item";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsLegend" => self.is_legend = ctx.decode_field(val, Self::TABLE_NAME, "IsLegend")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"SubIds" => self.sub_ids = ctx.decode_field(val, Self::TABLE_NAME, "SubIds")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_dropfilter_sys_selector_item.out`.
pub type ConfigDropfilterSysSelectorItemTable = DataTable<u16, ConfigDropfilterSysSelectorItemRow>;

/// Parses `config_dropfilter_sys_selector_item.out` into a [`ConfigDropfilterSysSelectorItemTable`].
pub fn parse_config_dropfilter_sys_selector_item(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterSysSelectorItemTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_sys_selector_mod.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterSysSelectorModRow {
    pub affix_key: Box<[Box<str>]>,
    pub id: i16,
    pub modifier_type: Box<[u8]>,
    pub name: Box<str>,
    pub parent_id: u16,
    pub sub_ids: Box<[u16]>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigDropfilterSysSelectorModRow {
    const TABLE_NAME: &'static str = "config_dropfilter_sys_selector_mod";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AffixKey" => self.affix_key = ctx.decode_field(val, Self::TABLE_NAME, "AffixKey")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ModifierType" => {
                self.modifier_type = ctx.decode_field(val, Self::TABLE_NAME, "ModifierType")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"ParentId" => self.parent_id = ctx.decode_field(val, Self::TABLE_NAME, "ParentId")?,
            b"SubIds" => self.sub_ids = ctx.decode_field(val, Self::TABLE_NAME, "SubIds")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_dropfilter_sys_selector_mod.out`.
pub type ConfigDropfilterSysSelectorModTable = DataTable<u16, ConfigDropfilterSysSelectorModRow>;

/// Parses `config_dropfilter_sys_selector_mod.out` into a [`ConfigDropfilterSysSelectorModTable`].
pub fn parse_config_dropfilter_sys_selector_mod(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterSysSelectorModTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_sys_selector_talent.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterSysSelectorTalentRow {
    pub id: i16,
    pub name: Box<str>,
    pub parent_id: u16,
    pub sub_ids: Box<[u16]>,
    pub talent_id: Box<[u32]>,
    pub r#type: u8,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigDropfilterSysSelectorTalentRow {
    const TABLE_NAME: &'static str = "config_dropfilter_sys_selector_talent";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"ParentId" => self.parent_id = ctx.decode_field(val, Self::TABLE_NAME, "ParentId")?,
            b"SubIds" => self.sub_ids = ctx.decode_field(val, Self::TABLE_NAME, "SubIds")?,
            b"TalentId" => self.talent_id = ctx.decode_field(val, Self::TABLE_NAME, "TalentId")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_dropfilter_sys_selector_talent.out`.
pub type ConfigDropfilterSysSelectorTalentTable =
    DataTable<u16, ConfigDropfilterSysSelectorTalentRow>;

/// Parses `config_dropfilter_sys_selector_talent.out` into a [`ConfigDropfilterSysSelectorTalentTable`].
pub fn parse_config_dropfilter_sys_selector_talent(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterSysSelectorTalentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_sys_sift.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterSysSiftRow {
    pub condition_id: Box<[u8]>,
    pub default_condition_id: Box<[u8]>,
    pub description: Box<str>,
    pub entrance_id: u8,
    pub icon: Box<str>,
    pub id: i16,
    pub islegend: u8,
    pub item_type: Box<[u8]>,
    pub name: Box<str>,
    pub node_id: u16,
    pub _description: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigDropfilterSysSiftRow {
    const TABLE_NAME: &'static str = "config_dropfilter_sys_sift";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ConditionId" => {
                self.condition_id = ctx.decode_field(val, Self::TABLE_NAME, "ConditionId")?
            }
            b"DefaultConditionId" => {
                self.default_condition_id =
                    ctx.decode_field(val, Self::TABLE_NAME, "DefaultConditionId")?
            }
            b"Description" => {
                self.description = ctx.decode_field(val, Self::TABLE_NAME, "Description")?
            }
            b"EntranceId" => {
                self.entrance_id = ctx.decode_field(val, Self::TABLE_NAME, "EntranceId")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Islegend" => self.islegend = ctx.decode_field(val, Self::TABLE_NAME, "Islegend")?,
            b"ItemType" => self.item_type = ctx.decode_field(val, Self::TABLE_NAME, "ItemType")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"NodeId" => self.node_id = ctx.decode_field(val, Self::TABLE_NAME, "NodeId")?,
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_dropfilter_sys_sift.out`.
pub type ConfigDropfilterSysSiftTable = DataTable<u16, ConfigDropfilterSysSiftRow>;

/// Parses `config_dropfilter_sys_sift.out` into a [`ConfigDropfilterSysSiftTable`].
pub fn parse_config_dropfilter_sys_sift(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterSysSiftTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_var_color.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterVarColorRow {
    pub color_id: u16,
    pub id: i16,
    pub name: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigDropfilterVarColorRow {
    const TABLE_NAME: &'static str = "config_dropfilter_var_color";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ColorId" => self.color_id = ctx.decode_field(val, Self::TABLE_NAME, "ColorId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_dropfilter_var_color.out`.
pub type ConfigDropfilterVarColorTable = DataTable<u16, ConfigDropfilterVarColorRow>;

/// Parses `config_dropfilter_var_color.out` into a [`ConfigDropfilterVarColorTable`].
pub fn parse_config_dropfilter_var_color(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterVarColorTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_dropfilter_var_condition.BaseModifierListArgs`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DropfilterVarConditionBaseModifierListArgs {
    pub mode: u8,
    pub args: Box<[u8]>,
}

impl FromEvalVal for DropfilterVarConditionBaseModifierListArgs {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (mode, args) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { mode, args })
    }
}

/// Entry for `config_dropfilter_var_condition.ItemTypeList`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DropfilterVarConditionItemTypeListEntry {
    pub mode: u8,
    pub type1: u8,
    pub type2: u8,
    pub type3: u8,
    pub type4: u16,
}

impl FromEvalVal for DropfilterVarConditionItemTypeListEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (mode, (type1, type2, type3, type4)) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            mode,
            type1,
            type2,
            type3,
            type4,
        })
    }
}

/// Entry for `config_dropfilter_var_condition.ModListArgs`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DropfilterVarConditionModListArgs {
    pub mode: u8,
    pub args: Box<[u8]>,
}

impl FromEvalVal for DropfilterVarConditionModListArgs {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (mode, args) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { mode, args })
    }
}

/// Entry for `config_dropfilter_var_condition.TalentType`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DropfilterVarConditionTalentTypeEntry {
    pub talent_type: u8,
    pub sub_types: Box<[u8]>,
}

impl FromEvalVal for DropfilterVarConditionTalentTypeEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (talent_type, sub_types) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            talent_type,
            sub_types,
        })
    }
}

/// Row record for `config_dropfilter_var_condition.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterVarConditionRow {
    pub base_modifier_list: [u8; 0],
    pub base_modifier_list_args: DropfilterVarConditionBaseModifierListArgs,
    pub base_modifier_num: Box<[u8]>,
    pub brand: [u8; 0],
    pub cost: [u8; 0],
    pub drop_audio: u8,
    pub drop_lv: [u8; 0],
    pub engram: [u8; 0],
    pub entrance_id: u8,
    pub fashion_audio_cue: u8,
    pub hide_tips: u8,
    pub id: i16,
    pub is_dreamtalking: i8,
    pub is_eroded: i8,
    pub is_identified: i8,
    pub is_s10_enchant: i8,
    pub is_s11_chipmatching: i8,
    pub is_wishpool: i8,
    pub item_base_id: Box<[u32]>,
    pub item_gold_id: Box<[u32]>,
    pub item_lv: Box<[u8]>,
    pub item_type_list: Box<[DropfilterVarConditionItemTypeListEntry]>,
    pub itemgold_slot: [u8; 0],
    pub mod_list: [u8; 0],
    pub mod_list_args: DropfilterVarConditionModListArgs,
    pub modifier_num: Box<[u8]>,
    pub name: Box<str>,
    pub plasticity: [u8; 0],
    pub priority: u16,
    pub prism_socket_num: Box<[u8]>,
    pub rarity: Box<[u8]>,
    pub readability: [u8; 0],
    pub sift: u8,
    pub stack_size: Box<[u8]>,
    pub style_id: u16,
    pub talent_list: [u8; 0],
    pub talent_list_args: [u8; 0],
    pub talent_num: Box<[u8]>,
    pub talent_type: Box<[DropfilterVarConditionTalentTypeEntry]>,
    pub talentblock_slot: [u8; 0],
    pub talentblock_type1_slot: [u8; 0],
    pub talentblock_type2_slot: [u8; 0],
    pub talentblock_type3_slot: Box<[u8]>,
    pub talentblock_type4_slot: Box<[u8]>,
    pub _name: Box<str>,
}

impl ConfigDropfilterVarConditionRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"BaseModifierList" => {
                self.base_modifier_list =
                    ctx.decode_field(val, Self::TABLE_NAME, "BaseModifierList")?
            }
            b"BaseModifierListArgs" => {
                self.base_modifier_list_args =
                    ctx.decode_field(val, Self::TABLE_NAME, "BaseModifierListArgs")?
            }
            b"BaseModifierNum" => {
                self.base_modifier_num =
                    ctx.decode_field(val, Self::TABLE_NAME, "BaseModifierNum")?
            }
            b"Brand" => self.brand = ctx.decode_field(val, Self::TABLE_NAME, "Brand")?,
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"DropAudio" => {
                self.drop_audio = ctx.decode_field(val, Self::TABLE_NAME, "DropAudio")?
            }
            b"DropLv" => self.drop_lv = ctx.decode_field(val, Self::TABLE_NAME, "DropLv")?,
            b"Engram" => self.engram = ctx.decode_field(val, Self::TABLE_NAME, "Engram")?,
            b"EntranceId" => {
                self.entrance_id = ctx.decode_field(val, Self::TABLE_NAME, "EntranceId")?
            }
            b"FashionAudioCue" => {
                self.fashion_audio_cue =
                    ctx.decode_field(val, Self::TABLE_NAME, "FashionAudioCue")?
            }
            b"HideTips" => self.hide_tips = ctx.decode_field(val, Self::TABLE_NAME, "HideTips")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
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
            b"IsDreamtalking" => {
                self.is_dreamtalking = ctx.decode_field(val, Self::TABLE_NAME, "IsDreamtalking")?
            }
            b"IsEroded" => self.is_eroded = ctx.decode_field(val, Self::TABLE_NAME, "IsEroded")?,
            b"IsIdentified" => {
                self.is_identified = ctx.decode_field(val, Self::TABLE_NAME, "IsIdentified")?
            }
            b"IsS10Enchant" => {
                self.is_s10_enchant = ctx.decode_field(val, Self::TABLE_NAME, "IsS10Enchant")?
            }
            b"IsS11Chipmatching" => {
                self.is_s11_chipmatching =
                    ctx.decode_field(val, Self::TABLE_NAME, "IsS11Chipmatching")?
            }
            b"IsWishpool" => {
                self.is_wishpool = ctx.decode_field(val, Self::TABLE_NAME, "IsWishpool")?
            }
            b"ItemBaseId" => {
                self.item_base_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemBaseId")?
            }
            b"ItemGoldId" => {
                self.item_gold_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemGoldId")?
            }
            b"ItemLv" => self.item_lv = ctx.decode_field(val, Self::TABLE_NAME, "ItemLv")?,
            b"ItemTypeList" => {
                self.item_type_list = ctx.decode_field(val, Self::TABLE_NAME, "ItemTypeList")?
            }
            b"ItemgoldSlot" => {
                self.itemgold_slot = ctx.decode_field(val, Self::TABLE_NAME, "ItemgoldSlot")?
            }
            b"ModList" => self.mod_list = ctx.decode_field(val, Self::TABLE_NAME, "ModList")?,
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
            b"ModListArgs" => {
                self.mod_list_args = ctx.decode_field(val, Self::TABLE_NAME, "ModListArgs")?
            }
            b"ModifierNum" => {
                self.modifier_num = ctx.decode_field(val, Self::TABLE_NAME, "ModifierNum")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Plasticity" => {
                self.plasticity = ctx.decode_field(val, Self::TABLE_NAME, "Plasticity")?
            }
            b"Priority" => self.priority = ctx.decode_field(val, Self::TABLE_NAME, "Priority")?,
            b"PrismSocketNum" => {
                self.prism_socket_num = ctx.decode_field(val, Self::TABLE_NAME, "PrismSocketNum")?
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Readability" => {
                self.readability = ctx.decode_field(val, Self::TABLE_NAME, "Readability")?
            }
            b"Sift" => self.sift = ctx.decode_field(val, Self::TABLE_NAME, "Sift")?,
            b"StackSize" => {
                self.stack_size = ctx.decode_field(val, Self::TABLE_NAME, "StackSize")?
            }
            b"StyleId" => self.style_id = ctx.decode_field(val, Self::TABLE_NAME, "StyleId")?,
            b"TalentList" => {
                self.talent_list = ctx.decode_field(val, Self::TABLE_NAME, "TalentList")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_field_3(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"TalentListArgs" => {
                self.talent_list_args = ctx.decode_field(val, Self::TABLE_NAME, "TalentListArgs")?
            }
            b"TalentNum" => {
                self.talent_num = ctx.decode_field(val, Self::TABLE_NAME, "TalentNum")?
            }
            b"TalentType" => {
                self.talent_type = ctx.decode_field(val, Self::TABLE_NAME, "TalentType")?
            }
            b"TalentblockSlot" => {
                self.talentblock_slot =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentblockSlot")?
            }
            b"TalentblockType1Slot" => {
                self.talentblock_type1_slot =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentblockType1Slot")?
            }
            b"TalentblockType2Slot" => {
                self.talentblock_type2_slot =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentblockType2Slot")?
            }
            b"TalentblockType3Slot" => {
                self.talentblock_type3_slot =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentblockType3Slot")?
            }
            b"TalentblockType4Slot" => {
                self.talentblock_type4_slot =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentblockType4Slot")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigDropfilterVarConditionRow {
    const TABLE_NAME: &'static str = "config_dropfilter_var_condition";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        if self.set_field_0(ctx, field, val)?
            || self.set_field_1(ctx, field, val)?
            || self.set_field_2(ctx, field, val)?
            || self.set_field_3(ctx, field, val)?
        {
            return Ok(());
        }
        Err(ctx.unknown_field(Self::TABLE_NAME, field))
    }
}

/// Typed `DataTable` for `config_dropfilter_var_condition.out`.
pub type ConfigDropfilterVarConditionTable = DataTable<u16, ConfigDropfilterVarConditionRow>;

/// Parses `config_dropfilter_var_condition.out` into a [`ConfigDropfilterVarConditionTable`].
pub fn parse_config_dropfilter_var_condition(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterVarConditionTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_var_decid.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterVarDecidRow {
    pub dec: Box<str>,
    pub des_suf: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigDropfilterVarDecidRow {
    const TABLE_NAME: &'static str = "config_dropfilter_var_decid";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Dec" => self.dec = ctx.decode_field(val, Self::TABLE_NAME, "Dec")?,
            b"DesSuf" => self.des_suf = ctx.decode_field(val, Self::TABLE_NAME, "DesSuf")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_dropfilter_var_decid.out`.
pub type ConfigDropfilterVarDecidTable = DataTable<u8, ConfigDropfilterVarDecidRow>;

/// Parses `config_dropfilter_var_decid.out` into a [`ConfigDropfilterVarDecidTable`].
pub fn parse_config_dropfilter_var_decid(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterVarDecidTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_var_head.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterVarHeadRow {
    pub condition: Box<[u16]>,
    pub condition_ui_parse: Box<str>,
    pub des: Box<str>,
    pub hero_purchase_id: u8,
    pub hide: u8,
    pub id: i8,
    pub name: Box<str>,
    pub sys_phase_id: u8,
    pub _des: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigDropfilterVarHeadRow {
    const TABLE_NAME: &'static str = "config_dropfilter_var_head";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Condition" => {
                self.condition = ctx.decode_field(val, Self::TABLE_NAME, "Condition")?
            }
            b"ConditionUiParse" => {
                self.condition_ui_parse =
                    ctx.decode_field(val, Self::TABLE_NAME, "ConditionUiParse")?
            }
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"HeroPurchaseId" => {
                self.hero_purchase_id = ctx.decode_field(val, Self::TABLE_NAME, "HeroPurchaseId")?
            }
            b"Hide" => self.hide = ctx.decode_field(val, Self::TABLE_NAME, "Hide")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"SysPhaseId" => {
                self.sys_phase_id = ctx.decode_field(val, Self::TABLE_NAME, "SysPhaseId")?
            }
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_dropfilter_var_head.out`.
pub type ConfigDropfilterVarHeadTable = DataTable<u8, ConfigDropfilterVarHeadRow>;

/// Parses `config_dropfilter_var_head.out` into a [`ConfigDropfilterVarHeadTable`].
pub fn parse_config_dropfilter_var_head(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterVarHeadTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_var_icon.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDropfilterVarIconRow {
    pub icon: Box<str>,
    pub id: i8,
    pub rarity: u8,
    pub type1: u8,
    pub type2: u8,
    pub type3: u8,
    pub type4: u8,
}

impl DataCfgRow for ConfigDropfilterVarIconRow {
    const TABLE_NAME: &'static str = "config_dropfilter_var_icon";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_dropfilter_var_icon.out`.
pub type ConfigDropfilterVarIconTable = DataTable<u8, ConfigDropfilterVarIconRow>;

/// Parses `config_dropfilter_var_icon.out` into a [`ConfigDropfilterVarIconTable`].
pub fn parse_config_dropfilter_var_icon(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterVarIconTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_dropfilter_var_style.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigDropfilterVarStyleRow {
    pub audio_level: u8,
    pub beam_color: u8,
    pub frame_size: f64,
    pub hide: bool,
    pub icon_bd_color: u16,
    pub id: i16,
    pub mapicon_id: u8,
    pub mapicon_size: f64,
    pub pick_color: u8,
    pub pick_priority: i16,
    pub pick_value: f64,
    pub show_icon: bool,
    pub text_bd_color: u16,
    pub text_bg_color: u16,
    pub text_color: u16,
    pub text_decid: u8,
    pub visible: bool,
}

impl ConfigDropfilterVarStyleRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AudioLevel" => {
                self.audio_level = ctx.decode_field(val, Self::TABLE_NAME, "AudioLevel")?
            }
            b"BeamColor" => {
                self.beam_color = ctx.decode_field(val, Self::TABLE_NAME, "BeamColor")?
            }
            b"FrameSize" => {
                self.frame_size = ctx.decode_field(val, Self::TABLE_NAME, "FrameSize")?
            }
            b"Hide" => self.hide = ctx.decode_field(val, Self::TABLE_NAME, "Hide")?,
            b"IconBdColor" => {
                self.icon_bd_color = ctx.decode_field(val, Self::TABLE_NAME, "IconBdColor")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MapiconId" => {
                self.mapicon_id = ctx.decode_field(val, Self::TABLE_NAME, "MapiconId")?
            }
            b"MapiconSize" => {
                self.mapicon_size = ctx.decode_field(val, Self::TABLE_NAME, "MapiconSize")?
            }
            b"PickColor" => {
                self.pick_color = ctx.decode_field(val, Self::TABLE_NAME, "PickColor")?
            }
            b"PickPriority" => {
                self.pick_priority = ctx.decode_field(val, Self::TABLE_NAME, "PickPriority")?
            }
            b"PickValue" => {
                self.pick_value = ctx.decode_field(val, Self::TABLE_NAME, "PickValue")?
            }
            b"ShowIcon" => self.show_icon = ctx.decode_field(val, Self::TABLE_NAME, "ShowIcon")?,
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
            b"TextBdColor" => {
                self.text_bd_color = ctx.decode_field(val, Self::TABLE_NAME, "TextBdColor")?
            }
            b"TextBgColor" => {
                self.text_bg_color = ctx.decode_field(val, Self::TABLE_NAME, "TextBgColor")?
            }
            b"TextColor" => {
                self.text_color = ctx.decode_field(val, Self::TABLE_NAME, "TextColor")?
            }
            b"TextDecid" => {
                self.text_decid = ctx.decode_field(val, Self::TABLE_NAME, "TextDecid")?
            }
            b"Visible" => self.visible = ctx.decode_field(val, Self::TABLE_NAME, "Visible")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigDropfilterVarStyleRow {
    const TABLE_NAME: &'static str = "config_dropfilter_var_style";

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

/// Typed `DataTable` for `config_dropfilter_var_style.out`.
pub type ConfigDropfilterVarStyleTable = DataTable<u16, ConfigDropfilterVarStyleRow>;

/// Parses `config_dropfilter_var_style.out` into a [`ConfigDropfilterVarStyleTable`].
pub fn parse_config_dropfilter_var_style(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDropfilterVarStyleTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

fn verify_drop_chunk_0(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_drop_base" => Some(parse_config_drop_base(buf, scratch).map(|t| t.len())),
        "config_drop_group" => Some(parse_config_drop_group(buf, scratch).map(|t| t.len())),
        "config_drop_type_info" => Some(parse_config_drop_type_info(buf, scratch).map(|t| t.len())),
        "config_dropfilter_sys_condition" => {
            Some(parse_config_dropfilter_sys_condition(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_sys_entrance" => {
            Some(parse_config_dropfilter_sys_entrance(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_sys_phase" => {
            Some(parse_config_dropfilter_sys_phase(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_sys_selector_item" => {
            Some(parse_config_dropfilter_sys_selector_item(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_sys_selector_mod" => {
            Some(parse_config_dropfilter_sys_selector_mod(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_sys_selector_talent" => {
            Some(parse_config_dropfilter_sys_selector_talent(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_sys_sift" => {
            Some(parse_config_dropfilter_sys_sift(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_var_color" => {
            Some(parse_config_dropfilter_var_color(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_var_condition" => {
            Some(parse_config_dropfilter_var_condition(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_var_decid" => {
            Some(parse_config_dropfilter_var_decid(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_var_head" => {
            Some(parse_config_dropfilter_var_head(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

fn verify_drop_chunk_1(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_dropfilter_var_icon" => {
            Some(parse_config_dropfilter_var_icon(buf, scratch).map(|t| t.len()))
        }
        "config_dropfilter_var_style" => {
            Some(parse_config_dropfilter_var_style(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

pub(crate) fn verify_drop_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    if let Some(res) = verify_drop_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_drop_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    None
}

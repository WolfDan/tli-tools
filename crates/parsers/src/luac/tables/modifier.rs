//! Auto-generated `LuaTable/DataCfg` types and parsers for the `modifier` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, parse_luatable_datacfg,
};
use crate::luac::error::LuacError;

/// Row record for `config_attribute_category.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAttributeCategoryRow {
    pub category: Box<[u8]>,
    pub id: i32,
}

impl DataCfgRow for ConfigAttributeCategoryRow {
    const TABLE_NAME: &'static str = "config_attribute_category";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Category" => self.category = ctx.decode_field(val, Self::TABLE_NAME, "Category")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_attribute_category.out`.
pub type ConfigAttributeCategoryTable = DataTable<u32, ConfigAttributeCategoryRow>;

/// Parses `config_attribute_category.out` into a [`ConfigAttributeCategoryTable`].
pub fn parse_config_attribute_category(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAttributeCategoryTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_attributes_show_cal.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAttributesShowCalRow {
    pub attribute: Box<[u32]>,
    pub id: i8,
    pub skill_tag: Box<[u32]>,
}

impl DataCfgRow for ConfigAttributesShowCalRow {
    const TABLE_NAME: &'static str = "config_attributes_show_cal";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Attribute" => {
                self.attribute = ctx.decode_field(val, Self::TABLE_NAME, "Attribute")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"SkillTag" => self.skill_tag = ctx.decode_field(val, Self::TABLE_NAME, "SkillTag")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_attributes_show_cal.out`.
pub type ConfigAttributesShowCalTable = DataTable<u8, ConfigAttributesShowCalRow>;

/// Parses `config_attributes_show_cal.out` into a [`ConfigAttributesShowCalTable`].
pub fn parse_config_attributes_show_cal(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAttributesShowCalTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_function_des.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFunctionDesRow {
    pub des: Box<str>,
    pub detail: Box<str>,
    pub icon: Box<str>,
    pub id: i32,
    pub name: Box<str>,
    pub rare_value: u8,
    pub r#type: u8,
    pub _des: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigFunctionDesRow {
    const TABLE_NAME: &'static str = "config_function_des";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Detail" => self.detail = ctx.decode_field(val, Self::TABLE_NAME, "Detail")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"RareValue" => {
                self.rare_value = ctx.decode_field(val, Self::TABLE_NAME, "RareValue")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_function_des.out`.
pub type ConfigFunctionDesTable = DataTable<u32, ConfigFunctionDesRow>;

/// Parses `config_function_des.out` into a [`ConfigFunctionDesTable`].
pub fn parse_config_function_des(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigFunctionDesTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_modifier_tier_icon.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigModifierTierIconRow {
    pub quality: u8,
    pub tier: i8,
}

impl DataCfgRow for ConfigModifierTierIconRow {
    const TABLE_NAME: &'static str = "config_modifier_tier_icon";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Quality" => self.quality = ctx.decode_field(val, Self::TABLE_NAME, "Quality")?,
            b"Tier" => self.tier = ctx.decode_field(val, Self::TABLE_NAME, "Tier")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_modifier_tier_icon.out`.
pub type ConfigModifierTierIconTable = DataTable<u8, ConfigModifierTierIconRow>;

/// Parses `config_modifier_tier_icon.out` into a [`ConfigModifierTierIconTable`].
pub fn parse_config_modifier_tier_icon(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigModifierTierIconTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_modifier_type_desc.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigModifierTypeDescRow {
    pub desc: Box<str>,
    pub id: i8,
    pub _desc: Box<str>,
}

impl DataCfgRow for ConfigModifierTypeDescRow {
    const TABLE_NAME: &'static str = "config_modifier_type_desc";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_modifier_type_desc.out`.
pub type ConfigModifierTypeDescTable = DataTable<u8, ConfigModifierTypeDescRow>;

/// Parses `config_modifier_type_desc.out` into a [`ConfigModifierTypeDescTable`].
pub fn parse_config_modifier_type_desc(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigModifierTypeDescTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

pub(crate) fn verify_modifier_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_attribute_category" => {
            Some(parse_config_attribute_category(buf, scratch).map(|t| t.len()))
        }
        "config_attributes_show_cal" => {
            Some(parse_config_attributes_show_cal(buf, scratch).map(|t| t.len()))
        }
        "config_function_des" => Some(parse_config_function_des(buf, scratch).map(|t| t.len())),
        "config_modifier_tier_icon" => {
            Some(parse_config_modifier_tier_icon(buf, scratch).map(|t| t.len()))
        }
        "config_modifier_type_desc" => {
            Some(parse_config_modifier_type_desc(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

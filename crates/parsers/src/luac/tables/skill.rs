//! Auto-generated `LuaTable/DataCfg` types and parsers for the `skill` domain.
use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;
/// Row record for `config_skill_des.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillDesRow {
    pub des_config: Box<str>,
    pub des_config_param: Box<[Box<str>]>,
    pub _des_config: Box<str>,
    pub id: i32,
}
impl DataCfgRow for ConfigSkillDesRow {
    const TABLE_NAME: &'static str = "config_skill_des";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DesConfig" => {
                self.des_config = ctx.decode_field(val, Self::TABLE_NAME, "DesConfig")?;
            }
            b"DesConfigParam" => {
                self.des_config_param = ctx
                    .decode_field(val, Self::TABLE_NAME, "DesConfigParam")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"_DesConfig" => {
                self._des_config = ctx.decode_field(val, Self::TABLE_NAME, "_DesConfig")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_skill_des.out`.
pub type ConfigSkillDesTable = DataTable<u32, ConfigSkillDesRow>;
/// Parses `config_skill_des.out` into a [`ConfigSkillDesTable`].
pub fn parse_config_skill_des(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSkillDesTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_skill_special_level_up.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillSpecialLevelUpRow {
    pub converted_tlevel_exp: FxHashMap<u8, u8>,
    pub cost_item_base: u32,
    pub converted_exp: u8,
    pub id: i8,
    pub level: u8,
    pub level_up_exp_cost: u8,
    pub level_up_item_num_cost: u8,
    pub r#type: u8,
}
impl DataCfgRow for ConfigSkillSpecialLevelUpRow {
    const TABLE_NAME: &'static str = "config_skill_special_level_up";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ConvertedExp" => {
                self.converted_exp = ctx
                    .decode_field(val, Self::TABLE_NAME, "ConvertedExp")?;
            }
            b"ConvertedTlevelExp" => {
                self.converted_tlevel_exp = ctx
                    .decode_field(val, Self::TABLE_NAME, "ConvertedTlevelExp")?;
            }
            b"CostItemBase" => {
                self.cost_item_base = ctx
                    .decode_field(val, Self::TABLE_NAME, "CostItemBase")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"LevelUpExpCost" => {
                self.level_up_exp_cost = ctx
                    .decode_field(val, Self::TABLE_NAME, "LevelUpExpCost")?;
            }
            b"LevelUpItemNumCost" => {
                self.level_up_item_num_cost = ctx
                    .decode_field(val, Self::TABLE_NAME, "LevelUpItemNumCost")?;
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_skill_special_level_up.out`.
pub type ConfigSkillSpecialLevelUpTable = DataTable<u8, ConfigSkillSpecialLevelUpRow>;
/// Parses `config_skill_special_level_up.out` into a [`ConfigSkillSpecialLevelUpTable`].
pub fn parse_config_skill_special_level_up(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSkillSpecialLevelUpTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_skill_stone_modifier.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSkillStoneModifierRow {
    pub modifier1: FxHashMap<u32, Box<[i8]>>,
    pub modifier1_des: Box<str>,
    pub modifier2: FxHashMap<u32, Box<[i8]>>,
    pub modifier2_des: Box<str>,
    pub modifier3: FxHashMap<u32, Box<[i8]>>,
    pub modifier3_des: Box<str>,
    pub modifier4_des: Box<str>,
    pub modifier5_des: Box<str>,
    pub modifier6_des: Box<str>,
    pub _modifier1_des: Box<str>,
    pub _modifier2_des: Box<str>,
    pub _modifier3_des: Box<str>,
    pub _modifier4_des: Box<str>,
    pub _modifier5_des: Box<str>,
    pub _modifier6_des: Box<str>,
    pub id: i32,
    pub modifier4: [u8; 0],
    pub modifier5: [u8; 0],
    pub modifier6: [u8; 0],
}
impl ConfigSkillStoneModifierRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Modifier1" => {
                self.modifier1 = ctx.decode_field(val, Self::TABLE_NAME, "Modifier1")?;
            }
            b"Modifier1Des" => {
                self.modifier1_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "Modifier1Des")?;
            }
            b"Modifier2" => {
                self.modifier2 = ctx.decode_field(val, Self::TABLE_NAME, "Modifier2")?;
            }
            b"Modifier2Des" => {
                self.modifier2_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "Modifier2Des")?;
            }
            b"Modifier3" => {
                self.modifier3 = ctx.decode_field(val, Self::TABLE_NAME, "Modifier3")?;
            }
            b"Modifier3Des" => {
                self.modifier3_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "Modifier3Des")?;
            }
            b"Modifier4" => {
                self.modifier4 = ctx.decode_field(val, Self::TABLE_NAME, "Modifier4")?;
            }
            b"Modifier4Des" => {
                self.modifier4_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "Modifier4Des")?;
            }
            b"Modifier5" => {
                self.modifier5 = ctx.decode_field(val, Self::TABLE_NAME, "Modifier5")?;
            }
            b"Modifier5Des" => {
                self.modifier5_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "Modifier5Des")?;
            }
            b"Modifier6" => {
                self.modifier6 = ctx.decode_field(val, Self::TABLE_NAME, "Modifier6")?;
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
            b"Modifier6Des" => {
                self.modifier6_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "Modifier6Des")?;
            }
            b"_Modifier1Des" => {
                self._modifier1_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "_Modifier1Des")?;
            }
            b"_Modifier2Des" => {
                self._modifier2_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "_Modifier2Des")?;
            }
            b"_Modifier3Des" => {
                self._modifier3_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "_Modifier3Des")?;
            }
            b"_Modifier4Des" => {
                self._modifier4_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "_Modifier4Des")?;
            }
            b"_Modifier5Des" => {
                self._modifier5_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "_Modifier5Des")?;
            }
            b"_Modifier6Des" => {
                self._modifier6_des = ctx
                    .decode_field(val, Self::TABLE_NAME, "_Modifier6Des")?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigSkillStoneModifierRow {
    const TABLE_NAME: &'static str = "config_skill_stone_modifier";
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
/// Typed `DataTable` for `config_skill_stone_modifier.out`.
pub type ConfigSkillStoneModifierTable = DataTable<u32, ConfigSkillStoneModifierRow>;
/// Parses `config_skill_stone_modifier.out` into a [`ConfigSkillStoneModifierTable`].
pub fn parse_config_skill_stone_modifier(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSkillStoneModifierTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
pub(crate) fn verify_skill_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_skill_des" => Some(parse_config_skill_des(buf, scratch).map(|t| t.len())),
        "config_skill_special_level_up" => {
            Some(parse_config_skill_special_level_up(buf, scratch).map(|t| t.len()))
        }
        "config_skill_stone_modifier" => {
            Some(parse_config_skill_stone_modifier(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

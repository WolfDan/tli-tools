//! Auto-generated `LuaTable/DataCfg` types and parsers for the `system` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;

/// Row record for `config_account_bind.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAccountBindRow {
    pub function: Box<[u8]>,
    pub id: i8,
    pub level: u8,
    pub trigger: u8,
}

impl DataCfgRow for ConfigAccountBindRow {
    const TABLE_NAME: &'static str = "config_account_bind";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Trigger" => self.trigger = ctx.decode_field(val, Self::TABLE_NAME, "Trigger")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_account_bind.out`.
pub type ConfigAccountBindTable = DataTable<u8, ConfigAccountBindRow>;

/// Parses `config_account_bind.out` into a [`ConfigAccountBindTable`].
pub fn parse_config_account_bind(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAccountBindTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_common_tips.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCommonTipsRow {
    pub delay: u8,
    pub id: i8,
    pub tips_desc: Box<str>,
    pub _tips_desc: Box<str>,
}

impl DataCfgRow for ConfigCommonTipsRow {
    const TABLE_NAME: &'static str = "config_common_tips";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Delay" => self.delay = ctx.decode_field(val, Self::TABLE_NAME, "Delay")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"TipsDesc" => self.tips_desc = ctx.decode_field(val, Self::TABLE_NAME, "TipsDesc")?,
            b"_TipsDesc" => {
                self._tips_desc = ctx.decode_field(val, Self::TABLE_NAME, "_TipsDesc")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_common_tips.out`.
pub type ConfigCommonTipsTable = DataTable<u8, ConfigCommonTipsRow>;

/// Parses `config_common_tips.out` into a [`ConfigCommonTipsTable`].
pub fn parse_config_common_tips(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCommonTipsTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_cond_event.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCondEventRow {
    pub args: Box<str>,
    pub des: Box<str>,
    pub id: i32,
    pub use_des: bool,
    pub use_total: u16,
    pub _des: Box<str>,
}

impl DataCfgRow for ConfigCondEventRow {
    const TABLE_NAME: &'static str = "config_cond_event";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Args" => self.args = ctx.decode_field(val, Self::TABLE_NAME, "Args")?,
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"UseDes" => self.use_des = ctx.decode_field(val, Self::TABLE_NAME, "UseDes")?,
            b"UseTotal" => self.use_total = ctx.decode_field(val, Self::TABLE_NAME, "UseTotal")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_cond_event.out`.
pub type ConfigCondEventTable = DataTable<u32, ConfigCondEventRow>;

/// Parses `config_cond_event.out` into a [`ConfigCondEventTable`].
pub fn parse_config_cond_event(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCondEventTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_data_fix.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDataFixRow {
    pub id: i8,
    pub modifier_ids: Box<[u32]>,
}

impl DataCfgRow for ConfigDataFixRow {
    const TABLE_NAME: &'static str = "config_data_fix";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ModifierIds" => {
                self.modifier_ids = ctx.decode_field(val, Self::TABLE_NAME, "ModifierIds")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_data_fix.out`.
pub type ConfigDataFixTable = DataTable<u8, ConfigDataFixRow>;

/// Parses `config_data_fix.out` into a [`ConfigDataFixTable`].
pub fn parse_config_data_fix(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigDataFixTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_gamepad_tips.Content`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GamepadTipsContentEntry {
    pub button_id: u8,
    pub text_key: Box<str>,
}

impl FromEvalVal for GamepadTipsContentEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (button_id, text_key) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            button_id,
            text_key,
        })
    }
}

/// Row record for `config_gamepad_tips.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGamepadTipsRow {
    pub content: Box<[GamepadTipsContentEntry]>,
    pub hide: u8,
    pub id: i16,
}

impl DataCfgRow for ConfigGamepadTipsRow {
    const TABLE_NAME: &'static str = "config_gamepad_tips";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Content" => self.content = ctx.decode_field(val, Self::TABLE_NAME, "Content")?,
            b"Hide" => self.hide = ctx.decode_field(val, Self::TABLE_NAME, "Hide")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_gamepad_tips.out`.
pub type ConfigGamepadTipsTable = DataTable<u16, ConfigGamepadTipsRow>;

/// Parses `config_gamepad_tips.out` into a [`ConfigGamepadTipsTable`].
pub fn parse_config_gamepad_tips(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGamepadTipsTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_gameplay_duration.DifficultySpRankMap`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameplayDurationDifficultySpRankMapEntry {
    pub difficulty: u8,
    pub sp_rank_ids: Box<[u8]>,
}

impl FromEvalVal for GameplayDurationDifficultySpRankMapEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (difficulty, sp_rank_ids) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            difficulty,
            sp_rank_ids,
        })
    }
}

/// Entry for `config_gameplay_duration.PageTimeoutSecMap`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameplayDurationPageTimeoutSecMapEntry {
    pub page_name: Box<str>,
    pub timeout_sec: u16,
}

impl FromEvalVal for GameplayDurationPageTimeoutSecMapEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (page_name, timeout_sec) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            page_name,
            timeout_sec,
        })
    }
}

/// Row record for `config_gameplay_duration.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGameplayDurationRow {
    pub difficulty_map_type: u8,
    pub difficulty_sp_rank_map: Box<[GameplayDurationDifficultySpRankMapEntry]>,
    pub duration_type: u8,
    pub enable_report: bool,
    pub id: i8,
    pub page_timeout_sec_map: Box<[GameplayDurationPageTimeoutSecMapEntry]>,
}

impl DataCfgRow for ConfigGameplayDurationRow {
    const TABLE_NAME: &'static str = "config_gameplay_duration";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DifficultyMapType" => {
                self.difficulty_map_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "DifficultyMapType")?
            }
            b"DifficultySpRankMap" => {
                self.difficulty_sp_rank_map =
                    ctx.decode_field(val, Self::TABLE_NAME, "DifficultySpRankMap")?
            }
            b"DurationType" => {
                self.duration_type = ctx.decode_field(val, Self::TABLE_NAME, "DurationType")?
            }
            b"EnableReport" => {
                self.enable_report = ctx.decode_field(val, Self::TABLE_NAME, "EnableReport")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"PageTimeoutSecMap" => {
                self.page_timeout_sec_map =
                    ctx.decode_field(val, Self::TABLE_NAME, "PageTimeoutSecMap")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_gameplay_duration.out`.
pub type ConfigGameplayDurationTable = DataTable<u8, ConfigGameplayDurationRow>;

/// Parses `config_gameplay_duration.out` into a [`ConfigGameplayDurationTable`].
pub fn parse_config_gameplay_duration(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGameplayDurationTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_gameplay_entry_content.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGameplayEntryContentRow {
    pub icon_img: Box<str>,
    pub id: i16,
    pub order: u8,
    pub progress_title: Box<str>,
    pub title_name: Box<str>,
    pub type_title: Box<str>,
    pub _progress_title: Box<str>,
    pub _title_name: Box<str>,
    pub _type_title: Box<str>,
}

impl DataCfgRow for ConfigGameplayEntryContentRow {
    const TABLE_NAME: &'static str = "config_gameplay_entry_content";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"IconImg" => self.icon_img = ctx.decode_field(val, Self::TABLE_NAME, "IconImg")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"ProgressTitle" => {
                self.progress_title = ctx.decode_field(val, Self::TABLE_NAME, "ProgressTitle")?
            }
            b"TitleName" => {
                self.title_name = ctx.decode_field(val, Self::TABLE_NAME, "TitleName")?
            }
            b"TypeTitle" => {
                self.type_title = ctx.decode_field(val, Self::TABLE_NAME, "TypeTitle")?
            }
            b"_ProgressTitle" => {
                self._progress_title = ctx.decode_field(val, Self::TABLE_NAME, "_ProgressTitle")?
            }
            b"_TitleName" => {
                self._title_name = ctx.decode_field(val, Self::TABLE_NAME, "_TitleName")?
            }
            b"_TypeTitle" => {
                self._type_title = ctx.decode_field(val, Self::TABLE_NAME, "_TypeTitle")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_gameplay_entry_content.out`.
pub type ConfigGameplayEntryContentTable = DataTable<u16, ConfigGameplayEntryContentRow>;

/// Parses `config_gameplay_entry_content.out` into a [`ConfigGameplayEntryContentTable`].
pub fn parse_config_gameplay_entry_content(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGameplayEntryContentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_gameplay_show_resources.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGameplayShowResourcesRow {
    pub attach_tag: Box<str>,
    pub id: i16,
    pub idle1: Box<str>,
    pub idle2: Box<str>,
    pub interaction1: Box<str>,
    pub skeletal_res: Box<str>,
    pub trans: Box<str>,
}

impl DataCfgRow for ConfigGameplayShowResourcesRow {
    const TABLE_NAME: &'static str = "config_gameplay_show_resources";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AttachTag" => {
                self.attach_tag = ctx.decode_field(val, Self::TABLE_NAME, "AttachTag")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Idle1" => self.idle1 = ctx.decode_field(val, Self::TABLE_NAME, "Idle1")?,
            b"Idle2" => self.idle2 = ctx.decode_field(val, Self::TABLE_NAME, "Idle2")?,
            b"Interaction1" => {
                self.interaction1 = ctx.decode_field(val, Self::TABLE_NAME, "Interaction1")?
            }
            b"SkeletalRes" => {
                self.skeletal_res = ctx.decode_field(val, Self::TABLE_NAME, "SkeletalRes")?
            }
            b"Trans" => self.trans = ctx.decode_field(val, Self::TABLE_NAME, "Trans")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_gameplay_show_resources.out`.
pub type ConfigGameplayShowResourcesTable = DataTable<u16, ConfigGameplayShowResourcesRow>;

/// Parses `config_gameplay_show_resources.out` into a [`ConfigGameplayShowResourcesTable`].
pub fn parse_config_gameplay_show_resources(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGameplayShowResourcesTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_gm_common.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGmCommonRow {
    pub default_value: Box<str>,
    pub func: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub order: u8,
}

impl DataCfgRow for ConfigGmCommonRow {
    const TABLE_NAME: &'static str = "config_gm_common";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DefaultValue" => {
                self.default_value = ctx.decode_field(val, Self::TABLE_NAME, "DefaultValue")?
            }
            b"Func" => self.func = ctx.decode_field(val, Self::TABLE_NAME, "Func")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_gm_common.out`.
pub type ConfigGmCommonTable = DataTable<u8, ConfigGmCommonRow>;

/// Parses `config_gm_common.out` into a [`ConfigGmCommonTable`].
pub fn parse_config_gm_common(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGmCommonTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_gm_item_type_list.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGmItemTypeListRow {
    pub desc: Box<str>,
    pub filter: Box<str>,
    pub id: i8,
    pub source_table: Box<str>,
}

impl DataCfgRow for ConfigGmItemTypeListRow {
    const TABLE_NAME: &'static str = "config_gm_item_type_list";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Filter" => self.filter = ctx.decode_field(val, Self::TABLE_NAME, "Filter")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"SourceTable" => {
                self.source_table = ctx.decode_field(val, Self::TABLE_NAME, "SourceTable")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_gm_item_type_list.out`.
pub type ConfigGmItemTypeListTable = DataTable<u8, ConfigGmItemTypeListRow>;

/// Parses `config_gm_item_type_list.out` into a [`ConfigGmItemTypeListTable`].
pub fn parse_config_gm_item_type_list(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGmItemTypeListTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_gm_v2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGmV2Row {
    pub desc: Box<str>,
    pub func: Box<str>,
    pub id: i32,
    pub is_using: Box<str>,
    pub param10_constrain: Box<str>,
    pub param10_desc: Box<str>,
    pub param10_type: Box<str>,
    pub param1_constrain: Box<str>,
    pub param1_desc: Box<str>,
    pub param1_type: Box<str>,
    pub param2_constrain: Box<str>,
    pub param2_desc: Box<str>,
    pub param2_type: Box<str>,
    pub param3_constrain: Box<str>,
    pub param3_desc: Box<str>,
    pub param3_type: Box<str>,
    pub param4_constrain: Box<str>,
    pub param4_desc: Box<str>,
    pub param4_type: Box<str>,
    pub param5_constrain: Box<str>,
    pub param5_desc: Box<str>,
    pub param5_type: Box<str>,
    pub param6_constrain: Box<str>,
    pub param6_desc: Box<str>,
    pub param6_type: Box<str>,
    pub param7_constrain: Box<str>,
    pub param7_desc: Box<str>,
    pub param7_type: Box<str>,
    pub param8_constrain: Box<str>,
    pub param8_desc: Box<str>,
    pub param8_type: Box<str>,
    pub param9_constrain: Box<str>,
    pub param9_desc: Box<str>,
    pub param9_type: Box<str>,
    pub r#type: Box<str>,
}

impl ConfigGmV2Row {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Func" => self.func = ctx.decode_field(val, Self::TABLE_NAME, "Func")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsUsing" => self.is_using = ctx.decode_field(val, Self::TABLE_NAME, "IsUsing")?,
            b"Param10Constrain" => {
                self.param10_constrain =
                    ctx.decode_field(val, Self::TABLE_NAME, "Param10Constrain")?
            }
            b"Param10Desc" => {
                self.param10_desc = ctx.decode_field(val, Self::TABLE_NAME, "Param10Desc")?
            }
            b"Param10Type" => {
                self.param10_type = ctx.decode_field(val, Self::TABLE_NAME, "Param10Type")?
            }
            b"Param1Constrain" => {
                self.param1_constrain =
                    ctx.decode_field(val, Self::TABLE_NAME, "Param1Constrain")?
            }
            b"Param1Desc" => {
                self.param1_desc = ctx.decode_field(val, Self::TABLE_NAME, "Param1Desc")?
            }
            b"Param1Type" => {
                self.param1_type = ctx.decode_field(val, Self::TABLE_NAME, "Param1Type")?
            }
            b"Param2Constrain" => {
                self.param2_constrain =
                    ctx.decode_field(val, Self::TABLE_NAME, "Param2Constrain")?
            }
            b"Param2Desc" => {
                self.param2_desc = ctx.decode_field(val, Self::TABLE_NAME, "Param2Desc")?
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
            b"Param2Type" => {
                self.param2_type = ctx.decode_field(val, Self::TABLE_NAME, "Param2Type")?
            }
            b"Param3Constrain" => {
                self.param3_constrain =
                    ctx.decode_field(val, Self::TABLE_NAME, "Param3Constrain")?
            }
            b"Param3Desc" => {
                self.param3_desc = ctx.decode_field(val, Self::TABLE_NAME, "Param3Desc")?
            }
            b"Param3Type" => {
                self.param3_type = ctx.decode_field(val, Self::TABLE_NAME, "Param3Type")?
            }
            b"Param4Constrain" => {
                self.param4_constrain =
                    ctx.decode_field(val, Self::TABLE_NAME, "Param4Constrain")?
            }
            b"Param4Desc" => {
                self.param4_desc = ctx.decode_field(val, Self::TABLE_NAME, "Param4Desc")?
            }
            b"Param4Type" => {
                self.param4_type = ctx.decode_field(val, Self::TABLE_NAME, "Param4Type")?
            }
            b"Param5Constrain" => {
                self.param5_constrain =
                    ctx.decode_field(val, Self::TABLE_NAME, "Param5Constrain")?
            }
            b"Param5Desc" => {
                self.param5_desc = ctx.decode_field(val, Self::TABLE_NAME, "Param5Desc")?
            }
            b"Param5Type" => {
                self.param5_type = ctx.decode_field(val, Self::TABLE_NAME, "Param5Type")?
            }
            b"Param6Constrain" => {
                self.param6_constrain =
                    ctx.decode_field(val, Self::TABLE_NAME, "Param6Constrain")?
            }
            b"Param6Desc" => {
                self.param6_desc = ctx.decode_field(val, Self::TABLE_NAME, "Param6Desc")?
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
            b"Param6Type" => {
                self.param6_type = ctx.decode_field(val, Self::TABLE_NAME, "Param6Type")?
            }
            b"Param7Constrain" => {
                self.param7_constrain =
                    ctx.decode_field(val, Self::TABLE_NAME, "Param7Constrain")?
            }
            b"Param7Desc" => {
                self.param7_desc = ctx.decode_field(val, Self::TABLE_NAME, "Param7Desc")?
            }
            b"Param7Type" => {
                self.param7_type = ctx.decode_field(val, Self::TABLE_NAME, "Param7Type")?
            }
            b"Param8Constrain" => {
                self.param8_constrain =
                    ctx.decode_field(val, Self::TABLE_NAME, "Param8Constrain")?
            }
            b"Param8Desc" => {
                self.param8_desc = ctx.decode_field(val, Self::TABLE_NAME, "Param8Desc")?
            }
            b"Param8Type" => {
                self.param8_type = ctx.decode_field(val, Self::TABLE_NAME, "Param8Type")?
            }
            b"Param9Constrain" => {
                self.param9_constrain =
                    ctx.decode_field(val, Self::TABLE_NAME, "Param9Constrain")?
            }
            b"Param9Desc" => {
                self.param9_desc = ctx.decode_field(val, Self::TABLE_NAME, "Param9Desc")?
            }
            b"Param9Type" => {
                self.param9_type = ctx.decode_field(val, Self::TABLE_NAME, "Param9Type")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigGmV2Row {
    const TABLE_NAME: &'static str = "config_gm_v2";

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

/// Typed `DataTable` for `config_gm_v2.out`.
pub type ConfigGmV2Table = DataTable<u32, ConfigGmV2Row>;

/// Parses `config_gm_v2.out` into a [`ConfigGmV2Table`].
pub fn parse_config_gm_v2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGmV2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_gmlist_v2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGmlistV2Row {
    pub desc: Box<str>,
    pub gm: Box<str>,
    pub group: u16,
    pub group_order: u8,
    pub id: i32,
    pub param: Box<str>,
    pub r#type: Box<str>,
}

impl DataCfgRow for ConfigGmlistV2Row {
    const TABLE_NAME: &'static str = "config_gmlist_v2";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"GM" => self.gm = ctx.decode_field(val, Self::TABLE_NAME, "GM")?,
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"GroupOrder" => {
                self.group_order = ctx.decode_field(val, Self::TABLE_NAME, "GroupOrder")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Param" => self.param = ctx.decode_field(val, Self::TABLE_NAME, "Param")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_gmlist_v2.out`.
pub type ConfigGmlistV2Table = DataTable<u32, ConfigGmlistV2Row>;

/// Parses `config_gmlist_v2.out` into a [`ConfigGmlistV2Table`].
pub fn parse_config_gmlist_v2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGmlistV2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_god_filter.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGodFilterRow {
    pub id: i32,
    pub profession_ids: Box<[u8]>,
}

impl DataCfgRow for ConfigGodFilterRow {
    const TABLE_NAME: &'static str = "config_god_filter";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ProfessionIds" => {
                self.profession_ids = ctx.decode_field(val, Self::TABLE_NAME, "ProfessionIds")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_god_filter.out`.
pub type ConfigGodFilterTable = DataTable<u32, ConfigGodFilterRow>;

/// Parses `config_god_filter.out` into a [`ConfigGodFilterTable`].
pub fn parse_config_god_filter(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGodFilterTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_googleurl.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGoogleurlRow {
    pub language_code: Box<str>,
    pub language_google: Box<str>,
}

impl DataCfgRow for ConfigGoogleurlRow {
    const TABLE_NAME: &'static str = "config_googleurl";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"LanguageCode" => {
                self.language_code = ctx.decode_field(val, Self::TABLE_NAME, "LanguageCode")?
            }
            b"LanguageGoogle" => {
                self.language_google = ctx.decode_field(val, Self::TABLE_NAME, "LanguageGoogle")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_googleurl.out`.
pub type ConfigGoogleurlTable = DataTable<Box<str>, ConfigGoogleurlRow>;

/// Parses `config_googleurl.out` into a [`ConfigGoogleurlTable`].
pub fn parse_config_googleurl(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGoogleurlTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_hyperlink_param.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHyperlinkParamRow {
    pub id: i8,
    pub r#type: u8,
    pub value: Box<str>,
}

impl DataCfgRow for ConfigHyperlinkParamRow {
    const TABLE_NAME: &'static str = "config_hyperlink_param";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"Value" => self.value = ctx.decode_field(val, Self::TABLE_NAME, "Value")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_hyperlink_param.out`.
pub type ConfigHyperlinkParamTable = DataTable<u8, ConfigHyperlinkParamRow>;

/// Parses `config_hyperlink_param.out` into a [`ConfigHyperlinkParamTable`].
pub fn parse_config_hyperlink_param(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigHyperlinkParamTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_in_game_download.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigInGameDownloadRow {
    pub auto: u8,
    pub bg_img: Box<str>,
    pub file: Box<str>,
    pub id: i16,
    pub level: u8,
    pub name: Box<str>,
    pub order: u8,
    pub r#type: u8,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigInGameDownloadRow {
    const TABLE_NAME: &'static str = "config_in_game_download";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Auto" => self.auto = ctx.decode_field(val, Self::TABLE_NAME, "Auto")?,
            b"BgImg" => self.bg_img = ctx.decode_field(val, Self::TABLE_NAME, "BgImg")?,
            b"File" => self.file = ctx.decode_field(val, Self::TABLE_NAME, "File")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_in_game_download.out`.
pub type ConfigInGameDownloadTable = DataTable<u16, ConfigInGameDownloadRow>;

/// Parses `config_in_game_download.out` into a [`ConfigInGameDownloadTable`].
pub fn parse_config_in_game_download(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigInGameDownloadTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_in_game_download_dependency.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigInGameDownloadDependencyRow {
    pub chunk_id: Box<[u16]>,
    pub file_path: Box<str>,
    pub func_id: u8,
    pub id: i8,
    pub r#type: u8,
}

impl DataCfgRow for ConfigInGameDownloadDependencyRow {
    const TABLE_NAME: &'static str = "config_in_game_download_dependency";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ChunkId" => self.chunk_id = ctx.decode_field(val, Self::TABLE_NAME, "ChunkId")?,
            b"FilePath" => self.file_path = ctx.decode_field(val, Self::TABLE_NAME, "FilePath")?,
            b"FuncId" => self.func_id = ctx.decode_field(val, Self::TABLE_NAME, "FuncId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_in_game_download_dependency.out`.
pub type ConfigInGameDownloadDependencyTable = DataTable<u8, ConfigInGameDownloadDependencyRow>;

/// Parses `config_in_game_download_dependency.out` into a [`ConfigInGameDownloadDependencyTable`].
pub fn parse_config_in_game_download_dependency(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigInGameDownloadDependencyTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_jumpto.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigJumptoRow {
    pub data_report_type: Box<str>,
    pub id: i32,
    pub jump_str: Box<str>,
}

impl DataCfgRow for ConfigJumptoRow {
    const TABLE_NAME: &'static str = "config_jumpto";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DataReportType" => {
                self.data_report_type = ctx.decode_field(val, Self::TABLE_NAME, "DataReportType")?
            }
            b"ID" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "ID")?,
            b"JumpStr" => self.jump_str = ctx.decode_field(val, Self::TABLE_NAME, "JumpStr")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_jumpto.out`.
pub type ConfigJumptoTable = DataTable<u32, ConfigJumptoRow>;

/// Parses `config_jumpto.out` into a [`ConfigJumptoTable`].
pub fn parse_config_jumpto(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigJumptoTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_server_name.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServerNameRow {
    pub description: Box<str>,
    pub order: u8,
    pub server_app_id: Box<str>,
    pub _description: Box<str>,
}

impl DataCfgRow for ConfigServerNameRow {
    const TABLE_NAME: &'static str = "config_server_name";

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
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"ServerAppId" => {
                self.server_app_id = ctx.decode_field(val, Self::TABLE_NAME, "ServerAppId")?
            }
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_server_name.out`.
pub type ConfigServerNameTable = DataTable<Box<str>, ConfigServerNameRow>;

/// Parses `config_server_name.out` into a [`ConfigServerNameTable`].
pub fn parse_config_server_name(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigServerNameTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_setting_account_binding.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSettingAccountBindingRow {
    pub allow_email_binding: bool,
    pub allow_phone_binding: bool,
    pub display_order: u8,
    pub language_code: Box<str>,
    pub phone_prefix: Box<str>,
    pub region_code: Box<str>,
    pub region_name: Box<str>,
    pub _region_name: Box<str>,
}

impl DataCfgRow for ConfigSettingAccountBindingRow {
    const TABLE_NAME: &'static str = "config_setting_account_binding";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AllowEmailBinding" => {
                self.allow_email_binding =
                    ctx.decode_field(val, Self::TABLE_NAME, "AllowEmailBinding")?
            }
            b"AllowPhoneBinding" => {
                self.allow_phone_binding =
                    ctx.decode_field(val, Self::TABLE_NAME, "AllowPhoneBinding")?
            }
            b"DisplayOrder" => {
                self.display_order = ctx.decode_field(val, Self::TABLE_NAME, "DisplayOrder")?
            }
            b"LanguageCode" => {
                self.language_code = ctx.decode_field(val, Self::TABLE_NAME, "LanguageCode")?
            }
            b"PhonePrefix" => {
                self.phone_prefix = ctx.decode_field(val, Self::TABLE_NAME, "PhonePrefix")?
            }
            b"RegionCode" => {
                self.region_code = ctx.decode_field(val, Self::TABLE_NAME, "RegionCode")?
            }
            b"RegionName" => {
                self.region_name = ctx.decode_field(val, Self::TABLE_NAME, "RegionName")?
            }
            b"_RegionName" => {
                self._region_name = ctx.decode_field(val, Self::TABLE_NAME, "_RegionName")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_setting_account_binding.out`.
pub type ConfigSettingAccountBindingTable = DataTable<Box<str>, ConfigSettingAccountBindingRow>;

/// Parses `config_setting_account_binding.out` into a [`ConfigSettingAccountBindingTable`].
pub fn parse_config_setting_account_binding(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSettingAccountBindingTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_setting_agreement.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSettingAgreementRow {
    pub language_code: Box<str>,
    pub privacy_agreement: Box<str>,
    pub privacy_agreement_cn: Box<str>,
    pub special_agreement: Box<str>,
    pub special_name: Box<str>,
    pub user_agreement: Box<str>,
    pub user_agreement_cn: Box<str>,
}

impl DataCfgRow for ConfigSettingAgreementRow {
    const TABLE_NAME: &'static str = "config_setting_agreement";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"LanguageCode" => {
                self.language_code = ctx.decode_field(val, Self::TABLE_NAME, "LanguageCode")?
            }
            b"PrivacyAgreement" => {
                self.privacy_agreement =
                    ctx.decode_field(val, Self::TABLE_NAME, "PrivacyAgreement")?
            }
            b"PrivacyAgreementCn" => {
                self.privacy_agreement_cn =
                    ctx.decode_field(val, Self::TABLE_NAME, "PrivacyAgreementCn")?
            }
            b"SpecialAgreement" => {
                self.special_agreement =
                    ctx.decode_field(val, Self::TABLE_NAME, "SpecialAgreement")?
            }
            b"SpecialName" => {
                self.special_name = ctx.decode_field(val, Self::TABLE_NAME, "SpecialName")?
            }
            b"UserAgreement" => {
                self.user_agreement = ctx.decode_field(val, Self::TABLE_NAME, "UserAgreement")?
            }
            b"UserAgreementCn" => {
                self.user_agreement_cn =
                    ctx.decode_field(val, Self::TABLE_NAME, "UserAgreementCn")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_setting_agreement.out`.
pub type ConfigSettingAgreementTable = DataTable<Box<str>, ConfigSettingAgreementRow>;

/// Parses `config_setting_agreement.out` into a [`ConfigSettingAgreementTable`].
pub fn parse_config_setting_agreement(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSettingAgreementTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_setting_graphics.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSettingGraphicsRow {
    pub defult_balanced: u8,
    pub defult_epic: u8,
    pub defult_high: u8,
    pub defult_low: u8,
    pub defult_medium: u8,
    pub des: Box<str>,
    pub id: i8,
    pub options: Box<[u8]>,
    pub order: u8,
    pub platform: u8,
    pub r#type: u8,
    pub _des: Box<str>,
}

impl DataCfgRow for ConfigSettingGraphicsRow {
    const TABLE_NAME: &'static str = "config_setting_graphics";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DefultBalanced" => {
                self.defult_balanced = ctx.decode_field(val, Self::TABLE_NAME, "DefultBalanced")?
            }
            b"DefultEpic" => {
                self.defult_epic = ctx.decode_field(val, Self::TABLE_NAME, "DefultEpic")?
            }
            b"DefultHigh" => {
                self.defult_high = ctx.decode_field(val, Self::TABLE_NAME, "DefultHigh")?
            }
            b"DefultLow" => {
                self.defult_low = ctx.decode_field(val, Self::TABLE_NAME, "DefultLow")?
            }
            b"DefultMedium" => {
                self.defult_medium = ctx.decode_field(val, Self::TABLE_NAME, "DefultMedium")?
            }
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Options" => self.options = ctx.decode_field(val, Self::TABLE_NAME, "Options")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"Platform" => self.platform = ctx.decode_field(val, Self::TABLE_NAME, "Platform")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_setting_graphics.out`.
pub type ConfigSettingGraphicsTable = DataTable<u8, ConfigSettingGraphicsRow>;

/// Parses `config_setting_graphics.out` into a [`ConfigSettingGraphicsTable`].
pub fn parse_config_setting_graphics(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSettingGraphicsTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_setting_tab.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSettingTabRow {
    pub content: Box<[u16]>,
    pub defaults: Box<[u8]>,
    pub icon: Box<str>,
    pub id: i16,
    pub order: u8,
    pub show: u8,
    pub tab_title_desc: Box<str>,
    pub r#type: Box<str>,
}

impl DataCfgRow for ConfigSettingTabRow {
    const TABLE_NAME: &'static str = "config_setting_tab";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Content" => self.content = ctx.decode_field(val, Self::TABLE_NAME, "Content")?,
            b"Defaults" => self.defaults = ctx.decode_field(val, Self::TABLE_NAME, "Defaults")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"Show" => self.show = ctx.decode_field(val, Self::TABLE_NAME, "Show")?,
            b"TabTitleDesc" => {
                self.tab_title_desc = ctx.decode_field(val, Self::TABLE_NAME, "TabTitleDesc")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_setting_tab.out`.
pub type ConfigSettingTabTable = DataTable<u16, ConfigSettingTabRow>;

/// Parses `config_setting_tab.out` into a [`ConfigSettingTabTable`].
pub fn parse_config_setting_tab(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSettingTabTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_system_help.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSystemHelpRow {
    pub description: Box<str>,
    pub function_id: u16,
    pub help_type: u8,
    pub id: i8,
    pub name: Box<str>,
    pub picture_id: Box<[u16]>,
    pub scene_id: Box<str>,
    pub _description: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigSystemHelpRow {
    const TABLE_NAME: &'static str = "config_system_help";

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
            b"FunctionId" => {
                self.function_id = ctx.decode_field(val, Self::TABLE_NAME, "FunctionId")?
            }
            b"HelpType" => self.help_type = ctx.decode_field(val, Self::TABLE_NAME, "HelpType")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"PictureId" => {
                self.picture_id = ctx.decode_field(val, Self::TABLE_NAME, "PictureId")?
            }
            b"SceneId" => self.scene_id = ctx.decode_field(val, Self::TABLE_NAME, "SceneId")?,
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_system_help.out`.
pub type ConfigSystemHelpTable = DataTable<u8, ConfigSystemHelpRow>;

/// Parses `config_system_help.out` into a [`ConfigSystemHelpTable`].
pub fn parse_config_system_help(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSystemHelpTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_system_help_picture.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSystemHelpPictureRow {
    pub content: Box<str>,
    pub id: i16,
    pub pic: Box<str>,
    pub pic_en: Box<str>,
    pub title: Box<str>,
    pub _content: Box<str>,
    pub _title: Box<str>,
}

impl DataCfgRow for ConfigSystemHelpPictureRow {
    const TABLE_NAME: &'static str = "config_system_help_picture";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Content" => self.content = ctx.decode_field(val, Self::TABLE_NAME, "Content")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Pic" => self.pic = ctx.decode_field(val, Self::TABLE_NAME, "Pic")?,
            b"PicEn" => self.pic_en = ctx.decode_field(val, Self::TABLE_NAME, "PicEn")?,
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"_Content" => self._content = ctx.decode_field(val, Self::TABLE_NAME, "_Content")?,
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_system_help_picture.out`.
pub type ConfigSystemHelpPictureTable = DataTable<u16, ConfigSystemHelpPictureRow>;

/// Parses `config_system_help_picture.out` into a [`ConfigSystemHelpPictureTable`].
pub fn parse_config_system_help_picture(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSystemHelpPictureTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_ui_level_modelshow.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiLevelModelshowRow {
    pub character_rotation: Box<str>,
    pub character_scale: Box<str>,
    pub id: i8,
    pub pet_rotation: Box<str>,
    pub pet_scale: Box<str>,
}

impl DataCfgRow for ConfigUiLevelModelshowRow {
    const TABLE_NAME: &'static str = "config_ui_level_modelshow";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CharacterRotation" => {
                self.character_rotation =
                    ctx.decode_field(val, Self::TABLE_NAME, "CharacterRotation")?
            }
            b"CharacterScale" => {
                self.character_scale = ctx.decode_field(val, Self::TABLE_NAME, "CharacterScale")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"PetRotation" => {
                self.pet_rotation = ctx.decode_field(val, Self::TABLE_NAME, "PetRotation")?
            }
            b"PetScale" => self.pet_scale = ctx.decode_field(val, Self::TABLE_NAME, "PetScale")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_ui_level_modelshow.out`.
pub type ConfigUiLevelModelshowTable = DataTable<u8, ConfigUiLevelModelshowRow>;

/// Parses `config_ui_level_modelshow.out` into a [`ConfigUiLevelModelshowTable`].
pub fn parse_config_ui_level_modelshow(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigUiLevelModelshowTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_ui_level_path.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigUiLevelPathRow {
    pub name: Box<str>,
    pub path: Box<str>,
}

impl DataCfgRow for ConfigUiLevelPathRow {
    const TABLE_NAME: &'static str = "config_ui_level_path";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Path" => self.path = ctx.decode_field(val, Self::TABLE_NAME, "Path")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_ui_level_path.out`.
pub type ConfigUiLevelPathTable = DataTable<Box<str>, ConfigUiLevelPathRow>;

/// Parses `config_ui_level_path.out` into a [`ConfigUiLevelPathTable`].
pub fn parse_config_ui_level_path(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigUiLevelPathTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_ui_scale_special_ruler.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigUiScaleSpecialRulerRow {
    pub id: i8,
    pub scaling_factor: f64,
}

impl DataCfgRow for ConfigUiScaleSpecialRulerRow {
    const TABLE_NAME: &'static str = "config_ui_scale_special_ruler";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ScalingFactor" => {
                self.scaling_factor = ctx.decode_field(val, Self::TABLE_NAME, "ScalingFactor")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_ui_scale_special_ruler.out`.
pub type ConfigUiScaleSpecialRulerTable = DataTable<u8, ConfigUiScaleSpecialRulerRow>;

/// Parses `config_ui_scale_special_ruler.out` into a [`ConfigUiScaleSpecialRulerTable`].
pub fn parse_config_ui_scale_special_ruler(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigUiScaleSpecialRulerTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

fn verify_system_chunk_0(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_account_bind" => Some(parse_config_account_bind(buf, scratch).map(|t| t.len())),
        "config_common_tips" => Some(parse_config_common_tips(buf, scratch).map(|t| t.len())),
        "config_cond_event" => Some(parse_config_cond_event(buf, scratch).map(|t| t.len())),
        "config_data_fix" => Some(parse_config_data_fix(buf, scratch).map(|t| t.len())),
        "config_gamepad_tips" => Some(parse_config_gamepad_tips(buf, scratch).map(|t| t.len())),
        "config_gameplay_duration" => {
            Some(parse_config_gameplay_duration(buf, scratch).map(|t| t.len()))
        }
        "config_gameplay_entry_content" => {
            Some(parse_config_gameplay_entry_content(buf, scratch).map(|t| t.len()))
        }
        "config_gameplay_show_resources" => {
            Some(parse_config_gameplay_show_resources(buf, scratch).map(|t| t.len()))
        }
        "config_gm_common" => Some(parse_config_gm_common(buf, scratch).map(|t| t.len())),
        "config_gm_item_type_list" => {
            Some(parse_config_gm_item_type_list(buf, scratch).map(|t| t.len()))
        }
        "config_gm_v2" => Some(parse_config_gm_v2(buf, scratch).map(|t| t.len())),
        "config_gmlist_v2" => Some(parse_config_gmlist_v2(buf, scratch).map(|t| t.len())),
        "config_god_filter" => Some(parse_config_god_filter(buf, scratch).map(|t| t.len())),
        "config_googleurl" => Some(parse_config_googleurl(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

fn verify_system_chunk_1(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_hyperlink_param" => {
            Some(parse_config_hyperlink_param(buf, scratch).map(|t| t.len()))
        }
        "config_in_game_download" => {
            Some(parse_config_in_game_download(buf, scratch).map(|t| t.len()))
        }
        "config_in_game_download_dependency" => {
            Some(parse_config_in_game_download_dependency(buf, scratch).map(|t| t.len()))
        }
        "config_jumpto" => Some(parse_config_jumpto(buf, scratch).map(|t| t.len())),
        "config_server_name" => Some(parse_config_server_name(buf, scratch).map(|t| t.len())),
        "config_setting_account_binding" => {
            Some(parse_config_setting_account_binding(buf, scratch).map(|t| t.len()))
        }
        "config_setting_agreement" => {
            Some(parse_config_setting_agreement(buf, scratch).map(|t| t.len()))
        }
        "config_setting_graphics" => {
            Some(parse_config_setting_graphics(buf, scratch).map(|t| t.len()))
        }
        "config_setting_tab" => Some(parse_config_setting_tab(buf, scratch).map(|t| t.len())),
        "config_system_help" => Some(parse_config_system_help(buf, scratch).map(|t| t.len())),
        "config_system_help_picture" => {
            Some(parse_config_system_help_picture(buf, scratch).map(|t| t.len()))
        }
        "config_ui_level_modelshow" => {
            Some(parse_config_ui_level_modelshow(buf, scratch).map(|t| t.len()))
        }
        "config_ui_level_path" => Some(parse_config_ui_level_path(buf, scratch).map(|t| t.len())),
        "config_ui_scale_special_ruler" => {
            Some(parse_config_ui_scale_special_ruler(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

pub(crate) fn verify_system_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    if let Some(res) = verify_system_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_system_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    None
}

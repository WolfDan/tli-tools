//! Auto-generated `LuaTable/DataCfg` types and parsers for the `servant` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;

/// Row record for `config_servant_auto_apply.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantAutoApplyRow {
    pub free_use: u8,
    pub id: i16,
    pub lock_des: Box<str>,
    pub name: Box<str>,
    pub order: u8,
    pub rule_des: Box<str>,
    pub _lock_des: Box<str>,
    pub _name: Box<str>,
    pub _rule_des: Box<str>,
}

impl DataCfgRow for ConfigServantAutoApplyRow {
    const TABLE_NAME: &'static str = "config_servant_auto_apply";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"FreeUse" => self.free_use = ctx.decode_field(val, Self::TABLE_NAME, "FreeUse")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LockDes" => self.lock_des = ctx.decode_field(val, Self::TABLE_NAME, "LockDes")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"RuleDes" => self.rule_des = ctx.decode_field(val, Self::TABLE_NAME, "RuleDes")?,
            b"_LockDes" => self._lock_des = ctx.decode_field(val, Self::TABLE_NAME, "_LockDes")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_RuleDes" => self._rule_des = ctx.decode_field(val, Self::TABLE_NAME, "_RuleDes")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_servant_auto_apply.out`.
pub type ConfigServantAutoApplyTable = DataTable<u16, ConfigServantAutoApplyRow>;

/// Parses `config_servant_auto_apply.out` into a [`ConfigServantAutoApplyTable`].
pub fn parse_config_servant_auto_apply(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigServantAutoApplyTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_servant_auto_switch.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantAutoSwitchRow {
    pub id: i8,
    pub is_deprecated: bool,
    pub map_types: Box<[u8]>,
    pub order: u8,
    pub run_server: u8,
    pub title: Box<str>,
    pub _title: Box<str>,
}

impl DataCfgRow for ConfigServantAutoSwitchRow {
    const TABLE_NAME: &'static str = "config_servant_auto_switch";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsDeprecated" => {
                self.is_deprecated = ctx.decode_field(val, Self::TABLE_NAME, "IsDeprecated")?
            }
            b"MapTypes" => self.map_types = ctx.decode_field(val, Self::TABLE_NAME, "MapTypes")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"RunServer" => {
                self.run_server = ctx.decode_field(val, Self::TABLE_NAME, "RunServer")?
            }
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_servant_auto_switch.out`.
pub type ConfigServantAutoSwitchTable = DataTable<u8, ConfigServantAutoSwitchRow>;

/// Parses `config_servant_auto_switch.out` into a [`ConfigServantAutoSwitchTable`].
pub fn parse_config_servant_auto_switch(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigServantAutoSwitchTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_servant_show_resources.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantShowResourcesRow {
    pub id: i16,
    pub material_res: Box<str>,
    pub resource_material_ids: Box<[u32]>,
    pub show_action: Box<str>,
    pub show_action2: Box<str>,
    pub show_action3: Box<str>,
    pub skeletal_res: Box<str>,
}

impl DataCfgRow for ConfigServantShowResourcesRow {
    const TABLE_NAME: &'static str = "config_servant_show_resources";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MaterialRes" => {
                self.material_res = ctx.decode_field(val, Self::TABLE_NAME, "MaterialRes")?
            }
            b"ResourceMaterialIds" => {
                self.resource_material_ids =
                    ctx.decode_field(val, Self::TABLE_NAME, "ResourceMaterialIds")?
            }
            b"ShowAction" => {
                self.show_action = ctx.decode_field(val, Self::TABLE_NAME, "ShowAction")?
            }
            b"ShowAction2" => {
                self.show_action2 = ctx.decode_field(val, Self::TABLE_NAME, "ShowAction2")?
            }
            b"ShowAction3" => {
                self.show_action3 = ctx.decode_field(val, Self::TABLE_NAME, "ShowAction3")?
            }
            b"SkeletalRes" => {
                self.skeletal_res = ctx.decode_field(val, Self::TABLE_NAME, "SkeletalRes")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_servant_show_resources.out`.
pub type ConfigServantShowResourcesTable = DataTable<u8, ConfigServantShowResourcesRow>;

/// Parses `config_servant_show_resources.out` into a [`ConfigServantShowResourcesTable`].
pub fn parse_config_servant_show_resources(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigServantShowResourcesTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_servant_slot_unlock_condition.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantSlotUnlockConditionRow {
    pub id: i8,
    pub servant_install_type: u8,
    pub unlock_level: u8,
    pub unlock_task_id: u32,
}

impl DataCfgRow for ConfigServantSlotUnlockConditionRow {
    const TABLE_NAME: &'static str = "config_servant_slot_unlock_condition";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ServantInstallType" => {
                self.servant_install_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "ServantInstallType")?
            }
            b"UnlockLevel" => {
                self.unlock_level = ctx.decode_field(val, Self::TABLE_NAME, "UnlockLevel")?
            }
            b"UnlockTaskId" => {
                self.unlock_task_id = ctx.decode_field(val, Self::TABLE_NAME, "UnlockTaskId")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_servant_slot_unlock_condition.out`.
pub type ConfigServantSlotUnlockConditionTable = DataTable<u8, ConfigServantSlotUnlockConditionRow>;

/// Parses `config_servant_slot_unlock_condition.out` into a [`ConfigServantSlotUnlockConditionTable`].
pub fn parse_config_servant_slot_unlock_condition(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigServantSlotUnlockConditionTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_servant_subscription.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantSubscriptionRow {
    pub group_id: Box<[u8]>,
    pub id: i32,
    pub replace_interval: u32,
    pub seasons: Box<[u16]>,
    pub slot_num: u8,
}

impl DataCfgRow for ConfigServantSubscriptionRow {
    const TABLE_NAME: &'static str = "config_servant_subscription";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"GroupId" => self.group_id = ctx.decode_field(val, Self::TABLE_NAME, "GroupId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ReplaceInterval" => {
                self.replace_interval =
                    ctx.decode_field(val, Self::TABLE_NAME, "ReplaceInterval")?
            }
            b"Seasons" => self.seasons = ctx.decode_field(val, Self::TABLE_NAME, "Seasons")?,
            b"SlotNum" => self.slot_num = ctx.decode_field(val, Self::TABLE_NAME, "SlotNum")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_servant_subscription.out`.
pub type ConfigServantSubscriptionTable = DataTable<u32, ConfigServantSubscriptionRow>;

/// Parses `config_servant_subscription.out` into a [`ConfigServantSubscriptionTable`].
pub fn parse_config_servant_subscription(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigServantSubscriptionTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Sub-record for `config_servant_subscription_group.LeftServantPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServantSubscriptionGroupLeftServantPos {
    pub x: u8,
    pub y: i16,
    pub z: u8,
}

impl FromEvalVal for ServantSubscriptionGroupLeftServantPos {
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let mut out = Self::default();
        for &(k, v) in ctx.table_map(val, table, field)? {
            match ctx.decode_str_key(k, table, field)? {
                b"X" => out.x = ctx.decode_field(v, table, field)?,
                b"Y" => out.y = ctx.decode_field(v, table, field)?,
                b"Z" => out.z = ctx.decode_field(v, table, field)?,
                other => return Err(ctx.unknown_field(table, other)),
            }
        }
        Ok(out)
    }
}

/// Sub-record for `config_servant_subscription_group.RightServantPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServantSubscriptionGroupRightServantPos {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl FromEvalVal for ServantSubscriptionGroupRightServantPos {
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let mut out = Self::default();
        for &(k, v) in ctx.table_map(val, table, field)? {
            match ctx.decode_str_key(k, table, field)? {
                b"X" => out.x = ctx.decode_field(v, table, field)?,
                b"Y" => out.y = ctx.decode_field(v, table, field)?,
                b"Z" => out.z = ctx.decode_field(v, table, field)?,
                other => return Err(ctx.unknown_field(table, other)),
            }
        }
        Ok(out)
    }
}

/// Entry for `config_servant_subscription_group.Servant`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServantSubscriptionGroupServantEntry {
    pub servant_id: u32,
    pub star_level: u8,
}

impl FromEvalVal for ServantSubscriptionGroupServantEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (servant_id, star_level) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            servant_id,
            star_level,
        })
    }
}

/// Row record for `config_servant_subscription_group.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantSubscriptionGroupRow {
    pub id: i8,
    pub left_servant_pos: ServantSubscriptionGroupLeftServantPos,
    pub order: u8,
    pub right_servant_pos: ServantSubscriptionGroupRightServantPos,
    pub servant: Box<[ServantSubscriptionGroupServantEntry]>,
    pub r#type: u8,
}

impl DataCfgRow for ConfigServantSubscriptionGroupRow {
    const TABLE_NAME: &'static str = "config_servant_subscription_group";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LeftServantPos" => {
                self.left_servant_pos = ctx.decode_field(val, Self::TABLE_NAME, "LeftServantPos")?
            }
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"RightServantPos" => {
                self.right_servant_pos =
                    ctx.decode_field(val, Self::TABLE_NAME, "RightServantPos")?
            }
            b"Servant" => self.servant = ctx.decode_field(val, Self::TABLE_NAME, "Servant")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_servant_subscription_group.out`.
pub type ConfigServantSubscriptionGroupTable = DataTable<u8, ConfigServantSubscriptionGroupRow>;

/// Parses `config_servant_subscription_group.out` into a [`ConfigServantSubscriptionGroupTable`].
pub fn parse_config_servant_subscription_group(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigServantSubscriptionGroupTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_servant_subscription_mode.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigServantSubscriptionModeRow {
    pub diameter_m: f64,
    pub id: i8,
    pub servant_id: u8,
}

impl DataCfgRow for ConfigServantSubscriptionModeRow {
    const TABLE_NAME: &'static str = "config_servant_subscription_mode";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DiameterM" => {
                self.diameter_m = ctx.decode_field(val, Self::TABLE_NAME, "DiameterM")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ServantId" => {
                self.servant_id = ctx.decode_field(val, Self::TABLE_NAME, "ServantId")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_servant_subscription_mode.out`.
pub type ConfigServantSubscriptionModeTable = DataTable<u8, ConfigServantSubscriptionModeRow>;

/// Parses `config_servant_subscription_mode.out` into a [`ConfigServantSubscriptionModeTable`].
pub fn parse_config_servant_subscription_mode(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigServantSubscriptionModeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_servant_timelimit.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantTimelimitRow {
    pub id: i32,
    pub original_id: u16,
}

impl DataCfgRow for ConfigServantTimelimitRow {
    const TABLE_NAME: &'static str = "config_servant_timelimit";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"OriginalId" => {
                self.original_id = ctx.decode_field(val, Self::TABLE_NAME, "OriginalId")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_servant_timelimit.out`.
pub type ConfigServantTimelimitTable = DataTable<u32, ConfigServantTimelimitRow>;

/// Parses `config_servant_timelimit.out` into a [`ConfigServantTimelimitTable`].
pub fn parse_config_servant_timelimit(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigServantTimelimitTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_servant_type.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigServantTypeRow {
    pub id: i16,
    pub name: Box<str>,
    pub pickcard_order: u8,
    pub pickcard_usage_show: u8,
    pub tag_order: u8,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigServantTypeRow {
    const TABLE_NAME: &'static str = "config_servant_type";

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
            b"PickcardOrder" => {
                self.pickcard_order = ctx.decode_field(val, Self::TABLE_NAME, "PickcardOrder")?
            }
            b"PickcardUsageShow" => {
                self.pickcard_usage_show =
                    ctx.decode_field(val, Self::TABLE_NAME, "PickcardUsageShow")?
            }
            b"TagOrder" => self.tag_order = ctx.decode_field(val, Self::TABLE_NAME, "TagOrder")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_servant_type.out`.
pub type ConfigServantTypeTable = DataTable<u16, ConfigServantTypeRow>;

/// Parses `config_servant_type.out` into a [`ConfigServantTypeTable`].
pub fn parse_config_servant_type(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigServantTypeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

pub(crate) fn verify_servant_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_servant_auto_apply" => {
            Some(parse_config_servant_auto_apply(buf, scratch).map(|t| t.len()))
        }
        "config_servant_auto_switch" => {
            Some(parse_config_servant_auto_switch(buf, scratch).map(|t| t.len()))
        }
        "config_servant_show_resources" => {
            Some(parse_config_servant_show_resources(buf, scratch).map(|t| t.len()))
        }
        "config_servant_slot_unlock_condition" => {
            Some(parse_config_servant_slot_unlock_condition(buf, scratch).map(|t| t.len()))
        }
        "config_servant_subscription" => {
            Some(parse_config_servant_subscription(buf, scratch).map(|t| t.len()))
        }
        "config_servant_subscription_group" => {
            Some(parse_config_servant_subscription_group(buf, scratch).map(|t| t.len()))
        }
        "config_servant_subscription_mode" => {
            Some(parse_config_servant_subscription_mode(buf, scratch).map(|t| t.len()))
        }
        "config_servant_timelimit" => {
            Some(parse_config_servant_timelimit(buf, scratch).map(|t| t.len()))
        }
        "config_servant_type" => Some(parse_config_servant_type(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

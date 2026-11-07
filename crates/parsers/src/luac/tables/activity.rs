//! Auto-generated `LuaTable/DataCfg` types and parsers for the `activity` domain.
use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
/// Row record for `config_accrual_recharge.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAccrualRechargeRow {
    pub reward_group_id: u32,
    pub id: i16,
    pub season_id: u16,
    pub target: u16,
}
impl DataCfgRow for ConfigAccrualRechargeRow {
    const TABLE_NAME: &'static str = "config_accrual_recharge";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"RewardGroupId" => {
                self.reward_group_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "RewardGroupId")?;
            }
            b"SeasonId" => {
                self.season_id = ctx.decode_field(val, Self::TABLE_NAME, "SeasonId")?;
            }
            b"Target" => self.target = ctx.decode_field(val, Self::TABLE_NAME, "Target")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_accrual_recharge.out`.
pub type ConfigAccrualRechargeTable = DataTable<u16, ConfigAccrualRechargeRow>;
/// Parses `config_accrual_recharge.out` into a [`ConfigAccrualRechargeTable`].
pub fn parse_config_accrual_recharge(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAccrualRechargeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_activity_form.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigActivityFormRow {
    pub name: Box<str>,
    pub type_icon_selected: Box<str>,
    pub type_icon_unselected: Box<str>,
    pub _name: Box<str>,
    pub id: i8,
    pub order: u8,
}
impl DataCfgRow for ConfigActivityFormRow {
    const TABLE_NAME: &'static str = "config_activity_form";
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
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"TypeIconSelected" => {
                self.type_icon_selected = ctx
                    .decode_field(val, Self::TABLE_NAME, "TypeIconSelected")?;
            }
            b"TypeIconUnselected" => {
                self.type_icon_unselected = ctx
                    .decode_field(val, Self::TABLE_NAME, "TypeIconUnselected")?;
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_activity_form.out`.
pub type ConfigActivityFormTable = DataTable<u8, ConfigActivityFormRow>;
/// Parses `config_activity_form.out` into a [`ConfigActivityFormTable`].
pub fn parse_config_activity_form(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigActivityFormTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_activity_level_map.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigActivityLevelMapRow {
    pub finisheventmsg: Box<str>,
    pub resurrection_type: Box<[u8]>,
    pub _finisheventmsg: Box<str>,
    pub level_id: u32,
    pub id: i16,
    pub diedeductexperience: u8,
}
impl DataCfgRow for ConfigActivityLevelMapRow {
    const TABLE_NAME: &'static str = "config_activity_level_map";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Diedeductexperience" => {
                self.diedeductexperience = ctx
                    .decode_field(val, Self::TABLE_NAME, "Diedeductexperience")?;
            }
            b"Finisheventmsg" => {
                self.finisheventmsg = ctx
                    .decode_field(val, Self::TABLE_NAME, "Finisheventmsg")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LevelId" => {
                self.level_id = ctx.decode_field(val, Self::TABLE_NAME, "LevelId")?;
            }
            b"ResurrectionType" => {
                self.resurrection_type = ctx
                    .decode_field(val, Self::TABLE_NAME, "ResurrectionType")?;
            }
            b"_Finisheventmsg" => {
                self._finisheventmsg = ctx
                    .decode_field(val, Self::TABLE_NAME, "_Finisheventmsg")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_activity_level_map.out`.
pub type ConfigActivityLevelMapTable = DataTable<u16, ConfigActivityLevelMapRow>;
/// Parses `config_activity_level_map.out` into a [`ConfigActivityLevelMapTable`].
pub fn parse_config_activity_level_map(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigActivityLevelMapTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_activity_socialmedia_link.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigActivitySocialmediaLinkRow {
    pub description: Box<str>,
    pub link: Box<str>,
    pub pic: Box<str>,
    pub _description: Box<str>,
    pub id: i8,
}
impl DataCfgRow for ConfigActivitySocialmediaLinkRow {
    const TABLE_NAME: &'static str = "config_activity_socialmedia_link";
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
            b"Link" => self.link = ctx.decode_field(val, Self::TABLE_NAME, "Link")?,
            b"Pic" => self.pic = ctx.decode_field(val, Self::TABLE_NAME, "Pic")?,
            b"_Description" => {
                self._description = ctx
                    .decode_field(val, Self::TABLE_NAME, "_Description")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_activity_socialmedia_link.out`.
pub type ConfigActivitySocialmediaLinkTable = DataTable<
    u8,
    ConfigActivitySocialmediaLinkRow,
>;
/// Parses `config_activity_socialmedia_link.out` into a [`ConfigActivitySocialmediaLinkTable`].
pub fn parse_config_activity_socialmedia_link(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigActivitySocialmediaLinkTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_appstorecard.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAppstorecardRow {
    pub jump_expression: Box<str>,
    pub web_url: Box<str>,
}
impl DataCfgRow for ConfigAppstorecardRow {
    const TABLE_NAME: &'static str = "config_appstorecard";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"JumpExpression" => {
                self.jump_expression = ctx
                    .decode_field(val, Self::TABLE_NAME, "JumpExpression")?;
            }
            b"WebUrl" => {
                self.web_url = ctx.decode_field(val, Self::TABLE_NAME, "WebUrl")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_appstorecard.out`.
pub type ConfigAppstorecardTable = DataTable<Box<str>, ConfigAppstorecardRow>;
/// Parses `config_appstorecard.out` into a [`ConfigAppstorecardTable`].
pub fn parse_config_appstorecard(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAppstorecardTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_boss_challenge_detail.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigBossChallengeDetailRow {
    pub mechanic_desc: Box<str>,
    pub _mechanic_desc: Box<str>,
    pub id: i8,
}
impl DataCfgRow for ConfigBossChallengeDetailRow {
    const TABLE_NAME: &'static str = "config_boss_challenge_detail";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MechanicDesc" => {
                self.mechanic_desc = ctx
                    .decode_field(val, Self::TABLE_NAME, "MechanicDesc")?;
            }
            b"_MechanicDesc" => {
                self._mechanic_desc = ctx
                    .decode_field(val, Self::TABLE_NAME, "_MechanicDesc")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_boss_challenge_detail.out`.
pub type ConfigBossChallengeDetailTable = DataTable<u8, ConfigBossChallengeDetailRow>;
/// Parses `config_boss_challenge_detail.out` into a [`ConfigBossChallengeDetailTable`].
pub fn parse_config_boss_challenge_detail(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigBossChallengeDetailTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_functionpreview.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFunctionpreviewRow {
    pub desc: Box<str>,
    pub function_prereivew_task_id: Box<[u16]>,
    pub lock_desc: Box<str>,
    pub lock_desc_param: Box<str>,
    pub name: Box<str>,
    pub short_desc: Box<str>,
    pub tag_desc: Box<str>,
    pub _desc: Box<str>,
    pub _lock_desc: Box<str>,
    pub _lock_desc_param: Box<str>,
    pub _name: Box<str>,
    pub _short_desc: Box<str>,
    pub _tag_desc: Box<str>,
    pub unlock_event_id: u32,
    pub id: i16,
    pub order_id: u16,
}
impl ConfigFunctionpreviewRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"FunctionPrereivewTaskId" => {
                self.function_prereivew_task_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "FunctionPrereivewTaskId")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LockDesc" => {
                self.lock_desc = ctx.decode_field(val, Self::TABLE_NAME, "LockDesc")?;
            }
            b"LockDescParam" => {
                self.lock_desc_param = ctx
                    .decode_field(val, Self::TABLE_NAME, "LockDescParam")?;
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"OrderId" => {
                self.order_id = ctx.decode_field(val, Self::TABLE_NAME, "OrderId")?;
            }
            b"ShortDesc" => {
                self.short_desc = ctx.decode_field(val, Self::TABLE_NAME, "ShortDesc")?;
            }
            b"TagDesc" => {
                self.tag_desc = ctx.decode_field(val, Self::TABLE_NAME, "TagDesc")?;
            }
            b"UnlockEventId" => {
                self.unlock_event_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "UnlockEventId")?;
            }
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_LockDesc" => {
                self._lock_desc = ctx.decode_field(val, Self::TABLE_NAME, "_LockDesc")?;
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
            b"_LockDescParam" => {
                self._lock_desc_param = ctx
                    .decode_field(val, Self::TABLE_NAME, "_LockDescParam")?;
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_ShortDesc" => {
                self._short_desc = ctx.decode_field(val, Self::TABLE_NAME, "_ShortDesc")?;
            }
            b"_TagDesc" => {
                self._tag_desc = ctx.decode_field(val, Self::TABLE_NAME, "_TagDesc")?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigFunctionpreviewRow {
    const TABLE_NAME: &'static str = "config_functionpreview";
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
/// Typed `DataTable` for `config_functionpreview.out`.
pub type ConfigFunctionpreviewTable = DataTable<u16, ConfigFunctionpreviewRow>;
/// Parses `config_functionpreview.out` into a [`ConfigFunctionpreviewTable`].
pub fn parse_config_functionpreview(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigFunctionpreviewTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_functionpreview_task.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFunctionpreviewTaskRow {
    pub reward_id: Box<[u32]>,
    pub event_id: u32,
    pub unlock_event_id: u32,
    pub id: i16,
    pub group: u8,
}
impl DataCfgRow for ConfigFunctionpreviewTaskRow {
    const TABLE_NAME: &'static str = "config_functionpreview_task";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"EventId" => {
                self.event_id = ctx.decode_field(val, Self::TABLE_NAME, "EventId")?;
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"RewardId" => {
                self.reward_id = ctx.decode_field(val, Self::TABLE_NAME, "RewardId")?;
            }
            b"UnlockEventId" => {
                self.unlock_event_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "UnlockEventId")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_functionpreview_task.out`.
pub type ConfigFunctionpreviewTaskTable = DataTable<u16, ConfigFunctionpreviewTaskRow>;
/// Parses `config_functionpreview_task.out` into a [`ConfigFunctionpreviewTaskTable`].
pub fn parse_config_functionpreview_task(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigFunctionpreviewTaskTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_growth_fund.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGrowthFundRow {
    pub desc: Box<str>,
    pub events: Box<[u8]>,
    pub season_range: Box<[u8]>,
    pub _desc: Box<str>,
    pub unlock_reward: u32,
    pub id: i16,
}
impl DataCfgRow for ConfigGrowthFundRow {
    const TABLE_NAME: &'static str = "config_growth_fund";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Events" => self.events = ctx.decode_field(val, Self::TABLE_NAME, "Events")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"SeasonRange" => {
                self.season_range = ctx
                    .decode_field(val, Self::TABLE_NAME, "SeasonRange")?;
            }
            b"UnlockReward" => {
                self.unlock_reward = ctx
                    .decode_field(val, Self::TABLE_NAME, "UnlockReward")?;
            }
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_growth_fund.out`.
pub type ConfigGrowthFundTable = DataTable<u16, ConfigGrowthFundRow>;
/// Parses `config_growth_fund.out` into a [`ConfigGrowthFundTable`].
pub fn parse_config_growth_fund(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGrowthFundTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_growth_fund_event.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGrowthFundEventRow {
    pub cond_event_id: u32,
    pub hyper_link_event_id: u32,
    pub reward: u32,
    pub id: i8,
}
impl DataCfgRow for ConfigGrowthFundEventRow {
    const TABLE_NAME: &'static str = "config_growth_fund_event";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CondEventId" => {
                self.cond_event_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "CondEventId")?;
            }
            b"HyperLinkEventId" => {
                self.hyper_link_event_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "HyperLinkEventId")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Reward" => self.reward = ctx.decode_field(val, Self::TABLE_NAME, "Reward")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_growth_fund_event.out`.
pub type ConfigGrowthFundEventTable = DataTable<u8, ConfigGrowthFundEventRow>;
/// Parses `config_growth_fund_event.out` into a [`ConfigGrowthFundEventTable`].
pub fn parse_config_growth_fund_event(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGrowthFundEventTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_levelupvoucher.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigLevelupvoucherRow {
    pub voucher_money_detail: Box<str>,
    pub existing_character_level_required: u8,
    pub existing_character_num_required: u8,
    pub id: i8,
    pub limit_offset_time: u8,
    pub target_level: u8,
    pub voucher_cost_num: u8,
}
impl DataCfgRow for ConfigLevelupvoucherRow {
    const TABLE_NAME: &'static str = "config_levelupvoucher";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ExistingCharacterLevelRequired" => {
                self.existing_character_level_required = ctx
                    .decode_field(
                        val,
                        Self::TABLE_NAME,
                        "ExistingCharacterLevelRequired",
                    )?;
            }
            b"ExistingCharacterNumRequired" => {
                self.existing_character_num_required = ctx
                    .decode_field(val, Self::TABLE_NAME, "ExistingCharacterNumRequired")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LimitOffsetTime" => {
                self.limit_offset_time = ctx
                    .decode_field(val, Self::TABLE_NAME, "LimitOffsetTime")?;
            }
            b"TargetLevel" => {
                self.target_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetLevel")?;
            }
            b"VoucherCostNum" => {
                self.voucher_cost_num = ctx
                    .decode_field(val, Self::TABLE_NAME, "VoucherCostNum")?;
            }
            b"VoucherMoneyDetail" => {
                self.voucher_money_detail = ctx
                    .decode_field(val, Self::TABLE_NAME, "VoucherMoneyDetail")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_levelupvoucher.out`.
pub type ConfigLevelupvoucherTable = DataTable<u8, ConfigLevelupvoucherRow>;
/// Parses `config_levelupvoucher.out` into a [`ConfigLevelupvoucherTable`].
pub fn parse_config_levelupvoucher(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigLevelupvoucherTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_month_card.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMonthCardRow {
    pub daily_reward_group: u32,
    pub purchase_reward_group: u32,
    pub id: i8,
    pub r#type: u8,
}
impl DataCfgRow for ConfigMonthCardRow {
    const TABLE_NAME: &'static str = "config_month_card";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DailyRewardGroup" => {
                self.daily_reward_group = ctx
                    .decode_field(val, Self::TABLE_NAME, "DailyRewardGroup")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"PurchaseRewardGroup" => {
                self.purchase_reward_group = ctx
                    .decode_field(val, Self::TABLE_NAME, "PurchaseRewardGroup")?;
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_month_card.out`.
pub type ConfigMonthCardTable = DataTable<u8, ConfigMonthCardRow>;
/// Parses `config_month_card.out` into a [`ConfigMonthCardTable`].
pub fn parse_config_month_card(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMonthCardTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_newbie_build_list.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigNewbieBuildListRow {
    pub builds_id: Box<[u32]>,
    pub character_id: i16,
}
impl DataCfgRow for ConfigNewbieBuildListRow {
    const TABLE_NAME: &'static str = "config_newbie_build_list";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BuildsId" => {
                self.builds_id = ctx.decode_field(val, Self::TABLE_NAME, "BuildsId")?;
            }
            b"CharacterId" => {
                self.character_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "CharacterId")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_newbie_build_list.out`.
pub type ConfigNewbieBuildListTable = DataTable<u16, ConfigNewbieBuildListRow>;
/// Parses `config_newbie_build_list.out` into a [`ConfigNewbieBuildListTable`].
pub fn parse_config_newbie_build_list(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigNewbieBuildListTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Entry for `config_newbie_builds.Content`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewbieBuildsContentEntry {
    pub content_ids: Box<[u32]>,
    pub group_id: u8,
}
impl FromEvalVal for NewbieBuildsContentEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (group_id, content_ids) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { content_ids, group_id })
    }
}
/// Row record for `config_newbie_builds.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigNewbieBuildsRow {
    pub build_tips_id: Box<[u32]>,
    pub content: Box<[NewbieBuildsContentEntry]>,
    pub content_text: Box<str>,
    pub main_title: Box<str>,
    pub sub_node_id: Box<[u32]>,
    pub sub_title: Box<str>,
    pub tab_icon: Box<str>,
    pub tips_bg_icon: Box<str>,
    pub type2_icon: Box<str>,
    pub _content_text: Box<str>,
    pub _main_title: Box<str>,
    pub _sub_title: Box<str>,
    pub id: i32,
    pub jump_function_id: u8,
    pub r#type: u8,
}
impl ConfigNewbieBuildsRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"BuildTipsId" => {
                self.build_tips_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "BuildTipsId")?;
            }
            b"Content" => {
                self.content = ctx.decode_field(val, Self::TABLE_NAME, "Content")?;
            }
            b"ContentText" => {
                self.content_text = ctx
                    .decode_field(val, Self::TABLE_NAME, "ContentText")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"JumpFunctionId" => {
                self.jump_function_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "JumpFunctionId")?;
            }
            b"MainTitle" => {
                self.main_title = ctx.decode_field(val, Self::TABLE_NAME, "MainTitle")?;
            }
            b"SubNodeId" => {
                self.sub_node_id = ctx.decode_field(val, Self::TABLE_NAME, "SubNodeId")?;
            }
            b"SubTitle" => {
                self.sub_title = ctx.decode_field(val, Self::TABLE_NAME, "SubTitle")?;
            }
            b"TabIcon" => {
                self.tab_icon = ctx.decode_field(val, Self::TABLE_NAME, "TabIcon")?;
            }
            b"TipsBgIcon" => {
                self.tips_bg_icon = ctx
                    .decode_field(val, Self::TABLE_NAME, "TipsBgIcon")?;
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"Type2Icon" => {
                self.type2_icon = ctx.decode_field(val, Self::TABLE_NAME, "Type2Icon")?;
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
            b"_ContentText" => {
                self._content_text = ctx
                    .decode_field(val, Self::TABLE_NAME, "_ContentText")?;
            }
            b"_MainTitle" => {
                self._main_title = ctx.decode_field(val, Self::TABLE_NAME, "_MainTitle")?;
            }
            b"_SubTitle" => {
                self._sub_title = ctx.decode_field(val, Self::TABLE_NAME, "_SubTitle")?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigNewbieBuildsRow {
    const TABLE_NAME: &'static str = "config_newbie_builds";
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
/// Typed `DataTable` for `config_newbie_builds.out`.
pub type ConfigNewbieBuildsTable = DataTable<u32, ConfigNewbieBuildsRow>;
/// Parses `config_newbie_builds.out` into a [`ConfigNewbieBuildsTable`].
pub fn parse_config_newbie_builds(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigNewbieBuildsTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_newbie_builds_tips.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigNewbieBuildsTipsRow {
    pub content_text: Box<str>,
    pub title: Box<str>,
    pub _content_text: Box<str>,
    pub _title: Box<str>,
    pub id: i32,
}
impl DataCfgRow for ConfigNewbieBuildsTipsRow {
    const TABLE_NAME: &'static str = "config_newbie_builds_tips";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ContentText" => {
                self.content_text = ctx
                    .decode_field(val, Self::TABLE_NAME, "ContentText")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"_ContentText" => {
                self._content_text = ctx
                    .decode_field(val, Self::TABLE_NAME, "_ContentText")?;
            }
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_newbie_builds_tips.out`.
pub type ConfigNewbieBuildsTipsTable = DataTable<u32, ConfigNewbieBuildsTipsRow>;
/// Parses `config_newbie_builds_tips.out` into a [`ConfigNewbieBuildsTipsTable`].
pub fn parse_config_newbie_builds_tips(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigNewbieBuildsTipsTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_world_boss_challenge.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigWorldBossChallengeRow {
    pub boss_id: u32,
    pub level_id: u32,
    pub everyday_reward: u16,
    pub id: i16,
    pub step: u16,
    pub reward: u8,
    pub reward_season: u8,
}
impl DataCfgRow for ConfigWorldBossChallengeRow {
    const TABLE_NAME: &'static str = "config_world_boss_challenge";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BossId" => {
                self.boss_id = ctx.decode_field(val, Self::TABLE_NAME, "BossId")?;
            }
            b"EverydayReward" => {
                self.everyday_reward = ctx
                    .decode_field(val, Self::TABLE_NAME, "EverydayReward")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LevelId" => {
                self.level_id = ctx.decode_field(val, Self::TABLE_NAME, "LevelId")?;
            }
            b"Reward" => self.reward = ctx.decode_field(val, Self::TABLE_NAME, "Reward")?,
            b"RewardSeason" => {
                self.reward_season = ctx
                    .decode_field(val, Self::TABLE_NAME, "RewardSeason")?;
            }
            b"Step" => self.step = ctx.decode_field(val, Self::TABLE_NAME, "Step")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_world_boss_challenge.out`.
pub type ConfigWorldBossChallengeTable = DataTable<u8, ConfigWorldBossChallengeRow>;
/// Parses `config_world_boss_challenge.out` into a [`ConfigWorldBossChallengeTable`].
pub fn parse_config_world_boss_challenge(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigWorldBossChallengeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_world_boss_info.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigWorldBossInfoRow {
    pub boss_desc: Box<str>,
    pub boss_name: Box<str>,
    pub boss_pic: Box<str>,
    pub _boss_desc: Box<str>,
    pub _boss_name: Box<str>,
    pub id: i32,
}
impl DataCfgRow for ConfigWorldBossInfoRow {
    const TABLE_NAME: &'static str = "config_world_boss_info";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BossDesc" => {
                self.boss_desc = ctx.decode_field(val, Self::TABLE_NAME, "BossDesc")?;
            }
            b"BossName" => {
                self.boss_name = ctx.decode_field(val, Self::TABLE_NAME, "BossName")?;
            }
            b"BossPic" => {
                self.boss_pic = ctx.decode_field(val, Self::TABLE_NAME, "BossPic")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"_BossDesc" => {
                self._boss_desc = ctx.decode_field(val, Self::TABLE_NAME, "_BossDesc")?;
            }
            b"_BossName" => {
                self._boss_name = ctx.decode_field(val, Self::TABLE_NAME, "_BossName")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_world_boss_info.out`.
pub type ConfigWorldBossInfoTable = DataTable<u32, ConfigWorldBossInfoRow>;
/// Parses `config_world_boss_info.out` into a [`ConfigWorldBossInfoTable`].
pub fn parse_config_world_boss_info(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigWorldBossInfoTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_world_boss_skill.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigWorldBossSkillRow {
    pub skill_desc: Box<str>,
    pub skill_icon: Box<str>,
    pub skill_name: Box<str>,
    pub _skill_desc: Box<str>,
    pub _skill_name: Box<str>,
    pub boss_id: u32,
    pub id: i8,
}
impl DataCfgRow for ConfigWorldBossSkillRow {
    const TABLE_NAME: &'static str = "config_world_boss_skill";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BossId" => {
                self.boss_id = ctx.decode_field(val, Self::TABLE_NAME, "BossId")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"SkillDesc" => {
                self.skill_desc = ctx.decode_field(val, Self::TABLE_NAME, "SkillDesc")?;
            }
            b"SkillIcon" => {
                self.skill_icon = ctx.decode_field(val, Self::TABLE_NAME, "SkillIcon")?;
            }
            b"SkillName" => {
                self.skill_name = ctx.decode_field(val, Self::TABLE_NAME, "SkillName")?;
            }
            b"_SkillDesc" => {
                self._skill_desc = ctx.decode_field(val, Self::TABLE_NAME, "_SkillDesc")?;
            }
            b"_SkillName" => {
                self._skill_name = ctx.decode_field(val, Self::TABLE_NAME, "_SkillName")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_world_boss_skill.out`.
pub type ConfigWorldBossSkillTable = DataTable<u8, ConfigWorldBossSkillRow>;
/// Parses `config_world_boss_skill.out` into a [`ConfigWorldBossSkillTable`].
pub fn parse_config_world_boss_skill(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigWorldBossSkillTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
#[inline(never)]
fn verify_activity_chunk_0(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_accrual_recharge" => {
            Some(parse_config_accrual_recharge(buf, scratch).map(|t| t.len()))
        }
        "config_activity_form" => {
            Some(parse_config_activity_form(buf, scratch).map(|t| t.len()))
        }
        "config_activity_level_map" => {
            Some(parse_config_activity_level_map(buf, scratch).map(|t| t.len()))
        }
        "config_activity_socialmedia_link" => {
            Some(parse_config_activity_socialmedia_link(buf, scratch).map(|t| t.len()))
        }
        "config_appstorecard" => {
            Some(parse_config_appstorecard(buf, scratch).map(|t| t.len()))
        }
        "config_boss_challenge_detail" => {
            Some(parse_config_boss_challenge_detail(buf, scratch).map(|t| t.len()))
        }
        "config_functionpreview" => {
            Some(parse_config_functionpreview(buf, scratch).map(|t| t.len()))
        }
        "config_functionpreview_task" => {
            Some(parse_config_functionpreview_task(buf, scratch).map(|t| t.len()))
        }
        "config_growth_fund" => {
            Some(parse_config_growth_fund(buf, scratch).map(|t| t.len()))
        }
        "config_growth_fund_event" => {
            Some(parse_config_growth_fund_event(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_activity_chunk_1(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_levelupvoucher" => {
            Some(parse_config_levelupvoucher(buf, scratch).map(|t| t.len()))
        }
        "config_month_card" => {
            Some(parse_config_month_card(buf, scratch).map(|t| t.len()))
        }
        "config_newbie_build_list" => {
            Some(parse_config_newbie_build_list(buf, scratch).map(|t| t.len()))
        }
        "config_newbie_builds" => {
            Some(parse_config_newbie_builds(buf, scratch).map(|t| t.len()))
        }
        "config_newbie_builds_tips" => {
            Some(parse_config_newbie_builds_tips(buf, scratch).map(|t| t.len()))
        }
        "config_world_boss_challenge" => {
            Some(parse_config_world_boss_challenge(buf, scratch).map(|t| t.len()))
        }
        "config_world_boss_info" => {
            Some(parse_config_world_boss_info(buf, scratch).map(|t| t.len()))
        }
        "config_world_boss_skill" => {
            Some(parse_config_world_boss_skill(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
pub(crate) fn verify_activity_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    if let Some(res) = verify_activity_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_activity_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    None
}

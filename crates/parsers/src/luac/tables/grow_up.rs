//! Auto-generated `LuaTable/DataCfg` types and parsers for the `grow_up` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;

/// Entry for `config_gradenotice.NoticeCondition01`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GradenoticeNoticeCondition01 {
    pub condition_kind: Box<str>,
    pub task_id: u32,
}

impl FromEvalVal for GradenoticeNoticeCondition01 {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (condition_kind, task_id) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            condition_kind,
            task_id,
        })
    }
}

/// Entry for `config_gradenotice.NoticeCondition02`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GradenoticeNoticeCondition02 {
    pub condition_kind: Box<str>,
    pub task_id: u32,
}

impl FromEvalVal for GradenoticeNoticeCondition02 {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (condition_kind, task_id) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            condition_kind,
            task_id,
        })
    }
}

/// Row record for `config_gradenotice.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGradenoticeRow {
    pub apple_cn: Box<str>,
    pub id: i8,
    pub jump_url_apple: Box<str>,
    pub jump_url_google: Box<str>,
    pub jump_url_offical: Box<str>,
    pub jump_url_steam: Box<str>,
    pub jump_url_steam_cn: Box<str>,
    pub notice_condition01: GradenoticeNoticeCondition01,
    pub notice_condition02: GradenoticeNoticeCondition02,
    pub official_cn: Box<str>,
}

impl DataCfgRow for ConfigGradenoticeRow {
    const TABLE_NAME: &'static str = "config_gradenotice";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AppleCn" => self.apple_cn = ctx.decode_field(val, Self::TABLE_NAME, "AppleCn")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"JumpUrlApple" => {
                self.jump_url_apple = ctx.decode_field(val, Self::TABLE_NAME, "JumpUrlApple")?
            }
            b"JumpUrlGoogle" => {
                self.jump_url_google = ctx.decode_field(val, Self::TABLE_NAME, "JumpUrlGoogle")?
            }
            b"JumpUrlOffical" => {
                self.jump_url_offical = ctx.decode_field(val, Self::TABLE_NAME, "JumpUrlOffical")?
            }
            b"JumpUrlSteam" => {
                self.jump_url_steam = ctx.decode_field(val, Self::TABLE_NAME, "JumpUrlSteam")?
            }
            b"JumpUrlSteamCn" => {
                self.jump_url_steam_cn =
                    ctx.decode_field(val, Self::TABLE_NAME, "JumpUrlSteamCn")?
            }
            b"NoticeCondition01" => {
                self.notice_condition01 =
                    ctx.decode_field(val, Self::TABLE_NAME, "NoticeCondition01")?
            }
            b"NoticeCondition02" => {
                self.notice_condition02 =
                    ctx.decode_field(val, Self::TABLE_NAME, "NoticeCondition02")?
            }
            b"OfficialCn" => {
                self.official_cn = ctx.decode_field(val, Self::TABLE_NAME, "OfficialCn")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_gradenotice.out`.
pub type ConfigGradenoticeTable = DataTable<u8, ConfigGradenoticeRow>;

/// Parses `config_gradenotice.out` into a [`ConfigGradenoticeTable`].
pub fn parse_config_gradenotice(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGradenoticeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_grow_up_group.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGrowUpGroupRow {
    pub finish_reward_id: u32,
    pub first_finish_reward_id: u32,
    pub grow_up_task_ids: Box<[u32]>,
    pub id: i16,
    pub name: Box<str>,
    pub name_tip: Box<str>,
    pub order_id: u16,
    pub _name: Box<str>,
    pub _name_tip: Box<str>,
}

impl DataCfgRow for ConfigGrowUpGroupRow {
    const TABLE_NAME: &'static str = "config_grow_up_group";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"FinishRewardId" => {
                self.finish_reward_id = ctx.decode_field(val, Self::TABLE_NAME, "FinishRewardId")?
            }
            b"FirstFinishRewardId" => {
                self.first_finish_reward_id =
                    ctx.decode_field(val, Self::TABLE_NAME, "FirstFinishRewardId")?
            }
            b"GrowUpTaskIds" => {
                self.grow_up_task_ids = ctx.decode_field(val, Self::TABLE_NAME, "GrowUpTaskIds")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"NameTip" => self.name_tip = ctx.decode_field(val, Self::TABLE_NAME, "NameTip")?,
            b"OrderId" => self.order_id = ctx.decode_field(val, Self::TABLE_NAME, "OrderId")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_NameTip" => self._name_tip = ctx.decode_field(val, Self::TABLE_NAME, "_NameTip")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_grow_up_group.out`.
pub type ConfigGrowUpGroupTable = DataTable<u16, ConfigGrowUpGroupRow>;

/// Parses `config_grow_up_group.out` into a [`ConfigGrowUpGroupTable`].
pub fn parse_config_grow_up_group(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGrowUpGroupTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_grow_up_index.BannerImgPosition`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GrowUpIndexBannerImgPosition {
    pub x: i16,
    pub y: i8,
}

impl FromEvalVal for GrowUpIndexBannerImgPosition {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (x, y) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { x, y })
    }
}

/// Row record for `config_grow_up_index.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGrowUpIndexRow {
    pub banner_img: Box<str>,
    pub banner_img_position: GrowUpIndexBannerImgPosition,
    pub grow_up_group_ids: Box<[u16]>,
    pub icon_img: Box<str>,
    pub id: i8,
    pub locked_icon_img: Box<str>,
    pub name: Box<str>,
    pub order_id: u8,
    pub top_bg_img: Box<str>,
    pub unlock_event_id: u32,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigGrowUpIndexRow {
    const TABLE_NAME: &'static str = "config_grow_up_index";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BannerImg" => {
                self.banner_img = ctx.decode_field(val, Self::TABLE_NAME, "BannerImg")?
            }
            b"BannerImgPosition" => {
                self.banner_img_position =
                    ctx.decode_field(val, Self::TABLE_NAME, "BannerImgPosition")?
            }
            b"GrowUpGroupIds" => {
                self.grow_up_group_ids =
                    ctx.decode_field(val, Self::TABLE_NAME, "GrowUpGroupIds")?
            }
            b"IconImg" => self.icon_img = ctx.decode_field(val, Self::TABLE_NAME, "IconImg")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LockedIconImg" => {
                self.locked_icon_img = ctx.decode_field(val, Self::TABLE_NAME, "LockedIconImg")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"OrderId" => self.order_id = ctx.decode_field(val, Self::TABLE_NAME, "OrderId")?,
            b"TopBgImg" => self.top_bg_img = ctx.decode_field(val, Self::TABLE_NAME, "TopBgImg")?,
            b"UnlockEventId" => {
                self.unlock_event_id = ctx.decode_field(val, Self::TABLE_NAME, "UnlockEventId")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_grow_up_index.out`.
pub type ConfigGrowUpIndexTable = DataTable<u8, ConfigGrowUpIndexRow>;

/// Parses `config_grow_up_index.out` into a [`ConfigGrowUpIndexTable`].
pub fn parse_config_grow_up_index(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGrowUpIndexTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_grow_up_label.Unlock`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GrowUpLabelUnlock {
    pub unlock_kind: Box<str>,
    pub required_level: u8,
}

impl FromEvalVal for GrowUpLabelUnlock {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (unlock_kind, required_level) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            unlock_kind,
            required_level,
        })
    }
}

/// Row record for `config_grow_up_label.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGrowUpLabelRow {
    pub icon: Box<str>,
    pub id: i32,
    pub label: Box<str>,
    pub order: u8,
    pub parent_node: u8,
    pub select_icon: Box<str>,
    pub unlock: GrowUpLabelUnlock,
    pub _label: Box<str>,
}

impl DataCfgRow for ConfigGrowUpLabelRow {
    const TABLE_NAME: &'static str = "config_grow_up_label";

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
            b"Label" => self.label = ctx.decode_field(val, Self::TABLE_NAME, "Label")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"ParentNode" => {
                self.parent_node = ctx.decode_field(val, Self::TABLE_NAME, "ParentNode")?
            }
            b"SelectIcon" => {
                self.select_icon = ctx.decode_field(val, Self::TABLE_NAME, "SelectIcon")?
            }
            b"Unlock" => self.unlock = ctx.decode_field(val, Self::TABLE_NAME, "Unlock")?,
            b"_Label" => self._label = ctx.decode_field(val, Self::TABLE_NAME, "_Label")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_grow_up_label.out`.
pub type ConfigGrowUpLabelTable = DataTable<u32, ConfigGrowUpLabelRow>;

/// Parses `config_grow_up_label.out` into a [`ConfigGrowUpLabelTable`].
pub fn parse_config_grow_up_label(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGrowUpLabelTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_grow_up_modifier.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGrowUpModifierRow {
    pub icon_img: Box<str>,
    pub id: i8,
    pub locked_icon_img: Box<str>,
    pub map_modifier_ids: Box<[u32]>,
    pub order: u8,
}

impl DataCfgRow for ConfigGrowUpModifierRow {
    const TABLE_NAME: &'static str = "config_grow_up_modifier";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"IconImg" => self.icon_img = ctx.decode_field(val, Self::TABLE_NAME, "IconImg")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LockedIconImg" => {
                self.locked_icon_img = ctx.decode_field(val, Self::TABLE_NAME, "LockedIconImg")?
            }
            b"MapModifierIds" => {
                self.map_modifier_ids = ctx.decode_field(val, Self::TABLE_NAME, "MapModifierIds")?
            }
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_grow_up_modifier.out`.
pub type ConfigGrowUpModifierTable = DataTable<u8, ConfigGrowUpModifierRow>;

/// Parses `config_grow_up_modifier.out` into a [`ConfigGrowUpModifierTable`].
pub fn parse_config_grow_up_modifier(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGrowUpModifierTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_grow_up_task.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGrowUpTaskRow {
    pub event_id: u32,
    pub first_reward_id: Box<[u32]>,
    pub id: i32,
    pub is_statistics: bool,
    pub is_talent_task: bool,
    pub jump_string: Box<str>,
    pub reward_function_des: u32,
    pub reward_id: Box<[u32]>,
    pub task_type: u8,
}

impl DataCfgRow for ConfigGrowUpTaskRow {
    const TABLE_NAME: &'static str = "config_grow_up_task";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"EventId" => self.event_id = ctx.decode_field(val, Self::TABLE_NAME, "EventId")?,
            b"FirstRewardId" => {
                self.first_reward_id = ctx.decode_field(val, Self::TABLE_NAME, "FirstRewardId")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsStatistics" => {
                self.is_statistics = ctx.decode_field(val, Self::TABLE_NAME, "IsStatistics")?
            }
            b"IsTalentTask" => {
                self.is_talent_task = ctx.decode_field(val, Self::TABLE_NAME, "IsTalentTask")?
            }
            b"JumpString" => {
                self.jump_string = ctx.decode_field(val, Self::TABLE_NAME, "JumpString")?
            }
            b"RewardFunctionDes" => {
                self.reward_function_des =
                    ctx.decode_field(val, Self::TABLE_NAME, "RewardFunctionDes")?
            }
            b"RewardId" => self.reward_id = ctx.decode_field(val, Self::TABLE_NAME, "RewardId")?,
            b"TaskType" => self.task_type = ctx.decode_field(val, Self::TABLE_NAME, "TaskType")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_grow_up_task.out`.
pub type ConfigGrowUpTaskTable = DataTable<u32, ConfigGrowUpTaskRow>;

/// Parses `config_grow_up_task.out` into a [`ConfigGrowUpTaskTable`].
pub fn parse_config_grow_up_task(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGrowUpTaskTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_guide.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuideRow {
    pub args: Box<str>,
    pub args_ui: Box<str>,
    pub close_ui: u8,
    pub des: Box<str>,
    pub des_gamepad: Box<str>,
    pub des_pc: Box<str>,
    pub guide_frame_type: Box<str>,
    pub guide_hand_type: Box<str>,
    pub id: i16,
    pub initial_steps: Box<str>,
    pub is_force_guide: u8,
    pub key_steps: u8,
    pub level_id: Box<str>,
    pub margin: Box<[u8]>,
    pub message_position_type: Box<str>,
    pub next_id: i16,
    pub next_id_right: i16,
    pub pause: u8,
    pub precondition: u8,
    pub reminder_id: i8,
    pub response_id: Box<[u8]>,
    pub show_full_guide_btn: u8,
    pub skip: u8,
    pub r#type: u8,
    pub widget_params: Box<str>,
    pub _des: Box<str>,
    pub _des_gamepad: Box<str>,
    pub _des_pc: Box<str>,
}

impl ConfigGuideRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Args" => self.args = ctx.decode_field(val, Self::TABLE_NAME, "Args")?,
            b"ArgsUi" => self.args_ui = ctx.decode_field(val, Self::TABLE_NAME, "ArgsUi")?,
            b"CloseUi" => self.close_ui = ctx.decode_field(val, Self::TABLE_NAME, "CloseUi")?,
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"DesGamepad" => {
                self.des_gamepad = ctx.decode_field(val, Self::TABLE_NAME, "DesGamepad")?
            }
            b"DesPc" => self.des_pc = ctx.decode_field(val, Self::TABLE_NAME, "DesPc")?,
            b"GuideFrameType" => {
                self.guide_frame_type = ctx.decode_field(val, Self::TABLE_NAME, "GuideFrameType")?
            }
            b"GuideHandType" => {
                self.guide_hand_type = ctx.decode_field(val, Self::TABLE_NAME, "GuideHandType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InitialSteps" => {
                self.initial_steps = ctx.decode_field(val, Self::TABLE_NAME, "InitialSteps")?
            }
            b"IsForceGuide" => {
                self.is_force_guide = ctx.decode_field(val, Self::TABLE_NAME, "IsForceGuide")?
            }
            b"KeySteps" => self.key_steps = ctx.decode_field(val, Self::TABLE_NAME, "KeySteps")?,
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
            b"LevelId" => self.level_id = ctx.decode_field(val, Self::TABLE_NAME, "LevelId")?,
            b"Margin" => self.margin = ctx.decode_field(val, Self::TABLE_NAME, "Margin")?,
            b"MessagePositionType" => {
                self.message_position_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "MessagePositionType")?
            }
            b"NextId" => self.next_id = ctx.decode_field(val, Self::TABLE_NAME, "NextId")?,
            b"NextIdRight" => {
                self.next_id_right = ctx.decode_field(val, Self::TABLE_NAME, "NextIdRight")?
            }
            b"Pause" => self.pause = ctx.decode_field(val, Self::TABLE_NAME, "Pause")?,
            b"Precondition" => {
                self.precondition = ctx.decode_field(val, Self::TABLE_NAME, "Precondition")?
            }
            b"ReminderId" => {
                self.reminder_id = ctx.decode_field(val, Self::TABLE_NAME, "ReminderId")?
            }
            b"ResponseId" => {
                self.response_id = ctx.decode_field(val, Self::TABLE_NAME, "ResponseId")?
            }
            b"ShowFullGuideBtn" => {
                self.show_full_guide_btn =
                    ctx.decode_field(val, Self::TABLE_NAME, "ShowFullGuideBtn")?
            }
            b"Skip" => self.skip = ctx.decode_field(val, Self::TABLE_NAME, "Skip")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    #[inline(never)]
    fn set_field_2(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"WidgetParams" => {
                self.widget_params = ctx.decode_field(val, Self::TABLE_NAME, "WidgetParams")?
            }
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            b"_DesGamepad" => {
                self._des_gamepad = ctx.decode_field(val, Self::TABLE_NAME, "_DesGamepad")?
            }
            b"_DesPc" => self._des_pc = ctx.decode_field(val, Self::TABLE_NAME, "_DesPc")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigGuideRow {
    const TABLE_NAME: &'static str = "config_guide";

    #[inline(never)]
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

/// Typed `DataTable` for `config_guide.out`.
pub type ConfigGuideTable = DataTable<u16, ConfigGuideRow>;

/// Parses `config_guide.out` into a [`ConfigGuideTable`].
pub fn parse_config_guide(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGuideTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

pub(crate) fn verify_grow_up_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_gradenotice" => Some(parse_config_gradenotice(buf, scratch).map(|t| t.len())),
        "config_grow_up_group" => Some(parse_config_grow_up_group(buf, scratch).map(|t| t.len())),
        "config_grow_up_index" => Some(parse_config_grow_up_index(buf, scratch).map(|t| t.len())),
        "config_grow_up_label" => Some(parse_config_grow_up_label(buf, scratch).map(|t| t.len())),
        "config_grow_up_modifier" => {
            Some(parse_config_grow_up_modifier(buf, scratch).map(|t| t.len()))
        }
        "config_grow_up_task" => Some(parse_config_grow_up_task(buf, scratch).map(|t| t.len())),
        "config_guide" => Some(parse_config_guide(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

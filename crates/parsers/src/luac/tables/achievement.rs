//! Auto-generated `LuaTable/DataCfg` types and parsers for the `achievement` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;

/// Row record for `config_achievement.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAchievementRow {
    pub args: Box<str>,
    pub condition: u8,
    pub desc: Box<str>,
    pub desc_linshi: Box<str>,
    pub end_time: u32,
    pub event_id: u32,
    pub go_to: Box<str>,
    pub group_id: u16,
    pub grow_up_crux_node: u8,
    pub handbook_id: u32,
    pub icon: Box<str>,
    pub id: i32,
    pub is_daily_task: u8,
    pub is_show: u8,
    pub manual_id: Box<[u8]>,
    pub next_id: u32,
    pub open_time: u32,
    pub parent_node: Box<[u32]>,
    pub point: u8,
    pub position: Box<str>,
    pub progress: Box<str>,
    pub select_icon: Box<str>,
    pub statistics: Box<str>,
    pub tab: Box<str>,
    pub title: Box<str>,
    pub r#type: u8,
    pub _desc: Box<str>,
    pub _desc_linshi: Box<str>,
    pub _title: Box<str>,
}

impl ConfigAchievementRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Args" => self.args = ctx.decode_field(val, Self::TABLE_NAME, "Args")?,
            b"Condition" => {
                self.condition = ctx.decode_field(val, Self::TABLE_NAME, "Condition")?
            }
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DescLinshi" => {
                self.desc_linshi = ctx.decode_field(val, Self::TABLE_NAME, "DescLinshi")?
            }
            b"EndTime" => self.end_time = ctx.decode_field(val, Self::TABLE_NAME, "EndTime")?,
            b"EventId" => self.event_id = ctx.decode_field(val, Self::TABLE_NAME, "EventId")?,
            b"GoTo" => self.go_to = ctx.decode_field(val, Self::TABLE_NAME, "GoTo")?,
            b"GroupId" => self.group_id = ctx.decode_field(val, Self::TABLE_NAME, "GroupId")?,
            b"GrowUpCruxNode" => {
                self.grow_up_crux_node =
                    ctx.decode_field(val, Self::TABLE_NAME, "GrowUpCruxNode")?
            }
            b"HandbookId" => {
                self.handbook_id = ctx.decode_field(val, Self::TABLE_NAME, "HandbookId")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
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
            b"IsDailyTask" => {
                self.is_daily_task = ctx.decode_field(val, Self::TABLE_NAME, "IsDailyTask")?
            }
            b"IsShow" => self.is_show = ctx.decode_field(val, Self::TABLE_NAME, "IsShow")?,
            b"ManualId" => self.manual_id = ctx.decode_field(val, Self::TABLE_NAME, "ManualId")?,
            b"NextId" => self.next_id = ctx.decode_field(val, Self::TABLE_NAME, "NextId")?,
            b"OpenTime" => self.open_time = ctx.decode_field(val, Self::TABLE_NAME, "OpenTime")?,
            b"ParentNode" => {
                self.parent_node = ctx.decode_field(val, Self::TABLE_NAME, "ParentNode")?
            }
            b"Point" => self.point = ctx.decode_field(val, Self::TABLE_NAME, "Point")?,
            b"Position" => self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?,
            b"Progress" => self.progress = ctx.decode_field(val, Self::TABLE_NAME, "Progress")?,
            b"SelectIcon" => {
                self.select_icon = ctx.decode_field(val, Self::TABLE_NAME, "SelectIcon")?
            }
            b"Statistics" => {
                self.statistics = ctx.decode_field(val, Self::TABLE_NAME, "Statistics")?
            }
            b"Tab" => self.tab = ctx.decode_field(val, Self::TABLE_NAME, "Tab")?,
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
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_DescLinshi" => {
                self._desc_linshi = ctx.decode_field(val, Self::TABLE_NAME, "_DescLinshi")?
            }
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigAchievementRow {
    const TABLE_NAME: &'static str = "config_achievement";

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

/// Typed `DataTable` for `config_achievement.out`.
pub type ConfigAchievementTable = DataTable<u32, ConfigAchievementRow>;

/// Parses `config_achievement.out` into a [`ConfigAchievementTable`].
pub fn parse_config_achievement(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAchievementTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_achievement_label.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAchievementLabelRow {
    pub icon: Box<str>,
    pub icon_chose: Box<str>,
    pub id: i32,
    pub label: Box<str>,
    pub order: u8,
    pub parent_node: u32,
    pub _label: Box<str>,
}

impl DataCfgRow for ConfigAchievementLabelRow {
    const TABLE_NAME: &'static str = "config_achievement_label";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"IconChose" => {
                self.icon_chose = ctx.decode_field(val, Self::TABLE_NAME, "IconChose")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Label" => self.label = ctx.decode_field(val, Self::TABLE_NAME, "Label")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"ParentNode" => {
                self.parent_node = ctx.decode_field(val, Self::TABLE_NAME, "ParentNode")?
            }
            b"_Label" => self._label = ctx.decode_field(val, Self::TABLE_NAME, "_Label")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_achievement_label.out`.
pub type ConfigAchievementLabelTable = DataTable<u32, ConfigAchievementLabelRow>;

/// Parses `config_achievement_label.out` into a [`ConfigAchievementLabelTable`].
pub fn parse_config_achievement_label(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAchievementLabelTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_collection.Event`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CollectionEvent {
    pub kind: u8,
    pub params: Box<[u32]>,
}

impl FromEvalVal for CollectionEvent {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (kind, params) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { kind, params })
    }
}

/// Row record for `config_collection.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCollectionRow {
    pub event: CollectionEvent,
    pub id: i16,
    pub locked_text: Box<str>,
    pub main_story_id: u32,
    pub name: Box<str>,
    pub sort: u16,
    pub sub_name: Box<str>,
    pub type1: u8,
    pub type2: u8,
    pub unlocked_text: Box<str>,
    pub _locked_text: Box<str>,
    pub _name: Box<str>,
    pub _sub_name: Box<str>,
    pub _unlocked_text: Box<str>,
}

impl ConfigCollectionRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Event" => self.event = ctx.decode_field(val, Self::TABLE_NAME, "Event")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LockedText" => {
                self.locked_text = ctx.decode_field(val, Self::TABLE_NAME, "LockedText")?
            }
            b"MainStoryId" => {
                self.main_story_id = ctx.decode_field(val, Self::TABLE_NAME, "MainStoryId")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"SubName" => self.sub_name = ctx.decode_field(val, Self::TABLE_NAME, "SubName")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"UnlockedText" => {
                self.unlocked_text = ctx.decode_field(val, Self::TABLE_NAME, "UnlockedText")?
            }
            b"_LockedText" => {
                self._locked_text = ctx.decode_field(val, Self::TABLE_NAME, "_LockedText")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
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
            b"_SubName" => self._sub_name = ctx.decode_field(val, Self::TABLE_NAME, "_SubName")?,
            b"_UnlockedText" => {
                self._unlocked_text = ctx.decode_field(val, Self::TABLE_NAME, "_UnlockedText")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigCollectionRow {
    const TABLE_NAME: &'static str = "config_collection";

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

/// Typed `DataTable` for `config_collection.out`.
pub type ConfigCollectionTable = DataTable<u16, ConfigCollectionRow>;

/// Parses `config_collection.out` into a [`ConfigCollectionTable`].
pub fn parse_config_collection(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCollectionTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_collection_type1.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCollectionType1Row {
    pub icon: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub sort: u8,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigCollectionType1Row {
    const TABLE_NAME: &'static str = "config_collection_type1";

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
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_collection_type1.out`.
pub type ConfigCollectionType1Table = DataTable<u8, ConfigCollectionType1Row>;

/// Parses `config_collection_type1.out` into a [`ConfigCollectionType1Table`].
pub fn parse_config_collection_type1(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCollectionType1Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_collection_type2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCollectionType2Row {
    pub icon: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub sort: u8,
    pub type1: u8,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigCollectionType2Row {
    const TABLE_NAME: &'static str = "config_collection_type2";

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
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_collection_type2.out`.
pub type ConfigCollectionType2Table = DataTable<u8, ConfigCollectionType2Row>;

/// Parses `config_collection_type2.out` into a [`ConfigCollectionType2Table`].
pub fn parse_config_collection_type2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCollectionType2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_fight_title.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFightTitleRow {
    pub bpname: Box<str>,
    pub desc1: Box<str>,
    pub desc2: Box<str>,
    pub desc3: Box<str>,
    pub id: i8,
}

impl DataCfgRow for ConfigFightTitleRow {
    const TABLE_NAME: &'static str = "config_fight_title";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Bpname" => self.bpname = ctx.decode_field(val, Self::TABLE_NAME, "Bpname")?,
            b"Desc1" => self.desc1 = ctx.decode_field(val, Self::TABLE_NAME, "Desc1")?,
            b"Desc2" => self.desc2 = ctx.decode_field(val, Self::TABLE_NAME, "Desc2")?,
            b"Desc3" => self.desc3 = ctx.decode_field(val, Self::TABLE_NAME, "Desc3")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_fight_title.out`.
pub type ConfigFightTitleTable = DataTable<u8, ConfigFightTitleRow>;

/// Parses `config_fight_title.out` into a [`ConfigFightTitleTable`].
pub fn parse_config_fight_title(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigFightTitleTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_handbook.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigHandbookRow {
    pub client_judge: u8,
    pub condition: Box<str>,
    pub description: Box<str>,
    pub effect: u8,
    pub id: i32,
    pub name: Box<str>,
    pub order: u8,
    pub pic: Box<str>,
    pub _description: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigHandbookRow {
    const TABLE_NAME: &'static str = "config_handbook";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ClientJudge" => {
                self.client_judge = ctx.decode_field(val, Self::TABLE_NAME, "ClientJudge")?
            }
            b"Condition" => {
                self.condition = ctx.decode_field(val, Self::TABLE_NAME, "Condition")?
            }
            b"Description" => {
                self.description = ctx.decode_field(val, Self::TABLE_NAME, "Description")?
            }
            b"Effect" => self.effect = ctx.decode_field(val, Self::TABLE_NAME, "Effect")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"Pic" => self.pic = ctx.decode_field(val, Self::TABLE_NAME, "Pic")?,
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_handbook.out`.
pub type ConfigHandbookTable = DataTable<u32, ConfigHandbookRow>;

/// Parses `config_handbook.out` into a [`ConfigHandbookTable`].
pub fn parse_config_handbook(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigHandbookTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_manual_entrance.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigManualEntranceRow {
    pub group_id: u8,
    pub group_name: Box<str>,
    pub icon: Box<str>,
    pub id: i16,
    pub item_type_list_id: Box<[u8]>,
    pub order: u16,
    pub page_type: u8,
    pub page_type_name: Box<str>,
    pub param: Box<str>,
    pub _group_name: Box<str>,
    pub _page_type_name: Box<str>,
}

impl DataCfgRow for ConfigManualEntranceRow {
    const TABLE_NAME: &'static str = "config_manual_entrance";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"GroupId" => self.group_id = ctx.decode_field(val, Self::TABLE_NAME, "GroupId")?,
            b"GroupName" => {
                self.group_name = ctx.decode_field(val, Self::TABLE_NAME, "GroupName")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemTypeListId" => {
                self.item_type_list_id =
                    ctx.decode_field(val, Self::TABLE_NAME, "ItemTypeListId")?
            }
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"PageType" => self.page_type = ctx.decode_field(val, Self::TABLE_NAME, "PageType")?,
            b"PageTypeName" => {
                self.page_type_name = ctx.decode_field(val, Self::TABLE_NAME, "PageTypeName")?
            }
            b"Param" => self.param = ctx.decode_field(val, Self::TABLE_NAME, "Param")?,
            b"_GroupName" => {
                self._group_name = ctx.decode_field(val, Self::TABLE_NAME, "_GroupName")?
            }
            b"_PageTypeName" => {
                self._page_type_name = ctx.decode_field(val, Self::TABLE_NAME, "_PageTypeName")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_manual_entrance.out`.
pub type ConfigManualEntranceTable = DataTable<u16, ConfigManualEntranceRow>;

/// Parses `config_manual_entrance.out` into a [`ConfigManualEntranceTable`].
pub fn parse_config_manual_entrance(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigManualEntranceTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_manual_rule.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigManualRuleRow {
    pub brief_group_id: u32,
    pub detail_group_id: u32,
    pub id: i32,
    pub name: Box<str>,
    pub order: u8,
    pub rule_type: u32,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigManualRuleRow {
    const TABLE_NAME: &'static str = "config_manual_rule";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BriefGroupId" => {
                self.brief_group_id = ctx.decode_field(val, Self::TABLE_NAME, "BriefGroupId")?
            }
            b"DetailGroupId" => {
                self.detail_group_id = ctx.decode_field(val, Self::TABLE_NAME, "DetailGroupId")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"RuleType" => self.rule_type = ctx.decode_field(val, Self::TABLE_NAME, "RuleType")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_manual_rule.out`.
pub type ConfigManualRuleTable = DataTable<u32, ConfigManualRuleRow>;

/// Parses `config_manual_rule.out` into a [`ConfigManualRuleTable`].
pub fn parse_config_manual_rule(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigManualRuleTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_manual_rule_des.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigManualRuleDesRow {
    pub des: Box<str>,
    pub group_id: u32,
    pub id: i64,
    pub order: u8,
    pub param: Box<[Box<str>]>,
    pub _des: Box<str>,
}

impl DataCfgRow for ConfigManualRuleDesRow {
    const TABLE_NAME: &'static str = "config_manual_rule_des";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"GroupId" => self.group_id = ctx.decode_field(val, Self::TABLE_NAME, "GroupId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"Param" => self.param = ctx.decode_field(val, Self::TABLE_NAME, "Param")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_manual_rule_des.out`.
pub type ConfigManualRuleDesTable = DataTable<u64, ConfigManualRuleDesRow>;

/// Parses `config_manual_rule_des.out` into a [`ConfigManualRuleDesTable`].
pub fn parse_config_manual_rule_des(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigManualRuleDesTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_manual_ruledes_filter.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigManualRuledesFilterRow {
    pub des: Box<str>,
    pub id: i32,
    pub order: u8,
    pub root_id: u16,
    pub rule_type: u32,
    pub _des: Box<str>,
}

impl DataCfgRow for ConfigManualRuledesFilterRow {
    const TABLE_NAME: &'static str = "config_manual_ruledes_filter";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"RootId" => self.root_id = ctx.decode_field(val, Self::TABLE_NAME, "RootId")?,
            b"RuleType" => self.rule_type = ctx.decode_field(val, Self::TABLE_NAME, "RuleType")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_manual_ruledes_filter.out`.
pub type ConfigManualRuledesFilterTable = DataTable<u32, ConfigManualRuledesFilterRow>;

/// Parses `config_manual_ruledes_filter.out` into a [`ConfigManualRuledesFilterTable`].
pub fn parse_config_manual_ruledes_filter(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigManualRuledesFilterTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_strategy_by_language.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigStrategyByLanguageRow {
    pub area_id: Box<[u8]>,
    pub area_name: Box<str>,
    pub language_code: Box<str>,
    pub strategy_id: Box<[u8]>,
    pub _area_name: Box<str>,
}

impl DataCfgRow for ConfigStrategyByLanguageRow {
    const TABLE_NAME: &'static str = "config_strategy_by_language";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AreaId" => self.area_id = ctx.decode_field(val, Self::TABLE_NAME, "AreaId")?,
            b"AreaName" => self.area_name = ctx.decode_field(val, Self::TABLE_NAME, "AreaName")?,
            b"LanguageCode" => {
                self.language_code = ctx.decode_field(val, Self::TABLE_NAME, "LanguageCode")?
            }
            b"StrategyId" => {
                self.strategy_id = ctx.decode_field(val, Self::TABLE_NAME, "StrategyId")?
            }
            b"_AreaName" => {
                self._area_name = ctx.decode_field(val, Self::TABLE_NAME, "_AreaName")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_strategy_by_language.out`.
pub type ConfigStrategyByLanguageTable = DataTable<Box<str>, ConfigStrategyByLanguageRow>;

/// Parses `config_strategy_by_language.out` into a [`ConfigStrategyByLanguageTable`].
pub fn parse_config_strategy_by_language(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigStrategyByLanguageTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_strategy_content_tab.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigStrategyContentTabRow {
    pub id: i8,
    pub sub_node_id: Box<[u8]>,
    pub title: Box<str>,
    pub _title: Box<str>,
}

impl DataCfgRow for ConfigStrategyContentTabRow {
    const TABLE_NAME: &'static str = "config_strategy_content_tab";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"SubNodeId" => {
                self.sub_node_id = ctx.decode_field(val, Self::TABLE_NAME, "SubNodeId")?
            }
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_strategy_content_tab.out`.
pub type ConfigStrategyContentTabTable = DataTable<u8, ConfigStrategyContentTabRow>;

/// Parses `config_strategy_content_tab.out` into a [`ConfigStrategyContentTabTable`].
pub fn parse_config_strategy_content_tab(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigStrategyContentTabTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

pub(crate) fn verify_achievement_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_achievement" => Some(parse_config_achievement(buf, scratch).map(|t| t.len())),
        "config_achievement_label" => {
            Some(parse_config_achievement_label(buf, scratch).map(|t| t.len()))
        }
        "config_collection" => Some(parse_config_collection(buf, scratch).map(|t| t.len())),
        "config_collection_type1" => {
            Some(parse_config_collection_type1(buf, scratch).map(|t| t.len()))
        }
        "config_collection_type2" => {
            Some(parse_config_collection_type2(buf, scratch).map(|t| t.len()))
        }
        "config_fight_title" => Some(parse_config_fight_title(buf, scratch).map(|t| t.len())),
        "config_handbook" => Some(parse_config_handbook(buf, scratch).map(|t| t.len())),
        "config_manual_entrance" => {
            Some(parse_config_manual_entrance(buf, scratch).map(|t| t.len()))
        }
        "config_manual_rule" => Some(parse_config_manual_rule(buf, scratch).map(|t| t.len())),
        "config_manual_rule_des" => {
            Some(parse_config_manual_rule_des(buf, scratch).map(|t| t.len()))
        }
        "config_manual_ruledes_filter" => {
            Some(parse_config_manual_ruledes_filter(buf, scratch).map(|t| t.len()))
        }
        "config_strategy_by_language" => {
            Some(parse_config_strategy_by_language(buf, scratch).map(|t| t.len()))
        }
        "config_strategy_content_tab" => {
            Some(parse_config_strategy_content_tab(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

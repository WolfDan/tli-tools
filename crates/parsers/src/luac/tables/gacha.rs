//! Auto-generated `LuaTable/DataCfg` types and parsers for the `gacha` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;

/// Entry for `config_gacha.PoolRarityWeight`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GachaPoolRarityWeightEntry {
    pub rarity: u8,
    pub weight: f64,
}

impl FromEvalVal for GachaPoolRarityWeightEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (rarity, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { rarity, weight })
    }
}

/// Entry for `config_gacha.UpBanner`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GachaUpBannerEntry {
    pub reward_kind: Box<str>,
    pub id: u32,
}

impl FromEvalVal for GachaUpBannerEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (reward_kind, id) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { reward_kind, id })
    }
}

/// Entry for `config_gacha.UpPossibilityInformation`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GachaUpPossibilityInformationEntry {
    pub reward_kind: Box<str>,
    pub id: u16,
}

impl FromEvalVal for GachaUpPossibilityInformationEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (reward_kind, id) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { reward_kind, id })
    }
}

/// Row record for `config_gacha.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigGachaRow {
    pub bonus: Box<str>,
    pub cheat_count: u8,
    pub cheat_pool: u8,
    pub cost: Box<str>,
    pub cost10: Box<str>,
    pub des: Box<str>,
    pub description_pick_record: Box<str>,
    pub detail_info_rate_up_desc: Box<str>,
    pub detail_info_rate_up_title: Box<str>,
    pub discount_money_id: u16,
    pub ensure: u8,
    pub ensure_parm: Box<str>,
    pub finish_condition: Box<str>,
    pub free_count: u8,
    pub gacha_count_reward: FxHashMap<u8, u32>,
    pub gacha_limit: Box<[u8]>,
    pub gacha_money_id: Box<[u16]>,
    pub gacha_type: u8,
    pub id: i32,
    pub is_return: u8,
    pub item1_weight_ratio: Box<str>,
    pub item2_weight_ratio: Box<str>,
    pub item3_weight_ratio: Box<str>,
    pub item4_weight_ratio: Box<str>,
    pub name: Box<str>,
    pub no_repeat: u8,
    pub no_repeat_parm: Box<[u32]>,
    pub open: u8,
    pub optional_parm: Box<[i8]>,
    pub optional_up_label: Box<[u8]>,
    pub pool_rarity_weight: Box<[GachaPoolRarityWeightEntry]>,
    pub prd: u8,
    pub prd_count: u8,
    pub prd_parm: Box<str>,
    pub rarity_weight: Box<str>,
    pub record_type: u16,
    pub refresh_rule: Box<str>,
    pub reward_group: Box<str>,
    pub reward_time: u8,
    pub servent_select: Box<[u16]>,
    pub time_over: Box<str>,
    pub time_start: Box<str>,
    pub time_zone: Box<str>,
    pub r#type: u8,
    pub unlock_level: u8,
    pub unlock_task: u32,
    pub unlock_time: u8,
    pub up_banner: Box<[GachaUpBannerEntry]>,
    pub up_open: u8,
    pub up_possibility_information: Box<[GachaUpPossibilityInformationEntry]>,
    pub up_rarity: u8,
    pub up_weight: f64,
    pub _des: Box<str>,
    pub _description_pick_record: Box<str>,
    pub _detail_info_rate_up_desc: Box<str>,
    pub _detail_info_rate_up_title: Box<str>,
    pub _name: Box<str>,
}

impl ConfigGachaRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Bonus" => self.bonus = ctx.decode_field(val, Self::TABLE_NAME, "Bonus")?,
            b"CheatCount" => {
                self.cheat_count = ctx.decode_field(val, Self::TABLE_NAME, "CheatCount")?
            }
            b"CheatPool" => {
                self.cheat_pool = ctx.decode_field(val, Self::TABLE_NAME, "CheatPool")?
            }
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"Cost10" => self.cost10 = ctx.decode_field(val, Self::TABLE_NAME, "Cost10")?,
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"DescriptionPickRecord" => {
                self.description_pick_record =
                    ctx.decode_field(val, Self::TABLE_NAME, "DescriptionPickRecord")?
            }
            b"DetailInfoRateUpDesc" => {
                self.detail_info_rate_up_desc =
                    ctx.decode_field(val, Self::TABLE_NAME, "DetailInfoRateUpDesc")?
            }
            b"DetailInfoRateUpTitle" => {
                self.detail_info_rate_up_title =
                    ctx.decode_field(val, Self::TABLE_NAME, "DetailInfoRateUpTitle")?
            }
            b"DiscountMoneyId" => {
                self.discount_money_id =
                    ctx.decode_field(val, Self::TABLE_NAME, "DiscountMoneyId")?
            }
            b"Ensure" => self.ensure = ctx.decode_field(val, Self::TABLE_NAME, "Ensure")?,
            b"EnsureParm" => {
                self.ensure_parm = ctx.decode_field(val, Self::TABLE_NAME, "EnsureParm")?
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
            b"FinishCondition" => {
                self.finish_condition =
                    ctx.decode_field(val, Self::TABLE_NAME, "FinishCondition")?
            }
            b"FreeCount" => {
                self.free_count = ctx.decode_field(val, Self::TABLE_NAME, "FreeCount")?
            }
            b"GachaCountReward" => {
                self.gacha_count_reward =
                    ctx.decode_field(val, Self::TABLE_NAME, "GachaCountReward")?
            }
            b"GachaLimit" => {
                self.gacha_limit = ctx.decode_field(val, Self::TABLE_NAME, "GachaLimit")?
            }
            b"GachaMoneyId" => {
                self.gacha_money_id = ctx.decode_field(val, Self::TABLE_NAME, "GachaMoneyId")?
            }
            b"GachaType" => {
                self.gacha_type = ctx.decode_field(val, Self::TABLE_NAME, "GachaType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsReturn" => self.is_return = ctx.decode_field(val, Self::TABLE_NAME, "IsReturn")?,
            b"Item1WeightRatio" => {
                self.item1_weight_ratio =
                    ctx.decode_field(val, Self::TABLE_NAME, "Item1WeightRatio")?
            }
            b"Item2WeightRatio" => {
                self.item2_weight_ratio =
                    ctx.decode_field(val, Self::TABLE_NAME, "Item2WeightRatio")?
            }
            b"Item3WeightRatio" => {
                self.item3_weight_ratio =
                    ctx.decode_field(val, Self::TABLE_NAME, "Item3WeightRatio")?
            }
            b"Item4WeightRatio" => {
                self.item4_weight_ratio =
                    ctx.decode_field(val, Self::TABLE_NAME, "Item4WeightRatio")?
            }
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
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"NoRepeat" => self.no_repeat = ctx.decode_field(val, Self::TABLE_NAME, "NoRepeat")?,
            b"NoRepeatParm" => {
                self.no_repeat_parm = ctx.decode_field(val, Self::TABLE_NAME, "NoRepeatParm")?
            }
            b"Open" => self.open = ctx.decode_field(val, Self::TABLE_NAME, "Open")?,
            b"OptionalParm" => {
                self.optional_parm = ctx.decode_field(val, Self::TABLE_NAME, "OptionalParm")?
            }
            b"OptionalUpLabel" => {
                self.optional_up_label =
                    ctx.decode_field(val, Self::TABLE_NAME, "OptionalUpLabel")?
            }
            b"PoolRarityWeight" => {
                self.pool_rarity_weight =
                    ctx.decode_field(val, Self::TABLE_NAME, "PoolRarityWeight")?
            }
            b"Prd" => self.prd = ctx.decode_field(val, Self::TABLE_NAME, "Prd")?,
            b"PrdCount" => self.prd_count = ctx.decode_field(val, Self::TABLE_NAME, "PrdCount")?,
            b"PrdParm" => self.prd_parm = ctx.decode_field(val, Self::TABLE_NAME, "PrdParm")?,
            b"RarityWeight" => {
                self.rarity_weight = ctx.decode_field(val, Self::TABLE_NAME, "RarityWeight")?
            }
            b"RecordType" => {
                self.record_type = ctx.decode_field(val, Self::TABLE_NAME, "RecordType")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    #[inline(never)]
    fn set_field_3(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"RefreshRule" => {
                self.refresh_rule = ctx.decode_field(val, Self::TABLE_NAME, "RefreshRule")?
            }
            b"RewardGroup" => {
                self.reward_group = ctx.decode_field(val, Self::TABLE_NAME, "RewardGroup")?
            }
            b"RewardTime" => {
                self.reward_time = ctx.decode_field(val, Self::TABLE_NAME, "RewardTime")?
            }
            b"ServentSelect" => {
                self.servent_select = ctx.decode_field(val, Self::TABLE_NAME, "ServentSelect")?
            }
            b"TimeOver" => self.time_over = ctx.decode_field(val, Self::TABLE_NAME, "TimeOver")?,
            b"TimeStart" => {
                self.time_start = ctx.decode_field(val, Self::TABLE_NAME, "TimeStart")?
            }
            b"TimeZone" => self.time_zone = ctx.decode_field(val, Self::TABLE_NAME, "TimeZone")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"UnlockLevel" => {
                self.unlock_level = ctx.decode_field(val, Self::TABLE_NAME, "UnlockLevel")?
            }
            b"UnlockTask" => {
                self.unlock_task = ctx.decode_field(val, Self::TABLE_NAME, "UnlockTask")?
            }
            b"UnlockTime" => {
                self.unlock_time = ctx.decode_field(val, Self::TABLE_NAME, "UnlockTime")?
            }
            b"UpBanner" => self.up_banner = ctx.decode_field(val, Self::TABLE_NAME, "UpBanner")?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    #[inline(never)]
    fn set_field_4(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"UpOpen" => self.up_open = ctx.decode_field(val, Self::TABLE_NAME, "UpOpen")?,
            b"UpPossibilityInformation" => {
                self.up_possibility_information =
                    ctx.decode_field(val, Self::TABLE_NAME, "UpPossibilityInformation")?
            }
            b"UpRarity" => self.up_rarity = ctx.decode_field(val, Self::TABLE_NAME, "UpRarity")?,
            b"UpWeight" => self.up_weight = ctx.decode_field(val, Self::TABLE_NAME, "UpWeight")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            b"_DescriptionPickRecord" => {
                self._description_pick_record =
                    ctx.decode_field(val, Self::TABLE_NAME, "_DescriptionPickRecord")?
            }
            b"_DetailInfoRateUpDesc" => {
                self._detail_info_rate_up_desc =
                    ctx.decode_field(val, Self::TABLE_NAME, "_DetailInfoRateUpDesc")?
            }
            b"_DetailInfoRateUpTitle" => {
                self._detail_info_rate_up_title =
                    ctx.decode_field(val, Self::TABLE_NAME, "_DetailInfoRateUpTitle")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigGachaRow {
    const TABLE_NAME: &'static str = "config_gacha";

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
            || self.set_field_3(ctx, field, val)?
            || self.set_field_4(ctx, field, val)?
        {
            return Ok(());
        }
        Err(ctx.unknown_field(Self::TABLE_NAME, field))
    }
}

/// Typed `DataTable` for `config_gacha.out`.
pub type ConfigGachaTable = DataTable<u32, ConfigGachaRow>;

/// Parses `config_gacha.out` into a [`ConfigGachaTable`].
pub fn parse_config_gacha(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGachaTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_gacha_label.ItemPreview`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GachaLabelItemPreviewEntry {
    pub reward_kind: Box<str>,
    pub id: u32,
}

impl FromEvalVal for GachaLabelItemPreviewEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (reward_kind, id) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { reward_kind, id })
    }
}

/// Row record for `config_gacha_label.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGachaLabelRow {
    pub background_img: Box<str>,
    pub description_detail: Box<str>,
    pub description_ensure: Box<str>,
    pub description_ensure_simple: Box<str>,
    pub description_rate_up: Box<str>,
    pub description_title: Box<str>,
    pub gacha_id: u32,
    pub gift_list: Box<[u32]>,
    pub icon: Box<str>,
    pub id: i32,
    pub is_show_gifts: u8,
    pub item_preview: Box<[GachaLabelItemPreviewEntry]>,
    pub name: Box<str>,
    pub order: u32,
    pub pool_banner_position: FxHashMap<Box<str>, Box<str>>,
    pub pool_font_list: Box<[u16]>,
    pub pool_front_cover_type: u8,
    pub pool_info_tag_img: Box<str>,
    pub pool_name: Box<str>,
    pub pool_title_tag_img: Box<str>,
    pub return_pool_activity_bg_img: Box<str>,
    pub return_pool_activity_tag_img: Box<str>,
    pub return_pool_activity_title: Box<str>,
    pub return_pool_type: Box<[u8]>,
    pub sub_title: Box<str>,
    pub sub_title_img: Box<str>,
    pub task_id: u32,
    pub unlock_level: u8,
    pub up_banner_img: Box<str>,
    pub video_preview: Box<str>,
    pub _description_detail: Box<str>,
    pub _description_ensure: Box<str>,
    pub _description_ensure_simple: Box<str>,
    pub _description_rate_up: Box<str>,
    pub _description_title: Box<str>,
    pub _name: Box<str>,
    pub _pool_name: Box<str>,
    pub _return_pool_activity_title: Box<str>,
    pub _sub_title: Box<str>,
}

impl ConfigGachaLabelRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"BackgroundImg" => {
                self.background_img = ctx.decode_field(val, Self::TABLE_NAME, "BackgroundImg")?
            }
            b"DescriptionDetail" => {
                self.description_detail =
                    ctx.decode_field(val, Self::TABLE_NAME, "DescriptionDetail")?
            }
            b"DescriptionEnsure" => {
                self.description_ensure =
                    ctx.decode_field(val, Self::TABLE_NAME, "DescriptionEnsure")?
            }
            b"DescriptionEnsureSimple" => {
                self.description_ensure_simple =
                    ctx.decode_field(val, Self::TABLE_NAME, "DescriptionEnsureSimple")?
            }
            b"DescriptionRateUp" => {
                self.description_rate_up =
                    ctx.decode_field(val, Self::TABLE_NAME, "DescriptionRateUp")?
            }
            b"DescriptionTitle" => {
                self.description_title =
                    ctx.decode_field(val, Self::TABLE_NAME, "DescriptionTitle")?
            }
            b"GachaId" => self.gacha_id = ctx.decode_field(val, Self::TABLE_NAME, "GachaId")?,
            b"GiftList" => self.gift_list = ctx.decode_field(val, Self::TABLE_NAME, "GiftList")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsShowGifts" => {
                self.is_show_gifts = ctx.decode_field(val, Self::TABLE_NAME, "IsShowGifts")?
            }
            b"ItemPreview" => {
                self.item_preview = ctx.decode_field(val, Self::TABLE_NAME, "ItemPreview")?
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
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"PoolBannerPosition" => {
                self.pool_banner_position =
                    ctx.decode_field(val, Self::TABLE_NAME, "PoolBannerPosition")?
            }
            b"PoolFontList" => {
                self.pool_font_list = ctx.decode_field(val, Self::TABLE_NAME, "PoolFontList")?
            }
            b"PoolFrontCoverType" => {
                self.pool_front_cover_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "PoolFrontCoverType")?
            }
            b"PoolInfoTagImg" => {
                self.pool_info_tag_img =
                    ctx.decode_field(val, Self::TABLE_NAME, "PoolInfoTagImg")?
            }
            b"PoolName" => self.pool_name = ctx.decode_field(val, Self::TABLE_NAME, "PoolName")?,
            b"PoolTitleTagImg" => {
                self.pool_title_tag_img =
                    ctx.decode_field(val, Self::TABLE_NAME, "PoolTitleTagImg")?
            }
            b"ReturnPoolActivityBgImg" => {
                self.return_pool_activity_bg_img =
                    ctx.decode_field(val, Self::TABLE_NAME, "ReturnPoolActivityBgImg")?
            }
            b"ReturnPoolActivityTagImg" => {
                self.return_pool_activity_tag_img =
                    ctx.decode_field(val, Self::TABLE_NAME, "ReturnPoolActivityTagImg")?
            }
            b"ReturnPoolActivityTitle" => {
                self.return_pool_activity_title =
                    ctx.decode_field(val, Self::TABLE_NAME, "ReturnPoolActivityTitle")?
            }
            b"ReturnPoolType" => {
                self.return_pool_type = ctx.decode_field(val, Self::TABLE_NAME, "ReturnPoolType")?
            }
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
            b"SubTitle" => self.sub_title = ctx.decode_field(val, Self::TABLE_NAME, "SubTitle")?,
            b"SubTitleImg" => {
                self.sub_title_img = ctx.decode_field(val, Self::TABLE_NAME, "SubTitleImg")?
            }
            b"TaskId" => self.task_id = ctx.decode_field(val, Self::TABLE_NAME, "TaskId")?,
            b"UnlockLevel" => {
                self.unlock_level = ctx.decode_field(val, Self::TABLE_NAME, "UnlockLevel")?
            }
            b"UpBannerImg" => {
                self.up_banner_img = ctx.decode_field(val, Self::TABLE_NAME, "UpBannerImg")?
            }
            b"VideoPreview" => {
                self.video_preview = ctx.decode_field(val, Self::TABLE_NAME, "VideoPreview")?
            }
            b"_DescriptionDetail" => {
                self._description_detail =
                    ctx.decode_field(val, Self::TABLE_NAME, "_DescriptionDetail")?
            }
            b"_DescriptionEnsure" => {
                self._description_ensure =
                    ctx.decode_field(val, Self::TABLE_NAME, "_DescriptionEnsure")?
            }
            b"_DescriptionEnsureSimple" => {
                self._description_ensure_simple =
                    ctx.decode_field(val, Self::TABLE_NAME, "_DescriptionEnsureSimple")?
            }
            b"_DescriptionRateUp" => {
                self._description_rate_up =
                    ctx.decode_field(val, Self::TABLE_NAME, "_DescriptionRateUp")?
            }
            b"_DescriptionTitle" => {
                self._description_title =
                    ctx.decode_field(val, Self::TABLE_NAME, "_DescriptionTitle")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    #[inline(never)]
    fn set_field_3(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"_PoolName" => {
                self._pool_name = ctx.decode_field(val, Self::TABLE_NAME, "_PoolName")?
            }
            b"_ReturnPoolActivityTitle" => {
                self._return_pool_activity_title =
                    ctx.decode_field(val, Self::TABLE_NAME, "_ReturnPoolActivityTitle")?
            }
            b"_SubTitle" => {
                self._sub_title = ctx.decode_field(val, Self::TABLE_NAME, "_SubTitle")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigGachaLabelRow {
    const TABLE_NAME: &'static str = "config_gacha_label";

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
            || self.set_field_3(ctx, field, val)?
        {
            return Ok(());
        }
        Err(ctx.unknown_field(Self::TABLE_NAME, field))
    }
}

/// Typed `DataTable` for `config_gacha_label.out`.
pub type ConfigGachaLabelTable = DataTable<u32, ConfigGachaLabelRow>;

/// Parses `config_gacha_label.out` into a [`ConfigGachaLabelTable`].
pub fn parse_config_gacha_label(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGachaLabelTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_gacha_pool_category.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGachaPoolCategoryRow {
    pub filter_type: u8,
    pub filter_value: u8,
    pub icon_img: Box<str>,
    pub id: i8,
    pub is_root: bool,
    pub order: u8,
    pub selected_icon_img: Box<str>,
    pub sub_ids: Box<[u8]>,
    pub title: Box<str>,
    pub _title: Box<str>,
}

impl DataCfgRow for ConfigGachaPoolCategoryRow {
    const TABLE_NAME: &'static str = "config_gacha_pool_category";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"FilterType" => {
                self.filter_type = ctx.decode_field(val, Self::TABLE_NAME, "FilterType")?
            }
            b"FilterValue" => {
                self.filter_value = ctx.decode_field(val, Self::TABLE_NAME, "FilterValue")?
            }
            b"IconImg" => self.icon_img = ctx.decode_field(val, Self::TABLE_NAME, "IconImg")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsRoot" => self.is_root = ctx.decode_field(val, Self::TABLE_NAME, "IsRoot")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"SelectedIconImg" => {
                self.selected_icon_img =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectedIconImg")?
            }
            b"SubIds" => self.sub_ids = ctx.decode_field(val, Self::TABLE_NAME, "SubIds")?,
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_gacha_pool_category.out`.
pub type ConfigGachaPoolCategoryTable = DataTable<u8, ConfigGachaPoolCategoryRow>;

/// Parses `config_gacha_pool_category.out` into a [`ConfigGachaPoolCategoryTable`].
pub fn parse_config_gacha_pool_category(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGachaPoolCategoryTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

pub(crate) fn verify_gacha_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_gacha" => Some(parse_config_gacha(buf, scratch).map(|t| t.len())),
        "config_gacha_label" => Some(parse_config_gacha_label(buf, scratch).map(|t| t.len())),
        "config_gacha_pool_category" => {
            Some(parse_config_gacha_pool_category(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

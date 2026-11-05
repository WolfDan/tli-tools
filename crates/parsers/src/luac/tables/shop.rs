//! Auto-generated `LuaTable/DataCfg` types and parsers for the `shop` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;

/// Entry for `config_shop.RefreshRule`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShopRefreshRule {
    pub rule_type: u8,
    pub args: Box<[Box<str>]>,
}

impl FromEvalVal for ShopRefreshRule {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (rule_type, args) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { rule_type, args })
    }
}

/// Entry for `config_shop.Tab`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShopTabEntry {
    pub first_type: u8,
    pub second_type: u8,
}

impl FromEvalVal for ShopTabEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (first_type, second_type) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            first_type,
            second_type,
        })
    }
}

/// Row record for `config_shop.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigShopRow {
    pub can_trade: bool,
    pub condition: Box<str>,
    pub cost: Box<str>,
    pub des: Box<str>,
    pub description: Box<str>,
    pub discount: f64,
    pub end_time: Box<str>,
    pub fashion_gift_full_bg: Box<str>,
    pub fashion_gift_icon: Box<str>,
    pub fashion_gift_name: Box<str>,
    pub function: Box<str>,
    pub goods_buy_state: Box<str>,
    pub goods_buy_way: Box<str>,
    pub hide_limit: u8,
    pub hot_sale: u8,
    pub icon_goods: Box<str>,
    pub icon_money: Box<str>,
    pub id: i32,
    pub ifonly: bool,
    pub limit_count: u8,
    pub limit_once: u32,
    pub name: Box<str>,
    pub order: Box<[u16]>,
    pub pay_inherit: Box<str>,
    pub rarity: u8,
    pub refresh_rule: ShopRefreshRule,
    pub rule_text: Box<str>,
    pub season: u8,
    pub season_type: Box<[u16]>,
    pub start_time: Box<str>,
    pub subtab: u8,
    pub tab: Box<[ShopTabEntry]>,
    pub unlock_level: u8,
    pub _description: Box<str>,
    pub _fashion_gift_name: Box<str>,
    pub _goods_buy_way: Box<str>,
    pub _name: Box<str>,
    pub _rule_text: Box<str>,
}

impl ConfigShopRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"CanTrade" => self.can_trade = ctx.decode_field(val, Self::TABLE_NAME, "CanTrade")?,
            b"Condition" => {
                self.condition = ctx.decode_field(val, Self::TABLE_NAME, "Condition")?
            }
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Description" => {
                self.description = ctx.decode_field(val, Self::TABLE_NAME, "Description")?
            }
            b"Discount" => self.discount = ctx.decode_field(val, Self::TABLE_NAME, "Discount")?,
            b"EndTime" => self.end_time = ctx.decode_field(val, Self::TABLE_NAME, "EndTime")?,
            b"FashionGiftFullBg" => {
                self.fashion_gift_full_bg =
                    ctx.decode_field(val, Self::TABLE_NAME, "FashionGiftFullBg")?
            }
            b"FashionGiftIcon" => {
                self.fashion_gift_icon =
                    ctx.decode_field(val, Self::TABLE_NAME, "FashionGiftIcon")?
            }
            b"FashionGiftName" => {
                self.fashion_gift_name =
                    ctx.decode_field(val, Self::TABLE_NAME, "FashionGiftName")?
            }
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"GoodsBuyState" => {
                self.goods_buy_state = ctx.decode_field(val, Self::TABLE_NAME, "GoodsBuyState")?
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
            b"GoodsBuyWay" => {
                self.goods_buy_way = ctx.decode_field(val, Self::TABLE_NAME, "GoodsBuyWay")?
            }
            b"HideLimit" => {
                self.hide_limit = ctx.decode_field(val, Self::TABLE_NAME, "HideLimit")?
            }
            b"HotSale" => self.hot_sale = ctx.decode_field(val, Self::TABLE_NAME, "HotSale")?,
            b"IconGoods" => {
                self.icon_goods = ctx.decode_field(val, Self::TABLE_NAME, "IconGoods")?
            }
            b"IconMoney" => {
                self.icon_money = ctx.decode_field(val, Self::TABLE_NAME, "IconMoney")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Ifonly" => self.ifonly = ctx.decode_field(val, Self::TABLE_NAME, "Ifonly")?,
            b"LimitCount" => {
                self.limit_count = ctx.decode_field(val, Self::TABLE_NAME, "LimitCount")?
            }
            b"LimitOnce" => {
                self.limit_once = ctx.decode_field(val, Self::TABLE_NAME, "LimitOnce")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"PayInherit" => {
                self.pay_inherit = ctx.decode_field(val, Self::TABLE_NAME, "PayInherit")?
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
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"RefreshRule" => {
                self.refresh_rule = ctx.decode_field(val, Self::TABLE_NAME, "RefreshRule")?
            }
            b"RuleText" => self.rule_text = ctx.decode_field(val, Self::TABLE_NAME, "RuleText")?,
            b"Season" => self.season = ctx.decode_field(val, Self::TABLE_NAME, "Season")?,
            b"SeasonType" => {
                self.season_type = ctx.decode_field(val, Self::TABLE_NAME, "SeasonType")?
            }
            b"StartTime" => {
                self.start_time = ctx.decode_field(val, Self::TABLE_NAME, "StartTime")?
            }
            b"Subtab" => self.subtab = ctx.decode_field(val, Self::TABLE_NAME, "Subtab")?,
            b"Tab" => self.tab = ctx.decode_field(val, Self::TABLE_NAME, "Tab")?,
            b"UnlockLevel" => {
                self.unlock_level = ctx.decode_field(val, Self::TABLE_NAME, "UnlockLevel")?
            }
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_FashionGiftName" => {
                self._fashion_gift_name =
                    ctx.decode_field(val, Self::TABLE_NAME, "_FashionGiftName")?
            }
            b"_GoodsBuyWay" => {
                self._goods_buy_way = ctx.decode_field(val, Self::TABLE_NAME, "_GoodsBuyWay")?
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
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_RuleText" => {
                self._rule_text = ctx.decode_field(val, Self::TABLE_NAME, "_RuleText")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigShopRow {
    const TABLE_NAME: &'static str = "config_shop";

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

/// Typed `DataTable` for `config_shop.out`.
pub type ConfigShopTable = DataTable<u32, ConfigShopRow>;

/// Parses `config_shop.out` into a [`ConfigShopTable`].
pub fn parse_config_shop(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigShopTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_shop_form.RefreshRule`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShopFormRefreshRule {
    pub rule_type: u8,
    pub args: Box<[Box<str>]>,
}

impl FromEvalVal for ShopFormRefreshRule {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (rule_type, args) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { rule_type, args })
    }
}

/// Row record for `config_shop_form.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigShopFormRow {
    pub allsubtab: Box<str>,
    pub existgoods: u8,
    pub moneytoshow: Box<[u8]>,
    pub order: u8,
    pub order_in_rare: u8,
    pub pageviewtype: u8,
    pub preview_order: u8,
    pub refresh_rule: ShopFormRefreshRule,
    pub subtab: Box<str>,
    pub tab: Box<str>,
    pub r#type: i8,
    pub _allsubtab: Box<str>,
    pub _subtab: Box<str>,
    pub _tab: Box<str>,
}

impl ConfigShopFormRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Allsubtab" => {
                self.allsubtab = ctx.decode_field(val, Self::TABLE_NAME, "Allsubtab")?
            }
            b"Existgoods" => {
                self.existgoods = ctx.decode_field(val, Self::TABLE_NAME, "Existgoods")?
            }
            b"Moneytoshow" => {
                self.moneytoshow = ctx.decode_field(val, Self::TABLE_NAME, "Moneytoshow")?
            }
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"OrderInRare" => {
                self.order_in_rare = ctx.decode_field(val, Self::TABLE_NAME, "OrderInRare")?
            }
            b"Pageviewtype" => {
                self.pageviewtype = ctx.decode_field(val, Self::TABLE_NAME, "Pageviewtype")?
            }
            b"PreviewOrder" => {
                self.preview_order = ctx.decode_field(val, Self::TABLE_NAME, "PreviewOrder")?
            }
            b"RefreshRule" => {
                self.refresh_rule = ctx.decode_field(val, Self::TABLE_NAME, "RefreshRule")?
            }
            b"Subtab" => self.subtab = ctx.decode_field(val, Self::TABLE_NAME, "Subtab")?,
            b"Tab" => self.tab = ctx.decode_field(val, Self::TABLE_NAME, "Tab")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Allsubtab" => {
                self._allsubtab = ctx.decode_field(val, Self::TABLE_NAME, "_Allsubtab")?
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
            b"_Subtab" => self._subtab = ctx.decode_field(val, Self::TABLE_NAME, "_Subtab")?,
            b"_Tab" => self._tab = ctx.decode_field(val, Self::TABLE_NAME, "_Tab")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigShopFormRow {
    const TABLE_NAME: &'static str = "config_shop_form";

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

/// Typed `DataTable` for `config_shop_form.out`.
pub type ConfigShopFormTable = DataTable<u8, ConfigShopFormRow>;

/// Parses `config_shop_form.out` into a [`ConfigShopFormTable`].
pub fn parse_config_shop_form(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigShopFormTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_shop_recommond.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigShopRecommondRow {
    pub begin_time: u32,
    pub end_time: u32,
    pub image_path: Box<str>,
    pub jump: Box<str>,
    pub name: Box<str>,
    pub order: u8,
    pub text_cfg1: Box<str>,
    pub text_cfg2: Box<str>,
    pub text_cfg3: Box<str>,
    pub text_cfg4: Box<str>,
    pub text_content1: Box<str>,
    pub text_content2: Box<str>,
    pub text_content3: Box<str>,
    pub text_content4: Box<str>,
    pub r#type: i16,
    pub _name: Box<str>,
    pub _text_content1: Box<str>,
    pub _text_content2: Box<str>,
    pub _text_content3: Box<str>,
    pub _text_content4: Box<str>,
}

impl ConfigShopRecommondRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"BeginTime" => {
                self.begin_time = ctx.decode_field(val, Self::TABLE_NAME, "BeginTime")?
            }
            b"EndTime" => self.end_time = ctx.decode_field(val, Self::TABLE_NAME, "EndTime")?,
            b"ImagePath" => {
                self.image_path = ctx.decode_field(val, Self::TABLE_NAME, "ImagePath")?
            }
            b"Jump" => self.jump = ctx.decode_field(val, Self::TABLE_NAME, "Jump")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"TextCfg1" => self.text_cfg1 = ctx.decode_field(val, Self::TABLE_NAME, "TextCfg1")?,
            b"TextCfg2" => self.text_cfg2 = ctx.decode_field(val, Self::TABLE_NAME, "TextCfg2")?,
            b"TextCfg3" => self.text_cfg3 = ctx.decode_field(val, Self::TABLE_NAME, "TextCfg3")?,
            b"TextCfg4" => self.text_cfg4 = ctx.decode_field(val, Self::TABLE_NAME, "TextCfg4")?,
            b"TextContent1" => {
                self.text_content1 = ctx.decode_field(val, Self::TABLE_NAME, "TextContent1")?
            }
            b"TextContent2" => {
                self.text_content2 = ctx.decode_field(val, Self::TABLE_NAME, "TextContent2")?
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
            b"TextContent3" => {
                self.text_content3 = ctx.decode_field(val, Self::TABLE_NAME, "TextContent3")?
            }
            b"TextContent4" => {
                self.text_content4 = ctx.decode_field(val, Self::TABLE_NAME, "TextContent4")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_TextContent1" => {
                self._text_content1 = ctx.decode_field(val, Self::TABLE_NAME, "_TextContent1")?
            }
            b"_TextContent2" => {
                self._text_content2 = ctx.decode_field(val, Self::TABLE_NAME, "_TextContent2")?
            }
            b"_TextContent3" => {
                self._text_content3 = ctx.decode_field(val, Self::TABLE_NAME, "_TextContent3")?
            }
            b"_TextContent4" => {
                self._text_content4 = ctx.decode_field(val, Self::TABLE_NAME, "_TextContent4")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigShopRecommondRow {
    const TABLE_NAME: &'static str = "config_shop_recommond";

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

/// Typed `DataTable` for `config_shop_recommond.out`.
pub type ConfigShopRecommondTable = DataTable<u16, ConfigShopRecommondRow>;

/// Parses `config_shop_recommond.out` into a [`ConfigShopRecommondTable`].
pub fn parse_config_shop_recommond(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigShopRecommondTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_vendor.RefreshConsume`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VendorRefreshConsumeEntry {
    pub item_id: u16,
    pub count: u8,
}

impl FromEvalVal for VendorRefreshConsumeEntry {
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

/// Row record for `config_vendor.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigVendorRow {
    pub currency: Box<[u32]>,
    pub free_text: Box<str>,
    pub function: Box<[u16]>,
    pub id: i16,
    pub money: Box<[u16]>,
    pub name: Box<str>,
    pub passive_refresh: [u8; 0],
    pub purchase_limit: u8,
    pub refresh_consume: Box<[VendorRefreshConsumeEntry]>,
    pub refresh_limit: u8,
    pub refresh_reset: u8,
    pub season: Box<[u16]>,
    pub sort_order: u8,
    pub unlock_des: Box<str>,
    pub unlock_requirements: FxHashMap<u8, u32>,
    pub _name: Box<str>,
    pub _unlock_des: Box<str>,
}

impl ConfigVendorRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Currency" => self.currency = ctx.decode_field(val, Self::TABLE_NAME, "Currency")?,
            b"FreeText" => self.free_text = ctx.decode_field(val, Self::TABLE_NAME, "FreeText")?,
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Money" => self.money = ctx.decode_field(val, Self::TABLE_NAME, "Money")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"PassiveRefresh" => {
                self.passive_refresh = ctx.decode_field(val, Self::TABLE_NAME, "PassiveRefresh")?
            }
            b"PurchaseLimit" => {
                self.purchase_limit = ctx.decode_field(val, Self::TABLE_NAME, "PurchaseLimit")?
            }
            b"RefreshConsume" => {
                self.refresh_consume = ctx.decode_field(val, Self::TABLE_NAME, "RefreshConsume")?
            }
            b"RefreshLimit" => {
                self.refresh_limit = ctx.decode_field(val, Self::TABLE_NAME, "RefreshLimit")?
            }
            b"RefreshReset" => {
                self.refresh_reset = ctx.decode_field(val, Self::TABLE_NAME, "RefreshReset")?
            }
            b"Season" => self.season = ctx.decode_field(val, Self::TABLE_NAME, "Season")?,
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
            b"SortOrder" => {
                self.sort_order = ctx.decode_field(val, Self::TABLE_NAME, "SortOrder")?
            }
            b"UnlockDes" => {
                self.unlock_des = ctx.decode_field(val, Self::TABLE_NAME, "UnlockDes")?
            }
            b"UnlockRequirements" => {
                self.unlock_requirements =
                    ctx.decode_field(val, Self::TABLE_NAME, "UnlockRequirements")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_UnlockDes" => {
                self._unlock_des = ctx.decode_field(val, Self::TABLE_NAME, "_UnlockDes")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigVendorRow {
    const TABLE_NAME: &'static str = "config_vendor";

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

/// Typed `DataTable` for `config_vendor.out`.
pub type ConfigVendorTable = DataTable<u16, ConfigVendorRow>;

/// Parses `config_vendor.out` into a [`ConfigVendorTable`].
pub fn parse_config_vendor(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigVendorTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_vendor_goods.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigVendorGoodsRow {
    pub buy_limit: i16,
    pub group: u32,
    pub id: i32,
    pub item_desc: Box<str>,
    pub item_icon: Box<str>,
    pub item_name: Box<str>,
    pub level_limit: u16,
    pub price: Box<str>,
    pub price_type: u8,
    pub quality_color: u8,
    pub random_reward: u8,
    pub refresh_limit: Box<[u16]>,
    pub reward_id: u32,
    pub show_limit: u8,
    pub weight: u32,
    pub _item_desc: Box<str>,
    pub _item_name: Box<str>,
}

impl ConfigVendorGoodsRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"BuyLimit" => self.buy_limit = ctx.decode_field(val, Self::TABLE_NAME, "BuyLimit")?,
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemDesc" => self.item_desc = ctx.decode_field(val, Self::TABLE_NAME, "ItemDesc")?,
            b"ItemIcon" => self.item_icon = ctx.decode_field(val, Self::TABLE_NAME, "ItemIcon")?,
            b"ItemName" => self.item_name = ctx.decode_field(val, Self::TABLE_NAME, "ItemName")?,
            b"LevelLimit" => {
                self.level_limit = ctx.decode_field(val, Self::TABLE_NAME, "LevelLimit")?
            }
            b"Price" => self.price = ctx.decode_field(val, Self::TABLE_NAME, "Price")?,
            b"PriceType" => {
                self.price_type = ctx.decode_field(val, Self::TABLE_NAME, "PriceType")?
            }
            b"QualityColor" => {
                self.quality_color = ctx.decode_field(val, Self::TABLE_NAME, "QualityColor")?
            }
            b"RandomReward" => {
                self.random_reward = ctx.decode_field(val, Self::TABLE_NAME, "RandomReward")?
            }
            b"RefreshLimit" => {
                self.refresh_limit = ctx.decode_field(val, Self::TABLE_NAME, "RefreshLimit")?
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
            b"RewardId" => self.reward_id = ctx.decode_field(val, Self::TABLE_NAME, "RewardId")?,
            b"ShowLimit" => {
                self.show_limit = ctx.decode_field(val, Self::TABLE_NAME, "ShowLimit")?
            }
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_ItemDesc" => {
                self._item_desc = ctx.decode_field(val, Self::TABLE_NAME, "_ItemDesc")?
            }
            b"_ItemName" => {
                self._item_name = ctx.decode_field(val, Self::TABLE_NAME, "_ItemName")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigVendorGoodsRow {
    const TABLE_NAME: &'static str = "config_vendor_goods";

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

/// Typed `DataTable` for `config_vendor_goods.out`.
pub type ConfigVendorGoodsTable = DataTable<u32, ConfigVendorGoodsRow>;

/// Parses `config_vendor_goods.out` into a [`ConfigVendorGoodsTable`].
pub fn parse_config_vendor_goods(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigVendorGoodsTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_vendor_skill.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigVendorSkillRow {
    pub cost_item_id: u16,
    pub hero_id: u16,
    pub hero_purchase_id: u16,
    pub id: i32,
    pub item_base: u16,
    pub level: u8,
    pub num2: u8,
    pub require_level: u8,
    pub require_task: u8,
    pub slot_num: u8,
}

impl DataCfgRow for ConfigVendorSkillRow {
    const TABLE_NAME: &'static str = "config_vendor_skill";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CostItemId" => {
                self.cost_item_id = ctx.decode_field(val, Self::TABLE_NAME, "CostItemId")?
            }
            b"HeroId" => self.hero_id = ctx.decode_field(val, Self::TABLE_NAME, "HeroId")?,
            b"HeroPurchaseId" => {
                self.hero_purchase_id = ctx.decode_field(val, Self::TABLE_NAME, "HeroPurchaseId")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemBase" => self.item_base = ctx.decode_field(val, Self::TABLE_NAME, "ItemBase")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Num2" => self.num2 = ctx.decode_field(val, Self::TABLE_NAME, "Num2")?,
            b"RequireLevel" => {
                self.require_level = ctx.decode_field(val, Self::TABLE_NAME, "RequireLevel")?
            }
            b"RequireTask" => {
                self.require_task = ctx.decode_field(val, Self::TABLE_NAME, "RequireTask")?
            }
            b"SlotNum" => self.slot_num = ctx.decode_field(val, Self::TABLE_NAME, "SlotNum")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_vendor_skill.out`.
pub type ConfigVendorSkillTable = DataTable<u32, ConfigVendorSkillRow>;

/// Parses `config_vendor_skill.out` into a [`ConfigVendorSkillTable`].
pub fn parse_config_vendor_skill(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigVendorSkillTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

pub(crate) fn verify_shop_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_shop" => Some(parse_config_shop(buf, scratch).map(|t| t.len())),
        "config_shop_form" => Some(parse_config_shop_form(buf, scratch).map(|t| t.len())),
        "config_shop_recommond" => Some(parse_config_shop_recommond(buf, scratch).map(|t| t.len())),
        "config_vendor" => Some(parse_config_vendor(buf, scratch).map(|t| t.len())),
        "config_vendor_goods" => Some(parse_config_vendor_goods(buf, scratch).map(|t| t.len())),
        "config_vendor_skill" => Some(parse_config_vendor_skill(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

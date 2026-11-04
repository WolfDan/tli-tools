//! Auto-generated `LuaTable/DataCfg` types and parsers for the `auction` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;

/// Row record for `config_auction_goodsnum.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAuctionGoodsnumRow {
    pub id: i8,
    pub max_num: u16,
    pub name: Box<str>,
    pub type1: u8,
    pub type2: u8,
    pub type3: u8,
    pub type4: u8,
}

impl DataCfgRow for ConfigAuctionGoodsnumRow {
    const TABLE_NAME: &'static str = "config_auction_goodsnum";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MaxNum" => self.max_num = ctx.decode_field(val, Self::TABLE_NAME, "MaxNum")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_auction_goodsnum.out`.
pub type ConfigAuctionGoodsnumTable = DataTable<u8, ConfigAuctionGoodsnumRow>;

/// Parses `config_auction_goodsnum.out` into a [`ConfigAuctionGoodsnumTable`].
pub fn parse_config_auction_goodsnum(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAuctionGoodsnumTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_auction_menu.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAuctionMenuRow {
    pub fast_buy: u8,
    pub id: i16,
    pub type1: u8,
    pub type2: u16,
    pub type3: u16,
    pub type4: u8,
}

impl DataCfgRow for ConfigAuctionMenuRow {
    const TABLE_NAME: &'static str = "config_auction_menu";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"FastBuy" => self.fast_buy = ctx.decode_field(val, Self::TABLE_NAME, "FastBuy")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_auction_menu.out`.
pub type ConfigAuctionMenuTable = DataTable<u8, ConfigAuctionMenuRow>;

/// Parses `config_auction_menu.out` into a [`ConfigAuctionMenuTable`].
pub fn parse_config_auction_menu(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAuctionMenuTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_auction_tax.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAuctionTaxRow {
    pub id: i32,
    pub tax_exemption: u8,
}

impl DataCfgRow for ConfigAuctionTaxRow {
    const TABLE_NAME: &'static str = "config_auction_tax";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"TaxExemption" => {
                self.tax_exemption = ctx.decode_field(val, Self::TABLE_NAME, "TaxExemption")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_auction_tax.out`.
pub type ConfigAuctionTaxTable = DataTable<u32, ConfigAuctionTaxRow>;

/// Parses `config_auction_tax.out` into a [`ConfigAuctionTaxTable`].
pub fn parse_config_auction_tax(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAuctionTaxTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_auctionhouse_conditionlist.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAuctionhouseConditionlistRow {
    pub condition_type: u8,
    pub description: Box<str>,
    pub id: i8,
    pub index_id: Box<[u16]>,
    pub is_multi_select: bool,
    pub math_form: Box<str>,
    pub notallowed_input_param: [u8; 0],
    pub selection_content: Box<[u8]>,
    pub selection_des: Box<str>,
    pub _description: Box<str>,
    pub _selection_des: Box<str>,
}

impl DataCfgRow for ConfigAuctionhouseConditionlistRow {
    const TABLE_NAME: &'static str = "config_auctionhouse_conditionlist";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ConditionType" => {
                self.condition_type = ctx.decode_field(val, Self::TABLE_NAME, "ConditionType")?
            }
            b"Description" => {
                self.description = ctx.decode_field(val, Self::TABLE_NAME, "Description")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IndexId" => self.index_id = ctx.decode_field(val, Self::TABLE_NAME, "IndexId")?,
            b"IsMultiSelect" => {
                self.is_multi_select = ctx.decode_field(val, Self::TABLE_NAME, "IsMultiSelect")?
            }
            b"MathForm" => self.math_form = ctx.decode_field(val, Self::TABLE_NAME, "MathForm")?,
            b"NotallowedInputParam" => {
                self.notallowed_input_param =
                    ctx.decode_field(val, Self::TABLE_NAME, "NotallowedInputParam")?
            }
            b"SelectionContent" => {
                self.selection_content =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionContent")?
            }
            b"SelectionDes" => {
                self.selection_des = ctx.decode_field(val, Self::TABLE_NAME, "SelectionDes")?
            }
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_SelectionDes" => {
                self._selection_des = ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDes")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_auctionhouse_conditionlist.out`.
pub type ConfigAuctionhouseConditionlistTable = DataTable<u8, ConfigAuctionhouseConditionlistRow>;

/// Parses `config_auctionhouse_conditionlist.out` into a [`ConfigAuctionhouseConditionlistTable`].
pub fn parse_config_auctionhouse_conditionlist(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAuctionhouseConditionlistTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_auctionhouse_detail_search.DealSearchDefaultModifier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AuctionhouseDetailSearchDealSearchDefaultModifierEntry {
    pub match_mode: u8,
    pub affix_type: u8,
    pub min_value: u16,
}

impl FromEvalVal for AuctionhouseDetailSearchDealSearchDefaultModifierEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (match_mode, (affix_type, min_value)) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            match_mode,
            affix_type,
            min_value,
        })
    }
}

/// Row record for `config_auctionhouse_detail_search.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAuctionhouseDetailSearchRow {
    pub auto_deal_search: u8,
    pub condition_id: Box<[u8]>,
    pub condition_modifier_type: Box<[u8]>,
    pub deal_search_condition: Box<[u8]>,
    pub deal_search_default_condition: Box<[u8]>,
    pub deal_search_default_modifier: Box<[AuctionhouseDetailSearchDealSearchDefaultModifierEntry]>,
    pub deal_search_with_base: u8,
    pub description: Box<str>,
    pub extra_tag_list: Box<[u8]>,
    pub icon: Box<str>,
    pub id: i32,
    pub implid_condition: FxHashMap<u8, u8>,
    pub islegend: u8,
    pub item_type: [u8; 0],
    pub menu_id: u8,
    pub name: Box<str>,
    pub node_id: u16,
    pub sort: u8,
    pub _description: Box<str>,
    pub _name: Box<str>,
}

impl ConfigAuctionhouseDetailSearchRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AutoDealSearch" => {
                self.auto_deal_search = ctx.decode_field(val, Self::TABLE_NAME, "AutoDealSearch")?
            }
            b"ConditionId" => {
                self.condition_id = ctx.decode_field(val, Self::TABLE_NAME, "ConditionId")?
            }
            b"ConditionModifierType" => {
                self.condition_modifier_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "ConditionModifierType")?
            }
            b"DealSearchCondition" => {
                self.deal_search_condition =
                    ctx.decode_field(val, Self::TABLE_NAME, "DealSearchCondition")?
            }
            b"DealSearchDefaultCondition" => {
                self.deal_search_default_condition =
                    ctx.decode_field(val, Self::TABLE_NAME, "DealSearchDefaultCondition")?
            }
            b"DealSearchDefaultModifier" => {
                self.deal_search_default_modifier =
                    ctx.decode_field(val, Self::TABLE_NAME, "DealSearchDefaultModifier")?
            }
            b"DealSearchWithBase" => {
                self.deal_search_with_base =
                    ctx.decode_field(val, Self::TABLE_NAME, "DealSearchWithBase")?
            }
            b"Description" => {
                self.description = ctx.decode_field(val, Self::TABLE_NAME, "Description")?
            }
            b"ExtraTagList" => {
                self.extra_tag_list = ctx.decode_field(val, Self::TABLE_NAME, "ExtraTagList")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ImplidCondition" => {
                self.implid_condition =
                    ctx.decode_field(val, Self::TABLE_NAME, "ImplidCondition")?
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
            b"Islegend" => self.islegend = ctx.decode_field(val, Self::TABLE_NAME, "Islegend")?,
            b"ItemType" => self.item_type = ctx.decode_field(val, Self::TABLE_NAME, "ItemType")?,
            b"MenuId" => self.menu_id = ctx.decode_field(val, Self::TABLE_NAME, "MenuId")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"NodeId" => self.node_id = ctx.decode_field(val, Self::TABLE_NAME, "NodeId")?,
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigAuctionhouseDetailSearchRow {
    const TABLE_NAME: &'static str = "config_auctionhouse_detail_search";

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

/// Typed `DataTable` for `config_auctionhouse_detail_search.out`.
pub type ConfigAuctionhouseDetailSearchTable = DataTable<u32, ConfigAuctionhouseDetailSearchRow>;

/// Parses `config_auctionhouse_detail_search.out` into a [`ConfigAuctionhouseDetailSearchTable`].
pub fn parse_config_auctionhouse_detail_search(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAuctionhouseDetailSearchTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_auctionhouse_detail_search_tag.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAuctionhouseDetailSearchTagRow {
    pub id: i8,
    pub name: Box<str>,
    pub tag_param: u8,
    pub tag_type: u8,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigAuctionhouseDetailSearchTagRow {
    const TABLE_NAME: &'static str = "config_auctionhouse_detail_search_tag";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"TagParam" => self.tag_param = ctx.decode_field(val, Self::TABLE_NAME, "TagParam")?,
            b"TagType" => self.tag_type = ctx.decode_field(val, Self::TABLE_NAME, "TagType")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_auctionhouse_detail_search_tag.out`.
pub type ConfigAuctionhouseDetailSearchTagTable =
    DataTable<u8, ConfigAuctionhouseDetailSearchTagRow>;

/// Parses `config_auctionhouse_detail_search_tag.out` into a [`ConfigAuctionhouseDetailSearchTagTable`].
pub fn parse_config_auctionhouse_detail_search_tag(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAuctionhouseDetailSearchTagTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_auctionhouse_invalid_itemid.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAuctionhouseInvalidItemidRow {
    pub id: i8,
    pub invalid_type: u8,
    pub item_id: u8,
    pub legend_id: u8,
    pub list_type: u8,
    pub name: Box<str>,
}

impl DataCfgRow for ConfigAuctionhouseInvalidItemidRow {
    const TABLE_NAME: &'static str = "config_auctionhouse_invalid_itemid";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InvalidType" => {
                self.invalid_type = ctx.decode_field(val, Self::TABLE_NAME, "InvalidType")?
            }
            b"ItemId" => self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?,
            b"LegendId" => self.legend_id = ctx.decode_field(val, Self::TABLE_NAME, "LegendId")?,
            b"ListType" => self.list_type = ctx.decode_field(val, Self::TABLE_NAME, "ListType")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_auctionhouse_invalid_itemid.out`.
pub type ConfigAuctionhouseInvalidItemidTable = DataTable<u32, ConfigAuctionhouseInvalidItemidRow>;

/// Parses `config_auctionhouse_invalid_itemid.out` into a [`ConfigAuctionhouseInvalidItemidTable`].
pub fn parse_config_auctionhouse_invalid_itemid(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAuctionhouseInvalidItemidTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_auctionhouse_invalid_itemtype.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAuctionhouseInvalidItemtypeRow {
    pub id: i8,
    pub invalid_type: u8,
    pub list_type: u8,
    pub name: Box<str>,
    pub type1: u8,
    pub type2: u8,
    pub type3: u8,
    pub type4: u8,
}

impl DataCfgRow for ConfigAuctionhouseInvalidItemtypeRow {
    const TABLE_NAME: &'static str = "config_auctionhouse_invalid_itemtype";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InvalidType" => {
                self.invalid_type = ctx.decode_field(val, Self::TABLE_NAME, "InvalidType")?
            }
            b"ListType" => self.list_type = ctx.decode_field(val, Self::TABLE_NAME, "ListType")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_auctionhouse_invalid_itemtype.out`.
pub type ConfigAuctionhouseInvalidItemtypeTable =
    DataTable<u8, ConfigAuctionhouseInvalidItemtypeRow>;

/// Parses `config_auctionhouse_invalid_itemtype.out` into a [`ConfigAuctionhouseInvalidItemtypeTable`].
pub fn parse_config_auctionhouse_invalid_itemtype(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAuctionhouseInvalidItemtypeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_auctionhouse_rough_search.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAuctionhouseRoughSearchRow {
    pub description: Box<str>,
    pub description02: Box<str>,
    pub icon02: Box<str>,
    pub id: i16,
    pub menu_id: u8,
    pub order: u8,
    pub _description: Box<str>,
    pub _description02: Box<str>,
}

impl DataCfgRow for ConfigAuctionhouseRoughSearchRow {
    const TABLE_NAME: &'static str = "config_auctionhouse_rough_search";

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
            b"Description02" => {
                self.description02 = ctx.decode_field(val, Self::TABLE_NAME, "Description02")?
            }
            b"Icon02" => self.icon02 = ctx.decode_field(val, Self::TABLE_NAME, "Icon02")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MenuId" => self.menu_id = ctx.decode_field(val, Self::TABLE_NAME, "MenuId")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_Description02" => {
                self._description02 = ctx.decode_field(val, Self::TABLE_NAME, "_Description02")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_auctionhouse_rough_search.out`.
pub type ConfigAuctionhouseRoughSearchTable = DataTable<u16, ConfigAuctionhouseRoughSearchRow>;

/// Parses `config_auctionhouse_rough_search.out` into a [`ConfigAuctionhouseRoughSearchTable`].
pub fn parse_config_auctionhouse_rough_search(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAuctionhouseRoughSearchTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_auctionhouse_taxrebate.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAuctionhouseTaxrebateRow {
    pub currency_id: u32,
    pub id: i8,
    pub item_id: u32,
}

impl DataCfgRow for ConfigAuctionhouseTaxrebateRow {
    const TABLE_NAME: &'static str = "config_auctionhouse_taxrebate";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CurrencyId" => {
                self.currency_id = ctx.decode_field(val, Self::TABLE_NAME, "CurrencyId")?
            }
            b"ID" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "ID")?,
            b"ItemId" => self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_auctionhouse_taxrebate.out`.
pub type ConfigAuctionhouseTaxrebateTable = DataTable<u8, ConfigAuctionhouseTaxrebateRow>;

/// Parses `config_auctionhouse_taxrebate.out` into a [`ConfigAuctionhouseTaxrebateTable`].
pub fn parse_config_auctionhouse_taxrebate(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigAuctionhouseTaxrebateTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

pub(crate) fn verify_auction_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_auction_goodsnum" => {
            Some(parse_config_auction_goodsnum(buf, scratch).map(|t| t.len()))
        }
        "config_auction_menu" => Some(parse_config_auction_menu(buf, scratch).map(|t| t.len())),
        "config_auction_tax" => Some(parse_config_auction_tax(buf, scratch).map(|t| t.len())),
        "config_auctionhouse_conditionlist" => {
            Some(parse_config_auctionhouse_conditionlist(buf, scratch).map(|t| t.len()))
        }
        "config_auctionhouse_detail_search" => {
            Some(parse_config_auctionhouse_detail_search(buf, scratch).map(|t| t.len()))
        }
        "config_auctionhouse_detail_search_tag" => {
            Some(parse_config_auctionhouse_detail_search_tag(buf, scratch).map(|t| t.len()))
        }
        "config_auctionhouse_invalid_itemid" => {
            Some(parse_config_auctionhouse_invalid_itemid(buf, scratch).map(|t| t.len()))
        }
        "config_auctionhouse_invalid_itemtype" => {
            Some(parse_config_auctionhouse_invalid_itemtype(buf, scratch).map(|t| t.len()))
        }
        "config_auctionhouse_rough_search" => {
            Some(parse_config_auctionhouse_rough_search(buf, scratch).map(|t| t.len()))
        }
        "config_auctionhouse_taxrebate" => {
            Some(parse_config_auctionhouse_taxrebate(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

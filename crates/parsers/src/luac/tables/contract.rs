//! Auto-generated `LuaTable/DataCfg` types and parsers for the `contract` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;

/// Row record for `config_contract_level.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigContractLevelRow {
    pub id: i8,
    pub number: u8,
}

impl DataCfgRow for ConfigContractLevelRow {
    const TABLE_NAME: &'static str = "config_contract_level";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Number" => self.number = ctx.decode_field(val, Self::TABLE_NAME, "Number")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_contract_level.out`.
pub type ConfigContractLevelTable = DataTable<u8, ConfigContractLevelRow>;

/// Parses `config_contract_level.out` into a [`ConfigContractLevelTable`].
pub fn parse_config_contract_level(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigContractLevelTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_contract_point_tattoo_slot.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigContractPointTattooSlotRow {
    pub add_description: Box<str>,
    pub cond_event_id: u16,
    pub contract_point_add: u8,
    pub contract_point_unlock: Box<[u16]>,
    pub flash_back_season_type: Box<[u8]>,
    pub id: i8,
    pub slot_add_num: FxHashMap<u8, u8>,
    pub slot_add_num_flash_back: FxHashMap<u8, u16>,
    pub unlock_level: u8,
    pub _add_description: Box<str>,
}

impl DataCfgRow for ConfigContractPointTattooSlotRow {
    const TABLE_NAME: &'static str = "config_contract_point_tattoo_slot";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AddDescription" => {
                self.add_description = ctx.decode_field(val, Self::TABLE_NAME, "AddDescription")?
            }
            b"CondEventId" => {
                self.cond_event_id = ctx.decode_field(val, Self::TABLE_NAME, "CondEventId")?
            }
            b"ContractPointAdd" => {
                self.contract_point_add =
                    ctx.decode_field(val, Self::TABLE_NAME, "ContractPointAdd")?
            }
            b"ContractPointUnlock" => {
                self.contract_point_unlock =
                    ctx.decode_field(val, Self::TABLE_NAME, "ContractPointUnlock")?
            }
            b"FlashBackSeasonType" => {
                self.flash_back_season_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "FlashBackSeasonType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"SlotAddNum" => {
                self.slot_add_num = ctx.decode_field(val, Self::TABLE_NAME, "SlotAddNum")?
            }
            b"SlotAddNumFlashBack" => {
                self.slot_add_num_flash_back =
                    ctx.decode_field(val, Self::TABLE_NAME, "SlotAddNumFlashBack")?
            }
            b"UnlockLevel" => {
                self.unlock_level = ctx.decode_field(val, Self::TABLE_NAME, "UnlockLevel")?
            }
            b"_AddDescription" => {
                self._add_description =
                    ctx.decode_field(val, Self::TABLE_NAME, "_AddDescription")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_contract_point_tattoo_slot.out`.
pub type ConfigContractPointTattooSlotTable = DataTable<u8, ConfigContractPointTattooSlotRow>;

/// Parses `config_contract_point_tattoo_slot.out` into a [`ConfigContractPointTattooSlotTable`].
pub fn parse_config_contract_point_tattoo_slot(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigContractPointTattooSlotTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_contract_point_ui_content.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigContractPointUiContentRow {
    pub icon: Box<str>,
    pub id: i16,
    pub name: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigContractPointUiContentRow {
    const TABLE_NAME: &'static str = "config_contract_point_ui_content";

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
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_contract_point_ui_content.out`.
pub type ConfigContractPointUiContentTable = DataTable<u8, ConfigContractPointUiContentRow>;

/// Parses `config_contract_point_ui_content.out` into a [`ConfigContractPointUiContentTable`].
pub fn parse_config_contract_point_ui_content(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigContractPointUiContentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

pub(crate) fn verify_contract_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_contract_level" => Some(parse_config_contract_level(buf, scratch).map(|t| t.len())),
        "config_contract_point_tattoo_slot" => {
            Some(parse_config_contract_point_tattoo_slot(buf, scratch).map(|t| t.len()))
        }
        "config_contract_point_ui_content" => {
            Some(parse_config_contract_point_ui_content(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

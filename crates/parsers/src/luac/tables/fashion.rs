//! Auto-generated `LuaTable/DataCfg` types and parsers for the `fashion` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;

/// Sub-record for `config_fashion_show_resources.Position`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FashionShowResourcesPosition {
    pub x: f64,
    pub y: i8,
    pub z: i8,
}

impl FromEvalVal for FashionShowResourcesPosition {
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

/// Sub-record for `config_fashion_show_resources.Rotation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FashionShowResourcesRotation {
    pub x: i8,
    pub y: u8,
    pub z: u8,
}

impl FromEvalVal for FashionShowResourcesRotation {
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

/// Sub-record for `config_fashion_show_resources.Scale`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FashionShowResourcesScale {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl FromEvalVal for FashionShowResourcesScale {
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

/// Row record for `config_fashion_show_resources.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigFashionShowResourcesRow {
    pub id: i32,
    pub material_res: Box<str>,
    pub position: FashionShowResourcesPosition,
    pub rotation: FashionShowResourcesRotation,
    pub scale: FashionShowResourcesScale,
    pub show_action: Box<str>,
    pub show_action2: Box<str>,
    pub show_action3: Box<str>,
    pub skeletal_res: Box<str>,
    pub static_mesh_res: Box<str>,
}

impl DataCfgRow for ConfigFashionShowResourcesRow {
    const TABLE_NAME: &'static str = "config_fashion_show_resources";

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
            b"Position" => self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?,
            b"Rotation" => self.rotation = ctx.decode_field(val, Self::TABLE_NAME, "Rotation")?,
            b"Scale" => self.scale = ctx.decode_field(val, Self::TABLE_NAME, "Scale")?,
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
            b"StaticMeshRes" => {
                self.static_mesh_res = ctx.decode_field(val, Self::TABLE_NAME, "StaticMeshRes")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_fashion_show_resources.out`.
pub type ConfigFashionShowResourcesTable = DataTable<u16, ConfigFashionShowResourcesRow>;

/// Parses `config_fashion_show_resources.out` into a [`ConfigFashionShowResourcesTable`].
pub fn parse_config_fashion_show_resources(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigFashionShowResourcesTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_fashion_suit.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFashionSuitRow {
    pub des: Box<str>,
    pub fashion_id: [u8; 0],
    pub icon: Box<str>,
    pub id: i8,
    pub name: Box<str>,
}

impl DataCfgRow for ConfigFashionSuitRow {
    const TABLE_NAME: &'static str = "config_fashion_suit";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"FashionId" => {
                self.fashion_id = ctx.decode_field(val, Self::TABLE_NAME, "FashionId")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_fashion_suit.out`.
pub type ConfigFashionSuitTable = DataTable<u32, ConfigFashionSuitRow>;

/// Parses `config_fashion_suit.out` into a [`ConfigFashionSuitTable`].
pub fn parse_config_fashion_suit(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigFashionSuitTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_fashion_type.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFashionTypeRow {
    pub default_icon: Box<str>,
    pub default_name: Box<str>,
    pub group_id: u8,
    pub group_name: Box<str>,
    pub group_order: u8,
    pub id: i8,
    pub is_hide: u8,
    pub name: Box<str>,
    pub _default_name: Box<str>,
    pub _group_name: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigFashionTypeRow {
    const TABLE_NAME: &'static str = "config_fashion_type";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DefaultIcon" => {
                self.default_icon = ctx.decode_field(val, Self::TABLE_NAME, "DefaultIcon")?
            }
            b"DefaultName" => {
                self.default_name = ctx.decode_field(val, Self::TABLE_NAME, "DefaultName")?
            }
            b"GroupId" => self.group_id = ctx.decode_field(val, Self::TABLE_NAME, "GroupId")?,
            b"GroupName" => {
                self.group_name = ctx.decode_field(val, Self::TABLE_NAME, "GroupName")?
            }
            b"GroupOrder" => {
                self.group_order = ctx.decode_field(val, Self::TABLE_NAME, "GroupOrder")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsHide" => self.is_hide = ctx.decode_field(val, Self::TABLE_NAME, "IsHide")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"_DefaultName" => {
                self._default_name = ctx.decode_field(val, Self::TABLE_NAME, "_DefaultName")?
            }
            b"_GroupName" => {
                self._group_name = ctx.decode_field(val, Self::TABLE_NAME, "_GroupName")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_fashion_type.out`.
pub type ConfigFashionTypeTable = DataTable<u8, ConfigFashionTypeRow>;

/// Parses `config_fashion_type.out` into a [`ConfigFashionTypeTable`].
pub fn parse_config_fashion_type(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigFashionTypeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

pub(crate) fn verify_fashion_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_fashion_show_resources" => {
            Some(parse_config_fashion_show_resources(buf, scratch).map(|t| t.len()))
        }
        "config_fashion_suit" => Some(parse_config_fashion_suit(buf, scratch).map(|t| t.len())),
        "config_fashion_type" => Some(parse_config_fashion_type(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}

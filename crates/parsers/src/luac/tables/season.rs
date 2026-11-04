//! Auto-generated `LuaTable/DataCfg` types and parsers for the `season` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;

/// Row record for `config_enum_gameplay_season_plague.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEnumGameplaySeasonPlagueRow {
    pub des: Box<str>,
    pub r#enum: Box<str>,
    pub icon: Box<str>,
    pub id: i16,
    pub _des: Box<str>,
}

impl DataCfgRow for ConfigEnumGameplaySeasonPlagueRow {
    const TABLE_NAME: &'static str = "config_enum_gameplay_season_plague";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Enum" => self.r#enum = ctx.decode_field(val, Self::TABLE_NAME, "Enum")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_enum_gameplay_season_plague.out`.
pub type ConfigEnumGameplaySeasonPlagueTable = DataTable<u16, ConfigEnumGameplaySeasonPlagueRow>;

/// Parses `config_enum_gameplay_season_plague.out` into a [`ConfigEnumGameplaySeasonPlagueTable`].
pub fn parse_config_enum_gameplay_season_plague(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEnumGameplaySeasonPlagueTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_enum_season_tycoon.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigEnumSeasonTycoonRow {
    pub des: Box<str>,
    pub r#enum: Box<str>,
    pub id: i16,
    pub _des: Box<str>,
}

impl DataCfgRow for ConfigEnumSeasonTycoonRow {
    const TABLE_NAME: &'static str = "config_enum_season_tycoon";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Enum" => self.r#enum = ctx.decode_field(val, Self::TABLE_NAME, "Enum")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_enum_season_tycoon.out`.
pub type ConfigEnumSeasonTycoonTable = DataTable<u16, ConfigEnumSeasonTycoonRow>;

/// Parses `config_enum_season_tycoon.out` into a [`ConfigEnumSeasonTycoonTable`].
pub fn parse_config_enum_season_tycoon(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigEnumSeasonTycoonTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s11_gamble.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GambleRow {
    pub cost: FxHashMap<u16, u8>,
    pub id: i16,
    pub order: u8,
    pub win_chance: u8,
}

impl DataCfgRow for ConfigS11GambleRow {
    const TABLE_NAME: &'static str = "config_s11_gamble";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"WinChance" => {
                self.win_chance = ctx.decode_field(val, Self::TABLE_NAME, "WinChance")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s11_gamble.out`.
pub type ConfigS11GambleTable = DataTable<u16, ConfigS11GambleRow>;

/// Parses `config_s11_gamble.out` into a [`ConfigS11GambleTable`].
pub fn parse_config_s11_gamble(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS11GambleTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s11_gameplay_hero_anim.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GameplayHeroAnimRow {
    pub anim_set_id: u16,
    pub body_type: Box<str>,
    pub fashion_id: u16,
    pub hero_purchase_id: u16,
    pub id: i8,
    pub is_mermaid: bool,
}

impl DataCfgRow for ConfigS11GameplayHeroAnimRow {
    const TABLE_NAME: &'static str = "config_s11_gameplay_hero_anim";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AnimSetId" => {
                self.anim_set_id = ctx.decode_field(val, Self::TABLE_NAME, "AnimSetId")?
            }
            b"BodyType" => self.body_type = ctx.decode_field(val, Self::TABLE_NAME, "BodyType")?,
            b"FashionId" => {
                self.fashion_id = ctx.decode_field(val, Self::TABLE_NAME, "FashionId")?
            }
            b"HeroPurchaseId" => {
                self.hero_purchase_id = ctx.decode_field(val, Self::TABLE_NAME, "HeroPurchaseId")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsMermaid" => {
                self.is_mermaid = ctx.decode_field(val, Self::TABLE_NAME, "IsMermaid")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s11_gameplay_hero_anim.out`.
pub type ConfigS11GameplayHeroAnimTable = DataTable<u8, ConfigS11GameplayHeroAnimRow>;

/// Parses `config_s11_gameplay_hero_anim.out` into a [`ConfigS11GameplayHeroAnimTable`].
pub fn parse_config_s11_gameplay_hero_anim(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS11GameplayHeroAnimTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_s12gameplay_talenttree.Function`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct S12gameplayTalenttreeFunctionEntry {
    pub kind: u8,
    pub params: Box<[u16]>,
}

impl FromEvalVal for S12gameplayTalenttreeFunctionEntry {
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

/// Row record for `config_s12gameplay_talenttree.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS12gameplayTalenttreeRow {
    pub desc: Box<str>,
    pub flavour_text: Box<str>,
    pub function: Box<[S12gameplayTalenttreeFunctionEntry]>,
    pub icon: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub position: u8,
    pub pre_node: Box<[u8]>,
    pub priority: u8,
    pub req_num: u8,
    pub _desc: Box<str>,
    pub _flavour_text: Box<str>,
    pub _name: Box<str>,
}

impl ConfigS12gameplayTalenttreeRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"FlavourText" => {
                self.flavour_text = ctx.decode_field(val, Self::TABLE_NAME, "FlavourText")?
            }
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Position" => self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?,
            b"PreNode" => self.pre_node = ctx.decode_field(val, Self::TABLE_NAME, "PreNode")?,
            b"Priority" => self.priority = ctx.decode_field(val, Self::TABLE_NAME, "Priority")?,
            b"ReqNum" => self.req_num = ctx.decode_field(val, Self::TABLE_NAME, "ReqNum")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_FlavourText" => {
                self._flavour_text = ctx.decode_field(val, Self::TABLE_NAME, "_FlavourText")?
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
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS12gameplayTalenttreeRow {
    const TABLE_NAME: &'static str = "config_s12gameplay_talenttree";

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

/// Typed `DataTable` for `config_s12gameplay_talenttree.out`.
pub type ConfigS12gameplayTalenttreeTable = DataTable<u8, ConfigS12gameplayTalenttreeRow>;

/// Parses `config_s12gameplay_talenttree.out` into a [`ConfigS12gameplayTalenttreeTable`].
pub fn parse_config_s12gameplay_talenttree(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS12gameplayTalenttreeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s15_gameplay_suprise.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS15GameplaySupriseRow {
    pub id: i8,
    pub max_count: u8,
    pub suprise_timing: u8,
    pub time: u8,
    pub time_suprise_hard_pity: Box<[u8]>,
    pub time_suprise_id: u16,
    pub time_suprise_weight: u16,
}

impl DataCfgRow for ConfigS15GameplaySupriseRow {
    const TABLE_NAME: &'static str = "config_s15_gameplay_suprise";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MaxCount" => self.max_count = ctx.decode_field(val, Self::TABLE_NAME, "MaxCount")?,
            b"SupriseTiming" => {
                self.suprise_timing = ctx.decode_field(val, Self::TABLE_NAME, "SupriseTiming")?
            }
            b"Time" => self.time = ctx.decode_field(val, Self::TABLE_NAME, "Time")?,
            b"TimeSupriseHardPity" => {
                self.time_suprise_hard_pity =
                    ctx.decode_field(val, Self::TABLE_NAME, "TimeSupriseHardPity")?
            }
            b"TimeSupriseId" => {
                self.time_suprise_id = ctx.decode_field(val, Self::TABLE_NAME, "TimeSupriseId")?
            }
            b"TimeSupriseWeight" => {
                self.time_suprise_weight =
                    ctx.decode_field(val, Self::TABLE_NAME, "TimeSupriseWeight")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s15_gameplay_suprise.out`.
pub type ConfigS15GameplaySupriseTable = DataTable<u8, ConfigS15GameplaySupriseRow>;

/// Parses `config_s15_gameplay_suprise.out` into a [`ConfigS15GameplaySupriseTable`].
pub fn parse_config_s15_gameplay_suprise(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS15GameplaySupriseTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s15_gameplay_surprise_info.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS15GameplaySurpriseInfoRow {
    pub banner_des: Box<str>,
    pub banner_icon: Box<str>,
    pub banner_name: Box<str>,
    pub banner_type: u8,
    pub id: i16,
    pub _banner_des: Box<str>,
    pub _banner_name: Box<str>,
}

impl DataCfgRow for ConfigS15GameplaySurpriseInfoRow {
    const TABLE_NAME: &'static str = "config_s15_gameplay_surprise_info";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BannerDes" => {
                self.banner_des = ctx.decode_field(val, Self::TABLE_NAME, "BannerDes")?
            }
            b"BannerIcon" => {
                self.banner_icon = ctx.decode_field(val, Self::TABLE_NAME, "BannerIcon")?
            }
            b"BannerName" => {
                self.banner_name = ctx.decode_field(val, Self::TABLE_NAME, "BannerName")?
            }
            b"BannerType" => {
                self.banner_type = ctx.decode_field(val, Self::TABLE_NAME, "BannerType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"_BannerDes" => {
                self._banner_des = ctx.decode_field(val, Self::TABLE_NAME, "_BannerDes")?
            }
            b"_BannerName" => {
                self._banner_name = ctx.decode_field(val, Self::TABLE_NAME, "_BannerName")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s15_gameplay_surprise_info.out`.
pub type ConfigS15GameplaySurpriseInfoTable = DataTable<u16, ConfigS15GameplaySurpriseInfoRow>;

/// Parses `config_s15_gameplay_surprise_info.out` into a [`ConfigS15GameplaySurpriseInfoTable`].
pub fn parse_config_s15_gameplay_surprise_info(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS15GameplaySurpriseInfoTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_s15_gameplay_talenttree.Function`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct S15GameplayTalenttreeFunctionEntry {
    pub kind: u8,
    pub params: Box<[u32]>,
}

impl FromEvalVal for S15GameplayTalenttreeFunctionEntry {
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

/// Row record for `config_s15_gameplay_talenttree.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS15GameplayTalenttreeRow {
    pub desc: Box<str>,
    pub flavour_text: Box<str>,
    pub function: Box<[S15GameplayTalenttreeFunctionEntry]>,
    pub icon: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub position: u16,
    pub pre_node: Box<[u8]>,
    pub _desc: Box<str>,
    pub _flavour_text: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigS15GameplayTalenttreeRow {
    const TABLE_NAME: &'static str = "config_s15_gameplay_talenttree";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"FlavourText" => {
                self.flavour_text = ctx.decode_field(val, Self::TABLE_NAME, "FlavourText")?
            }
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Position" => self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?,
            b"PreNode" => self.pre_node = ctx.decode_field(val, Self::TABLE_NAME, "PreNode")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_FlavourText" => {
                self._flavour_text = ctx.decode_field(val, Self::TABLE_NAME, "_FlavourText")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s15_gameplay_talenttree.out`.
pub type ConfigS15GameplayTalenttreeTable = DataTable<u8, ConfigS15GameplayTalenttreeRow>;

/// Parses `config_s15_gameplay_talenttree.out` into a [`ConfigS15GameplayTalenttreeTable`].
pub fn parse_config_s15_gameplay_talenttree(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS15GameplayTalenttreeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s15_gameplay_theme.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS15GameplayThemeRow {
    pub id: i8,
    pub theme_weight: FxHashMap<u16, u16>,
    pub time: u8,
    pub time_theme_hard_pity: Box<[u8]>,
    pub time_theme_weight: u16,
}

impl DataCfgRow for ConfigS15GameplayThemeRow {
    const TABLE_NAME: &'static str = "config_s15_gameplay_theme";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ThemeWeight" => {
                self.theme_weight = ctx.decode_field(val, Self::TABLE_NAME, "ThemeWeight")?
            }
            b"Time" => self.time = ctx.decode_field(val, Self::TABLE_NAME, "Time")?,
            b"TimeThemeHardPity" => {
                self.time_theme_hard_pity =
                    ctx.decode_field(val, Self::TABLE_NAME, "TimeThemeHardPity")?
            }
            b"TimeThemeWeight" => {
                self.time_theme_weight =
                    ctx.decode_field(val, Self::TABLE_NAME, "TimeThemeWeight")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s15_gameplay_theme.out`.
pub type ConfigS15GameplayThemeTable = DataTable<u8, ConfigS15GameplayThemeRow>;

/// Parses `config_s15_gameplay_theme.out` into a [`ConfigS15GameplayThemeTable`].
pub fn parse_config_s15_gameplay_theme(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS15GameplayThemeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s15_gameplay_theme_info.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS15GameplayThemeInfoRow {
    pub id: i16,
    pub main_window_desc: Box<str>,
    pub main_window_flavour_text: Box<str>,
    pub main_window_sprite: Box<str>,
    pub main_window_type: u8,
    pub show_progress: u8,
    pub show_side_window: u8,
    pub side_window_des: Box<str>,
    pub side_window_icon: Box<str>,
    pub special_icon: Box<str>,
    pub theme_name: Box<str>,
    pub theme_quest_des: Box<str>,
    pub theme_quest_progress: Box<str>,
    pub _main_window_desc: Box<str>,
    pub _main_window_flavour_text: Box<str>,
    pub _side_window_des: Box<str>,
    pub _theme_name: Box<str>,
    pub _theme_quest_des: Box<str>,
    pub _theme_quest_progress: Box<str>,
}

impl ConfigS15GameplayThemeInfoRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MainWindowDesc" => {
                self.main_window_desc = ctx.decode_field(val, Self::TABLE_NAME, "MainWindowDesc")?
            }
            b"MainWindowFlavourText" => {
                self.main_window_flavour_text =
                    ctx.decode_field(val, Self::TABLE_NAME, "MainWindowFlavourText")?
            }
            b"MainWindowSprite" => {
                self.main_window_sprite =
                    ctx.decode_field(val, Self::TABLE_NAME, "MainWindowSprite")?
            }
            b"MainWindowType" => {
                self.main_window_type = ctx.decode_field(val, Self::TABLE_NAME, "MainWindowType")?
            }
            b"ShowProgress" => {
                self.show_progress = ctx.decode_field(val, Self::TABLE_NAME, "ShowProgress")?
            }
            b"ShowSideWindow" => {
                self.show_side_window = ctx.decode_field(val, Self::TABLE_NAME, "ShowSideWindow")?
            }
            b"SideWindowDes" => {
                self.side_window_des = ctx.decode_field(val, Self::TABLE_NAME, "SideWindowDes")?
            }
            b"SideWindowIcon" => {
                self.side_window_icon = ctx.decode_field(val, Self::TABLE_NAME, "SideWindowIcon")?
            }
            b"SpecialIcon" => {
                self.special_icon = ctx.decode_field(val, Self::TABLE_NAME, "SpecialIcon")?
            }
            b"ThemeName" => {
                self.theme_name = ctx.decode_field(val, Self::TABLE_NAME, "ThemeName")?
            }
            b"ThemeQuestDes" => {
                self.theme_quest_des = ctx.decode_field(val, Self::TABLE_NAME, "ThemeQuestDes")?
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
            b"ThemeQuestProgress" => {
                self.theme_quest_progress =
                    ctx.decode_field(val, Self::TABLE_NAME, "ThemeQuestProgress")?
            }
            b"_MainWindowDesc" => {
                self._main_window_desc =
                    ctx.decode_field(val, Self::TABLE_NAME, "_MainWindowDesc")?
            }
            b"_MainWindowFlavourText" => {
                self._main_window_flavour_text =
                    ctx.decode_field(val, Self::TABLE_NAME, "_MainWindowFlavourText")?
            }
            b"_SideWindowDes" => {
                self._side_window_des = ctx.decode_field(val, Self::TABLE_NAME, "_SideWindowDes")?
            }
            b"_ThemeName" => {
                self._theme_name = ctx.decode_field(val, Self::TABLE_NAME, "_ThemeName")?
            }
            b"_ThemeQuestDes" => {
                self._theme_quest_des = ctx.decode_field(val, Self::TABLE_NAME, "_ThemeQuestDes")?
            }
            b"_ThemeQuestProgress" => {
                self._theme_quest_progress =
                    ctx.decode_field(val, Self::TABLE_NAME, "_ThemeQuestProgress")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS15GameplayThemeInfoRow {
    const TABLE_NAME: &'static str = "config_s15_gameplay_theme_info";

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

/// Typed `DataTable` for `config_s15_gameplay_theme_info.out`.
pub type ConfigS15GameplayThemeInfoTable = DataTable<u16, ConfigS15GameplayThemeInfoRow>;

/// Parses `config_s15_gameplay_theme_info.out` into a [`ConfigS15GameplayThemeInfoTable`].
pub fn parse_config_s15_gameplay_theme_info(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS15GameplayThemeInfoTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s16_gameplay_attribute.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS16GameplayAttributeRow {
    pub can_be_negative: u8,
    pub id: i8,
    pub init_value: u8,
    pub limit: u8,
    pub param1: Box<str>,
    pub param2: Box<str>,
}

impl DataCfgRow for ConfigS16GameplayAttributeRow {
    const TABLE_NAME: &'static str = "config_s16_gameplay_attribute";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CanBeNegative" => {
                self.can_be_negative = ctx.decode_field(val, Self::TABLE_NAME, "CanBeNegative")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InitValue" => {
                self.init_value = ctx.decode_field(val, Self::TABLE_NAME, "InitValue")?
            }
            b"Limit" => self.limit = ctx.decode_field(val, Self::TABLE_NAME, "Limit")?,
            b"Param1" => self.param1 = ctx.decode_field(val, Self::TABLE_NAME, "Param1")?,
            b"Param2" => self.param2 = ctx.decode_field(val, Self::TABLE_NAME, "Param2")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s16_gameplay_attribute.out`.
pub type ConfigS16GameplayAttributeTable = DataTable<u8, ConfigS16GameplayAttributeRow>;

/// Parses `config_s16_gameplay_attribute.out` into a [`ConfigS16GameplayAttributeTable`].
pub fn parse_config_s16_gameplay_attribute(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS16GameplayAttributeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s16_gameplay_colored_reward.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS16GameplayColoredRewardRow {
    pub effect_id: u8,
    pub id: i8,
    pub name: Box<str>,
    pub pic: Box<str>,
    pub reward_type: u16,
    pub text: Box<str>,
    pub text_param: Box<str>,
    pub _name: Box<str>,
    pub _text: Box<str>,
}

impl DataCfgRow for ConfigS16GameplayColoredRewardRow {
    const TABLE_NAME: &'static str = "config_s16_gameplay_colored_reward";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"EffectId" => self.effect_id = ctx.decode_field(val, Self::TABLE_NAME, "EffectId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Pic" => self.pic = ctx.decode_field(val, Self::TABLE_NAME, "Pic")?,
            b"RewardType" => {
                self.reward_type = ctx.decode_field(val, Self::TABLE_NAME, "RewardType")?
            }
            b"Text" => self.text = ctx.decode_field(val, Self::TABLE_NAME, "Text")?,
            b"TextParam" => {
                self.text_param = ctx.decode_field(val, Self::TABLE_NAME, "TextParam")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_Text" => self._text = ctx.decode_field(val, Self::TABLE_NAME, "_Text")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s16_gameplay_colored_reward.out`.
pub type ConfigS16GameplayColoredRewardTable = DataTable<u8, ConfigS16GameplayColoredRewardRow>;

/// Parses `config_s16_gameplay_colored_reward.out` into a [`ConfigS16GameplayColoredRewardTable`].
pub fn parse_config_s16_gameplay_colored_reward(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS16GameplayColoredRewardTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s16_gameplay_enum.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS16GameplayEnumRow {
    pub r#enum: Box<str>,
    pub id: i16,
}

impl DataCfgRow for ConfigS16GameplayEnumRow {
    const TABLE_NAME: &'static str = "config_s16_gameplay_enum";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Enum" => self.r#enum = ctx.decode_field(val, Self::TABLE_NAME, "Enum")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s16_gameplay_enum.out`.
pub type ConfigS16GameplayEnumTable = DataTable<u16, ConfigS16GameplayEnumRow>;

/// Parses `config_s16_gameplay_enum.out` into a [`ConfigS16GameplayEnumTable`].
pub fn parse_config_s16_gameplay_enum(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS16GameplayEnumTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s16_gameplay_event.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS16GameplayEventRow {
    pub desc: Box<str>,
    pub desc_story: Box<str>,
    pub effect_id: u8,
    pub event_type: u16,
    pub exclude_relic: Box<[u8]>,
    pub id: i16,
    pub name: Box<str>,
    pub pic: Box<str>,
    pub repeat_limit: Box<str>,
    pub sp_sound: Box<str>,
    pub spawn_cell_state: u16,
    pub unlock_param: u8,
    pub unlock_type: u16,
    pub weight: u8,
    pub _desc: Box<str>,
    pub _desc_story: Box<str>,
    pub _name: Box<str>,
}

impl ConfigS16GameplayEventRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DescStory" => {
                self.desc_story = ctx.decode_field(val, Self::TABLE_NAME, "DescStory")?
            }
            b"EffectId" => self.effect_id = ctx.decode_field(val, Self::TABLE_NAME, "EffectId")?,
            b"EventType" => {
                self.event_type = ctx.decode_field(val, Self::TABLE_NAME, "EventType")?
            }
            b"ExcludeRelic" => {
                self.exclude_relic = ctx.decode_field(val, Self::TABLE_NAME, "ExcludeRelic")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Pic" => self.pic = ctx.decode_field(val, Self::TABLE_NAME, "Pic")?,
            b"RepeatLimit" => {
                self.repeat_limit = ctx.decode_field(val, Self::TABLE_NAME, "RepeatLimit")?
            }
            b"SpSound" => self.sp_sound = ctx.decode_field(val, Self::TABLE_NAME, "SpSound")?,
            b"SpawnCellState" => {
                self.spawn_cell_state = ctx.decode_field(val, Self::TABLE_NAME, "SpawnCellState")?
            }
            b"UnlockParam" => {
                self.unlock_param = ctx.decode_field(val, Self::TABLE_NAME, "UnlockParam")?
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
            b"UnlockType" => {
                self.unlock_type = ctx.decode_field(val, Self::TABLE_NAME, "UnlockType")?
            }
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_DescStory" => {
                self._desc_story = ctx.decode_field(val, Self::TABLE_NAME, "_DescStory")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS16GameplayEventRow {
    const TABLE_NAME: &'static str = "config_s16_gameplay_event";

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

/// Typed `DataTable` for `config_s16_gameplay_event.out`.
pub type ConfigS16GameplayEventTable = DataTable<u8, ConfigS16GameplayEventRow>;

/// Parses `config_s16_gameplay_event.out` into a [`ConfigS16GameplayEventTable`].
pub fn parse_config_s16_gameplay_event(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS16GameplayEventTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s16_gameplay_log.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS16GameplayLogRow {
    pub condition_param: Box<str>,
    pub condition_type: u16,
    pub group: Box<str>,
    pub id: i8,
    pub text: Box<str>,
    pub text_param: Box<str>,
    pub trigger_node: u16,
    pub _text: Box<str>,
}

impl DataCfgRow for ConfigS16GameplayLogRow {
    const TABLE_NAME: &'static str = "config_s16_gameplay_log";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ConditionParam" => {
                self.condition_param = ctx.decode_field(val, Self::TABLE_NAME, "ConditionParam")?
            }
            b"ConditionType" => {
                self.condition_type = ctx.decode_field(val, Self::TABLE_NAME, "ConditionType")?
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Text" => self.text = ctx.decode_field(val, Self::TABLE_NAME, "Text")?,
            b"TextParam" => {
                self.text_param = ctx.decode_field(val, Self::TABLE_NAME, "TextParam")?
            }
            b"TriggerNode" => {
                self.trigger_node = ctx.decode_field(val, Self::TABLE_NAME, "TriggerNode")?
            }
            b"_Text" => self._text = ctx.decode_field(val, Self::TABLE_NAME, "_Text")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s16_gameplay_log.out`.
pub type ConfigS16GameplayLogTable = DataTable<u8, ConfigS16GameplayLogRow>;

/// Parses `config_s16_gameplay_log.out` into a [`ConfigS16GameplayLogTable`].
pub fn parse_config_s16_gameplay_log(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS16GameplayLogTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s16_gameplay_relic.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS16GameplayRelicRow {
    pub build_id: Box<[u8]>,
    pub build_weight_inc: u8,
    pub desc: Box<str>,
    pub desc_params: [u8; 0],
    pub desc_story: Box<str>,
    pub effect_id: u16,
    pub exclude: Box<[u16]>,
    pub id: i16,
    pub max_obtain: u8,
    pub name: Box<str>,
    pub one_turn_trigger_limit: u8,
    pub pic: Box<str>,
    pub pic_icon_sp: Box<str>,
    pub priority: u16,
    pub relic_type: u16,
    pub total_trigger_limit: u8,
    pub unlock_param: u8,
    pub unlock_type: u16,
    pub weight: u8,
    pub _desc: Box<str>,
    pub _desc_story: Box<str>,
    pub _name: Box<str>,
}

impl ConfigS16GameplayRelicRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"BuildID" => self.build_id = ctx.decode_field(val, Self::TABLE_NAME, "BuildID")?,
            b"BuildWeightInc" => {
                self.build_weight_inc = ctx.decode_field(val, Self::TABLE_NAME, "BuildWeightInc")?
            }
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DescParams" => {
                self.desc_params = ctx.decode_field(val, Self::TABLE_NAME, "DescParams")?
            }
            b"DescStory" => {
                self.desc_story = ctx.decode_field(val, Self::TABLE_NAME, "DescStory")?
            }
            b"EffectId" => self.effect_id = ctx.decode_field(val, Self::TABLE_NAME, "EffectId")?,
            b"Exclude" => self.exclude = ctx.decode_field(val, Self::TABLE_NAME, "Exclude")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MaxObtain" => {
                self.max_obtain = ctx.decode_field(val, Self::TABLE_NAME, "MaxObtain")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"OneTurnTriggerLimit" => {
                self.one_turn_trigger_limit =
                    ctx.decode_field(val, Self::TABLE_NAME, "OneTurnTriggerLimit")?
            }
            b"Pic" => self.pic = ctx.decode_field(val, Self::TABLE_NAME, "Pic")?,
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
            b"PicIconSP" => {
                self.pic_icon_sp = ctx.decode_field(val, Self::TABLE_NAME, "PicIconSP")?
            }
            b"Priority" => self.priority = ctx.decode_field(val, Self::TABLE_NAME, "Priority")?,
            b"RelicType" => {
                self.relic_type = ctx.decode_field(val, Self::TABLE_NAME, "RelicType")?
            }
            b"TotalTriggerLimit" => {
                self.total_trigger_limit =
                    ctx.decode_field(val, Self::TABLE_NAME, "TotalTriggerLimit")?
            }
            b"UnlockParam" => {
                self.unlock_param = ctx.decode_field(val, Self::TABLE_NAME, "UnlockParam")?
            }
            b"UnlockType" => {
                self.unlock_type = ctx.decode_field(val, Self::TABLE_NAME, "UnlockType")?
            }
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_DescStory" => {
                self._desc_story = ctx.decode_field(val, Self::TABLE_NAME, "_DescStory")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS16GameplayRelicRow {
    const TABLE_NAME: &'static str = "config_s16_gameplay_relic";

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

/// Typed `DataTable` for `config_s16_gameplay_relic.out`.
pub type ConfigS16GameplayRelicTable = DataTable<u16, ConfigS16GameplayRelicRow>;

/// Parses `config_s16_gameplay_relic.out` into a [`ConfigS16GameplayRelicTable`].
pub fn parse_config_s16_gameplay_relic(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS16GameplayRelicTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s16_gameplay_task.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS16GameplayTaskRow {
    pub desc: Box<str>,
    pub desc_story: Box<str>,
    pub effect_id: u8,
    pub id: i8,
    pub name: Box<str>,
    pub progress_param: u8,
    pub task_type: u16,
    pub unlock_param: u8,
    pub unlock_type: u16,
    pub weight: u8,
    pub _desc: Box<str>,
    pub _desc_story: Box<str>,
    pub _name: Box<str>,
}

impl ConfigS16GameplayTaskRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DescStory" => {
                self.desc_story = ctx.decode_field(val, Self::TABLE_NAME, "DescStory")?
            }
            b"EffectId" => self.effect_id = ctx.decode_field(val, Self::TABLE_NAME, "EffectId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"ProgressParam" => {
                self.progress_param = ctx.decode_field(val, Self::TABLE_NAME, "ProgressParam")?
            }
            b"TaskType" => self.task_type = ctx.decode_field(val, Self::TABLE_NAME, "TaskType")?,
            b"UnlockParam" => {
                self.unlock_param = ctx.decode_field(val, Self::TABLE_NAME, "UnlockParam")?
            }
            b"UnlockType" => {
                self.unlock_type = ctx.decode_field(val, Self::TABLE_NAME, "UnlockType")?
            }
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_DescStory" => {
                self._desc_story = ctx.decode_field(val, Self::TABLE_NAME, "_DescStory")?
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
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS16GameplayTaskRow {
    const TABLE_NAME: &'static str = "config_s16_gameplay_task";

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

/// Typed `DataTable` for `config_s16_gameplay_task.out`.
pub type ConfigS16GameplayTaskTable = DataTable<u8, ConfigS16GameplayTaskRow>;

/// Parses `config_s16_gameplay_task.out` into a [`ConfigS16GameplayTaskTable`].
pub fn parse_config_s16_gameplay_task(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS16GameplayTaskTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s5_baddream_affix.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS5BaddreamAffixRow {
    pub desc: Box<str>,
    pub icon: Box<str>,
    pub id: i16,
    pub t_level: u8,
    pub _desc: Box<str>,
}

impl DataCfgRow for ConfigS5BaddreamAffixRow {
    const TABLE_NAME: &'static str = "config_s5_baddream_affix";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"TLevel" => self.t_level = ctx.decode_field(val, Self::TABLE_NAME, "TLevel")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s5_baddream_affix.out`.
pub type ConfigS5BaddreamAffixTable = DataTable<u8, ConfigS5BaddreamAffixRow>;

/// Parses `config_s5_baddream_affix.out` into a [`ConfigS5BaddreamAffixTable`].
pub fn parse_config_s5_baddream_affix(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS5BaddreamAffixTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_s5_gooddream_affix.AreaLevelRange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct S5GooddreamAffixAreaLevelRange {
    pub min: u8,
    pub max: u8,
}

impl FromEvalVal for S5GooddreamAffixAreaLevelRange {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (min, max) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { min, max })
    }
}

/// Entry for `config_s5_gooddream_affix.DreamLevelRange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct S5GooddreamAffixDreamLevelRange {
    pub min: u8,
    pub max: u8,
}

impl FromEvalVal for S5GooddreamAffixDreamLevelRange {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (min, max) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { min, max })
    }
}

/// Entry for `config_s5_gooddream_affix.Group`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct S5GooddreamAffixGroup {
    pub kind: u8,
    pub params: Box<[u8]>,
}

impl FromEvalVal for S5GooddreamAffixGroup {
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

/// Row record for `config_s5_gooddream_affix.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS5GooddreamAffixRow {
    pub area_level_range: S5GooddreamAffixAreaLevelRange,
    pub desc: Box<str>,
    pub dream_level_range: S5GooddreamAffixDreamLevelRange,
    pub group: S5GooddreamAffixGroup,
    pub hint_id: u32,
    pub icon: Box<str>,
    pub id: i16,
    pub is_base: bool,
    pub t_level: u8,
    pub tag: Box<[u8]>,
    pub _desc: Box<str>,
}

impl DataCfgRow for ConfigS5GooddreamAffixRow {
    const TABLE_NAME: &'static str = "config_s5_gooddream_affix";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AreaLevelRange" => {
                self.area_level_range = ctx.decode_field(val, Self::TABLE_NAME, "AreaLevelRange")?
            }
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DreamLevelRange" => {
                self.dream_level_range =
                    ctx.decode_field(val, Self::TABLE_NAME, "DreamLevelRange")?
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"HintId" => self.hint_id = ctx.decode_field(val, Self::TABLE_NAME, "HintId")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsBase" => self.is_base = ctx.decode_field(val, Self::TABLE_NAME, "IsBase")?,
            b"TLevel" => self.t_level = ctx.decode_field(val, Self::TABLE_NAME, "TLevel")?,
            b"Tag" => self.tag = ctx.decode_field(val, Self::TABLE_NAME, "Tag")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s5_gooddream_affix.out`.
pub type ConfigS5GooddreamAffixTable = DataTable<u16, ConfigS5GooddreamAffixRow>;

/// Parses `config_s5_gooddream_affix.out` into a [`ConfigS5GooddreamAffixTable`].
pub fn parse_config_s5_gooddream_affix(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS5GooddreamAffixTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s5_gooddream_affix_tips_hint.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS5GooddreamAffixTipsHintRow {
    pub id: i32,
    pub tips_hint: Box<str>,
    pub _tips_hint: Box<str>,
}

impl DataCfgRow for ConfigS5GooddreamAffixTipsHintRow {
    const TABLE_NAME: &'static str = "config_s5_gooddream_affix_tips_hint";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"TipsHint" => self.tips_hint = ctx.decode_field(val, Self::TABLE_NAME, "TipsHint")?,
            b"_TipsHint" => {
                self._tips_hint = ctx.decode_field(val, Self::TABLE_NAME, "_TipsHint")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s5_gooddream_affix_tips_hint.out`.
pub type ConfigS5GooddreamAffixTipsHintTable = DataTable<u32, ConfigS5GooddreamAffixTipsHintRow>;

/// Parses `config_s5_gooddream_affix_tips_hint.out` into a [`ConfigS5GooddreamAffixTipsHintTable`].
pub fn parse_config_s5_gooddream_affix_tips_hint(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS5GooddreamAffixTipsHintTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s5_gooddream_reward_quality_ui.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS5GooddreamRewardQualityUiRow {
    pub icon: Box<str>,
    pub id: i8,
}

impl DataCfgRow for ConfigS5GooddreamRewardQualityUiRow {
    const TABLE_NAME: &'static str = "config_s5_gooddream_reward_quality_ui";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s5_gooddream_reward_quality_ui.out`.
pub type ConfigS5GooddreamRewardQualityUiTable = DataTable<u8, ConfigS5GooddreamRewardQualityUiRow>;

/// Parses `config_s5_gooddream_reward_quality_ui.out` into a [`ConfigS5GooddreamRewardQualityUiTable`].
pub fn parse_config_s5_gooddream_reward_quality_ui(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS5GooddreamRewardQualityUiTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s5_gooddream_reward_type.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS5GooddreamRewardTypeRow {
    pub icon: Box<str>,
    pub id: i8,
    pub name: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigS5GooddreamRewardTypeRow {
    const TABLE_NAME: &'static str = "config_s5_gooddream_reward_type";

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

/// Typed `DataTable` for `config_s5_gooddream_reward_type.out`.
pub type ConfigS5GooddreamRewardTypeTable = DataTable<u8, ConfigS5GooddreamRewardTypeRow>;

/// Parses `config_s5_gooddream_reward_type.out` into a [`ConfigS5GooddreamRewardTypeTable`].
pub fn parse_config_s5_gooddream_reward_type(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS5GooddreamRewardTypeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_attribute.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayAttributeRow {
    pub can_be_negative: u8,
    pub id: i8,
    pub init_value: u8,
    pub limit: u8,
    pub param1: Box<str>,
    pub param2: Box<str>,
}

impl DataCfgRow for ConfigS6GameplayAttributeRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_attribute";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CanBeNegative" => {
                self.can_be_negative = ctx.decode_field(val, Self::TABLE_NAME, "CanBeNegative")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InitValue" => {
                self.init_value = ctx.decode_field(val, Self::TABLE_NAME, "InitValue")?
            }
            b"Limit" => self.limit = ctx.decode_field(val, Self::TABLE_NAME, "Limit")?,
            b"Param1" => self.param1 = ctx.decode_field(val, Self::TABLE_NAME, "Param1")?,
            b"Param2" => self.param2 = ctx.decode_field(val, Self::TABLE_NAME, "Param2")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s6_gameplay_attribute.out`.
pub type ConfigS6GameplayAttributeTable = DataTable<u8, ConfigS6GameplayAttributeRow>;

/// Parses `config_s6_gameplay_attribute.out` into a [`ConfigS6GameplayAttributeTable`].
pub fn parse_config_s6_gameplay_attribute(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayAttributeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_buff.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayBuffRow {
    pub add_event: Box<[u32]>,
    pub count: u8,
    pub desc: Box<str>,
    pub event: Box<[u32]>,
    pub id: i16,
    pub prob: u8,
    pub remove_event: Box<[u32]>,
    pub trigger: Box<[u16]>,
    pub _desc: Box<str>,
}

impl DataCfgRow for ConfigS6GameplayBuffRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_buff";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AddEvent" => self.add_event = ctx.decode_field(val, Self::TABLE_NAME, "AddEvent")?,
            b"Count" => self.count = ctx.decode_field(val, Self::TABLE_NAME, "Count")?,
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Event" => self.event = ctx.decode_field(val, Self::TABLE_NAME, "Event")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Prob" => self.prob = ctx.decode_field(val, Self::TABLE_NAME, "Prob")?,
            b"RemoveEvent" => {
                self.remove_event = ctx.decode_field(val, Self::TABLE_NAME, "RemoveEvent")?
            }
            b"Trigger" => self.trigger = ctx.decode_field(val, Self::TABLE_NAME, "Trigger")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s6_gameplay_buff.out`.
pub type ConfigS6GameplayBuffTable = DataTable<u16, ConfigS6GameplayBuffRow>;

/// Parses `config_s6_gameplay_buff.out` into a [`ConfigS6GameplayBuffTable`].
pub fn parse_config_s6_gameplay_buff(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayBuffTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_dungeon.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayDungeonRow {
    pub exploration_sanity: u8,
    pub id: i16,
    pub is_max_level: u8,
    pub level: Box<[u8]>,
    pub level_desc: Box<str>,
    pub level_difficulty: Box<[u8]>,
    pub sanity_consume_show: u8,
}

impl DataCfgRow for ConfigS6GameplayDungeonRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_dungeon";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ExplorationSanity" => {
                self.exploration_sanity =
                    ctx.decode_field(val, Self::TABLE_NAME, "ExplorationSanity")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsMaxLevel" => {
                self.is_max_level = ctx.decode_field(val, Self::TABLE_NAME, "IsMaxLevel")?
            }
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"LevelDesc" => {
                self.level_desc = ctx.decode_field(val, Self::TABLE_NAME, "LevelDesc")?
            }
            b"LevelDifficulty" => {
                self.level_difficulty =
                    ctx.decode_field(val, Self::TABLE_NAME, "LevelDifficulty")?
            }
            b"SanityConsumeShow" => {
                self.sanity_consume_show =
                    ctx.decode_field(val, Self::TABLE_NAME, "SanityConsumeShow")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s6_gameplay_dungeon.out`.
pub type ConfigS6GameplayDungeonTable = DataTable<u16, ConfigS6GameplayDungeonRow>;

/// Parses `config_s6_gameplay_dungeon.out` into a [`ConfigS6GameplayDungeonTable`].
pub fn parse_config_s6_gameplay_dungeon(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayDungeonTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_encounter.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayEncounterRow {
    pub desc: Box<str>,
    pub encounter_req: [u8; 0],
    pub exclusive_group: u8,
    pub group: u16,
    pub id: i16,
    pub pic: Box<str>,
    pub pre_encounter: Box<[u8]>,
    pub selection_color1: u8,
    pub selection_color2: u8,
    pub selection_color3: u8,
    pub selection_desc11: Box<str>,
    pub selection_desc12: Box<str>,
    pub selection_desc13: Box<str>,
    pub selection_desc21: Box<str>,
    pub selection_desc22: Box<str>,
    pub selection_desc23: Box<str>,
    pub selection_desc31: Box<str>,
    pub selection_desc32: Box<str>,
    pub selection_desc33: Box<str>,
    pub selection_encounter1: FxHashMap<u16, u8>,
    pub selection_encounter2: FxHashMap<u16, u8>,
    pub selection_encounter3: FxHashMap<u16, u8>,
    pub selection_event1: FxHashMap<u8, Box<[u32]>>,
    pub selection_event2: FxHashMap<u8, Box<[u32]>>,
    pub selection_event3: FxHashMap<u8, Box<[u32]>>,
    pub selection_req1: FxHashMap<u8, Box<[u8]>>,
    pub selection_req2: FxHashMap<u8, Box<[u8]>>,
    pub selection_req3: [u8; 0],
    pub selection_tips1: u8,
    pub selection_tips2: u8,
    pub selection_tips3: u8,
    pub title: Box<str>,
    pub unlock_req: Box<[u8]>,
    pub weight: u8,
    pub _desc: Box<str>,
    pub _selection_desc11: Box<str>,
    pub _selection_desc12: Box<str>,
    pub _selection_desc13: Box<str>,
    pub _selection_desc21: Box<str>,
    pub _selection_desc22: Box<str>,
    pub _selection_desc23: Box<str>,
    pub _selection_desc31: Box<str>,
    pub _selection_desc32: Box<str>,
    pub _selection_desc33: Box<str>,
    pub _title: Box<str>,
}

impl ConfigS6GameplayEncounterRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"EncounterReq" => {
                self.encounter_req = ctx.decode_field(val, Self::TABLE_NAME, "EncounterReq")?
            }
            b"ExclusiveGroup" => {
                self.exclusive_group = ctx.decode_field(val, Self::TABLE_NAME, "ExclusiveGroup")?
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Pic" => self.pic = ctx.decode_field(val, Self::TABLE_NAME, "Pic")?,
            b"PreEncounter" => {
                self.pre_encounter = ctx.decode_field(val, Self::TABLE_NAME, "PreEncounter")?
            }
            b"SelectionColor1" => {
                self.selection_color1 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionColor1")?
            }
            b"SelectionColor2" => {
                self.selection_color2 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionColor2")?
            }
            b"SelectionColor3" => {
                self.selection_color3 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionColor3")?
            }
            b"SelectionDesc11" => {
                self.selection_desc11 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionDesc11")?
            }
            b"SelectionDesc12" => {
                self.selection_desc12 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionDesc12")?
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
            b"SelectionDesc13" => {
                self.selection_desc13 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionDesc13")?
            }
            b"SelectionDesc21" => {
                self.selection_desc21 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionDesc21")?
            }
            b"SelectionDesc22" => {
                self.selection_desc22 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionDesc22")?
            }
            b"SelectionDesc23" => {
                self.selection_desc23 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionDesc23")?
            }
            b"SelectionDesc31" => {
                self.selection_desc31 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionDesc31")?
            }
            b"SelectionDesc32" => {
                self.selection_desc32 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionDesc32")?
            }
            b"SelectionDesc33" => {
                self.selection_desc33 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionDesc33")?
            }
            b"SelectionEncounter1" => {
                self.selection_encounter1 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionEncounter1")?
            }
            b"SelectionEncounter2" => {
                self.selection_encounter2 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionEncounter2")?
            }
            b"SelectionEncounter3" => {
                self.selection_encounter3 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionEncounter3")?
            }
            b"SelectionEvent1" => {
                self.selection_event1 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionEvent1")?
            }
            b"SelectionEvent2" => {
                self.selection_event2 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionEvent2")?
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
            b"SelectionEvent3" => {
                self.selection_event3 =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionEvent3")?
            }
            b"SelectionReq1" => {
                self.selection_req1 = ctx.decode_field(val, Self::TABLE_NAME, "SelectionReq1")?
            }
            b"SelectionReq2" => {
                self.selection_req2 = ctx.decode_field(val, Self::TABLE_NAME, "SelectionReq2")?
            }
            b"SelectionReq3" => {
                self.selection_req3 = ctx.decode_field(val, Self::TABLE_NAME, "SelectionReq3")?
            }
            b"SelectionTips1" => {
                self.selection_tips1 = ctx.decode_field(val, Self::TABLE_NAME, "SelectionTips1")?
            }
            b"SelectionTips2" => {
                self.selection_tips2 = ctx.decode_field(val, Self::TABLE_NAME, "SelectionTips2")?
            }
            b"SelectionTips3" => {
                self.selection_tips3 = ctx.decode_field(val, Self::TABLE_NAME, "SelectionTips3")?
            }
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_SelectionDesc11" => {
                self._selection_desc11 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDesc11")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_field_3(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"_SelectionDesc12" => {
                self._selection_desc12 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDesc12")?
            }
            b"_SelectionDesc13" => {
                self._selection_desc13 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDesc13")?
            }
            b"_SelectionDesc21" => {
                self._selection_desc21 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDesc21")?
            }
            b"_SelectionDesc22" => {
                self._selection_desc22 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDesc22")?
            }
            b"_SelectionDesc23" => {
                self._selection_desc23 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDesc23")?
            }
            b"_SelectionDesc31" => {
                self._selection_desc31 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDesc31")?
            }
            b"_SelectionDesc32" => {
                self._selection_desc32 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDesc32")?
            }
            b"_SelectionDesc33" => {
                self._selection_desc33 =
                    ctx.decode_field(val, Self::TABLE_NAME, "_SelectionDesc33")?
            }
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS6GameplayEncounterRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_encounter";

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

/// Typed `DataTable` for `config_s6_gameplay_encounter.out`.
pub type ConfigS6GameplayEncounterTable = DataTable<u16, ConfigS6GameplayEncounterRow>;

/// Parses `config_s6_gameplay_encounter.out` into a [`ConfigS6GameplayEncounterTable`].
pub fn parse_config_s6_gameplay_encounter(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayEncounterTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_event.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayEventRow {
    pub desc: Box<str>,
    pub function: u32,
    pub id: i32,
    pub p1: i32,
    pub p2: i32,
    pub p3: u8,
    pub p4: u8,
    pub p5: u8,
    pub p6: u8,
    pub sp1: Box<str>,
    pub sp2: Box<str>,
    pub sp3: Box<str>,
    pub sp4: Box<str>,
    pub sp5: Box<str>,
    pub sp6: Box<str>,
    pub tag: u8,
    pub _desc: Box<str>,
}

impl ConfigS6GameplayEventRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"P1" => self.p1 = ctx.decode_field(val, Self::TABLE_NAME, "P1")?,
            b"P2" => self.p2 = ctx.decode_field(val, Self::TABLE_NAME, "P2")?,
            b"P3" => self.p3 = ctx.decode_field(val, Self::TABLE_NAME, "P3")?,
            b"P4" => self.p4 = ctx.decode_field(val, Self::TABLE_NAME, "P4")?,
            b"P5" => self.p5 = ctx.decode_field(val, Self::TABLE_NAME, "P5")?,
            b"P6" => self.p6 = ctx.decode_field(val, Self::TABLE_NAME, "P6")?,
            b"Sp1" => self.sp1 = ctx.decode_field(val, Self::TABLE_NAME, "Sp1")?,
            b"Sp2" => self.sp2 = ctx.decode_field(val, Self::TABLE_NAME, "Sp2")?,
            b"Sp3" => self.sp3 = ctx.decode_field(val, Self::TABLE_NAME, "Sp3")?,
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
            b"Sp4" => self.sp4 = ctx.decode_field(val, Self::TABLE_NAME, "Sp4")?,
            b"Sp5" => self.sp5 = ctx.decode_field(val, Self::TABLE_NAME, "Sp5")?,
            b"Sp6" => self.sp6 = ctx.decode_field(val, Self::TABLE_NAME, "Sp6")?,
            b"Tag" => self.tag = ctx.decode_field(val, Self::TABLE_NAME, "Tag")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS6GameplayEventRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_event";

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

/// Typed `DataTable` for `config_s6_gameplay_event.out`.
pub type ConfigS6GameplayEventTable = DataTable<u32, ConfigS6GameplayEventRow>;

/// Parses `config_s6_gameplay_event.out` into a [`ConfigS6GameplayEventTable`].
pub fn parse_config_s6_gameplay_event(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayEventTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_interaction.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayInteractionRow {
    pub desc: Box<str>,
    pub function: u16,
    pub id: i16,
    pub p1: u8,
    pub p2: u32,
    pub p3: u8,
    pub p4: u8,
    pub p5: u8,
    pub p6: u8,
    pub sp1: Box<str>,
    pub sp2: Box<str>,
    pub sp3: Box<str>,
    pub sp4: Box<str>,
    pub sp5: Box<str>,
    pub sp6: Box<str>,
    pub _desc: Box<str>,
}

impl ConfigS6GameplayInteractionRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"P1" => self.p1 = ctx.decode_field(val, Self::TABLE_NAME, "P1")?,
            b"P2" => self.p2 = ctx.decode_field(val, Self::TABLE_NAME, "P2")?,
            b"P3" => self.p3 = ctx.decode_field(val, Self::TABLE_NAME, "P3")?,
            b"P4" => self.p4 = ctx.decode_field(val, Self::TABLE_NAME, "P4")?,
            b"P5" => self.p5 = ctx.decode_field(val, Self::TABLE_NAME, "P5")?,
            b"P6" => self.p6 = ctx.decode_field(val, Self::TABLE_NAME, "P6")?,
            b"Sp1" => self.sp1 = ctx.decode_field(val, Self::TABLE_NAME, "Sp1")?,
            b"Sp2" => self.sp2 = ctx.decode_field(val, Self::TABLE_NAME, "Sp2")?,
            b"Sp3" => self.sp3 = ctx.decode_field(val, Self::TABLE_NAME, "Sp3")?,
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
            b"Sp4" => self.sp4 = ctx.decode_field(val, Self::TABLE_NAME, "Sp4")?,
            b"Sp5" => self.sp5 = ctx.decode_field(val, Self::TABLE_NAME, "Sp5")?,
            b"Sp6" => self.sp6 = ctx.decode_field(val, Self::TABLE_NAME, "Sp6")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS6GameplayInteractionRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_interaction";

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

/// Typed `DataTable` for `config_s6_gameplay_interaction.out`.
pub type ConfigS6GameplayInteractionTable = DataTable<u16, ConfigS6GameplayInteractionRow>;

/// Parses `config_s6_gameplay_interaction.out` into a [`ConfigS6GameplayInteractionTable`].
pub fn parse_config_s6_gameplay_interaction(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayInteractionTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_item.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayItemRow {
    pub desc: Box<str>,
    pub function: Box<[u32]>,
    pub group: u32,
    pub icon: Box<str>,
    pub icon_grey: Box<str>,
    pub id: i16,
    pub interaction: u16,
    pub name: Box<str>,
    pub price: u16,
    pub show_doc: u8,
    pub sort: u16,
    pub tile: u8,
    pub time: u8,
    pub r#type: u8,
    pub unlock_req: u8,
    pub weight: u8,
    pub weight_shop: u8,
    pub _desc: Box<str>,
    pub _name: Box<str>,
}

impl ConfigS6GameplayItemRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"IconGrey" => self.icon_grey = ctx.decode_field(val, Self::TABLE_NAME, "IconGrey")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Interaction" => {
                self.interaction = ctx.decode_field(val, Self::TABLE_NAME, "Interaction")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Price" => self.price = ctx.decode_field(val, Self::TABLE_NAME, "Price")?,
            b"ShowDoc" => self.show_doc = ctx.decode_field(val, Self::TABLE_NAME, "ShowDoc")?,
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"Tile" => self.tile = ctx.decode_field(val, Self::TABLE_NAME, "Tile")?,
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
            b"Time" => self.time = ctx.decode_field(val, Self::TABLE_NAME, "Time")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"WeightShop" => {
                self.weight_shop = ctx.decode_field(val, Self::TABLE_NAME, "WeightShop")?
            }
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS6GameplayItemRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_item";

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

/// Typed `DataTable` for `config_s6_gameplay_item.out`.
pub type ConfigS6GameplayItemTable = DataTable<u16, ConfigS6GameplayItemRow>;

/// Parses `config_s6_gameplay_item.out` into a [`ConfigS6GameplayItemTable`].
pub fn parse_config_s6_gameplay_item(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayItemTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_leveldiffculty.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayLeveldiffcultyRow {
    pub cost: FxHashMap<u8, Box<[u32]>>,
    pub cost_type: u8,
    pub desc1: Box<str>,
    pub desc2: Box<str>,
    pub desc3: Box<str>,
    pub desc4: Box<str>,
    pub diffculty: u8,
    pub id: i8,
    pub level_req: u8,
    pub max_level: u8,
    pub rule: Box<str>,
    pub rule_type: u8,
    pub title: Box<str>,
    pub unlock_req: FxHashMap<u8, u8>,
    pub _desc1: Box<str>,
    pub _desc2: Box<str>,
    pub _desc3: Box<str>,
    pub _desc4: Box<str>,
    pub _title: Box<str>,
}

impl ConfigS6GameplayLeveldiffcultyRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"CostType" => self.cost_type = ctx.decode_field(val, Self::TABLE_NAME, "CostType")?,
            b"Desc1" => self.desc1 = ctx.decode_field(val, Self::TABLE_NAME, "Desc1")?,
            b"Desc2" => self.desc2 = ctx.decode_field(val, Self::TABLE_NAME, "Desc2")?,
            b"Desc3" => self.desc3 = ctx.decode_field(val, Self::TABLE_NAME, "Desc3")?,
            b"Desc4" => self.desc4 = ctx.decode_field(val, Self::TABLE_NAME, "Desc4")?,
            b"Diffculty" => {
                self.diffculty = ctx.decode_field(val, Self::TABLE_NAME, "Diffculty")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LevelReq" => self.level_req = ctx.decode_field(val, Self::TABLE_NAME, "LevelReq")?,
            b"MaxLevel" => self.max_level = ctx.decode_field(val, Self::TABLE_NAME, "MaxLevel")?,
            b"Rule" => self.rule = ctx.decode_field(val, Self::TABLE_NAME, "Rule")?,
            b"RuleType" => self.rule_type = ctx.decode_field(val, Self::TABLE_NAME, "RuleType")?,
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
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            b"_Desc1" => self._desc1 = ctx.decode_field(val, Self::TABLE_NAME, "_Desc1")?,
            b"_Desc2" => self._desc2 = ctx.decode_field(val, Self::TABLE_NAME, "_Desc2")?,
            b"_Desc3" => self._desc3 = ctx.decode_field(val, Self::TABLE_NAME, "_Desc3")?,
            b"_Desc4" => self._desc4 = ctx.decode_field(val, Self::TABLE_NAME, "_Desc4")?,
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS6GameplayLeveldiffcultyRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_leveldiffculty";

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

/// Typed `DataTable` for `config_s6_gameplay_leveldiffculty.out`.
pub type ConfigS6GameplayLeveldiffcultyTable = DataTable<u8, ConfigS6GameplayLeveldiffcultyRow>;

/// Parses `config_s6_gameplay_leveldiffculty.out` into a [`ConfigS6GameplayLeveldiffcultyTable`].
pub fn parse_config_s6_gameplay_leveldiffculty(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayLeveldiffcultyTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_mechanism.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayMechanismRow {
    pub buff: Box<[u16]>,
    pub content_desc: Box<str>,
    pub content_name: Box<str>,
    pub exploration_event: [u8; 0],
    pub icon: Box<str>,
    pub id: i8,
    pub interact_event: Box<[u32]>,
    pub reveal_event: Box<[u32]>,
    pub r#type: u8,
    pub _content_desc: Box<str>,
    pub _content_name: Box<str>,
}

impl DataCfgRow for ConfigS6GameplayMechanismRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_mechanism";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Buff" => self.buff = ctx.decode_field(val, Self::TABLE_NAME, "Buff")?,
            b"ContentDesc" => {
                self.content_desc = ctx.decode_field(val, Self::TABLE_NAME, "ContentDesc")?
            }
            b"ContentName" => {
                self.content_name = ctx.decode_field(val, Self::TABLE_NAME, "ContentName")?
            }
            b"ExplorationEvent" => {
                self.exploration_event =
                    ctx.decode_field(val, Self::TABLE_NAME, "ExplorationEvent")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InteractEvent" => {
                self.interact_event = ctx.decode_field(val, Self::TABLE_NAME, "InteractEvent")?
            }
            b"RevealEvent" => {
                self.reveal_event = ctx.decode_field(val, Self::TABLE_NAME, "RevealEvent")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_ContentDesc" => {
                self._content_desc = ctx.decode_field(val, Self::TABLE_NAME, "_ContentDesc")?
            }
            b"_ContentName" => {
                self._content_name = ctx.decode_field(val, Self::TABLE_NAME, "_ContentName")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s6_gameplay_mechanism.out`.
pub type ConfigS6GameplayMechanismTable = DataTable<u8, ConfigS6GameplayMechanismRow>;

/// Parses `config_s6_gameplay_mechanism.out` into a [`ConfigS6GameplayMechanismTable`].
pub fn parse_config_s6_gameplay_mechanism(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayMechanismTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_monster.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayMonsterRow {
    pub buff: Box<[u16]>,
    pub content_desc: Box<str>,
    pub content_name: Box<str>,
    pub defeat_event: Box<[u32]>,
    pub exploration_event: [u8; 0],
    pub icon: Box<str>,
    pub id: i8,
    pub interact_event: [u8; 0],
    pub rarity: u8,
    pub reveal_event: [u8; 0],
    pub sanity_consume: FxHashMap<u8, u8>,
    pub r#type: u8,
    pub _content_desc: Box<str>,
    pub _content_name: Box<str>,
}

impl ConfigS6GameplayMonsterRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Buff" => self.buff = ctx.decode_field(val, Self::TABLE_NAME, "Buff")?,
            b"ContentDesc" => {
                self.content_desc = ctx.decode_field(val, Self::TABLE_NAME, "ContentDesc")?
            }
            b"ContentName" => {
                self.content_name = ctx.decode_field(val, Self::TABLE_NAME, "ContentName")?
            }
            b"DefeatEvent" => {
                self.defeat_event = ctx.decode_field(val, Self::TABLE_NAME, "DefeatEvent")?
            }
            b"ExplorationEvent" => {
                self.exploration_event =
                    ctx.decode_field(val, Self::TABLE_NAME, "ExplorationEvent")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InteractEvent" => {
                self.interact_event = ctx.decode_field(val, Self::TABLE_NAME, "InteractEvent")?
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"RevealEvent" => {
                self.reveal_event = ctx.decode_field(val, Self::TABLE_NAME, "RevealEvent")?
            }
            b"SanityConsume" => {
                self.sanity_consume = ctx.decode_field(val, Self::TABLE_NAME, "SanityConsume")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
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
            b"_ContentDesc" => {
                self._content_desc = ctx.decode_field(val, Self::TABLE_NAME, "_ContentDesc")?
            }
            b"_ContentName" => {
                self._content_name = ctx.decode_field(val, Self::TABLE_NAME, "_ContentName")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS6GameplayMonsterRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_monster";

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

/// Typed `DataTable` for `config_s6_gameplay_monster.out`.
pub type ConfigS6GameplayMonsterTable = DataTable<u8, ConfigS6GameplayMonsterRow>;

/// Parses `config_s6_gameplay_monster.out` into a [`ConfigS6GameplayMonsterTable`].
pub fn parse_config_s6_gameplay_monster(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayMonsterTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_narrtive.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayNarrtiveRow {
    pub content: Box<str>,
    pub id: i16,
    pub r#type: u8,
    pub weight: u8,
    pub _content: Box<str>,
}

impl DataCfgRow for ConfigS6GameplayNarrtiveRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_narrtive";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Content" => self.content = ctx.decode_field(val, Self::TABLE_NAME, "Content")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_Content" => self._content = ctx.decode_field(val, Self::TABLE_NAME, "_Content")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s6_gameplay_narrtive.out`.
pub type ConfigS6GameplayNarrtiveTable = DataTable<u16, ConfigS6GameplayNarrtiveRow>;

/// Parses `config_s6_gameplay_narrtive.out` into a [`ConfigS6GameplayNarrtiveTable`].
pub fn parse_config_s6_gameplay_narrtive(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayNarrtiveTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_preparation.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayPreparationRow {
    pub buff: [u8; 0],
    pub consume: u16,
    pub desc: Box<str>,
    pub event: Box<[u32]>,
    pub id: i8,
    pub unlock_req: Box<[u8]>,
    pub _desc: Box<str>,
}

impl DataCfgRow for ConfigS6GameplayPreparationRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_preparation";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Buff" => self.buff = ctx.decode_field(val, Self::TABLE_NAME, "Buff")?,
            b"Consume" => self.consume = ctx.decode_field(val, Self::TABLE_NAME, "Consume")?,
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Event" => self.event = ctx.decode_field(val, Self::TABLE_NAME, "Event")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s6_gameplay_preparation.out`.
pub type ConfigS6GameplayPreparationTable = DataTable<u8, ConfigS6GameplayPreparationRow>;

/// Parses `config_s6_gameplay_preparation.out` into a [`ConfigS6GameplayPreparationTable`].
pub fn parse_config_s6_gameplay_preparation(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayPreparationTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_relic.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayRelicRow {
    pub buff: FxHashMap<u8, Box<[u16]>>,
    pub desc: Box<str>,
    pub effect_level: u8,
    pub exclusive_group: Box<[u16]>,
    pub get_event: [u8; 0],
    pub group: u32,
    pub icon: Box<str>,
    pub icon_grey: Box<str>,
    pub id: i16,
    pub name: Box<str>,
    pub price: u16,
    pub show_doc: u8,
    pub sort: u16,
    pub tag: Box<[u8]>,
    pub unlock_req: u8,
    pub upgrade_price: u16,
    pub weight: u8,
    pub weight_modifier: FxHashMap<u16, Box<[u8]>>,
    pub weight_shop: u8,
    pub weight_shop_modifier: FxHashMap<u16, Box<[u8]>>,
    pub _desc: Box<str>,
    pub _name: Box<str>,
}

impl ConfigS6GameplayRelicRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Buff" => self.buff = ctx.decode_field(val, Self::TABLE_NAME, "Buff")?,
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"EffectLevel" => {
                self.effect_level = ctx.decode_field(val, Self::TABLE_NAME, "EffectLevel")?
            }
            b"ExclusiveGroup" => {
                self.exclusive_group = ctx.decode_field(val, Self::TABLE_NAME, "ExclusiveGroup")?
            }
            b"GetEvent" => self.get_event = ctx.decode_field(val, Self::TABLE_NAME, "GetEvent")?,
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"IconGrey" => self.icon_grey = ctx.decode_field(val, Self::TABLE_NAME, "IconGrey")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Price" => self.price = ctx.decode_field(val, Self::TABLE_NAME, "Price")?,
            b"ShowDoc" => self.show_doc = ctx.decode_field(val, Self::TABLE_NAME, "ShowDoc")?,
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
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"Tag" => self.tag = ctx.decode_field(val, Self::TABLE_NAME, "Tag")?,
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            b"UpgradePrice" => {
                self.upgrade_price = ctx.decode_field(val, Self::TABLE_NAME, "UpgradePrice")?
            }
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"WeightModifier" => {
                self.weight_modifier = ctx.decode_field(val, Self::TABLE_NAME, "WeightModifier")?
            }
            b"WeightShop" => {
                self.weight_shop = ctx.decode_field(val, Self::TABLE_NAME, "WeightShop")?
            }
            b"WeightShopModifier" => {
                self.weight_shop_modifier =
                    ctx.decode_field(val, Self::TABLE_NAME, "WeightShopModifier")?
            }
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS6GameplayRelicRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_relic";

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

/// Typed `DataTable` for `config_s6_gameplay_relic.out`.
pub type ConfigS6GameplayRelicTable = DataTable<u16, ConfigS6GameplayRelicRow>;

/// Parses `config_s6_gameplay_relic.out` into a [`ConfigS6GameplayRelicTable`].
pub fn parse_config_s6_gameplay_relic(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayRelicTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_s6_gameplay_talent.Param`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct S6GameplayTalentParamEntry {
    pub kind: u8,
    pub params: Box<[u32]>,
}

impl FromEvalVal for S6GameplayTalentParamEntry {
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

/// Row record for `config_s6_gameplay_talent.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayTalentRow {
    pub desc: Box<str>,
    pub desc_func: Box<str>,
    pub func_parameter: Box<[u8]>,
    pub function: u8,
    pub icon: Box<str>,
    pub icon_grey: Box<str>,
    pub id: i8,
    pub level: u8,
    pub param: Box<[S6GameplayTalentParamEntry]>,
    pub tier: u8,
    pub title: Box<str>,
    pub upgrade_consume: Box<[u8]>,
    pub _desc: Box<str>,
    pub _desc_func: Box<str>,
    pub _title: Box<str>,
}

impl ConfigS6GameplayTalentRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DescFunc" => self.desc_func = ctx.decode_field(val, Self::TABLE_NAME, "DescFunc")?,
            b"FuncParameter" => {
                self.func_parameter = ctx.decode_field(val, Self::TABLE_NAME, "FuncParameter")?
            }
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"IconGrey" => self.icon_grey = ctx.decode_field(val, Self::TABLE_NAME, "IconGrey")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Param" => self.param = ctx.decode_field(val, Self::TABLE_NAME, "Param")?,
            b"Tier" => self.tier = ctx.decode_field(val, Self::TABLE_NAME, "Tier")?,
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"UpgradeConsume" => {
                self.upgrade_consume = ctx.decode_field(val, Self::TABLE_NAME, "UpgradeConsume")?
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
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_DescFunc" => {
                self._desc_func = ctx.decode_field(val, Self::TABLE_NAME, "_DescFunc")?
            }
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS6GameplayTalentRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_talent";

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

/// Typed `DataTable` for `config_s6_gameplay_talent.out`.
pub type ConfigS6GameplayTalentTable = DataTable<u8, ConfigS6GameplayTalentRow>;

/// Parses `config_s6_gameplay_talent.out` into a [`ConfigS6GameplayTalentTable`].
pub fn parse_config_s6_gameplay_talent(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayTalentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s6_gameplay_tile.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS6GameplayTileRow {
    pub content_desc: Box<str>,
    pub content_name: Box<str>,
    pub exclusive_tile: FxHashMap<u16, u8>,
    pub exploration_event: [u8; 0],
    pub group: u32,
    pub icon: Box<str>,
    pub icon_disabled: Box<str>,
    pub id: i32,
    pub init_event: Box<[u32]>,
    pub interact_event: Box<[u32]>,
    pub interaction: u16,
    pub level: Box<[u8]>,
    pub reveal_event: Box<[u32]>,
    pub rule: Box<str>,
    pub tag: u8,
    pub r#type: u8,
    pub _content_desc: Box<str>,
    pub _content_name: Box<str>,
}

impl ConfigS6GameplayTileRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"ContentDesc" => {
                self.content_desc = ctx.decode_field(val, Self::TABLE_NAME, "ContentDesc")?
            }
            b"ContentName" => {
                self.content_name = ctx.decode_field(val, Self::TABLE_NAME, "ContentName")?
            }
            b"ExclusiveTile" => {
                self.exclusive_tile = ctx.decode_field(val, Self::TABLE_NAME, "ExclusiveTile")?
            }
            b"ExplorationEvent" => {
                self.exploration_event =
                    ctx.decode_field(val, Self::TABLE_NAME, "ExplorationEvent")?
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"IconDisabled" => {
                self.icon_disabled = ctx.decode_field(val, Self::TABLE_NAME, "IconDisabled")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InitEvent" => {
                self.init_event = ctx.decode_field(val, Self::TABLE_NAME, "InitEvent")?
            }
            b"InteractEvent" => {
                self.interact_event = ctx.decode_field(val, Self::TABLE_NAME, "InteractEvent")?
            }
            b"Interaction" => {
                self.interaction = ctx.decode_field(val, Self::TABLE_NAME, "Interaction")?
            }
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
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
            b"RevealEvent" => {
                self.reveal_event = ctx.decode_field(val, Self::TABLE_NAME, "RevealEvent")?
            }
            b"Rule" => self.rule = ctx.decode_field(val, Self::TABLE_NAME, "Rule")?,
            b"Tag" => self.tag = ctx.decode_field(val, Self::TABLE_NAME, "Tag")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_ContentDesc" => {
                self._content_desc = ctx.decode_field(val, Self::TABLE_NAME, "_ContentDesc")?
            }
            b"_ContentName" => {
                self._content_name = ctx.decode_field(val, Self::TABLE_NAME, "_ContentName")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS6GameplayTileRow {
    const TABLE_NAME: &'static str = "config_s6_gameplay_tile";

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

/// Typed `DataTable` for `config_s6_gameplay_tile.out`.
pub type ConfigS6GameplayTileTable = DataTable<u32, ConfigS6GameplayTileRow>;

/// Parses `config_s6_gameplay_tile.out` into a [`ConfigS6GameplayTileTable`].
pub fn parse_config_s6_gameplay_tile(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS6GameplayTileTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_s8_gameplay_difficulty.UninstallCost`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct S8GameplayDifficultyUninstallCost {
    pub item_id: u32,
    pub counts: Box<[u8]>,
}

impl FromEvalVal for S8GameplayDifficultyUninstallCost {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (item_id, counts) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { item_id, counts })
    }
}

/// Row record for `config_s8_gameplay_difficulty.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS8GameplayDifficultyRow {
    pub available_dir: Box<[u8]>,
    pub block_max: u8,
    pub block_min: u8,
    pub collect_illustration: Box<str>,
    pub cost: FxHashMap<u32, u8>,
    pub cost_type: u8,
    pub diffculty: u8,
    pub diffculty_des: Box<str>,
    pub diffculty_name: Box<str>,
    pub fixed_effect_lib_cost: FxHashMap<u32, u8>,
    pub id: i8,
    pub init_max_level: Box<[u8]>,
    pub level_req: u8,
    pub map_modifier: Box<[u32]>,
    pub map_rule: FxHashMap<u16, u16>,
    pub plot_item_collect: u8,
    pub plot_item_source: u8,
    pub reward: FxHashMap<u32, u8>,
    pub rule: Box<str>,
    pub rule_type: u8,
    pub uninstall_cost: S8GameplayDifficultyUninstallCost,
    pub unlock_illustration: Box<str>,
    pub unlock_req: u8,
    pub _collect_illustration: Box<str>,
    pub _diffculty_des: Box<str>,
    pub _diffculty_name: Box<str>,
    pub _unlock_illustration: Box<str>,
}

impl ConfigS8GameplayDifficultyRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AvailableDir" => {
                self.available_dir = ctx.decode_field(val, Self::TABLE_NAME, "AvailableDir")?
            }
            b"BlockMax" => self.block_max = ctx.decode_field(val, Self::TABLE_NAME, "BlockMax")?,
            b"BlockMin" => self.block_min = ctx.decode_field(val, Self::TABLE_NAME, "BlockMin")?,
            b"CollectIllustration" => {
                self.collect_illustration =
                    ctx.decode_field(val, Self::TABLE_NAME, "CollectIllustration")?
            }
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"CostType" => self.cost_type = ctx.decode_field(val, Self::TABLE_NAME, "CostType")?,
            b"Diffculty" => {
                self.diffculty = ctx.decode_field(val, Self::TABLE_NAME, "Diffculty")?
            }
            b"DiffcultyDes" => {
                self.diffculty_des = ctx.decode_field(val, Self::TABLE_NAME, "DiffcultyDes")?
            }
            b"DiffcultyName" => {
                self.diffculty_name = ctx.decode_field(val, Self::TABLE_NAME, "DiffcultyName")?
            }
            b"FixedEffectLibCost" => {
                self.fixed_effect_lib_cost =
                    ctx.decode_field(val, Self::TABLE_NAME, "FixedEffectLibCost")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InitMaxLevel" => {
                self.init_max_level = ctx.decode_field(val, Self::TABLE_NAME, "InitMaxLevel")?
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
            b"LevelReq" => self.level_req = ctx.decode_field(val, Self::TABLE_NAME, "LevelReq")?,
            b"MapModifier" => {
                self.map_modifier = ctx.decode_field(val, Self::TABLE_NAME, "MapModifier")?
            }
            b"MapRule" => self.map_rule = ctx.decode_field(val, Self::TABLE_NAME, "MapRule")?,
            b"PlotItemCollect" => {
                self.plot_item_collect =
                    ctx.decode_field(val, Self::TABLE_NAME, "PlotItemCollect")?
            }
            b"PlotItemSource" => {
                self.plot_item_source = ctx.decode_field(val, Self::TABLE_NAME, "PlotItemSource")?
            }
            b"Reward" => self.reward = ctx.decode_field(val, Self::TABLE_NAME, "Reward")?,
            b"Rule" => self.rule = ctx.decode_field(val, Self::TABLE_NAME, "Rule")?,
            b"RuleType" => self.rule_type = ctx.decode_field(val, Self::TABLE_NAME, "RuleType")?,
            b"UninstallCost" => {
                self.uninstall_cost = ctx.decode_field(val, Self::TABLE_NAME, "UninstallCost")?
            }
            b"UnlockIllustration" => {
                self.unlock_illustration =
                    ctx.decode_field(val, Self::TABLE_NAME, "UnlockIllustration")?
            }
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            b"_CollectIllustration" => {
                self._collect_illustration =
                    ctx.decode_field(val, Self::TABLE_NAME, "_CollectIllustration")?
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
            b"_DiffcultyDes" => {
                self._diffculty_des = ctx.decode_field(val, Self::TABLE_NAME, "_DiffcultyDes")?
            }
            b"_DiffcultyName" => {
                self._diffculty_name = ctx.decode_field(val, Self::TABLE_NAME, "_DiffcultyName")?
            }
            b"_UnlockIllustration" => {
                self._unlock_illustration =
                    ctx.decode_field(val, Self::TABLE_NAME, "_UnlockIllustration")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS8GameplayDifficultyRow {
    const TABLE_NAME: &'static str = "config_s8_gameplay_difficulty";

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

/// Typed `DataTable` for `config_s8_gameplay_difficulty.out`.
pub type ConfigS8GameplayDifficultyTable = DataTable<u8, ConfigS8GameplayDifficultyRow>;

/// Parses `config_s8_gameplay_difficulty.out` into a [`ConfigS8GameplayDifficultyTable`].
pub fn parse_config_s8_gameplay_difficulty(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS8GameplayDifficultyTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s8_gameplay_itemeffect.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS8GameplayItemeffectRow {
    pub action_direction: u8,
    pub action_param: u8,
    pub action_type: u8,
    pub effect_des: Box<str>,
    pub effect_type: u8,
    pub id: i16,
    pub library_id: u8,
    pub task_des: Box<str>,
    pub task_direction: u8,
    pub task_id: u16,
    pub task_num: u8,
    pub task_type: u8,
    pub unlock_difficulty: u8,
    pub weight: u16,
    pub _effect_des: Box<str>,
    pub _task_des: Box<str>,
}

impl ConfigS8GameplayItemeffectRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"ActionDirection" => {
                self.action_direction =
                    ctx.decode_field(val, Self::TABLE_NAME, "ActionDirection")?
            }
            b"ActionParam" => {
                self.action_param = ctx.decode_field(val, Self::TABLE_NAME, "ActionParam")?
            }
            b"ActionType" => {
                self.action_type = ctx.decode_field(val, Self::TABLE_NAME, "ActionType")?
            }
            b"EffectDes" => {
                self.effect_des = ctx.decode_field(val, Self::TABLE_NAME, "EffectDes")?
            }
            b"EffectType" => {
                self.effect_type = ctx.decode_field(val, Self::TABLE_NAME, "EffectType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LibraryId" => {
                self.library_id = ctx.decode_field(val, Self::TABLE_NAME, "LibraryId")?
            }
            b"TaskDes" => self.task_des = ctx.decode_field(val, Self::TABLE_NAME, "TaskDes")?,
            b"TaskDirection" => {
                self.task_direction = ctx.decode_field(val, Self::TABLE_NAME, "TaskDirection")?
            }
            b"TaskId" => self.task_id = ctx.decode_field(val, Self::TABLE_NAME, "TaskId")?,
            b"TaskNum" => self.task_num = ctx.decode_field(val, Self::TABLE_NAME, "TaskNum")?,
            b"TaskType" => self.task_type = ctx.decode_field(val, Self::TABLE_NAME, "TaskType")?,
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
            b"UnlockDifficulty" => {
                self.unlock_difficulty =
                    ctx.decode_field(val, Self::TABLE_NAME, "UnlockDifficulty")?
            }
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_EffectDes" => {
                self._effect_des = ctx.decode_field(val, Self::TABLE_NAME, "_EffectDes")?
            }
            b"_TaskDes" => self._task_des = ctx.decode_field(val, Self::TABLE_NAME, "_TaskDes")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS8GameplayItemeffectRow {
    const TABLE_NAME: &'static str = "config_s8_gameplay_itemeffect";

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

/// Typed `DataTable` for `config_s8_gameplay_itemeffect.out`.
pub type ConfigS8GameplayItemeffectTable = DataTable<u16, ConfigS8GameplayItemeffectRow>;

/// Parses `config_s8_gameplay_itemeffect.out` into a [`ConfigS8GameplayItemeffectTable`].
pub fn parse_config_s8_gameplay_itemeffect(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS8GameplayItemeffectTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s8_gameplay_plotitem.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS8GameplayPlotitemRow {
    pub collect_des: Box<str>,
    pub collect_reward_id: u8,
    pub effect_library_id: u8,
    pub icon: Box<str>,
    pub id: i32,
    pub item_gameplay: u32,
    pub item_gameplay_des: Box<str>,
    pub item_level: u8,
    pub item_name: Box<str>,
    pub item_quality: u8,
    pub item_series_id: u16,
    pub item_type: u8,
    pub s8_map_res: Box<str>,
    pub _collect_des: Box<str>,
    pub _item_gameplay_des: Box<str>,
    pub _item_name: Box<str>,
}

impl ConfigS8GameplayPlotitemRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"CollectDes" => {
                self.collect_des = ctx.decode_field(val, Self::TABLE_NAME, "CollectDes")?
            }
            b"CollectRewardId" => {
                self.collect_reward_id =
                    ctx.decode_field(val, Self::TABLE_NAME, "CollectRewardId")?
            }
            b"EffectLibraryId" => {
                self.effect_library_id =
                    ctx.decode_field(val, Self::TABLE_NAME, "EffectLibraryId")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemGameplay" => {
                self.item_gameplay = ctx.decode_field(val, Self::TABLE_NAME, "ItemGameplay")?
            }
            b"ItemGameplayDes" => {
                self.item_gameplay_des =
                    ctx.decode_field(val, Self::TABLE_NAME, "ItemGameplayDes")?
            }
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?
            }
            b"ItemName" => self.item_name = ctx.decode_field(val, Self::TABLE_NAME, "ItemName")?,
            b"ItemQuality" => {
                self.item_quality = ctx.decode_field(val, Self::TABLE_NAME, "ItemQuality")?
            }
            b"ItemSeriesId" => {
                self.item_series_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemSeriesId")?
            }
            b"ItemType" => self.item_type = ctx.decode_field(val, Self::TABLE_NAME, "ItemType")?,
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
            b"S8MapRes" => self.s8_map_res = ctx.decode_field(val, Self::TABLE_NAME, "S8MapRes")?,
            b"_CollectDes" => {
                self._collect_des = ctx.decode_field(val, Self::TABLE_NAME, "_CollectDes")?
            }
            b"_ItemGameplayDes" => {
                self._item_gameplay_des =
                    ctx.decode_field(val, Self::TABLE_NAME, "_ItemGameplayDes")?
            }
            b"_ItemName" => {
                self._item_name = ctx.decode_field(val, Self::TABLE_NAME, "_ItemName")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS8GameplayPlotitemRow {
    const TABLE_NAME: &'static str = "config_s8_gameplay_plotitem";

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

/// Typed `DataTable` for `config_s8_gameplay_plotitem.out`.
pub type ConfigS8GameplayPlotitemTable = DataTable<u32, ConfigS8GameplayPlotitemRow>;

/// Parses `config_s8_gameplay_plotitem.out` into a [`ConfigS8GameplayPlotitemTable`].
pub fn parse_config_s8_gameplay_plotitem(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS8GameplayPlotitemTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_s8_gameplay_revive.Cost`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct S8GameplayReviveCost {
    pub item_id: u32,
    pub count: u8,
}

impl FromEvalVal for S8GameplayReviveCost {
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

/// Row record for `config_s8_gameplay_revive.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS8GameplayReviveRow {
    pub cost: S8GameplayReviveCost,
    pub cost_type: u8,
    pub effect_des: Box<str>,
    pub effect_param1: u8,
    pub effect_param2: u8,
    pub effect_param3: u8,
    pub effect_type: u8,
    pub id: i16,
    pub revive_count: u8,
    pub _effect_des: Box<str>,
}

impl DataCfgRow for ConfigS8GameplayReviveRow {
    const TABLE_NAME: &'static str = "config_s8_gameplay_revive";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"CostType" => self.cost_type = ctx.decode_field(val, Self::TABLE_NAME, "CostType")?,
            b"EffectDes" => {
                self.effect_des = ctx.decode_field(val, Self::TABLE_NAME, "EffectDes")?
            }
            b"EffectParam1" => {
                self.effect_param1 = ctx.decode_field(val, Self::TABLE_NAME, "EffectParam1")?
            }
            b"EffectParam2" => {
                self.effect_param2 = ctx.decode_field(val, Self::TABLE_NAME, "EffectParam2")?
            }
            b"EffectParam3" => {
                self.effect_param3 = ctx.decode_field(val, Self::TABLE_NAME, "EffectParam3")?
            }
            b"EffectType" => {
                self.effect_type = ctx.decode_field(val, Self::TABLE_NAME, "EffectType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ReviveCount" => {
                self.revive_count = ctx.decode_field(val, Self::TABLE_NAME, "ReviveCount")?
            }
            b"_EffectDes" => {
                self._effect_des = ctx.decode_field(val, Self::TABLE_NAME, "_EffectDes")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s8_gameplay_revive.out`.
pub type ConfigS8GameplayReviveTable = DataTable<u16, ConfigS8GameplayReviveRow>;

/// Parses `config_s8_gameplay_revive.out` into a [`ConfigS8GameplayReviveTable`].
pub fn parse_config_s8_gameplay_revive(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS8GameplayReviveTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s8_gameplay_selectableeffect.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS8GameplaySelectableeffectRow {
    pub effect_des: Box<str>,
    pub effect_param1: u8,
    pub effect_param2: Box<str>,
    pub effect_param3: u8,
    pub effect_type: u16,
    pub id: i16,
    pub library_des: Box<str>,
    pub library_id: u8,
    pub library_name: Box<str>,
    pub unlock_difficulty: u8,
    pub _effect_des: Box<str>,
    pub _library_des: Box<str>,
    pub _library_name: Box<str>,
}

impl ConfigS8GameplaySelectableeffectRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"EffectDes" => {
                self.effect_des = ctx.decode_field(val, Self::TABLE_NAME, "EffectDes")?
            }
            b"EffectParam1" => {
                self.effect_param1 = ctx.decode_field(val, Self::TABLE_NAME, "EffectParam1")?
            }
            b"EffectParam2" => {
                self.effect_param2 = ctx.decode_field(val, Self::TABLE_NAME, "EffectParam2")?
            }
            b"EffectParam3" => {
                self.effect_param3 = ctx.decode_field(val, Self::TABLE_NAME, "EffectParam3")?
            }
            b"EffectType" => {
                self.effect_type = ctx.decode_field(val, Self::TABLE_NAME, "EffectType")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LibraryDes" => {
                self.library_des = ctx.decode_field(val, Self::TABLE_NAME, "LibraryDes")?
            }
            b"LibraryId" => {
                self.library_id = ctx.decode_field(val, Self::TABLE_NAME, "LibraryId")?
            }
            b"LibraryName" => {
                self.library_name = ctx.decode_field(val, Self::TABLE_NAME, "LibraryName")?
            }
            b"UnlockDifficulty" => {
                self.unlock_difficulty =
                    ctx.decode_field(val, Self::TABLE_NAME, "UnlockDifficulty")?
            }
            b"_EffectDes" => {
                self._effect_des = ctx.decode_field(val, Self::TABLE_NAME, "_EffectDes")?
            }
            b"_LibraryDes" => {
                self._library_des = ctx.decode_field(val, Self::TABLE_NAME, "_LibraryDes")?
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
            b"_LibraryName" => {
                self._library_name = ctx.decode_field(val, Self::TABLE_NAME, "_LibraryName")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS8GameplaySelectableeffectRow {
    const TABLE_NAME: &'static str = "config_s8_gameplay_selectableeffect";

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

/// Typed `DataTable` for `config_s8_gameplay_selectableeffect.out`.
pub type ConfigS8GameplaySelectableeffectTable =
    DataTable<u16, ConfigS8GameplaySelectableeffectRow>;

/// Parses `config_s8_gameplay_selectableeffect.out` into a [`ConfigS8GameplaySelectableeffectTable`].
pub fn parse_config_s8_gameplay_selectableeffect(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS8GameplaySelectableeffectTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s8_gameplay_talent.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS8GameplayTalentRow {
    pub cost: FxHashMap<u8, Box<[u32]>>,
    pub desc_args: FxHashMap<u8, Box<[u8]>>,
    pub description: Box<str>,
    pub effect_param1: Box<[u32]>,
    pub effect_param2: Box<[u32]>,
    pub effect_param3: Box<[u8]>,
    pub effect_type: u8,
    pub frozen_drop_id: Box<[u32]>,
    pub icon: Box<str>,
    pub id: i16,
    pub layer: u8,
    pub layer_unlock_point: u8,
    pub name: Box<str>,
    pub no_refund: u8,
    pub point: u8,
    pub position: u8,
    pub position_num: u8,
    pub r#type: u8,
    pub _description: Box<str>,
    pub _name: Box<str>,
}

impl ConfigS8GameplayTalentRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"DescArgs" => self.desc_args = ctx.decode_field(val, Self::TABLE_NAME, "DescArgs")?,
            b"Description" => {
                self.description = ctx.decode_field(val, Self::TABLE_NAME, "Description")?
            }
            b"EffectParam1" => {
                self.effect_param1 = ctx.decode_field(val, Self::TABLE_NAME, "EffectParam1")?
            }
            b"EffectParam2" => {
                self.effect_param2 = ctx.decode_field(val, Self::TABLE_NAME, "EffectParam2")?
            }
            b"EffectParam3" => {
                self.effect_param3 = ctx.decode_field(val, Self::TABLE_NAME, "EffectParam3")?
            }
            b"EffectType" => {
                self.effect_type = ctx.decode_field(val, Self::TABLE_NAME, "EffectType")?
            }
            b"FrozenDropId" => {
                self.frozen_drop_id = ctx.decode_field(val, Self::TABLE_NAME, "FrozenDropId")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Layer" => self.layer = ctx.decode_field(val, Self::TABLE_NAME, "Layer")?,
            b"LayerUnlockPoint" => {
                self.layer_unlock_point =
                    ctx.decode_field(val, Self::TABLE_NAME, "LayerUnlockPoint")?
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
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"NoRefund" => self.no_refund = ctx.decode_field(val, Self::TABLE_NAME, "NoRefund")?,
            b"Point" => self.point = ctx.decode_field(val, Self::TABLE_NAME, "Point")?,
            b"Position" => self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?,
            b"PositionNum" => {
                self.position_num = ctx.decode_field(val, Self::TABLE_NAME, "PositionNum")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Description" => {
                self._description = ctx.decode_field(val, Self::TABLE_NAME, "_Description")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigS8GameplayTalentRow {
    const TABLE_NAME: &'static str = "config_s8_gameplay_talent";

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

/// Typed `DataTable` for `config_s8_gameplay_talent.out`.
pub type ConfigS8GameplayTalentTable = DataTable<u16, ConfigS8GameplayTalentRow>;

/// Parses `config_s8_gameplay_talent.out` into a [`ConfigS8GameplayTalentTable`].
pub fn parse_config_s8_gameplay_talent(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS8GameplayTalentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s9_gameplay_card.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS9GameplayCardRow {
    pub event_draw: Box<[u16]>,
    pub icon: Box<str>,
    pub id: i16,
    pub interaction_draw: Box<[u8]>,
    pub name: Box<str>,
    pub random_weight: u32,
    pub rarity: u8,
    pub selection_weight: u32,
    pub sort: u16,
    pub tag: u8,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigS9GameplayCardRow {
    const TABLE_NAME: &'static str = "config_s9_gameplay_card";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"EventDraw" => {
                self.event_draw = ctx.decode_field(val, Self::TABLE_NAME, "EventDraw")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InteractionDraw" => {
                self.interaction_draw =
                    ctx.decode_field(val, Self::TABLE_NAME, "InteractionDraw")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"RandomWeight" => {
                self.random_weight = ctx.decode_field(val, Self::TABLE_NAME, "RandomWeight")?
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"SelectionWeight" => {
                self.selection_weight =
                    ctx.decode_field(val, Self::TABLE_NAME, "SelectionWeight")?
            }
            b"Sort" => self.sort = ctx.decode_field(val, Self::TABLE_NAME, "Sort")?,
            b"Tag" => self.tag = ctx.decode_field(val, Self::TABLE_NAME, "Tag")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s9_gameplay_card.out`.
pub type ConfigS9GameplayCardTable = DataTable<u16, ConfigS9GameplayCardRow>;

/// Parses `config_s9_gameplay_card.out` into a [`ConfigS9GameplayCardTable`].
pub fn parse_config_s9_gameplay_card(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS9GameplayCardTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s9_gameplay_event.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS9GameplayEventRow {
    pub desc: Box<str>,
    pub function: u8,
    pub id: i16,
    pub p1: i8,
    pub p2: u16,
    pub p3: u8,
    pub sp1: Box<[i8]>,
    pub sp2: Box<[u8]>,
    pub sp3: Box<[u16]>,
    pub _desc: Box<str>,
}

impl DataCfgRow for ConfigS9GameplayEventRow {
    const TABLE_NAME: &'static str = "config_s9_gameplay_event";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"P1" => self.p1 = ctx.decode_field(val, Self::TABLE_NAME, "P1")?,
            b"P2" => self.p2 = ctx.decode_field(val, Self::TABLE_NAME, "P2")?,
            b"P3" => self.p3 = ctx.decode_field(val, Self::TABLE_NAME, "P3")?,
            b"Sp1" => self.sp1 = ctx.decode_field(val, Self::TABLE_NAME, "Sp1")?,
            b"Sp2" => self.sp2 = ctx.decode_field(val, Self::TABLE_NAME, "Sp2")?,
            b"Sp3" => self.sp3 = ctx.decode_field(val, Self::TABLE_NAME, "Sp3")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s9_gameplay_event.out`.
pub type ConfigS9GameplayEventTable = DataTable<u16, ConfigS9GameplayEventRow>;

/// Parses `config_s9_gameplay_event.out` into a [`ConfigS9GameplayEventTable`].
pub fn parse_config_s9_gameplay_event(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS9GameplayEventTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s9_gameplay_interaction.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS9GameplayInteractionRow {
    pub desc: Box<str>,
    pub function: u16,
    pub id: i16,
    pub p1: u8,
    pub p2: u8,
    pub p3: u8,
    pub sp1: Box<[u8]>,
    pub sp2: Box<[u8]>,
    pub sp3: Box<[u16]>,
    pub _desc: Box<str>,
}

impl DataCfgRow for ConfigS9GameplayInteractionRow {
    const TABLE_NAME: &'static str = "config_s9_gameplay_interaction";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Function" => self.function = ctx.decode_field(val, Self::TABLE_NAME, "Function")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"P1" => self.p1 = ctx.decode_field(val, Self::TABLE_NAME, "P1")?,
            b"P2" => self.p2 = ctx.decode_field(val, Self::TABLE_NAME, "P2")?,
            b"P3" => self.p3 = ctx.decode_field(val, Self::TABLE_NAME, "P3")?,
            b"Sp1" => self.sp1 = ctx.decode_field(val, Self::TABLE_NAME, "Sp1")?,
            b"Sp2" => self.sp2 = ctx.decode_field(val, Self::TABLE_NAME, "Sp2")?,
            b"Sp3" => self.sp3 = ctx.decode_field(val, Self::TABLE_NAME, "Sp3")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s9_gameplay_interaction.out`.
pub type ConfigS9GameplayInteractionTable = DataTable<u16, ConfigS9GameplayInteractionRow>;

/// Parses `config_s9_gameplay_interaction.out` into a [`ConfigS9GameplayInteractionTable`].
pub fn parse_config_s9_gameplay_interaction(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS9GameplayInteractionTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s9_gameplay_mapgameplay.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS9GameplayMapgameplayRow {
    pub desc: Box<str>,
    pub icon: Box<str>,
    pub id: i16,
    pub name: Box<str>,
    pub _desc: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigS9GameplayMapgameplayRow {
    const TABLE_NAME: &'static str = "config_s9_gameplay_mapgameplay";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s9_gameplay_mapgameplay.out`.
pub type ConfigS9GameplayMapgameplayTable = DataTable<u16, ConfigS9GameplayMapgameplayRow>;

/// Parses `config_s9_gameplay_mapgameplay.out` into a [`ConfigS9GameplayMapgameplayTable`].
pub fn parse_config_s9_gameplay_mapgameplay(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS9GameplayMapgameplayTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_s9_gameplay_tile.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS9GameplayTileRow {
    pub event_settle: Box<[u16]>,
    pub icon: Box<str>,
    pub id: i16,
    pub interaction_settle: Box<[u8]>,
    pub rarity_weight: FxHashMap<u8, u16>,
    pub r#type: u8,
}

impl DataCfgRow for ConfigS9GameplayTileRow {
    const TABLE_NAME: &'static str = "config_s9_gameplay_tile";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"EventSettle" => {
                self.event_settle = ctx.decode_field(val, Self::TABLE_NAME, "EventSettle")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InteractionSettle" => {
                self.interaction_settle =
                    ctx.decode_field(val, Self::TABLE_NAME, "InteractionSettle")?
            }
            b"RarityWeight" => {
                self.rarity_weight = ctx.decode_field(val, Self::TABLE_NAME, "RarityWeight")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_s9_gameplay_tile.out`.
pub type ConfigS9GameplayTileTable = DataTable<u16, ConfigS9GameplayTileRow>;

/// Parses `config_s9_gameplay_tile.out` into a [`ConfigS9GameplayTileTable`].
pub fn parse_config_s9_gameplay_tile(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigS9GameplayTileTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_season.AffiliatedSeasonId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SeasonAffiliatedSeasonIdEntry {
    pub season_type: u8,
    pub season_id: u16,
}

impl FromEvalVal for SeasonAffiliatedSeasonIdEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (season_type, season_id) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            season_type,
            season_id,
        })
    }
}

/// Row record for `config_season.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSeasonRow {
    pub affiliated_season_id: Box<[SeasonAffiliatedSeasonIdEntry]>,
    pub default_create_order: u8,
    pub des: Box<str>,
    pub icon1: Box<str>,
    pub icon2: Box<str>,
    pub id: i16,
    pub index: u8,
    pub is_recommend: bool,
    pub name1: Box<str>,
    pub name2: Box<str>,
    pub name3: Box<str>,
    pub name4: Box<str>,
    pub role_season_font: u16,
    pub season_current: u16,
    pub season_des: Box<str>,
    pub season_des2: Box<str>,
    pub season_id: u16,
    pub season_intro_pic: Box<str>,
    pub season_intro_show: u8,
    pub season_intro_text: Box<str>,
    pub season_intro_ulrc: Box<str>,
    pub season_intro_ulre: Box<str>,
    pub season_main: u8,
    pub season_pic: Box<str>,
    pub season_pic2: Box<str>,
    pub season_text: Box<str>,
    pub season_type: u8,
    pub season_window1: Box<str>,
    pub season_window2: Box<str>,
    pub _des: Box<str>,
    pub _name1: Box<str>,
    pub _name2: Box<str>,
    pub _name3: Box<str>,
    pub _season_des: Box<str>,
    pub _season_des2: Box<str>,
    pub _season_intro_text: Box<str>,
    pub _season_text: Box<str>,
    pub _season_window1: Box<str>,
    pub _season_window2: Box<str>,
}

impl ConfigSeasonRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AffiliatedSeasonId" => {
                self.affiliated_season_id =
                    ctx.decode_field(val, Self::TABLE_NAME, "AffiliatedSeasonId")?
            }
            b"DefaultCreateOrder" => {
                self.default_create_order =
                    ctx.decode_field(val, Self::TABLE_NAME, "DefaultCreateOrder")?
            }
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Icon1" => self.icon1 = ctx.decode_field(val, Self::TABLE_NAME, "Icon1")?,
            b"Icon2" => self.icon2 = ctx.decode_field(val, Self::TABLE_NAME, "Icon2")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Index" => self.index = ctx.decode_field(val, Self::TABLE_NAME, "Index")?,
            b"IsRecommend" => {
                self.is_recommend = ctx.decode_field(val, Self::TABLE_NAME, "IsRecommend")?
            }
            b"Name1" => self.name1 = ctx.decode_field(val, Self::TABLE_NAME, "Name1")?,
            b"Name2" => self.name2 = ctx.decode_field(val, Self::TABLE_NAME, "Name2")?,
            b"Name3" => self.name3 = ctx.decode_field(val, Self::TABLE_NAME, "Name3")?,
            b"Name4" => self.name4 = ctx.decode_field(val, Self::TABLE_NAME, "Name4")?,
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
            b"RoleSeasonFont" => {
                self.role_season_font = ctx.decode_field(val, Self::TABLE_NAME, "RoleSeasonFont")?
            }
            b"SeasonCurrent" => {
                self.season_current = ctx.decode_field(val, Self::TABLE_NAME, "SeasonCurrent")?
            }
            b"SeasonDes" => {
                self.season_des = ctx.decode_field(val, Self::TABLE_NAME, "SeasonDes")?
            }
            b"SeasonDes2" => {
                self.season_des2 = ctx.decode_field(val, Self::TABLE_NAME, "SeasonDes2")?
            }
            b"SeasonId" => self.season_id = ctx.decode_field(val, Self::TABLE_NAME, "SeasonId")?,
            b"SeasonIntroPic" => {
                self.season_intro_pic = ctx.decode_field(val, Self::TABLE_NAME, "SeasonIntroPic")?
            }
            b"SeasonIntroShow" => {
                self.season_intro_show =
                    ctx.decode_field(val, Self::TABLE_NAME, "SeasonIntroShow")?
            }
            b"SeasonIntroText" => {
                self.season_intro_text =
                    ctx.decode_field(val, Self::TABLE_NAME, "SeasonIntroText")?
            }
            b"SeasonIntroUlrc" => {
                self.season_intro_ulrc =
                    ctx.decode_field(val, Self::TABLE_NAME, "SeasonIntroUlrc")?
            }
            b"SeasonIntroUlre" => {
                self.season_intro_ulre =
                    ctx.decode_field(val, Self::TABLE_NAME, "SeasonIntroUlre")?
            }
            b"SeasonMain" => {
                self.season_main = ctx.decode_field(val, Self::TABLE_NAME, "SeasonMain")?
            }
            b"SeasonPic" => {
                self.season_pic = ctx.decode_field(val, Self::TABLE_NAME, "SeasonPic")?
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
            b"SeasonPic2" => {
                self.season_pic2 = ctx.decode_field(val, Self::TABLE_NAME, "SeasonPic2")?
            }
            b"SeasonText" => {
                self.season_text = ctx.decode_field(val, Self::TABLE_NAME, "SeasonText")?
            }
            b"SeasonType" => {
                self.season_type = ctx.decode_field(val, Self::TABLE_NAME, "SeasonType")?
            }
            b"SeasonWindow1" => {
                self.season_window1 = ctx.decode_field(val, Self::TABLE_NAME, "SeasonWindow1")?
            }
            b"SeasonWindow2" => {
                self.season_window2 = ctx.decode_field(val, Self::TABLE_NAME, "SeasonWindow2")?
            }
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            b"_Name1" => self._name1 = ctx.decode_field(val, Self::TABLE_NAME, "_Name1")?,
            b"_Name2" => self._name2 = ctx.decode_field(val, Self::TABLE_NAME, "_Name2")?,
            b"_Name3" => self._name3 = ctx.decode_field(val, Self::TABLE_NAME, "_Name3")?,
            b"_SeasonDes" => {
                self._season_des = ctx.decode_field(val, Self::TABLE_NAME, "_SeasonDes")?
            }
            b"_SeasonDes2" => {
                self._season_des2 = ctx.decode_field(val, Self::TABLE_NAME, "_SeasonDes2")?
            }
            b"_SeasonIntroText" => {
                self._season_intro_text =
                    ctx.decode_field(val, Self::TABLE_NAME, "_SeasonIntroText")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_field_3(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"_SeasonText" => {
                self._season_text = ctx.decode_field(val, Self::TABLE_NAME, "_SeasonText")?
            }
            b"_SeasonWindow1" => {
                self._season_window1 = ctx.decode_field(val, Self::TABLE_NAME, "_SeasonWindow1")?
            }
            b"_SeasonWindow2" => {
                self._season_window2 = ctx.decode_field(val, Self::TABLE_NAME, "_SeasonWindow2")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigSeasonRow {
    const TABLE_NAME: &'static str = "config_season";

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

/// Typed `DataTable` for `config_season.out`.
pub type ConfigSeasonTable = DataTable<u16, ConfigSeasonRow>;

/// Parses `config_season.out` into a [`ConfigSeasonTable`].
pub fn parse_config_season(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSeasonTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_season_handbook.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSeasonHandbookRow {
    pub avatarframe_reward: u32,
    pub end_time: u8,
    pub gift_reward: Box<str>,
    pub hero_character_free: u8,
    pub hero_character_id: u16,
    pub id: i32,
    pub is_preseason: bool,
    pub key_level: Box<str>,
    pub level_point: Box<str>,
    pub normal_reward: Box<str>,
    pub open_time: u8,
    pub point_cost: Box<str>,
    pub season_id: u16,
    pub seasons: Box<[u16]>,
    pub tab: Box<str>,
    pub tips_time: u8,
    pub title: Box<str>,
    pub vip_display_mode: u8,
    pub _tab: Box<str>,
    pub _title: Box<str>,
}

impl ConfigSeasonHandbookRow {
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AvatarframeReward" => {
                self.avatarframe_reward =
                    ctx.decode_field(val, Self::TABLE_NAME, "AvatarframeReward")?
            }
            b"EndTime" => self.end_time = ctx.decode_field(val, Self::TABLE_NAME, "EndTime")?,
            b"GiftReward" => {
                self.gift_reward = ctx.decode_field(val, Self::TABLE_NAME, "GiftReward")?
            }
            b"HeroCharacterFree" => {
                self.hero_character_free =
                    ctx.decode_field(val, Self::TABLE_NAME, "HeroCharacterFree")?
            }
            b"HeroCharacterId" => {
                self.hero_character_id =
                    ctx.decode_field(val, Self::TABLE_NAME, "HeroCharacterId")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsPreseason" => {
                self.is_preseason = ctx.decode_field(val, Self::TABLE_NAME, "IsPreseason")?
            }
            b"KeyLevel" => self.key_level = ctx.decode_field(val, Self::TABLE_NAME, "KeyLevel")?,
            b"LevelPoint" => {
                self.level_point = ctx.decode_field(val, Self::TABLE_NAME, "LevelPoint")?
            }
            b"NormalReward" => {
                self.normal_reward = ctx.decode_field(val, Self::TABLE_NAME, "NormalReward")?
            }
            b"OpenTime" => self.open_time = ctx.decode_field(val, Self::TABLE_NAME, "OpenTime")?,
            b"PointCost" => {
                self.point_cost = ctx.decode_field(val, Self::TABLE_NAME, "PointCost")?
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
            b"SeasonId" => self.season_id = ctx.decode_field(val, Self::TABLE_NAME, "SeasonId")?,
            b"Seasons" => self.seasons = ctx.decode_field(val, Self::TABLE_NAME, "Seasons")?,
            b"Tab" => self.tab = ctx.decode_field(val, Self::TABLE_NAME, "Tab")?,
            b"TipsTime" => self.tips_time = ctx.decode_field(val, Self::TABLE_NAME, "TipsTime")?,
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"VipDisplayMode" => {
                self.vip_display_mode = ctx.decode_field(val, Self::TABLE_NAME, "VipDisplayMode")?
            }
            b"_Tab" => self._tab = ctx.decode_field(val, Self::TABLE_NAME, "_Tab")?,
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigSeasonHandbookRow {
    const TABLE_NAME: &'static str = "config_season_handbook";

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

/// Typed `DataTable` for `config_season_handbook.out`.
pub type ConfigSeasonHandbookTable = DataTable<u32, ConfigSeasonHandbookRow>;

/// Parses `config_season_handbook.out` into a [`ConfigSeasonHandbookTable`].
pub fn parse_config_season_handbook(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSeasonHandbookTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_season_handbook_tab.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSeasonHandbookTabRow {
    pub id: i8,
    pub season_id: u32,
    pub tab_barindex: u8,
    pub tab_id: u32,
    pub tab_title: Box<str>,
    pub _tab_title: Box<str>,
}

impl DataCfgRow for ConfigSeasonHandbookTabRow {
    const TABLE_NAME: &'static str = "config_season_handbook_tab";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"SeasonId" => self.season_id = ctx.decode_field(val, Self::TABLE_NAME, "SeasonId")?,
            b"TabBarindex" => {
                self.tab_barindex = ctx.decode_field(val, Self::TABLE_NAME, "TabBarindex")?
            }
            b"TabId" => self.tab_id = ctx.decode_field(val, Self::TABLE_NAME, "TabId")?,
            b"TabTitle" => self.tab_title = ctx.decode_field(val, Self::TABLE_NAME, "TabTitle")?,
            b"_TabTitle" => {
                self._tab_title = ctx.decode_field(val, Self::TABLE_NAME, "_TabTitle")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_season_handbook_tab.out`.
pub type ConfigSeasonHandbookTabTable = DataTable<u8, ConfigSeasonHandbookTabRow>;

/// Parses `config_season_handbook_tab.out` into a [`ConfigSeasonHandbookTabTable`].
pub fn parse_config_season_handbook_tab(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSeasonHandbookTabTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_seasonplay_entrance.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSeasonplayEntranceRow {
    pub id: i8,
    pub order: u8,
}

impl DataCfgRow for ConfigSeasonplayEntranceRow {
    const TABLE_NAME: &'static str = "config_seasonplay_entrance";

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_seasonplay_entrance.out`.
pub type ConfigSeasonplayEntranceTable = DataTable<u8, ConfigSeasonplayEntranceRow>;

/// Parses `config_seasonplay_entrance.out` into a [`ConfigSeasonplayEntranceTable`].
pub fn parse_config_seasonplay_entrance(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSeasonplayEntranceTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

fn verify_season_chunk_0(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_enum_gameplay_season_plague" => {
            Some(parse_config_enum_gameplay_season_plague(buf, scratch).map(|t| t.len()))
        }
        "config_enum_season_tycoon" => {
            Some(parse_config_enum_season_tycoon(buf, scratch).map(|t| t.len()))
        }
        "config_s11_gamble" => Some(parse_config_s11_gamble(buf, scratch).map(|t| t.len())),
        "config_s11_gameplay_hero_anim" => {
            Some(parse_config_s11_gameplay_hero_anim(buf, scratch).map(|t| t.len()))
        }
        "config_s12gameplay_talenttree" => {
            Some(parse_config_s12gameplay_talenttree(buf, scratch).map(|t| t.len()))
        }
        "config_s15_gameplay_suprise" => {
            Some(parse_config_s15_gameplay_suprise(buf, scratch).map(|t| t.len()))
        }
        "config_s15_gameplay_surprise_info" => {
            Some(parse_config_s15_gameplay_surprise_info(buf, scratch).map(|t| t.len()))
        }
        "config_s15_gameplay_talenttree" => {
            Some(parse_config_s15_gameplay_talenttree(buf, scratch).map(|t| t.len()))
        }
        "config_s15_gameplay_theme" => {
            Some(parse_config_s15_gameplay_theme(buf, scratch).map(|t| t.len()))
        }
        "config_s15_gameplay_theme_info" => {
            Some(parse_config_s15_gameplay_theme_info(buf, scratch).map(|t| t.len()))
        }
        "config_s16_gameplay_attribute" => {
            Some(parse_config_s16_gameplay_attribute(buf, scratch).map(|t| t.len()))
        }
        "config_s16_gameplay_colored_reward" => {
            Some(parse_config_s16_gameplay_colored_reward(buf, scratch).map(|t| t.len()))
        }
        "config_s16_gameplay_enum" => {
            Some(parse_config_s16_gameplay_enum(buf, scratch).map(|t| t.len()))
        }
        "config_s16_gameplay_event" => {
            Some(parse_config_s16_gameplay_event(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

fn verify_season_chunk_1(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_s16_gameplay_log" => {
            Some(parse_config_s16_gameplay_log(buf, scratch).map(|t| t.len()))
        }
        "config_s16_gameplay_relic" => {
            Some(parse_config_s16_gameplay_relic(buf, scratch).map(|t| t.len()))
        }
        "config_s16_gameplay_task" => {
            Some(parse_config_s16_gameplay_task(buf, scratch).map(|t| t.len()))
        }
        "config_s5_baddream_affix" => {
            Some(parse_config_s5_baddream_affix(buf, scratch).map(|t| t.len()))
        }
        "config_s5_gooddream_affix" => {
            Some(parse_config_s5_gooddream_affix(buf, scratch).map(|t| t.len()))
        }
        "config_s5_gooddream_affix_tips_hint" => {
            Some(parse_config_s5_gooddream_affix_tips_hint(buf, scratch).map(|t| t.len()))
        }
        "config_s5_gooddream_reward_quality_ui" => {
            Some(parse_config_s5_gooddream_reward_quality_ui(buf, scratch).map(|t| t.len()))
        }
        "config_s5_gooddream_reward_type" => {
            Some(parse_config_s5_gooddream_reward_type(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_attribute" => {
            Some(parse_config_s6_gameplay_attribute(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_buff" => {
            Some(parse_config_s6_gameplay_buff(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_dungeon" => {
            Some(parse_config_s6_gameplay_dungeon(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_encounter" => {
            Some(parse_config_s6_gameplay_encounter(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_event" => {
            Some(parse_config_s6_gameplay_event(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_interaction" => {
            Some(parse_config_s6_gameplay_interaction(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

fn verify_season_chunk_2(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_s6_gameplay_item" => {
            Some(parse_config_s6_gameplay_item(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_leveldiffculty" => {
            Some(parse_config_s6_gameplay_leveldiffculty(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_mechanism" => {
            Some(parse_config_s6_gameplay_mechanism(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_monster" => {
            Some(parse_config_s6_gameplay_monster(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_narrtive" => {
            Some(parse_config_s6_gameplay_narrtive(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_preparation" => {
            Some(parse_config_s6_gameplay_preparation(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_relic" => {
            Some(parse_config_s6_gameplay_relic(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_talent" => {
            Some(parse_config_s6_gameplay_talent(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_tile" => {
            Some(parse_config_s6_gameplay_tile(buf, scratch).map(|t| t.len()))
        }
        "config_s8_gameplay_difficulty" => {
            Some(parse_config_s8_gameplay_difficulty(buf, scratch).map(|t| t.len()))
        }
        "config_s8_gameplay_itemeffect" => {
            Some(parse_config_s8_gameplay_itemeffect(buf, scratch).map(|t| t.len()))
        }
        "config_s8_gameplay_plotitem" => {
            Some(parse_config_s8_gameplay_plotitem(buf, scratch).map(|t| t.len()))
        }
        "config_s8_gameplay_revive" => {
            Some(parse_config_s8_gameplay_revive(buf, scratch).map(|t| t.len()))
        }
        "config_s8_gameplay_selectableeffect" => {
            Some(parse_config_s8_gameplay_selectableeffect(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

fn verify_season_chunk_3(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_s8_gameplay_talent" => {
            Some(parse_config_s8_gameplay_talent(buf, scratch).map(|t| t.len()))
        }
        "config_s9_gameplay_card" => {
            Some(parse_config_s9_gameplay_card(buf, scratch).map(|t| t.len()))
        }
        "config_s9_gameplay_event" => {
            Some(parse_config_s9_gameplay_event(buf, scratch).map(|t| t.len()))
        }
        "config_s9_gameplay_interaction" => {
            Some(parse_config_s9_gameplay_interaction(buf, scratch).map(|t| t.len()))
        }
        "config_s9_gameplay_mapgameplay" => {
            Some(parse_config_s9_gameplay_mapgameplay(buf, scratch).map(|t| t.len()))
        }
        "config_s9_gameplay_tile" => {
            Some(parse_config_s9_gameplay_tile(buf, scratch).map(|t| t.len()))
        }
        "config_season" => Some(parse_config_season(buf, scratch).map(|t| t.len())),
        "config_season_handbook" => {
            Some(parse_config_season_handbook(buf, scratch).map(|t| t.len()))
        }
        "config_season_handbook_tab" => {
            Some(parse_config_season_handbook_tab(buf, scratch).map(|t| t.len()))
        }
        "config_seasonplay_entrance" => {
            Some(parse_config_seasonplay_entrance(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

pub(crate) fn verify_season_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    if let Some(res) = verify_season_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_season_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_season_chunk_2(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_season_chunk_3(stem, buf, scratch) {
        return Some(res);
    }
    None
}

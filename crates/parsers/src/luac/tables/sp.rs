//! Auto-generated `LuaTable/DataCfg` types and parsers for the `sp` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;

/// Entry for `config_sp_area_v2.TalentpointLimit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpAreaV2TalentpointLimitEntry {
    pub area_level: u8,
    pub talent_point_limit: u8,
}

impl FromEvalVal for SpAreaV2TalentpointLimitEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (area_level, talent_point_limit) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            area_level,
            talent_point_limit,
        })
    }
}

/// Entry for `config_sp_area_v2.UnlockReq`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpAreaV2UnlockReqEntry {
    pub area_id: u16,
    pub area_level: u8,
}

impl FromEvalVal for SpAreaV2UnlockReqEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (area_id, area_level) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            area_id,
            area_level,
        })
    }
}

/// Row record for `config_sp_area_v2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpAreaV2Row {
    pub background: Box<str>,
    pub beacon_icon: Box<str>,
    pub frame_type: u8,
    pub icon: Box<str>,
    pub icon_bg_hard: Box<str>,
    pub id: i16,
    pub initial_flame1: u8,
    pub max_flame1: u8,
    pub max_flame2: u8,
    pub name: Box<str>,
    pub neighbor_area: Box<[u16]>,
    pub phase: u8,
    pub place: Box<[i16]>,
    pub require_flame1: u8,
    pub talentpoint_limit: Box<[SpAreaV2TalentpointLimitEntry]>,
    pub title_icon: Box<str>,
    pub unlock_req: Box<[SpAreaV2UnlockReqEntry]>,
    pub unlock_req_text: Box<str>,
    pub _name: Box<str>,
    pub _unlock_req_text: Box<str>,
}

impl ConfigSpAreaV2Row {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Background" => {
                self.background = ctx.decode_field(val, Self::TABLE_NAME, "Background")?
            }
            b"BeaconIcon" => {
                self.beacon_icon = ctx.decode_field(val, Self::TABLE_NAME, "BeaconIcon")?
            }
            b"FrameType" => {
                self.frame_type = ctx.decode_field(val, Self::TABLE_NAME, "FrameType")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"IconBgHard" => {
                self.icon_bg_hard = ctx.decode_field(val, Self::TABLE_NAME, "IconBgHard")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InitialFlame1" => {
                self.initial_flame1 = ctx.decode_field(val, Self::TABLE_NAME, "InitialFlame1")?
            }
            b"MaxFlame1" => {
                self.max_flame1 = ctx.decode_field(val, Self::TABLE_NAME, "MaxFlame1")?
            }
            b"MaxFlame2" => {
                self.max_flame2 = ctx.decode_field(val, Self::TABLE_NAME, "MaxFlame2")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"NeighborArea" => {
                self.neighbor_area = ctx.decode_field(val, Self::TABLE_NAME, "NeighborArea")?
            }
            b"Phase" => self.phase = ctx.decode_field(val, Self::TABLE_NAME, "Phase")?,
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
            b"Place" => self.place = ctx.decode_field(val, Self::TABLE_NAME, "Place")?,
            b"RequireFlame1" => {
                self.require_flame1 = ctx.decode_field(val, Self::TABLE_NAME, "RequireFlame1")?
            }
            b"TalentpointLimit" => {
                self.talentpoint_limit =
                    ctx.decode_field(val, Self::TABLE_NAME, "TalentpointLimit")?
            }
            b"TitleIcon" => {
                self.title_icon = ctx.decode_field(val, Self::TABLE_NAME, "TitleIcon")?
            }
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            b"UnlockReqText" => {
                self.unlock_req_text = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReqText")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_UnlockReqText" => {
                self._unlock_req_text = ctx.decode_field(val, Self::TABLE_NAME, "_UnlockReqText")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigSpAreaV2Row {
    const TABLE_NAME: &'static str = "config_sp_area_v2";

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

/// Typed `DataTable` for `config_sp_area_v2.out`.
pub type ConfigSpAreaV2Table = DataTable<u16, ConfigSpAreaV2Row>;

/// Parses `config_sp_area_v2.out` into a [`ConfigSpAreaV2Table`].
pub fn parse_config_sp_area_v2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpAreaV2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_sp_card_class_v2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpCardClassV2Row {
    pub id: i16,
    pub name: Box<str>,
    pub repel_class: [u8; 0],
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigSpCardClassV2Row {
    const TABLE_NAME: &'static str = "config_sp_card_class_v2";

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
            b"RepelClass" => {
                self.repel_class = ctx.decode_field(val, Self::TABLE_NAME, "RepelClass")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sp_card_class_v2.out`.
pub type ConfigSpCardClassV2Table = DataTable<u8, ConfigSpCardClassV2Row>;

/// Parses `config_sp_card_class_v2.out` into a [`ConfigSpCardClassV2Table`].
pub fn parse_config_sp_card_class_v2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpCardClassV2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_sp_card_keyword_v2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpCardKeywordV2Row {
    pub effect: u8,
    pub icon: Box<str>,
    pub id: i16,
    pub name: Box<str>,
    pub num: i8,
    pub p1: u16,
    pub p2: i8,
    pub p3: u8,
    pub p4: u8,
    pub p5: u8,
    pub time: u8,
    pub times: Box<[u8]>,
    pub _name: Box<str>,
}

impl ConfigSpCardKeywordV2Row {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Effect" => self.effect = ctx.decode_field(val, Self::TABLE_NAME, "Effect")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Num" => self.num = ctx.decode_field(val, Self::TABLE_NAME, "Num")?,
            b"P1" => self.p1 = ctx.decode_field(val, Self::TABLE_NAME, "P1")?,
            b"P2" => self.p2 = ctx.decode_field(val, Self::TABLE_NAME, "P2")?,
            b"P3" => self.p3 = ctx.decode_field(val, Self::TABLE_NAME, "P3")?,
            b"P4" => self.p4 = ctx.decode_field(val, Self::TABLE_NAME, "P4")?,
            b"P5" => self.p5 = ctx.decode_field(val, Self::TABLE_NAME, "P5")?,
            b"Time" => self.time = ctx.decode_field(val, Self::TABLE_NAME, "Time")?,
            b"Times" => self.times = ctx.decode_field(val, Self::TABLE_NAME, "Times")?,
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
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigSpCardKeywordV2Row {
    const TABLE_NAME: &'static str = "config_sp_card_keyword_v2";

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

/// Typed `DataTable` for `config_sp_card_keyword_v2.out`.
pub type ConfigSpCardKeywordV2Table = DataTable<u16, ConfigSpCardKeywordV2Row>;

/// Parses `config_sp_card_keyword_v2.out` into a [`ConfigSpCardKeywordV2Table`].
pub fn parse_config_sp_card_keyword_v2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpCardKeywordV2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_sp_card_v2.Keyword`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpCardV2Keyword {
    pub keyword_id: u8,
    pub value: u8,
}

impl FromEvalVal for SpCardV2Keyword {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (keyword_id, value) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { keyword_id, value })
    }
}

/// Entry for `config_sp_card_v2.Modifiers`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpCardV2ModifiersEntry {
    pub modifier_id: u32,
    pub keywords: Box<[u8]>,
}

impl FromEvalVal for SpCardV2ModifiersEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (modifier_id, keywords) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            modifier_id,
            keywords,
        })
    }
}

/// Row record for `config_sp_card_v2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpCardV2Row {
    pub appear_info: Box<str>,
    pub appear_time: Box<[u8]>,
    pub class: u8,
    pub des: Box<str>,
    pub evoke_chance: u8,
    pub evoke_id: u8,
    pub frame: Box<str>,
    pub get_time: [u8; 0],
    pub group: u32,
    pub hidden_modifier: [u8; 0],
    pub icon: Box<str>,
    pub id: i32,
    pub key_modifiers: [u8; 0],
    pub keyword: SpCardV2Keyword,
    pub keywords: [u8; 0],
    pub lv: u8,
    pub max_progress: u8,
    pub modifier: [u8; 0],
    pub modifiers: Box<[SpCardV2ModifiersEntry]>,
    pub name: Box<str>,
    pub rarity: u8,
    pub rd_group: u8,
    pub show_in_card_list: Box<[u8]>,
    pub small_frame: Box<str>,
    pub small_icon: Box<str>,
    pub r#type: u8,
    pub unlock_req: u8,
    pub weight: u8,
    pub _appear_info: Box<str>,
    pub _des: Box<str>,
    pub _name: Box<str>,
}

impl ConfigSpCardV2Row {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AppearInfo" => {
                self.appear_info = ctx.decode_field(val, Self::TABLE_NAME, "AppearInfo")?
            }
            b"AppearTime" => {
                self.appear_time = ctx.decode_field(val, Self::TABLE_NAME, "AppearTime")?
            }
            b"Class" => self.class = ctx.decode_field(val, Self::TABLE_NAME, "Class")?,
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"EvokeChance" => {
                self.evoke_chance = ctx.decode_field(val, Self::TABLE_NAME, "EvokeChance")?
            }
            b"EvokeId" => self.evoke_id = ctx.decode_field(val, Self::TABLE_NAME, "EvokeId")?,
            b"Frame" => self.frame = ctx.decode_field(val, Self::TABLE_NAME, "Frame")?,
            b"GetTime" => self.get_time = ctx.decode_field(val, Self::TABLE_NAME, "GetTime")?,
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"HiddenModifier" => {
                self.hidden_modifier = ctx.decode_field(val, Self::TABLE_NAME, "HiddenModifier")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
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
            b"KeyModifiers" => {
                self.key_modifiers = ctx.decode_field(val, Self::TABLE_NAME, "KeyModifiers")?
            }
            b"Keyword" => self.keyword = ctx.decode_field(val, Self::TABLE_NAME, "Keyword")?,
            b"Keywords" => self.keywords = ctx.decode_field(val, Self::TABLE_NAME, "Keywords")?,
            b"Lv" => self.lv = ctx.decode_field(val, Self::TABLE_NAME, "Lv")?,
            b"MaxProgress" => {
                self.max_progress = ctx.decode_field(val, Self::TABLE_NAME, "MaxProgress")?
            }
            b"Modifier" => self.modifier = ctx.decode_field(val, Self::TABLE_NAME, "Modifier")?,
            b"Modifiers" => {
                self.modifiers = ctx.decode_field(val, Self::TABLE_NAME, "Modifiers")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"RdGroup" => self.rd_group = ctx.decode_field(val, Self::TABLE_NAME, "RdGroup")?,
            b"ShowInCardList" => {
                self.show_in_card_list =
                    ctx.decode_field(val, Self::TABLE_NAME, "ShowInCardList")?
            }
            b"SmallFrame" => {
                self.small_frame = ctx.decode_field(val, Self::TABLE_NAME, "SmallFrame")?
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
            b"SmallIcon" => {
                self.small_icon = ctx.decode_field(val, Self::TABLE_NAME, "SmallIcon")?
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_AppearInfo" => {
                self._appear_info = ctx.decode_field(val, Self::TABLE_NAME, "_AppearInfo")?
            }
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigSpCardV2Row {
    const TABLE_NAME: &'static str = "config_sp_card_v2";

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

/// Typed `DataTable` for `config_sp_card_v2.out`.
pub type ConfigSpCardV2Table = DataTable<u32, ConfigSpCardV2Row>;

/// Parses `config_sp_card_v2.out` into a [`ConfigSpCardV2Table`].
pub fn parse_config_sp_card_v2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpCardV2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_sp_corrosion_combine.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpCorrosionCombineRow {
    pub boss_name: Box<str>,
    pub boss_title: Box<str>,
    pub combine: Box<[u32]>,
    pub id: i8,
    pub map_id: u32,
    pub mid_icons: Box<str>,
    pub pre_id: Box<[u8]>,
    pub sys_unlock: u8,
    pub whisper: Box<str>,
    pub _boss_name: Box<str>,
    pub _boss_title: Box<str>,
    pub _whisper: Box<str>,
}

impl DataCfgRow for ConfigSpCorrosionCombineRow {
    const TABLE_NAME: &'static str = "config_sp_corrosion_combine";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BossName" => self.boss_name = ctx.decode_field(val, Self::TABLE_NAME, "BossName")?,
            b"BossTitle" => {
                self.boss_title = ctx.decode_field(val, Self::TABLE_NAME, "BossTitle")?
            }
            b"Combine" => self.combine = ctx.decode_field(val, Self::TABLE_NAME, "Combine")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MapId" => self.map_id = ctx.decode_field(val, Self::TABLE_NAME, "MapId")?,
            b"MidIcons" => self.mid_icons = ctx.decode_field(val, Self::TABLE_NAME, "MidIcons")?,
            b"PreId" => self.pre_id = ctx.decode_field(val, Self::TABLE_NAME, "PreId")?,
            b"SysUnlock" => {
                self.sys_unlock = ctx.decode_field(val, Self::TABLE_NAME, "SysUnlock")?
            }
            b"Whisper" => self.whisper = ctx.decode_field(val, Self::TABLE_NAME, "Whisper")?,
            b"_BossName" => {
                self._boss_name = ctx.decode_field(val, Self::TABLE_NAME, "_BossName")?
            }
            b"_BossTitle" => {
                self._boss_title = ctx.decode_field(val, Self::TABLE_NAME, "_BossTitle")?
            }
            b"_Whisper" => self._whisper = ctx.decode_field(val, Self::TABLE_NAME, "_Whisper")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sp_corrosion_combine.out`.
pub type ConfigSpCorrosionCombineTable = DataTable<u8, ConfigSpCorrosionCombineRow>;

/// Parses `config_sp_corrosion_combine.out` into a [`ConfigSpCorrosionCombineTable`].
pub fn parse_config_sp_corrosion_combine(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpCorrosionCombineTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_sp_gameplayboss_combine.Combine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpGameplaybossCombineCombineEntry {
    pub combine_key: u8,
    pub boss_id: u32,
}

impl FromEvalVal for SpGameplaybossCombineCombineEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (combine_key, boss_id) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            combine_key,
            boss_id,
        })
    }
}

/// Row record for `config_sp_gameplayboss_combine.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpGameplaybossCombineRow {
    pub boss_name: Box<str>,
    pub boss_title: Box<str>,
    pub combine: Box<[SpGameplaybossCombineCombineEntry]>,
    pub dps: Box<str>,
    pub enter_hint: Box<str>,
    pub func_id: u16,
    pub id: i8,
    pub level: u8,
    pub map_id: u32,
    pub _boss_name: Box<str>,
    pub _boss_title: Box<str>,
}

impl DataCfgRow for ConfigSpGameplaybossCombineRow {
    const TABLE_NAME: &'static str = "config_sp_gameplayboss_combine";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BossName" => self.boss_name = ctx.decode_field(val, Self::TABLE_NAME, "BossName")?,
            b"BossTitle" => {
                self.boss_title = ctx.decode_field(val, Self::TABLE_NAME, "BossTitle")?
            }
            b"Combine" => self.combine = ctx.decode_field(val, Self::TABLE_NAME, "Combine")?,
            b"Dps" => self.dps = ctx.decode_field(val, Self::TABLE_NAME, "Dps")?,
            b"EnterHint" => {
                self.enter_hint = ctx.decode_field(val, Self::TABLE_NAME, "EnterHint")?
            }
            b"FuncId" => self.func_id = ctx.decode_field(val, Self::TABLE_NAME, "FuncId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"MapId" => self.map_id = ctx.decode_field(val, Self::TABLE_NAME, "MapId")?,
            b"_BossName" => {
                self._boss_name = ctx.decode_field(val, Self::TABLE_NAME, "_BossName")?
            }
            b"_BossTitle" => {
                self._boss_title = ctx.decode_field(val, Self::TABLE_NAME, "_BossTitle")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sp_gameplayboss_combine.out`.
pub type ConfigSpGameplaybossCombineTable = DataTable<u8, ConfigSpGameplaybossCombineRow>;

/// Parses `config_sp_gameplayboss_combine.out` into a [`ConfigSpGameplaybossCombineTable`].
pub fn parse_config_sp_gameplayboss_combine(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpGameplaybossCombineTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_sp_gameplayboss_list.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpGameplaybossListRow {
    pub boss_title: Box<str>,
    pub dps: Box<str>,
    pub id: i16,
    pub position: u8,
    pub season_id: u16,
    pub _boss_title: Box<str>,
}

impl DataCfgRow for ConfigSpGameplaybossListRow {
    const TABLE_NAME: &'static str = "config_sp_gameplayboss_list";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BossTitle" => {
                self.boss_title = ctx.decode_field(val, Self::TABLE_NAME, "BossTitle")?
            }
            b"Dps" => self.dps = ctx.decode_field(val, Self::TABLE_NAME, "Dps")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Position" => self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?,
            b"SeasonId" => self.season_id = ctx.decode_field(val, Self::TABLE_NAME, "SeasonId")?,
            b"_BossTitle" => {
                self._boss_title = ctx.decode_field(val, Self::TABLE_NAME, "_BossTitle")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sp_gameplayboss_list.out`.
pub type ConfigSpGameplaybossListTable = DataTable<u16, ConfigSpGameplaybossListRow>;

/// Parses `config_sp_gameplayboss_list.out` into a [`ConfigSpGameplaybossListTable`].
pub fn parse_config_sp_gameplayboss_list(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpGameplaybossListTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_sp_gameplayboss_socket.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpGameplaybossSocketRow {
    pub back_icon: Box<str>,
    pub func_id: u16,
    pub id: i8,
    pub item_id: Box<[u32]>,
    pub lock_hint: Box<str>,
    pub lock_view: u8,
    pub season_id: u16,
    pub unlock_req: Box<[u32]>,
}

impl DataCfgRow for ConfigSpGameplaybossSocketRow {
    const TABLE_NAME: &'static str = "config_sp_gameplayboss_socket";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BackIcon" => self.back_icon = ctx.decode_field(val, Self::TABLE_NAME, "BackIcon")?,
            b"FuncId" => self.func_id = ctx.decode_field(val, Self::TABLE_NAME, "FuncId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemId" => self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?,
            b"LockHint" => self.lock_hint = ctx.decode_field(val, Self::TABLE_NAME, "LockHint")?,
            b"LockView" => self.lock_view = ctx.decode_field(val, Self::TABLE_NAME, "LockView")?,
            b"SeasonId" => self.season_id = ctx.decode_field(val, Self::TABLE_NAME, "SeasonId")?,
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sp_gameplayboss_socket.out`.
pub type ConfigSpGameplaybossSocketTable = DataTable<u8, ConfigSpGameplaybossSocketRow>;

/// Parses `config_sp_gameplayboss_socket.out` into a [`ConfigSpGameplaybossSocketTable`].
pub fn parse_config_sp_gameplayboss_socket(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpGameplaybossSocketTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_sp_guide_chapter_v2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpGuideChapterV2Row {
    pub id: i8,
    pub is_locked: bool,
    pub name: Box<str>,
    pub progress: u8,
    pub reward: u32,
    pub unlock: u8,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigSpGuideChapterV2Row {
    const TABLE_NAME: &'static str = "config_sp_guide_chapter_v2";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsLocked" => self.is_locked = ctx.decode_field(val, Self::TABLE_NAME, "IsLocked")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Progress" => self.progress = ctx.decode_field(val, Self::TABLE_NAME, "Progress")?,
            b"Reward" => self.reward = ctx.decode_field(val, Self::TABLE_NAME, "Reward")?,
            b"Unlock" => self.unlock = ctx.decode_field(val, Self::TABLE_NAME, "Unlock")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sp_guide_chapter_v2.out`.
pub type ConfigSpGuideChapterV2Table = DataTable<u8, ConfigSpGuideChapterV2Row>;

/// Parses `config_sp_guide_chapter_v2.out` into a [`ConfigSpGuideChapterV2Table`].
pub fn parse_config_sp_guide_chapter_v2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpGuideChapterV2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_sp_guide_task_v2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpGuideTaskV2Row {
    pub chapter: u8,
    pub event: u32,
    pub id: i16,
    pub info: Box<str>,
    pub optional: bool,
    pub reward: u16,
    pub statistics: u8,
    pub _info: Box<str>,
}

impl DataCfgRow for ConfigSpGuideTaskV2Row {
    const TABLE_NAME: &'static str = "config_sp_guide_task_v2";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Chapter" => self.chapter = ctx.decode_field(val, Self::TABLE_NAME, "Chapter")?,
            b"Event" => self.event = ctx.decode_field(val, Self::TABLE_NAME, "Event")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Info" => self.info = ctx.decode_field(val, Self::TABLE_NAME, "Info")?,
            b"Optional" => self.optional = ctx.decode_field(val, Self::TABLE_NAME, "Optional")?,
            b"Reward" => self.reward = ctx.decode_field(val, Self::TABLE_NAME, "Reward")?,
            b"Statistics" => {
                self.statistics = ctx.decode_field(val, Self::TABLE_NAME, "Statistics")?
            }
            b"_Info" => self._info = ctx.decode_field(val, Self::TABLE_NAME, "_Info")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sp_guide_task_v2.out`.
pub type ConfigSpGuideTaskV2Table = DataTable<u16, ConfigSpGuideTaskV2Row>;

/// Parses `config_sp_guide_task_v2.out` into a [`ConfigSpGuideTaskV2Table`].
pub fn parse_config_sp_guide_task_v2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpGuideTaskV2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_sp_tree_cluster.UnlockReq`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpTreeClusterUnlockReqEntry {
    pub area_id: u16,
    pub area_level: u8,
}

impl FromEvalVal for SpTreeClusterUnlockReqEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (area_id, area_level) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            area_id,
            area_level,
        })
    }
}

/// Row record for `config_sp_tree_cluster.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpTreeClusterRow {
    pub back_image: Box<str>,
    pub icon: Box<str>,
    pub id: i16,
    pub line_color: u16,
    pub name: Box<str>,
    pub node_color: u16,
    pub position: Box<[i16]>,
    pub season: Box<[u16]>,
    pub show_order: u8,
    pub tip_color: u16,
    pub r#type: u8,
    pub unlock_req: Box<[SpTreeClusterUnlockReqEntry]>,
    pub unlock_req_rank: u8,
    pub unlock_text: Box<str>,
    pub _name: Box<str>,
    pub _unlock_text: Box<str>,
}

impl ConfigSpTreeClusterRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"BackImage" => {
                self.back_image = ctx.decode_field(val, Self::TABLE_NAME, "BackImage")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LineColor" => {
                self.line_color = ctx.decode_field(val, Self::TABLE_NAME, "LineColor")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"NodeColor" => {
                self.node_color = ctx.decode_field(val, Self::TABLE_NAME, "NodeColor")?
            }
            b"Position" => self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?,
            b"Season" => self.season = ctx.decode_field(val, Self::TABLE_NAME, "Season")?,
            b"ShowOrder" => {
                self.show_order = ctx.decode_field(val, Self::TABLE_NAME, "ShowOrder")?
            }
            b"TipColor" => self.tip_color = ctx.decode_field(val, Self::TABLE_NAME, "TipColor")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
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
            b"UnlockReqRank" => {
                self.unlock_req_rank = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReqRank")?
            }
            b"UnlockText" => {
                self.unlock_text = ctx.decode_field(val, Self::TABLE_NAME, "UnlockText")?
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_UnlockText" => {
                self._unlock_text = ctx.decode_field(val, Self::TABLE_NAME, "_UnlockText")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigSpTreeClusterRow {
    const TABLE_NAME: &'static str = "config_sp_tree_cluster";

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

/// Typed `DataTable` for `config_sp_tree_cluster.out`.
pub type ConfigSpTreeClusterTable = DataTable<u16, ConfigSpTreeClusterRow>;

/// Parses `config_sp_tree_cluster.out` into a [`ConfigSpTreeClusterTable`].
pub fn parse_config_sp_tree_cluster(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpTreeClusterTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_sp_tree_node.EnableCost`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpTreeNodeEnableCost {
    pub cost_key: Box<str>,
    pub count: u8,
}

impl FromEvalVal for SpTreeNodeEnableCost {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (cost_key, count) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { cost_key, count })
    }
}

/// Entry for `config_sp_tree_node.FunctionModifier`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpTreeNodeFunctionModifierEntry {
    pub modifier_id: u32,
    pub params: Box<[u8]>,
}

impl FromEvalVal for SpTreeNodeFunctionModifierEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (modifier_id, params) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            modifier_id,
            params,
        })
    }
}

/// Entry for `config_sp_tree_node.MapModifier`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpTreeNodeMapModifierEntry {
    pub modifier_id: u32,
    pub params: Box<[u8]>,
}

impl FromEvalVal for SpTreeNodeMapModifierEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (modifier_id, params) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            modifier_id,
            params,
        })
    }
}

/// Row record for `config_sp_tree_node.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpTreeNodeRow {
    pub cluster: u16,
    pub core_point_cost: u8,
    pub enable_cost: SpTreeNodeEnableCost,
    pub function_modifier: Box<[SpTreeNodeFunctionModifierEntry]>,
    pub icon: Box<str>,
    pub id: i32,
    pub link_node: Box<[u16]>,
    pub map_modifier: Box<[SpTreeNodeMapModifierEntry]>,
    pub name: Box<str>,
    pub point_cost: u8,
    pub position: Box<[i16]>,
    pub talentnode_type: u8,
    pub tips_icon: Box<str>,
    pub r#type: u8,
    pub _name: Box<str>,
}

impl ConfigSpTreeNodeRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Cluster" => self.cluster = ctx.decode_field(val, Self::TABLE_NAME, "Cluster")?,
            b"CorePointCost" => {
                self.core_point_cost = ctx.decode_field(val, Self::TABLE_NAME, "CorePointCost")?
            }
            b"EnableCost" => {
                self.enable_cost = ctx.decode_field(val, Self::TABLE_NAME, "EnableCost")?
            }
            b"FunctionModifier" => {
                self.function_modifier =
                    ctx.decode_field(val, Self::TABLE_NAME, "FunctionModifier")?
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LinkNode" => self.link_node = ctx.decode_field(val, Self::TABLE_NAME, "LinkNode")?,
            b"MapModifier" => {
                self.map_modifier = ctx.decode_field(val, Self::TABLE_NAME, "MapModifier")?
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"PointCost" => {
                self.point_cost = ctx.decode_field(val, Self::TABLE_NAME, "PointCost")?
            }
            b"Position" => self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?,
            b"TalentnodeType" => {
                self.talentnode_type = ctx.decode_field(val, Self::TABLE_NAME, "TalentnodeType")?
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
            b"TipsIcon" => self.tips_icon = ctx.decode_field(val, Self::TABLE_NAME, "TipsIcon")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigSpTreeNodeRow {
    const TABLE_NAME: &'static str = "config_sp_tree_node";

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

/// Typed `DataTable` for `config_sp_tree_node.out`.
pub type ConfigSpTreeNodeTable = DataTable<u16, ConfigSpTreeNodeRow>;

/// Parses `config_sp_tree_node.out` into a [`ConfigSpTreeNodeTable`].
pub fn parse_config_sp_tree_node(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpTreeNodeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_sp_unlock_v2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpUnlockV2Row {
    pub des: Box<str>,
    pub flame1: u8,
    pub icon: Box<str>,
    pub id: i16,
    pub priority: u8,
    pub title: Box<str>,
    pub _des: Box<str>,
    pub _title: Box<str>,
}

impl DataCfgRow for ConfigSpUnlockV2Row {
    const TABLE_NAME: &'static str = "config_sp_unlock_v2";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"Flame1" => self.flame1 = ctx.decode_field(val, Self::TABLE_NAME, "Flame1")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Priority" => self.priority = ctx.decode_field(val, Self::TABLE_NAME, "Priority")?,
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_sp_unlock_v2.out`.
pub type ConfigSpUnlockV2Table = DataTable<u16, ConfigSpUnlockV2Row>;

/// Parses `config_sp_unlock_v2.out` into a [`ConfigSpUnlockV2Table`].
pub fn parse_config_sp_unlock_v2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpUnlockV2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_sp_world_boss_v2.UnlockReq`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpWorldBossV2UnlockReqEntry {
    pub area_id: u16,
    pub area_level: u8,
}

impl FromEvalVal for SpWorldBossV2UnlockReqEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (area_id, area_level) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            area_id,
            area_level,
        })
    }
}

/// Row record for `config_sp_world_boss_v2.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSpWorldBossV2Row {
    pub boss_lvl: u8,
    pub boss_name: Box<str>,
    pub boss_title: Box<str>,
    pub comment: Box<str>,
    pub flame1_require: u8,
    pub id: i16,
    pub item_id: u32,
    pub item_num: u8,
    pub level_id: u16,
    pub open_area_level: u8,
    pub phase: u8,
    pub phase_unlock: u8,
    pub place: Box<[i8]>,
    pub progress: u8,
    pub rank: Box<[u8]>,
    pub unlock_req: Box<[SpWorldBossV2UnlockReqEntry]>,
    pub unlock_req_rank: u8,
    pub upgrade_type: u8,
    pub _boss_name: Box<str>,
    pub _boss_title: Box<str>,
}

impl ConfigSpWorldBossV2Row {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"BossLvl" => self.boss_lvl = ctx.decode_field(val, Self::TABLE_NAME, "BossLvl")?,
            b"BossName" => self.boss_name = ctx.decode_field(val, Self::TABLE_NAME, "BossName")?,
            b"BossTitle" => {
                self.boss_title = ctx.decode_field(val, Self::TABLE_NAME, "BossTitle")?
            }
            b"Comment" => self.comment = ctx.decode_field(val, Self::TABLE_NAME, "Comment")?,
            b"Flame1Require" => {
                self.flame1_require = ctx.decode_field(val, Self::TABLE_NAME, "Flame1Require")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemId" => self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?,
            b"ItemNum" => self.item_num = ctx.decode_field(val, Self::TABLE_NAME, "ItemNum")?,
            b"LevelId" => self.level_id = ctx.decode_field(val, Self::TABLE_NAME, "LevelId")?,
            b"OpenAreaLevel" => {
                self.open_area_level = ctx.decode_field(val, Self::TABLE_NAME, "OpenAreaLevel")?
            }
            b"Phase" => self.phase = ctx.decode_field(val, Self::TABLE_NAME, "Phase")?,
            b"PhaseUnlock" => {
                self.phase_unlock = ctx.decode_field(val, Self::TABLE_NAME, "PhaseUnlock")?
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
            b"Place" => self.place = ctx.decode_field(val, Self::TABLE_NAME, "Place")?,
            b"Progress" => self.progress = ctx.decode_field(val, Self::TABLE_NAME, "Progress")?,
            b"Rank" => self.rank = ctx.decode_field(val, Self::TABLE_NAME, "Rank")?,
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            b"UnlockReqRank" => {
                self.unlock_req_rank = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReqRank")?
            }
            b"UpgradeType" => {
                self.upgrade_type = ctx.decode_field(val, Self::TABLE_NAME, "UpgradeType")?
            }
            b"_BossName" => {
                self._boss_name = ctx.decode_field(val, Self::TABLE_NAME, "_BossName")?
            }
            b"_BossTitle" => {
                self._boss_title = ctx.decode_field(val, Self::TABLE_NAME, "_BossTitle")?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl DataCfgRow for ConfigSpWorldBossV2Row {
    const TABLE_NAME: &'static str = "config_sp_world_boss_v2";

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

/// Typed `DataTable` for `config_sp_world_boss_v2.out`.
pub type ConfigSpWorldBossV2Table = DataTable<u16, ConfigSpWorldBossV2Row>;

/// Parses `config_sp_world_boss_v2.out` into a [`ConfigSpWorldBossV2Table`].
pub fn parse_config_sp_world_boss_v2(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSpWorldBossV2Table, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

pub(crate) fn verify_sp_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_sp_area_v2" => Some(parse_config_sp_area_v2(buf, scratch).map(|t| t.len())),
        "config_sp_card_class_v2" => {
            Some(parse_config_sp_card_class_v2(buf, scratch).map(|t| t.len()))
        }
        "config_sp_card_keyword_v2" => {
            Some(parse_config_sp_card_keyword_v2(buf, scratch).map(|t| t.len()))
        }
        "config_sp_card_v2" => Some(parse_config_sp_card_v2(buf, scratch).map(|t| t.len())),
        "config_sp_corrosion_combine" => {
            Some(parse_config_sp_corrosion_combine(buf, scratch).map(|t| t.len()))
        }
        "config_sp_gameplayboss_combine" => {
            Some(parse_config_sp_gameplayboss_combine(buf, scratch).map(|t| t.len()))
        }
        "config_sp_gameplayboss_list" => {
            Some(parse_config_sp_gameplayboss_list(buf, scratch).map(|t| t.len()))
        }
        "config_sp_gameplayboss_socket" => {
            Some(parse_config_sp_gameplayboss_socket(buf, scratch).map(|t| t.len()))
        }
        "config_sp_guide_chapter_v2" => {
            Some(parse_config_sp_guide_chapter_v2(buf, scratch).map(|t| t.len()))
        }
        "config_sp_guide_task_v2" => {
            Some(parse_config_sp_guide_task_v2(buf, scratch).map(|t| t.len()))
        }
        "config_sp_tree_cluster" => {
            Some(parse_config_sp_tree_cluster(buf, scratch).map(|t| t.len()))
        }
        "config_sp_tree_node" => Some(parse_config_sp_tree_node(buf, scratch).map(|t| t.len())),
        "config_sp_unlock_v2" => Some(parse_config_sp_unlock_v2(buf, scratch).map(|t| t.len())),
        "config_sp_world_boss_v2" => {
            Some(parse_config_sp_world_boss_v2(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

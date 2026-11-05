//! Auto-generated `LuaTable/DataCfg` types and parsers for the `chat` domain.

use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;

/// Row record for `config_chat_moment.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigChatMomentRow {
    pub event: i8,
    pub moment_text: Box<str>,
    pub _moment_text: Box<str>,
}

impl DataCfgRow for ConfigChatMomentRow {
    const TABLE_NAME: &'static str = "config_chat_moment";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Event" => self.event = ctx.decode_field(val, Self::TABLE_NAME, "Event")?,
            b"MomentText" => {
                self.moment_text = ctx.decode_field(val, Self::TABLE_NAME, "MomentText")?
            }
            b"_MomentText" => {
                self._moment_text = ctx.decode_field(val, Self::TABLE_NAME, "_MomentText")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_chat_moment.out`.
pub type ConfigChatMomentTable = DataTable<u8, ConfigChatMomentRow>;

/// Parses `config_chat_moment.out` into a [`ConfigChatMomentTable`].
pub fn parse_config_chat_moment(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigChatMomentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_friend_gift_ceiling.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFriendGiftCeilingRow {
    pub ceiling_reset_time: u32,
    pub gift_ceiling: u8,
    pub id: i8,
    pub rule_id: u16,
    pub season_type: u8,
    pub tips: Box<str>,
    pub _tips: Box<str>,
}

impl DataCfgRow for ConfigFriendGiftCeilingRow {
    const TABLE_NAME: &'static str = "config_friend_gift_ceiling";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CeilingResetTime" => {
                self.ceiling_reset_time =
                    ctx.decode_field(val, Self::TABLE_NAME, "CeilingResetTime")?
            }
            b"GiftCeiling" => {
                self.gift_ceiling = ctx.decode_field(val, Self::TABLE_NAME, "GiftCeiling")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"RuleId" => self.rule_id = ctx.decode_field(val, Self::TABLE_NAME, "RuleId")?,
            b"SeasonType" => {
                self.season_type = ctx.decode_field(val, Self::TABLE_NAME, "SeasonType")?
            }
            b"Tips" => self.tips = ctx.decode_field(val, Self::TABLE_NAME, "Tips")?,
            b"_Tips" => self._tips = ctx.decode_field(val, Self::TABLE_NAME, "_Tips")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_friend_gift_ceiling.out`.
pub type ConfigFriendGiftCeilingTable = DataTable<u8, ConfigFriendGiftCeilingRow>;

/// Parses `config_friend_gift_ceiling.out` into a [`ConfigFriendGiftCeilingTable`].
pub fn parse_config_friend_gift_ceiling(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigFriendGiftCeilingTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_friend_gift_item.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigFriendGiftItemRow {
    pub ceiling_rule: u16,
    pub id: i8,
    pub is_bind: u8,
    pub item_id: u8,
    pub item_type: u8,
    pub priority: u8,
    pub rarity_level: u8,
    pub type1: u8,
    pub type2: u8,
    pub type3: u8,
    pub type4: u8,
}

impl DataCfgRow for ConfigFriendGiftItemRow {
    const TABLE_NAME: &'static str = "config_friend_gift_item";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CeilingRule" => {
                self.ceiling_rule = ctx.decode_field(val, Self::TABLE_NAME, "CeilingRule")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsBind" => self.is_bind = ctx.decode_field(val, Self::TABLE_NAME, "IsBind")?,
            b"ItemId" => self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?,
            b"ItemType" => self.item_type = ctx.decode_field(val, Self::TABLE_NAME, "ItemType")?,
            b"Priority" => self.priority = ctx.decode_field(val, Self::TABLE_NAME, "Priority")?,
            b"RarityLevel" => {
                self.rarity_level = ctx.decode_field(val, Self::TABLE_NAME, "RarityLevel")?
            }
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_friend_gift_item.out`.
pub type ConfigFriendGiftItemTable = DataTable<u8, ConfigFriendGiftItemRow>;

/// Parses `config_friend_gift_item.out` into a [`ConfigFriendGiftItemTable`].
pub fn parse_config_friend_gift_item(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigFriendGiftItemTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_guild_func.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuildFuncRow {
    pub func_name: Box<str>,
    pub id: i8,
    pub is_member_oper: bool,
    pub _func_name: Box<str>,
}

impl DataCfgRow for ConfigGuildFuncRow {
    const TABLE_NAME: &'static str = "config_guild_func";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"FuncName" => self.func_name = ctx.decode_field(val, Self::TABLE_NAME, "FuncName")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsMemberOper" => {
                self.is_member_oper = ctx.decode_field(val, Self::TABLE_NAME, "IsMemberOper")?
            }
            b"_FuncName" => {
                self._func_name = ctx.decode_field(val, Self::TABLE_NAME, "_FuncName")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_guild_func.out`.
pub type ConfigGuildFuncTable = DataTable<u8, ConfigGuildFuncRow>;

/// Parses `config_guild_func.out` into a [`ConfigGuildFuncTable`].
pub fn parse_config_guild_func(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGuildFuncTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_guild_icon.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuildIconRow {
    pub icon_id: u8,
    pub id: i8,
    pub image: Box<str>,
}

impl DataCfgRow for ConfigGuildIconRow {
    const TABLE_NAME: &'static str = "config_guild_icon";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"IconId" => self.icon_id = ctx.decode_field(val, Self::TABLE_NAME, "IconId")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Image" => self.image = ctx.decode_field(val, Self::TABLE_NAME, "Image")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_guild_icon.out`.
pub type ConfigGuildIconTable = DataTable<u8, ConfigGuildIconRow>;

/// Parses `config_guild_icon.out` into a [`ConfigGuildIconTable`].
pub fn parse_config_guild_icon(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGuildIconTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_guild_icon_bg.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuildIconBgRow {
    pub bg_id: u8,
    pub c3_b: u8,
    pub c3_g: u8,
    pub c3_r: u8,
    pub id: i8,
}

impl DataCfgRow for ConfigGuildIconBgRow {
    const TABLE_NAME: &'static str = "config_guild_icon_bg";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BgId" => self.bg_id = ctx.decode_field(val, Self::TABLE_NAME, "BgId")?,
            b"C3B" => self.c3_b = ctx.decode_field(val, Self::TABLE_NAME, "C3B")?,
            b"C3G" => self.c3_g = ctx.decode_field(val, Self::TABLE_NAME, "C3G")?,
            b"C3R" => self.c3_r = ctx.decode_field(val, Self::TABLE_NAME, "C3R")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_guild_icon_bg.out`.
pub type ConfigGuildIconBgTable = DataTable<u8, ConfigGuildIconBgRow>;

/// Parses `config_guild_icon_bg.out` into a [`ConfigGuildIconBgTable`].
pub fn parse_config_guild_icon_bg(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGuildIconBgTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_guild_level.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuildLevelRow {
    pub exp: u32,
    pub level: i8,
    pub maxmember: u8,
}

impl DataCfgRow for ConfigGuildLevelRow {
    const TABLE_NAME: &'static str = "config_guild_level";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Exp" => self.exp = ctx.decode_field(val, Self::TABLE_NAME, "Exp")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Maxmember" => {
                self.maxmember = ctx.decode_field(val, Self::TABLE_NAME, "Maxmember")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_guild_level.out`.
pub type ConfigGuildLevelTable = DataTable<u8, ConfigGuildLevelRow>;

/// Parses `config_guild_level.out` into a [`ConfigGuildLevelTable`].
pub fn parse_config_guild_level(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGuildLevelTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_guild_moment.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuildMomentRow {
    pub data: Box<[u8]>,
    pub event: i8,
    pub interaction_type: u8,
    pub moment_text: Box<str>,
    pub moment_type: u8,
    pub _moment_text: Box<str>,
}

impl DataCfgRow for ConfigGuildMomentRow {
    const TABLE_NAME: &'static str = "config_guild_moment";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Data" => self.data = ctx.decode_field(val, Self::TABLE_NAME, "Data")?,
            b"Event" => self.event = ctx.decode_field(val, Self::TABLE_NAME, "Event")?,
            b"InteractionType" => {
                self.interaction_type =
                    ctx.decode_field(val, Self::TABLE_NAME, "InteractionType")?
            }
            b"MomentText" => {
                self.moment_text = ctx.decode_field(val, Self::TABLE_NAME, "MomentText")?
            }
            b"MomentType" => {
                self.moment_type = ctx.decode_field(val, Self::TABLE_NAME, "MomentType")?
            }
            b"_MomentText" => {
                self._moment_text = ctx.decode_field(val, Self::TABLE_NAME, "_MomentText")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_guild_moment.out`.
pub type ConfigGuildMomentTable = DataTable<u8, ConfigGuildMomentRow>;

/// Parses `config_guild_moment.out` into a [`ConfigGuildMomentTable`].
pub fn parse_config_guild_moment(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGuildMomentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_guild_operator_member.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuildOperatorMemberRow {
    pub authority: Box<[u8]>,
    pub id: i8,
    pub oper_from: u8,
    pub oper_to: u8,
}

impl DataCfgRow for ConfigGuildOperatorMemberRow {
    const TABLE_NAME: &'static str = "config_guild_operator_member";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Authority" => {
                self.authority = ctx.decode_field(val, Self::TABLE_NAME, "Authority")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"OperFrom" => self.oper_from = ctx.decode_field(val, Self::TABLE_NAME, "OperFrom")?,
            b"OperTo" => self.oper_to = ctx.decode_field(val, Self::TABLE_NAME, "OperTo")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_guild_operator_member.out`.
pub type ConfigGuildOperatorMemberTable = DataTable<u8, ConfigGuildOperatorMemberRow>;

/// Parses `config_guild_operator_member.out` into a [`ConfigGuildOperatorMemberTable`].
pub fn parse_config_guild_operator_member(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGuildOperatorMemberTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_guild_position.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGuildPositionRow {
    pub authority: Box<[u8]>,
    pub iconpath: Box<str>,
    pub id: i8,
    pub position_name: Box<str>,
}

impl DataCfgRow for ConfigGuildPositionRow {
    const TABLE_NAME: &'static str = "config_guild_position";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Authority" => {
                self.authority = ctx.decode_field(val, Self::TABLE_NAME, "Authority")?
            }
            b"Iconpath" => self.iconpath = ctx.decode_field(val, Self::TABLE_NAME, "Iconpath")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"PositionName" => {
                self.position_name = ctx.decode_field(val, Self::TABLE_NAME, "PositionName")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_guild_position.out`.
pub type ConfigGuildPositionTable = DataTable<u8, ConfigGuildPositionRow>;

/// Parses `config_guild_position.out` into a [`ConfigGuildPositionTable`].
pub fn parse_config_guild_position(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigGuildPositionTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_mail_content.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMailContentRow {
    pub child_type: u32,
    pub client_type: u8,
    pub content: Box<str>,
    pub id: i32,
    pub main_type: u8,
    pub title: Box<str>,
    pub _content: Box<str>,
    pub _title: Box<str>,
}

impl DataCfgRow for ConfigMailContentRow {
    const TABLE_NAME: &'static str = "config_mail_content";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ChildType" => {
                self.child_type = ctx.decode_field(val, Self::TABLE_NAME, "ChildType")?
            }
            b"ClientType" => {
                self.client_type = ctx.decode_field(val, Self::TABLE_NAME, "ClientType")?
            }
            b"Content" => self.content = ctx.decode_field(val, Self::TABLE_NAME, "Content")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MainType" => self.main_type = ctx.decode_field(val, Self::TABLE_NAME, "MainType")?,
            b"Title" => self.title = ctx.decode_field(val, Self::TABLE_NAME, "Title")?,
            b"_Content" => self._content = ctx.decode_field(val, Self::TABLE_NAME, "_Content")?,
            b"_Title" => self._title = ctx.decode_field(val, Self::TABLE_NAME, "_Title")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_mail_content.out`.
pub type ConfigMailContentTable = DataTable<u32, ConfigMailContentRow>;

/// Parses `config_mail_content.out` into a [`ConfigMailContentTable`].
pub fn parse_config_mail_content(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMailContentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_p2p_buttonlist.Priority`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct P2pButtonlistPriorityEntry {
    pub button_id: u8,
    pub priority: u8,
}

impl FromEvalVal for P2pButtonlistPriorityEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (button_id, priority) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            button_id,
            priority,
        })
    }
}

/// Row record for `config_p2p_buttonlist.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigP2pButtonlistRow {
    pub des: Box<str>,
    pub r#enum: Box<str>,
    pub id: i8,
    pub priority: Box<[P2pButtonlistPriorityEntry]>,
}

impl DataCfgRow for ConfigP2pButtonlistRow {
    const TABLE_NAME: &'static str = "config_p2p_buttonlist";

    #[inline(never)]
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
            b"Priority" => self.priority = ctx.decode_field(val, Self::TABLE_NAME, "Priority")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_p2p_buttonlist.out`.
pub type ConfigP2pButtonlistTable = DataTable<u8, ConfigP2pButtonlistRow>;

/// Parses `config_p2p_buttonlist.out` into a [`ConfigP2pButtonlistTable`].
pub fn parse_config_p2p_buttonlist(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigP2pButtonlistTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_system_chat.DefaultLanguage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SystemChatDefaultLanguage {
    pub min: u16,
    pub max: u16,
}

impl FromEvalVal for SystemChatDefaultLanguage {
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

/// Row record for `config_system_chat.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSystemChatRow {
    pub channel_name: Box<str>,
    pub character_limit: u8,
    pub default_language: SystemChatDefaultLanguage,
    pub id: i8,
    pub line: u16,
    pub max_message_numb: u8,
    pub member_limit: u16,
    pub require_level: u8,
    pub speech_interval: u32,
    pub speech_lag: u8,
    pub temp: Box<str>,
}

impl DataCfgRow for ConfigSystemChatRow {
    const TABLE_NAME: &'static str = "config_system_chat";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ChannelName" => {
                self.channel_name = ctx.decode_field(val, Self::TABLE_NAME, "ChannelName")?
            }
            b"CharacterLimit" => {
                self.character_limit = ctx.decode_field(val, Self::TABLE_NAME, "CharacterLimit")?
            }
            b"DefaultLanguage" => {
                self.default_language =
                    ctx.decode_field(val, Self::TABLE_NAME, "DefaultLanguage")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Line" => self.line = ctx.decode_field(val, Self::TABLE_NAME, "Line")?,
            b"MaxMessageNumb" => {
                self.max_message_numb = ctx.decode_field(val, Self::TABLE_NAME, "MaxMessageNumb")?
            }
            b"MemberLimit" => {
                self.member_limit = ctx.decode_field(val, Self::TABLE_NAME, "MemberLimit")?
            }
            b"RequireLevel" => {
                self.require_level = ctx.decode_field(val, Self::TABLE_NAME, "RequireLevel")?
            }
            b"SpeechInterval" => {
                self.speech_interval = ctx.decode_field(val, Self::TABLE_NAME, "SpeechInterval")?
            }
            b"SpeechLag" => {
                self.speech_lag = ctx.decode_field(val, Self::TABLE_NAME, "SpeechLag")?
            }
            b"Temp" => self.temp = ctx.decode_field(val, Self::TABLE_NAME, "Temp")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_system_chat.out`.
pub type ConfigSystemChatTable = DataTable<u8, ConfigSystemChatRow>;

/// Parses `config_system_chat.out` into a [`ConfigSystemChatTable`].
pub fn parse_config_system_chat(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSystemChatTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_system_chat_language.LineNum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SystemChatLanguageLineNum {
    pub min: u16,
    pub max: u16,
}

impl FromEvalVal for SystemChatLanguageLineNum {
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

/// Row record for `config_system_chat_language.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigSystemChatLanguageRow {
    pub id: i8,
    pub language: Box<str>,
    pub language_name: Box<str>,
    pub line_num: SystemChatLanguageLineNum,
}

impl DataCfgRow for ConfigSystemChatLanguageRow {
    const TABLE_NAME: &'static str = "config_system_chat_language";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Language" => self.language = ctx.decode_field(val, Self::TABLE_NAME, "Language")?,
            b"LanguageName" => {
                self.language_name = ctx.decode_field(val, Self::TABLE_NAME, "LanguageName")?
            }
            b"LineNum" => self.line_num = ctx.decode_field(val, Self::TABLE_NAME, "LineNum")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_system_chat_language.out`.
pub type ConfigSystemChatLanguageTable = DataTable<u8, ConfigSystemChatLanguageRow>;

/// Parses `config_system_chat_language.out` into a [`ConfigSystemChatLanguageTable`].
pub fn parse_config_system_chat_language(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigSystemChatLanguageTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_team_bonus_show.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTeamBonusShowRow {
    pub icon: Box<str>,
    pub id: i8,
    pub incress: Box<[Box<str>]>,
}

impl DataCfgRow for ConfigTeamBonusShowRow {
    const TABLE_NAME: &'static str = "config_team_bonus_show";

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
            b"Incress" => self.incress = ctx.decode_field(val, Self::TABLE_NAME, "Incress")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_team_bonus_show.out`.
pub type ConfigTeamBonusShowTable = DataTable<u8, ConfigTeamBonusShowRow>;

/// Parses `config_team_bonus_show.out` into a [`ConfigTeamBonusShowTable`].
pub fn parse_config_team_bonus_show(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTeamBonusShowTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_team_moment.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTeamMomentRow {
    pub data: Box<[u8]>,
    pub event: i8,
    pub moment_text: Box<str>,
    pub _moment_text: Box<str>,
}

impl DataCfgRow for ConfigTeamMomentRow {
    const TABLE_NAME: &'static str = "config_team_moment";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Data" => self.data = ctx.decode_field(val, Self::TABLE_NAME, "Data")?,
            b"Event" => self.event = ctx.decode_field(val, Self::TABLE_NAME, "Event")?,
            b"MomentText" => {
                self.moment_text = ctx.decode_field(val, Self::TABLE_NAME, "MomentText")?
            }
            b"_MomentText" => {
                self._moment_text = ctx.decode_field(val, Self::TABLE_NAME, "_MomentText")?
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_team_moment.out`.
pub type ConfigTeamMomentTable = DataTable<u8, ConfigTeamMomentRow>;

/// Parses `config_team_moment.out` into a [`ConfigTeamMomentTable`].
pub fn parse_config_team_moment(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTeamMomentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Row record for `config_team_speedup_content.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTeamSpeedupContentRow {
    pub content: Box<str>,
    pub id: i8,
    pub speedup: Box<str>,
    pub _content: Box<str>,
}

impl DataCfgRow for ConfigTeamSpeedupContentRow {
    const TABLE_NAME: &'static str = "config_team_speedup_content";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Content" => self.content = ctx.decode_field(val, Self::TABLE_NAME, "Content")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Speedup" => self.speedup = ctx.decode_field(val, Self::TABLE_NAME, "Speedup")?,
            b"_Content" => self._content = ctx.decode_field(val, Self::TABLE_NAME, "_Content")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_team_speedup_content.out`.
pub type ConfigTeamSpeedupContentTable = DataTable<u8, ConfigTeamSpeedupContentRow>;

/// Parses `config_team_speedup_content.out` into a [`ConfigTeamSpeedupContentTable`].
pub fn parse_config_team_speedup_content(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTeamSpeedupContentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

/// Entry for `config_team_td_challenge.FirstAward`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TeamTdChallengeFirstAwardEntry {
    pub round_id: u8,
    pub reward_group_id: u16,
}

impl FromEvalVal for TeamTdChallengeFirstAwardEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (round_id, reward_group_id) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            round_id,
            reward_group_id,
        })
    }
}

/// Row record for `config_team_td_challenge.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigTeamTdChallengeRow {
    pub before_id: u8,
    pub desc: Box<str>,
    pub first_award: Box<[TeamTdChallengeFirstAwardEntry]>,
    pub first_award_role: [u8; 0],
    pub game_play_id: u16,
    pub id: i8,
    pub level_id: u32,
    pub name: Box<str>,
    pub unlock_req: u8,
    pub _desc: Box<str>,
    pub _name: Box<str>,
}

impl DataCfgRow for ConfigTeamTdChallengeRow {
    const TABLE_NAME: &'static str = "config_team_td_challenge";

    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BeforeId" => self.before_id = ctx.decode_field(val, Self::TABLE_NAME, "BeforeId")?,
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"FirstAward" => {
                self.first_award = ctx.decode_field(val, Self::TABLE_NAME, "FirstAward")?
            }
            b"FirstAwardRole" => {
                self.first_award_role = ctx.decode_field(val, Self::TABLE_NAME, "FirstAwardRole")?
            }
            b"GamePlayId" => {
                self.game_play_id = ctx.decode_field(val, Self::TABLE_NAME, "GamePlayId")?
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LevelId" => self.level_id = ctx.decode_field(val, Self::TABLE_NAME, "LevelId")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"UnlockReq" => {
                self.unlock_req = ctx.decode_field(val, Self::TABLE_NAME, "UnlockReq")?
            }
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}

/// Typed `DataTable` for `config_team_td_challenge.out`.
pub type ConfigTeamTdChallengeTable = DataTable<u8, ConfigTeamTdChallengeRow>;

/// Parses `config_team_td_challenge.out` into a [`ConfigTeamTdChallengeTable`].
pub fn parse_config_team_td_challenge(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigTeamTdChallengeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}

fn verify_chat_chunk_0(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_chat_moment" => Some(parse_config_chat_moment(buf, scratch).map(|t| t.len())),
        "config_friend_gift_ceiling" => {
            Some(parse_config_friend_gift_ceiling(buf, scratch).map(|t| t.len()))
        }
        "config_friend_gift_item" => {
            Some(parse_config_friend_gift_item(buf, scratch).map(|t| t.len()))
        }
        "config_guild_func" => Some(parse_config_guild_func(buf, scratch).map(|t| t.len())),
        "config_guild_icon" => Some(parse_config_guild_icon(buf, scratch).map(|t| t.len())),
        "config_guild_icon_bg" => Some(parse_config_guild_icon_bg(buf, scratch).map(|t| t.len())),
        "config_guild_level" => Some(parse_config_guild_level(buf, scratch).map(|t| t.len())),
        "config_guild_moment" => Some(parse_config_guild_moment(buf, scratch).map(|t| t.len())),
        "config_guild_operator_member" => {
            Some(parse_config_guild_operator_member(buf, scratch).map(|t| t.len()))
        }
        "config_guild_position" => Some(parse_config_guild_position(buf, scratch).map(|t| t.len())),
        "config_mail_content" => Some(parse_config_mail_content(buf, scratch).map(|t| t.len())),
        "config_p2p_buttonlist" => Some(parse_config_p2p_buttonlist(buf, scratch).map(|t| t.len())),
        "config_system_chat" => Some(parse_config_system_chat(buf, scratch).map(|t| t.len())),
        "config_system_chat_language" => {
            Some(parse_config_system_chat_language(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

fn verify_chat_chunk_1(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_team_bonus_show" => {
            Some(parse_config_team_bonus_show(buf, scratch).map(|t| t.len()))
        }
        "config_team_moment" => Some(parse_config_team_moment(buf, scratch).map(|t| t.len())),
        "config_team_speedup_content" => {
            Some(parse_config_team_speedup_content(buf, scratch).map(|t| t.len()))
        }
        "config_team_td_challenge" => {
            Some(parse_config_team_td_challenge(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

pub(crate) fn verify_chat_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    if let Some(res) = verify_chat_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_chat_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    None
}

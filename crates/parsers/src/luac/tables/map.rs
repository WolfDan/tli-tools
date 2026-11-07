//! Auto-generated `LuaTable/DataCfg` types and parsers for the `map` domain.
use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg, parse_luatable_datacfg_3d,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;
/// Entry for `config_map_default_mod_random.ModNum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MapDefaultModRandomModNumEntry {
    pub weight: u16,
    pub count: u8,
}
impl FromEvalVal for MapDefaultModRandomModNumEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (count, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { weight, count })
    }
}
/// Row record for `config_map_default_mod_random.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapDefaultModRandomRow {
    pub map_modifier_group: Box<[u8]>,
    pub mod_num: Box<[MapDefaultModRandomModNumEntry]>,
    pub id: i8,
}
impl DataCfgRow for ConfigMapDefaultModRandomRow {
    const TABLE_NAME: &'static str = "config_map_default_mod_random";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MapModifierGroup" => {
                self.map_modifier_group = ctx
                    .decode_field(val, Self::TABLE_NAME, "MapModifierGroup")?;
            }
            b"ModNum" => {
                self.mod_num = ctx.decode_field(val, Self::TABLE_NAME, "ModNum")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_default_mod_random.out`.
pub type ConfigMapDefaultModRandomTable = DataTable<u8, ConfigMapDefaultModRandomRow>;
/// Parses `config_map_default_mod_random.out` into a [`ConfigMapDefaultModRandomTable`].
pub fn parse_config_map_default_mod_random(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapDefaultModRandomTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_environment.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapEnvironmentRow {
    pub environment_levels: Box<[Box<str>]>,
    pub environment_type: u8,
    pub id: i8,
    pub priority: u8,
    pub replaced_level: u8,
}
impl DataCfgRow for ConfigMapEnvironmentRow {
    const TABLE_NAME: &'static str = "config_map_environment";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"EnvironmentLevels" => {
                self.environment_levels = ctx
                    .decode_field(val, Self::TABLE_NAME, "EnvironmentLevels")?;
            }
            b"EnvironmentType" => {
                self.environment_type = ctx
                    .decode_field(val, Self::TABLE_NAME, "EnvironmentType")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Priority" => {
                self.priority = ctx.decode_field(val, Self::TABLE_NAME, "Priority")?;
            }
            b"ReplacedLevel" => {
                self.replaced_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "ReplacedLevel")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_environment.out`.
pub type ConfigMapEnvironmentTable = DataTable<u8, ConfigMapEnvironmentRow>;
/// Parses `config_map_environment.out` into a [`ConfigMapEnvironmentTable`].
pub fn parse_config_map_environment(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapEnvironmentTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_climbtower.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplayClimbtowerRow {
    pub dps_adv: u64,
    pub level_id: Box<[u32]>,
    pub drop_virtual_entity_id: u16,
    pub level: u8,
    pub tier: i8,
}
impl DataCfgRow for ConfigMapGameplayClimbtowerRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_climbtower";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DpsAdv" => {
                self.dps_adv = ctx.decode_field(val, Self::TABLE_NAME, "DpsAdv")?;
            }
            b"DropVirtualEntityId" => {
                self.drop_virtual_entity_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropVirtualEntityId")?;
            }
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"LevelId" => {
                self.level_id = ctx.decode_field(val, Self::TABLE_NAME, "LevelId")?;
            }
            b"Tier" => self.tier = ctx.decode_field(val, Self::TABLE_NAME, "Tier")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_climbtower.out`.
pub type ConfigMapGameplayClimbtowerTable = DataTable<
    u8,
    ConfigMapGameplayClimbtowerRow,
>;
/// Parses `config_map_gameplay_climbtower.out` into a [`ConfigMapGameplayClimbtowerTable`].
pub fn parse_config_map_gameplay_climbtower(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplayClimbtowerTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_climbtower_ticket.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplayClimbtowerTicketRow {
    pub icon: Box<str>,
    pub ticket_id: i32,
    pub end: u8,
    pub item_base: u8,
    pub life: u8,
    pub start: u8,
}
impl DataCfgRow for ConfigMapGameplayClimbtowerTicketRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_climbtower_ticket";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"End" => self.end = ctx.decode_field(val, Self::TABLE_NAME, "End")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"ItemBase" => {
                self.item_base = ctx.decode_field(val, Self::TABLE_NAME, "ItemBase")?;
            }
            b"Life" => self.life = ctx.decode_field(val, Self::TABLE_NAME, "Life")?,
            b"Start" => self.start = ctx.decode_field(val, Self::TABLE_NAME, "Start")?,
            b"TicketId" => {
                self.ticket_id = ctx.decode_field(val, Self::TABLE_NAME, "TicketId")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_climbtower_ticket.out`.
pub type ConfigMapGameplayClimbtowerTicketTable = DataTable<
    u32,
    ConfigMapGameplayClimbtowerTicketRow,
>;
/// Parses `config_map_gameplay_climbtower_ticket.out` into a [`ConfigMapGameplayClimbtowerTicketTable`].
pub fn parse_config_map_gameplay_climbtower_ticket(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplayClimbtowerTicketTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_hero_trial.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplayHeroTrialRow {
    pub des: Box<str>,
    pub icon_pic: Box<str>,
    pub name: Box<str>,
    pub reward_group: Box<[u32]>,
    pub title_pic: Box<str>,
    pub _des: Box<str>,
    pub _name: Box<str>,
    pub level_id: u32,
    pub smoke_color: u16,
    pub hint_level: u8,
    pub id: i8,
    pub require_level: u8,
    pub unlock: u8,
}
impl ConfigMapGameplayHeroTrialRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"HintLevel" => {
                self.hint_level = ctx.decode_field(val, Self::TABLE_NAME, "HintLevel")?;
            }
            b"IconPic" => {
                self.icon_pic = ctx.decode_field(val, Self::TABLE_NAME, "IconPic")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LevelId" => {
                self.level_id = ctx.decode_field(val, Self::TABLE_NAME, "LevelId")?;
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"RequireLevel" => {
                self.require_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "RequireLevel")?;
            }
            b"RewardGroup" => {
                self.reward_group = ctx
                    .decode_field(val, Self::TABLE_NAME, "RewardGroup")?;
            }
            b"SmokeColor" => {
                self.smoke_color = ctx.decode_field(val, Self::TABLE_NAME, "SmokeColor")?;
            }
            b"TitlePic" => {
                self.title_pic = ctx.decode_field(val, Self::TABLE_NAME, "TitlePic")?;
            }
            b"Unlock" => self.unlock = ctx.decode_field(val, Self::TABLE_NAME, "Unlock")?,
            b"_Des" => self._des = ctx.decode_field(val, Self::TABLE_NAME, "_Des")?,
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
impl DataCfgRow for ConfigMapGameplayHeroTrialRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_hero_trial";
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
/// Typed `DataTable` for `config_map_gameplay_hero_trial.out`.
pub type ConfigMapGameplayHeroTrialTable = DataTable<u8, ConfigMapGameplayHeroTrialRow>;
/// Parses `config_map_gameplay_hero_trial.out` into a [`ConfigMapGameplayHeroTrialTable`].
pub fn parse_config_map_gameplay_hero_trial(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplayHeroTrialTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_doll_text_display.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonDollTextDisplayRow {
    pub content: Box<str>,
    pub npc_face: Box<str>,
    pub _content: Box<str>,
    pub npc_tale_id: u32,
    pub modifier_id: u16,
    pub rare_num: u8,
    pub rare_type: i8,
}
impl DataCfgRow for ConfigMapGameplaySeasonDollTextDisplayRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_doll_text_display";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Content" => {
                self.content = ctx.decode_field(val, Self::TABLE_NAME, "Content")?;
            }
            b"ModifierId" => {
                self.modifier_id = ctx.decode_field(val, Self::TABLE_NAME, "ModifierId")?;
            }
            b"NpcFace" => {
                self.npc_face = ctx.decode_field(val, Self::TABLE_NAME, "NpcFace")?;
            }
            b"NpcTaleId" => {
                self.npc_tale_id = ctx.decode_field(val, Self::TABLE_NAME, "NpcTaleId")?;
            }
            b"RareNum" => {
                self.rare_num = ctx.decode_field(val, Self::TABLE_NAME, "RareNum")?;
            }
            b"RareType" => {
                self.rare_type = ctx.decode_field(val, Self::TABLE_NAME, "RareType")?;
            }
            b"_Content" => {
                self._content = ctx.decode_field(val, Self::TABLE_NAME, "_Content")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_doll_text_display.out`.
pub type ConfigMapGameplaySeasonDollTextDisplayTable = DataTable<
    (u8, u8, u16),
    ConfigMapGameplaySeasonDollTextDisplayRow,
>;
/// Parses `config_map_gameplay_season_doll_text_display.out` into a [`ConfigMapGameplaySeasonDollTextDisplayTable`].
pub fn parse_config_map_gameplay_season_doll_text_display(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonDollTextDisplayTable, LuacError> {
    parse_luatable_datacfg_3d(buf, scratch)
}
/// Row record for `config_map_gameplay_season_dream_talent_costlv.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonDreamTalentCostlvRow {
    pub dream_layer_req: u8,
    pub id: i8,
    pub point: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonDreamTalentCostlvRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_dream_talent_costlv";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DreamLayerReq" => {
                self.dream_layer_req = ctx
                    .decode_field(val, Self::TABLE_NAME, "DreamLayerReq")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Point" => self.point = ctx.decode_field(val, Self::TABLE_NAME, "Point")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_dream_talent_costlv.out`.
pub type ConfigMapGameplaySeasonDreamTalentCostlvTable = DataTable<
    u8,
    ConfigMapGameplaySeasonDreamTalentCostlvRow,
>;
/// Parses `config_map_gameplay_season_dream_talent_costlv.out` into a [`ConfigMapGameplaySeasonDreamTalentCostlvTable`].
pub fn parse_config_map_gameplay_season_dream_talent_costlv(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonDreamTalentCostlvTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Entry for `config_map_gameplay_season_dream_talent_tree.MapModifier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MapGameplaySeasonDreamTalentTreeMapModifierEntry {
    pub modifier_id: u32,
    pub value: u8,
}
impl FromEvalVal for MapGameplaySeasonDreamTalentTreeMapModifierEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (modifier_id, value) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { modifier_id, value })
    }
}
/// Row record for `config_map_gameplay_season_dream_talent_tree.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonDreamTalentTreeRow {
    pub desc: Box<str>,
    pub function_modifier: Box<[u32]>,
    pub icon: Box<str>,
    pub icon_grey: Box<str>,
    pub map_modifier: Box<[MapGameplaySeasonDreamTalentTreeMapModifierEntry]>,
    pub name: Box<str>,
    pub pre_req: Box<[u16]>,
    pub _desc: Box<str>,
    pub _name: Box<str>,
    pub id: i16,
    pub bg_type: u8,
    pub cost: u8,
    pub group: u8,
    pub inside_req: u8,
    pub lv: u8,
    pub offset_id: u8,
    pub outside_req: u8,
    pub r#type: u8,
}
impl ConfigMapGameplaySeasonDreamTalentTreeRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"BgType" => {
                self.bg_type = ctx.decode_field(val, Self::TABLE_NAME, "BgType")?;
            }
            b"Cost" => self.cost = ctx.decode_field(val, Self::TABLE_NAME, "Cost")?,
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"FunctionModifier" => {
                self.function_modifier = ctx
                    .decode_field(val, Self::TABLE_NAME, "FunctionModifier")?;
            }
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"IconGrey" => {
                self.icon_grey = ctx.decode_field(val, Self::TABLE_NAME, "IconGrey")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"InsideReq" => {
                self.inside_req = ctx.decode_field(val, Self::TABLE_NAME, "InsideReq")?;
            }
            b"Lv" => self.lv = ctx.decode_field(val, Self::TABLE_NAME, "Lv")?,
            b"MapModifier" => {
                self.map_modifier = ctx
                    .decode_field(val, Self::TABLE_NAME, "MapModifier")?;
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
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
            b"OffsetId" => {
                self.offset_id = ctx.decode_field(val, Self::TABLE_NAME, "OffsetId")?;
            }
            b"OutsideReq" => {
                self.outside_req = ctx.decode_field(val, Self::TABLE_NAME, "OutsideReq")?;
            }
            b"PreReq" => {
                self.pre_req = ctx.decode_field(val, Self::TABLE_NAME, "PreReq")?;
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigMapGameplaySeasonDreamTalentTreeRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_dream_talent_tree";
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
/// Typed `DataTable` for `config_map_gameplay_season_dream_talent_tree.out`.
pub type ConfigMapGameplaySeasonDreamTalentTreeTable = DataTable<
    u16,
    ConfigMapGameplaySeasonDreamTalentTreeRow,
>;
/// Parses `config_map_gameplay_season_dream_talent_tree.out` into a [`ConfigMapGameplaySeasonDreamTalentTreeTable`].
pub fn parse_config_map_gameplay_season_dream_talent_tree(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonDreamTalentTreeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_lost_bigmap_difficulty.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonLostBigmapDifficultyRow {
    pub map_modifier: Box<[u32]>,
    pub difficulty: i16,
}
impl DataCfgRow for ConfigMapGameplaySeasonLostBigmapDifficultyRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_lost_bigmap_difficulty";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Difficulty" => {
                self.difficulty = ctx.decode_field(val, Self::TABLE_NAME, "Difficulty")?;
            }
            b"MapModifier" => {
                self.map_modifier = ctx
                    .decode_field(val, Self::TABLE_NAME, "MapModifier")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_lost_bigmap_difficulty.out`.
pub type ConfigMapGameplaySeasonLostBigmapDifficultyTable = DataTable<
    u16,
    ConfigMapGameplaySeasonLostBigmapDifficultyRow,
>;
/// Parses `config_map_gameplay_season_lost_bigmap_difficulty.out` into a [`ConfigMapGameplaySeasonLostBigmapDifficultyTable`].
pub fn parse_config_map_gameplay_season_lost_bigmap_difficulty(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonLostBigmapDifficultyTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_lost_reward_boss.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplaySeasonLostRewardBossRow {
    pub bonus_desc: Box<str>,
    pub drop_quantity: f64,
    pub id: i32,
    pub difficulty: u8,
    pub energy: u8,
    pub rarity: u8,
    pub reward_type: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonLostRewardBossRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_lost_reward_boss";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BonusDesc" => {
                self.bonus_desc = ctx.decode_field(val, Self::TABLE_NAME, "BonusDesc")?;
            }
            b"Difficulty" => {
                self.difficulty = ctx.decode_field(val, Self::TABLE_NAME, "Difficulty")?;
            }
            b"DropQuantity" => {
                self.drop_quantity = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropQuantity")?;
            }
            b"Energy" => self.energy = ctx.decode_field(val, Self::TABLE_NAME, "Energy")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"RewardType" => {
                self.reward_type = ctx.decode_field(val, Self::TABLE_NAME, "RewardType")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_lost_reward_boss.out`.
pub type ConfigMapGameplaySeasonLostRewardBossTable = DataTable<
    u32,
    ConfigMapGameplaySeasonLostRewardBossRow,
>;
/// Parses `config_map_gameplay_season_lost_reward_boss.out` into a [`ConfigMapGameplaySeasonLostRewardBossTable`].
pub fn parse_config_map_gameplay_season_lost_reward_boss(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonLostRewardBossTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_lost_reward_drop.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonLostRewardDropRow {
    pub desc: Box<str>,
    pub icon: Box<str>,
    pub name: Box<str>,
    pub _desc: Box<str>,
    pub _name: Box<str>,
    pub drop_id: i16,
    pub num: u8,
    pub rarity: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonLostRewardDropRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_lost_reward_drop";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DropId" => {
                self.drop_id = ctx.decode_field(val, Self::TABLE_NAME, "DropId")?;
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Num" => self.num = ctx.decode_field(val, Self::TABLE_NAME, "Num")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_lost_reward_drop.out`.
pub type ConfigMapGameplaySeasonLostRewardDropTable = DataTable<
    u16,
    ConfigMapGameplaySeasonLostRewardDropRow,
>;
/// Parses `config_map_gameplay_season_lost_reward_drop.out` into a [`ConfigMapGameplaySeasonLostRewardDropTable`].
pub fn parse_config_map_gameplay_season_lost_reward_drop(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonLostRewardDropTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_lost_reward_type.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonLostRewardTypeRow {
    pub icon: Box<str>,
    pub name: Box<str>,
    pub _name: Box<str>,
    pub drop_energy: u8,
    pub drop_energy2: u8,
    pub id: i8,
}
impl DataCfgRow for ConfigMapGameplaySeasonLostRewardTypeRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_lost_reward_type";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DropEnergy" => {
                self.drop_energy = ctx.decode_field(val, Self::TABLE_NAME, "DropEnergy")?;
            }
            b"DropEnergy2" => {
                self.drop_energy2 = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropEnergy2")?;
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_lost_reward_type.out`.
pub type ConfigMapGameplaySeasonLostRewardTypeTable = DataTable<
    u8,
    ConfigMapGameplaySeasonLostRewardTypeRow,
>;
/// Parses `config_map_gameplay_season_lost_reward_type.out` into a [`ConfigMapGameplaySeasonLostRewardTypeTable`].
pub fn parse_config_map_gameplay_season_lost_reward_type(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonLostRewardTypeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Entry for `config_map_gameplay_season_lost_talent_tree.Position`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MapGameplaySeasonLostTalentTreePosition {
    pub x: u16,
    pub y: u16,
}
impl FromEvalVal for MapGameplaySeasonLostTalentTreePosition {
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
/// Entry for `config_map_gameplay_season_lost_talent_tree.PreReq`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MapGameplaySeasonLostTalentTreePreReqEntry {
    pub prev_node_id: u8,
    pub prev_node_level: u8,
}
impl FromEvalVal for MapGameplaySeasonLostTalentTreePreReqEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (prev_node_id, prev_node_level) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            prev_node_id,
            prev_node_level,
        })
    }
}
/// Row record for `config_map_gameplay_season_lost_talent_tree.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonLostTalentTreeRow {
    pub icon: Box<str>,
    pub link_node: Box<[u8]>,
    pub name: Box<str>,
    pub note: Box<str>,
    pub pre_req: Box<[MapGameplaySeasonLostTalentTreePreReqEntry]>,
    pub pre_req_info: Box<str>,
    pub tale: Box<str>,
    pub _name: Box<str>,
    pub _note: Box<str>,
    pub _pre_req_info: Box<str>,
    pub _tale: Box<str>,
    pub position: MapGameplaySeasonLostTalentTreePosition,
    pub id: i8,
    pub lv_max: u8,
    pub rarity: u8,
    pub show_info: bool,
    pub show_init: bool,
    pub r#type: u8,
}
impl ConfigMapGameplaySeasonLostTalentTreeRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LinkNode" => {
                self.link_node = ctx.decode_field(val, Self::TABLE_NAME, "LinkNode")?;
            }
            b"LvMax" => self.lv_max = ctx.decode_field(val, Self::TABLE_NAME, "LvMax")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Note" => self.note = ctx.decode_field(val, Self::TABLE_NAME, "Note")?,
            b"Position" => {
                self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?;
            }
            b"PreReq" => {
                self.pre_req = ctx.decode_field(val, Self::TABLE_NAME, "PreReq")?;
            }
            b"PreReqInfo" => {
                self.pre_req_info = ctx
                    .decode_field(val, Self::TABLE_NAME, "PreReqInfo")?;
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"ShowInfo" => {
                self.show_info = ctx.decode_field(val, Self::TABLE_NAME, "ShowInfo")?;
            }
            b"ShowInit" => {
                self.show_init = ctx.decode_field(val, Self::TABLE_NAME, "ShowInit")?;
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
            b"Tale" => self.tale = ctx.decode_field(val, Self::TABLE_NAME, "Tale")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            b"_Note" => self._note = ctx.decode_field(val, Self::TABLE_NAME, "_Note")?,
            b"_PreReqInfo" => {
                self._pre_req_info = ctx
                    .decode_field(val, Self::TABLE_NAME, "_PreReqInfo")?;
            }
            b"_Tale" => self._tale = ctx.decode_field(val, Self::TABLE_NAME, "_Tale")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigMapGameplaySeasonLostTalentTreeRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_lost_talent_tree";
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
/// Typed `DataTable` for `config_map_gameplay_season_lost_talent_tree.out`.
pub type ConfigMapGameplaySeasonLostTalentTreeTable = DataTable<
    u8,
    ConfigMapGameplaySeasonLostTalentTreeRow,
>;
/// Parses `config_map_gameplay_season_lost_talent_tree.out` into a [`ConfigMapGameplaySeasonLostTalentTreeTable`].
pub fn parse_config_map_gameplay_season_lost_talent_tree(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonLostTalentTreeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Entry for `config_map_gameplay_season_lost_talent_tree_level.MapModifier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MapGameplaySeasonLostTalentTreeLevelMapModifierEntry {
    pub modifier_id: u32,
    pub value: u8,
}
impl FromEvalVal for MapGameplaySeasonLostTalentTreeLevelMapModifierEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (modifier_id, value) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { modifier_id, value })
    }
}
/// Row record for `config_map_gameplay_season_lost_talent_tree_level.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonLostTalentTreeLevelRow {
    pub function_modifier: Box<[u32]>,
    pub map_modifier: Box<[MapGameplaySeasonLostTalentTreeLevelMapModifierEntry]>,
    pub id: i32,
    pub cost_item: u16,
    pub cost_num: u8,
    pub lv: u8,
    pub node_id: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonLostTalentTreeLevelRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_lost_talent_tree_level";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CostItem" => {
                self.cost_item = ctx.decode_field(val, Self::TABLE_NAME, "CostItem")?;
            }
            b"CostNum" => {
                self.cost_num = ctx.decode_field(val, Self::TABLE_NAME, "CostNum")?;
            }
            b"FunctionModifier" => {
                self.function_modifier = ctx
                    .decode_field(val, Self::TABLE_NAME, "FunctionModifier")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Lv" => self.lv = ctx.decode_field(val, Self::TABLE_NAME, "Lv")?,
            b"MapModifier" => {
                self.map_modifier = ctx
                    .decode_field(val, Self::TABLE_NAME, "MapModifier")?;
            }
            b"NodeId" => {
                self.node_id = ctx.decode_field(val, Self::TABLE_NAME, "NodeId")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_lost_talent_tree_level.out`.
pub type ConfigMapGameplaySeasonLostTalentTreeLevelTable = DataTable<
    u32,
    ConfigMapGameplaySeasonLostTalentTreeLevelRow,
>;
/// Parses `config_map_gameplay_season_lost_talent_tree_level.out` into a [`ConfigMapGameplaySeasonLostTalentTreeLevelTable`].
pub fn parse_config_map_gameplay_season_lost_talent_tree_level(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonLostTalentTreeLevelTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_plague_card_bundle.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonPlagueCardBundleRow {
    pub bundle_rarity_weight: Box<[u8]>,
    pub desc: Box<str>,
    pub desc_params: Box<[u8]>,
    pub icon: Box<str>,
    pub icon2: Box<str>,
    pub name: Box<str>,
    pub _desc: Box<str>,
    pub _name: Box<str>,
    pub amount: u8,
    pub bundle_type: u8,
    pub id: i8,
    pub weight: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonPlagueCardBundleRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_plague_card_bundle";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Amount" => self.amount = ctx.decode_field(val, Self::TABLE_NAME, "Amount")?,
            b"BundleRarityWeight" => {
                self.bundle_rarity_weight = ctx
                    .decode_field(val, Self::TABLE_NAME, "BundleRarityWeight")?;
            }
            b"BundleType" => {
                self.bundle_type = ctx.decode_field(val, Self::TABLE_NAME, "BundleType")?;
            }
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DescParams" => {
                self.desc_params = ctx.decode_field(val, Self::TABLE_NAME, "DescParams")?;
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Icon2" => self.icon2 = ctx.decode_field(val, Self::TABLE_NAME, "Icon2")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_plague_card_bundle.out`.
pub type ConfigMapGameplaySeasonPlagueCardBundleTable = DataTable<
    u8,
    ConfigMapGameplaySeasonPlagueCardBundleRow,
>;
/// Parses `config_map_gameplay_season_plague_card_bundle.out` into a [`ConfigMapGameplaySeasonPlagueCardBundleTable`].
pub fn parse_config_map_gameplay_season_plague_card_bundle(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonPlagueCardBundleTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_plague_card_effect.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonPlagueCardEffectRow {
    pub desc: Box<str>,
    pub desc_params: Box<[u8]>,
    pub icon: Box<str>,
    pub icon2: Box<str>,
    pub name: Box<str>,
    pub _desc: Box<str>,
    pub _name: Box<str>,
    pub effect: u16,
    pub id: i16,
    pub appear_round_max: u8,
    pub appear_round_min: u8,
    pub rarity: u8,
    pub weight: u8,
}
impl ConfigMapGameplaySeasonPlagueCardEffectRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AppearRoundMax" => {
                self.appear_round_max = ctx
                    .decode_field(val, Self::TABLE_NAME, "AppearRoundMax")?;
            }
            b"AppearRoundMin" => {
                self.appear_round_min = ctx
                    .decode_field(val, Self::TABLE_NAME, "AppearRoundMin")?;
            }
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DescParams" => {
                self.desc_params = ctx.decode_field(val, Self::TABLE_NAME, "DescParams")?;
            }
            b"Effect" => self.effect = ctx.decode_field(val, Self::TABLE_NAME, "Effect")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Icon2" => self.icon2 = ctx.decode_field(val, Self::TABLE_NAME, "Icon2")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
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
impl DataCfgRow for ConfigMapGameplaySeasonPlagueCardEffectRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_plague_card_effect";
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
/// Typed `DataTable` for `config_map_gameplay_season_plague_card_effect.out`.
pub type ConfigMapGameplaySeasonPlagueCardEffectTable = DataTable<
    u16,
    ConfigMapGameplaySeasonPlagueCardEffectRow,
>;
/// Parses `config_map_gameplay_season_plague_card_effect.out` into a [`ConfigMapGameplaySeasonPlagueCardEffectTable`].
pub fn parse_config_map_gameplay_season_plague_card_effect(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonPlagueCardEffectTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_plague_card_modifier.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonPlagueCardModifierRow {
    pub desc: Box<str>,
    pub desc_params: Box<[u8]>,
    pub icon: Box<str>,
    pub icon2: Box<str>,
    pub modifier: Box<[u32]>,
    pub name: Box<str>,
    pub _desc: Box<str>,
    pub _name: Box<str>,
    pub r#type: u16,
    pub id: i8,
    pub rarity: u8,
    pub score_inc: u8,
    pub weight: u8,
}
impl ConfigMapGameplaySeasonPlagueCardModifierRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DescParams" => {
                self.desc_params = ctx.decode_field(val, Self::TABLE_NAME, "DescParams")?;
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Icon2" => self.icon2 = ctx.decode_field(val, Self::TABLE_NAME, "Icon2")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Modifier" => {
                self.modifier = ctx.decode_field(val, Self::TABLE_NAME, "Modifier")?;
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"ScoreInc" => {
                self.score_inc = ctx.decode_field(val, Self::TABLE_NAME, "ScoreInc")?;
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
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
impl DataCfgRow for ConfigMapGameplaySeasonPlagueCardModifierRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_plague_card_modifier";
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
/// Typed `DataTable` for `config_map_gameplay_season_plague_card_modifier.out`.
pub type ConfigMapGameplaySeasonPlagueCardModifierTable = DataTable<
    u8,
    ConfigMapGameplaySeasonPlagueCardModifierRow,
>;
/// Parses `config_map_gameplay_season_plague_card_modifier.out` into a [`ConfigMapGameplaySeasonPlagueCardModifierTable`].
pub fn parse_config_map_gameplay_season_plague_card_modifier(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonPlagueCardModifierTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_plague_card_relic.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonPlagueCardRelicRow {
    pub desc: Box<str>,
    pub desc_params: Box<[u16]>,
    pub effect: Box<[u32]>,
    pub icon: Box<str>,
    pub icon2: Box<str>,
    pub name: Box<str>,
    pub _desc: Box<str>,
    pub _name: Box<str>,
    pub id: i16,
    pub trigger: u16,
    pub r#type: u16,
    pub weight: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonPlagueCardRelicRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_plague_card_relic";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DescParams" => {
                self.desc_params = ctx.decode_field(val, Self::TABLE_NAME, "DescParams")?;
            }
            b"Effect" => self.effect = ctx.decode_field(val, Self::TABLE_NAME, "Effect")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Icon2" => self.icon2 = ctx.decode_field(val, Self::TABLE_NAME, "Icon2")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"Trigger" => {
                self.trigger = ctx.decode_field(val, Self::TABLE_NAME, "Trigger")?;
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"Weight" => self.weight = ctx.decode_field(val, Self::TABLE_NAME, "Weight")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_plague_card_relic.out`.
pub type ConfigMapGameplaySeasonPlagueCardRelicTable = DataTable<
    u8,
    ConfigMapGameplaySeasonPlagueCardRelicRow,
>;
/// Parses `config_map_gameplay_season_plague_card_relic.out` into a [`ConfigMapGameplaySeasonPlagueCardRelicTable`].
pub fn parse_config_map_gameplay_season_plague_card_relic(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonPlagueCardRelicTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_plague_card_relic_trigger.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonPlagueCardRelicTriggerRow {
    pub desc: Box<str>,
    pub sp1: Box<str>,
    pub _desc: Box<str>,
    pub id: i16,
    pub trigger_type: u16,
    pub enable_requirement: u8,
    pub is_continuous: bool,
    pub lifetime_target_trigger_limit: u8,
    pub one_turn_trigger_limit: u8,
    pub priority: u8,
    pub total_trigger_limit: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonPlagueCardRelicTriggerRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_plague_card_relic_trigger";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"EnableRequirement" => {
                self.enable_requirement = ctx
                    .decode_field(val, Self::TABLE_NAME, "EnableRequirement")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsContinuous" => {
                self.is_continuous = ctx
                    .decode_field(val, Self::TABLE_NAME, "IsContinuous")?;
            }
            b"LifetimeTargetTriggerLimit" => {
                self.lifetime_target_trigger_limit = ctx
                    .decode_field(val, Self::TABLE_NAME, "LifetimeTargetTriggerLimit")?;
            }
            b"OneTurnTriggerLimit" => {
                self.one_turn_trigger_limit = ctx
                    .decode_field(val, Self::TABLE_NAME, "OneTurnTriggerLimit")?;
            }
            b"Priority" => {
                self.priority = ctx.decode_field(val, Self::TABLE_NAME, "Priority")?;
            }
            b"Sp1" => self.sp1 = ctx.decode_field(val, Self::TABLE_NAME, "Sp1")?,
            b"TotalTriggerLimit" => {
                self.total_trigger_limit = ctx
                    .decode_field(val, Self::TABLE_NAME, "TotalTriggerLimit")?;
            }
            b"TriggerType" => {
                self.trigger_type = ctx
                    .decode_field(val, Self::TABLE_NAME, "TriggerType")?;
            }
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_plague_card_relic_trigger.out`.
pub type ConfigMapGameplaySeasonPlagueCardRelicTriggerTable = DataTable<
    u16,
    ConfigMapGameplaySeasonPlagueCardRelicTriggerRow,
>;
/// Parses `config_map_gameplay_season_plague_card_relic_trigger.out` into a [`ConfigMapGameplaySeasonPlagueCardRelicTriggerTable`].
pub fn parse_config_map_gameplay_season_plague_card_relic_trigger(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonPlagueCardRelicTriggerTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_plague_effect.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonPlagueEffectRow {
    pub sp1: Box<str>,
    pub sp2: Box<str>,
    pub sp3: Box<str>,
    pub sp4: Box<str>,
    pub sp5: Box<str>,
    pub sp6: Box<str>,
    pub id: i32,
    pub p2: u32,
    pub effect_type: u16,
    pub p1: u8,
    pub p3: u8,
    pub p4: u8,
    pub p5: u8,
    pub p6: u8,
}
impl ConfigMapGameplaySeasonPlagueEffectRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"EffectType" => {
                self.effect_type = ctx.decode_field(val, Self::TABLE_NAME, "EffectType")?;
            }
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
            b"Sp4" => self.sp4 = ctx.decode_field(val, Self::TABLE_NAME, "Sp4")?,
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
            b"Sp5" => self.sp5 = ctx.decode_field(val, Self::TABLE_NAME, "Sp5")?,
            b"Sp6" => self.sp6 = ctx.decode_field(val, Self::TABLE_NAME, "Sp6")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigMapGameplaySeasonPlagueEffectRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_plague_effect";
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
/// Typed `DataTable` for `config_map_gameplay_season_plague_effect.out`.
pub type ConfigMapGameplaySeasonPlagueEffectTable = DataTable<
    u32,
    ConfigMapGameplaySeasonPlagueEffectRow,
>;
/// Parses `config_map_gameplay_season_plague_effect.out` into a [`ConfigMapGameplaySeasonPlagueEffectTable`].
pub fn parse_config_map_gameplay_season_plague_effect(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonPlagueEffectTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_plague_monster_model.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonPlagueMonsterModelRow {
    pub anim_disappear: Box<str>,
    pub anim_idle: Box<str>,
    pub anim_show: Box<str>,
    pub sk_mesh: Box<str>,
    pub id: i8,
    pub race: u8,
    pub rarity: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonPlagueMonsterModelRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_plague_monster_model";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AnimDisappear" => {
                self.anim_disappear = ctx
                    .decode_field(val, Self::TABLE_NAME, "AnimDisappear")?;
            }
            b"AnimIdle" => {
                self.anim_idle = ctx.decode_field(val, Self::TABLE_NAME, "AnimIdle")?;
            }
            b"AnimShow" => {
                self.anim_show = ctx.decode_field(val, Self::TABLE_NAME, "AnimShow")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Race" => self.race = ctx.decode_field(val, Self::TABLE_NAME, "Race")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"SkMesh" => {
                self.sk_mesh = ctx.decode_field(val, Self::TABLE_NAME, "SkMesh")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_plague_monster_model.out`.
pub type ConfigMapGameplaySeasonPlagueMonsterModelTable = DataTable<
    u8,
    ConfigMapGameplaySeasonPlagueMonsterModelRow,
>;
/// Parses `config_map_gameplay_season_plague_monster_model.out` into a [`ConfigMapGameplaySeasonPlagueMonsterModelTable`].
pub fn parse_config_map_gameplay_season_plague_monster_model(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonPlagueMonsterModelTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_plague_score_tier.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonPlagueScoreTierRow {
    pub desc: Box<str>,
    pub desc_params: Box<[u8]>,
    pub icon: Box<str>,
    pub _desc: Box<str>,
    pub p1: u32,
    pub score: u32,
    pub r#type: u16,
    pub id: i8,
    pub p2: u8,
    pub p3: u8,
    pub relic_reward: u8,
    pub tier: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonPlagueScoreTierRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_plague_score_tier";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DescParams" => {
                self.desc_params = ctx.decode_field(val, Self::TABLE_NAME, "DescParams")?;
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"P1" => self.p1 = ctx.decode_field(val, Self::TABLE_NAME, "P1")?,
            b"P2" => self.p2 = ctx.decode_field(val, Self::TABLE_NAME, "P2")?,
            b"P3" => self.p3 = ctx.decode_field(val, Self::TABLE_NAME, "P3")?,
            b"RelicReward" => {
                self.relic_reward = ctx
                    .decode_field(val, Self::TABLE_NAME, "RelicReward")?;
            }
            b"Score" => self.score = ctx.decode_field(val, Self::TABLE_NAME, "Score")?,
            b"Tier" => self.tier = ctx.decode_field(val, Self::TABLE_NAME, "Tier")?,
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_plague_score_tier.out`.
pub type ConfigMapGameplaySeasonPlagueScoreTierTable = DataTable<
    u8,
    ConfigMapGameplaySeasonPlagueScoreTierRow,
>;
/// Parses `config_map_gameplay_season_plague_score_tier.out` into a [`ConfigMapGameplaySeasonPlagueScoreTierTable`].
pub fn parse_config_map_gameplay_season_plague_score_tier(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonPlagueScoreTierTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_roulette.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonRouletteRow {
    pub lottery_cost1: Box<[u8]>,
    pub progress_add_bomb: Box<[u8]>,
    pub progress_add_normal: Box<[u8]>,
    pub id: i16,
    pub grid1_type: u8,
    pub grid2_type: u8,
    pub grid3_type: u8,
    pub grid4_type: u8,
    pub grid5_type: u8,
    pub grid6_type: u8,
    pub grid7_type: u8,
    pub grid8_type: u8,
    pub progress: u8,
    pub r#type: u8,
}
impl ConfigMapGameplaySeasonRouletteRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Grid1Type" => {
                self.grid1_type = ctx.decode_field(val, Self::TABLE_NAME, "Grid1Type")?;
            }
            b"Grid2Type" => {
                self.grid2_type = ctx.decode_field(val, Self::TABLE_NAME, "Grid2Type")?;
            }
            b"Grid3Type" => {
                self.grid3_type = ctx.decode_field(val, Self::TABLE_NAME, "Grid3Type")?;
            }
            b"Grid4Type" => {
                self.grid4_type = ctx.decode_field(val, Self::TABLE_NAME, "Grid4Type")?;
            }
            b"Grid5Type" => {
                self.grid5_type = ctx.decode_field(val, Self::TABLE_NAME, "Grid5Type")?;
            }
            b"Grid6Type" => {
                self.grid6_type = ctx.decode_field(val, Self::TABLE_NAME, "Grid6Type")?;
            }
            b"Grid7Type" => {
                self.grid7_type = ctx.decode_field(val, Self::TABLE_NAME, "Grid7Type")?;
            }
            b"Grid8Type" => {
                self.grid8_type = ctx.decode_field(val, Self::TABLE_NAME, "Grid8Type")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"LotteryCost1" => {
                self.lottery_cost1 = ctx
                    .decode_field(val, Self::TABLE_NAME, "LotteryCost1")?;
            }
            b"Progress" => {
                self.progress = ctx.decode_field(val, Self::TABLE_NAME, "Progress")?;
            }
            b"ProgressAddBomb" => {
                self.progress_add_bomb = ctx
                    .decode_field(val, Self::TABLE_NAME, "ProgressAddBomb")?;
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
            b"ProgressAddNormal" => {
                self.progress_add_normal = ctx
                    .decode_field(val, Self::TABLE_NAME, "ProgressAddNormal")?;
            }
            b"Type" => self.r#type = ctx.decode_field(val, Self::TABLE_NAME, "Type")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigMapGameplaySeasonRouletteRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_roulette";
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
/// Typed `DataTable` for `config_map_gameplay_season_roulette.out`.
pub type ConfigMapGameplaySeasonRouletteTable = DataTable<
    u16,
    ConfigMapGameplaySeasonRouletteRow,
>;
/// Parses `config_map_gameplay_season_roulette.out` into a [`ConfigMapGameplaySeasonRouletteTable`].
pub fn parse_config_map_gameplay_season_roulette(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonRouletteTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_roulette_raity.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonRouletteRaityRow {
    pub id: i16,
    pub raity: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonRouletteRaityRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_roulette_raity";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Raity" => self.raity = ctx.decode_field(val, Self::TABLE_NAME, "Raity")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_roulette_raity.out`.
pub type ConfigMapGameplaySeasonRouletteRaityTable = DataTable<
    u16,
    ConfigMapGameplaySeasonRouletteRaityRow,
>;
/// Parses `config_map_gameplay_season_roulette_raity.out` into a [`ConfigMapGameplaySeasonRouletteRaityTable`].
pub fn parse_config_map_gameplay_season_roulette_raity(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonRouletteRaityTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_roulette_servant.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonRouletteServantRow {
    pub modifier: Box<[u32]>,
    pub servant_id: u16,
    pub id: i8,
    pub postion: u8,
    pub star_level: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonRouletteServantRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_roulette_servant";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Modifier" => {
                self.modifier = ctx.decode_field(val, Self::TABLE_NAME, "Modifier")?;
            }
            b"Postion" => {
                self.postion = ctx.decode_field(val, Self::TABLE_NAME, "Postion")?;
            }
            b"ServantId" => {
                self.servant_id = ctx.decode_field(val, Self::TABLE_NAME, "ServantId")?;
            }
            b"StarLevel" => {
                self.star_level = ctx.decode_field(val, Self::TABLE_NAME, "StarLevel")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_roulette_servant.out`.
pub type ConfigMapGameplaySeasonRouletteServantTable = DataTable<
    u8,
    ConfigMapGameplaySeasonRouletteServantRow,
>;
/// Parses `config_map_gameplay_season_roulette_servant.out` into a [`ConfigMapGameplaySeasonRouletteServantTable`].
pub fn parse_config_map_gameplay_season_roulette_servant(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonRouletteServantTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_facility_upgrade.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonTycoonFacilityUpgradeRow {
    pub criteria: FxHashMap<u16, u8>,
    pub criteria_port: FxHashMap<u8, u8>,
    pub drop_group: FxHashMap<u32, u32>,
    pub drop_group_equip: FxHashMap<u32, u32>,
    pub drop_resource: FxHashMap<u16, u8>,
    pub map_modifier: Box<[u32]>,
    pub resource_id: Box<[u8]>,
    pub upgrade_consume: FxHashMap<u8, u16>,
    pub upgrade_desc: Box<str>,
    pub upgrade_desc_para: Box<[u8]>,
    pub _upgrade_desc: Box<str>,
    pub criteria_boss: u16,
    pub enum_facility_type: u16,
    pub drop_lv: u8,
    pub drop_lv_equip: u8,
    pub efficiency: u8,
    pub group_amount: u8,
    pub id: i8,
    pub level: u8,
    pub worker_amount: u8,
}
impl ConfigMapGameplaySeasonTycoonFacilityUpgradeRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Criteria" => {
                self.criteria = ctx.decode_field(val, Self::TABLE_NAME, "Criteria")?;
            }
            b"CriteriaBoss" => {
                self.criteria_boss = ctx
                    .decode_field(val, Self::TABLE_NAME, "CriteriaBoss")?;
            }
            b"CriteriaPort" => {
                self.criteria_port = ctx
                    .decode_field(val, Self::TABLE_NAME, "CriteriaPort")?;
            }
            b"DropGroup" => {
                self.drop_group = ctx.decode_field(val, Self::TABLE_NAME, "DropGroup")?;
            }
            b"DropGroupEquip" => {
                self.drop_group_equip = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropGroupEquip")?;
            }
            b"DropLv" => {
                self.drop_lv = ctx.decode_field(val, Self::TABLE_NAME, "DropLv")?;
            }
            b"DropLvEquip" => {
                self.drop_lv_equip = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropLvEquip")?;
            }
            b"DropResource" => {
                self.drop_resource = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropResource")?;
            }
            b"Efficiency" => {
                self.efficiency = ctx.decode_field(val, Self::TABLE_NAME, "Efficiency")?;
            }
            b"EnumFacilityType" => {
                self.enum_facility_type = ctx
                    .decode_field(val, Self::TABLE_NAME, "EnumFacilityType")?;
            }
            b"GroupAmount" => {
                self.group_amount = ctx
                    .decode_field(val, Self::TABLE_NAME, "GroupAmount")?;
            }
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
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"MapModifier" => {
                self.map_modifier = ctx
                    .decode_field(val, Self::TABLE_NAME, "MapModifier")?;
            }
            b"ResourceId" => {
                self.resource_id = ctx.decode_field(val, Self::TABLE_NAME, "ResourceId")?;
            }
            b"UpgradeConsume" => {
                self.upgrade_consume = ctx
                    .decode_field(val, Self::TABLE_NAME, "UpgradeConsume")?;
            }
            b"UpgradeDesc" => {
                self.upgrade_desc = ctx
                    .decode_field(val, Self::TABLE_NAME, "UpgradeDesc")?;
            }
            b"UpgradeDescPara" => {
                self.upgrade_desc_para = ctx
                    .decode_field(val, Self::TABLE_NAME, "UpgradeDescPara")?;
            }
            b"WorkerAmount" => {
                self.worker_amount = ctx
                    .decode_field(val, Self::TABLE_NAME, "WorkerAmount")?;
            }
            b"_UpgradeDesc" => {
                self._upgrade_desc = ctx
                    .decode_field(val, Self::TABLE_NAME, "_UpgradeDesc")?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonFacilityUpgradeRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_facility_upgrade";
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
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_facility_upgrade.out`.
pub type ConfigMapGameplaySeasonTycoonFacilityUpgradeTable = DataTable<
    u8,
    ConfigMapGameplaySeasonTycoonFacilityUpgradeRow,
>;
/// Parses `config_map_gameplay_season_tycoon_facility_upgrade.out` into a [`ConfigMapGameplaySeasonTycoonFacilityUpgradeTable`].
pub fn parse_config_map_gameplay_season_tycoon_facility_upgrade(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonFacilityUpgradeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_facility_upgrade_data_fix.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonTycoonFacilityUpgradeDataFixRow {
    pub return_coin: u32,
    pub enum_facility_type: u16,
    pub id: i8,
    pub level: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonFacilityUpgradeDataFixRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_facility_upgrade_data_fix";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"EnumFacilityType" => {
                self.enum_facility_type = ctx
                    .decode_field(val, Self::TABLE_NAME, "EnumFacilityType")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"ReturnCoin" => {
                self.return_coin = ctx.decode_field(val, Self::TABLE_NAME, "ReturnCoin")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_facility_upgrade_data_fix.out`.
pub type ConfigMapGameplaySeasonTycoonFacilityUpgradeDataFixTable = DataTable<
    u8,
    ConfigMapGameplaySeasonTycoonFacilityUpgradeDataFixRow,
>;
/// Parses `config_map_gameplay_season_tycoon_facility_upgrade_data_fix.out` into a [`ConfigMapGameplaySeasonTycoonFacilityUpgradeDataFixTable`].
pub fn parse_config_map_gameplay_season_tycoon_facility_upgrade_data_fix(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonFacilityUpgradeDataFixTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_pillage.out`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigMapGameplaySeasonTycoonPillageRow {
    pub event_weight: FxHashMap<u16, u8>,
    pub positive_event_weight: FxHashMap<u16, u8>,
    pub special_rate: f64,
    pub target_amount_max_v2: FxHashMap<u8, u8>,
    pub target_weight: FxHashMap<u8, u8>,
    pub cd_reset_cost: u16,
    pub list_cd: u16,
    pub cost_calc_para: u8,
    pub id: i8,
    pub list_length_max: u8,
    pub special_amount_max: u8,
    pub target_amount_max: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonPillageRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_pillage";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"CdResetCost" => {
                self.cd_reset_cost = ctx
                    .decode_field(val, Self::TABLE_NAME, "CdResetCost")?;
            }
            b"CostCalcPara" => {
                self.cost_calc_para = ctx
                    .decode_field(val, Self::TABLE_NAME, "CostCalcPara")?;
            }
            b"EventWeight" => {
                self.event_weight = ctx
                    .decode_field(val, Self::TABLE_NAME, "EventWeight")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ListCd" => {
                self.list_cd = ctx.decode_field(val, Self::TABLE_NAME, "ListCd")?;
            }
            b"ListLengthMax" => {
                self.list_length_max = ctx
                    .decode_field(val, Self::TABLE_NAME, "ListLengthMax")?;
            }
            b"PositiveEventWeight" => {
                self.positive_event_weight = ctx
                    .decode_field(val, Self::TABLE_NAME, "PositiveEventWeight")?;
            }
            b"SpecialAmountMax" => {
                self.special_amount_max = ctx
                    .decode_field(val, Self::TABLE_NAME, "SpecialAmountMax")?;
            }
            b"SpecialRate" => {
                self.special_rate = ctx
                    .decode_field(val, Self::TABLE_NAME, "SpecialRate")?;
            }
            b"TargetAmountMax" => {
                self.target_amount_max = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetAmountMax")?;
            }
            b"TargetAmountMaxV2" => {
                self.target_amount_max_v2 = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetAmountMaxV2")?;
            }
            b"TargetWeight" => {
                self.target_weight = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetWeight")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_pillage.out`.
pub type ConfigMapGameplaySeasonTycoonPillageTable = DataTable<
    u8,
    ConfigMapGameplaySeasonTycoonPillageRow,
>;
/// Parses `config_map_gameplay_season_tycoon_pillage.out` into a [`ConfigMapGameplaySeasonTycoonPillageTable`].
pub fn parse_config_map_gameplay_season_tycoon_pillage(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonPillageTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_pillage_event.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonTycoonPillageEventRow {
    pub event_desc_group: Box<str>,
    pub event_param: FxHashMap<u32, u32>,
    pub image: Box<str>,
    pub _event_desc_group: Box<str>,
    pub event_type: u16,
    pub id: i16,
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonPillageEventRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_pillage_event";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"EventDescGroup" => {
                self.event_desc_group = ctx
                    .decode_field(val, Self::TABLE_NAME, "EventDescGroup")?;
            }
            b"EventParam" => {
                self.event_param = ctx.decode_field(val, Self::TABLE_NAME, "EventParam")?;
            }
            b"EventType" => {
                self.event_type = ctx.decode_field(val, Self::TABLE_NAME, "EventType")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Image" => self.image = ctx.decode_field(val, Self::TABLE_NAME, "Image")?,
            b"_EventDescGroup" => {
                self._event_desc_group = ctx
                    .decode_field(val, Self::TABLE_NAME, "_EventDescGroup")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_pillage_event.out`.
pub type ConfigMapGameplaySeasonTycoonPillageEventTable = DataTable<
    u16,
    ConfigMapGameplaySeasonTycoonPillageEventRow,
>;
/// Parses `config_map_gameplay_season_tycoon_pillage_event.out` into a [`ConfigMapGameplaySeasonTycoonPillageEventTable`].
pub fn parse_config_map_gameplay_season_tycoon_pillage_event(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonPillageEventTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_pillage_target.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonTycoonPillageTargetRow {
    pub drop_group_equip: FxHashMap<u32, u8>,
    pub event_rate: Box<[u8]>,
    pub image: Box<str>,
    pub name: Box<str>,
    pub reward_group: FxHashMap<u32, u8>,
    pub _name: Box<str>,
    pub event_id: u16,
    pub id: i16,
    pub target_time_max: u16,
    pub target_time_min: u16,
    pub drop_level_equip: u8,
    pub positive_event_rate: u8,
    pub reward_modifier: u8,
    pub special_weight: u8,
    pub target_level: u8,
    pub usage: u8,
}
impl ConfigMapGameplaySeasonTycoonPillageTargetRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"DropGroupEquip" => {
                self.drop_group_equip = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropGroupEquip")?;
            }
            b"DropLevelEquip" => {
                self.drop_level_equip = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropLevelEquip")?;
            }
            b"EventId" => {
                self.event_id = ctx.decode_field(val, Self::TABLE_NAME, "EventId")?;
            }
            b"EventRate" => {
                self.event_rate = ctx.decode_field(val, Self::TABLE_NAME, "EventRate")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Image" => self.image = ctx.decode_field(val, Self::TABLE_NAME, "Image")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"PositiveEventRate" => {
                self.positive_event_rate = ctx
                    .decode_field(val, Self::TABLE_NAME, "PositiveEventRate")?;
            }
            b"RewardGroup" => {
                self.reward_group = ctx
                    .decode_field(val, Self::TABLE_NAME, "RewardGroup")?;
            }
            b"RewardModifier" => {
                self.reward_modifier = ctx
                    .decode_field(val, Self::TABLE_NAME, "RewardModifier")?;
            }
            b"SpecialWeight" => {
                self.special_weight = ctx
                    .decode_field(val, Self::TABLE_NAME, "SpecialWeight")?;
            }
            b"TargetLevel" => {
                self.target_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetLevel")?;
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
            b"TargetTimeMax" => {
                self.target_time_max = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetTimeMax")?;
            }
            b"TargetTimeMin" => {
                self.target_time_min = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetTimeMin")?;
            }
            b"Usage" => self.usage = ctx.decode_field(val, Self::TABLE_NAME, "Usage")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonPillageTargetRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_pillage_target";
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
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_pillage_target.out`.
pub type ConfigMapGameplaySeasonTycoonPillageTargetTable = DataTable<
    u16,
    ConfigMapGameplaySeasonTycoonPillageTargetRow,
>;
/// Parses `config_map_gameplay_season_tycoon_pillage_target.out` into a [`ConfigMapGameplaySeasonTycoonPillageTargetTable`].
pub fn parse_config_map_gameplay_season_tycoon_pillage_target(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonPillageTargetTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_resource.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonTycoonResourceRow {
    pub consume_money_id: FxHashMap<u8, u8>,
    pub enum_resource_type: Box<[u8]>,
    pub icon: Box<str>,
    pub name: Box<str>,
    pub _name: Box<str>,
    pub id: i16,
    pub money_id: u16,
    pub production_amount: u16,
    pub trade_value: u8,
    pub worker_requirement: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonResourceRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_resource";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ConsumeMoneyId" => {
                self.consume_money_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "ConsumeMoneyId")?;
            }
            b"EnumResourceType" => {
                self.enum_resource_type = ctx
                    .decode_field(val, Self::TABLE_NAME, "EnumResourceType")?;
            }
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"MoneyId" => {
                self.money_id = ctx.decode_field(val, Self::TABLE_NAME, "MoneyId")?;
            }
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"ProductionAmount" => {
                self.production_amount = ctx
                    .decode_field(val, Self::TABLE_NAME, "ProductionAmount")?;
            }
            b"TradeValue" => {
                self.trade_value = ctx.decode_field(val, Self::TABLE_NAME, "TradeValue")?;
            }
            b"WorkerRequirement" => {
                self.worker_requirement = ctx
                    .decode_field(val, Self::TABLE_NAME, "WorkerRequirement")?;
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_resource.out`.
pub type ConfigMapGameplaySeasonTycoonResourceTable = DataTable<
    u8,
    ConfigMapGameplaySeasonTycoonResourceRow,
>;
/// Parses `config_map_gameplay_season_tycoon_resource.out` into a [`ConfigMapGameplaySeasonTycoonResourceTable`].
pub fn parse_config_map_gameplay_season_tycoon_resource(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonResourceTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_trade_port.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonTycoonTradePortRow {
    pub desc: Box<str>,
    pub drop_group: FxHashMap<u32, u32>,
    pub drop_group_equip: FxHashMap<u32, u32>,
    pub drop_level_v2: FxHashMap<u8, u8>,
    pub drop_modifier: Box<str>,
    pub item_show_id: Box<[u8]>,
    pub name: Box<str>,
    pub quest_group: Box<[u16]>,
    pub quest_group_v2: FxHashMap<u16, u8>,
    pub upgrade_desc: Box<str>,
    pub upgrade_desc_para: Box<[u8]>,
    pub _desc: Box<str>,
    pub _drop_modifier: Box<str>,
    pub _name: Box<str>,
    pub _upgrade_desc: Box<str>,
    pub resource_limit: u32,
    pub port_id: u16,
    pub trade_time: u16,
    pub drop_level: u8,
    pub drop_level_equip: u8,
    pub id: i8,
    pub level: u8,
    pub position: u8,
    pub unlock: u8,
    pub upgrade_requirement: u8,
}
impl ConfigMapGameplaySeasonTycoonTradePortRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"DropGroup" => {
                self.drop_group = ctx.decode_field(val, Self::TABLE_NAME, "DropGroup")?;
            }
            b"DropGroupEquip" => {
                self.drop_group_equip = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropGroupEquip")?;
            }
            b"DropLevel" => {
                self.drop_level = ctx.decode_field(val, Self::TABLE_NAME, "DropLevel")?;
            }
            b"DropLevelEquip" => {
                self.drop_level_equip = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropLevelEquip")?;
            }
            b"DropLevelV2" => {
                self.drop_level_v2 = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropLevelV2")?;
            }
            b"DropModifier" => {
                self.drop_modifier = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropModifier")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemShowId" => {
                self.item_show_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemShowId")?;
            }
            b"Level" => self.level = ctx.decode_field(val, Self::TABLE_NAME, "Level")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"PortId" => {
                self.port_id = ctx.decode_field(val, Self::TABLE_NAME, "PortId")?;
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
            b"Position" => {
                self.position = ctx.decode_field(val, Self::TABLE_NAME, "Position")?;
            }
            b"QuestGroup" => {
                self.quest_group = ctx.decode_field(val, Self::TABLE_NAME, "QuestGroup")?;
            }
            b"QuestGroupV2" => {
                self.quest_group_v2 = ctx
                    .decode_field(val, Self::TABLE_NAME, "QuestGroupV2")?;
            }
            b"ResourceLimit" => {
                self.resource_limit = ctx
                    .decode_field(val, Self::TABLE_NAME, "ResourceLimit")?;
            }
            b"TradeTime" => {
                self.trade_time = ctx.decode_field(val, Self::TABLE_NAME, "TradeTime")?;
            }
            b"Unlock" => self.unlock = ctx.decode_field(val, Self::TABLE_NAME, "Unlock")?,
            b"UpgradeDesc" => {
                self.upgrade_desc = ctx
                    .decode_field(val, Self::TABLE_NAME, "UpgradeDesc")?;
            }
            b"UpgradeDescPara" => {
                self.upgrade_desc_para = ctx
                    .decode_field(val, Self::TABLE_NAME, "UpgradeDescPara")?;
            }
            b"UpgradeRequirement" => {
                self.upgrade_requirement = ctx
                    .decode_field(val, Self::TABLE_NAME, "UpgradeRequirement")?;
            }
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            b"_DropModifier" => {
                self._drop_modifier = ctx
                    .decode_field(val, Self::TABLE_NAME, "_DropModifier")?;
            }
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
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
            b"_UpgradeDesc" => {
                self._upgrade_desc = ctx
                    .decode_field(val, Self::TABLE_NAME, "_UpgradeDesc")?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonTradePortRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_trade_port";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        if self.set_field_0(ctx, field, val)? || self.set_field_1(ctx, field, val)?
            || self.set_field_2(ctx, field, val)?
        {
            return Ok(());
        }
        Err(ctx.unknown_field(Self::TABLE_NAME, field))
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_trade_port.out`.
pub type ConfigMapGameplaySeasonTycoonTradePortTable = DataTable<
    u8,
    ConfigMapGameplaySeasonTycoonTradePortRow,
>;
/// Parses `config_map_gameplay_season_tycoon_trade_port.out` into a [`ConfigMapGameplaySeasonTycoonTradePortTable`].
pub fn parse_config_map_gameplay_season_tycoon_trade_port(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonTradePortTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_trade_port_quest.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonTycoonTradePortQuestRow {
    pub drop_group_currency: Box<[u32]>,
    pub drop_group_equip: Box<[u32]>,
    pub item_show_id: Box<[u16]>,
    pub resource_amount_v2: Box<[u32]>,
    pub reward_info: Box<str>,
    pub _reward_info: Box<str>,
    pub resource_amount: u32,
    pub id: i16,
    pub efficiency: u8,
    pub resource_id: u8,
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonTradePortQuestRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_trade_port_quest";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"DropGroupCurrency" => {
                self.drop_group_currency = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropGroupCurrency")?;
            }
            b"DropGroupEquip" => {
                self.drop_group_equip = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropGroupEquip")?;
            }
            b"Efficiency" => {
                self.efficiency = ctx.decode_field(val, Self::TABLE_NAME, "Efficiency")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemShowId" => {
                self.item_show_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemShowId")?;
            }
            b"ResourceAmount" => {
                self.resource_amount = ctx
                    .decode_field(val, Self::TABLE_NAME, "ResourceAmount")?;
            }
            b"ResourceAmountV2" => {
                self.resource_amount_v2 = ctx
                    .decode_field(val, Self::TABLE_NAME, "ResourceAmountV2")?;
            }
            b"ResourceId" => {
                self.resource_id = ctx.decode_field(val, Self::TABLE_NAME, "ResourceId")?;
            }
            b"RewardInfo" => {
                self.reward_info = ctx.decode_field(val, Self::TABLE_NAME, "RewardInfo")?;
            }
            b"_RewardInfo" => {
                self._reward_info = ctx
                    .decode_field(val, Self::TABLE_NAME, "_RewardInfo")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_trade_port_quest.out`.
pub type ConfigMapGameplaySeasonTycoonTradePortQuestTable = DataTable<
    u16,
    ConfigMapGameplaySeasonTycoonTradePortQuestRow,
>;
/// Parses `config_map_gameplay_season_tycoon_trade_port_quest.out` into a [`ConfigMapGameplaySeasonTycoonTradePortQuestTable`].
pub fn parse_config_map_gameplay_season_tycoon_trade_port_quest(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonTradePortQuestTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_worker.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonTycoonWorkerRow {
    pub dismiss_reward: FxHashMap<u8, u8>,
    pub drop_weight: FxHashMap<u8, u8>,
    pub facility_level: FxHashMap<u16, u8>,
    pub job_icon: Box<str>,
    pub job_name: Box<str>,
    pub modifier_group: FxHashMap<u32, Box<[u32]>>,
    pub _job_name: Box<str>,
    pub worker_item: u32,
    pub coin_consume_max: u16,
    pub coin_consume_min: u16,
    pub facility_type: u16,
    pub icon_group: u16,
    pub name_group: u16,
    pub efficiency: u8,
    pub id: i8,
    pub modifier_amount: u8,
    pub worker_level: u8,
}
impl ConfigMapGameplaySeasonTycoonWorkerRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"CoinConsumeMax" => {
                self.coin_consume_max = ctx
                    .decode_field(val, Self::TABLE_NAME, "CoinConsumeMax")?;
            }
            b"CoinConsumeMin" => {
                self.coin_consume_min = ctx
                    .decode_field(val, Self::TABLE_NAME, "CoinConsumeMin")?;
            }
            b"DismissReward" => {
                self.dismiss_reward = ctx
                    .decode_field(val, Self::TABLE_NAME, "DismissReward")?;
            }
            b"DropWeight" => {
                self.drop_weight = ctx.decode_field(val, Self::TABLE_NAME, "DropWeight")?;
            }
            b"Efficiency" => {
                self.efficiency = ctx.decode_field(val, Self::TABLE_NAME, "Efficiency")?;
            }
            b"FacilityLevel" => {
                self.facility_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "FacilityLevel")?;
            }
            b"FacilityType" => {
                self.facility_type = ctx
                    .decode_field(val, Self::TABLE_NAME, "FacilityType")?;
            }
            b"IconGroup" => {
                self.icon_group = ctx.decode_field(val, Self::TABLE_NAME, "IconGroup")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"JobIcon" => {
                self.job_icon = ctx.decode_field(val, Self::TABLE_NAME, "JobIcon")?;
            }
            b"JobName" => {
                self.job_name = ctx.decode_field(val, Self::TABLE_NAME, "JobName")?;
            }
            b"ModifierAmount" => {
                self.modifier_amount = ctx
                    .decode_field(val, Self::TABLE_NAME, "ModifierAmount")?;
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
            b"ModifierGroup" => {
                self.modifier_group = ctx
                    .decode_field(val, Self::TABLE_NAME, "ModifierGroup")?;
            }
            b"NameGroup" => {
                self.name_group = ctx.decode_field(val, Self::TABLE_NAME, "NameGroup")?;
            }
            b"WorkerItem" => {
                self.worker_item = ctx.decode_field(val, Self::TABLE_NAME, "WorkerItem")?;
            }
            b"WorkerLevel" => {
                self.worker_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "WorkerLevel")?;
            }
            b"_JobName" => {
                self._job_name = ctx.decode_field(val, Self::TABLE_NAME, "_JobName")?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonWorkerRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_worker";
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
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_worker.out`.
pub type ConfigMapGameplaySeasonTycoonWorkerTable = DataTable<
    u8,
    ConfigMapGameplaySeasonTycoonWorkerRow,
>;
/// Parses `config_map_gameplay_season_tycoon_worker.out` into a [`ConfigMapGameplaySeasonTycoonWorkerTable`].
pub fn parse_config_map_gameplay_season_tycoon_worker(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonWorkerTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_worker_icon.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonTycoonWorkerIconRow {
    pub icon: Box<str>,
    pub icon64: Box<str>,
    pub group: u16,
    pub id: i8,
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonWorkerIconRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_worker_icon";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Icon" => self.icon = ctx.decode_field(val, Self::TABLE_NAME, "Icon")?,
            b"Icon64" => self.icon64 = ctx.decode_field(val, Self::TABLE_NAME, "Icon64")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_worker_icon.out`.
pub type ConfigMapGameplaySeasonTycoonWorkerIconTable = DataTable<
    u8,
    ConfigMapGameplaySeasonTycoonWorkerIconRow,
>;
/// Parses `config_map_gameplay_season_tycoon_worker_icon.out` into a [`ConfigMapGameplaySeasonTycoonWorkerIconTable`].
pub fn parse_config_map_gameplay_season_tycoon_worker_icon(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonWorkerIconTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_map_gameplay_season_tycoon_worker_name.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMapGameplaySeasonTycoonWorkerNameRow {
    pub name: Box<str>,
    pub _name: Box<str>,
    pub group: u16,
    pub id: i16,
}
impl DataCfgRow for ConfigMapGameplaySeasonTycoonWorkerNameRow {
    const TABLE_NAME: &'static str = "config_map_gameplay_season_tycoon_worker_name";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Group" => self.group = ctx.decode_field(val, Self::TABLE_NAME, "Group")?,
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"Name" => self.name = ctx.decode_field(val, Self::TABLE_NAME, "Name")?,
            b"_Name" => self._name = ctx.decode_field(val, Self::TABLE_NAME, "_Name")?,
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_map_gameplay_season_tycoon_worker_name.out`.
pub type ConfigMapGameplaySeasonTycoonWorkerNameTable = DataTable<
    u8,
    ConfigMapGameplaySeasonTycoonWorkerNameRow,
>;
/// Parses `config_map_gameplay_season_tycoon_worker_name.out` into a [`ConfigMapGameplaySeasonTycoonWorkerNameTable`].
pub fn parse_config_map_gameplay_season_tycoon_worker_name(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigMapGameplaySeasonTycoonWorkerNameTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
#[inline(never)]
fn verify_map_chunk_0(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_map_default_mod_random" => {
            Some(parse_config_map_default_mod_random(buf, scratch).map(|t| t.len()))
        }
        "config_map_environment" => {
            Some(parse_config_map_environment(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_climbtower" => {
            Some(parse_config_map_gameplay_climbtower(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_climbtower_ticket" => {
            Some(
                parse_config_map_gameplay_climbtower_ticket(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_hero_trial" => {
            Some(parse_config_map_gameplay_hero_trial(buf, scratch).map(|t| t.len()))
        }
        "config_map_gameplay_season_doll_text_display" => {
            Some(
                parse_config_map_gameplay_season_doll_text_display(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_dream_talent_costlv" => {
            Some(
                parse_config_map_gameplay_season_dream_talent_costlv(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_dream_talent_tree" => {
            Some(
                parse_config_map_gameplay_season_dream_talent_tree(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_lost_bigmap_difficulty" => {
            Some(
                parse_config_map_gameplay_season_lost_bigmap_difficulty(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_lost_reward_boss" => {
            Some(
                parse_config_map_gameplay_season_lost_reward_boss(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_map_chunk_1(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_map_gameplay_season_lost_reward_drop" => {
            Some(
                parse_config_map_gameplay_season_lost_reward_drop(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_lost_reward_type" => {
            Some(
                parse_config_map_gameplay_season_lost_reward_type(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_lost_talent_tree" => {
            Some(
                parse_config_map_gameplay_season_lost_talent_tree(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_lost_talent_tree_level" => {
            Some(
                parse_config_map_gameplay_season_lost_talent_tree_level(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_plague_card_bundle" => {
            Some(
                parse_config_map_gameplay_season_plague_card_bundle(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_plague_card_effect" => {
            Some(
                parse_config_map_gameplay_season_plague_card_effect(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_plague_card_modifier" => {
            Some(
                parse_config_map_gameplay_season_plague_card_modifier(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_plague_card_relic" => {
            Some(
                parse_config_map_gameplay_season_plague_card_relic(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_plague_card_relic_trigger" => {
            Some(
                parse_config_map_gameplay_season_plague_card_relic_trigger(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_plague_effect" => {
            Some(
                parse_config_map_gameplay_season_plague_effect(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_map_chunk_2(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_map_gameplay_season_plague_monster_model" => {
            Some(
                parse_config_map_gameplay_season_plague_monster_model(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_plague_score_tier" => {
            Some(
                parse_config_map_gameplay_season_plague_score_tier(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_roulette" => {
            Some(
                parse_config_map_gameplay_season_roulette(buf, scratch).map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_roulette_raity" => {
            Some(
                parse_config_map_gameplay_season_roulette_raity(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_roulette_servant" => {
            Some(
                parse_config_map_gameplay_season_roulette_servant(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_tycoon_facility_upgrade" => {
            Some(
                parse_config_map_gameplay_season_tycoon_facility_upgrade(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_tycoon_facility_upgrade_data_fix" => {
            Some(
                parse_config_map_gameplay_season_tycoon_facility_upgrade_data_fix(
                        buf,
                        scratch,
                    )
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_tycoon_pillage" => {
            Some(
                parse_config_map_gameplay_season_tycoon_pillage(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_tycoon_pillage_event" => {
            Some(
                parse_config_map_gameplay_season_tycoon_pillage_event(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_tycoon_pillage_target" => {
            Some(
                parse_config_map_gameplay_season_tycoon_pillage_target(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_map_chunk_3(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_map_gameplay_season_tycoon_resource" => {
            Some(
                parse_config_map_gameplay_season_tycoon_resource(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_tycoon_trade_port" => {
            Some(
                parse_config_map_gameplay_season_tycoon_trade_port(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_tycoon_trade_port_quest" => {
            Some(
                parse_config_map_gameplay_season_tycoon_trade_port_quest(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_tycoon_worker" => {
            Some(
                parse_config_map_gameplay_season_tycoon_worker(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_tycoon_worker_icon" => {
            Some(
                parse_config_map_gameplay_season_tycoon_worker_icon(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_map_gameplay_season_tycoon_worker_name" => {
            Some(
                parse_config_map_gameplay_season_tycoon_worker_name(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        _ => None,
    }
}
pub(crate) fn verify_map_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    if let Some(res) = verify_map_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_map_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_map_chunk_2(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_map_chunk_3(stem, buf, scratch) {
        return Some(res);
    }
    None
}

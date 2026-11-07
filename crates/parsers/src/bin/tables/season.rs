//! Auto-generated `.bin` configuration table types and parsers for the `season` domain.
use crate::bin::affix::FConfigAffix;
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::FConfigI32I32;
use crate::bin::types::{BinTable, VecDim};
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_S11_Gameplay_Bounty_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GameplayBountyRow {
    pub quality_name: Box<str>,
    pub id: i32,
    pub quality: i32,
}
impl BinRow for ConfigS11GameplayBountyRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.quality = cur.read_i32()?;
        self.quality_name = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_Bounty_ChunkA0.bin`.
pub type ConfigS11GameplayBountyTable = BinTable<i32, ConfigS11GameplayBountyRow>;
/// Parses `config_S11_Gameplay_Bounty_ChunkA0.bin` into a [`ConfigS11GameplayBountyTable`].
pub fn parse_config_s11_gameplay_bounty(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplayBountyTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S11_Gameplay_BountyCondtion_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GameplayBountyCondtionRow {
    pub condition_name: Box<str>,
    pub id: i32,
    pub final_game: i32,
    pub series: i32,
    pub level: i32,
    pub quality: i32,
    pub condition_type: i32,
    pub gameplay_type: i32,
    pub weight: i32,
    pub achieve_type: i32,
    pub count_num: i32,
}
impl BinRow for ConfigS11GameplayBountyCondtionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.final_game = cur.read_i32()?;
        self.series = cur.read_i32()?;
        self.level = cur.read_i32()?;
        self.quality = cur.read_i32()?;
        self.condition_type = cur.read_i32()?;
        self.gameplay_type = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        self.achieve_type = cur.read_i32()?;
        self.count_num = cur.read_i32()?;
        self.condition_name = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_BountyCondtion_ChunkA0.bin`.
pub type ConfigS11GameplayBountyCondtionTable = BinTable<
    i32,
    ConfigS11GameplayBountyCondtionRow,
>;
/// Parses `config_S11_Gameplay_BountyCondtion_ChunkA0.bin` into a [`ConfigS11GameplayBountyCondtionTable`].
pub fn parse_config_s11_gameplay_bounty_condtion(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplayBountyCondtionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S11_Gameplay_BountyLevel_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GameplayBountyLevelRow {
    pub cost: FxHashMap<i32, i32>,
    pub cond1_des: Box<str>,
    pub cond2_des: Box<str>,
    pub cond3_des: Box<str>,
    pub coffer_level: FxHashMap<i32, i32>,
    pub affix: FxHashMap<i32, Box<[i32]>>,
    pub npc_lines: Box<str>,
    pub id: i32,
    pub level: i32,
    pub effect_area_lv: i32,
    pub cost_type: i32,
    pub cond1_type: i32,
    pub cond1_param1: i32,
    pub cond2_type: i32,
    pub cond2_param1: i32,
    pub cond3_type: i32,
    pub cond3_param1: i32,
    pub bounty_rate: i32,
    pub coin_bonus: i32,
    pub quality_id: i32,
    pub bounty_min_times: i32,
    pub max_currency: i32,
    pub cond_area_lv: i32,
    pub max_monster_limit: i32,
}
impl ConfigS11GameplayBountyLevelRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.level = cur.read_i32()?;
        self.effect_area_lv = cur.read_i32()?;
        self.cost_type = cur.read_i32()?;
        self.cost = cur.read_map_i32_i32()?;
        self.cond1_type = cur.read_i32()?;
        self.cond1_param1 = cur.read_i32()?;
        self.cond1_des = cur.read_str()?;
        self.cond2_type = cur.read_i32()?;
        self.cond2_param1 = cur.read_i32()?;
        self.cond2_des = cur.read_str()?;
        self.cond3_type = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.cond3_param1 = cur.read_i32()?;
        self.cond3_des = cur.read_str()?;
        self.coffer_level = cur.read_map_i32_i32()?;
        self.bounty_rate = cur.read_i32()?;
        self.coin_bonus = cur.read_i32()?;
        self.quality_id = cur.read_i32()?;
        self.bounty_min_times = cur.read_i32()?;
        self.affix = cur.read_map_i32_array_i32()?;
        self.max_currency = cur.read_i32()?;
        self.npc_lines = cur.read_str()?;
        self.cond_area_lv = cur.read_i32()?;
        self.max_monster_limit = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigS11GameplayBountyLevelRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_BountyLevel_ChunkA0.bin`.
pub type ConfigS11GameplayBountyLevelTable = BinTable<
    i32,
    ConfigS11GameplayBountyLevelRow,
>;
/// Parses `config_S11_Gameplay_BountyLevel_ChunkA0.bin` into a [`ConfigS11GameplayBountyLevelTable`].
pub fn parse_config_s11_gameplay_bounty_level(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplayBountyLevelTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S11_Gameplay_BountyReward_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GameplayBountyRewardRow {
    pub id: i32,
    pub col_1: i32,
    pub col_2: i32,
    pub col_3: bool,
}
impl BinRow for ConfigS11GameplayBountyRewardRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.col_1 = cur.read_i32()?;
        self.col_2 = cur.read_i32()?;
        self.col_3 = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_BountyReward_ChunkA0.bin`.
pub type ConfigS11GameplayBountyRewardTable = BinTable<
    i32,
    ConfigS11GameplayBountyRewardRow,
>;
/// Parses `config_S11_Gameplay_BountyReward_ChunkA0.bin` into a [`ConfigS11GameplayBountyRewardTable`].
pub fn parse_config_s11_gameplay_bounty_reward(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplayBountyRewardTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S11_Gameplay_Coffer_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GameplayCofferRow {
    pub bounty_quality: FxHashMap<i32, i32>,
    pub bounty_cost: FxHashMap<i32, i32>,
    pub id: i32,
    pub coffer_level: i32,
    pub bounty_cost_type: i32,
    pub col_5: bool,
}
impl BinRow for ConfigS11GameplayCofferRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.coffer_level = cur.read_i32()?;
        self.bounty_quality = cur.read_map_i32_i32()?;
        self.bounty_cost_type = cur.read_i32()?;
        self.bounty_cost = cur.read_map_i32_i32()?;
        self.col_5 = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_Coffer_ChunkA0.bin`.
pub type ConfigS11GameplayCofferTable = BinTable<i32, ConfigS11GameplayCofferRow>;
/// Parses `config_S11_Gameplay_Coffer_ChunkA0.bin` into a [`ConfigS11GameplayCofferTable`].
pub fn parse_config_s11_gameplay_coffer(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplayCofferTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S11_Gameplay_CofferBattle_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GameplayCofferBattleRow {
    pub coffer_des: Box<str>,
    pub id: i32,
    pub battle_type: i32,
    pub coffer_level: i32,
    pub gameplay_type: i32,
    pub weight: i32,
    pub limit_time: i32,
    pub coffer_number: i32,
    pub final_game: bool,
}
impl BinRow for ConfigS11GameplayCofferBattleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.battle_type = cur.read_i32()?;
        self.coffer_level = cur.read_i32()?;
        self.gameplay_type = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        self.limit_time = cur.read_i32()?;
        self.final_game = cur.read_bool()?;
        self.coffer_des = cur.read_str()?;
        self.coffer_number = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_CofferBattle_ChunkA0.bin`.
pub type ConfigS11GameplayCofferBattleTable = BinTable<
    i32,
    ConfigS11GameplayCofferBattleRow,
>;
/// Parses `config_S11_Gameplay_CofferBattle_ChunkA0.bin` into a [`ConfigS11GameplayCofferBattleTable`].
pub fn parse_config_s11_gameplay_coffer_battle(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplayCofferBattleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S11_Gameplay_FinalGame_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GameplayFinalGameRow {
    pub difficulty_name: Box<str>,
    pub difficulty_des: Box<str>,
    pub difficulty_level: Box<str>,
    pub debuff_quality: Box<str>,
    pub item_access: Box<[FConfigI32I32]>,
    pub currency_access: Box<[FConfigI32I32]>,
    pub unlock_condition: FxHashMap<i32, i32>,
    pub id: i32,
    pub difficulty: i32,
    pub drop_bonus: i32,
    pub level_id: i32,
    pub limit_time: i32,
    pub allow_fail_count: i32,
    pub progress_value: i32,
    pub coffer_add_progress: i32,
    pub coffer_num: i32,
    pub coffer_level: i32,
}
impl ConfigS11GameplayFinalGameRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.difficulty = cur.read_i32()?;
        self.difficulty_name = cur.read_str()?;
        self.difficulty_des = cur.read_str()?;
        self.difficulty_level = cur.read_str()?;
        self.drop_bonus = cur.read_i32()?;
        self.debuff_quality = cur.read_str()?;
        self.item_access = FConfigI32I32::read_array(cur)?;
        self.currency_access = FConfigI32I32::read_array(cur)?;
        self.unlock_condition = cur.read_map_i32_i32()?;
        self.level_id = cur.read_i32()?;
        self.limit_time = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.allow_fail_count = cur.read_i32()?;
        self.progress_value = cur.read_i32()?;
        self.coffer_add_progress = cur.read_i32()?;
        self.coffer_num = cur.read_i32()?;
        self.coffer_level = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigS11GameplayFinalGameRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_FinalGame_ChunkA0.bin`.
pub type ConfigS11GameplayFinalGameTable = BinTable<i32, ConfigS11GameplayFinalGameRow>;
/// Parses `config_S11_Gameplay_FinalGame_ChunkA0.bin` into a [`ConfigS11GameplayFinalGameTable`].
pub fn parse_config_s11_gameplay_final_game(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplayFinalGameTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S11_Gameplay_FinalGameRaise_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigS11GameplayFinalGameRaiseRow {
    pub raise_cost: FxHashMap<i32, i32>,
    pub reward_affix: Box<[FConfigAffix]>,
    pub id: i32,
    pub difficulty: i32,
    pub raise_times: i32,
    pub raise_cost_type: i32,
}
impl BinRow for ConfigS11GameplayFinalGameRaiseRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.difficulty = cur.read_i32()?;
        self.raise_times = cur.read_i32()?;
        self.raise_cost_type = cur.read_i32()?;
        self.raise_cost = cur.read_map_i32_i32()?;
        self.reward_affix = FConfigAffix::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_FinalGameRaise_ChunkA0.bin`.
pub type ConfigS11GameplayFinalGameRaiseTable = BinTable<
    i32,
    ConfigS11GameplayFinalGameRaiseRow,
>;
/// Parses `config_S11_Gameplay_FinalGameRaise_ChunkA0.bin` into a [`ConfigS11GameplayFinalGameRaiseTable`].
pub fn parse_config_s11_gameplay_final_game_raise(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplayFinalGameRaiseTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S11_Gameplay_RandomEvent_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GameplayRandomEventRow {
    pub opportunity_weight: FxHashMap<i32, i32>,
    pub event_weight: FxHashMap<i32, i32>,
    pub id: i32,
    pub opportunity_type: i32,
    pub quality: i32,
    pub gameplay_type: i32,
    pub coffer_level_unlock: i32,
}
impl BinRow for ConfigS11GameplayRandomEventRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.opportunity_type = cur.read_i32()?;
        self.quality = cur.read_i32()?;
        self.gameplay_type = cur.read_i32()?;
        self.coffer_level_unlock = cur.read_i32()?;
        self.opportunity_weight = cur.read_map_i32_i32()?;
        self.event_weight = cur.read_map_i32_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_RandomEvent_ChunkA0.bin`.
pub type ConfigS11GameplayRandomEventTable = BinTable<
    i32,
    ConfigS11GameplayRandomEventRow,
>;
/// Parses `config_S11_Gameplay_RandomEvent_ChunkA0.bin` into a [`ConfigS11GameplayRandomEventTable`].
pub fn parse_config_s11_gameplay_random_event(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplayRandomEventTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S11_Gameplay_Simulate_Base_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigS11GameplaySimulateBaseRow {
    pub count_rare: f64,
    pub npc_rare: Box<[i32]>,
    pub count_magic: f64,
    pub npc_magic: Box<[i32]>,
    pub count_normal: f64,
    pub npc_normal: Box<[i32]>,
    pub count_drop_carrier: f64,
    pub drop_carrier: Box<[i32]>,
    pub replace_talent_prop: Box<[i32]>,
    pub replace_card_prop: Box<[i32]>,
    pub id: i32,
}
impl BinRow for ConfigS11GameplaySimulateBaseRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.count_rare = cur.read_f64()?;
        self.npc_rare = cur.read_array_i32()?;
        self.count_magic = cur.read_f64()?;
        self.npc_magic = cur.read_array_i32()?;
        self.count_normal = cur.read_f64()?;
        self.npc_normal = cur.read_array_i32()?;
        self.count_drop_carrier = cur.read_f64()?;
        self.drop_carrier = cur.read_array_i32()?;
        self.replace_talent_prop = cur.read_array_i32()?;
        self.replace_card_prop = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_Simulate_Base_ChunkA0.bin`.
pub type ConfigS11GameplaySimulateBaseTable = BinTable<
    i32,
    ConfigS11GameplaySimulateBaseRow,
>;
/// Parses `config_S11_Gameplay_Simulate_Base_ChunkA0.bin` into a [`ConfigS11GameplaySimulateBaseTable`].
pub fn parse_config_s11_gameplay_simulate_base(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplaySimulateBaseTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S11_Gameplay_Simulate_Event_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS11GameplaySimulateEventRow {
    pub result_monster1: Box<[i32]>,
    pub result_monster2: Box<[i32]>,
    pub result_monster3: Box<[i32]>,
    pub result_drop_carrier1: Box<[i32]>,
    pub result_drop_carrier2: Box<[i32]>,
    pub result_drop_carrier3: Box<[i32]>,
    pub id: i32,
    pub trigger_type: i32,
    pub trigger_event_id: i32,
    pub limit: i32,
    pub prop: i32,
}
impl BinRow for ConfigS11GameplaySimulateEventRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.trigger_type = cur.read_i32()?;
        self.trigger_event_id = cur.read_i32()?;
        self.limit = cur.read_i32()?;
        self.prop = cur.read_i32()?;
        self.result_monster1 = cur.read_array_i32()?;
        self.result_monster2 = cur.read_array_i32()?;
        self.result_monster3 = cur.read_array_i32()?;
        self.result_drop_carrier1 = cur.read_array_i32()?;
        self.result_drop_carrier2 = cur.read_array_i32()?;
        self.result_drop_carrier3 = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S11_Gameplay_Simulate_Event_ChunkA0.bin`.
pub type ConfigS11GameplaySimulateEventTable = BinTable<
    (i32, i32),
    ConfigS11GameplaySimulateEventRow,
>;
/// Parses `config_S11_Gameplay_Simulate_Event_ChunkA0.bin` into a [`ConfigS11GameplaySimulateEventTable`].
pub fn parse_config_s11_gameplay_simulate_event(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS11GameplaySimulateEventTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S12Gameplay_LevelGameplay_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS12GameplayLevelGameplayRow {
    pub gameplay_types: Box<[i32]>,
    pub id: i32,
    pub group: i32,
    pub enable: i32,
    pub layer1_rule: i32,
    pub layer2_rule: i32,
    pub layer3_rule: i32,
    pub layer4_rule: i32,
    pub succeed: i32,
    pub energy_multiplier: i32,
}
impl BinRow for ConfigS12GameplayLevelGameplayRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.gameplay_types = cur.read_array_i32()?;
        self.group = cur.read_i32()?;
        self.enable = cur.read_i32()?;
        self.layer1_rule = cur.read_i32()?;
        self.layer2_rule = cur.read_i32()?;
        self.layer3_rule = cur.read_i32()?;
        self.layer4_rule = cur.read_i32()?;
        self.succeed = cur.read_i32()?;
        self.energy_multiplier = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S12Gameplay_LevelGameplay_ChunkA0.bin`.
pub type ConfigS12GameplayLevelGameplayTable = BinTable<
    i32,
    ConfigS12GameplayLevelGameplayRow,
>;
/// Parses `config_S12Gameplay_LevelGameplay_ChunkA0.bin` into a [`ConfigS12GameplayLevelGameplayTable`].
pub fn parse_config_s12_gameplay_level_gameplay(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS12GameplayLevelGameplayTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S12Gameplay_LevelModifier_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS12GameplayLevelModifierRow {
    pub modifier: Box<[i32]>,
    pub id: i32,
    pub group: i32,
    pub enable: i32,
    pub layer1_rule: i32,
    pub layer2_rule: i32,
    pub layer3_rule: i32,
    pub layer4_rule: i32,
    pub succeed: i32,
    pub energy_multiplier: i32,
}
impl BinRow for ConfigS12GameplayLevelModifierRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.modifier = cur.read_array_i32()?;
        self.group = cur.read_i32()?;
        self.enable = cur.read_i32()?;
        self.layer1_rule = cur.read_i32()?;
        self.layer2_rule = cur.read_i32()?;
        self.layer3_rule = cur.read_i32()?;
        self.layer4_rule = cur.read_i32()?;
        self.succeed = cur.read_i32()?;
        self.energy_multiplier = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S12Gameplay_LevelModifier_ChunkA0.bin`.
pub type ConfigS12GameplayLevelModifierTable = BinTable<
    i32,
    ConfigS12GameplayLevelModifierRow,
>;
/// Parses `config_S12Gameplay_LevelModifier_ChunkA0.bin` into a [`ConfigS12GameplayLevelModifierTable`].
pub fn parse_config_s12_gameplay_level_modifier(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS12GameplayLevelModifierTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S12Gameplay_rewardRune_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS12GameplayRewardRuneRow {
    pub tag: Box<[i32]>,
    pub id: i32,
    pub rarity: i32,
    pub is_key_card: i32,
    pub req: i32,
    pub weight: i32,
}
impl BinRow for ConfigS12GameplayRewardRuneRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.rarity = cur.read_i32()?;
        self.is_key_card = cur.read_i32()?;
        self.tag = cur.read_array_i32()?;
        self.req = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S12Gameplay_rewardRune_ChunkA0.bin`.
pub type ConfigS12GameplayRewardRuneTable = BinTable<
    i32,
    ConfigS12GameplayRewardRuneRow,
>;
/// Parses `config_S12Gameplay_rewardRune_ChunkA0.bin` into a [`ConfigS12GameplayRewardRuneTable`].
pub fn parse_config_s12_gameplay_reward_rune(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS12GameplayRewardRuneTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S14_Gameplay_Base_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS14GameplayBaseRow {
    pub id: i32,
}
impl BinRow for ConfigS14GameplayBaseRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S14_Gameplay_Base_ChunkA0.bin`.
pub type ConfigS14GameplayBaseTable = BinTable<i32, ConfigS14GameplayBaseRow>;
/// Parses `config_S14_Gameplay_Base_ChunkA0.bin` into a [`ConfigS14GameplayBaseTable`].
pub fn parse_config_s14_gameplay_base(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS14GameplayBaseTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S14_Gameplay_Device_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS14GameplayDeviceRow {
    pub upgrade_des: Box<str>,
    pub device_level_combo_bonus_des: Box<str>,
    pub unlock_slot_new: Box<[i32]>,
    pub id: i32,
    pub device_level: i32,
    pub upgrade_exp: i32,
    pub upgrade_gameplay_level: i32,
    pub device_level_combo_bonus: i32,
    pub max_combo_bonus: i32,
    pub unlock_slot: i32,
    pub base_range: i32,
    pub base_charge_efficiency: i32,
    pub base_charge_initcount: i32,
    pub base_charge_retaincount: i32,
    pub base_surprise_probability: i32,
}
impl ConfigS14GameplayDeviceRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.device_level = cur.read_i32()?;
        self.upgrade_exp = cur.read_i32()?;
        self.upgrade_gameplay_level = cur.read_i32()?;
        self.upgrade_des = cur.read_str()?;
        self.device_level_combo_bonus = cur.read_i32()?;
        self.max_combo_bonus = cur.read_i32()?;
        self.device_level_combo_bonus_des = cur.read_str()?;
        self.unlock_slot = cur.read_i32()?;
        self.unlock_slot_new = cur.read_array_i32()?;
        self.base_range = cur.read_i32()?;
        self.base_charge_efficiency = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.base_charge_initcount = cur.read_i32()?;
        self.base_charge_retaincount = cur.read_i32()?;
        self.base_surprise_probability = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigS14GameplayDeviceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S14_Gameplay_Device_ChunkA0.bin`.
pub type ConfigS14GameplayDeviceTable = BinTable<i32, ConfigS14GameplayDeviceRow>;
/// Parses `config_S14_Gameplay_Device_ChunkA0.bin` into a [`ConfigS14GameplayDeviceTable`].
pub fn parse_config_s14_gameplay_device(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS14GameplayDeviceTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S14_Gameplay_DeviceParts_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigS14GameplayDevicePartsRow {
    pub part_name: Box<str>,
    pub part_background: Box<str>,
    pub icon: Box<str>,
    pub part_ability_value: f64,
    pub part_ability_des: Box<str>,
    pub effect_type_value: f64,
    pub recipe: FxHashMap<i32, Box<[i32]>>,
    pub recipe_item: FxHashMap<i32, Box<[i32]>>,
    pub id: i32,
    pub equip_cost: i32,
    pub effect_type: i32,
    pub install_in_game: i32,
}
impl BinRow for ConfigS14GameplayDevicePartsRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.part_name = cur.read_str()?;
        self.part_background = cur.read_str()?;
        self.icon = cur.read_str()?;
        self.part_ability_value = cur.read_f64()?;
        self.part_ability_des = cur.read_str()?;
        self.equip_cost = cur.read_i32()?;
        self.effect_type = cur.read_i32()?;
        self.effect_type_value = cur.read_f64()?;
        self.recipe = cur.read_map_i32_array_i32()?;
        self.recipe_item = cur.read_map_i32_array_i32()?;
        self.install_in_game = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S14_Gameplay_DeviceParts_ChunkA0.bin`.
pub type ConfigS14GameplayDevicePartsTable = BinTable<
    i32,
    ConfigS14GameplayDevicePartsRow,
>;
/// Parses `config_S14_Gameplay_DeviceParts_ChunkA0.bin` into a [`ConfigS14GameplayDevicePartsTable`].
pub fn parse_config_s14_gameplay_device_parts(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS14GameplayDevicePartsTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S14_Gameplay_DevicePartsAffix_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigS14GameplayDevicePartsAffixRow {
    pub effect_type_value: f64,
    pub effect_type_des: Box<str>,
    pub id: i32,
    pub tier_type: i32,
    pub effect_type: i32,
    pub parts_rarity: i32,
}
impl BinRow for ConfigS14GameplayDevicePartsAffixRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.tier_type = cur.read_i32()?;
        self.effect_type = cur.read_i32()?;
        self.effect_type_value = cur.read_f64()?;
        self.effect_type_des = cur.read_str()?;
        self.parts_rarity = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S14_Gameplay_DevicePartsAffix_ChunkA0.bin`.
pub type ConfigS14GameplayDevicePartsAffixTable = BinTable<
    i32,
    ConfigS14GameplayDevicePartsAffixRow,
>;
/// Parses `config_S14_Gameplay_DevicePartsAffix_ChunkA0.bin` into a [`ConfigS14GameplayDevicePartsAffixTable`].
pub fn parse_config_s14_gameplay_device_parts_affix(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS14GameplayDevicePartsAffixTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S14_Gameplay_Surprise_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS14GameplaySurpriseRow {
    pub type1_extra_num: FxHashMap<i32, i32>,
    pub id: i32,
    pub gameplay_level_unlock: i32,
    pub r#type: i32,
    pub probability: i32,
    pub gameplaytype: i32,
    pub max_count: i32,
}
impl BinRow for ConfigS14GameplaySurpriseRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.gameplay_level_unlock = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.probability = cur.read_i32()?;
        self.gameplaytype = cur.read_i32()?;
        self.max_count = cur.read_i32()?;
        self.type1_extra_num = cur.read_map_i32_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S14_Gameplay_Surprise_ChunkA0.bin`.
pub type ConfigS14GameplaySurpriseTable = BinTable<i32, ConfigS14GameplaySurpriseRow>;
/// Parses `config_S14_Gameplay_Surprise_ChunkA0.bin` into a [`ConfigS14GameplaySurpriseTable`].
pub fn parse_config_s14_gameplay_surprise(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS14GameplaySurpriseTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S14_Gameplay_Theme_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS14GameplayThemeRow {
    pub theme_name: Box<str>,
    pub theme_des: Box<str>,
    pub theme_icon: Box<str>,
    pub id: i32,
    pub gameplay_level_unlock: i32,
    pub gameplaytype: i32,
}
impl BinRow for ConfigS14GameplayThemeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.gameplay_level_unlock = cur.read_i32()?;
        self.gameplaytype = cur.read_i32()?;
        self.theme_name = cur.read_str()?;
        self.theme_des = cur.read_str()?;
        self.theme_icon = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S14_Gameplay_Theme_ChunkA0.bin`.
pub type ConfigS14GameplayThemeTable = BinTable<i32, ConfigS14GameplayThemeRow>;
/// Parses `config_S14_Gameplay_Theme_ChunkA0.bin` into a [`ConfigS14GameplayThemeTable`].
pub fn parse_config_s14_gameplay_theme(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS14GameplayThemeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S15_Gameplay_Car_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS15GameplayCarRow {
    pub id: i32,
    pub map_performance_actor: i32,
    pub move_speed: i32,
    pub ai_config_id: i32,
}
impl BinRow for ConfigS15GameplayCarRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.map_performance_actor = cur.read_i32()?;
        self.move_speed = cur.read_i32()?;
        self.ai_config_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S15_Gameplay_Car_ChunkA0.bin`.
pub type ConfigS15GameplayCarTable = BinTable<i32, ConfigS15GameplayCarRow>;
/// Parses `config_S15_Gameplay_Car_ChunkA0.bin` into a [`ConfigS15GameplayCarTable`].
pub fn parse_config_s15_gameplay_car(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS15GameplayCarTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S7_treasurebox_reward_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS7TreasureboxRewardRow {
    pub icon: Box<str>,
    pub name: Box<str>,
    pub desc: Box<str>,
    pub dropid: FxHashMap<i32, Box<[i32]>>,
    pub id: i32,
    pub quality: i32,
    pub r#type: i32,
    pub interactive_id: i32,
}
impl BinRow for ConfigS7TreasureboxRewardRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.name = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.quality = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.dropid = cur.read_map_i32_array_i32()?;
        self.interactive_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S7_treasurebox_reward_ChunkA0.bin`.
pub type ConfigS7TreasureboxRewardTable = BinTable<i32, ConfigS7TreasureboxRewardRow>;
/// Parses `config_S7_treasurebox_reward_ChunkA0.bin` into a [`ConfigS7TreasureboxRewardTable`].
pub fn parse_config_s7_treasurebox_reward(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS7TreasureboxRewardTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S8_GamePlay_EggText_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS8GamePlayEggTextRow {
    pub desc: Box<str>,
    pub id: i32,
    pub quality: i32,
}
impl BinRow for ConfigS8GamePlayEggTextRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.quality = cur.read_i32()?;
        self.desc = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S8_GamePlay_EggText_ChunkA0.bin`.
pub type ConfigS8GamePlayEggTextTable = BinTable<(i32, i32), ConfigS8GamePlayEggTextRow>;
/// Parses `config_S8_GamePlay_EggText_ChunkA0.bin` into a [`ConfigS8GamePlayEggTextTable`].
pub fn parse_config_s8_game_play_egg_text(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS8GamePlayEggTextTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S8_Gameplay_ItemContent_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS8GameplayItemContentRow {
    pub gameplay_name: Box<str>,
    pub target_des: Box<str>,
    pub plot_modifier: FxHashMap<i32, i32>,
    pub limit_time: Box<[i32]>,
    pub id: i32,
    pub plot_item_id: i32,
    pub item_series_id: i32,
    pub item_type: i32,
    pub item_quality: i32,
    pub item_level: i32,
    pub gameplay_type: i32,
    pub ini_trap_id: i32,
    pub plot_modifier_num: i32,
    pub target_num: i32,
}
impl ConfigS8GameplayItemContentRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.plot_item_id = cur.read_i32()?;
        self.item_series_id = cur.read_i32()?;
        self.item_type = cur.read_i32()?;
        self.item_quality = cur.read_i32()?;
        self.item_level = cur.read_i32()?;
        self.gameplay_type = cur.read_i32()?;
        self.gameplay_name = cur.read_str()?;
        self.target_des = cur.read_str()?;
        self.ini_trap_id = cur.read_i32()?;
        self.plot_modifier = cur.read_map_i32_i32()?;
        self.plot_modifier_num = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.limit_time = cur.read_array_i32()?;
        self.target_num = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigS8GameplayItemContentRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S8_Gameplay_ItemContent_ChunkA0.bin`.
pub type ConfigS8GameplayItemContentTable = BinTable<
    i32,
    ConfigS8GameplayItemContentRow,
>;
/// Parses `config_S8_Gameplay_ItemContent_ChunkA0.bin` into a [`ConfigS8GameplayItemContentTable`].
pub fn parse_config_s8_gameplay_item_content(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS8GameplayItemContentTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_S9_gameplay_reward_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS9GameplayRewardRow {
    pub icon: Box<str>,
    pub id: i32,
    pub r#type: i32,
    pub rarity: i32,
}
impl BinRow for ConfigS9GameplayRewardRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.rarity = cur.read_i32()?;
        self.icon = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_S9_gameplay_reward_ChunkA0.bin`.
pub type ConfigS9GameplayRewardTable = BinTable<i32, ConfigS9GameplayRewardRow>;
/// Parses `config_S9_gameplay_reward_ChunkA0.bin` into a [`ConfigS9GameplayRewardTable`].
pub fn parse_config_s9_gameplay_reward(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS9GameplayRewardTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_activity_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigActivityRow {
    pub activity_type: Box<str>,
    pub bgp: Box<str>,
    pub notice_pic: Box<str>,
    pub link_id: Box<str>,
    pub id: i32,
    pub is_show: i32,
    pub show_type: i32,
}
impl BinRow for ConfigActivityRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.activity_type = cur.read_str()?;
        self.bgp = cur.read_str()?;
        self.notice_pic = cur.read_str()?;
        self.link_id = cur.read_str()?;
        self.is_show = cur.read_i32()?;
        self.show_type = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_activity_ChunkA0.bin`.
pub type ConfigActivityTable = BinTable<i32, ConfigActivityRow>;
/// Parses `config_activity_ChunkA0.bin` into a [`ConfigActivityTable`].
pub fn parse_config_activity(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigActivityTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gameplay_action_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGameplayActionRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
    pub gameplay_type: i32,
    pub limit: i32,
}
impl BinRow for ConfigGameplayActionRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.gameplay_type = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.limit = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gameplay_action_ChunkA0.bin`.
pub type ConfigGameplayActionTable = BinTable<i32, ConfigGameplayActionRow>;
/// Parses `config_gameplay_action_ChunkA0.bin` into a [`ConfigGameplayActionTable`].
pub fn parse_config_gameplay_action(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGameplayActionTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gameplay_audio_event_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGameplayAudioEventTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigGameplayAudioEventTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gameplay_audio_event_type_ChunkA0.bin`.
pub type ConfigGameplayAudioEventTypeTable = BinTable<
    i32,
    ConfigGameplayAudioEventTypeRow,
>;
/// Parses `config_gameplay_audio_event_type_ChunkA0.bin` into a [`ConfigGameplayAudioEventTypeTable`].
pub fn parse_config_gameplay_audio_event_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGameplayAudioEventTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gameplay_generate_monster_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigGameplayGenerateMonsterRow {
    pub monsters_lib: Box<[FConfigI32I32]>,
    pub added_affixs: Box<[FConfigAffix]>,
    pub group: i32,
    pub seat: i32,
    pub weight: i32,
    pub rarity: i32,
    pub monster_num: FConfigI32I32,
    pub monster_side: i32,
}
impl BinRow for ConfigGameplayGenerateMonsterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.group = cur.read_i32()?;
        self.seat = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        self.monsters_lib = FConfigI32I32::read_array(cur)?;
        self.rarity = cur.read_i32()?;
        self.monster_num = FConfigI32I32::read(cur)?;
        self.monster_side = cur.read_i32()?;
        self.added_affixs = FConfigAffix::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gameplay_generate_monster_ChunkA0.bin`.
pub type ConfigGameplayGenerateMonsterTable = BinTable<
    (i32, i32),
    ConfigGameplayGenerateMonsterRow,
>;
/// Parses `config_gameplay_generate_monster_ChunkA0.bin` into a [`ConfigGameplayGenerateMonsterTable`].
pub fn parse_config_gameplay_generate_monster(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGameplayGenerateMonsterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gameplay_point_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGameplayPointTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigGameplayPointTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gameplay_point_type_ChunkA0.bin`.
pub type ConfigGameplayPointTypeTable = BinTable<i32, ConfigGameplayPointTypeRow>;
/// Parses `config_gameplay_point_type_ChunkA0.bin` into a [`ConfigGameplayPointTypeTable`].
pub fn parse_config_gameplay_point_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGameplayPointTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gameplay_postprocess_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigGameplayPostprocessRow {
    pub res: Box<str>,
    pub param: Box<str>,
    pub start_param: f64,
    pub end_param: f64,
    pub param_time: f64,
    pub id: i32,
}
impl BinRow for ConfigGameplayPostprocessRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.res = cur.read_str()?;
        self.param = cur.read_str()?;
        self.start_param = cur.read_f64()?;
        self.end_param = cur.read_f64()?;
        self.param_time = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gameplay_postprocess_ChunkA0.bin`.
pub type ConfigGameplayPostprocessTable = BinTable<i32, ConfigGameplayPostprocessRow>;
/// Parses `config_gameplay_postprocess_ChunkA0.bin` into a [`ConfigGameplayPostprocessTable`].
pub fn parse_config_gameplay_postprocess(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGameplayPostprocessTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gameplay_report_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGameplayReportRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigGameplayReportRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gameplay_report_ChunkA0.bin`.
pub type ConfigGameplayReportTable = BinTable<i32, ConfigGameplayReportRow>;
/// Parses `config_gameplay_report_ChunkA0.bin` into a [`ConfigGameplayReportTable`].
pub fn parse_config_gameplay_report(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGameplayReportTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_gameplay_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigGameplayTypeRow {
    pub r#enum: Box<str>,
    pub match_point_rule: Box<str>,
    pub des: Box<str>,
    pub id: i32,
    pub create_rule: i32,
    pub point_num: i32,
    pub need_match_point: bool,
    pub is_ts_gameplay: bool,
}
impl BinRow for ConfigGameplayTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.create_rule = cur.read_i32()?;
        self.need_match_point = cur.read_bool()?;
        self.match_point_rule = cur.read_str()?;
        self.point_num = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.is_ts_gameplay = cur.read_bool()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_gameplay_type_ChunkA0.bin`.
pub type ConfigGameplayTypeTable = BinTable<i32, ConfigGameplayTypeRow>;
/// Parses `config_gameplay_type_ChunkA0.bin` into a [`ConfigGameplayTypeTable`].
pub fn parse_config_gameplay_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigGameplayTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_s16_gameplay_config_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS16GameplayConfigRow {
    pub r#type: Box<str>,
    pub param: Box<str>,
    pub desc: Box<str>,
    pub icon: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigS16GameplayConfigRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_str()?;
        self.param = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.icon = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_s16_gameplay_config_ChunkA0.bin`.
pub type ConfigS16GameplayConfigTable = BinTable<i32, ConfigS16GameplayConfigRow>;
/// Parses `config_s16_gameplay_config_ChunkA0.bin` into a [`ConfigS16GameplayConfigTable`].
pub fn parse_config_s16_gameplay_config(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS16GameplayConfigTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_s16_gameplay_monster_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS16GameplayMonsterRow {
    pub name: Box<str>,
    pub target: Box<[i32]>,
    pub rating: Box<[i32]>,
    pub normal_npc_id: Box<[i32]>,
    pub magic_npc_id: Box<[i32]>,
    pub rare_npc_id: Box<[i32]>,
    pub boss_npc_id: Box<[i32]>,
    pub desc: Box<str>,
    pub pic_room: Box<str>,
    pub pic_tips: Box<str>,
    pub id: i32,
    pub species: i32,
    pub weight: i32,
    pub normal_drop_carrier: i32,
    pub open_trigger_drop_carrier: i32,
    pub normal_npc_num: i32,
    pub magic_npc_num: i32,
    pub rare_npc_num: i32,
    pub boss_npc_num: i32,
}
impl ConfigS16GameplayMonsterRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.species = cur.read_i32()?;
        self.target = cur.read_array_i32()?;
        self.weight = cur.read_i32()?;
        self.rating = cur.read_array_i32()?;
        self.normal_drop_carrier = cur.read_i32()?;
        self.open_trigger_drop_carrier = cur.read_i32()?;
        self.normal_npc_id = cur.read_array_i32()?;
        self.normal_npc_num = cur.read_i32()?;
        self.magic_npc_id = cur.read_array_i32()?;
        self.magic_npc_num = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.rare_npc_id = cur.read_array_i32()?;
        self.rare_npc_num = cur.read_i32()?;
        self.boss_npc_id = cur.read_array_i32()?;
        self.boss_npc_num = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.pic_room = cur.read_str()?;
        self.pic_tips = cur.read_str()?;
        Ok(())
    }
}
impl BinRow for ConfigS16GameplayMonsterRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_s16_gameplay_monster_ChunkA0.bin`.
pub type ConfigS16GameplayMonsterTable = BinTable<i32, ConfigS16GameplayMonsterRow>;
/// Parses `config_s16_gameplay_monster_ChunkA0.bin` into a [`ConfigS16GameplayMonsterTable`].
pub fn parse_config_s16_gameplay_monster(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS16GameplayMonsterTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_s5_gooddream_buff_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS5GooddreamBuffRow {
    pub name: Box<str>,
    pub id: i32,
    pub r#type: i32,
    pub weight: i32,
    pub count: i32,
    pub buff_id: i32,
}
impl BinRow for ConfigS5GooddreamBuffRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        self.count = cur.read_i32()?;
        self.buff_id = cur.read_i32()?;
        self.name = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_s5_gooddream_buff_ChunkA0.bin`.
pub type ConfigS5GooddreamBuffTable = BinTable<i32, ConfigS5GooddreamBuffRow>;
/// Parses `config_s5_gooddream_buff_ChunkA0.bin` into a [`ConfigS5GooddreamBuffTable`].
pub fn parse_config_s5_gooddream_buff(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS5GooddreamBuffTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_s5_gooddream_reward_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigS5GooddreamRewardRow {
    pub icon: Box<str>,
    pub name: Box<str>,
    pub desc: Box<str>,
    pub dropid: FxHashMap<i32, Box<[i32]>>,
    pub id: i32,
    pub quality: i32,
    pub r#type: i32,
    pub interactive_id: i32,
    pub trap_id: i32,
}
impl BinRow for ConfigS5GooddreamRewardRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.name = cur.read_str()?;
        self.desc = cur.read_str()?;
        self.quality = cur.read_i32()?;
        self.r#type = cur.read_i32()?;
        self.dropid = cur.read_map_i32_array_i32()?;
        self.interactive_id = cur.read_i32()?;
        self.trap_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_s5_gooddream_reward_ChunkA0.bin`.
pub type ConfigS5GooddreamRewardTable = BinTable<i32, ConfigS5GooddreamRewardRow>;
/// Parses `config_s5_gooddream_reward_ChunkA0.bin` into a [`ConfigS5GooddreamRewardTable`].
pub fn parse_config_s5_gooddream_reward(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS5GooddreamRewardTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_s6_gameplay_levelbattle_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigS6GameplayLevelbattleRow {
    pub gameplayname: Box<str>,
    pub targetdes: Box<str>,
    pub targetintro: Box<str>,
    pub difficultyintro: Box<str>,
    pub intro_img: Box<str>,
    pub startareapos: VecDim,
    pub startarearotate: VecDim,
    pub battleareapos: VecDim,
    pub battlearearotate: VecDim,
    pub bossareapos: VecDim,
    pub bossarearotate: VecDim,
    pub rewardareapos: VecDim,
    pub rewardarearotate: VecDim,
    pub deducttime: Box<[i32]>,
    pub id: i32,
    pub battletype: i32,
    pub gameplaytype: i32,
    pub difficulty: i32,
    pub layerlowerlimit: i32,
    pub layerupperlimit: i32,
    pub openlevel: i32,
    pub levelid: i32,
    pub limittime: i32,
    pub restoressanity: i32,
    pub speciallevelrate: i32,
    pub isstartposrandom: bool,
}
impl ConfigS6GameplayLevelbattleRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.battletype = cur.read_i32()?;
        self.gameplaytype = cur.read_i32()?;
        self.gameplayname = cur.read_str()?;
        self.difficulty = cur.read_i32()?;
        self.targetdes = cur.read_str()?;
        self.targetintro = cur.read_str()?;
        self.difficultyintro = cur.read_str()?;
        self.intro_img = cur.read_str()?;
        self.layerlowerlimit = cur.read_i32()?;
        self.layerupperlimit = cur.read_i32()?;
        self.openlevel = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.levelid = cur.read_i32()?;
        self.isstartposrandom = cur.read_bool()?;
        self.startareapos = cur.read_vec_dim()?;
        self.startarearotate = cur.read_vec_dim()?;
        self.battleareapos = cur.read_vec_dim()?;
        self.battlearearotate = cur.read_vec_dim()?;
        self.bossareapos = cur.read_vec_dim()?;
        self.bossarearotate = cur.read_vec_dim()?;
        self.rewardareapos = cur.read_vec_dim()?;
        self.rewardarearotate = cur.read_vec_dim()?;
        self.limittime = cur.read_i32()?;
        self.deducttime = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_2(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.restoressanity = cur.read_i32()?;
        self.speciallevelrate = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigS6GameplayLevelbattleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        self.read_fields_2(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_s6_gameplay_levelbattle_ChunkA0.bin`.
pub type ConfigS6GameplayLevelbattleTable = BinTable<
    i32,
    ConfigS6GameplayLevelbattleRow,
>;
/// Parses `config_s6_gameplay_levelbattle_ChunkA0.bin` into a [`ConfigS6GameplayLevelbattleTable`].
pub fn parse_config_s6_gameplay_levelbattle(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigS6GameplayLevelbattleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
#[inline(never)]
fn verify_season_chunk_0(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_S11_Gameplay_Bounty" => {
            Some(parse_config_s11_gameplay_bounty(buf, scratch).map(|t| t.len()))
        }
        "config_S11_Gameplay_BountyCondtion" => {
            Some(
                parse_config_s11_gameplay_bounty_condtion(buf, scratch).map(|t| t.len()),
            )
        }
        "config_S11_Gameplay_BountyLevel" => {
            Some(parse_config_s11_gameplay_bounty_level(buf, scratch).map(|t| t.len()))
        }
        "config_S11_Gameplay_BountyReward" => {
            Some(parse_config_s11_gameplay_bounty_reward(buf, scratch).map(|t| t.len()))
        }
        "config_S11_Gameplay_Coffer" => {
            Some(parse_config_s11_gameplay_coffer(buf, scratch).map(|t| t.len()))
        }
        "config_S11_Gameplay_CofferBattle" => {
            Some(parse_config_s11_gameplay_coffer_battle(buf, scratch).map(|t| t.len()))
        }
        "config_S11_Gameplay_FinalGame" => {
            Some(parse_config_s11_gameplay_final_game(buf, scratch).map(|t| t.len()))
        }
        "config_S11_Gameplay_FinalGameRaise" => {
            Some(
                parse_config_s11_gameplay_final_game_raise(buf, scratch).map(|t| t.len()),
            )
        }
        "config_S11_Gameplay_RandomEvent" => {
            Some(parse_config_s11_gameplay_random_event(buf, scratch).map(|t| t.len()))
        }
        "config_S11_Gameplay_Simulate_Base" => {
            Some(parse_config_s11_gameplay_simulate_base(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_season_chunk_1(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_S11_Gameplay_Simulate_Event" => {
            Some(parse_config_s11_gameplay_simulate_event(buf, scratch).map(|t| t.len()))
        }
        "config_S12Gameplay_LevelGameplay" => {
            Some(parse_config_s12_gameplay_level_gameplay(buf, scratch).map(|t| t.len()))
        }
        "config_S12Gameplay_LevelModifier" => {
            Some(parse_config_s12_gameplay_level_modifier(buf, scratch).map(|t| t.len()))
        }
        "config_S12Gameplay_rewardRune" => {
            Some(parse_config_s12_gameplay_reward_rune(buf, scratch).map(|t| t.len()))
        }
        "config_S14_Gameplay_Base" => {
            Some(parse_config_s14_gameplay_base(buf, scratch).map(|t| t.len()))
        }
        "config_S14_Gameplay_Device" => {
            Some(parse_config_s14_gameplay_device(buf, scratch).map(|t| t.len()))
        }
        "config_S14_Gameplay_DeviceParts" => {
            Some(parse_config_s14_gameplay_device_parts(buf, scratch).map(|t| t.len()))
        }
        "config_S14_Gameplay_DevicePartsAffix" => {
            Some(
                parse_config_s14_gameplay_device_parts_affix(buf, scratch)
                    .map(|t| t.len()),
            )
        }
        "config_S14_Gameplay_Surprise" => {
            Some(parse_config_s14_gameplay_surprise(buf, scratch).map(|t| t.len()))
        }
        "config_S14_Gameplay_Theme" => {
            Some(parse_config_s14_gameplay_theme(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_season_chunk_2(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_S15_Gameplay_Car" => {
            Some(parse_config_s15_gameplay_car(buf, scratch).map(|t| t.len()))
        }
        "config_S7_treasurebox_reward" => {
            Some(parse_config_s7_treasurebox_reward(buf, scratch).map(|t| t.len()))
        }
        "config_S8_GamePlay_EggText" => {
            Some(parse_config_s8_game_play_egg_text(buf, scratch).map(|t| t.len()))
        }
        "config_S8_Gameplay_ItemContent" => {
            Some(parse_config_s8_gameplay_item_content(buf, scratch).map(|t| t.len()))
        }
        "config_S9_gameplay_reward" => {
            Some(parse_config_s9_gameplay_reward(buf, scratch).map(|t| t.len()))
        }
        "config_activity" => Some(parse_config_activity(buf, scratch).map(|t| t.len())),
        "config_gameplay_action" => {
            Some(parse_config_gameplay_action(buf, scratch).map(|t| t.len()))
        }
        "config_gameplay_audio_event_type" => {
            Some(parse_config_gameplay_audio_event_type(buf, scratch).map(|t| t.len()))
        }
        "config_gameplay_generate_monster" => {
            Some(parse_config_gameplay_generate_monster(buf, scratch).map(|t| t.len()))
        }
        "config_gameplay_point_type" => {
            Some(parse_config_gameplay_point_type(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_season_chunk_3(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_gameplay_postprocess" => {
            Some(parse_config_gameplay_postprocess(buf, scratch).map(|t| t.len()))
        }
        "config_gameplay_report" => {
            Some(parse_config_gameplay_report(buf, scratch).map(|t| t.len()))
        }
        "config_gameplay_type" => {
            Some(parse_config_gameplay_type(buf, scratch).map(|t| t.len()))
        }
        "config_s16_gameplay_config" => {
            Some(parse_config_s16_gameplay_config(buf, scratch).map(|t| t.len()))
        }
        "config_s16_gameplay_monster" => {
            Some(parse_config_s16_gameplay_monster(buf, scratch).map(|t| t.len()))
        }
        "config_s5_gooddream_buff" => {
            Some(parse_config_s5_gooddream_buff(buf, scratch).map(|t| t.len()))
        }
        "config_s5_gooddream_reward" => {
            Some(parse_config_s5_gooddream_reward(buf, scratch).map(|t| t.len()))
        }
        "config_s6_gameplay_levelbattle" => {
            Some(parse_config_s6_gameplay_levelbattle(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
pub(crate) fn verify_season_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
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

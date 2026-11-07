//! Auto-generated `LuaTable/DataCfg` types and parsers for the `forge` domain.
use crate::luac::datacfg::{
    DataCfgRow, DataCfgScratch, DataTable, EvalContext, EvalVal, FromEvalVal,
    parse_luatable_datacfg,
};
use crate::luac::error::LuacError;
use rustc_hash::FxHashMap;
/// Entry for `config_cardrecipe.ItemGoldId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardrecipeItemGoldIdEntry {
    pub item_gold_id: u32,
    pub weight: u8,
}
impl FromEvalVal for CardrecipeItemGoldIdEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (item_gold_id, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { item_gold_id, weight })
    }
}
/// Entry for `config_cardrecipe.RecipeNum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardrecipeRecipeNumEntry {
    pub count: u16,
    pub weight: u8,
}
impl FromEvalVal for CardrecipeRecipeNumEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (count, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { count, weight })
    }
}
/// Entry for `config_cardrecipe.RecipeRandomWeight`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardrecipeRecipeRandomWeightEntry {
    pub group_id: u8,
    pub weight: u8,
}
impl FromEvalVal for CardrecipeRecipeRandomWeightEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (group_id, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { group_id, weight })
    }
}
/// Entry for `config_cardrecipe.RecipeUnique`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardrecipeRecipeUniqueEntry {
    pub recipe_id: u32,
    pub target_id: u32,
    pub weight: u8,
}
impl FromEvalVal for CardrecipeRecipeUniqueEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (recipe_id, (target_id, weight)) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self {
            recipe_id,
            target_id,
            weight,
        })
    }
}
/// Entry for `config_cardrecipe.RewardGroupId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardrecipeRewardGroupIdEntry {
    pub reward_group_id: u32,
    pub weight: u8,
}
impl FromEvalVal for CardrecipeRewardGroupIdEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (reward_group_id, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { reward_group_id, weight })
    }
}
/// Entry for `config_cardrecipe.SkillType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardrecipeSkillType {
    pub skill_type: u8,
    pub weight: u8,
}
impl FromEvalVal for CardrecipeSkillType {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (skill_type, weight) = ctx.decode_kv_pair(val, table, field)?;
        Ok(Self { skill_type, weight })
    }
}
/// Row record for `config_cardrecipe.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigCardrecipeRow {
    pub classification: Box<str>,
    pub classification_block: Box<str>,
    pub item_gold_id: Box<[CardrecipeItemGoldIdEntry]>,
    pub item_skill_classification: Box<str>,
    pub item_skill_classification_block: Box<str>,
    pub random_icon: Box<str>,
    pub recipe_num: Box<[CardrecipeRecipeNumEntry]>,
    pub recipe_random_weight: Box<[CardrecipeRecipeRandomWeightEntry]>,
    pub recipe_unique: Box<[CardrecipeRecipeUniqueEntry]>,
    pub reward_group_id: Box<[CardrecipeRewardGroupIdEntry]>,
    pub slot_num: FxHashMap<u8, u8>,
    pub tips_summary: Box<str>,
    pub unique_item_rarity_level: Box<str>,
    pub _tips_summary: Box<str>,
    pub id: i16,
    pub args: u8,
    pub is_random: u8,
    pub item_skill_level: u8,
    pub rarity: u8,
    pub recipe_random_type: u8,
    pub skill_type: CardrecipeSkillType,
    pub stack: u8,
    pub tips_specified: u8,
    pub drop_general_id: [u8; 0],
}
impl ConfigCardrecipeRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"Args" => self.args = ctx.decode_field(val, Self::TABLE_NAME, "Args")?,
            b"Classification" => {
                self.classification = ctx
                    .decode_field(val, Self::TABLE_NAME, "Classification")?;
            }
            b"ClassificationBlock" => {
                self.classification_block = ctx
                    .decode_field(val, Self::TABLE_NAME, "ClassificationBlock")?;
            }
            b"DropGeneralId" => {
                self.drop_general_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "DropGeneralId")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsRandom" => {
                self.is_random = ctx.decode_field(val, Self::TABLE_NAME, "IsRandom")?;
            }
            b"ItemGoldId" => {
                self.item_gold_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemGoldId")?;
            }
            b"ItemSkillClassification" => {
                self.item_skill_classification = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemSkillClassification")?;
            }
            b"ItemSkillClassificationBlock" => {
                self.item_skill_classification_block = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemSkillClassificationBlock")?;
            }
            b"ItemSkillLevel" => {
                self.item_skill_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemSkillLevel")?;
            }
            b"RandomIcon" => {
                self.random_icon = ctx.decode_field(val, Self::TABLE_NAME, "RandomIcon")?;
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
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
            b"RecipeNum" => {
                self.recipe_num = ctx.decode_field(val, Self::TABLE_NAME, "RecipeNum")?;
            }
            b"RecipeRandomType" => {
                self.recipe_random_type = ctx
                    .decode_field(val, Self::TABLE_NAME, "RecipeRandomType")?;
            }
            b"RecipeRandomWeight" => {
                self.recipe_random_weight = ctx
                    .decode_field(val, Self::TABLE_NAME, "RecipeRandomWeight")?;
            }
            b"RecipeUnique" => {
                self.recipe_unique = ctx
                    .decode_field(val, Self::TABLE_NAME, "RecipeUnique")?;
            }
            b"RewardGroupId" => {
                self.reward_group_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "RewardGroupId")?;
            }
            b"SkillType" => {
                self.skill_type = ctx.decode_field(val, Self::TABLE_NAME, "SkillType")?;
            }
            b"SlotNum" => {
                self.slot_num = ctx.decode_field(val, Self::TABLE_NAME, "SlotNum")?;
            }
            b"Stack" => self.stack = ctx.decode_field(val, Self::TABLE_NAME, "Stack")?,
            b"TipsSpecified" => {
                self.tips_specified = ctx
                    .decode_field(val, Self::TABLE_NAME, "TipsSpecified")?;
            }
            b"TipsSummary" => {
                self.tips_summary = ctx
                    .decode_field(val, Self::TABLE_NAME, "TipsSummary")?;
            }
            b"UniqueItemRarityLevel" => {
                self.unique_item_rarity_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "UniqueItemRarityLevel")?;
            }
            b"_TipsSummary" => {
                self._tips_summary = ctx
                    .decode_field(val, Self::TABLE_NAME, "_TipsSummary")?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigCardrecipeRow {
    const TABLE_NAME: &'static str = "config_cardrecipe";
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
/// Typed `DataTable` for `config_cardrecipe.out`.
pub type ConfigCardrecipeTable = DataTable<u16, ConfigCardrecipeRow>;
/// Parses `config_cardrecipe.out` into a [`ConfigCardrecipeTable`].
pub fn parse_config_cardrecipe(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigCardrecipeTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_forge_dream.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigForgeDreamRow {
    pub consume: FxHashMap<u32, u8>,
    pub item_level: Box<[u16]>,
    pub type4: Box<[u8]>,
    pub id: i8,
    pub time: u8,
    pub type_class: u8,
    pub consume_special: [u8; 0],
}
impl DataCfgRow for ConfigForgeDreamRow {
    const TABLE_NAME: &'static str = "config_forge_dream";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"Consume" => {
                self.consume = ctx.decode_field(val, Self::TABLE_NAME, "Consume")?;
            }
            b"ConsumeSpecial" => {
                self.consume_special = ctx
                    .decode_field(val, Self::TABLE_NAME, "ConsumeSpecial")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?;
            }
            b"Time" => self.time = ctx.decode_field(val, Self::TABLE_NAME, "Time")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            b"TypeClass" => {
                self.type_class = ctx.decode_field(val, Self::TABLE_NAME, "TypeClass")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_forge_dream.out`.
pub type ConfigForgeDreamTable = DataTable<u8, ConfigForgeDreamRow>;
/// Parses `config_forge_dream.out` into a [`ConfigForgeDreamTable`].
pub fn parse_config_forge_dream(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigForgeDreamTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_forge_dream_material.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigForgeDreamMaterialRow {
    pub affix_group: u32,
    pub material_id: i32,
    pub reward_group_id: u32,
    pub source_id: u32,
    pub consume_num: u8,
}
impl DataCfgRow for ConfigForgeDreamMaterialRow {
    const TABLE_NAME: &'static str = "config_forge_dream_material";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"AffixGroup" => {
                self.affix_group = ctx.decode_field(val, Self::TABLE_NAME, "AffixGroup")?;
            }
            b"ConsumeNum" => {
                self.consume_num = ctx.decode_field(val, Self::TABLE_NAME, "ConsumeNum")?;
            }
            b"MaterialId" => {
                self.material_id = ctx.decode_field(val, Self::TABLE_NAME, "MaterialId")?;
            }
            b"RewardGroupId" => {
                self.reward_group_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "RewardGroupId")?;
            }
            b"SourceId" => {
                self.source_id = ctx.decode_field(val, Self::TABLE_NAME, "SourceId")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_forge_dream_material.out`.
pub type ConfigForgeDreamMaterialTable = DataTable<u32, ConfigForgeDreamMaterialRow>;
/// Parses `config_forge_dream_material.out` into a [`ConfigForgeDreamMaterialTable`].
pub fn parse_config_forge_dream_material(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigForgeDreamMaterialTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Entry for `config_forge_precise_config.TargetAffixTier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ForgePreciseConfigTargetAffixTierEntry {
    pub prob_per_myriad: u16,
    pub tier: u8,
    pub prob_type: u8,
}
impl FromEvalVal for ForgePreciseConfigTargetAffixTierEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (tier, (prob_type, prob_per_myriad)) = ctx
            .decode_kv_pair(val, table, field)?;
        Ok(Self {
            prob_per_myriad,
            tier,
            prob_type,
        })
    }
}
/// Row record for `config_forge_precise_config.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigForgePreciseConfigRow {
    pub desc: Box<str>,
    pub forge_affix_library: Box<[u8]>,
    pub forge_consume: FxHashMap<u32, u16>,
    pub item_level: Box<[u8]>,
    pub num: Box<[u8]>,
    pub rarity: Box<[u8]>,
    pub target_affix_tier: Box<[ForgePreciseConfigTargetAffixTierEntry]>,
    pub type4: Box<[u8]>,
    pub _desc: Box<str>,
    pub ashes_id: u32,
    pub fake_suceess_t1: u16,
    pub forge_type: u8,
    pub id: i8,
    pub min: u8,
    pub target_affix_library: u8,
    pub item_gold_rarity: [u8; 0],
    pub legendary_slot_num: [u8; 0],
    pub source_tag: [u8; 0],
    pub target_tag: [u8; 0],
}
impl ConfigForgePreciseConfigRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AshesID" => {
                self.ashes_id = ctx.decode_field(val, Self::TABLE_NAME, "AshesID")?;
            }
            b"Desc" => self.desc = ctx.decode_field(val, Self::TABLE_NAME, "Desc")?,
            b"FakeSuceessT1" => {
                self.fake_suceess_t1 = ctx
                    .decode_field(val, Self::TABLE_NAME, "FakeSuceessT1")?;
            }
            b"ForgeAffixLibrary" => {
                self.forge_affix_library = ctx
                    .decode_field(val, Self::TABLE_NAME, "ForgeAffixLibrary")?;
            }
            b"ForgeConsume" => {
                self.forge_consume = ctx
                    .decode_field(val, Self::TABLE_NAME, "ForgeConsume")?;
            }
            b"ForgeType" => {
                self.forge_type = ctx.decode_field(val, Self::TABLE_NAME, "ForgeType")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemGoldRarity" => {
                self.item_gold_rarity = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemGoldRarity")?;
            }
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?;
            }
            b"LegendarySlotNum" => {
                self.legendary_slot_num = ctx
                    .decode_field(val, Self::TABLE_NAME, "LegendarySlotNum")?;
            }
            b"Min" => self.min = ctx.decode_field(val, Self::TABLE_NAME, "Min")?,
            b"Num" => self.num = ctx.decode_field(val, Self::TABLE_NAME, "Num")?,
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
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"SourceTag" => {
                self.source_tag = ctx.decode_field(val, Self::TABLE_NAME, "SourceTag")?;
            }
            b"TargetAffixLibrary" => {
                self.target_affix_library = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetAffixLibrary")?;
            }
            b"TargetAffixTier" => {
                self.target_affix_tier = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetAffixTier")?;
            }
            b"TargetTag" => {
                self.target_tag = ctx.decode_field(val, Self::TABLE_NAME, "TargetTag")?;
            }
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            b"_Desc" => self._desc = ctx.decode_field(val, Self::TABLE_NAME, "_Desc")?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigForgePreciseConfigRow {
    const TABLE_NAME: &'static str = "config_forge_precise_config";
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
/// Typed `DataTable` for `config_forge_precise_config.out`.
pub type ConfigForgePreciseConfigTable = DataTable<u8, ConfigForgePreciseConfigRow>;
/// Parses `config_forge_precise_config.out` into a [`ConfigForgePreciseConfigTable`].
pub fn parse_config_forge_precise_config(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigForgePreciseConfigTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Entry for `config_forge_s13_organ_config.TargetAffixTier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ForgeS13OrganConfigTargetAffixTierEntry {
    pub prob_per_myriad: u16,
    pub tier: u8,
    pub prob_type: u8,
}
impl FromEvalVal for ForgeS13OrganConfigTargetAffixTierEntry {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let (tier, (prob_type, prob_per_myriad)) = ctx
            .decode_kv_pair(val, table, field)?;
        Ok(Self {
            prob_per_myriad,
            tier,
            prob_type,
        })
    }
}
/// Row record for `config_forge_s13_organ_config.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigForgeS13OrganConfigRow {
    pub item_level: Box<[u8]>,
    pub target_affix_tier: Box<[ForgeS13OrganConfigTargetAffixTierEntry]>,
    pub organ_id: u32,
    pub id: i16,
    pub organ_consume_num: u8,
    pub organ_type: u8,
    pub forge_consume: [u8; 0],
    pub success_rate_up: [u8; 0],
}
impl DataCfgRow for ConfigForgeS13OrganConfigRow {
    const TABLE_NAME: &'static str = "config_forge_s13_organ_config";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"ForgeConsume" => {
                self.forge_consume = ctx
                    .decode_field(val, Self::TABLE_NAME, "ForgeConsume")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?;
            }
            b"OrganConsumeNum" => {
                self.organ_consume_num = ctx
                    .decode_field(val, Self::TABLE_NAME, "OrganConsumeNum")?;
            }
            b"OrganId" => {
                self.organ_id = ctx.decode_field(val, Self::TABLE_NAME, "OrganId")?;
            }
            b"OrganType" => {
                self.organ_type = ctx.decode_field(val, Self::TABLE_NAME, "OrganType")?;
            }
            b"SuccessRateUp" => {
                self.success_rate_up = ctx
                    .decode_field(val, Self::TABLE_NAME, "SuccessRateUp")?;
            }
            b"TargetAffixTier" => {
                self.target_affix_tier = ctx
                    .decode_field(val, Self::TABLE_NAME, "TargetAffixTier")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_forge_s13_organ_config.out`.
pub type ConfigForgeS13OrganConfigTable = DataTable<u16, ConfigForgeS13OrganConfigRow>;
/// Parses `config_forge_s13_organ_config.out` into a [`ConfigForgeS13OrganConfigTable`].
pub fn parse_config_forge_s13_organ_config(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigForgeS13OrganConfigTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_forge_upgarde_config.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigForgeUpgardeConfigRow {
    pub affix_library: Box<[u8]>,
    pub durability_consume: Box<[u8]>,
    pub durability_interval: Box<[u16]>,
    pub forge_consume: FxHashMap<u32, u16>,
    pub item_level: Box<[u8]>,
    pub rarity: Box<[u8]>,
    pub type4: Box<[u8]>,
    pub current_tier: u8,
    pub durability_consume_type: u8,
    pub id: i8,
    pub remove_rule: u8,
    pub success_rate: u8,
    pub upgraded_tier: u8,
    pub item_gold_rarity: [u8; 0],
    pub legendary_slot_num: [u8; 0],
}
impl ConfigForgeUpgardeConfigRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"AffixLibrary" => {
                self.affix_library = ctx
                    .decode_field(val, Self::TABLE_NAME, "AffixLibrary")?;
            }
            b"CurrentTier" => {
                self.current_tier = ctx
                    .decode_field(val, Self::TABLE_NAME, "CurrentTier")?;
            }
            b"DurabilityConsume" => {
                self.durability_consume = ctx
                    .decode_field(val, Self::TABLE_NAME, "DurabilityConsume")?;
            }
            b"DurabilityConsumeType" => {
                self.durability_consume_type = ctx
                    .decode_field(val, Self::TABLE_NAME, "DurabilityConsumeType")?;
            }
            b"DurabilityInterval" => {
                self.durability_interval = ctx
                    .decode_field(val, Self::TABLE_NAME, "DurabilityInterval")?;
            }
            b"ForgeConsume" => {
                self.forge_consume = ctx
                    .decode_field(val, Self::TABLE_NAME, "ForgeConsume")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"ItemGoldRarity" => {
                self.item_gold_rarity = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemGoldRarity")?;
            }
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?;
            }
            b"LegendarySlotNum" => {
                self.legendary_slot_num = ctx
                    .decode_field(val, Self::TABLE_NAME, "LegendarySlotNum")?;
            }
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"RemoveRule" => {
                self.remove_rule = ctx.decode_field(val, Self::TABLE_NAME, "RemoveRule")?;
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
            b"SuccessRate" => {
                self.success_rate = ctx
                    .decode_field(val, Self::TABLE_NAME, "SuccessRate")?;
            }
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            b"UpgradedTier" => {
                self.upgraded_tier = ctx
                    .decode_field(val, Self::TABLE_NAME, "UpgradedTier")?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigForgeUpgardeConfigRow {
    const TABLE_NAME: &'static str = "config_forge_upgarde_config";
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
/// Typed `DataTable` for `config_forge_upgarde_config.out`.
pub type ConfigForgeUpgardeConfigTable = DataTable<u8, ConfigForgeUpgardeConfigRow>;
/// Parses `config_forge_upgarde_config.out` into a [`ConfigForgeUpgardeConfigTable`].
pub fn parse_config_forge_upgarde_config(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigForgeUpgardeConfigTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_recipe_list.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRecipeListRow {
    pub des_in: Box<str>,
    pub des_out: Box<str>,
    pub item_out: Box<[u32]>,
    pub recipe_new_id: Box<[u16]>,
    pub _des_in: Box<str>,
    pub _des_out: Box<str>,
    pub order: u16,
    pub bag_type: u8,
    pub id: i8,
    pub is_hide: u8,
}
impl DataCfgRow for ConfigRecipeListRow {
    const TABLE_NAME: &'static str = "config_recipe_list";
    #[inline(never)]
    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError> {
        match field {
            b"BagType" => {
                self.bag_type = ctx.decode_field(val, Self::TABLE_NAME, "BagType")?;
            }
            b"DesIn" => self.des_in = ctx.decode_field(val, Self::TABLE_NAME, "DesIn")?,
            b"DesOut" => {
                self.des_out = ctx.decode_field(val, Self::TABLE_NAME, "DesOut")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsHide" => {
                self.is_hide = ctx.decode_field(val, Self::TABLE_NAME, "IsHide")?;
            }
            b"ItemOut" => {
                self.item_out = ctx.decode_field(val, Self::TABLE_NAME, "ItemOut")?;
            }
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"RecipeNewId" => {
                self.recipe_new_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "RecipeNewId")?;
            }
            b"_DesIn" => {
                self._des_in = ctx.decode_field(val, Self::TABLE_NAME, "_DesIn")?;
            }
            b"_DesOut" => {
                self._des_out = ctx.decode_field(val, Self::TABLE_NAME, "_DesOut")?;
            }
            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
        }
        Ok(())
    }
}
/// Typed `DataTable` for `config_recipe_list.out`.
pub type ConfigRecipeListTable = DataTable<u8, ConfigRecipeListRow>;
/// Parses `config_recipe_list.out` into a [`ConfigRecipeListTable`].
pub fn parse_config_recipe_list(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigRecipeListTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
/// Row record for `config_recipe_new.out`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigRecipeNewRow {
    pub des: Box<str>,
    pub des_in: Box<str>,
    pub des_out: Box<str>,
    pub hero_item_enhance_level: Box<[u8]>,
    pub s13_equip_base_tier: FxHashMap<u8, u8>,
    pub skill_max_tier: Box<[u8]>,
    pub talent_point: FxHashMap<u8, u8>,
    pub _des_in: Box<str>,
    pub _des_out: Box<str>,
    pub item_id: u32,
    pub reward_group_id: u32,
    pub id: i16,
    pub item_max_level: u16,
    pub order: u16,
    pub type2: u16,
    pub base_talent_recipe_id: u8,
    pub is_bind: u8,
    pub is_corrosion: u8,
    pub is_hide: u8,
    pub is_precious: u8,
    pub item_level: u8,
    pub legendary_slot_num: u8,
    pub num: u8,
    pub rarity: u8,
    pub rarity_level: u8,
    pub slot_recipe_id: u8,
    pub type1: u8,
    pub type3: u8,
    pub type4: u8,
}
impl ConfigRecipeNewRow {
    #[inline(never)]
    fn set_field_0(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<bool, LuacError> {
        match field {
            b"BaseTalentRecipeId" => {
                self.base_talent_recipe_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "BaseTalentRecipeId")?;
            }
            b"Des" => self.des = ctx.decode_field(val, Self::TABLE_NAME, "Des")?,
            b"DesIn" => self.des_in = ctx.decode_field(val, Self::TABLE_NAME, "DesIn")?,
            b"DesOut" => {
                self.des_out = ctx.decode_field(val, Self::TABLE_NAME, "DesOut")?;
            }
            b"HeroItemEnhanceLevel" => {
                self.hero_item_enhance_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "HeroItemEnhanceLevel")?;
            }
            b"Id" => self.id = ctx.decode_field(val, Self::TABLE_NAME, "Id")?,
            b"IsBind" => {
                self.is_bind = ctx.decode_field(val, Self::TABLE_NAME, "IsBind")?;
            }
            b"IsCorrosion" => {
                self.is_corrosion = ctx
                    .decode_field(val, Self::TABLE_NAME, "IsCorrosion")?;
            }
            b"IsHide" => {
                self.is_hide = ctx.decode_field(val, Self::TABLE_NAME, "IsHide")?;
            }
            b"IsPrecious" => {
                self.is_precious = ctx.decode_field(val, Self::TABLE_NAME, "IsPrecious")?;
            }
            b"ItemId" => {
                self.item_id = ctx.decode_field(val, Self::TABLE_NAME, "ItemId")?;
            }
            b"ItemLevel" => {
                self.item_level = ctx.decode_field(val, Self::TABLE_NAME, "ItemLevel")?;
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
            b"ItemMaxLevel" => {
                self.item_max_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "ItemMaxLevel")?;
            }
            b"LegendarySlotNum" => {
                self.legendary_slot_num = ctx
                    .decode_field(val, Self::TABLE_NAME, "LegendarySlotNum")?;
            }
            b"Num" => self.num = ctx.decode_field(val, Self::TABLE_NAME, "Num")?,
            b"Order" => self.order = ctx.decode_field(val, Self::TABLE_NAME, "Order")?,
            b"Rarity" => self.rarity = ctx.decode_field(val, Self::TABLE_NAME, "Rarity")?,
            b"RarityLevel" => {
                self.rarity_level = ctx
                    .decode_field(val, Self::TABLE_NAME, "RarityLevel")?;
            }
            b"RewardGroupId" => {
                self.reward_group_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "RewardGroupId")?;
            }
            b"S13EquipBaseTier" => {
                self.s13_equip_base_tier = ctx
                    .decode_field(val, Self::TABLE_NAME, "S13EquipBaseTier")?;
            }
            b"SkillMaxTier" => {
                self.skill_max_tier = ctx
                    .decode_field(val, Self::TABLE_NAME, "SkillMaxTier")?;
            }
            b"SlotRecipeId" => {
                self.slot_recipe_id = ctx
                    .decode_field(val, Self::TABLE_NAME, "SlotRecipeId")?;
            }
            b"TalentPoint" => {
                self.talent_point = ctx
                    .decode_field(val, Self::TABLE_NAME, "TalentPoint")?;
            }
            b"Type1" => self.type1 = ctx.decode_field(val, Self::TABLE_NAME, "Type1")?,
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
            b"Type2" => self.type2 = ctx.decode_field(val, Self::TABLE_NAME, "Type2")?,
            b"Type3" => self.type3 = ctx.decode_field(val, Self::TABLE_NAME, "Type3")?,
            b"Type4" => self.type4 = ctx.decode_field(val, Self::TABLE_NAME, "Type4")?,
            b"_DesIn" => {
                self._des_in = ctx.decode_field(val, Self::TABLE_NAME, "_DesIn")?;
            }
            b"_DesOut" => {
                self._des_out = ctx.decode_field(val, Self::TABLE_NAME, "_DesOut")?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
impl DataCfgRow for ConfigRecipeNewRow {
    const TABLE_NAME: &'static str = "config_recipe_new";
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
/// Typed `DataTable` for `config_recipe_new.out`.
pub type ConfigRecipeNewTable = DataTable<u16, ConfigRecipeNewRow>;
/// Parses `config_recipe_new.out` into a [`ConfigRecipeNewTable`].
pub fn parse_config_recipe_new(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<ConfigRecipeNewTable, LuacError> {
    parse_luatable_datacfg(buf, scratch)
}
pub(crate) fn verify_forge_datacfg_by_stem(
    stem: &str,
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Option<Result<u32, LuacError>> {
    match stem {
        "config_cardrecipe" => {
            Some(parse_config_cardrecipe(buf, scratch).map(|t| t.len()))
        }
        "config_forge_dream" => {
            Some(parse_config_forge_dream(buf, scratch).map(|t| t.len()))
        }
        "config_forge_dream_material" => {
            Some(parse_config_forge_dream_material(buf, scratch).map(|t| t.len()))
        }
        "config_forge_precise_config" => {
            Some(parse_config_forge_precise_config(buf, scratch).map(|t| t.len()))
        }
        "config_forge_s13_organ_config" => {
            Some(parse_config_forge_s13_organ_config(buf, scratch).map(|t| t.len()))
        }
        "config_forge_upgarde_config" => {
            Some(parse_config_forge_upgarde_config(buf, scratch).map(|t| t.len()))
        }
        "config_recipe_list" => {
            Some(parse_config_recipe_list(buf, scratch).map(|t| t.len()))
        }
        "config_recipe_new" => {
            Some(parse_config_recipe_new(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

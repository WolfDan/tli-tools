//! Auto-generated `.bin` configuration table types and parsers for the `monster` domain.
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::FConfigStrI32;
use crate::bin::types::{BinTable, VecDim};
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_boss_challenge_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigBossChallengeRow {
    pub level_name: Box<str>,
    pub star_req: Box<[i32]>,
    pub star_reward: Box<[i32]>,
    pub star_reward_season_share: FxHashMap<i32, i32>,
    pub star_reward_season: FxHashMap<i32, i32>,
    pub outra_pic: Box<str>,
    pub inner_pic: Box<str>,
    pub dps: Box<str>,
    pub mechanic_desc: Box<str>,
    pub boss_name: Box<str>,
    pub trap_pos: VecDim,
    pub id: i32,
    pub level_id: i32,
    pub mechanic_desc_id: i32,
    pub boss_id: i32,
}
impl ConfigBossChallengeRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.level_name = cur.read_str()?;
        self.level_id = cur.read_i32()?;
        self.star_req = cur.read_array_i32()?;
        self.star_reward = cur.read_array_i32()?;
        self.star_reward_season_share = cur.read_map_i32_i32()?;
        self.star_reward_season = cur.read_map_i32_i32()?;
        self.outra_pic = cur.read_str()?;
        self.inner_pic = cur.read_str()?;
        self.dps = cur.read_str()?;
        self.mechanic_desc = cur.read_str()?;
        self.mechanic_desc_id = cur.read_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.boss_name = cur.read_str()?;
        self.boss_id = cur.read_i32()?;
        self.trap_pos = cur.read_vec_dim()?;
        Ok(())
    }
}
impl BinRow for ConfigBossChallengeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_boss_challenge_ChunkA0.bin`.
pub type ConfigBossChallengeTable = BinTable<i32, ConfigBossChallengeRow>;
/// Parses `config_boss_challenge_ChunkA0.bin` into a [`ConfigBossChallengeTable`].
pub fn parse_config_boss_challenge(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigBossChallengeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_monster_appear_anim_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMonsterAppearAnimRow {
    pub montage: Box<[FConfigStrI32]>,
    pub id: i32,
    pub index: i32,
}
impl BinRow for ConfigMonsterAppearAnimRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.index = cur.read_i32()?;
        self.montage = FConfigStrI32::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_monster_appear_anim_ChunkA0.bin`.
pub type ConfigMonsterAppearAnimTable = BinTable<(i32, i32), ConfigMonsterAppearAnimRow>;
/// Parses `config_monster_appear_anim_ChunkA0.bin` into a [`ConfigMonsterAppearAnimTable`].
pub fn parse_config_monster_appear_anim(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMonsterAppearAnimTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_monster_kind_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMonsterKindRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigMonsterKindRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_monster_kind_ChunkA0.bin`.
pub type ConfigMonsterKindTable = BinTable<i32, ConfigMonsterKindRow>;
/// Parses `config_monster_kind_ChunkA0.bin` into a [`ConfigMonsterKindTable`].
pub fn parse_config_monster_kind(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMonsterKindTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_monster_source_type_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMonsterSourceTypeRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigMonsterSourceTypeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_monster_source_type_ChunkA0.bin`.
pub type ConfigMonsterSourceTypeTable = BinTable<i32, ConfigMonsterSourceTypeRow>;
/// Parses `config_monster_source_type_ChunkA0.bin` into a [`ConfigMonsterSourceTypeTable`].
pub fn parse_config_monster_source_type(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMonsterSourceTypeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_monster_stage_drop_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMonsterStageDropRow {
    pub monster_id: i32,
    pub attr: i32,
    pub point: i32,
    pub count: i32,
    pub drop: i32,
}
impl BinRow for ConfigMonsterStageDropRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.monster_id = cur.read_i32()?;
        self.attr = cur.read_i32()?;
        self.point = cur.read_i32()?;
        self.count = cur.read_i32()?;
        self.drop = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_monster_stage_drop_ChunkA0.bin`.
pub type ConfigMonsterStageDropTable = BinTable<
    (i32, i32, i32),
    ConfigMonsterStageDropRow,
>;
/// Parses `config_monster_stage_drop_ChunkA0.bin` into a [`ConfigMonsterStageDropTable`].
pub fn parse_config_monster_stage_drop(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMonsterStageDropTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_monster_statistics_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMonsterStatisticsRow {
    pub des: Box<str>,
    pub r#enum: Box<str>,
    pub monster_cfgid: Box<[i32]>,
    pub id: i32,
    pub gameplaytype: i32,
}
impl BinRow for ConfigMonsterStatisticsRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.r#enum = cur.read_str()?;
        self.monster_cfgid = cur.read_array_i32()?;
        self.gameplaytype = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_monster_statistics_ChunkA0.bin`.
pub type ConfigMonsterStatisticsTable = BinTable<i32, ConfigMonsterStatisticsRow>;
/// Parses `config_monster_statistics_ChunkA0.bin` into a [`ConfigMonsterStatisticsTable`].
pub fn parse_config_monster_statistics(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMonsterStatisticsTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_monster_tag_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMonsterTagRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigMonsterTagRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_monster_tag_ChunkA0.bin`.
pub type ConfigMonsterTagTable = BinTable<i32, ConfigMonsterTagRow>;
/// Parses `config_monster_tag_ChunkA0.bin` into a [`ConfigMonsterTagTable`].
pub fn parse_config_monster_tag(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigMonsterTagTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_monster_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_boss_challenge" => {
            Some(parse_config_boss_challenge(buf, scratch).map(|t| t.len()))
        }
        "config_monster_appear_anim" => {
            Some(parse_config_monster_appear_anim(buf, scratch).map(|t| t.len()))
        }
        "config_monster_kind" => {
            Some(parse_config_monster_kind(buf, scratch).map(|t| t.len()))
        }
        "config_monster_source_type" => {
            Some(parse_config_monster_source_type(buf, scratch).map(|t| t.len()))
        }
        "config_monster_stage_drop" => {
            Some(parse_config_monster_stage_drop(buf, scratch).map(|t| t.len()))
        }
        "config_monster_statistics" => {
            Some(parse_config_monster_statistics(buf, scratch).map(|t| t.len()))
        }
        "config_monster_tag" => {
            Some(parse_config_monster_tag(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

//! Auto-generated `.bin` configuration table types and parsers for the `audio` domain.
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::FConfigI32I32;
use crate::bin::types::BinTable;
use crate::bin::wire::WireCursor;
/// Row record for `config_audio_boss_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAudioBossRow {
    pub stage: Box<[FConfigI32I32]>,
    pub id: i32,
}
impl BinRow for ConfigAudioBossRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.stage = FConfigI32I32::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_audio_boss_ChunkA0.bin`.
pub type ConfigAudioBossTable = BinTable<i32, ConfigAudioBossRow>;
/// Parses `config_audio_boss_ChunkA0.bin` into a [`ConfigAudioBossTable`].
pub fn parse_config_audio_boss(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAudioBossTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_audio_monologue_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAudioMonologueRow {
    pub path: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigAudioMonologueRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_audio_monologue_ChunkA0.bin`.
pub type ConfigAudioMonologueTable = BinTable<i32, ConfigAudioMonologueRow>;
/// Parses `config_audio_monologue_ChunkA0.bin` into a [`ConfigAudioMonologueTable`].
pub fn parse_config_audio_monologue(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAudioMonologueTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_audio_play_dialogue_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAudioPlayDialogueRow {
    pub path: Box<str>,
    pub group: i32,
    pub group_index: i32,
    pub weight: i32,
}
impl BinRow for ConfigAudioPlayDialogueRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.group = cur.read_i32()?;
        self.group_index = cur.read_i32()?;
        self.weight = cur.read_i32()?;
        self.path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_audio_play_dialogue_ChunkA0.bin`.
pub type ConfigAudioPlayDialogueTable = BinTable<(i32, i32), ConfigAudioPlayDialogueRow>;
/// Parses `config_audio_play_dialogue_ChunkA0.bin` into a [`ConfigAudioPlayDialogueTable`].
pub fn parse_config_audio_play_dialogue(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAudioPlayDialogueTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_audio_special_boss_music_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigAudioSpecialBossMusicRow {
    pub special_key: Box<str>,
    pub npc_id: i32,
}
impl BinRow for ConfigAudioSpecialBossMusicRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.npc_id = cur.read_i32()?;
        self.special_key = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_audio_special_boss_music_ChunkA0.bin`.
pub type ConfigAudioSpecialBossMusicTable = BinTable<
    i32,
    ConfigAudioSpecialBossMusicRow,
>;
/// Parses `config_audio_special_boss_music_ChunkA0.bin` into a [`ConfigAudioSpecialBossMusicTable`].
pub fn parse_config_audio_special_boss_music(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigAudioSpecialBossMusicTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
pub(crate) fn verify_audio_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_audio_boss" => {
            Some(parse_config_audio_boss(buf, scratch).map(|t| t.len()))
        }
        "config_audio_monologue" => {
            Some(parse_config_audio_monologue(buf, scratch).map(|t| t.len()))
        }
        "config_audio_play_dialogue" => {
            Some(parse_config_audio_play_dialogue(buf, scratch).map(|t| t.len()))
        }
        "config_audio_special_boss_music" => {
            Some(parse_config_audio_special_boss_music(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}

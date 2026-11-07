//! Auto-generated `.bin` configuration table types and parsers for the `resource` domain.
use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table};
use crate::bin::error::BinError;
use crate::bin::tables::substructs::{FConfigI32Str, FConfigStrArrayI32, FConfigStrI32};
use crate::bin::types::{BinTable, VecDim};
use crate::bin::wire::WireCursor;
use rustc_hash::FxHashMap;
/// Row record for `config_resource_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigResourceRow {
    pub name: Box<str>,
    pub title: Box<str>,
    pub mesh: Box<str>,
    pub material_set: FxHashMap<i32, Box<[i32]>>,
    pub scale: f64,
    pub offset: f64,
    pub anim_bp_c: Box<str>,
    pub physics_asset: FxHashMap<Box<str>, Box<str>>,
    pub vfx: Box<[FConfigStrArrayI32]>,
    pub weapon_scale: f64,
    pub head_icon: Box<str>,
    pub portrait: Box<str>,
    pub rotation_speed_override: Box<[f64]>,
    pub id: i32,
    pub resource_anim_cfg_set: i32,
    pub animation_share: i32,
    pub victim_snd_type: i32,
}
impl ConfigResourceRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.title = cur.read_str()?;
        self.mesh = cur.read_str()?;
        self.material_set = cur.read_map_i32_array_i32()?;
        self.scale = cur.read_f64()?;
        self.offset = cur.read_f64()?;
        self.anim_bp_c = cur.read_str()?;
        self.physics_asset = cur.read_map_str_str()?;
        self.resource_anim_cfg_set = cur.read_i32()?;
        self.vfx = FConfigStrArrayI32::read_array(cur)?;
        self.weapon_scale = cur.read_f64()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.animation_share = cur.read_i32()?;
        self.head_icon = cur.read_str()?;
        self.portrait = cur.read_str()?;
        self.victim_snd_type = cur.read_i32()?;
        self.rotation_speed_override = cur.read_array_f64()?;
        Ok(())
    }
}
impl BinRow for ConfigResourceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_ChunkA0.bin`.
pub type ConfigResourceTable = BinTable<i32, ConfigResourceRow>;
/// Parses `config_resource_ChunkA0.bin` into a [`ConfigResourceTable`].
pub fn parse_config_resource(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_anim_cfg_entry_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceAnimCfgEntryRow {
    pub desc: Box<str>,
    pub anim_string: Box<str>,
    pub main_weapon: Box<[i32]>,
    pub offhand_weapon: Box<[i32]>,
    pub anim_suffix: Box<str>,
    pub id: i32,
    pub resource_anim_res_id: i32,
}
impl BinRow for ConfigResourceAnimCfgEntryRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.resource_anim_res_id = cur.read_i32()?;
        self.anim_string = cur.read_str()?;
        self.main_weapon = cur.read_array_i32()?;
        self.offhand_weapon = cur.read_array_i32()?;
        self.anim_suffix = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_anim_cfg_entry_ChunkA0.bin`.
pub type ConfigResourceAnimCfgEntryTable = BinTable<i32, ConfigResourceAnimCfgEntryRow>;
/// Parses `config_resource_anim_cfg_entry_ChunkA0.bin` into a [`ConfigResourceAnimCfgEntryTable`].
pub fn parse_config_resource_anim_cfg_entry(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceAnimCfgEntryTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_anim_cfg_set_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceAnimCfgSetRow {
    pub name: Box<str>,
    pub resource_anim_cfg_entry_ids: Box<[i32]>,
    pub montage_anim: Box<[FConfigStrI32]>,
    pub id: i32,
}
impl BinRow for ConfigResourceAnimCfgSetRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.name = cur.read_str()?;
        self.resource_anim_cfg_entry_ids = cur.read_array_i32()?;
        self.montage_anim = FConfigStrI32::read_array(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_anim_cfg_set_ChunkA0.bin`.
pub type ConfigResourceAnimCfgSetTable = BinTable<i32, ConfigResourceAnimCfgSetRow>;
/// Parses `config_resource_anim_cfg_set_ChunkA0.bin` into a [`ConfigResourceAnimCfgSetTable`].
pub fn parse_config_resource_anim_cfg_set(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceAnimCfgSetTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_anim_montage_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceAnimMontageRow {
    pub des: Box<str>,
    pub anim_montage: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourceAnimMontageRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.anim_montage = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_anim_montage_ChunkA0.bin`.
pub type ConfigResourceAnimMontageTable = BinTable<i32, ConfigResourceAnimMontageRow>;
/// Parses `config_resource_anim_montage_ChunkA0.bin` into a [`ConfigResourceAnimMontageTable`].
pub fn parse_config_resource_anim_montage(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceAnimMontageTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_anim_res_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceAnimResRow {
    pub des: Box<str>,
    pub anim_res_path: Box<str>,
    pub id: i32,
    pub ui_show_case_anim_set_id: i32,
}
impl BinRow for ConfigResourceAnimResRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.anim_res_path = cur.read_str()?;
        self.ui_show_case_anim_set_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_anim_res_ChunkA0.bin`.
pub type ConfigResourceAnimResTable = BinTable<i32, ConfigResourceAnimResRow>;
/// Parses `config_resource_anim_res_ChunkA0.bin` into a [`ConfigResourceAnimResTable`].
pub fn parse_config_resource_anim_res(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceAnimResTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_award_type_icon_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceAwardTypeIconRow {
    pub icon: Box<str>,
    pub desc: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourceAwardTypeIconRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.desc = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_award_type_icon_ChunkA0.bin`.
pub type ConfigResourceAwardTypeIconTable = BinTable<
    i32,
    ConfigResourceAwardTypeIconRow,
>;
/// Parses `config_resource_award_type_icon_ChunkA0.bin` into a [`ConfigResourceAwardTypeIconTable`].
pub fn parse_config_resource_award_type_icon(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceAwardTypeIconTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_camera_anim_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceCameraAnimRow {
    pub des: Box<str>,
    pub camera_anim_res_path: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourceCameraAnimRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.camera_anim_res_path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_camera_anim_ChunkA0.bin`.
pub type ConfigResourceCameraAnimTable = BinTable<i32, ConfigResourceCameraAnimRow>;
/// Parses `config_resource_camera_anim_ChunkA0.bin` into a [`ConfigResourceCameraAnimTable`].
pub fn parse_config_resource_camera_anim(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceCameraAnimTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_camera_shake_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceCameraShakeRow {
    pub des: Box<str>,
    pub camera_shake_res_path: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourceCameraShakeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.camera_shake_res_path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_camera_shake_ChunkA0.bin`.
pub type ConfigResourceCameraShakeTable = BinTable<i32, ConfigResourceCameraShakeRow>;
/// Parses `config_resource_camera_shake_ChunkA0.bin` into a [`ConfigResourceCameraShakeTable`].
pub fn parse_config_resource_camera_shake(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceCameraShakeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_curve_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceCurveRow {
    pub des: Box<str>,
    pub res_path: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourceCurveRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.res_path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_curve_ChunkA0.bin`.
pub type ConfigResourceCurveTable = BinTable<i32, ConfigResourceCurveRow>;
/// Parses `config_resource_curve_ChunkA0.bin` into a [`ConfigResourceCurveTable`].
pub fn parse_config_resource_curve(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceCurveTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_drop_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceDropRow {
    pub desc: Box<str>,
    pub mesh: Box<str>,
    pub drop_audio_effect: Box<str>,
    pub audio_effect: Box<str>,
    pub material: Box<[i32]>,
    pub fx: Box<[i32]>,
    pub id: i32,
    pub referenced_id: i32,
}
impl BinRow for ConfigResourceDropRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.mesh = cur.read_str()?;
        self.referenced_id = cur.read_i32()?;
        self.drop_audio_effect = cur.read_str()?;
        self.audio_effect = cur.read_str()?;
        self.material = cur.read_array_i32()?;
        self.fx = cur.read_array_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_drop_ChunkA0.bin`.
pub type ConfigResourceDropTable = BinTable<i32, ConfigResourceDropRow>;
/// Parses `config_resource_drop_ChunkA0.bin` into a [`ConfigResourceDropTable`].
pub fn parse_config_resource_drop(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceDropTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_drop_anim_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigResourceDropAnimRow {
    pub desc: Box<str>,
    pub mesh: Box<str>,
    pub anim: Box<str>,
    pub last_frame_data: Box<[VecDim]>,
    pub id: i32,
}
impl BinRow for ConfigResourceDropAnimRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.mesh = cur.read_str()?;
        self.anim = cur.read_str()?;
        self.last_frame_data = cur.read_array_vec_dim()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_drop_anim_ChunkA0.bin`.
pub type ConfigResourceDropAnimTable = BinTable<i32, ConfigResourceDropAnimRow>;
/// Parses `config_resource_drop_anim_ChunkA0.bin` into a [`ConfigResourceDropAnimTable`].
pub fn parse_config_resource_drop_anim(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceDropAnimTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_item_type_icon_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceItemTypeIconRow {
    pub icon: Box<str>,
    pub id: i32,
    pub type1: i32,
    pub type2: i32,
    pub type3: i32,
    pub type4: i32,
}
impl BinRow for ConfigResourceItemTypeIconRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.icon = cur.read_str()?;
        self.type1 = cur.read_i32()?;
        self.type2 = cur.read_i32()?;
        self.type3 = cur.read_i32()?;
        self.type4 = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_item_type_icon_ChunkA0.bin`.
pub type ConfigResourceItemTypeIconTable = BinTable<i32, ConfigResourceItemTypeIconRow>;
/// Parses `config_resource_item_type_icon_ChunkA0.bin` into a [`ConfigResourceItemTypeIconTable`].
pub fn parse_config_resource_item_type_icon(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceItemTypeIconTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_level_sequence_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigResourceLevelSequenceRow {
    pub desc: Box<str>,
    pub scale: f64,
    pub sequence: Box<str>,
    pub group_id: Box<[i32]>,
    pub id: i32,
    pub subtitle_id: i32,
    pub r#loop: bool,
}
impl BinRow for ConfigResourceLevelSequenceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.scale = cur.read_f64()?;
        self.r#loop = cur.read_bool()?;
        self.sequence = cur.read_str()?;
        self.group_id = cur.read_array_i32()?;
        self.subtitle_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_level_sequence_ChunkA0.bin`.
pub type ConfigResourceLevelSequenceTable = BinTable<
    i32,
    ConfigResourceLevelSequenceRow,
>;
/// Parses `config_resource_level_sequence_ChunkA0.bin` into a [`ConfigResourceLevelSequenceTable`].
pub fn parse_config_resource_level_sequence(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceLevelSequenceTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_load_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceLoadRow {
    pub res: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourceLoadRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.res = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_load_ChunkA0.bin`.
pub type ConfigResourceLoadTable = BinTable<i32, ConfigResourceLoadRow>;
/// Parses `config_resource_load_ChunkA0.bin` into a [`ConfigResourceLoadTable`].
pub fn parse_config_resource_load(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceLoadTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_material_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceMaterialRow {
    pub des: Box<str>,
    pub material_res_path: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourceMaterialRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.material_res_path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_material_ChunkA0.bin`.
pub type ConfigResourceMaterialTable = BinTable<i32, ConfigResourceMaterialRow>;
/// Parses `config_resource_material_ChunkA0.bin` into a [`ConfigResourceMaterialTable`].
pub fn parse_config_resource_material(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceMaterialTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_particle_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigResourceParticleRow {
    pub des: Box<str>,
    pub particle_res_path: Box<str>,
    pub scale: f64,
    pub id: i32,
}
impl BinRow for ConfigResourceParticleRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.particle_res_path = cur.read_str()?;
        self.scale = cur.read_f64()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_particle_ChunkA0.bin`.
pub type ConfigResourceParticleTable = BinTable<i32, ConfigResourceParticleRow>;
/// Parses `config_resource_particle_ChunkA0.bin` into a [`ConfigResourceParticleTable`].
pub fn parse_config_resource_particle(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceParticleTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_phantom_weapon_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourcePhantomWeaponRow {
    pub id: i32,
    pub resouce_weapon_id: i32,
}
impl BinRow for ConfigResourcePhantomWeaponRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.resouce_weapon_id = cur.read_i32()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_phantom_weapon_ChunkA0.bin`.
pub type ConfigResourcePhantomWeaponTable = BinTable<
    i32,
    ConfigResourcePhantomWeaponRow,
>;
/// Parses `config_resource_phantom_weapon_ChunkA0.bin` into a [`ConfigResourcePhantomWeaponTable`].
pub fn parse_config_resource_phantom_weapon(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourcePhantomWeaponTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_pharmacy_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourcePharmacyRow {
    pub desc: Box<str>,
    pub bottle_icon: Box<str>,
    pub liquid_icon: Box<str>,
    pub gold_liquid_icon: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourcePharmacyRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.bottle_icon = cur.read_str()?;
        self.liquid_icon = cur.read_str()?;
        self.gold_liquid_icon = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_pharmacy_ChunkA0.bin`.
pub type ConfigResourcePharmacyTable = BinTable<i32, ConfigResourcePharmacyRow>;
/// Parses `config_resource_pharmacy_ChunkA0.bin` into a [`ConfigResourcePharmacyTable`].
pub fn parse_config_resource_pharmacy(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourcePharmacyTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_premake_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourcePremakeRow {
    pub mesh: Box<str>,
    pub physics_asset: FxHashMap<Box<str>, Box<str>>,
    pub id: i32,
}
impl BinRow for ConfigResourcePremakeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.mesh = cur.read_str()?;
        self.physics_asset = cur.read_map_str_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_premake_ChunkA0.bin`.
pub type ConfigResourcePremakeTable = BinTable<i32, ConfigResourcePremakeRow>;
/// Parses `config_resource_premake_ChunkA0.bin` into a [`ConfigResourcePremakeTable`].
pub fn parse_config_resource_premake(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourcePremakeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_sequence_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigResourceSequenceRow {
    pub desc: Box<str>,
    pub scale: f64,
    pub sequence: Box<str>,
    pub id: i32,
    pub r#loop: bool,
}
impl BinRow for ConfigResourceSequenceRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.scale = cur.read_f64()?;
        self.r#loop = cur.read_bool()?;
        self.sequence = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_sequence_ChunkA0.bin`.
pub type ConfigResourceSequenceTable = BinTable<i32, ConfigResourceSequenceRow>;
/// Parses `config_resource_sequence_ChunkA0.bin` into a [`ConfigResourceSequenceTable`].
pub fn parse_config_resource_sequence(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceSequenceTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_simple_anime_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceSimpleAnimeRow {
    pub desc: Box<str>,
    pub path: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourceSimpleAnimeRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_simple_anime_ChunkA0.bin`.
pub type ConfigResourceSimpleAnimeTable = BinTable<i32, ConfigResourceSimpleAnimeRow>;
/// Parses `config_resource_simple_anime_ChunkA0.bin` into a [`ConfigResourceSimpleAnimeTable`].
pub fn parse_config_resource_simple_anime(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceSimpleAnimeTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_skeletal_mesh_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceSkeletalMeshRow {
    pub des: Box<str>,
    pub skeletal_mesh_res_path: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourceSkeletalMeshRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.skeletal_mesh_res_path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_skeletal_mesh_ChunkA0.bin`.
pub type ConfigResourceSkeletalMeshTable = BinTable<i32, ConfigResourceSkeletalMeshRow>;
/// Parses `config_resource_skeletal_mesh_ChunkA0.bin` into a [`ConfigResourceSkeletalMeshTable`].
pub fn parse_config_resource_skeletal_mesh(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceSkeletalMeshTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_skill_ctrl_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigResourceSkillCtrlRow {
    pub desc: Box<str>,
    pub mesh: Box<str>,
    pub material: Box<[FConfigI32Str]>,
    pub scale: VecDim,
    pub id: i32,
    pub r#type: i32,
}
impl BinRow for ConfigResourceSkillCtrlRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.desc = cur.read_str()?;
        self.r#type = cur.read_i32()?;
        self.mesh = cur.read_str()?;
        self.material = FConfigI32Str::read_array(cur)?;
        self.scale = cur.read_vec_dim()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_skill_ctrl_ChunkA0.bin`.
pub type ConfigResourceSkillCtrlTable = BinTable<i32, ConfigResourceSkillCtrlRow>;
/// Parses `config_resource_skill_ctrl_ChunkA0.bin` into a [`ConfigResourceSkillCtrlTable`].
pub fn parse_config_resource_skill_ctrl(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceSkillCtrlTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_static_mesh_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceStaticMeshRow {
    pub des: Box<str>,
    pub static_mesh_res_path: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigResourceStaticMeshRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.static_mesh_res_path = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_static_mesh_ChunkA0.bin`.
pub type ConfigResourceStaticMeshTable = BinTable<i32, ConfigResourceStaticMeshRow>;
/// Parses `config_resource_static_mesh_ChunkA0.bin` into a [`ConfigResourceStaticMeshTable`].
pub fn parse_config_resource_static_mesh(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceStaticMeshTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_resource_weapon_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResourceWeaponRow {
    pub des: Box<str>,
    pub weapon_static_mesh_res: Box<str>,
    pub skeletal_mesh_res: Box<str>,
    pub anim_res: Box<str>,
    pub weapon_high_mesh_path: Box<str>,
    pub sockets: FxHashMap<i32, Box<str>>,
    pub vfx: Box<[i32]>,
    pub vfx_showcase: Box<[i32]>,
    pub sockets_v2: FxHashMap<i32, Box<str>>,
    pub id: i32,
    pub high_skeletal_mesh_res_id: i32,
    pub high_anim_res_id: i32,
    pub offhand_mirror_axes: i32,
    pub enable_physics: i32,
}
impl ConfigResourceWeaponRow {
    #[inline(never)]
    fn read_fields_0(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.des = cur.read_str()?;
        self.weapon_static_mesh_res = cur.read_str()?;
        self.skeletal_mesh_res = cur.read_str()?;
        self.anim_res = cur.read_str()?;
        self.weapon_high_mesh_path = cur.read_str()?;
        self.high_skeletal_mesh_res_id = cur.read_i32()?;
        self.high_anim_res_id = cur.read_i32()?;
        self.sockets = cur.read_map_i32_str()?;
        self.offhand_mirror_axes = cur.read_i32()?;
        self.vfx = cur.read_array_i32()?;
        self.vfx_showcase = cur.read_array_i32()?;
        Ok(())
    }
    #[inline(never)]
    fn read_fields_1(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.sockets_v2 = cur.read_map_i32_str()?;
        self.enable_physics = cur.read_i32()?;
        Ok(())
    }
}
impl BinRow for ConfigResourceWeaponRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.read_fields_0(cur)?;
        self.read_fields_1(cur)?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_resource_weapon_ChunkA0.bin`.
pub type ConfigResourceWeaponTable = BinTable<i32, ConfigResourceWeaponRow>;
/// Parses `config_resource_weapon_ChunkA0.bin` into a [`ConfigResourceWeaponTable`].
pub fn parse_config_resource_weapon(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigResourceWeaponTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
/// Row record for `config_vfx_event_ChunkA0.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigVfxEventRow {
    pub r#enum: Box<str>,
    pub des: Box<str>,
    pub vfx: Box<[i32]>,
    pub sfx: Box<[Box<str>]>,
    pub param: Box<str>,
    pub id: i32,
}
impl BinRow for ConfigVfxEventRow {
    #[inline(never)]
    fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
        self.id = cur.read_i32()?;
        self.r#enum = cur.read_str()?;
        self.des = cur.read_str()?;
        self.vfx = cur.read_array_i32()?;
        self.sfx = cur.read_array_str()?;
        self.param = cur.read_str()?;
        Ok(())
    }
}
/// Typed `BinTable` for `config_vfx_event_ChunkA0.bin`.
pub type ConfigVfxEventTable = BinTable<i32, ConfigVfxEventRow>;
/// Parses `config_vfx_event_ChunkA0.bin` into a [`ConfigVfxEventTable`].
pub fn parse_config_vfx_event(
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Result<ConfigVfxEventTable, BinError> {
    parse_chunk_bin_table(buf, scratch)
}
#[inline(never)]
fn verify_resource_chunk_0(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_resource" => Some(parse_config_resource(buf, scratch).map(|t| t.len())),
        "config_resource_anim_cfg_entry" => {
            Some(parse_config_resource_anim_cfg_entry(buf, scratch).map(|t| t.len()))
        }
        "config_resource_anim_cfg_set" => {
            Some(parse_config_resource_anim_cfg_set(buf, scratch).map(|t| t.len()))
        }
        "config_resource_anim_montage" => {
            Some(parse_config_resource_anim_montage(buf, scratch).map(|t| t.len()))
        }
        "config_resource_anim_res" => {
            Some(parse_config_resource_anim_res(buf, scratch).map(|t| t.len()))
        }
        "config_resource_award_type_icon" => {
            Some(parse_config_resource_award_type_icon(buf, scratch).map(|t| t.len()))
        }
        "config_resource_camera_anim" => {
            Some(parse_config_resource_camera_anim(buf, scratch).map(|t| t.len()))
        }
        "config_resource_camera_shake" => {
            Some(parse_config_resource_camera_shake(buf, scratch).map(|t| t.len()))
        }
        "config_resource_curve" => {
            Some(parse_config_resource_curve(buf, scratch).map(|t| t.len()))
        }
        "config_resource_drop" => {
            Some(parse_config_resource_drop(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_resource_chunk_1(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_resource_drop_anim" => {
            Some(parse_config_resource_drop_anim(buf, scratch).map(|t| t.len()))
        }
        "config_resource_item_type_icon" => {
            Some(parse_config_resource_item_type_icon(buf, scratch).map(|t| t.len()))
        }
        "config_resource_level_sequence" => {
            Some(parse_config_resource_level_sequence(buf, scratch).map(|t| t.len()))
        }
        "config_resource_load" => {
            Some(parse_config_resource_load(buf, scratch).map(|t| t.len()))
        }
        "config_resource_material" => {
            Some(parse_config_resource_material(buf, scratch).map(|t| t.len()))
        }
        "config_resource_particle" => {
            Some(parse_config_resource_particle(buf, scratch).map(|t| t.len()))
        }
        "config_resource_phantom_weapon" => {
            Some(parse_config_resource_phantom_weapon(buf, scratch).map(|t| t.len()))
        }
        "config_resource_pharmacy" => {
            Some(parse_config_resource_pharmacy(buf, scratch).map(|t| t.len()))
        }
        "config_resource_premake" => {
            Some(parse_config_resource_premake(buf, scratch).map(|t| t.len()))
        }
        "config_resource_sequence" => {
            Some(parse_config_resource_sequence(buf, scratch).map(|t| t.len()))
        }
        _ => None,
    }
}
#[inline(never)]
fn verify_resource_chunk_2(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    match stem {
        "config_resource_simple_anime" => {
            Some(parse_config_resource_simple_anime(buf, scratch).map(|t| t.len()))
        }
        "config_resource_skeletal_mesh" => {
            Some(parse_config_resource_skeletal_mesh(buf, scratch).map(|t| t.len()))
        }
        "config_resource_skill_ctrl" => {
            Some(parse_config_resource_skill_ctrl(buf, scratch).map(|t| t.len()))
        }
        "config_resource_static_mesh" => {
            Some(parse_config_resource_static_mesh(buf, scratch).map(|t| t.len()))
        }
        "config_resource_weapon" => {
            Some(parse_config_resource_weapon(buf, scratch).map(|t| t.len()))
        }
        "config_vfx_event" => Some(parse_config_vfx_event(buf, scratch).map(|t| t.len())),
        _ => None,
    }
}
pub(crate) fn verify_resource_bin_by_stem(
    stem: &str,
    buf: &[u8],
    scratch: &mut BinScratch,
) -> Option<Result<u32, BinError>> {
    if let Some(res) = verify_resource_chunk_0(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_resource_chunk_1(stem, buf, scratch) {
        return Some(res);
    }
    if let Some(res) = verify_resource_chunk_2(stem, buf, scratch) {
        return Some(res);
    }
    None
}

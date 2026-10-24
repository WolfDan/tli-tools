//! Programmatic Unreal Engine 4.27 Pak v12 (`0x6B2A56B8`), AES-256, Oodle (`oozextract`),
//! `UTexture2D` (`PF_BC7` / `PF_DXT5` / `PF_B8G8R8A8`), UnLua `\x01XDI` bytecode, and
//! localization binary extractor for *Torchlight: Infinite*.

#![feature(portable_simd)]
#![forbid(unsafe_code)]

pub mod aes;
pub mod bc;
pub mod dxbc;
pub mod error;
pub mod luac;
pub mod material;
pub mod pak;
pub mod pak_cache;
pub mod particle;
pub mod shader_archive;
pub mod uasset;
pub mod umg;

use std::collections::BTreeMap;

pub use aes::{
    Aes256KeySchedule, DEFAULT_AES_KEY, DEFAULT_AES_KEY_HEX, decrypt_aes256_ecb_in_place,
    parse_aes_key_hex,
};
pub use bc::{PixelFormat, crop_rgba8, decode_mip_to_rgba8, decode_png_rgba8, encode_png_rgba8};
pub use dxbc::{
    DXBC_MAGIC, DxbcShader, DxbcSignatureElement, Sm5Instruction, Sm5Operand,
    decompile_dxbc_to_hlsl, disassemble_dxbc, format_sm5_operand, parse_dxbc, sm5_opcode_mnemonic,
};
pub use error::ExtractError;
pub use luac::{
    LUA53_HEADER_PREFIX, LuaConfigTable, LuaValue, XDI_LUA_HEADER, XDI_TO_LUA53_OPCODE,
    decompile_luac_bytes, decrypt_luac_bytes, extract_xdi_strings, parse_multi_language_bin,
    parse_xdi_config_table, strip_rich_text, xdi_to_lua53_bytes,
};
pub use material::{
    MaterialInfo, MaterialUniformExpressionSet, PreshaderHeader, ScalarParam,
    StaticComponentMaskParam, StaticSwitchParam, TextureParam, UniformScalarParamInfo,
    UniformTexture2DParamInfo, UniformVectorParamInfo, VectorParam, game_path_to_virtual_path,
    parse_material_asset,
};
pub use pak::{
    CompressionKind, ExtractScratch, PakArchive, PakEntryMeta, PakEntryRecord, PakVfs,
    ascii_lowercase_into_simd, discover_game_dir, extract_pak_entry, read_exact_at_offset,
    read_pak_index,
};
pub use particle::{
    LinearColorCurve, RichCurveKey, SimulatedUiParticle, UiParticleEmitterAsset,
    parse_ui_particle_asset,
};
pub use shader_archive::{
    DecompressedShaderBlob, SHADER_ARCHIVE_VERSION_UE427, ShaderArchiveIndex, ShaderCodeEntry,
    ShaderMapEntry, ShaderStage, decompress_shader_code_blob, extract_shader_from_pak_entry,
    extract_shader_from_record, read_shader_archive_index_from_pak,
    read_shader_archive_index_from_record,
};
pub use uasset::{
    DecodedTexture, PaperSpriteInfo, UassetExportEntry, UassetImportEntry, decode_texture2d,
    format_fname_idx, parse_paper_sprite, parse_uasset_exports, parse_uasset_imports,
    parse_uasset_names, resolve_package_index, uasset_is_texture2d,
};
pub use umg::{
    UmgAnchorData, UmgAnimationStateT0, UmgBlueprint, UmgWidgetNode, parse_umg_blueprint,
};

impl PakVfs {
    fn read_uasset_and_uexp(&self, base: &str) -> Result<(Vec<u8>, Vec<u8>), ExtractError> {
        assert!(!base.is_empty());
        let uasset_path = format!("{base}.uasset");
        let uasset_rec = self
            .get_record(&uasset_path)
            .ok_or_else(|| ExtractError::NotFound(uasset_path.clone()))?;
        let uasset = self.read_record(&uasset_rec)?;

        let uexp_path = format!("{base}.uexp");
        let uexp_rec = self
            .get_record_in_archive(uasset_rec.pak_index, &uexp_path)
            .or_else(|| self.get_record(&uexp_path))
            .ok_or(ExtractError::NotFound(uexp_path))?;
        let uexp = self.read_record(&uexp_rec)?;
        Ok((uasset, uexp))
    }

    /// Reads `.uasset`, fast-checks that it is a `UTexture2D` header, and only then reads
    /// co-located `.uexp` and optional `.ubulk` to decode the cooked texture to RGBA8.
    pub fn decode_texture(&self, asset_path: &str) -> Result<DecodedTexture, ExtractError> {
        assert!(!asset_path.is_empty());
        let base = asset_path
            .trim_start_matches('/')
            .trim_end_matches(".uasset")
            .trim_end_matches(".uexp")
            .trim_end_matches(".ubulk");
        assert!(!base.is_empty());

        let uasset_path = format!("{base}.uasset");
        let uasset_rec = self
            .get_record(&uasset_path)
            .ok_or_else(|| ExtractError::NotFound(uasset_path.clone()))?;
        let uasset = self.read_record(&uasset_rec)?;
        if !uasset_is_texture2d(&uasset) {
            return Err(ExtractError::UassetFormat(format!(
                "Not a UTexture2D asset: {base}"
            )));
        }

        let uexp_path = format!("{base}.uexp");
        let ubulk_path = format!("{base}.ubulk");
        let uexp_rec = self
            .get_record_in_archive(uasset_rec.pak_index, &uexp_path)
            .or_else(|| self.get_record(&uexp_path))
            .ok_or(ExtractError::NotFound(uexp_path))?;
        let uexp = self.read_record(&uexp_rec)?;
        let ubulk = self
            .get_record_in_archive(uasset_rec.pak_index, &ubulk_path)
            .and_then(|rec| self.read_record(&rec).ok());

        decode_texture2d(&uasset, &uexp, ubulk.as_deref())
    }

    /// Reads `.uasset` and `.uexp` for a Paper2D `UPaperSprite` asset and returns its
    /// parent atlas virtual path and crop rectangle (`BakedSourceUV`, `BakedSourceDimension`).
    pub fn read_paper_sprite(&self, asset_path: &str) -> Result<PaperSpriteInfo, ExtractError> {
        assert!(!asset_path.is_empty());
        let base = asset_path
            .trim_start_matches('/')
            .trim_end_matches(".uasset")
            .trim_end_matches(".uexp");
        assert!(!base.is_empty());

        let (uasset, uexp) = self.read_uasset_and_uexp(base)?;
        parse_paper_sprite(&uasset, &uexp).ok_or_else(|| {
            ExtractError::UassetFormat(format!("Not a valid UPaperSprite asset: {base}"))
        })
    }

    /// Reads and decrypts a `.out` UnLua bytecode file (`0x20210507` stream cipher
    /// -> `\x01XDI` Lua 5.3 bytecode).
    pub fn read_luac_decrypted(&self, virtual_path: &str) -> Result<Vec<u8>, ExtractError> {
        assert!(!virtual_path.is_empty());
        let raw = self.read_path(virtual_path)?;
        let dec = decrypt_luac_bytes(&raw);
        assert_eq!(dec.len(), raw.len());
        Ok(dec)
    }

    /// Reads, decrypts, and transpiles a `.out` UnLua bytecode file into standard 64-bit
    /// PUC-Lua 5.3 (`\x1bLua\x53`) bytecode.
    pub fn read_luac_lua53(&self, virtual_path: &str) -> Result<Vec<u8>, ExtractError> {
        assert!(!virtual_path.is_empty());
        let raw = self.read_path(virtual_path)?;
        assert!(!raw.is_empty());
        xdi_to_lua53_bytes(&raw)
    }

    /// Reads, decrypts, transpiles, and decompiles a `.out` UnLua bytecode file into
    /// formatted Lua 5.3 source code using `unluac` (`unluac-rs`).
    pub fn read_luac_decompiled(&self, virtual_path: &str) -> Result<String, ExtractError> {
        assert!(!virtual_path.is_empty());
        let raw = self.read_path(virtual_path)?;
        assert!(!raw.is_empty());
        decompile_luac_bytes(&raw)
    }

    /// Reads, decrypts, and evaluates a `DataCfg/config_*.out` Lua table file into structured rows.
    pub fn read_lua_config_table(
        &self,
        virtual_path: &str,
    ) -> Result<LuaConfigTable, ExtractError> {
        assert!(!virtual_path.is_empty());
        let raw = self.read_path(virtual_path)?;
        assert!(!raw.is_empty());
        parse_xdi_config_table(&raw)
    }

    /// Reads and parses `UE_game/Content/Script/Data/language/config_multi_language_<lang>.bin`,
    /// using the sorted on-disk binary index cache for `< 0.1 ms` prefix queries.
    pub fn read_multi_language(
        &self,
        lang_code: &str,
        prefix_filter: Option<&str>,
    ) -> Result<BTreeMap<String, String>, ExtractError> {
        assert!(!lang_code.is_empty());
        let vpath =
            format!("UE_game/Content/Script/Data/language/config_multi_language_{lang_code}.bin");
        assert!(vpath.ends_with(".bin"));
        let rec = self
            .get_record(&vpath)
            .ok_or_else(|| ExtractError::NotFound(vpath.clone()))?;
        pak_cache::read_or_cache_multi_language(lang_code, &rec, prefix_filter, || {
            self.read_record(&rec)
        })
    }

    /// Reads `.uasset` and `.uexp` for a cooked UMG Widget Blueprint and extracts its
    /// widget names, referenced asset paths, and `UCanvasPanelSlot` anchor/offset layout.
    pub fn read_umg_blueprint(&self, asset_path: &str) -> Result<UmgBlueprint, ExtractError> {
        assert!(!asset_path.is_empty());
        let base = asset_path
            .trim_start_matches('/')
            .trim_end_matches(".uasset")
            .trim_end_matches(".uexp");
        assert!(!base.is_empty());
        let (uasset, uexp) = self.read_uasset_and_uexp(base)?;
        parse_umg_blueprint(&uasset, &uexp)
    }

    /// Reads `.uasset` and `.uexp` for a cooked `UIParticleEmitterAsset` (`UI_P_V2_S16BoardCell_*`).
    pub fn read_ui_particle_asset(
        &self,
        asset_path: &str,
    ) -> Result<UiParticleEmitterAsset, ExtractError> {
        assert!(!asset_path.is_empty());
        let base = asset_path
            .trim_start_matches('/')
            .trim_end_matches(".uasset")
            .trim_end_matches(".uexp");
        assert!(!base.is_empty());
        let (uasset, uexp) = self.read_uasset_and_uexp(base)?;
        parse_ui_particle_asset(base, &uasset, &uexp)
    }

    /// Reads a cooked `UMaterial` or `UMaterialInstanceConstant` (`.uasset` + `.uexp`) and
    /// resolves its `Parent` material chain (up to 8 hops) when the child instance inherits
    /// its `ShaderMapHash` from a parent material.
    pub fn read_material_info(&self, asset_path: &str) -> Result<MaterialInfo, ExtractError> {
        assert!(!asset_path.is_empty());
        let base = asset_path
            .trim_start_matches('/')
            .trim_end_matches(".uasset")
            .trim_end_matches(".uexp");
        assert!(!base.is_empty());

        let (uasset, uexp) = self.read_uasset_and_uexp(base)?;
        let mut info = parse_material_asset(base, &uasset, &uexp)?;

        let mut cur_parent = info.parent_path.clone();
        for _ in 0..8 {
            let Some(parent_game_path) = cur_parent else {
                break;
            };
            info.parent_chain.push(parent_game_path.clone());
            let parent_vpath = game_path_to_virtual_path(&parent_game_path);
            let Ok((p_uasset, p_uexp)) = self.read_uasset_and_uexp(&parent_vpath) else {
                break;
            };
            let Ok(p_info) = parse_material_asset(&parent_vpath, &p_uasset, &p_uexp) else {
                break;
            };
            let next_parent = p_info.parent_path.clone();
            info.merge_parent(p_info);
            cur_parent = next_parent;
        }
        assert!(info.parent_chain.len() <= 8);
        Ok(info)
    }

    /// Indexes all `ShaderArchive-*.ushaderbytecode` archives by querying only `PreMountPak`
    /// and `pakchunk0` root `UE_game/Content` directories in $O(1)$.
    pub fn load_shader_archive_indices(
        &self,
    ) -> Result<Vec<(PakEntryRecord, ShaderArchiveIndex)>, ExtractError> {
        assert!(!self.archives().is_empty());
        let entries = self.collect_shader_archive_entries();
        let mut out = Vec::with_capacity(entries.len());
        for (vpath, rec) in entries {
            let arch = read_shader_archive_index_from_record(self, &vpath, &rec)?;
            out.push((rec, arch));
        }
        assert!(out.len() <= 32);
        Ok(out)
    }

    /// Extracts and Oodle-decompresses a single `shader_index` from a `ShaderArchiveIndex`.
    pub fn extract_shader_entry(
        &self,
        record: &PakEntryRecord,
        arch: &ShaderArchiveIndex,
        shader_index: usize,
    ) -> Result<DecompressedShaderBlob, ExtractError> {
        assert!(shader_index < arch.shader_entries.len());
        if let Ok(blob) = extract_shader_from_record(self, record, arch, shader_index) {
            return Ok(blob);
        }
        let full = self.read_record(record)?;
        let entry = &arch.shader_entries[shader_index];
        let hash = &arch.shader_hashes[shader_index];
        let start = (arch.payload_offset + entry.offset) as usize;
        let end = start + (entry.size as usize);
        let slice = full.get(start..end).ok_or_else(|| {
            ExtractError::ShaderFormat(format!(
                "Shader[{shader_index}] range {start}..{end} out of bounds in {}",
                arch.archive_virtual_path
            ))
        })?;
        self.with_scratch(|scratch| {
            shader_archive::decompress_shader_code_blob_with_oodle(
                shader_index,
                hash,
                entry,
                slice,
                &mut scratch.oodle,
            )
        })
    }
}

//! Programmatic Unreal Engine 4.27 Pak v12 (`0x6B2A56B8`), AES-256, Oodle (`oozextract`),
//! `UTexture2D` (`PF_BC7` / `PF_DXT5` / `PF_B8G8R8A8`), UnLua `\x01XDI` bytecode, and
//! localization binary extractor for *Torchlight: Infinite*.

#![forbid(unsafe_code)]

pub mod aes;
pub mod bc;
pub mod dxbc;
pub mod error;
pub mod luac;
pub mod material;
pub mod pak;
pub mod shader_archive;
pub mod uasset;
pub mod umg;

use std::collections::BTreeMap;

pub use aes::{
    DEFAULT_AES_KEY, DEFAULT_AES_KEY_HEX, decrypt_aes256_ecb_in_place, parse_aes_key_hex,
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
    MaterialInfo, ScalarParam, StaticComponentMaskParam, StaticSwitchParam, TextureParam,
    VectorParam, game_path_to_virtual_path, parse_material_asset,
};
pub use pak::{
    CompressionKind, PakEntryMeta, PakVfs, discover_game_dir, extract_pak_entry, read_pak_index,
};
pub use shader_archive::{
    DecompressedShaderBlob, SHADER_ARCHIVE_VERSION_UE427, ShaderArchiveIndex, ShaderCodeEntry,
    ShaderMapEntry, ShaderStage, decompress_shader_code_blob, extract_shader_from_pak_entry,
    read_shader_archive_index_from_pak,
};
pub use uasset::{
    DecodedTexture, PaperSpriteInfo, UassetImportEntry, decode_texture2d, parse_paper_sprite,
    parse_uasset_imports, parse_uasset_names, resolve_package_index,
};
pub use umg::{UmgAnchorData, UmgBlueprint, parse_umg_blueprint};

impl PakVfs {
    /// Reads `.uasset`, `.uexp`, and optional `.ubulk` for a virtual asset path
    /// (with or without extension) and decodes the cooked `UTexture2D` to RGBA8.
    pub fn decode_texture(&self, asset_path: &str) -> Result<DecodedTexture, ExtractError> {
        assert!(!asset_path.is_empty());
        let base = asset_path
            .trim_start_matches('/')
            .trim_end_matches(".uasset")
            .trim_end_matches(".uexp")
            .trim_end_matches(".ubulk");
        assert!(!base.is_empty());

        let uasset_path = format!("{base}.uasset");
        let uexp_path = format!("{base}.uexp");
        let ubulk_path = format!("{base}.ubulk");

        let uasset = self.read_path(&uasset_path)?;
        let uexp = self.read_path(&uexp_path)?;
        let ubulk = self.read_path(&ubulk_path).ok();

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

        let uasset = self.read_path(&format!("{base}.uasset"))?;
        let uexp = self.read_path(&format!("{base}.uexp"))?;
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

    /// Reads and parses `UE_game/Content/Script/Data/language/config_multi_language_<lang>.bin`.
    pub fn read_multi_language(
        &self,
        lang_code: &str,
        prefix_filter: Option<&str>,
    ) -> Result<BTreeMap<String, String>, ExtractError> {
        assert!(!lang_code.is_empty());
        let vpath =
            format!("UE_game/Content/Script/Data/language/config_multi_language_{lang_code}.bin");
        assert!(vpath.ends_with(".bin"));
        let raw = self.read_path(&vpath)?;
        Ok(parse_multi_language_bin(&raw, prefix_filter))
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
        let uasset = self.read_path(&format!("{base}.uasset"))?;
        let uexp = self.read_path(&format!("{base}.uexp"))?;
        parse_umg_blueprint(&uasset, &uexp)
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

        let uasset = self.read_path(&format!("{base}.uasset"))?;
        let uexp = self.read_path(&format!("{base}.uexp"))?;
        let mut info = parse_material_asset(base, &uasset, &uexp)?;

        let mut cur_parent = info.parent_path.clone();
        for _ in 0..8 {
            let Some(parent_game_path) = cur_parent else {
                break;
            };
            info.parent_chain.push(parent_game_path.clone());
            let parent_vpath = game_path_to_virtual_path(&parent_game_path);
            let Ok(p_uasset) = self.read_path(&format!("{parent_vpath}.uasset")) else {
                break;
            };
            let Ok(p_uexp) = self.read_path(&format!("{parent_vpath}.uexp")) else {
                break;
            };
            let Ok(p_info) = parse_material_asset(&parent_vpath, &p_uasset, &p_uexp) else {
                break;
            };
            if info.shader_map_hash.is_none() && p_info.shader_map_hash.is_some() {
                info.shader_map_hash = p_info.shader_map_hash;
                info.shader_map_hash_hex = p_info.shader_map_hash_hex;
                info.resolved_shader_source_path = parent_vpath;
            }
            if info.uniform_expression_names.is_empty()
                && !p_info.uniform_expression_names.is_empty()
            {
                info.uniform_expression_names = p_info.uniform_expression_names;
            }
            if info.blend_mode.is_none() {
                info.blend_mode = p_info.blend_mode;
            }
            if info.shading_model.is_none() {
                info.shading_model = p_info.shading_model;
            }
            cur_parent = p_info.parent_path;
        }
        assert!(info.parent_chain.len() <= 8);
        Ok(info)
    }

    /// Indexes all `ShaderArchive-*.ushaderbytecode` archives in the mounted `.pak` VFS.
    pub fn load_shader_archive_indices(
        &self,
    ) -> Result<Vec<(PakEntryMeta, ShaderArchiveIndex)>, ExtractError> {
        assert!(!self.entries.is_empty());
        let mut out = Vec::new();
        for meta in self.entries.values() {
            if meta.virtual_path.ends_with(".ushaderbytecode") {
                let arch = read_shader_archive_index_from_pak(meta, || self.read_entry(meta))?;
                out.push((meta.clone(), arch));
            }
        }
        assert!(out.len() <= self.entries.len());
        Ok(out)
    }

    /// Extracts and Oodle-decompresses a single `shader_index` from a `ShaderArchiveIndex`.
    pub fn extract_shader_entry(
        &self,
        meta: &PakEntryMeta,
        arch: &ShaderArchiveIndex,
        shader_index: usize,
    ) -> Result<DecompressedShaderBlob, ExtractError> {
        assert!(shader_index < arch.shader_entries.len());
        if let Ok(blob) = extract_shader_from_pak_entry(meta, arch, shader_index) {
            return Ok(blob);
        }
        let full = self.read_entry(meta)?;
        let entry = &arch.shader_entries[shader_index];
        let hash = &arch.shader_hashes[shader_index];
        let start = (arch.payload_offset + entry.offset) as usize;
        let end = start + (entry.size as usize);
        let slice = full.get(start..end).ok_or_else(|| {
            ExtractError::ShaderFormat(format!(
                "Shader[{shader_index}] range {start}..{end} out of bounds in {}",
                meta.virtual_path
            ))
        })?;
        decompress_shader_code_blob(shader_index, hash, entry, slice)
    }
}

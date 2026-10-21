//! Programmatic Unreal Engine 4.27 Pak v12 (`0x6B2A56B8`), AES-256, Oodle (`oozextract`),
//! `UTexture2D` (`PF_BC7` / `PF_DXT5` / `PF_B8G8R8A8`), UnLua `\x01XDI` bytecode, and
//! localization binary extractor for *Torchlight: Infinite*.

#![forbid(unsafe_code)]

pub mod aes;
pub mod bc;
pub mod error;
pub mod luac;
pub mod pak;
pub mod uasset;
pub mod umg;

use std::collections::BTreeMap;

pub use aes::{
    DEFAULT_AES_KEY, DEFAULT_AES_KEY_HEX, decrypt_aes256_ecb_in_place, parse_aes_key_hex,
};
pub use bc::{PixelFormat, crop_rgba8, decode_mip_to_rgba8, decode_png_rgba8, encode_png_rgba8};
pub use error::ExtractError;
pub use luac::{
    LuaConfigTable, LuaValue, XDI_LUA_HEADER, decrypt_luac_bytes, extract_xdi_strings,
    parse_multi_language_bin, parse_xdi_config_table, strip_rich_text,
};
pub use pak::{
    CompressionKind, PakEntryMeta, PakVfs, discover_game_dir, extract_pak_entry, read_pak_index,
};
pub use uasset::{
    DecodedTexture, PaperSpriteInfo, decode_texture2d, parse_paper_sprite, parse_uasset_names,
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
}

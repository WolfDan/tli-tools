//! 1:1 CPU UMG & Slate layer compositor for *Torchlight: Infinite* Season 16 (Tartarus)
//! monster portraits and rarity auras (`T = 0`).

use std::collections::BTreeMap;
use std::simd::{StdFloat, prelude::*};

use serde::{Deserialize, Serialize};
use tartarus_extract::{
    DxbcShader, ExtractError, MaterialInfo, PakVfs, SimulatedUiParticle, UiParticleEmitterAsset,
    UmgAnimationStateT0, UmgBlueprint, UmgWidgetNode, crop_rgba8, encode_png_rgba8,
    game_path_to_virtual_path, parse_dxbc,
};

use crate::sm5::{
    CompiledSm5Shader, LinearTexture2D, ShaderSpecialization, TexSlotMeta, compile_sm5_shader,
    compile_sm5_shader_specialized, compile_sm5_shader_with_cb2, f32x8, rasterize_compiled_quad,
    rasterize_sprite_quad_simd, select_slate_default_ps, srgb_to_linear_channel,
};

/// Reference 1080p unclipped canvas width for a single `S16BoardCell` (`256` px at `1.0x`).
pub const BASE_CANVAS_WIDTH: u32 = 256;
/// Reference 1080p unclipped canvas height for a single `S16BoardCell` (`320` px at `1.0x`).
pub const BASE_CANVAS_HEIGHT: u32 = 320;
/// Default deterministic PRNG seed for `T = 0` pre-warmed `UIParticleEmitter` simulation.
pub const DEFAULT_PARTICLE_SEED: u64 = 0x5160_2026;

const _: () = {
    assert!(BASE_CANVAS_WIDTH == 256);
    assert!(BASE_CANVAS_HEIGHT == 320);
};

/// Rarity tier (`Rating` `201..=206`) in Season 16 (Tartarus).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RarityTier {
    /// `201` — Tier 1 (White / `Bai`)
    Bai,
    /// `202` — Tier 2 (Blue / `Blue`)
    Blue,
    /// `203` — Tier 3 (Purple / `Zi`)
    Zi,
    /// `204` — Tier 4 (Orange / `Cheng`)
    Cheng,
    /// `205` — Tier 5 (Red / `Red`)
    Red,
    /// `206` — Tier 6 (Rainbow / `Cai`)
    Cai,
}

impl RarityTier {
    pub const ALL: [Self; 6] = [
        Self::Bai,
        Self::Blue,
        Self::Zi,
        Self::Cheng,
        Self::Red,
        Self::Cai,
    ];

    /// Parses a rarity tier from its color prefix (`bai`, `blue`, `zi`, `cheng`, `red`, `cai`),
    /// English name (`white`, `purple`, `orange`, `rainbow`), or `Rating` ID (`201..=206`).
    pub fn parse(s: &str) -> Option<Self> {
        assert!(s.len() <= 64);
        match s.trim().to_ascii_lowercase().as_str() {
            "201" | "bai" | "white" | "1" | "t1" => Some(Self::Bai),
            "202" | "blue" | "2" | "t2" => Some(Self::Blue),
            "203" | "zi" | "purple" | "3" | "t3" => Some(Self::Zi),
            "204" | "cheng" | "orange" | "gold" | "4" | "t4" => Some(Self::Cheng),
            "205" | "red" | "5" | "t5" => Some(Self::Red),
            "206" | "cai" | "rainbow" | "prism" | "6" | "t6" => Some(Self::Cai),
            _ => None,
        }
    }

    /// Returns the UMG widget/animation color prefix (`"Bai"`, `"Blue"`, `"Zi"`, `"Cheng"`, `"Red"`, `"Cai"`).
    pub const fn uasset_prefix(self) -> &'static str {
        match self {
            Self::Bai => "Bai",
            Self::Blue => "Blue",
            Self::Zi => "Zi",
            Self::Cheng => "Cheng",
            Self::Red => "Red",
            Self::Cai => "Cai",
        }
    }

    /// Returns the in-game `Rating` ID (`201..=206`).
    pub const fn rating_id(self) -> u32 {
        match self {
            Self::Bai => 201,
            Self::Blue => 202,
            Self::Zi => 203,
            Self::Cheng => 204,
            Self::Red => 205,
            Self::Cai => 206,
        }
    }
}

/// Rarity reveal mode (`Shao` = normal rarity aura, `Quan` = devourable/enhanced rarity aura).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RarityMode {
    #[default]
    Shao,
    Quan,
}

impl RarityMode {
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Shao => "Shao",
            Self::Quan => "Quan",
        }
    }
}

/// Canvas background mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum BackgroundMode {
    /// Transparent `(0, 0, 0, 0)` with reconstructed alpha for outer additive glows.
    #[default]
    Transparent,
    /// Opaque sRGB color `[r, g, b]` (`0..=255`).
    SolidSrgb([u8; 3]),
}

impl BackgroundMode {
    pub fn parse(s: &str) -> Option<Self> {
        assert!(s.len() <= 64);
        let t = s.trim().to_ascii_lowercase();
        match t.as_str() {
            "transparent" | "none" | "alpha" => Some(Self::Transparent),
            "dark" | "board" => Some(Self::SolidSrgb([18, 16, 24])),
            "black" => Some(Self::SolidSrgb([0, 0, 0])),
            _ => {
                let hex_str = t.strip_prefix('#').unwrap_or(&t);
                if hex_str.len() == 6 {
                    let r = u8::from_str_radix(&hex_str[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex_str[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex_str[4..6], 16).ok()?;
                    Some(Self::SolidSrgb([r, g, b]))
                } else {
                    None
                }
            }
        }
    }
}

/// Resolves a user-supplied monster portrait selector into a Pak VFS virtual texture path.
///
/// By default, `mammoth`, `lion`, `wolf`, `rat`, and `boss`/`jaws` resolve to the jailed board
/// cell portraits (`045`, `047`, `048`, `049`, `050` in `Textures/NoAtlas/`), which include the
/// dark stone cell interior and prison bars. Use `mammoth_free`, `lion_free`, `wolf_free`,
/// `rat_free` (or `101..104`) for the unjailed HUD icons.
pub fn resolve_portrait_virtual_path(selector: &str) -> String {
    assert!(!selector.is_empty());
    assert!(selector.len() <= 1024);
    let s = selector.trim();
    if s.starts_with("/Game/") {
        return game_path_to_virtual_path(s);
    }
    if s.starts_with("UE_game/") {
        return s.to_string();
    }
    let lower = s.to_ascii_lowercase();
    let jailed_code = match lower.as_str() {
        "mammoth" | "elephant" | "beast1" | "jail_mammoth" | "045" => Some("045"),
        "empty" | "jail_empty" | "046" => Some("046"),
        "lion" | "beast2" | "jail_lion" | "047" => Some("047"),
        "rat" | "mouse" | "beast4" | "jail_rat" | "048" => Some("048"),
        "mystery" | "unknown" | "wenhao" | "boss" | "jaws" | "jail_boss" | "049" => Some("049"),
        "wolf" | "beast3" | "jail_wolf" | "050" => Some("050"),
        _ => None,
    };
    if let Some(code) = jailed_code {
        return format!(
            "UE_game/Content/Art/UIV2/S16Gameplay/UI/Textures/NoAtlas/UI_T_V2_S16BoardCell_{code}"
        );
    }
    let free_code = match lower.as_str() {
        "mammoth_free" | "free_mammoth" | "101" => "101",
        "lion_free" | "free_lion" | "102" => "102",
        "wolf_free" | "free_wolf" | "103" => "103",
        "rat_free" | "free_rat" | "104" => "104",
        other if other.starts_with("ui_t_v2_s16boardcell_") => {
            return format!(
                "UE_game/Content/Art/UIV2/S16Gameplay/UIEffect/S16BoardCell/Texture/{s}"
            );
        }
        other => other,
    };
    format!(
        "UE_game/Content/Art/UIV2/S16Gameplay/UIEffect/S16BoardCell/Texture/UI_T_V2_S16BoardCell_{free_code}"
    )
}

/// Configuration for rendering a single monster cell at `T = 0`.
#[derive(Clone, Debug)]
pub struct CellRenderRequest {
    pub portrait_selector: String,
    pub rarity: RarityTier,
    pub mode: RarityMode,
    pub show_eat_icon: bool,
    pub include_shadow: bool,
    pub include_particles: bool,
    pub particle_seed: u64,
    pub scale: f32,
    pub background: BackgroundMode,
}

impl Default for CellRenderRequest {
    fn default() -> Self {
        Self {
            portrait_selector: String::from("mammoth"),
            rarity: RarityTier::Cai,
            mode: RarityMode::Shao,
            show_eat_icon: false,
            include_shadow: true,
            include_particles: true,
            particle_seed: DEFAULT_PARTICLE_SEED,
            scale: 1.0,
            background: BackgroundMode::Transparent,
        }
    }
}

/// Rendered RGBA8 image buffer.
#[derive(Clone, Debug)]
pub struct RenderedImage {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

impl RenderedImage {
    pub fn to_png_bytes(&self) -> Result<Vec<u8>, ExtractError> {
        assert!(self.width > 0 && self.height > 0);
        assert_eq!(self.rgba8.len(), (self.width * self.height * 4) as usize);
        encode_png_rgba8(self.width, self.height, &self.rgba8)
    }
}

/// Pre-compiled material + SM5 pixel shader + evaluated `cb2` ready for SIMD tile execution.
#[derive(Clone, Debug)]
pub struct CompiledSlateMaterial {
    pub info: MaterialInfo,
    pub pixel_shader: DxbcShader,
    pub compiled_ps: CompiledSm5Shader,
    pub compiled_ps_dynamic: CompiledSm5Shader,
    pub default_cb2: Vec<[f32; 4]>,
    pub default_texture_slots: Vec<String>,
    pub default_slot_clamps: Vec<(bool, bool)>,
    pub is_additive: bool,
}

/// Pre-compiled per-widget draw state (`(blueprint_id, export_index, rarity, mode)`) eliminating
/// all render-time `cb2` evaluation and SM5 shader compilation.
#[derive(Clone, Debug)]
pub struct PrecompiledWidgetDraw {
    pub compiled_ps: CompiledSm5Shader,
    pub cb2: Vec<[f32; 4]>,
    pub slot_paths: Vec<String>,
    pub slot_clamps: Vec<(bool, bool)>,
    pub t1_override_slots: Vec<usize>,
    pub vertex_color_linear: [f32; 4],
    pub is_additive: bool,
}

/// In-memory cache of UMG blueprints, compiled SM5 shaders, materials, particle emitters, and
/// decoded textures required to render any Tartarus monster cell at `T = 0`.
pub struct CellRenderContext {
    pub board_cell_umg: UmgBlueprint,
    pub cell_umg: UmgBlueprint,
    pub materials: BTreeMap<String, CompiledSlateMaterial>,
    pub particle_assets: BTreeMap<String, UiParticleEmitterAsset>,
    pub textures: BTreeMap<String, LinearTexture2D>,
    pub widget_draws: BTreeMap<(u8, usize, u8, u8), PrecompiledWidgetDraw>,
    pub default_particles: BTreeMap<usize, Vec<SimulatedUiParticle>>,
    pub white_fallback: LinearTexture2D,
}

#[derive(Clone, Copy, Debug)]
struct QuadRect {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    clip_x0: f32,
    clip_y0: f32,
    clip_x1: f32,
    clip_y1: f32,
}

impl QuadRect {
    fn unclipped(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        assert!(x0 <= x1 && y0 <= y1);
        Self {
            x0,
            y0,
            x1,
            y1,
            clip_x0: f32::NEG_INFINITY,
            clip_y0: f32::NEG_INFINITY,
            clip_x1: f32::INFINITY,
            clip_y1: f32::INFINITY,
        }
    }
}

fn find_asset_base_in_vfs(vfs: &PakVfs, path_or_name: &str) -> String {
    assert!(!path_or_name.is_empty());
    let base = game_path_to_virtual_path(path_or_name)
        .trim_end_matches(".uasset")
        .trim_end_matches(".uexp")
        .to_string();
    assert!(!base.is_empty());
    if vfs.contains_path(&format!("{base}.uasset")) {
        return base;
    }

    let fname = base.rsplit('/').next().unwrap_or(&base);
    for prefix in [
        "UE_game/Content/Art/UIV2/S16Gameplay/BluePrint",
        "UE_game/Content/Art/UIV2/S16Gameplay/UI",
        "UE_game/Content/Art/UIV2/S16Gameplay/UI/TexturesAtlas/Atlas",
        "UE_game/Content/Art/UIV2/S16Gameplay/UIEffect/S16BoardCell/Material",
        "UE_game/Content/Art/UIV2/S16Gameplay/UIEffect/S16BoardCell/Texture",
        "UE_game/Content/Art/UIV2/S16Gameplay/UIEffect/S16BoardCell/Particle",
    ] {
        let candidate = format!("{prefix}/{fname}");
        if vfs.contains_path(&format!("{candidate}.uasset")) {
            return candidate;
        }
    }

    let target_file_lower = format!("{}.uasset", fname.to_ascii_lowercase());
    let hits = vfs.collect_entries(
        |dir_lower| dir_lower.contains("s16") || dir_lower.contains("uiv2"),
        |_, file_lower| file_lower == target_file_lower,
    );
    if let Some((vpath, _)) = hits.first() {
        return vpath.trim_end_matches(".uasset").to_string();
    }
    base
}

fn load_texture_or_sprite(vfs: &PakVfs, vpath: &str) -> Result<LinearTexture2D, ExtractError> {
    assert!(!vpath.is_empty());
    let resolved = find_asset_base_in_vfs(vfs, vpath);
    if let Ok(sprite) = vfs.read_paper_sprite(&resolved) {
        let atlas_path = find_asset_base_in_vfs(vfs, &sprite.atlas_virtual_path);
        let atlas = vfs.decode_texture(&atlas_path)?;
        let cropped = crop_rgba8(
            &atlas.rgba8,
            atlas.width,
            atlas.height,
            sprite.uv_x as usize,
            sprite.uv_y as usize,
            sprite.width as usize,
            sprite.height as usize,
        );
        let sub_tex = tartarus_extract::DecodedTexture {
            width: sprite.width as usize,
            height: sprite.height as usize,
            format: atlas.format,
            num_mips: 1,
            srgb: atlas.srgb,
            clamp_x: true,
            clamp_y: true,
            names: atlas.names,
            rgba8: cropped,
        };
        return Ok(LinearTexture2D::from_decoded(&sub_tex));
    }
    let decoded = vfs.decode_texture(&resolved)?;
    Ok(LinearTexture2D::from_decoded(&decoded))
}

fn extract_ps_shaders_for_hash(
    vfs: &PakVfs,
    archives: &[(
        tartarus_extract::PakEntryRecord,
        tartarus_extract::ShaderArchiveIndex,
    )],
    hash: &[u8; 20],
) -> Result<Vec<DxbcShader>, ExtractError> {
    assert!(!archives.is_empty());
    for (rec, arch) in archives {
        let Some(&sm_idx) = arch.map_hash_to_index.get(hash) else {
            continue;
        };
        let sm = &arch.shader_map_entries[sm_idx];
        let mut ps_list = Vec::new();
        for k in 0..(sm.num_shaders as usize) {
            let shader_idx = arch.shader_indices[(sm.shader_indices_offset as usize) + k] as usize;
            let entry = &arch.shader_entries[shader_idx];
            if entry.frequency != tartarus_extract::ShaderStage::Pixel {
                continue;
            }
            let blob = vfs.extract_shader_entry(rec, arch, shader_idx)?;
            if !blob.dxbc_bytes.is_empty()
                && let Ok(parsed) = parse_dxbc(&blob.dxbc_bytes)
            {
                ps_list.push(parsed);
            }
        }
        if !ps_list.is_empty() {
            return Ok(ps_list);
        }
    }
    Err(ExtractError::ShaderFormat(
        "ShaderMapHash not found in any ShaderArchive".to_string(),
    ))
}

fn build_tex_slot_meta(
    slot_paths: &[String],
    slot_clamps: &[(bool, bool)],
    dynamic_slots: &[usize],
    textures: &BTreeMap<String, LinearTexture2D>,
    fallback: &LinearTexture2D,
) -> Vec<TexSlotMeta> {
    assert!(slot_paths.len() <= 32);
    let mut out = Vec::with_capacity(slot_paths.len());
    for (i, path) in slot_paths.iter().enumerate() {
        let clamp_i = slot_clamps.get(i).copied().unwrap_or((false, false));
        if dynamic_slots.contains(&i) {
            out.push(TexSlotMeta {
                canonical_slot: i as u8,
                clamp_u: clamp_i.0,
                clamp_v: clamp_i.1,
                has_zeros_mask: 0x0F,
                zero_frac: [64; 4],
                const_val: [None; 4],
                eq_r: [false; 4],
            });
            continue;
        }
        let mut canonical = i as u8;
        for (j, prev_path) in slot_paths[..i].iter().enumerate() {
            if !dynamic_slots.contains(&j) && prev_path == path {
                canonical = j as u8;
                break;
            }
        }
        let clamp_c = slot_clamps
            .get(canonical as usize)
            .copied()
            .unwrap_or(clamp_i);
        let tex = textures.get(path).unwrap_or(fallback);
        out.push(tex.slot_meta(canonical, clamp_c.0, clamp_c.1));
    }
    assert_eq!(out.len(), slot_paths.len());
    out
}

type ShaderDedupCache = BTreeMap<(String, Vec<[u32; 4]>, [u32; 4], bool), CompiledSm5Shader>;

impl CellRenderContext {
    /// Loads `S16BoardCell`, `S16Cell`, all referenced `UI_MI_V2_S16BoardCell_*` materials,
    /// `UI_P_V2_S16BoardCell_*` particle emitters, their compiled SM5 pixel shaders, and all
    /// portrait/glow textures from `PakVfs`.
    pub fn load(vfs: &PakVfs) -> Result<Self, ExtractError> {
        let board_cell_path = find_asset_base_in_vfs(vfs, "S16BoardCell");
        let cell_path = find_asset_base_in_vfs(vfs, "S16Cell");
        let board_cell_umg = vfs.read_umg_blueprint(&board_cell_path)?;
        let cell_umg = vfs.read_umg_blueprint(&cell_path)?;
        assert!(!board_cell_umg.widgets.is_empty());
        assert!(!cell_umg.widgets.is_empty());

        let archives = vfs.load_shader_archive_indices()?;
        let mut ctx = Self {
            board_cell_umg,
            cell_umg,
            materials: BTreeMap::new(),
            particle_assets: BTreeMap::new(),
            textures: BTreeMap::new(),
            widget_draws: BTreeMap::new(),
            default_particles: BTreeMap::new(),
            white_fallback: LinearTexture2D::solid_rgba([1.0, 1.0, 1.0, 1.0]),
        };
        ctx.preload_umg_assets(vfs, &archives)?;
        ctx.preload_portraits(vfs)?;
        ctx.precompile_widget_draws();
        ctx.precompute_default_particles();
        Ok(ctx)
    }

    fn compile_and_insert_material(
        &mut self,
        vfs: &PakVfs,
        archives: &[(
            tartarus_extract::PakEntryRecord,
            tartarus_extract::ShaderArchiveIndex,
        )],
        vpath: &str,
    ) -> Result<(), ExtractError> {
        assert!(!vpath.is_empty());
        if self.materials.contains_key(vpath) {
            return Ok(());
        }
        if let Ok(mat_info) = vfs.read_material_info(vpath)
            && let Some(hash) = mat_info.shader_map_hash
        {
            let ps_list = extract_ps_shaders_for_hash(vfs, archives, &hash)?;
            let num_tex = mat_info
                .uniform_expression_set
                .as_ref()
                .map_or(0, |u| u.tex2d_params.len());
            let ps_idx = select_slate_default_ps(&ps_list, num_tex).unwrap_or(0);
            let pixel_shader = ps_list[ps_idx].clone();
            let default_cb2 = mat_info.evaluate_uniform_buffer(&[], &[]);
            let default_texture_slots = mat_info.resolve_texture_slots(&[]);
            let default_slot_clamps = mat_info.resolve_slot_uv_clamps(&[]);
            for tpath in &default_texture_slots {
                if !tpath.is_empty()
                    && !self.textures.contains_key(tpath)
                    && let Ok(tex) = load_texture_or_sprite(vfs, tpath)
                {
                    self.textures.insert(tpath.clone(), tex);
                }
            }
            let tex_meta = build_tex_slot_meta(
                &default_texture_slots,
                &default_slot_clamps,
                &[],
                &self.textures,
                &self.white_fallback,
            );
            let compiled_ps = compile_sm5_shader_specialized(
                &pixel_shader,
                ShaderSpecialization {
                    static_cb2: Some(&default_cb2),
                    static_v1: None,
                    tex_meta: Some(&tex_meta),
                },
            );
            let compiled_ps_dynamic = compile_sm5_shader(&pixel_shader);
            let is_additive = mat_info
                .blend_mode
                .as_deref()
                .is_some_and(|b| b.contains("Additive"));
            self.materials.insert(
                vpath.to_string(),
                CompiledSlateMaterial {
                    info: mat_info,
                    pixel_shader,
                    compiled_ps,
                    compiled_ps_dynamic,
                    default_cb2,
                    default_texture_slots,
                    default_slot_clamps,
                    is_additive,
                },
            );
        }
        Ok(())
    }

    fn preload_umg_assets(
        &mut self,
        vfs: &PakVfs,
        archives: &[(
            tartarus_extract::PakEntryRecord,
            tartarus_extract::ShaderArchiveIndex,
        )],
    ) -> Result<(), ExtractError> {
        assert!(!archives.is_empty());
        let all_paths: Vec<String> = self
            .board_cell_umg
            .widgets
            .iter()
            .chain(self.cell_umg.widgets.iter())
            .filter_map(|w| w.soft_object_path.as_deref())
            .map(game_path_to_virtual_path)
            .collect();

        for vpath in all_paths {
            if self.materials.contains_key(&vpath)
                || self.textures.contains_key(&vpath)
                || self.particle_assets.contains_key(&vpath)
            {
                continue;
            }
            if vpath.contains("/Material/") || vpath.contains("UI_MI_") {
                self.compile_and_insert_material(vfs, archives, &vpath)?;
            } else if vpath.contains("/Particle/") || vpath.contains("UI_P_") {
                if let Ok(p_asset) = vfs.read_ui_particle_asset(&vpath) {
                    if let Some(mat_path) = p_asset.resource_material_path.clone() {
                        self.compile_and_insert_material(vfs, archives, &mat_path)?;
                    }
                    self.particle_assets.insert(vpath, p_asset);
                }
            } else if let Ok(tex) = load_texture_or_sprite(vfs, &vpath) {
                self.textures.insert(vpath, tex);
            }
        }
        assert!(!self.materials.is_empty());
        Ok(())
    }

    fn preload_portraits(&mut self, vfs: &PakVfs) -> Result<(), ExtractError> {
        for code in [
            "045", "046", "047", "048", "049", "050", "101", "102", "103", "104",
        ] {
            let vpath = resolve_portrait_virtual_path(code);
            if let std::collections::btree_map::Entry::Vacant(e) = self.textures.entry(vpath) {
                let tex = load_texture_or_sprite(vfs, e.key())?;
                e.insert(tex);
            }
        }
        assert!(self.textures.len() >= 10);
        Ok(())
    }

    fn build_precompiled_draw(
        &self,
        widget: &UmgWidgetNode,
        opacity_mul: f32,
        scalar_overrides: &[(&str, f32)],
        has_t1_override: bool,
        shader_cache: &mut ShaderDedupCache,
    ) -> Option<PrecompiledWidgetDraw> {
        assert!(opacity_mul >= 0.0);
        if opacity_mul <= 1e-5 {
            return None;
        }
        let soft_path = widget.soft_object_path.as_deref()?;
        let vpath = game_path_to_virtual_path(soft_path);
        let mat = self.materials.get(&vpath)?;
        let c = widget.color_and_opacity;
        let vertex_color_linear = [
            srgb_to_linear_channel(c[0]),
            srgb_to_linear_channel(c[1]),
            srgb_to_linear_channel(c[2]),
            (c[3] * widget.render_opacity * opacity_mul).clamp(0.0, 1.0),
        ];
        if vertex_color_linear[3] <= 1e-5 {
            return None;
        }
        let is_def = scalar_overrides.is_empty();
        let cb2 = if is_def {
            mat.default_cb2.clone()
        } else {
            mat.info.evaluate_uniform_buffer(scalar_overrides, &[])
        };
        let slot_paths = mat.default_texture_slots.clone();
        let slot_clamps = if is_def {
            mat.default_slot_clamps.clone()
        } else {
            mat.info.resolve_slot_uv_clamps(scalar_overrides)
        };
        let t1_override_slots: Vec<usize> = has_t1_override
            .then(|| mat.info.resolve_texture_slots(&[("T1", "__T1__")]))
            .into_iter()
            .flatten()
            .enumerate()
            .filter_map(|(i, p)| (p == "__T1__").then_some(i))
            .collect();
        let cb2_bits: Vec<[u32; 4]> = cb2.iter().map(|v| v.map(f32::to_bits)).collect();
        let v1_bits = vertex_color_linear.map(f32::to_bits);
        let cache_key = (vpath, cb2_bits, v1_bits, has_t1_override);
        let compiled_ps = if let Some(cached) = shader_cache.get(&cache_key) {
            cached.clone()
        } else {
            let tex_meta = build_tex_slot_meta(
                &slot_paths,
                &slot_clamps,
                &t1_override_slots,
                &self.textures,
                &self.white_fallback,
            );
            let compiled = compile_sm5_shader_specialized(
                &mat.pixel_shader,
                ShaderSpecialization {
                    static_cb2: Some(&cb2),
                    static_v1: Some(vertex_color_linear),
                    tex_meta: Some(&tex_meta),
                },
            );
            shader_cache.insert(cache_key, compiled.clone());
            compiled
        };
        Some(PrecompiledWidgetDraw {
            compiled_ps,
            cb2,
            slot_paths,
            slot_clamps,
            t1_override_slots,
            vertex_color_linear,
            is_additive: mat.is_additive,
        })
    }

    fn precompile_widget_draws(&mut self) {
        assert!(!self.cell_umg.widgets.is_empty());
        let mut shader_cache = BTreeMap::new();
        let mut draws = BTreeMap::new();
        let pic_opt = self
            .cell_umg
            .widgets
            .iter()
            .find(|w| w.name == "Img_S16Board_Cell_MonsterPic");

        for rarity in RarityTier::ALL {
            for mode in [RarityMode::Shao, RarityMode::Quan] {
                let anim_opt = self.find_animation_t0(rarity, mode);
                for suffix in ["D", "Up"] {
                    let panel_name =
                        format!("{}_{}_{suffix}", rarity.uasset_prefix(), mode.suffix());
                    let Some(panel) = self.cell_umg.widgets.iter().find(|w| w.name == panel_name)
                    else {
                        continue;
                    };
                    for child in self.cell_umg.widgets.iter().filter(|w| {
                        w.parent_export_index == panel.export_index && w.class_name == "UIImage"
                    }) {
                        let opacity_mul = anim_opt
                            .and_then(|a| a.render_opacities.get(&child.name).copied())
                            .unwrap_or(1.0);
                        let mut scalar_overrides = Vec::new();
                        if let Some(anim) = anim_opt
                            && let Some(params) = anim.material_scalars.get(&child.name)
                        {
                            for (k, v) in params {
                                scalar_overrides.push((k.as_str(), *v));
                            }
                        }
                        if let Some(pre) = self.build_precompiled_draw(
                            child,
                            opacity_mul,
                            &scalar_overrides,
                            false,
                            &mut shader_cache,
                        ) {
                            draws.insert((1, child.export_index, rarity as u8, mode as u8), pre);
                        }
                    }
                }
                if let Some(pic_w) = pic_opt {
                    let mut scalar_overrides = vec![("T1_U_DistortStr", 0.0f32)];
                    if let Some(anim) = anim_opt
                        && let Some(params) =
                            anim.material_scalars.get("Img_S16Board_Cell_MonsterPic")
                    {
                        for (k, v) in params {
                            scalar_overrides.push((k.as_str(), *v));
                        }
                    }
                    if let Some(pre) = self.build_precompiled_draw(
                        pic_w,
                        1.0,
                        &scalar_overrides,
                        true,
                        &mut shader_cache,
                    ) {
                        draws.insert((1, pic_w.export_index, rarity as u8, mode as u8), pre);
                    }
                }
            }
        }
        assert!(!draws.is_empty());
        self.widget_draws = draws;
    }

    fn precompute_default_particles(&mut self) {
        assert!(!self.cell_umg.widgets.is_empty());
        let mut cache = BTreeMap::new();
        for w in self
            .cell_umg
            .widgets
            .iter()
            .filter(|w| w.class_name == "UIParticleEmitter")
        {
            let Some(raw_path) = w
                .particle_asset_path
                .as_deref()
                .or(w.soft_object_path.as_deref())
            else {
                continue;
            };
            let p_vpath = game_path_to_virtual_path(raw_path);
            if let Some(emitter) = self.particle_assets.get(&p_vpath) {
                let widget_seed =
                    DEFAULT_PARTICLE_SEED ^ ((w.export_index as u64).wrapping_mul(0x9E37_79B9));
                cache.insert(w.export_index, emitter.simulate_prewarm_t0(widget_seed));
            }
        }
        assert!(!cache.is_empty() || self.particle_assets.is_empty());
        self.default_particles = cache;
    }

    /// Renders a single monster cell at `T = 0` according to `req`.
    pub fn render_cell(&self, req: &CellRenderRequest) -> Result<RenderedImage, ExtractError> {
        assert!(req.scale > 0.0 && req.scale <= 16.0);
        let width = ((BASE_CANVAS_WIDTH as f32) * req.scale).round().max(16.0) as u32;
        let height = ((BASE_CANVAS_HEIGHT as f32) * req.scale).round().max(16.0) as u32;
        let mut canvas = init_srgb_canvas(width, height, req.background);

        // Origin: center `(128, 160)` of the `256x320` reference canvas maps to `(0, +30)` in
        // `S16BoardCell` space (the exact center of `S16Cell` and `UIImage_Shadow`).
        // Therefore, widgets directly in `S16BoardCell` use `parent_off = (0.0, -30.0)`.
        let cell_scale = 0.75f32;
        let board_parent_off_x = 0.0f32;
        let board_parent_off_y = -30.0f32;

        // 1. Optional `UIImage_Shadow` from `S16BoardCell` (`(2, 30)`, `148x180`, scale `0.8`, `ZOrder = 0`).
        if req.include_shadow
            && let Some(shadow_w) = self
                .board_cell_umg
                .widgets
                .iter()
                .find(|w| w.name == "UIImage_Shadow")
        {
            let rect = widget_rect_in_canvas(
                shadow_w,
                board_parent_off_x,
                board_parent_off_y,
                1.0,
                req.scale,
            );
            self.draw_widget_quad(&mut canvas, width, height, shadow_w, rect, 1.0, &[], &[]);
        }

        // 2. Back rarity layer: `<Color>_<Shao|Quan>_D` inside `FX_Di_D` (`S16Cell`).
        let back_panel_name = format!("{}_{}_D", req.rarity.uasset_prefix(), req.mode.suffix());
        self.draw_panel_children(
            &mut canvas,
            width,
            height,
            &back_panel_name,
            req,
            cell_scale,
        );

        // 3. Middle Monster Portrait layer: `Img_S16Board_Cell_MonsterPic` clipped to `HUDIconSizeBox`.
        self.draw_monster_portrait(&mut canvas, width, height, req, cell_scale);

        // 4. Optional devour shield background: `UUIImage_Animal_Eat_Bg01` (`80x52` at `(0, -55.5)` in `S16Cell.Di`).
        if req.show_eat_icon
            && let Some(eat_bg_w) = self
                .cell_umg
                .widgets
                .iter()
                .find(|w| w.name == "UUIImage_Animal_Eat_Bg01")
        {
            let rect = widget_rect_in_canvas(eat_bg_w, 0.0, 0.0, cell_scale, req.scale);
            self.draw_widget_quad(&mut canvas, width, height, eat_bg_w, rect, 1.0, &[], &[]);
        }

        // 5. Front rarity layer: `<Color>_<Shao|Quan>_Up` inside `FX_Di_Up` (`S16Cell`).
        let front_panel_name = format!("{}_{}_Up", req.rarity.uasset_prefix(), req.mode.suffix());
        self.draw_panel_children(
            &mut canvas,
            width,
            height,
            &front_panel_name,
            req,
            cell_scale,
        );

        // 6. Optional devour crown/fangs icon: `Img_S16Board_Cell_DevourMark` (`52x52` at `(0, -11)` in `S16BoardCell`, `ZOrder = 5`).
        if req.show_eat_icon
            && let Some(devour_mark_w) = self
                .board_cell_umg
                .widgets
                .iter()
                .find(|w| w.name == "Img_S16Board_Cell_DevourMark")
        {
            let rect = widget_rect_in_canvas(
                devour_mark_w,
                board_parent_off_x,
                board_parent_off_y,
                1.0,
                req.scale,
            );
            self.draw_widget_quad(
                &mut canvas,
                width,
                height,
                devour_mark_w,
                rect,
                1.0,
                &[],
                &[],
            );
        }

        let rgba8 = finalize_canvas_to_srgb8(&canvas, req.background);
        assert_eq!(rgba8.len(), (width * height * 4) as usize);
        Ok(RenderedImage {
            width,
            height,
            rgba8,
        })
    }

    fn find_animation_t0(
        &self,
        rarity: RarityTier,
        mode: RarityMode,
    ) -> Option<&UmgAnimationStateT0> {
        let target = format!("Anim_{}_{}_INST", rarity.uasset_prefix(), mode.suffix());
        self.cell_umg
            .animations_t0
            .iter()
            .find(|a| a.name == target)
    }

    fn draw_panel_children(
        &self,
        canvas: &mut [[f32; 4]],
        width: u32,
        height: u32,
        panel_name: &str,
        req: &CellRenderRequest,
        cell_scale: f32,
    ) {
        assert!(!panel_name.is_empty());
        let Some(panel) = self.cell_umg.widgets.iter().find(|w| w.name == panel_name) else {
            return;
        };
        let anim_opt = self.find_animation_t0(req.rarity, req.mode);
        let mut children: Vec<&UmgWidgetNode> = self
            .cell_umg
            .widgets
            .iter()
            .filter(|w| w.parent_export_index == panel.export_index)
            .collect();
        children.sort_by_key(|w| (w.z_order, w.slot_order));

        for child in children {
            let opacity_mul = anim_opt
                .and_then(|a| a.render_opacities.get(&child.name).copied())
                .unwrap_or(1.0);
            if opacity_mul <= 1e-5 {
                continue;
            }
            if child.class_name == "UIImage" {
                let rect = widget_rect_in_canvas(child, 0.0, 0.0, cell_scale, req.scale);
                if let Some(pre) = self.widget_draws.get(&(
                    1,
                    child.export_index,
                    req.rarity as u8,
                    req.mode as u8,
                )) {
                    self.draw_precompiled_widget(canvas, width, height, rect, pre, None);
                } else {
                    let mut scalar_overrides = Vec::new();
                    if let Some(anim) = anim_opt
                        && let Some(params) = anim.material_scalars.get(&child.name)
                    {
                        for (k, v) in params {
                            scalar_overrides.push((k.as_str(), *v));
                        }
                    }
                    self.draw_widget_quad(
                        canvas,
                        width,
                        height,
                        child,
                        rect,
                        opacity_mul,
                        &scalar_overrides,
                        &[],
                    );
                }
            } else if child.class_name == "UIParticleEmitter" && req.include_particles {
                self.draw_particle_emitter_widget(
                    canvas,
                    width,
                    height,
                    child,
                    opacity_mul,
                    cell_scale,
                    req.scale,
                    req.particle_seed,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_particle_emitter_widget(
        &self,
        canvas: &mut [[f32; 4]],
        width: u32,
        height: u32,
        widget: &UmgWidgetNode,
        opacity_mul: f32,
        cell_scale: f32,
        dpi_scale: f32,
        base_seed: u64,
    ) {
        let Some(raw_path) = widget
            .particle_asset_path
            .as_deref()
            .or(widget.soft_object_path.as_deref())
        else {
            return;
        };
        let p_vpath = game_path_to_virtual_path(raw_path);
        let Some(emitter) = self.particle_assets.get(&p_vpath) else {
            return;
        };
        let Some(mat_vpath) = emitter.resource_material_path.as_deref() else {
            return;
        };
        let Some(mat) = self.materials.get(mat_vpath) else {
            return;
        };
        let bound_tex: Vec<&LinearTexture2D> = mat
            .default_texture_slots
            .iter()
            .map(|p| self.textures.get(p).unwrap_or(&self.white_fallback))
            .collect();

        let dynamic_particles;
        let particles: &[SimulatedUiParticle] = if base_seed == DEFAULT_PARTICLE_SEED
            && let Some(cached) = self.default_particles.get(&widget.export_index)
        {
            cached.as_slice()
        } else {
            let widget_seed = base_seed ^ ((widget.export_index as u64).wrapping_mul(0x9E37_79B9));
            dynamic_particles = emitter.simulate_prewarm_t0(widget_seed);
            dynamic_particles.as_slice()
        };
        let emit_off_x = widget.anchor.offset_left + widget.render_translation[0];
        let emit_off_y = widget.anchor.offset_top + widget.render_translation[1];

        for p in particles {
            let cx = ((BASE_CANVAS_WIDTH as f32) * 0.5 + (emit_off_x + p.local_x) * cell_scale)
                * dpi_scale;
            let cy = ((BASE_CANVAS_HEIGHT as f32) * 0.5 + (emit_off_y + p.local_y) * cell_scale)
                * dpi_scale;
            let half_sz = 0.5 * p.size * cell_scale * dpi_scale;
            let rect = QuadRect::unclipped(cx - half_sz, cy - half_sz, cx + half_sz, cy + half_sz);
            let vertex_color_linear = [
                p.linear_rgba[0],
                p.linear_rgba[1],
                p.linear_rgba[2],
                (p.linear_rgba[3] * widget.render_opacity * opacity_mul).clamp(0.0, 1.0),
            ];
            rasterize_material_quad(
                canvas,
                width,
                height,
                rect,
                &mat.compiled_ps,
                vertex_color_linear,
                &mat.default_cb2,
                &bound_tex,
                &mat.default_slot_clamps,
                mat.is_additive,
            );
        }
    }

    fn draw_monster_portrait(
        &self,
        canvas: &mut [[f32; 4]],
        width: u32,
        height: u32,
        req: &CellRenderRequest,
        cell_scale: f32,
    ) {
        let Some(size_box) = self
            .cell_umg
            .widgets
            .iter()
            .find(|w| w.name == "HUDIconSizeBox")
        else {
            return;
        };
        let Some(pic_w) = self
            .cell_umg
            .widgets
            .iter()
            .find(|w| w.name == "Img_S16Board_Cell_MonsterPic")
        else {
            return;
        };
        let box_rect = widget_rect_in_canvas(size_box, 0.0, 0.0, cell_scale, req.scale);
        let rect = QuadRect {
            clip_x0: box_rect.x0,
            clip_y0: box_rect.y0,
            clip_x1: box_rect.x1,
            clip_y1: box_rect.y1,
            ..box_rect
        };
        let portrait_vpath = resolve_portrait_virtual_path(&req.portrait_selector);
        if let Some(pre) =
            self.widget_draws
                .get(&(1, pic_w.export_index, req.rarity as u8, req.mode as u8))
        {
            self.draw_precompiled_widget(
                canvas,
                width,
                height,
                rect,
                pre,
                Some(portrait_vpath.as_str()),
            );
            return;
        }
        let mut scalar_overrides = vec![("T1_U_DistortStr", 0.0f32)];
        if let Some(anim) = self.find_animation_t0(req.rarity, req.mode)
            && let Some(params) = anim.material_scalars.get("Img_S16Board_Cell_MonsterPic")
        {
            for (k, v) in params {
                scalar_overrides.push((k.as_str(), *v));
            }
        }
        self.draw_widget_quad(
            canvas,
            width,
            height,
            pic_w,
            rect,
            1.0,
            &scalar_overrides,
            &[("T1", portrait_vpath.as_str())],
        );
    }

    fn draw_precompiled_widget(
        &self,
        canvas: &mut [[f32; 4]],
        width: u32,
        height: u32,
        rect: QuadRect,
        pre: &PrecompiledWidgetDraw,
        t1_override: Option<&str>,
    ) {
        assert_eq!(canvas.len(), (width * height) as usize);
        let mut bound_tex: Vec<&LinearTexture2D> = pre
            .slot_paths
            .iter()
            .map(|p| self.textures.get(p).unwrap_or(&self.white_fallback))
            .collect();
        if let Some(override_path) = t1_override {
            let tex = self
                .textures
                .get(override_path)
                .unwrap_or(&self.white_fallback);
            for &slot_idx in &pre.t1_override_slots {
                if slot_idx < bound_tex.len() {
                    bound_tex[slot_idx] = tex;
                }
            }
        }
        rasterize_material_quad(
            canvas,
            width,
            height,
            rect,
            &pre.compiled_ps,
            pre.vertex_color_linear,
            &pre.cb2,
            &bound_tex,
            &pre.slot_clamps,
            pre.is_additive,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_widget_quad(
        &self,
        canvas: &mut [[f32; 4]],
        width: u32,
        height: u32,
        widget: &UmgWidgetNode,
        rect: QuadRect,
        opacity_mul: f32,
        scalar_overrides: &[(&str, f32)],
        tex_overrides: &[(&str, &str)],
    ) {
        assert_eq!(canvas.len(), (width * height) as usize);
        let Some(soft_path) = widget.soft_object_path.as_deref() else {
            return;
        };
        let vpath = game_path_to_virtual_path(soft_path);
        let c = widget.color_and_opacity;
        let vertex_color_linear = [
            srgb_to_linear_channel(c[0]),
            srgb_to_linear_channel(c[1]),
            srgb_to_linear_channel(c[2]),
            (c[3] * widget.render_opacity * opacity_mul).clamp(0.0, 1.0),
        ];
        if vertex_color_linear[3] <= 1e-5 {
            return;
        }

        if let Some(mat) = self.materials.get(&vpath) {
            let cb2_owned;
            let cb2: &[[f32; 4]] = if scalar_overrides.is_empty() {
                &mat.default_cb2
            } else {
                cb2_owned = mat.info.evaluate_uniform_buffer(scalar_overrides, &[]);
                &cb2_owned
            };
            let slots_owned;
            let slot_paths: &[String] = if tex_overrides.is_empty() {
                &mat.default_texture_slots
            } else {
                slots_owned = mat.info.resolve_texture_slots(tex_overrides);
                &slots_owned
            };
            let clamps_owned;
            let slot_clamps: &[(bool, bool)] = if scalar_overrides.is_empty() {
                &mat.default_slot_clamps
            } else {
                clamps_owned = mat.info.resolve_slot_uv_clamps(scalar_overrides);
                &clamps_owned
            };
            let bound_tex: Vec<&LinearTexture2D> = slot_paths
                .iter()
                .map(|p| self.textures.get(p).unwrap_or(&self.white_fallback))
                .collect();
            let specialized_ps;
            let active_ps = if scalar_overrides.is_empty() {
                &mat.compiled_ps
            } else {
                specialized_ps = compile_sm5_shader_with_cb2(&mat.pixel_shader, Some(cb2));
                &specialized_ps
            };
            rasterize_material_quad(
                canvas,
                width,
                height,
                rect,
                active_ps,
                vertex_color_linear,
                cb2,
                &bound_tex,
                slot_clamps,
                mat.is_additive,
            );
        } else if let Some(tex) = self.textures.get(&vpath) {
            rasterize_sprite_quad(canvas, width, height, rect, tex, vertex_color_linear);
        }
    }
}

fn widget_rect_in_canvas(
    w: &UmgWidgetNode,
    parent_off_x: f32,
    parent_off_y: f32,
    parent_scale: f32,
    dpi_scale: f32,
) -> QuadRect {
    assert!(dpi_scale > 0.0);
    let base_w = if w.auto_size {
        w.brush_image_size[0]
    } else {
        w.anchor.offset_right
    };
    let base_h = if w.auto_size {
        w.brush_image_size[1]
    } else {
        w.anchor.offset_bottom
    };
    let local_x0 = w.anchor.offset_left - w.anchor.align_x * base_w + w.render_translation[0];
    let local_y0 = w.anchor.offset_top - w.anchor.align_y * base_h + w.render_translation[1];
    let local_cx = local_x0 + 0.5 * base_w;
    let local_cy = local_y0 + 0.5 * base_h;
    let half_w = 0.5 * base_w * w.render_scale[0].abs();
    let half_h = 0.5 * base_h * w.render_scale[1].abs();

    let cx = (BASE_CANVAS_WIDTH as f32) * 0.5 + (parent_off_x + local_cx * parent_scale);
    let cy = (BASE_CANVAS_HEIGHT as f32) * 0.5 + (parent_off_y + local_cy * parent_scale);
    let hw = half_w * parent_scale;
    let hh = half_h * parent_scale;

    QuadRect::unclipped(
        (cx - hw) * dpi_scale,
        (cy - hh) * dpi_scale,
        (cx + hw) * dpi_scale,
        (cy + hh) * dpi_scale,
    )
}

#[allow(clippy::too_many_arguments)]
fn rasterize_material_quad(
    canvas: &mut [[f32; 4]],
    width: u32,
    height: u32,
    rect: QuadRect,
    shader: &CompiledSm5Shader,
    vertex_color_linear: [f32; 4],
    cb2: &[[f32; 4]],
    textures: &[&LinearTexture2D],
    tex_clamps: &[(bool, bool)],
    is_additive: bool,
) {
    assert!(width > 0 && height > 0);
    rasterize_compiled_quad(
        canvas,
        width,
        height,
        [rect.x0, rect.y0, rect.x1, rect.y1],
        [rect.clip_x0, rect.clip_y0, rect.clip_x1, rect.clip_y1],
        shader,
        vertex_color_linear,
        cb2,
        textures,
        tex_clamps,
        0.0,
        is_additive,
    );
}

fn rasterize_sprite_quad(
    canvas: &mut [[f32; 4]],
    width: u32,
    height: u32,
    rect: QuadRect,
    tex: &LinearTexture2D,
    vertex_color_linear: [f32; 4],
) {
    assert!(width > 0 && height > 0);
    rasterize_sprite_quad_simd(
        canvas,
        width,
        height,
        [rect.x0, rect.y0, rect.x1, rect.y1],
        [rect.clip_x0, rect.clip_y0, rect.clip_x1, rect.clip_y1],
        tex,
        vertex_color_linear,
    );
}

fn init_srgb_canvas(width: u32, height: u32, bg: BackgroundMode) -> Vec<[f32; 4]> {
    assert!(width > 0 && height > 0);
    let fill = match bg {
        BackgroundMode::Transparent => [0.0, 0.0, 0.0, 0.0],
        BackgroundMode::SolidSrgb([r, g, b]) => {
            [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
        }
    };
    vec![fill; (width * height) as usize]
}

fn finalize_canvas_to_srgb8(canvas: &[[f32; 4]], bg: BackgroundMode) -> Vec<u8> {
    assert!(!canvas.is_empty());
    let mut out = vec![0u8; canvas.len() * 4];
    let zero = f32x8::splat(0.0);
    let one = f32x8::splat(1.0);
    let eps = f32x8::splat(1e-5);
    let scale_255 = f32x8::splat(255.0);
    let is_transparent = matches!(bg, BackgroundMode::Transparent);

    let (px_chunks, px_rem) = canvas.as_chunks::<8>();
    let (out_chunks, out_rem) = out.as_chunks_mut::<32>();
    for (c, dst) in px_chunks.iter().zip(out_chunks.iter_mut()) {
        let r = f32x8::from_array(c.map(|p| p[0]));
        let g = f32x8::from_array(c.map(|p| p[1]));
        let b = f32x8::from_array(c.map(|p| p[2]));
        let a_in = f32x8::from_array(c.map(|p| p[3]));

        if is_transparent {
            let max_rgb = r.simd_max(g).simd_max(b).simd_max(zero).simd_min(one);
            let a = a_in.simd_max(max_rgb).simd_max(zero).simd_min(one);
            let active = a.simd_gt(eps);
            if !active.any() {
                continue;
            }
            let inv_a = active.select(one / a.simd_max(eps), zero);
            let r8 = ((r * inv_a).simd_max(zero).simd_min(one) * scale_255)
                .round()
                .cast::<u32>()
                .to_array();
            let g8 = ((g * inv_a).simd_max(zero).simd_min(one) * scale_255)
                .round()
                .cast::<u32>()
                .to_array();
            let b8 = ((b * inv_a).simd_max(zero).simd_min(one) * scale_255)
                .round()
                .cast::<u32>()
                .to_array();
            let a8 = active
                .select(a * scale_255, zero)
                .round()
                .cast::<u32>()
                .to_array();
            for i in 0..8 {
                let off = i * 4;
                dst[off] = r8[i] as u8;
                dst[off + 1] = g8[i] as u8;
                dst[off + 2] = b8[i] as u8;
                dst[off + 3] = a8[i] as u8;
            }
        } else {
            let r8 = (r.simd_max(zero).simd_min(one) * scale_255)
                .round()
                .cast::<u32>()
                .to_array();
            let g8 = (g.simd_max(zero).simd_min(one) * scale_255)
                .round()
                .cast::<u32>()
                .to_array();
            let b8 = (b.simd_max(zero).simd_min(one) * scale_255)
                .round()
                .cast::<u32>()
                .to_array();
            for i in 0..8 {
                let off = i * 4;
                dst[off] = r8[i] as u8;
                dst[off + 1] = g8[i] as u8;
                dst[off + 2] = b8[i] as u8;
                dst[off + 3] = 255;
            }
        }
    }

    for (px, dst) in px_rem.iter().zip(out_rem.as_chunks_mut::<4>().0) {
        let max_rgb = px[0].max(px[1]).max(px[2]).clamp(0.0, 1.0);
        let a = if is_transparent {
            px[3].max(max_rgb).clamp(0.0, 1.0)
        } else {
            1.0
        };
        if a > 1e-5 {
            let inv_a = if is_transparent { 1.0 / a } else { 1.0 };
            dst[0] = ((px[0] * inv_a).clamp(0.0, 1.0) * 255.0).round() as u8;
            dst[1] = ((px[1] * inv_a).clamp(0.0, 1.0) * 255.0).round() as u8;
            dst[2] = ((px[2] * inv_a).clamp(0.0, 1.0) * 255.0).round() as u8;
            dst[3] = (a * 255.0).round() as u8;
        }
    }
    assert_eq!(out.len(), canvas.len() * 4);
    out
}

/// Renders a multi-column, multi-row comparison contact sheet of `rows` × 6 `RarityTier`s.
pub fn render_comparison_sheet(
    ctx: &CellRenderContext,
    rows: &[(String, RarityMode, bool)],
    scale: f32,
    background: BackgroundMode,
) -> Result<RenderedImage, ExtractError> {
    assert!(!rows.is_empty());
    assert!(scale > 0.0 && scale <= 8.0);
    let cell_w = ((BASE_CANVAS_WIDTH as f32) * scale).round().max(16.0) as u32;
    let cell_h = ((BASE_CANVAS_HEIGHT as f32) * scale).round().max(16.0) as u32;
    let cols = RarityTier::ALL.len() as u32;
    let total_w = cell_w * cols;
    let total_h = cell_h * (rows.len() as u32);
    let mut sheet_rgba8 = vec![0u8; (total_w * total_h * 4) as usize];

    for (row_idx, (portrait, mode, eat_icon)) in rows.iter().enumerate() {
        for (col_idx, &rarity) in RarityTier::ALL.iter().enumerate() {
            let req = CellRenderRequest {
                portrait_selector: portrait.clone(),
                rarity,
                mode: *mode,
                show_eat_icon: *eat_icon,
                include_shadow: true,
                include_particles: true,
                particle_seed: DEFAULT_PARTICLE_SEED,
                scale,
                background,
            };
            let cell = ctx.render_cell(&req)?;
            blit_rgba8(
                &mut sheet_rgba8,
                total_w,
                &cell.rgba8,
                cell.width,
                cell.height,
                (col_idx as u32) * cell_w,
                (row_idx as u32) * cell_h,
            );
        }
    }
    assert_eq!(sheet_rgba8.len(), (total_w * total_h * 4) as usize);
    Ok(RenderedImage {
        width: total_w,
        height: total_h,
        rgba8: sheet_rgba8,
    })
}

fn blit_rgba8(
    dst: &mut [u8],
    dst_w: u32,
    src: &[u8],
    src_w: u32,
    src_h: u32,
    dst_x: u32,
    dst_y: u32,
) {
    assert_eq!(src.len(), (src_w * src_h * 4) as usize);
    for y in 0..src_h {
        let src_off = (y * src_w * 4) as usize;
        let dst_off = (((dst_y + y) * dst_w + dst_x) * 4) as usize;
        let row_bytes = (src_w * 4) as usize;
        dst[dst_off..dst_off + row_bytes].copy_from_slice(&src[src_off..src_off + row_bytes]);
    }
}

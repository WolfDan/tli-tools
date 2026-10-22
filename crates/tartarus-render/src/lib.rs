//! 1:1 CPU UMG and DirectX SM5 shader renderer for *Torchlight: Infinite* Season 16 (Tartarus)
//! monster portraits and rarities (`T = 0`).

#![forbid(unsafe_code)]

pub mod compositor;
pub mod sm5_vm;

pub use compositor::{
    BASE_CANVAS_HEIGHT, BASE_CANVAS_WIDTH, BackgroundMode, CellRenderContext, CellRenderRequest,
    CompiledSlateMaterial, DEFAULT_PARTICLE_SEED, RarityMode, RarityTier, RenderedImage,
    render_comparison_sheet, resolve_portrait_virtual_path,
};
pub use sm5_vm::{
    LinearTexture2D, execute_sm5_ps, linear_to_srgb_channel, select_slate_default_ps,
    srgb_to_linear_channel,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_linear_roundtrip() {
        for b in [0u8, 1, 10, 64, 128, 200, 255] {
            let s = b as f32 / 255.0;
            let lin = srgb_to_linear_channel(s);
            let back = linear_to_srgb_channel(lin);
            assert!(
                (s - back).abs() < 1e-3,
                "sRGB roundtrip mismatch for {b}: {s} -> {lin} -> {back}"
            );
        }
    }

    #[test]
    fn test_render_cell_if_game_installed() {
        let Some(game_dir) = tartarus_extract::discover_game_dir() else {
            return;
        };
        let Ok(vfs) =
            tartarus_extract::PakVfs::open_game_dir(&game_dir, tartarus_extract::DEFAULT_AES_KEY)
        else {
            return;
        };
        let ctx = CellRenderContext::load(&vfs).expect("CellRenderContext::load");
        assert!(ctx.materials.len() >= 18);
        assert!(ctx.cell_umg.animations_t0.len() >= 12);

        let req = CellRenderRequest {
            portrait_selector: String::from("mammoth"),
            rarity: RarityTier::Cai,
            mode: RarityMode::Quan,
            show_eat_icon: true,
            include_shadow: true,
            include_particles: true,
            particle_seed: DEFAULT_PARTICLE_SEED,
            scale: 0.5,
            background: BackgroundMode::Transparent,
        };
        let img = ctx.render_cell(&req).expect("render_cell");
        assert_eq!(img.width, 128);
        assert_eq!(img.height, 160);
        let non_zero_pixels = img
            .rgba8
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|px| px[3] > 0)
            .count();
        assert!(non_zero_pixels > 2000);
    }
}

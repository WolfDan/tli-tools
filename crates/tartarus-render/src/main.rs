//! CLI entry point for `tartarus-render` (`cell` and `sheet` commands).

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use tartarus_extract::{DEFAULT_AES_KEY, PakVfs, discover_game_dir, parse_aes_key_hex};
use tartarus_render::{
    BackgroundMode, CellRenderContext, CellRenderRequest, DEFAULT_PARTICLE_SEED, RarityMode,
    RarityTier, render_comparison_sheet,
};

fn print_usage() {
    eprintln!(
        "Usage:
  tartarus-render cell [OPTIONS] -o <OUT.png>
  tartarus-render sheet [OPTIONS] -o <OUT_DIR>

Commands:
  cell    Render a single monster portrait + rarity cell at T=0
  sheet   Render all 6 rarities (Bai..Cai) in Shao and Quan modes + comparison contact sheet

Options:
  --game-dir <PATH>      Path to Torchlight: Infinite directory (auto-discovered if omitted)
  --aes-key <HEX>        AES-256 key hex (defaults to built-in key)
  --portrait <SELECTOR>  Monster portrait: mammoth|lion|wolf|rat|boss|mammoth_free.. (default: mammoth)
  --rarity <TIER>        Rarity tier: bai|blue|zi|cheng|red|cai|201..206 (default: cai)
  --quan                 Use devourable/enhanced rarity glow (Anim_<Color>_Quan) instead of Shao
  --eat-icon             Force devour badge (UUIImage_Animal_Eat_Bg01 + Img_S16Board_Cell_DevourMark)
  --no-devour-badge      Hide devour badge even in --quan mode
  --no-particles         Disable T=0 pre-warmed UIParticleEmitter simulation
  --particle-seed <U64>  Deterministic PRNG seed for pre-warmed particles
  --no-shadow            Omit parent S16BoardCell UIImage_Shadow
  --scale <FACTOR>       Resolution scale factor relative to 256x320 (default: 1.0)
  --height <PX>          Target output height in pixels (overrides --scale)
  --bg <MODE>            Background mode: transparent|dark|black|#RRGGBB (default: transparent)
  -o, --output <PATH>    Output .png file (for `cell`) or directory (for `sheet`)"
    );
}

fn open_vfs(
    game_dir_opt: Option<PathBuf>,
    aes_hex_opt: Option<String>,
) -> Result<PakVfs, Box<dyn std::error::Error>> {
    let game_dir = match game_dir_opt.or_else(discover_game_dir) {
        Some(d) => d,
        None => return Err("Could not auto-discover Torchlight: Infinite game directory".into()),
    };
    let key = match aes_hex_opt {
        Some(h) => parse_aes_key_hex(&h)?,
        None => DEFAULT_AES_KEY,
    };
    assert!(!game_dir.as_os_str().is_empty());
    Ok(PakVfs::open_game_dir(&game_dir, key)?)
}

fn run_cell_cmd(mut args: pico_args::Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let game_dir: Option<PathBuf> = args.opt_value_from_str("--game-dir")?;
    let aes_key: Option<String> = args.opt_value_from_str("--aes-key")?;
    let portrait: String = args
        .opt_value_from_str("--portrait")?
        .unwrap_or_else(|| String::from("mammoth"));
    let rarity_str: String = args
        .opt_value_from_str("--rarity")?
        .unwrap_or_else(|| String::from("cai"));
    let quan = args.contains("--quan");
    let eat_icon = args.contains("--eat-icon");
    let no_devour_badge = args.contains("--no-devour-badge");
    let no_particles = args.contains("--no-particles");
    let particle_seed: u64 = args
        .opt_value_from_str("--particle-seed")?
        .unwrap_or(DEFAULT_PARTICLE_SEED);
    let no_shadow = args.contains("--no-shadow");
    let mut scale: f32 = args.opt_value_from_str("--scale")?.unwrap_or(1.0);
    if let Some(h) = args.opt_value_from_str::<_, u32>("--height")? {
        scale = (h as f32) / 320.0;
    }
    let bg_str: String = args
        .opt_value_from_str("--bg")?
        .unwrap_or_else(|| String::from("transparent"));
    let out_path: PathBuf = args
        .opt_value_from_str(["-o", "--output"])?
        .unwrap_or_else(|| PathBuf::from("cell_render.png"));

    let rarity =
        RarityTier::parse(&rarity_str).ok_or_else(|| format!("Unknown --rarity '{rarity_str}'"))?;
    let background =
        BackgroundMode::parse(&bg_str).ok_or_else(|| format!("Unknown --bg '{bg_str}'"))?;
    assert!(scale > 0.0);

    let req = CellRenderRequest {
        portrait_selector: portrait,
        rarity,
        mode: if quan {
            RarityMode::Quan
        } else {
            RarityMode::Shao
        },
        show_eat_icon: !no_devour_badge && (eat_icon || quan),
        include_shadow: !no_shadow,
        include_particles: !no_particles,
        particle_seed,
        scale,
        background,
    };
    exec_cell_render(game_dir, aes_key, &req, &out_path)
}

fn exec_cell_render(
    game_dir: Option<PathBuf>,
    aes_key: Option<String>,
    req: &CellRenderRequest,
    out_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(req.scale > 0.0);
    assert!(!out_path.as_os_str().is_empty());
    let t0 = Instant::now();
    let vfs = open_vfs(game_dir, aes_key)?;
    let t_mount = t0.elapsed();
    let t1 = Instant::now();
    let ctx = CellRenderContext::load(&vfs)?;
    let t_ctx = t1.elapsed();
    let t2 = Instant::now();
    let img = ctx.render_cell(req)?;
    let t_render = t2.elapsed();
    if let Some(parent) = out_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(out_path, img.to_png_bytes()?)?;
    let t_total = t0.elapsed();
    println!(
        "Rendered {}x{} PNG -> {} \
         [mount={:.3} ms, ctx_load={:.2} ms, render={:.2} ms, total={:.2} ms]",
        img.width,
        img.height,
        out_path.display(),
        t_mount.as_secs_f64() * 1000.0,
        t_ctx.as_secs_f64() * 1000.0,
        t_render.as_secs_f64() * 1000.0,
        t_total.as_secs_f64() * 1000.0,
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_12_portrait_cells(
    ctx: &CellRenderContext,
    portrait: &str,
    no_devour_badge: bool,
    no_particles: bool,
    particle_seed: u64,
    scale: f32,
    background: BackgroundMode,
    out_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(!portrait.is_empty());
    assert!(scale > 0.0);
    for mode in [RarityMode::Shao, RarityMode::Quan] {
        for &rarity in &RarityTier::ALL {
            let req = CellRenderRequest {
                portrait_selector: portrait.to_string(),
                rarity,
                mode,
                show_eat_icon: !no_devour_badge && mode == RarityMode::Quan,
                include_shadow: true,
                include_particles: !no_particles,
                particle_seed,
                scale,
                background,
            };
            let img = ctx.render_cell(&req)?;
            let fname = format!(
                "{}_{}_{}.png",
                portrait,
                rarity.uasset_prefix().to_ascii_lowercase(),
                mode.suffix().to_ascii_lowercase()
            );
            fs::write(out_dir.join(fname), img.to_png_bytes()?)?;
        }
    }
    Ok(())
}

fn run_sheet_cmd(mut args: pico_args::Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let game_dir: Option<PathBuf> = args.opt_value_from_str("--game-dir")?;
    let aes_key: Option<String> = args.opt_value_from_str("--aes-key")?;
    let portrait: String = args
        .opt_value_from_str("--portrait")?
        .unwrap_or_else(|| String::from("mammoth"));
    let no_devour_badge = args.contains("--no-devour-badge");
    let no_particles = args.contains("--no-particles");
    let particle_seed: u64 = args
        .opt_value_from_str("--particle-seed")?
        .unwrap_or(DEFAULT_PARTICLE_SEED);
    let mut scale: f32 = args.opt_value_from_str("--scale")?.unwrap_or(1.0);
    if let Some(h) = args.opt_value_from_str::<_, u32>("--height")? {
        scale = (h as f32) / 320.0;
    }
    let bg_str: String = args
        .opt_value_from_str("--bg")?
        .unwrap_or_else(|| String::from("dark"));
    let out_dir: PathBuf = args
        .opt_value_from_str(["-o", "--output"])?
        .unwrap_or_else(|| PathBuf::from("renders"));

    let background =
        BackgroundMode::parse(&bg_str).ok_or_else(|| format!("Unknown --bg '{bg_str}'"))?;
    assert!(scale > 0.0);

    fs::create_dir_all(&out_dir)?;
    let t0 = Instant::now();
    let vfs = open_vfs(game_dir, aes_key)?;
    let t_mount = t0.elapsed();
    let t1 = Instant::now();
    let ctx = CellRenderContext::load(&vfs)?;
    let t_ctx = t1.elapsed();
    let t2 = Instant::now();
    render_12_portrait_cells(
        &ctx,
        &portrait,
        no_devour_badge,
        no_particles,
        particle_seed,
        scale,
        background,
        &out_dir,
    )?;
    let t_cells12 = t2.elapsed();

    let t3 = Instant::now();
    let quan_badge = !no_devour_badge;
    let rows = vec![
        (String::from("mammoth"), RarityMode::Shao, false),
        (String::from("mammoth"), RarityMode::Quan, quan_badge),
        (String::from("lion"), RarityMode::Shao, false),
        (String::from("wolf"), RarityMode::Quan, quan_badge),
        (String::from("rat"), RarityMode::Shao, false),
        (String::from("mystery"), RarityMode::Quan, quan_badge),
    ];
    let sheet = render_comparison_sheet(&ctx, &rows, scale, background)?;
    let t_sheet = t3.elapsed();
    let sheet_path = out_dir.join("rarity_comparison_sheet.png");
    fs::write(&sheet_path, sheet.to_png_bytes()?)?;
    let t_total = t0.elapsed();
    println!(
        "Rendered 12 individual rarity PNGs + {}x{} comparison sheet -> {}\n\
         [mount={:.3} ms, ctx_load={:.2} ms, 12_cells={:.2} ms, sheet_36_cells={:.2} ms, total={:.2} ms]",
        sheet.width,
        sheet.height,
        sheet_path.display(),
        t_mount.as_secs_f64() * 1000.0,
        t_ctx.as_secs_f64() * 1000.0,
        t_cells12.as_secs_f64() * 1000.0,
        t_sheet.as_secs_f64() * 1000.0,
        t_total.as_secs_f64() * 1000.0,
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["-h", "--help"]) {
        print_usage();
        return Ok(());
    }
    let Some(subcmd) = args.subcommand()? else {
        print_usage();
        return Ok(());
    };
    assert!(!subcmd.is_empty());
    match subcmd.as_str() {
        "cell" => run_cell_cmd(args),
        "sheet" => run_sheet_cmd(args),
        other => {
            print_usage();
            Err(format!("Unknown subcommand '{other}'").into())
        }
    }
}

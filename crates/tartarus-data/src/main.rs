use std::fs;
use std::path::PathBuf;
use tartarus_data::{
    S16GameDatabase, extract_s16_game_database, generate_cv_data_rs, load_s16_textures_by_name,
    render_catalog_beasts_and_cells_png, render_catalog_board_layout_png,
    render_catalog_gifts_themes_events_png, render_catalog_relics_png,
};
use tartarus_extract::PakVfs;

struct CliArgs {
    pak_dir: PathBuf,
    out_dir: PathBuf,
    lang: String,
    emit_rust: PathBuf,
}

const _: () = {
    assert!(std::mem::size_of::<CliArgs>() >= 32);
};

fn parse_cli_args() -> Result<CliArgs, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(!manifest_dir.as_os_str().is_empty());
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir)
        .to_path_buf();

    let mut pargs = pico_args::Arguments::from_env();
    let pak_dir: Option<PathBuf> = pargs
        .opt_value_from_str("--pak-dir")
        .map_err(|e| e.to_string())?;
    let out_dir: PathBuf = pargs
        .opt_value_from_str("--out-dir")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| workspace_root.join("assets/s16"));
    let lang: String = pargs
        .opt_value_from_str("--lang")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| String::from("en"));
    let emit_rust: PathBuf = pargs
        .opt_value_from_str("--emit-rust")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| workspace_root.join("crates/tartarus-scan/src/cv_data.rs"));
    let remaining = pargs.finish();
    if let Some(unknown) = remaining.first() {
        return Err(format!("Unknown argument: {}", unknown.to_string_lossy()));
    }

    let pak_dir = pak_dir
        .or_else(tartarus_extract::discover_game_dir)
        .ok_or_else(|| {
            "Could not auto-discover Torchlight: Infinite installation directory. \
             Pass --pak-dir <PATH> or set $TLI_GAME_DIR."
                .to_string()
        })?;
    assert!(!pak_dir.as_os_str().is_empty());

    Ok(CliArgs {
        pak_dir,
        out_dir,
        lang,
        emit_rust,
    })
}

fn render_all_catalogs_and_codegen(
    args: &CliArgs,
    vfs: &PakVfs,
    db: &S16GameDatabase,
) -> Result<(), String> {
    assert!(!args.out_dir.as_os_str().is_empty());
    println!("3. Decoding S16 Gameplay UI textures from .pak...");
    let textures = load_s16_textures_by_name(vfs);
    assert!(textures.len() <= vfs.entries.len());
    println!("   -> Decoded {} S16 UI textures", textures.len());

    println!("4. Rendering S16 visual catalog sheets...");
    let relics_png = args.out_dir.join("catalog_relics.png");
    render_catalog_relics_png(&relics_png, db, &textures)?;
    println!("   -> Saved {}", relics_png.display());

    let beasts_png = args.out_dir.join("catalog_beasts_and_cells.png");
    render_catalog_beasts_and_cells_png(&beasts_png, db, &textures)?;
    println!("   -> Saved {}", beasts_png.display());

    let gifts_png = args.out_dir.join("catalog_gifts_themes_events.png");
    render_catalog_gifts_themes_events_png(&gifts_png, db, &textures)?;
    println!("   -> Saved {}", gifts_png.display());

    let board_png = args.out_dir.join("catalog_board_layout.png");
    render_catalog_board_layout_png(&board_png, vfs, &textures)?;
    println!("   -> Saved {}", board_png.display());

    println!(
        "5. Synthesizing static Rust CV tables & S16 metadata 100% from .pak -> {}...",
        args.emit_rust.display()
    );
    generate_cv_data_rs(&args.emit_rust, vfs, db, &textures)?;
    println!("   -> Generated {}", args.emit_rust.display());
    Ok(())
}

fn main() -> Result<(), String> {
    let args = parse_cli_args()?;
    assert!(!args.lang.is_empty());
    fs::create_dir_all(&args.out_dir).map_err(|e| e.to_string())?;

    println!(
        "1. Mounting Torchlight Infinite Pak VFS from {}...",
        args.pak_dir.display()
    );
    let vfs = PakVfs::open_game_dir(&args.pak_dir, tartarus_extract::aes::DEFAULT_AES_KEY)
        .map_err(|e| format!("Failed to open PakVfs: {e}"))?;

    println!(
        "2. Extracting authoritative S16 Game Database (lang='{}')...",
        args.lang
    );
    let db = extract_s16_game_database(&vfs, &args.lang)?;
    assert!(!db.beasts.is_empty());
    println!(
        "   -> Extracted {} active relics, {} start themes, {} colored rewards, \
         {} beast pairs, {} events, {} tasks",
        db.relics.len(),
        db.start_themes.len(),
        db.colored_rewards.len(),
        db.beasts.len(),
        db.events.len(),
        db.tasks.len()
    );

    let json_path = args.out_dir.join("s16_game_data.json");
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&db).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!("   -> Saved {}", json_path.display());

    render_all_catalogs_and_codegen(&args, &vfs, &db)?;

    println!("\n=== S16 ACTIVE TORTURE INSTRUMENTS (31 RELICS) ===");
    for r in &db.relics {
        println!(
            "  #{:<3} [{:<10}] {:<27} | {:<38} | WHEN: {} | DOES: {}",
            r.id, r.js_sim, r.icon_asset, r.name, r.trigger_text, r.effect_text
        );
    }

    Ok(())
}

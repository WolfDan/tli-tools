use std::fs;
use std::path::PathBuf;
use tartarus_data::{
    extract_s16_game_database, generate_cv_data_rs, load_s16_textures_by_name,
    render_catalog_beasts_and_cells_png, render_catalog_board_layout_png,
    render_catalog_gifts_themes_events_png, render_catalog_relics_png,
};

fn main() -> Result<(), String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
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
            "Could not auto-discover Torchlight: Infinite installation directory. Pass --pak-dir <PATH> or set $TLI_GAME_DIR.".to_string()
        })?;

    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    println!(
        "1. Mounting Torchlight Infinite Pak VFS from {}...",
        pak_dir.display()
    );
    let vfs = tartarus_extract::pak::PakVfs::open_game_dir(
        &pak_dir,
        tartarus_extract::aes::DEFAULT_AES_KEY,
    )
    .map_err(|e| format!("Failed to open PakVfs: {e}"))?;

    println!(
        "2. Extracting authoritative S16 Game Database (lang='{}')...",
        lang
    );
    let db = extract_s16_game_database(&vfs, &lang)?;
    println!(
        "   -> Extracted {} active relics, {} start themes, {} colored rewards, {} beast pairs, {} events, {} tasks",
        db.relics.len(),
        db.start_themes.len(),
        db.colored_rewards.len(),
        db.beasts.len(),
        db.events.len(),
        db.tasks.len()
    );

    let json_path = out_dir.join("s16_game_data.json");
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&db).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!("   -> Saved {}", json_path.display());

    println!("3. Decoding S16 Gameplay UI textures from .pak...");
    let textures = load_s16_textures_by_name(&vfs);
    println!("   -> Decoded {} S16 UI textures", textures.len());

    println!("4. Rendering S16 visual catalog sheets...");
    let relics_png = out_dir.join("catalog_relics.png");
    render_catalog_relics_png(&relics_png, &db, &textures)?;
    println!("   -> Saved {}", relics_png.display());

    let beasts_png = out_dir.join("catalog_beasts_and_cells.png");
    render_catalog_beasts_and_cells_png(&beasts_png, &db, &textures)?;
    println!("   -> Saved {}", beasts_png.display());

    let gifts_png = out_dir.join("catalog_gifts_themes_events.png");
    render_catalog_gifts_themes_events_png(&gifts_png, &db, &textures)?;
    println!("   -> Saved {}", gifts_png.display());

    let board_png = out_dir.join("catalog_board_layout.png");
    render_catalog_board_layout_png(&board_png, &vfs, &textures)?;
    println!("   -> Saved {}", board_png.display());

    println!(
        "5. Synthesizing static Rust CV tables & S16 metadata 100% from .pak -> {}...",
        emit_rust.display()
    );
    generate_cv_data_rs(&emit_rust, &vfs, &db, &textures)?;
    println!("   -> Generated {}", emit_rust.display());

    println!("\n=== S16 ACTIVE TORTURE INSTRUMENTS (31 RELICS) ===");
    for r in &db.relics {
        println!(
            "  #{:<3} [{:<10}] {:<27} | {:<38} | WHEN: {} | DOES: {}",
            r.id, r.js_sim, r.icon_asset, r.name, r.trigger_text, r.effect_text
        );
    }

    Ok(())
}

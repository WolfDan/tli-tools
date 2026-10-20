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

    let mut pak_dir: Option<PathBuf> = None;
    let mut out_dir = workspace_root.join("assets/s16");
    let mut lang = String::from("en");
    let mut emit_rust = workspace_root.join("crates/tartarus-scan/src/cv_data.rs");

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--pak-dir" if i + 1 < args.len() => {
                pak_dir = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--out-dir" if i + 1 < args.len() => {
                out_dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--lang" if i + 1 < args.len() => {
                lang = args[i + 1].clone();
                i += 2;
            }
            "--emit-rust" if i + 1 < args.len() => {
                emit_rust = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            other => return Err(format!("Unknown argument: {other}")),
        }
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

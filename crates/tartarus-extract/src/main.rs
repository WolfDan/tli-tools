use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;
use tartarus_extract::{
    DEFAULT_AES_KEY_HEX, PakEntryMeta, PakVfs, PaperSpriteInfo, decrypt_luac_bytes,
    discover_game_dir, parse_aes_key_hex, parse_uasset_names, parse_xdi_config_table,
};

const _: () = {
    assert!(DEFAULT_AES_KEY_HEX.len() == 66);
};

fn print_usage() {
    let usage = "\
tartarus-extract — Torchlight: Infinite UE 4.27 Pak v12 + Oodle + BC7/DXT + Lua/Lang Extractor

USAGE:
  tartarus-extract <COMMAND> [OPTIONS]

COMMANDS:
  list    [FILTER]              List virtual files in all .pak archives matching FILTER
  raw     <FILTER> <OUT_DIR>    Extract raw decrypted + Oodle-decompressed files (.uasset/.uexp/...)
  png     <FILTER> <OUT_DIR>    Decode matching cooked UTexture2D & UPaperSprite assets to .png
  luac    <FILTER> <OUT_DIR>    Decrypt matching .out Lua bytecode (.luac) and DataCfg (.json)
  lang    [LANG] [PREFIX]       Dump localization pairs from config_multi_language_<LANG>.bin
  inspect <VIRTUAL_PATH>        Inspect a .uasset/.uexp pair (FName table, sizes, texture format)

OPTIONS:
  --game-dir <PATH>             Game root or .pak path (auto-discovered via Steam or $TLI_GAME_DIR)
  --key <HEX>                   256-bit AES key in hex (default: 0x4F17B93C...1C1B34E0)
";
    assert!(usage.contains("USAGE:"));
    assert!(usage.contains("tartarus-extract"));
    eprint!("{usage}");
}

fn main() {
    assert_eq!(DEFAULT_AES_KEY_HEX.len(), 66);
    let res = run();
    if let Err(e) = res {
        assert!(!e.is_empty());
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn cmd_list(vfs: &PakVfs, pos_args: &[String]) {
    assert!(!pos_args.is_empty());
    let filter = pos_args
        .get(1)
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    let mut count = 0usize;
    for (vpath, meta) in &vfs.entries {
        if filter.is_empty() || vpath.to_lowercase().contains(&filter) {
            println!(
                "{:<10} {:<7} {:>10} -> {:>10}  [{}] {}",
                format!("{:?}", meta.compression),
                if meta.encrypted { "AES256" } else { "PLAIN" },
                meta.comp_size,
                meta.uncomp_size,
                meta.pak_name,
                vpath
            );
            count += 1;
        }
    }
    assert!(count <= vfs.entries.len());
    eprintln!(
        "Matched {count} / {} total indexed files.",
        vfs.entries.len()
    );
}

fn cmd_raw(vfs: &PakVfs, pos_args: &[String]) -> Result<(), String> {
    assert!(!pos_args.is_empty());
    let filter = pos_args
        .get(1)
        .ok_or("Usage: tartarus-extract raw <FILTER> <OUT_DIR>")?
        .to_lowercase();
    let out_dir = PathBuf::from(
        pos_args
            .get(2)
            .ok_or("Usage: tartarus-extract raw <FILTER> <OUT_DIR>")?,
    );

    let matched: Vec<&PakEntryMeta> = vfs
        .entries
        .values()
        .filter(|m| m.virtual_path.to_lowercase().contains(&filter))
        .collect();
    assert!(matched.len() <= vfs.entries.len());

    eprintln!(
        "Extracting {} matching files to {}...",
        matched.len(),
        out_dir.display()
    );
    let ok_count = AtomicUsize::new(0);
    let err_count = AtomicUsize::new(0);

    matched
        .par_iter()
        .for_each(|meta| match vfs.read_entry(meta) {
            Ok(bytes) => {
                let dest = out_dir.join(&meta.virtual_path);
                if let Some(parent) = dest.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if fs::write(&dest, &bytes).is_ok() {
                    ok_count.fetch_add(1, Ordering::Relaxed);
                } else {
                    err_count.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(e) => {
                eprintln!("WARN: {}: {e}", meta.virtual_path);
                err_count.fetch_add(1, Ordering::Relaxed);
            }
        });

    eprintln!(
        "Done: {} extracted, {} failed.",
        ok_count.load(Ordering::Relaxed),
        err_count.load(Ordering::Relaxed)
    );
    Ok(())
}

fn slice_paper_sprites_to_png(vfs: &PakVfs, sprites: &[&PakEntryMeta], out_dir: &Path) -> usize {
    assert!(!out_dir.as_os_str().is_empty());
    let mut sprites_by_atlas: BTreeMap<String, Vec<(&PakEntryMeta, PaperSpriteInfo)>> =
        BTreeMap::new();
    for smeta in sprites {
        if let Ok(info) = vfs.read_paper_sprite(&smeta.virtual_path) {
            sprites_by_atlas
                .entry(info.atlas_virtual_path.clone())
                .or_default()
                .push((*smeta, info));
        }
    }

    let ok_sprites = AtomicUsize::new(0);
    let atlas_groups: Vec<(&String, &Vec<(&PakEntryMeta, PaperSpriteInfo)>)> =
        sprites_by_atlas.iter().collect();

    atlas_groups.par_iter().for_each(|(atlas_path, list)| {
        if let Ok(atlas_tex) = vfs.decode_texture(atlas_path) {
            for (smeta, info) in *list {
                let cw =
                    (info.width as usize).min(atlas_tex.width.saturating_sub(info.uv_x as usize));
                let ch =
                    (info.height as usize).min(atlas_tex.height.saturating_sub(info.uv_y as usize));
                if cw == 0 || ch == 0 {
                    continue;
                }
                let cropped = tartarus_extract::crop_rgba8(
                    &atlas_tex.rgba8,
                    atlas_tex.width,
                    atlas_tex.height,
                    info.uv_x as usize,
                    info.uv_y as usize,
                    cw,
                    ch,
                );
                if let Ok(png_bytes) =
                    tartarus_extract::encode_png_rgba8(cw as u32, ch as u32, &cropped)
                {
                    let rel_png = Path::new(&smeta.virtual_path).with_extension("png");
                    let dest = out_dir.join(rel_png);
                    if let Some(parent) = dest.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if fs::write(&dest, png_bytes).is_ok() {
                        ok_sprites.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
    });

    let count = ok_sprites.load(Ordering::Relaxed);
    assert!(count <= sprites.len());
    count
}

fn cmd_png(vfs: &PakVfs, pos_args: &[String]) -> Result<(), String> {
    assert!(!pos_args.is_empty());
    let usage = "Usage: tartarus-extract png <FILTER> <OUT_DIR>";
    let filter = pos_args.get(1).ok_or(usage)?.to_lowercase();
    let out_dir = PathBuf::from(pos_args.get(2).ok_or(usage)?);

    let uassets: Vec<&PakEntryMeta> = vfs
        .entries
        .values()
        .filter(|m| {
            m.virtual_path.ends_with(".uasset")
                && m.virtual_path.to_lowercase().contains(&filter)
                && !m.virtual_path.ends_with("_Sprite.uasset")
        })
        .collect();
    let sprites: Vec<&PakEntryMeta> = vfs
        .entries
        .values()
        .filter(|m| {
            m.virtual_path.ends_with("_Sprite.uasset")
                && m.virtual_path.to_lowercase().contains(&filter)
        })
        .collect();
    assert!(uassets.len() + sprites.len() <= vfs.entries.len());

    eprintln!(
        "Decoding {} UTexture2D assets and slicing {} UPaperSprite sprites ('{}') to {}...",
        uassets.len(),
        sprites.len(),
        filter,
        out_dir.display()
    );
    let ok_tex = AtomicUsize::new(0);
    let skip_tex = AtomicUsize::new(0);

    uassets.par_iter().for_each(|meta| {
        if let Ok(tex) = vfs.decode_texture(&meta.virtual_path) {
            let dest = out_dir.join(Path::new(&meta.virtual_path).with_extension("png"));
            if let Some(parent) = dest.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if fs::write(&dest, tex.to_png_bytes()).is_ok() {
                ok_tex.fetch_add(1, Ordering::Relaxed);
            }
        } else {
            skip_tex.fetch_add(1, Ordering::Relaxed);
        }
    });

    let ok_sprites = slice_paper_sprites_to_png(vfs, &sprites, &out_dir);
    eprintln!(
        "Done: {} textures decoded, {} Paper2D sprites sliced ({} non-texture/skipped).",
        ok_tex.load(Ordering::Relaxed),
        ok_sprites,
        skip_tex.load(Ordering::Relaxed)
    );
    Ok(())
}

fn cmd_luac(vfs: &PakVfs, pos_args: &[String]) -> Result<(), String> {
    assert!(!pos_args.is_empty());
    let filter = pos_args
        .get(1)
        .ok_or("Usage: tartarus-extract luac <FILTER> <OUT_DIR>")?
        .to_lowercase();
    let out_dir = PathBuf::from(
        pos_args
            .get(2)
            .ok_or("Usage: tartarus-extract luac <FILTER> <OUT_DIR>")?,
    );

    let matched: Vec<&PakEntryMeta> = vfs
        .entries
        .values()
        .filter(|m| {
            m.virtual_path.ends_with(".out") && m.virtual_path.to_lowercase().contains(&filter)
        })
        .collect();
    assert!(matched.len() <= vfs.entries.len());

    eprintln!(
        "Decrypting {} .out Lua bytecode files matching '{}' to {}...",
        matched.len(),
        filter,
        out_dir.display()
    );

    let ok_luac = AtomicUsize::new(0);
    let ok_json = AtomicUsize::new(0);

    matched.par_iter().for_each(|meta| {
        if let Ok(raw) = vfs.read_entry(meta) {
            let dec = decrypt_luac_bytes(&raw);
            let dest_luac = out_dir.join(&meta.virtual_path).with_extension("luac");
            if let Some(parent) = dest_luac.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if fs::write(&dest_luac, &dec).is_ok() {
                ok_luac.fetch_add(1, Ordering::Relaxed);
            }
            if meta.virtual_path.contains("/DataCfg/config_")
                && let Ok(tbl) = parse_xdi_config_table(&dec)
                && let Ok(json_str) = serde_json::to_string_pretty(&tbl)
            {
                let dest_json = dest_luac.with_extension("json");
                if fs::write(&dest_json, json_str).is_ok() {
                    ok_json.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    });

    eprintln!(
        "Done: {} .luac files decrypted, {} DataCfg tables exported to .json.",
        ok_luac.load(Ordering::Relaxed),
        ok_json.load(Ordering::Relaxed)
    );
    Ok(())
}

fn cmd_lang(vfs: &PakVfs, pos_args: &[String]) -> Result<(), String> {
    assert!(!pos_args.is_empty());
    let lang = pos_args.get(1).map(String::as_str).unwrap_or("en");
    assert!(!lang.is_empty());
    let prefix = pos_args.get(2).map(String::as_str);
    let map = vfs.read_multi_language(lang, prefix)?;
    for (k, v) in &map {
        println!("{k}\t{v}");
    }
    eprintln!(
        "Matched {} localization entries in config_multi_language_{lang}.bin.",
        map.len()
    );
    Ok(())
}

fn cmd_inspect(vfs: &PakVfs, pos_args: &[String]) -> Result<(), String> {
    assert!(!pos_args.is_empty());
    let vpath = pos_args
        .get(1)
        .ok_or("Usage: tartarus-extract inspect <VIRTUAL_PATH>")?;
    let base = vpath
        .trim_start_matches('/')
        .trim_end_matches(".uasset")
        .trim_end_matches(".uexp");
    assert!(!base.is_empty());
    let uasset_vpath = format!("{base}.uasset");
    let uexp_vpath = format!("{base}.uexp");

    let uasset = vfs.read_path(&uasset_vpath)?;
    let names = parse_uasset_names(&uasset);
    println!("Asset: {base}");
    println!("  .uasset size: {} bytes", uasset.len());
    if let Ok(uexp) = vfs.read_path(&uexp_vpath) {
        println!("  .uexp size:   {} bytes", uexp.len());
    }
    println!("  FNames ({}):", names.len());
    for (idx, n) in names.iter().enumerate() {
        println!("    [{idx:3}] {n}");
    }
    if let Ok(tex) = vfs.decode_texture(base) {
        println!(
            "  Decoded Texture2D: {}x{} {:?} ({} mips)",
            tex.width, tex.height, tex.format, tex.num_mips
        );
    }
    if let Ok(sprite) = vfs.read_paper_sprite(base) {
        println!(
            "  PaperSprite: atlas={} uv=({}, {}) size={}x{}",
            sprite.atlas_virtual_path, sprite.uv_x, sprite.uv_y, sprite.width, sprite.height
        );
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let mut pargs = pico_args::Arguments::from_env();
    if env::args_os().len() <= 1 || pargs.contains(["-h", "--help"]) {
        print_usage();
        return Ok(());
    }

    let game_dir: Option<PathBuf> = pargs
        .opt_value_from_str("--game-dir")
        .map_err(|e| e.to_string())?;
    let key_hex: String = pargs
        .opt_value_from_str("--key")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| DEFAULT_AES_KEY_HEX.to_string());
    let pos_args: Vec<String> = pargs
        .finish()
        .into_iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();

    let game_dir = game_dir.or_else(discover_game_dir).ok_or_else(|| {
        "Could not auto-discover Torchlight: Infinite installation directory. \
         Pass --game-dir <PATH> or set $TLI_GAME_DIR."
            .to_string()
    })?;
    assert!(!game_dir.as_os_str().is_empty());

    let cmd = pos_args.first().map(String::as_str).unwrap_or("list");
    let key = parse_aes_key_hex(&key_hex)?;
    assert_eq!(key.len(), 32);
    let vfs = PakVfs::open_game_dir(&game_dir, key)?;

    match cmd {
        "list" => {
            cmd_list(&vfs, &pos_args);
            Ok(())
        }
        "raw" => cmd_raw(&vfs, &pos_args),
        "png" => cmd_png(&vfs, &pos_args),
        "luac" => cmd_luac(&vfs, &pos_args),
        "lang" => cmd_lang(&vfs, &pos_args),
        "inspect" => cmd_inspect(&vfs, &pos_args),
        other => Err(format!("Unknown command '{other}'")),
    }
}

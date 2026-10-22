use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;
use serde::Serialize;
use tartarus_extract::{
    DEFAULT_AES_KEY_HEX, MaterialInfo, PakEntryMeta, PakVfs, PaperSpriteInfo, ShaderArchiveIndex,
    decompile_dxbc_to_hlsl, decompile_luac_bytes, disassemble_dxbc, discover_game_dir,
    parse_aes_key_hex, parse_dxbc, parse_uasset_names, parse_xdi_config_table, xdi_to_lua53_bytes,
};

const _: () = {
    assert!(DEFAULT_AES_KEY_HEX.len() == 66);
};

fn print_usage() {
    let usage = "\
tartarus-extract — Torchlight: Infinite UE 4.27 Pak v12 + Oodle + BC7/DXT + Lua/Shader Extractor

USAGE:
  tartarus-extract <COMMAND> [OPTIONS]

COMMANDS:
  list    [FILTER]              List virtual files in all .pak archives matching FILTER
  raw     <FILTER> <OUT_DIR>    Extract raw decrypted + Oodle-decompressed files (.uasset/.uexp/...)
  png     <FILTER> <OUT_DIR>    Decode matching cooked UTexture2D & UPaperSprite assets to .png
  luac    <FILTER> <OUT_DIR>    Decrypt & decompile matching .out Lua bytecode (.luac/.lua/.json)
  shader  <FILTER> <OUT_DIR>    Extract & decompile materials/permutations or .ushaderbytecode (.dxbc/.asm/.hlsl/.mat.json)
  lang    [LANG] [PREFIX]       Dump localization pairs from config_multi_language_<LANG>.bin
  inspect <VIRTUAL_PATH>        Inspect a .uasset/.uexp pair or .ushaderbytecode archive

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

fn extract_single_luac_entry(
    vfs: &PakVfs,
    meta: &PakEntryMeta,
    out_dir: &Path,
) -> Result<bool, String> {
    assert!(!meta.virtual_path.is_empty());
    assert!(!out_dir.as_os_str().is_empty());

    let vpath = &meta.virtual_path;
    let raw = vfs.read_entry(meta).map_err(|e| format!("{vpath}: {e}"))?;
    let lua53 = xdi_to_lua53_bytes(&raw).map_err(|e| format!("{vpath}: {e}"))?;
    let lua_src = decompile_luac_bytes(&lua53).map_err(|e| format!("{vpath}: {e}"))?;

    let dest_luac = out_dir.join(vpath).with_extension("luac");
    if let Some(parent) = dest_luac.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(&dest_luac, &lua53).map_err(|e| format!("{}: {e}", dest_luac.display()))?;

    let dest_lua = dest_luac.with_extension("lua");
    fs::write(&dest_lua, &lua_src).map_err(|e| format!("{}: {e}", dest_lua.display()))?;

    if vpath.contains("/DataCfg/config_") {
        let tbl = parse_xdi_config_table(&lua53).map_err(|e| format!("{vpath}: {e}"))?;
        let json_str = serde_json::to_string_pretty(&tbl).map_err(|e| format!("{vpath}: {e}"))?;
        let dest_json = dest_luac.with_extension("json");
        fs::write(&dest_json, json_str).map_err(|e| format!("{}: {e}", dest_json.display()))?;
        return Ok(true);
    }

    Ok(false)
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
        "Decrypting and decompiling {} .out Lua bytecode files matching '{}' to {}...",
        matched.len(),
        filter,
        out_dir.display()
    );

    let ok_luac = AtomicUsize::new(0);
    let ok_json = AtomicUsize::new(0);

    matched.par_iter().try_for_each(|meta| {
        let wrote_json = extract_single_luac_entry(vfs, meta, &out_dir)?;
        ok_luac.fetch_add(1, Ordering::Relaxed);
        if wrote_json {
            ok_json.fetch_add(1, Ordering::Relaxed);
        }
        Ok::<(), String>(())
    })?;

    eprintln!(
        "Done: {} .luac/.lua pairs written, {} DataCfg tables exported to .json.",
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

#[derive(Clone, Debug, Serialize)]
struct ExportedShaderRecord {
    shader_index: usize,
    shader_hash: String,
    stage: String,
    compressed_size: usize,
    uncompressed_size: usize,
    dxbc_file: String,
    asm_file: String,
    hlsl_file: String,
}

#[derive(Clone, Debug, Serialize)]
struct MaterialManifest {
    material: MaterialInfo,
    shader_archive: String,
    shader_map_index: usize,
    shaders: Vec<ExportedShaderRecord>,
}

fn write_shader_triple(
    vfs: &PakVfs,
    arch_meta: &PakEntryMeta,
    arch: &ShaderArchiveIndex,
    shader_idx: usize,
    shader_dir: &Path,
    mat_opt: Option<&MaterialInfo>,
) -> Result<ExportedShaderRecord, String> {
    assert!(shader_idx < arch.shader_entries.len());
    assert!(!shader_dir.as_os_str().is_empty());
    let blob = vfs
        .extract_shader_entry(arch_meta, arch, shader_idx)
        .map_err(|e| e.to_string())?;
    let parsed = parse_dxbc(&blob.dxbc_bytes).map_err(|e| e.to_string())?;
    let asm_text = disassemble_dxbc(&parsed);
    let hlsl_text = decompile_dxbc_to_hlsl(&parsed, mat_opt);

    let hash8 = &blob.shader_hash_hex[..8.min(blob.shader_hash_hex.len())];
    let stem = format!("{}_{shader_idx:06}_{hash8}", blob.stage.short_prefix());
    fs::create_dir_all(shader_dir).map_err(|e| e.to_string())?;

    let dxbc_name = format!("{stem}.dxbc");
    let asm_name = format!("{stem}.asm");
    let hlsl_name = format!("{stem}.hlsl");
    fs::write(shader_dir.join(&dxbc_name), &blob.dxbc_bytes).map_err(|e| e.to_string())?;
    fs::write(shader_dir.join(&asm_name), &asm_text).map_err(|e| e.to_string())?;
    fs::write(shader_dir.join(&hlsl_name), &hlsl_text).map_err(|e| e.to_string())?;

    Ok(ExportedShaderRecord {
        shader_index: shader_idx,
        shader_hash: blob.shader_hash_hex,
        stage: blob.stage.display_name().to_string(),
        compressed_size: blob.compressed_size,
        uncompressed_size: blob.uncompressed_size,
        dxbc_file: dxbc_name,
        asm_file: asm_name,
        hlsl_file: hlsl_name,
    })
}

fn extract_single_material_shaders(
    vfs: &PakVfs,
    archives: &[(PakEntryMeta, ShaderArchiveIndex)],
    vpath: &str,
    out_dir: &Path,
) -> Result<Option<usize>, String> {
    assert!(!vpath.is_empty());
    assert!(!out_dir.as_os_str().is_empty());
    let Ok(mat) = vfs.read_material_info(vpath) else {
        return Ok(None);
    };
    let Some(smap_hash) = mat.shader_map_hash else {
        return Ok(None);
    };
    let Some((arch_meta, arch, smap_idx)) = archives.iter().find_map(|(m, a)| {
        a.map_hash_to_index
            .get(&smap_hash)
            .copied()
            .map(|idx| (m, a, idx))
    }) else {
        return Ok(None);
    };

    let base_rel = Path::new(vpath).with_extension("");
    let shader_dir = out_dir.join(format!("{}_shaders", base_rel.display()));
    let shader_indices = arch.shader_indices_for_map(smap_idx);
    let mut records = Vec::with_capacity(shader_indices.len());
    for si in shader_indices {
        records.push(write_shader_triple(
            vfs,
            arch_meta,
            arch,
            si,
            &shader_dir,
            Some(&mat),
        )?);
    }

    let manifest = MaterialManifest {
        material: mat,
        shader_archive: arch.archive_virtual_path.clone(),
        shader_map_index: smap_idx,
        shaders: records,
    };
    let json_dest = out_dir.join(base_rel.with_extension("mat.json"));
    if let Some(parent) = json_dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json_str = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    fs::write(&json_dest, json_str).map_err(|e| e.to_string())?;
    Ok(Some(manifest.shaders.len()))
}

fn extract_archive_shaders_by_filter(
    vfs: &PakVfs,
    archives: &[(PakEntryMeta, ShaderArchiveIndex)],
    filter: &str,
    out_dir: &Path,
) -> Result<(usize, usize), String> {
    assert!(!filter.is_empty());
    assert!(!out_dir.as_os_str().is_empty());
    let mut map_count = 0usize;
    let mut shader_count = 0usize;

    for (arch_meta, arch) in archives {
        let arch_matches = arch.archive_virtual_path.to_lowercase().contains(filter);
        let arch_stem = Path::new(&arch.archive_virtual_path).with_extension("");
        for (smap_idx, smap_hash) in arch.shader_map_hashes.iter().enumerate() {
            let smap_hex = hex::encode(smap_hash);
            if arch_matches || smap_hex.contains(filter) {
                let s_dir = out_dir
                    .join(&arch_stem)
                    .join(format!("smap_{smap_idx:05}_{}", &smap_hex[..12]));
                for si in arch.shader_indices_for_map(smap_idx) {
                    write_shader_triple(vfs, arch_meta, arch, si, &s_dir, None)?;
                    shader_count += 1;
                }
                map_count += 1;
            }
        }
        if map_count == 0 {
            for (si, sh_hash) in arch.shader_hashes.iter().enumerate() {
                let sh_hex = hex::encode(sh_hash);
                if sh_hex.contains(filter) {
                    let s_dir = out_dir.join(&arch_stem).join("shaders");
                    write_shader_triple(vfs, arch_meta, arch, si, &s_dir, None)?;
                    shader_count += 1;
                }
            }
        }
    }
    Ok((map_count, shader_count))
}

fn cmd_shader(vfs: &PakVfs, pos_args: &[String]) -> Result<(), String> {
    assert!(!pos_args.is_empty());
    let usage = "Usage: tartarus-extract shader <FILTER> <OUT_DIR>";
    let filter = pos_args.get(1).ok_or(usage)?.to_lowercase();
    let out_dir = PathBuf::from(pos_args.get(2).ok_or(usage)?);

    let archives = vfs
        .load_shader_archive_indices()
        .map_err(|e| e.to_string())?;
    let uassets: Vec<&PakEntryMeta> = vfs
        .entries
        .values()
        .filter(|m| {
            m.virtual_path.ends_with(".uasset") && m.virtual_path.to_lowercase().contains(&filter)
        })
        .collect();
    assert!(uassets.len() <= vfs.entries.len());

    let ok_mats = AtomicUsize::new(0);
    let ok_shaders = AtomicUsize::new(0);
    uassets.par_iter().for_each(|meta| {
        if let Ok(Some(n)) =
            extract_single_material_shaders(vfs, &archives, &meta.virtual_path, &out_dir)
        {
            ok_mats.fetch_add(1, Ordering::Relaxed);
            ok_shaders.fetch_add(n, Ordering::Relaxed);
        }
    });

    let mut mats_done = ok_mats.load(Ordering::Relaxed);
    let mut shaders_done = ok_shaders.load(Ordering::Relaxed);
    if mats_done == 0 {
        let (mc, sc) = extract_archive_shaders_by_filter(vfs, &archives, &filter, &out_dir)?;
        mats_done = mc;
        shaders_done = sc;
    }

    eprintln!(
        "Done: {} material/shadermap manifests and {} decompiled shaders (.dxbc/.asm/.hlsl) written to {}.",
        mats_done,
        shaders_done,
        out_dir.display()
    );
    Ok(())
}

fn print_material_inspection(vfs: &PakVfs, mat: &MaterialInfo) {
    assert!(!mat.virtual_path.is_empty());
    println!("  Material / Permutation Info:");
    if !mat.parent_chain.is_empty() {
        println!("    Parent Chain : {}", mat.parent_chain.join(" -> "));
    }
    println!(
        "    Blend/Shading: {:?} / {:?} (bHasStaticPermutationResource={})",
        mat.blend_mode.as_deref().unwrap_or("Default"),
        mat.shading_model.as_deref().unwrap_or("Default"),
        mat.has_static_permutation_resource
    );
    for sw in &mat.static_switches {
        println!("    [StaticSwitch] {:<28} = {}", sw.name, sw.value);
    }
    for m in &mat.static_component_masks {
        println!(
            "    [StaticMask]   {:<28} = RGBA({}, {}, {}, {})",
            m.name, m.r as u8, m.g as u8, m.b as u8, m.a as u8
        );
    }
    for s in &mat.scalar_params {
        println!("    [ScalarParam]  {:<28} = {:.5}", s.name, s.value);
    }
    for v in &mat.vector_params {
        println!(
            "    [VectorParam]  {:<28} = ({:.4}, {:.4}, {:.4}, {:.4})",
            v.name, v.rgba[0], v.rgba[1], v.rgba[2], v.rgba[3]
        );
    }
    for t in &mat.texture_params {
        println!("    [TextureParam] {:<28} = {}", t.name, t.texture_path);
    }
    if !mat.uniform_expression_names.is_empty() {
        println!(
            "    Uniform Expressions ({}): {}",
            mat.uniform_expression_names.len(),
            mat.uniform_expression_names.join(", ")
        );
    }
    if let Some(smap_hash) = mat.shader_map_hash {
        println!(
            "    ShaderMapHash: {} (source: {})",
            hex::encode(smap_hash),
            mat.resolved_shader_source_path
        );
        if let Ok(archives) = vfs.load_shader_archive_indices() {
            for (_, arch) in &archives {
                if let Some(&smap_idx) = arch.map_hash_to_index.get(&smap_hash) {
                    let indices = arch.shader_indices_for_map(smap_idx);
                    println!(
                        "    Resolved in {} -> ShaderMap[{}] ({} unique shaders):",
                        arch.archive_virtual_path,
                        smap_idx,
                        indices.len()
                    );
                    for si in indices {
                        let e = &arch.shader_entries[si];
                        println!(
                            "      shader[{si:6}] {:<7} hash={} comp={:>5} -> uncomp={:>5}",
                            e.frequency.display_name(),
                            hex::encode(arch.shader_hashes[si]),
                            e.size,
                            e.uncompressed_size
                        );
                    }
                }
            }
        }
    }
}

fn cmd_inspect(vfs: &PakVfs, pos_args: &[String]) -> Result<(), String> {
    assert!(!pos_args.is_empty());
    let vpath = pos_args
        .get(1)
        .ok_or("Usage: tartarus-extract inspect <VIRTUAL_PATH>")?;
    if vpath.ends_with(".ushaderbytecode") {
        let clean = vpath.trim_start_matches('/');
        let meta = vfs
            .entries
            .get(clean)
            .ok_or_else(|| format!("Not found: {clean}"))?;
        let arch =
            tartarus_extract::read_shader_archive_index_from_pak(meta, || vfs.read_entry(meta))
                .map_err(|e| e.to_string())?;
        println!("ShaderArchive: {}", arch.archive_virtual_path);
        println!("  Version           : {}", arch.version);
        println!("  Payload Offset    : {} bytes", arch.payload_offset);
        println!("  ShaderMaps        : {}", arch.shader_map_entries.len());
        println!("  Unique Shaders    : {}", arch.shader_entries.len());
        println!("  Shader Index Refs : {}", arch.shader_indices.len());
        println!("  Stage Breakdown   : {:?}", arch.stage_counts());
        return Ok(());
    }

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
    if let Ok(mat) = vfs.read_material_info(base)
        && (mat.shader_map_hash.is_some()
            || mat.parent_path.is_some()
            || !mat.static_switches.is_empty())
    {
        print_material_inspection(vfs, &mat);
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
        "shader" => cmd_shader(&vfs, &pos_args),
        "lang" => cmd_lang(&vfs, &pos_args),
        "inspect" => cmd_inspect(&vfs, &pos_args),
        other => Err(format!("Unknown command '{other}'")),
    }
}

//! `DataCfg` and `LuaPreLoadTable` schema extractor and Rust parser code generator.

pub mod domains;
pub mod emit;
pub mod schema;
pub mod vm;

#[cfg(test)]
mod tests;

use crate::datacfg::domains::classify_domain;
use crate::datacfg::emit::{CodegenMode, emit_domain_module, emit_mod_rs, emit_preload_module};
use crate::datacfg::schema::{LuaTableSpec, analyze_luatable, analyze_preload};
use crate::error::XtaskError;
use pak::{AesKey, ReadScratch, ScratchLimits};
use pak_vfs::{FileId, PakVfs};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Sorted virtual file lists for `LuaTable/DataCfg` and `LuaPreLoadTable/DataCfg`.
struct DataCfgFileLists {
    luatable_files: Vec<(String, FileId)>,
    preload_files: Vec<(String, FileId)>,
}

fn collect_datacfg_files(vfs: &PakVfs) -> Result<DataCfgFileLists, XtaskError> {
    let mut path_buf = Vec::new();
    let mut luatable_files = Vec::new();
    let mut preload_files = Vec::new();
    for fid in vfs.iter_files() {
        path_buf.clear();
        vfs.write_file_path(fid, &mut path_buf);
        let s = core::str::from_utf8(&path_buf)?.to_owned();
        if s.starts_with("UE_game/Content/Script/Luac/Config/LuaTable/DataCfg/config_")
            && s.ends_with(".out")
        {
            luatable_files.push((s, fid));
        } else if s.starts_with(
            "UE_game/Content/Script/Luac/Config/LuaPreLoadTable/DataCfg/config_preload_",
        ) && s.ends_with(".out")
        {
            preload_files.push((s, fid));
        }
    }
    luatable_files.sort_by(|a, b| a.0.cmp(&b.0));
    preload_files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(DataCfgFileLists {
        luatable_files,
        preload_files,
    })
}

fn read_vfs_entry(
    vfs: &PakVfs,
    path: &str,
    fid: FileId,
    scratch: &mut ReadScratch,
    buf: &mut Vec<u8>,
) -> Result<String, XtaskError> {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| XtaskError::MalformedBytecode {
            stem: path.into(),
            context: "invalid file stem",
        })?
        .to_owned();
    let len = usize::try_from(vfs.file_meta(fid).uncompressed_bytes()).map_err(|_| {
        XtaskError::MalformedBytecode {
            stem: stem.clone().into(),
            context: "file size overflow",
        }
    })?;
    buf.resize(len, 0);
    vfs.read_into(fid, scratch, buf)?;
    Ok(stem)
}

fn resolve_tables_out_dir() -> Result<PathBuf, XtaskError> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or(XtaskError::Usage {
            message: "failed to resolve workspace root from CARGO_MANIFEST_DIR",
        })?;
    Ok(workspace_root.join("crates/parsers/src/luac/tables"))
}

/// Runs the `DataCfg` schema extractor and either writes or checks `crates/parsers/src/luac/tables/`.
pub fn run_codegen(mode: CodegenMode) -> Result<(), XtaskError> {
    let game_dir = std::env::var("TLI_GAME_DIR").map_err(|source| XtaskError::EnvVar {
        name: "TLI_GAME_DIR",
        source,
    })?;
    let key_hex = std::env::var("TLI_AES_KEY").map_err(|source| XtaskError::EnvVar {
        name: "TLI_AES_KEY",
        source,
    })?;
    let key = AesKey::try_from_hex(key_hex.trim())?;
    let vfs = PakVfs::open_tli_game_dir(Path::new(&game_dir), Some(key))?;
    let mut scratch = ReadScratch::with_limits(ScratchLimits::DEFAULT);
    let mut buf = Vec::new();
    let lists = collect_datacfg_files(&vfs)?;

    let mut domains: BTreeMap<&'static str, Vec<LuaTableSpec>> = BTreeMap::new();
    for &(ref path, fid) in &lists.luatable_files {
        let stem = read_vfs_entry(&vfs, path, fid, &mut scratch, &mut buf)?;
        let domain = classify_domain(&stem);
        let spec = analyze_luatable(stem, &mut buf)?;
        domains.entry(domain).or_default().push(spec);
    }

    let mut preload_specs = Vec::with_capacity(lists.preload_files.len());
    for &(ref path, fid) in &lists.preload_files {
        let stem = read_vfs_entry(&vfs, path, fid, &mut scratch, &mut buf)?;
        preload_specs.push(analyze_preload(stem, &mut buf)?);
    }

    let out_dir = resolve_tables_out_dir()?;
    if mode == CodegenMode::Write {
        fs::create_dir_all(&out_dir)?;
    }

    let mut all_modules = BTreeSet::new();
    for (domain, specs) in &domains {
        all_modules.insert(*domain);
        emit_domain_module(&out_dir, domain, specs, mode)?;
    }
    all_modules.insert("preload");
    emit_preload_module(&out_dir, &preload_specs, mode)?;
    emit_mod_rs(&out_dir, &all_modules, mode)?;

    let action = match mode {
        CodegenMode::Write => "Generated",
        CodegenMode::Check => "Verified",
    };
    println!(
        "{action} {} domain modules ({} total tables) in {}",
        all_modules.len(),
        lists.luatable_files.len() + lists.preload_files.len(),
        out_dir.display()
    );
    Ok(())
}

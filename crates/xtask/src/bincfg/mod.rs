//! `.bin` binary configuration schema extractor, corpus verifier, and Rust parser code generator.

pub mod disasm;
pub mod domains;
pub mod emit;
pub mod luabridge;
pub mod pe;

#[cfg(test)]
mod tests;

use crate::bincfg::disasm::{BinarySchemas, ExtractedTableSchema, extract_all_binary_schemas};
use crate::bincfg::domains::{MapKeyWire, SubOp, WireType, classify_bin_domain};
use crate::bincfg::emit::{
    BinTableSpec, SubStructRegistry, emit_action_data_rs, emit_affix_ops_rs,
    emit_bin_domain_module, emit_bin_mod_rs, emit_substructs_rs,
};
use crate::bincfg::luabridge::{extract_luabridge_structs, resolve_column_names};
use crate::bincfg::pe::{ExtractedKeySchema, PeImage, normalize_stem};
use crate::datacfg::domains::{strip_config_prefix, to_pascal_case, to_snake_case};
use crate::datacfg::emit::{CodegenMode, verify_or_prune_stale_rs_files};
use crate::error::XtaskError;
use crate::u32_to_usize;
use pak::{AesKey, ReadScratch, ScratchLimits};
use pak_vfs::{FileId, PakVfs};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use zerocopy::FromBytes;
use zerocopy::byteorder::little_endian::U32;

const CHUNK_LOCALE_SUFFIXES: [&str; 19] = [
    "_cn", "_de", "_en", "_es", "_fr", "_hk", "_id", "_ja", "_jp", "_ko", "_kr", "_pt", "_ru",
    "_th", "_tr", "_tw", "_vi", "_vn", "_zh",
];

fn strip_locale_suffix(raw_config: &str) -> &str {
    if raw_config.len() > 3
        && let Some((base, tail)) = raw_config.rsplit_once('_')
        && tail.len() == 2
    {
        let suffix_with_underscore = &raw_config[base.len()..];
        if CHUNK_LOCALE_SUFFIXES
            .iter()
            .any(|s| suffix_with_underscore.eq_ignore_ascii_case(s))
        {
            return base;
        }
    }
    raw_config
}

struct CorpusCursor<'a> {
    buf: &'a [u8],
}

impl<'a> CorpusCursor<'a> {
    const fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    fn skip(&mut self, len: usize, path: &str) -> Result<(), XtaskError> {
        let (_, rest) =
            self.buf
                .split_at_checked(len)
                .ok_or_else(|| XtaskError::BinCorpusVerify {
                    path: path.into(),
                    detail: "unexpected EOF skipping bytes".into(),
                })?;
        self.buf = rest;
        Ok(())
    }

    fn read_u8(&mut self, path: &str) -> Result<u8, XtaskError> {
        let (&b, rest) = self
            .buf
            .split_first()
            .ok_or_else(|| XtaskError::BinCorpusVerify {
                path: path.into(),
                detail: "unexpected EOF reading u8".into(),
            })?;
        self.buf = rest;
        Ok(b)
    }

    fn read_u32(&mut self, path: &str) -> Result<u32, XtaskError> {
        let (v, rest) =
            U32::ref_from_prefix(self.buf).map_err(|_| XtaskError::BinCorpusVerify {
                path: path.into(),
                detail: "unexpected EOF reading u32".into(),
            })?;
        self.buf = rest;
        Ok(v.get())
    }

    fn read_str_bytes(&mut self, path: &str) -> Result<&'a [u8], XtaskError> {
        let len = u32_to_usize(self.read_u32(path)?);
        let (bytes, rest) =
            self.buf
                .split_at_checked(len)
                .ok_or_else(|| XtaskError::BinCorpusVerify {
                    path: path.into(),
                    detail: "unexpected EOF reading string".into(),
                })?;
        self.buf = rest;
        Ok(bytes)
    }

    fn skip_sub_op(
        &mut self,
        sub: SubOp,
        op3_table: &[Vec<SubOp>],
        path: &str,
    ) -> Result<(), XtaskError> {
        match sub {
            SubOp::FormulaOrF64 => {
                if self.read_u8(path)? == 0 {
                    self.skip(8, path)
                } else {
                    self.skip_tree_nodes(op3_table, path)
                }
            }
            SubOp::FormulaOrI32 => {
                if self.read_u8(path)? == 0 {
                    self.skip(4, path)
                } else {
                    self.skip_tree_nodes(op3_table, path)
                }
            }
            SubOp::FormulaOrBool | SubOp::FormulaOrU8 => {
                if self.read_u8(path)? == 0 {
                    self.skip(1, path)
                } else {
                    self.skip_tree_nodes(op3_table, path)
                }
            }
            SubOp::ArrayFormulaOrI32 => {
                let c = self.read_u32(path)?;
                for _ in 0..c {
                    self.skip_sub_op(SubOp::FormulaOrI32, op3_table, path)?;
                }
                Ok(())
            }
        }
    }

    fn skip_tree_nodes(&mut self, op3_table: &[Vec<SubOp>], path: &str) -> Result<(), XtaskError> {
        let count = self.read_u32(path)?;
        for _ in 0..count {
            let tag = self.read_u8(path)?;
            match tag {
                0 | 5 | 6 | 7 => self.skip(4, path)?,
                1 => self.skip(8, path)?,
                2 => self.skip(1, path)?,
                3 => {
                    let op = usize::from(self.read_u8(path)?);
                    let subs = op3_table
                        .get(op)
                        .ok_or_else(|| XtaskError::BinCorpusVerify {
                            path: path.into(),
                            detail: format!("invalid AffixOp3 {op}").into(),
                        })?;
                    for &s in subs {
                        self.skip_sub_op(s, op3_table, path)?;
                    }
                }
                4 => {
                    let _buckets = self.read_u32(path)?;
                    let c = u32_to_usize(self.read_u32(path)?);
                    self.skip(c * 12, path)?;
                }
                8 => {}
                other => {
                    return Err(XtaskError::BinCorpusVerify {
                        path: path.into(),
                        detail: format!("invalid TreeNode tag {other}").into(),
                    });
                }
            }
        }
        Ok(())
    }

    fn skip_affix(&mut self, op3_table: &[Vec<SubOp>], path: &str) -> Result<(), XtaskError> {
        let _id = self.read_u32(path)?;
        if self.read_u8(path)? != 0 {
            let roots = self.read_u32(path)?;
            for _ in 0..roots {
                self.skip_sub_op(SubOp::FormulaOrF64, op3_table, path)?;
            }
        }
        Ok(())
    }

    fn skip_wire_type(
        &mut self,
        wt: &WireType,
        op3_table: &[Vec<SubOp>],
        action_table: &[Vec<WireType>],
        path: &str,
    ) -> Result<bool, XtaskError> {
        match wt {
            WireType::Bool => self.skip(1, path)?,
            WireType::I32 => self.skip(4, path)?,
            WireType::F64 => self.skip(8, path)?,
            WireType::Str | WireType::Bytes => {
                let bytes = self.read_str_bytes(path)?;
                return Ok(core::str::from_utf8(bytes).is_ok());
            }
            WireType::VecDim => {
                let dim = self.read_u8(path)?;
                self.skip(if dim == 3 { 24 } else { 16 }, path)?;
            }
            WireType::AffixEntry => self.skip_affix(op3_table, path)?,
            WireType::ActionData => {
                let tag = self.read_u32(path)?.cast_signed();
                if tag != 0 {
                    let idx =
                        usize::try_from(tag - 1000).map_err(|_| XtaskError::BinCorpusVerify {
                            path: path.into(),
                            detail: format!("invalid ActionData tag {tag}").into(),
                        })?;
                    let cols =
                        action_table
                            .get(idx)
                            .ok_or_else(|| XtaskError::BinCorpusVerify {
                                path: path.into(),
                                detail: format!("ActionData tag {tag} out of range").into(),
                            })?;
                    for k in cols {
                        self.skip_wire_type(k, op3_table, action_table, path)?;
                    }
                }
            }
            WireType::Array(elem) => {
                for _ in 0..self.read_u32(path)? {
                    self.skip_wire_type(elem, op3_table, action_table, path)?;
                }
            }
            WireType::Map(k, val) => {
                let _b = self.read_u32(path)?;
                for _ in 0..self.read_u32(path)? {
                    match k {
                        MapKeyWire::I32 => self.skip(4, path)?,
                        MapKeyWire::Str => {
                            let _ = self.read_str_bytes(path)?;
                        }
                    }
                    self.skip_wire_type(val, op3_table, action_table, path)?;
                }
            }
            WireType::Struct(fields) => {
                for f in fields {
                    self.skip_wire_type(f, op3_table, action_table, path)?;
                }
            }
        }
        Ok(true)
    }
}

fn verify_hierarchy_slice(
    buf: &[u8],
    key_schema: ExtractedKeySchema,
    cols: &mut [(WireType, Option<u32>)],
    op3_table: &[Vec<SubOp>],
    action_table: &[Vec<WireType>],
    path: &str,
) -> Result<(), XtaskError> {
    let mut cur = CorpusCursor::new(buf);
    let top_count = cur.read_u32(path)?;
    let depth = usize::from(key_schema.depth());
    let outer_is_str = matches!(
        key_schema,
        ExtractedKeySchema::Str1 | ExtractedKeySchema::StrInt(_)
    );
    let mut frames = arrayvec::ArrayVec::<u32, 4>::new();
    let push_res = frames.try_push(top_count);
    assert!(push_res.is_ok());

    while !frames.is_empty() {
        let level = frames.len() - 1;
        if frames[level] == 0 {
            frames.pop();
            continue;
        }
        frames[level] -= 1;
        if level == 0 && outer_is_str {
            let _ = cur.read_str_bytes(path)?;
        } else {
            cur.skip(4, path)?;
        }
        if level + 1 < depth {
            cur.skip(4, path)?;
            let child_count = cur.read_u32(path)?;
            let push_res = frames.try_push(child_count);
            assert!(push_res.is_ok());
        } else {
            for col in &mut *cols {
                let is_utf8 = cur.skip_wire_type(&col.0, op3_table, action_table, path)?;
                if col.0 == WireType::Str && !is_utf8 {
                    col.0 = WireType::Bytes;
                }
            }
        }
    }
    if !cur.buf.is_empty() {
        return Err(XtaskError::BinCorpusVerify {
            path: path.into(),
            detail: format!("{} trailing bytes remaining", cur.buf.len()).into(),
        });
    }
    Ok(())
}

fn verify_chunk_file(
    raw_buf: &[u8],
    dctx: &mut zstd_safe::DCtx<'static>,
    dec_buf: &mut Vec<u8>,
    schema: &mut ExtractedTableSchema,
    op3_table: &[Vec<SubOp>],
    action_table: &[Vec<WireType>],
    path: &str,
) -> Result<(), XtaskError> {
    let mut hdr_cur = CorpusCursor::new(raw_buf);
    let block_count = u32_to_usize(hdr_cur.read_u32(path)?);
    let _hash_buckets = hdr_cur.read_u32(path)?;
    let payload_off = 8 + block_count * 8;
    let payload = raw_buf
        .get(payload_off..)
        .ok_or_else(|| XtaskError::BinCorpusVerify {
            path: path.into(),
            detail: "block descriptor table out of bounds".into(),
        })?;

    for b in 0..block_count {
        let mut desc_cur = CorpusCursor::new(&raw_buf[8 + b * 8..8 + (b + 1) * 8]);
        let comp_off = u32_to_usize(desc_cur.read_u32(path)?);
        let uncomp_size = u32_to_usize(desc_cur.read_u32(path)?);
        let comp_end = if b + 1 < block_count {
            let mut next_cur = CorpusCursor::new(&raw_buf[8 + (b + 1) * 8..8 + (b + 2) * 8]);
            u32_to_usize(next_cur.read_u32(path)?)
        } else {
            payload.len()
        };
        let frame = payload
            .get(comp_off..comp_end)
            .ok_or_else(|| XtaskError::BinCorpusVerify {
                path: path.into(),
                detail: format!("invalid block {b} compressed range").into(),
            })?;
        if uncomp_size == 0 {
            if !frame.is_empty() {
                return Err(XtaskError::BinCorpusVerify {
                    path: path.into(),
                    detail: format!("non-empty frame for 0-byte block {b}").into(),
                });
            }
            continue;
        }
        dec_buf.resize(uncomp_size, 0);
        let written = dctx
            .decompress(dec_buf.as_mut_slice(), frame)
            .map_err(|code| XtaskError::BinCorpusVerify {
                path: path.into(),
                detail: format!("zstd decompress error {code} at block {b}").into(),
            })?;
        if written != uncomp_size {
            return Err(XtaskError::BinCorpusVerify {
                path: path.into(),
                detail: format!("decompressed size {written} != {uncomp_size}").into(),
            });
        }
        verify_hierarchy_slice(
            dec_buf,
            schema.key_schema,
            &mut schema.cols,
            op3_table,
            action_table,
            path,
        )?;
    }
    Ok(())
}

fn verify_all_chunk_files(
    vfs: &PakVfs,
    chunk_files: &[(String, FileId)],
    bin_schemas: &mut BinarySchemas,
    stem_to_config_name: &mut BTreeMap<String, String>,
    seen_chunk_stems: &mut BTreeSet<String>,
) -> Result<(), XtaskError> {
    let mut read_scratch = ReadScratch::with_limits(ScratchLimits::DEFAULT);
    let mut raw_buf = Vec::new();
    let mut dec_buf = Vec::new();
    let mut dctx = zstd_safe::DCtx::default();

    for (path, fid) in chunk_files {
        let fname = match path.rsplit_once('/') {
            Some((_, f)) => f,
            None => path.as_str(),
        };
        let raw_config = match fname.strip_suffix("_ChunkA0.bin") {
            Some(c) => strip_locale_suffix(c),
            None => fname,
        };
        let norm_stem = normalize_stem(fname);
        seen_chunk_stems.insert(norm_stem.clone());
        stem_to_config_name
            .entry(norm_stem.clone())
            .or_insert_with(|| raw_config.to_owned());
        let schema = bin_schemas
            .chunk_schemas
            .get_mut(&norm_stem)
            .ok_or_else(|| XtaskError::BinCorpusVerify {
                path: path.clone().into(),
                detail: format!("missing ChunkA0 schema for stem `{norm_stem}`").into(),
            })?;
        let size = usize::try_from(vfs.file_meta(*fid).uncompressed_bytes()).map_err(|_| {
            XtaskError::BinCorpusVerify {
                path: path.clone().into(),
                detail: "file size overflows usize".into(),
            }
        })?;
        raw_buf.resize(size, 0);
        vfs.read_into(*fid, &mut read_scratch, &mut raw_buf)?;
        verify_chunk_file(
            &raw_buf,
            &mut dctx,
            &mut dec_buf,
            schema,
            &bin_schemas.op3_table,
            &bin_schemas.action_table,
            path,
        )?;
    }
    Ok(())
}

fn verify_all_map_files(
    vfs: &PakVfs,
    map_files: &[(String, FileId)],
    bin_schemas: &mut BinarySchemas,
    stem_to_config_name: &mut BTreeMap<String, String>,
    seen_chunk_stems: &BTreeSet<String>,
) -> Result<(), XtaskError> {
    let mut read_scratch = ReadScratch::with_limits(ScratchLimits::DEFAULT);
    let mut raw_buf = Vec::new();

    for (path, fid) in map_files {
        let fname = match path.rsplit_once('/') {
            Some((_, f)) => f,
            None => path.as_str(),
        };
        let raw_config = match fname.strip_suffix(".bin") {
            Some(c) => c,
            None => fname,
        };
        let norm_stem = normalize_stem(fname);
        stem_to_config_name
            .entry(norm_stem.clone())
            .or_insert_with(|| raw_config.to_owned());
        if !bin_schemas.map_schemas.contains_key(&norm_stem) {
            let promoted = if seen_chunk_stems.contains(&norm_stem) {
                bin_schemas.chunk_schemas.get(&norm_stem).cloned()
            } else {
                bin_schemas.chunk_schemas.remove(&norm_stem)
            };
            if let Some(mut map_schema) = promoted {
                map_schema.is_map_table = true;
                bin_schemas
                    .map_schemas
                    .insert(norm_stem.clone(), map_schema);
            }
        }
        let schema = bin_schemas.map_schemas.get_mut(&norm_stem).ok_or_else(|| {
            XtaskError::BinCorpusVerify {
                path: path.clone().into(),
                detail: format!("missing Map schema for stem `{norm_stem}`").into(),
            }
        })?;
        let size = usize::try_from(vfs.file_meta(*fid).uncompressed_bytes()).map_err(|_| {
            XtaskError::BinCorpusVerify {
                path: path.clone().into(),
                detail: "file size overflows usize".into(),
            }
        })?;
        raw_buf.resize(size, 0);
        vfs.read_into(*fid, &mut read_scratch, &mut raw_buf)?;
        let body = raw_buf
            .get(4..)
            .ok_or_else(|| XtaskError::BinCorpusVerify {
                path: path.clone().into(),
                detail: "truncated Map .bin header".into(),
            })?;
        verify_hierarchy_slice(
            body,
            schema.key_schema,
            &mut schema.cols,
            &bin_schemas.op3_table,
            &bin_schemas.action_table,
            path,
        )?;
    }
    Ok(())
}

fn verify_corpus_and_collect_stems(
    vfs: &PakVfs,
    bin_schemas: &mut BinarySchemas,
) -> Result<BTreeMap<String, String>, XtaskError> {
    let mut stem_to_config_name = BTreeMap::<String, String>::new();
    let mut seen_chunk_stems = BTreeSet::<String>::new();
    let mut path_buf = Vec::new();
    let mut chunk_files = Vec::<(String, FileId)>::new();
    let mut map_files = Vec::<(String, FileId)>::new();

    for fid in vfs.iter_files() {
        path_buf.clear();
        vfs.write_file_path(fid, &mut path_buf);
        let s = core::str::from_utf8(&path_buf)?.to_owned();
        if s.ends_with("_ChunkA0.bin") {
            chunk_files.push((s, fid));
        } else if s.contains("/Maps/") && s.ends_with(".bin") && !s.ends_with("_idx.bin") {
            map_files.push((s, fid));
        }
    }
    chunk_files.sort_by(|a, b| a.0.cmp(&b.0));
    map_files.sort_by(|a, b| a.0.cmp(&b.0));

    verify_all_chunk_files(
        vfs,
        &chunk_files,
        bin_schemas,
        &mut stem_to_config_name,
        &mut seen_chunk_stems,
    )?;
    verify_all_map_files(
        vfs,
        &map_files,
        bin_schemas,
        &mut stem_to_config_name,
        &seen_chunk_stems,
    )?;
    Ok(stem_to_config_name)
}

fn default_config_stem_from_cpp(cpp_name: &str) -> String {
    let base = match cpp_name.strip_suffix("Info.cpp") {
        Some(b) => b,
        None => cpp_name,
    };
    format!("config_{}", to_snake_case(base))
}

fn resolve_bin_tables_out_dir() -> Result<PathBuf, XtaskError> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or(XtaskError::Usage {
            message: "failed to resolve workspace root from CARGO_MANIFEST_DIR",
        })?;
    Ok(workspace_root.join("crates/parsers/src/bin/tables"))
}

fn build_domain_specs(
    bin_schemas: &BinarySchemas,
    stem_to_config: &BTreeMap<String, String>,
    lua_structs: &BTreeMap<String, (String, Vec<crate::bincfg::luabridge::LuaFieldInfo>)>,
) -> BTreeMap<&'static str, Vec<BinTableSpec>> {
    let mut domains: BTreeMap<&'static str, Vec<BinTableSpec>> = BTreeMap::new();
    let all_extracted = bin_schemas
        .chunk_schemas
        .values()
        .chain(bin_schemas.map_schemas.values());

    for schema in all_extracted {
        let config_stem = match stem_to_config.get(&schema.stem) {
            Some(s) => s.clone(),
            None => default_config_stem_from_cpp(&schema.cpp_name),
        };
        let pascal = to_pascal_case(strip_config_prefix(&config_stem));
        let lua_fields = lua_structs.get(&schema.stem).map(|(_, f)| f.as_slice());
        let cols = resolve_column_names(&schema.cols, lua_fields);
        let domain = classify_bin_domain(&config_stem, schema.is_map_table);
        let shared_with_chunk_domain =
            if schema.is_map_table && bin_schemas.chunk_schemas.contains_key(&schema.stem) {
                Some(classify_bin_domain(&config_stem, false))
            } else {
                None
            };
        domains.entry(domain).or_default().push(BinTableSpec {
            config_stem,
            pascal,
            is_map_table: schema.is_map_table,
            shared_with_chunk_domain,
            key_schema: schema.key_schema,
            cols,
        });
    }
    for specs in domains.values_mut() {
        specs.sort_by(|a, b| a.config_stem.cmp(&b.config_stem));
    }
    domains
}

/// Extracts `.bin` schemas from `torchlight_infinite.exe`, verifies the `.pak` `.bin` corpus,
/// and writes or checks `crates/parsers/src/bin/tables/`.
pub fn run_codegen(mode: CodegenMode) -> Result<(), XtaskError> {
    let game_dir = std::env::var("TLI_GAME_DIR").map_err(|source| XtaskError::EnvVar {
        name: "TLI_GAME_DIR",
        source,
    })?;
    let key_hex = std::env::var("TLI_AES_KEY").map_err(|source| XtaskError::EnvVar {
        name: "TLI_AES_KEY",
        source,
    })?;
    let exe_path = Path::new(&game_dir).join("UE_game/Binaries/Win64/torchlight_infinite.exe");
    let pe = PeImage::load(&exe_path)?;
    let mut bin_schemas = extract_all_binary_schemas(&pe)?;
    let lua_structs = extract_luabridge_structs(&pe)?;

    let key = AesKey::try_from_hex(key_hex.trim())?;
    let vfs = PakVfs::open_tli_game_dir(Path::new(&game_dir), Some(key))?;
    let stem_to_config = verify_corpus_and_collect_stems(&vfs, &mut bin_schemas)?;
    let domains = build_domain_specs(&bin_schemas, &stem_to_config, &lua_structs);
    let reg = SubStructRegistry::from_schemas(&domains, &bin_schemas.action_table);

    let out_dir = resolve_bin_tables_out_dir()?;
    if mode == CodegenMode::Write {
        fs::create_dir_all(&out_dir)?;
    }

    emit_substructs_rs(&out_dir, &reg, mode)?;
    emit_affix_ops_rs(&out_dir, &bin_schemas.op3_table, mode)?;
    emit_action_data_rs(&out_dir, &bin_schemas.action_table, &reg, mode)?;

    let mut all_domains = BTreeSet::new();
    let mut expected_files = BTreeSet::from([
        "action_data.rs".to_owned(),
        "affix_ops.rs".to_owned(),
        "mod.rs".to_owned(),
        "substructs.rs".to_owned(),
    ]);
    let mut total_tables = 0usize;
    for (domain, specs) in &domains {
        total_tables += specs.len();
        all_domains.insert(*domain);
        expected_files.insert(format!("{domain}.rs"));
        emit_bin_domain_module(&out_dir, domain, specs, &reg, mode)?;
    }
    emit_bin_mod_rs(&out_dir, &all_domains, mode)?;
    verify_or_prune_stale_rs_files(&out_dir, &expected_files, mode)?;

    let action = match mode {
        CodegenMode::Write => "Generated",
        CodegenMode::Check => "Verified",
    };
    println!(
        "{action} {} .bin domain modules ({total_tables} total tables) in {}",
        all_domains.len(),
        out_dir.display()
    );
    Ok(())
}

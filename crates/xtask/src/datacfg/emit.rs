//! Deterministic Rust code emitter and atomic file writer for `DataCfg` domain modules.

use crate::bounded_len_u32;
use crate::datacfg::domains::{infer_kv_field_names, to_pascal_case, to_snake_case};
use crate::datacfg::schema::{
    ColType, LuaTableSpec, PreloadKind, PreloadSpec, RecordType, ScalarOrList, ScalarType,
};
use crate::error::XtaskError;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Write as _;
use std::path::Path;

/// Controls whether `xtask codegen` writes generated files or verifies them in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodegenMode {
    /// Atomically write generated `.rs` files to disk.
    Write,
    /// Verify that existing `.rs` files on disk match generated output byte-for-byte.
    Check,
}

/// Atomically writes `content` to `path` (`Write`) or verifies `path` matches `content` (`Check`).
pub fn write_or_check_file(
    path: &Path,
    content: &str,
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    match mode {
        CodegenMode::Write => {
            let tmp_path = path.with_extension("rs.tmp");
            let mut file = File::create(&tmp_path)?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
            drop(file);
            fs::rename(&tmp_path, path)?;
            Ok(())
        }
        CodegenMode::Check => {
            let Ok(existing) = fs::read_to_string(path) else {
                return Err(XtaskError::GeneratedFileDrift {
                    path: path.display().to_string().into(),
                });
            };
            if existing != content {
                return Err(XtaskError::GeneratedFileDrift {
                    path: path.display().to_string().into(),
                });
            }
            Ok(())
        }
    }
}

struct HelperHeader<'a> {
    stem: &'a str,
    col: &'a str,
    struct_name: &'a str,
    derive_line: &'a str,
}

fn emit_kv_helper_struct(
    code: &mut String,
    hdr: &HelperHeader<'_>,
    key: ScalarOrList,
    val: ScalarOrList,
    tuple_elems: Option<&[ScalarOrList]>,
) {
    let key_is_str = matches!(key, ScalarOrList::Scalar(ScalarType::Str));
    let val_is_list = matches!(val, ScalarOrList::List { .. });
    let tuple_arity = tuple_elems.map(|elems| bounded_len_u32(elems.len()));
    let names = infer_kv_field_names(hdr.stem, hdr.col, key_is_str, val_is_list, tuple_arity);
    let k_name = &names.key_name;
    let v_names = &names.val_names;
    let k_rust = key.rust_type();
    code.push_str(&format!(
        "/// Entry for `{}.{}`.\n{}\npub struct {} {{\n    pub {k_name}: {k_rust},\n",
        hdr.stem, hdr.col, hdr.derive_line, hdr.struct_name
    ));
    if let Some(elems) = tuple_elems {
        assert_eq!(elems.len(), v_names.len());
        for (&ety, vname) in elems.iter().zip(v_names) {
            let ety_rust = ety.rust_type();
            code.push_str(&format!("    pub {vname}: {ety_rust},\n"));
        }
    } else {
        let v_rust = val.rust_type();
        code.push_str(&format!("    pub {}: {v_rust},\n", v_names[0]));
    }
    code.push_str("}\n\n");
    code.push_str(&format!(
        "impl FromEvalVal for {} {{\n    #[inline]\n    fn from_eval_val(\n        ctx: &EvalContext<'_, '_>,\n        val: EvalVal,\n        table: &'static str,\n        field: &'static str,\n    ) -> Result<Self, LuacError> {{\n",
        hdr.struct_name
    ));
    if tuple_elems.is_some() {
        let tuple_pat = v_names.join(", ");
        let let_line = format!(
            "        let ({k_name}, ({tuple_pat})) = ctx.decode_kv_pair(val, table, field)?;"
        );
        if let_line.len() <= 96 {
            code.push_str(&let_line);
            code.push('\n');
        } else {
            code.push_str(&format!(
                "        let ({k_name}, ({tuple_pat})) =\n            ctx.decode_kv_pair(val, table, field)?;\n"
            ));
        }
        code.push_str(&format!("        Ok(Self {{\n            {k_name},\n"));
        for vname in v_names {
            code.push_str(&format!("            {vname},\n"));
        }
        code.push_str("        })\n");
    } else {
        let vname = &v_names[0];
        let lit = format!("Self {{ {k_name}, {vname} }}");
        if lit.len() <= 27 {
            code.push_str(&format!(
                "        let ({k_name}, {vname}) = ctx.decode_kv_pair(val, table, field)?;\n        Ok({lit})\n"
            ));
        } else {
            code.push_str(&format!(
                "        let ({k_name}, {vname}) = ctx.decode_kv_pair(val, table, field)?;\n        Ok(Self {{\n            {k_name},\n            {vname},\n        }})\n"
            ));
        }
    }
    code.push_str("    }\n}\n\n");
}

fn push_string_record_arm(code: &mut String, k: &str, f_snake: &str) {
    let single =
        format!("                b{k:?} => out.{f_snake} = ctx.decode_field(v, table, field)?,");
    if single.len() <= 97 {
        code.push_str(&single);
        code.push('\n');
    } else {
        code.push_str(&format!(
            "                b{k:?} => {{\n                    out.{f_snake} = ctx.decode_field(v, table, field)?\n                }}\n"
        ));
    }
}

fn emit_string_helper_struct(
    code: &mut String,
    hdr: &HelperHeader<'_>,
    m: &BTreeMap<String, ScalarOrList>,
) {
    code.push_str(&format!(
        "/// Sub-record for `{}.{}`.\n{}\npub struct {} {{\n",
        hdr.stem, hdr.col, hdr.derive_line, hdr.struct_name
    ));
    for (k, &v) in m {
        let f_snake = to_snake_case(k);
        let v_rust = v.rust_type();
        code.push_str(&format!("    pub {f_snake}: {v_rust},\n"));
    }
    code.push_str("}\n\n");
    code.push_str(&format!(
        "impl FromEvalVal for {} {{\n    fn from_eval_val(\n        ctx: &EvalContext<'_, '_>,\n        val: EvalVal,\n        table: &'static str,\n        field: &'static str,\n    ) -> Result<Self, LuacError> {{\n        let mut out = Self::default();\n        for &(k, v) in ctx.table_map(val, table, field)? {{\n            match ctx.decode_str_key(k, table, field)? {{\n",
        hdr.struct_name
    ));
    for k in m.keys() {
        let f_snake = to_snake_case(k);
        push_string_record_arm(code, k, &f_snake);
    }
    code.push_str(
        "                other => return Err(ctx.unknown_field(table, other)),\n            }\n        }\n        Ok(out)\n    }\n}\n\n",
    );
}

fn emit_helper_struct(
    code: &mut String,
    stem: &str,
    col: &str,
    struct_name: &str,
    rec_ty: &RecordType,
) {
    let mut traits = vec!["Debug", "Clone"];
    if rec_ty.is_copy() {
        traits.push("Copy");
    }
    traits.push("PartialEq");
    if !rec_ty.has_float() {
        traits.push("Eq");
    }
    traits.push("Default");
    let derive_line = format!("#[derive({})]", traits.join(", "));
    let hdr = HelperHeader {
        stem,
        col,
        struct_name,
        derive_line: &derive_line,
    };
    match rec_ty {
        RecordType::KvRecord {
            key,
            val,
            tuple_elems,
        } => emit_kv_helper_struct(code, &hdr, *key, *val, tuple_elems.as_deref()),
        RecordType::StringRecord(m) => emit_string_helper_struct(code, &hdr, m),
    }
}

fn push_set_field_arm(code: &mut String, col: &str, field_snake: &str) {
    let single = format!(
        "            b{col:?} => self.{field_snake} = ctx.decode_field(val, Self::TABLE_NAME, {col:?})?,"
    );
    if single.len() <= 98 {
        code.push_str(&single);
        code.push('\n');
        return;
    }
    let block_single = format!(
        "                self.{field_snake} = ctx.decode_field(val, Self::TABLE_NAME, {col:?})?"
    );
    if block_single.len() <= 98 {
        code.push_str(&format!(
            "            b{col:?} => {{\n{block_single}\n            }}\n"
        ));
    } else {
        code.push_str(&format!(
            "            b{col:?} => {{\n                self.{field_snake} =\n                    ctx.decode_field(val, Self::TABLE_NAME, {col:?})?\n            }}\n"
        ));
    }
}

fn emit_row_impl(code: &mut String, row_name: &str, spec: &LuaTableSpec) {
    let col_names: Vec<&String> = spec.cols.keys().collect();
    if col_names.len() > 12 {
        code.push_str(&format!("impl {row_name} {{\n"));
        for (idx, chunk) in col_names.chunks(12).enumerate() {
            code.push_str(&format!(
                "    #[inline(never)]\n    fn set_field_{idx}(\n        &mut self,\n        ctx: &EvalContext<'_, '_>,\n        field: &[u8],\n        val: EvalVal,\n    ) -> Result<bool, LuacError> {{\n        match field {{\n"
            ));
            for col in chunk {
                let field_snake = to_snake_case(col);
                push_set_field_arm(code, col, &field_snake);
            }
            code.push_str(
                "            _ => return Ok(false),\n        }\n        Ok(true)\n    }\n\n",
            );
        }
        assert!(code.ends_with("\n\n"));
        code.pop();
        code.push_str("}\n\n");
    }

    code.push_str(&format!(
        "impl DataCfgRow for {row_name} {{\n    const TABLE_NAME: &'static str = {:?};\n\n    #[inline(never)]\n    fn set_field(\n        &mut self,\n        ctx: &EvalContext<'_, '_>,\n        field: &[u8],\n        val: EvalVal,\n    ) -> Result<(), LuacError> {{\n",
        spec.stem
    ));
    if col_names.len() > 12 {
        let count = col_names.chunks(12).len();
        let checks: Vec<String> = (0..count)
            .map(|idx| format!("self.set_field_{idx}(ctx, field, val)?"))
            .collect();
        let single_if = format!("        if {} {{", checks.join(" || "));
        if count <= 2 && single_if.len() <= 100 {
            code.push_str(&format!(
                "{single_if}\n            return Ok(());\n        }}\n        Err(ctx.unknown_field(Self::TABLE_NAME, field))\n    }}\n}}\n\n"
            ));
        } else {
            code.push_str(&format!(
                "        if {}\n        {{\n            return Ok(());\n        }}\n        Err(ctx.unknown_field(Self::TABLE_NAME, field))\n    }}\n}}\n\n",
                checks.join("\n            || ")
            ));
        }
    } else {
        code.push_str("        match field {\n");
        for col in &col_names {
            let field_snake = to_snake_case(col);
            push_set_field_arm(code, col, &field_snake);
        }
        code.push_str(
            "            _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),\n        }\n        Ok(())\n    }\n}\n\n",
        );
    }
}

fn emit_table_spec(code: &mut String, spec: &LuaTableSpec) -> Result<(), XtaskError> {
    let mut col_helper_names: BTreeMap<String, String> = BTreeMap::new();
    for (col, ty) in &spec.cols {
        if let Some((rec_ty, is_collection)) = ty.nested_record() {
            let col_pascal = to_pascal_case(col);
            let struct_name = if is_collection {
                format!("{}{col_pascal}Entry", spec.pascal)
            } else {
                format!("{}{col_pascal}", spec.pascal)
            };
            emit_helper_struct(code, &spec.stem, col, &struct_name, rec_ty);
            col_helper_names.insert(col.clone(), struct_name);
        }
    }

    let row_name = format!("Config{}Row", spec.pascal);
    let table_alias = format!("Config{}Table", spec.pascal);
    let row_derive = if spec.cols.values().any(ColType::has_float) {
        "#[derive(Debug, Clone, PartialEq, Default)]"
    } else {
        "#[derive(Debug, Clone, PartialEq, Eq, Default)]"
    };
    code.push_str(&format!(
        "/// Row record for `{}.out`.\n{row_derive}\npub struct {row_name} {{\n",
        spec.stem
    ));
    for (col, ty) in &spec.cols {
        let field_snake = to_snake_case(col);
        let helper_name = col_helper_names.get(col).map(String::as_str);
        let field_rust = ty.rust_type(helper_name, &spec.stem)?;
        code.push_str(&format!("    pub {field_snake}: {field_rust},\n"));
    }
    code.push_str("}\n\n");

    emit_row_impl(code, &row_name, spec);

    let parse_fn = if spec.is_3d {
        "parse_luatable_datacfg_3d"
    } else {
        "parse_luatable_datacfg"
    };
    let type_line = format!(
        "pub type {table_alias} = DataTable<{}, {row_name}>;",
        spec.key_rust
    );
    if type_line.len() <= 100 {
        code.push_str(&format!(
            "/// Typed `DataTable` for `{0}.out`.\n{type_line}\n\n/// Parses `{0}.out` into a [`{table_alias}`].\npub fn parse_{0}(\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Result<{table_alias}, LuacError> {{\n    {parse_fn}(buf, scratch)\n}}\n\n",
            spec.stem
        ));
    } else {
        code.push_str(&format!(
            "/// Typed `DataTable` for `{0}.out`.\npub type {table_alias} =\n    DataTable<{1}, {row_name}>;\n\n/// Parses `{0}.out` into a [`{table_alias}`].\npub fn parse_{0}(\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Result<{table_alias}, LuacError> {{\n    {parse_fn}(buf, scratch)\n}}\n\n",
            spec.stem, spec.key_rust
        ));
    }
    Ok(())
}

fn push_verify_arm(code: &mut String, stem: &str, map_tail: &str) {
    let expr = format!("Some(parse_{stem}(buf, scratch).{map_tail})");
    let single = format!("        {stem:?} => {expr},");
    if single.len() <= 100 {
        code.push_str(&single);
        code.push('\n');
        return;
    }
    if 12 + expr.len() <= 100 {
        code.push_str(&format!(
            "        {stem:?} => {{\n            {expr}\n        }}\n"
        ));
        return;
    }
    let inner = format!("            parse_{stem}(buf, scratch).{map_tail},");
    if inner.len() <= 100 {
        code.push_str(&format!("        {stem:?} => Some(\n{inner}\n        ),\n"));
    } else {
        code.push_str(&format!(
            "        {stem:?} => Some(\n            parse_{stem}(buf, scratch)\n                .{map_tail},\n        ),\n"
        ));
    }
}

fn emit_domain_verifier(code: &mut String, domain: &str, specs: &[LuaTableSpec]) {
    if specs.len() > 14 {
        for (idx, chunk) in specs.chunks(14).enumerate() {
            code.push_str(&format!(
                "fn verify_{domain}_chunk_{idx}(\n    stem: &str,\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Option<Result<u32, LuacError>> {{\n    match stem {{\n"
            ));
            for spec in chunk {
                push_verify_arm(code, &spec.stem, "map(|t| t.len())");
            }
            code.push_str("        _ => None,\n    }\n}\n\n");
        }
        code.push_str(&format!(
            "pub(crate) fn verify_{domain}_datacfg_by_stem(\n    stem: &str,\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Option<Result<u32, LuacError>> {{\n"
        ));
        let count = specs.chunks(14).len();
        for idx in 0..count {
            code.push_str(&format!(
                "    if let Some(res) = verify_{domain}_chunk_{idx}(stem, buf, scratch) {{\n        return Some(res);\n    }}\n"
            ));
        }
        code.push_str("    None\n}\n");
    } else {
        code.push_str(&format!(
            "pub(crate) fn verify_{domain}_datacfg_by_stem(\n    stem: &str,\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Option<Result<u32, LuacError>> {{\n    match stem {{\n"
        ));
        for spec in specs {
            push_verify_arm(code, &spec.stem, "map(|t| t.len())");
        }
        code.push_str("        _ => None,\n    }\n}\n");
    }
}

fn push_datacfg_imports(code: &mut String, datacfg_imports: &[&str]) {
    let single_use = format!(
        "use crate::luac::datacfg::{{{}}};\n",
        datacfg_imports.join(", ")
    );
    if single_use.len() - 1 <= 100 {
        code.push_str(&single_use);
    } else {
        code.push_str("use crate::luac::datacfg::{\n");
        let mut line = String::from("    ");
        for (i, item) in datacfg_imports.iter().enumerate() {
            let sep = if i + 1 == datacfg_imports.len() {
                ","
            } else {
                ", "
            };
            let next_piece = format!("{item}{sep}");
            if line.len() > 4 && line.len() + next_piece.trim_end().len() > 100 {
                code.push_str(line.trim_end());
                code.push('\n');
                line.clear();
                line.push_str("    ");
            }
            line.push_str(&next_piece);
        }
        code.push_str(line.trim_end());
        code.push_str("\n};\n");
    }
}

/// Generates or verifies a single `<domain>.rs` module in `out_dir`.
pub fn emit_domain_module(
    out_dir: &Path,
    domain: &str,
    specs: &[LuaTableSpec],
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let mut code = format!(
        "//! Auto-generated `LuaTable/DataCfg` types and parsers for the `{domain}` domain.\n\n"
    );
    let any_3d = specs.iter().any(|s| s.is_3d);
    let any_helper = specs
        .iter()
        .any(|s| s.cols.values().any(|t| t.nested_record().is_some()));
    let any_fx_map = specs
        .iter()
        .any(|s| s.cols.values().any(ColType::uses_fx_hash_map));
    let any_int_or_list = specs
        .iter()
        .any(|s| s.cols.values().any(ColType::uses_int_or_list));

    let mut datacfg_imports = vec![
        "DataCfgRow",
        "DataCfgScratch",
        "DataTable",
        "EvalContext",
        "EvalVal",
    ];
    if any_helper {
        datacfg_imports.push("FromEvalVal");
    }
    datacfg_imports.push("parse_luatable_datacfg");
    if any_3d {
        datacfg_imports.push("parse_luatable_datacfg_3d");
    }
    push_datacfg_imports(&mut code, &datacfg_imports);
    code.push_str("use crate::luac::error::LuacError;\n");
    if any_int_or_list {
        code.push_str("use crate::luac::types::IntOrIntList;\n");
    }
    if any_fx_map {
        code.push_str("use rustc_hash::FxHashMap;\n");
    }
    code.push('\n');

    for spec in specs {
        emit_table_spec(&mut code, spec)?;
    }

    emit_domain_verifier(&mut code, domain, specs);
    write_or_check_file(&out_dir.join(format!("{domain}.rs")), &code, mode)
}

/// Generates or verifies `preload.rs` in `out_dir`.
pub fn emit_preload_module(
    out_dir: &Path,
    preload_specs: &[PreloadSpec],
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let mut code = String::from(
        "//! Auto-generated `LuaPreLoadTable/DataCfg` preload table types and parsers.\n\nuse crate::bounded_len_u32;\nuse crate::luac::datacfg::{\n    DataCfgScratch, PreloadTable, parse_preload_list, parse_preload_map, parse_preload_map_2d,\n};\nuse crate::luac::error::LuacError;\n",
    );
    if preload_specs.iter().any(|s| s.uses_int_or_list) {
        code.push_str("use crate::luac::types::IntOrIntList;\n");
    }
    code.push('\n');

    for p in preload_specs {
        let alias = format!("{}Table", p.pascal);
        let (doc_kind, rhs, parse_call) = match &p.kind {
            PreloadKind::List(elem_ty) => (
                "Typed preload list",
                format!("Box<[{elem_ty}]>"),
                format!("parse_preload_list(buf, scratch, {:?})", p.stem),
            ),
            PreloadKind::Map1D(k_ty, v_ty) => (
                "Typed preload table",
                format!("PreloadTable<{k_ty}, {v_ty}>"),
                format!("parse_preload_map(buf, scratch, {:?})", p.stem),
            ),
            PreloadKind::Map2D(k1_ty, k2_ty, v_ty) => (
                "Typed 2D preload table",
                format!("PreloadTable<({k1_ty}, {k2_ty}), {v_ty}>"),
                format!("parse_preload_map_2d(buf, scratch, {:?})", p.stem),
            ),
        };
        let type_line = format!("pub type {alias} = {rhs};");
        if type_line.len() <= 100 {
            code.push_str(&format!(
                "/// {doc_kind} for `{0}.out`.\n{type_line}\n\n/// Parses `{0}.out` into a [`{alias}`].\npub fn parse_{0}(\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Result<{alias}, LuacError> {{\n    {parse_call}\n}}\n\n",
                p.stem
            ));
        } else {
            code.push_str(&format!(
                "/// {doc_kind} for `{0}.out`.\npub type {alias} =\n    {rhs};\n\n/// Parses `{0}.out` into a [`{alias}`].\npub fn parse_{0}(\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Result<{alias}, LuacError> {{\n    {parse_call}\n}}\n\n",
                p.stem
            ));
        }
    }

    code.push_str(
        "pub(crate) fn verify_preload_datacfg_by_stem(\n    stem: &str,\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Option<Result<u32, LuacError>> {\n    match stem {\n",
    );
    for p in preload_specs {
        let map_tail = if matches!(p.kind, PreloadKind::List(_)) {
            "map(|t| bounded_len_u32(t.len()))"
        } else {
            "map(|t| t.len())"
        };
        push_verify_arm(&mut code, &p.stem, map_tail);
    }
    code.push_str("        _ => None,\n    }\n}\n");
    write_or_check_file(&out_dir.join("preload.rs"), &code, mode)
}

/// Generates or verifies `mod.rs` in `out_dir`.
pub fn emit_mod_rs(
    out_dir: &Path,
    all_modules: &BTreeSet<&'static str>,
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let mut mod_code = String::from(
        "//! Strongly-typed `DataCfg` and `LuaPreLoadTable` parsers grouped into 23 domain modules.\n\n",
    );
    for m in all_modules {
        mod_code.push_str(&format!("pub mod {m};\n"));
    }
    mod_code.push('\n');
    for m in all_modules {
        mod_code.push_str(&format!("pub use {m}::*;\n"));
    }
    mod_code.push_str(&format!(
        "\nuse crate::luac::datacfg::DataCfgScratch;\nuse crate::luac::error::{{DataCfgError, LuacError}};\n\ntype DomainVerifier = fn(&str, &mut [u8], &mut DataCfgScratch) -> Option<Result<u32, LuacError>>;\n\nconst DOMAIN_VERIFIERS: [DomainVerifier; {}] = [\n",
        all_modules.len()
    ));
    for m in all_modules {
        mod_code.push_str(&format!("    {m}::verify_{m}_datacfg_by_stem,\n"));
    }
    mod_code.push_str(
        "];\n\n/// Evaluates a `DataCfg` `.out` buffer by file stem (`config_*`) and returns its row/entry count.\npub fn verify_all_datacfg_by_stem(\n    stem: &str,\n    buf: &mut [u8],\n    scratch: &mut DataCfgScratch,\n) -> Result<u32, LuacError> {\n    for verifier in DOMAIN_VERIFIERS {\n        if let Some(res) = verifier(stem, buf, scratch) {\n            return res;\n        }\n    }\n    Err(DataCfgError::UnknownField {\n        table: \"<datacfg_corpus>\",\n        field: stem.into(),\n    }\n    .into())\n}\n",
    );
    write_or_check_file(&out_dir.join("mod.rs"), &mod_code, mode)
}

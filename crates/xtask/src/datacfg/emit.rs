//! Deterministic Rust AST code emitter for `crates/parsers/src/luac/tables/` using
//! `proc_macro2`, `quote`, `syn`, and `prettyplease`.

use crate::bounded_len_u32;
use crate::datacfg::domains::{infer_kv_field_names, to_snake_case};
use crate::datacfg::schema::{
    ColType, LuaTableSpec, PreloadKind, PreloadSpec, RecordType, ScalarOrList, ScalarType,
};
use crate::error::XtaskError;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote};
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

/// Constructs a `proc_macro2::Ident`, using `Ident::new_raw` when `name` has an `r#` prefix.
#[must_use]
pub fn make_ident(name: &str) -> Ident {
    if let Some(raw) = name.strip_prefix("r#") {
        Ident::new_raw(raw, Span::call_site())
    } else {
        Ident::new(name, Span::call_site())
    }
}

/// Emits `use #prefix::#single;` or `use #prefix::{#(#many),*};` when `idents` is non-empty.
#[must_use]
pub fn emit_use_group(prefix: &TokenStream, idents: &[Ident]) -> TokenStream {
    match idents {
        [] => TokenStream::new(),
        [single] => quote! { use #prefix::#single; },
        many => quote! { use #prefix::{#(#many),*}; },
    }
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

/// Parses `tokens` into a `syn::File`, formats it with `prettyplease`, and writes or checks `path`.
pub fn write_or_check_tokens(
    path: &Path,
    tokens: TokenStream,
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let syntax_tree: syn::File = syn::parse2(tokens)?;
    let formatted = prettyplease::unparse(&syntax_tree);
    write_or_check_file(path, &formatted, mode)
}

/// Prunes (`Write`) or rejects (`Check`) any unexpected `.rs` files in `out_dir` not in `expected_files`.
pub fn verify_or_prune_stale_rs_files(
    out_dir: &Path,
    expected_files: &BTreeSet<String>,
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let entries = fs::read_dir(out_dir)?;
    for entry_res in entries {
        let entry = entry_res?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Some(fname) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !expected_files.contains(fname) {
            match mode {
                CodegenMode::Write => fs::remove_file(&path)?,
                CodegenMode::Check => {
                    return Err(XtaskError::GeneratedFileDrift {
                        path: path.display().to_string().into(),
                    });
                }
            }
        }
    }
    Ok(())
}

struct HelperHeader<'a> {
    stem: &'a str,
    col: &'a str,
    struct_ident: Ident,
    derives: TokenStream,
}

fn emit_kv_helper_struct(
    hdr: &HelperHeader<'_>,
    key: ScalarOrList,
    val: ScalarOrList,
    tuple_elems: Option<&[ScalarOrList]>,
) -> TokenStream {
    let key_is_str = matches!(key, ScalarOrList::Scalar(ScalarType::Str));
    let val_is_list = matches!(val, ScalarOrList::List { .. });
    let tuple_arity = tuple_elems.map(|elems| bounded_len_u32(elems.len()));
    let names = infer_kv_field_names(hdr.stem, hdr.col, key_is_str, val_is_list, tuple_arity);

    let mut raw_fields: Vec<(&str, ScalarOrList)> = Vec::with_capacity(1 + names.val_names.len());
    raw_fields.push((&names.key_name, key));
    if let Some(elems) = tuple_elems {
        assert_eq!(elems.len(), names.val_names.len());
        for (&ety, vname) in elems.iter().zip(&names.val_names) {
            raw_fields.push((vname.as_str(), ety));
        }
    } else {
        raw_fields.push((names.val_names[0].as_str(), val));
    }

    let mut ordered: Vec<(usize, &str, ScalarOrList)> = raw_fields
        .into_iter()
        .enumerate()
        .map(|(i, (n, t))| (i, n, t))
        .collect();
    ordered.sort_by_key(|&(i, _, t)| (t.align_tier(), i));

    let field_defs = ordered.iter().map(|&(_, fname, fty)| {
        let f_ident = make_ident(fname);
        let ty = fty.rust_type_tokens();
        quote! { pub #f_ident: #ty }
    });
    let ordered_idents = ordered.iter().map(|&(_, fname, _)| make_ident(fname));
    let k_ident = make_ident(&names.key_name);
    let decode_stmt = if tuple_elems.is_some() {
        let v_idents = names.val_names.iter().map(|n| make_ident(n));
        quote! {
            let (#k_ident, (#(#v_idents),*)) = ctx.decode_kv_pair(val, table, field)?;
        }
    } else {
        let v_ident = make_ident(&names.val_names[0]);
        quote! {
            let (#k_ident, #v_ident) = ctx.decode_kv_pair(val, table, field)?;
        }
    };

    let doc = format!(" Entry for `{}.{}`.", hdr.stem, hdr.col);
    let struct_ident = &hdr.struct_ident;
    let derives = &hdr.derives;
    quote! {
        #[doc = #doc]
        #derives
        pub struct #struct_ident {
            #(#field_defs,)*
        }

        impl FromEvalVal for #struct_ident {
            #[inline]
            fn from_eval_val(
                ctx: &EvalContext<'_, '_>,
                val: EvalVal,
                table: &'static str,
                field: &'static str,
            ) -> Result<Self, LuacError> {
                #decode_stmt
                Ok(Self {
                    #(#ordered_idents,)*
                })
            }
        }
    }
}

fn emit_string_helper_struct(
    hdr: &HelperHeader<'_>,
    m: &BTreeMap<String, ScalarOrList>,
) -> TokenStream {
    let mut ordered: Vec<(&str, ScalarOrList)> = m.iter().map(|(k, &v)| (k.as_str(), v)).collect();
    ordered.sort_by_key(|&(k, v)| (v.align_tier(), k));

    let field_defs = ordered.iter().map(|&(k, v)| {
        let f_ident = make_ident(&to_snake_case(k));
        let v_ty = v.rust_type_tokens();
        quote! { pub #f_ident: #v_ty }
    });
    let arms = m.keys().map(|k| {
        let key_lit = Literal::byte_string(k.as_bytes());
        let f_ident = make_ident(&to_snake_case(k));
        quote! { #key_lit => out.#f_ident = ctx.decode_field(v, table, field)? }
    });

    let doc = format!(" Sub-record for `{}.{}`.", hdr.stem, hdr.col);
    let struct_ident = &hdr.struct_ident;
    let derives = &hdr.derives;
    quote! {
        #[doc = #doc]
        #derives
        pub struct #struct_ident {
            #(#field_defs,)*
        }

        impl FromEvalVal for #struct_ident {
            fn from_eval_val(
                ctx: &EvalContext<'_, '_>,
                val: EvalVal,
                table: &'static str,
                field: &'static str,
            ) -> Result<Self, LuacError> {
                let mut out = Self::default();
                for &(k, v) in ctx.table_map(val, table, field)? {
                    match ctx.decode_str_key(k, table, field)? {
                        #(#arms,)*
                        other => return Err(ctx.unknown_field(table, other)),
                    }
                }
                Ok(out)
            }
        }
    }
}

fn emit_helper_struct(
    stem: &str,
    col: &str,
    struct_name: &str,
    rec_ty: &RecordType,
) -> TokenStream {
    let derives = match (rec_ty.is_copy(), rec_ty.has_float()) {
        (true, true) => quote! { #[derive(Debug, Clone, Copy, PartialEq, Default)] },
        (true, false) => quote! { #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] },
        (false, true) => quote! { #[derive(Debug, Clone, PartialEq, Default)] },
        (false, false) => quote! { #[derive(Debug, Clone, PartialEq, Eq, Default)] },
    };
    let hdr = HelperHeader {
        stem,
        col,
        struct_ident: make_ident(struct_name),
        derives,
    };
    match rec_ty {
        RecordType::KvRecord {
            key,
            val,
            tuple_elems,
        } => emit_kv_helper_struct(&hdr, *key, *val, tuple_elems.as_deref()),
        RecordType::StringRecord(m) => emit_string_helper_struct(&hdr, m),
    }
}

fn emit_set_field_arm(col: &str) -> TokenStream {
    let key_lit = Literal::byte_string(col.as_bytes());
    let f_ident = make_ident(&to_snake_case(col));
    quote! {
        #key_lit => self.#f_ident = ctx.decode_field(val, Self::TABLE_NAME, #col)?
    }
}

fn emit_row_impl(row_ident: &Ident, spec: &LuaTableSpec) -> TokenStream {
    let col_names: Vec<&String> = spec.cols.keys().collect();
    let stem = &spec.stem;

    if col_names.len() > 12 {
        let chunk_fns = col_names.chunks(12).enumerate().map(|(idx, chunk)| {
            let fn_ident = format_ident!("set_field_{idx}");
            let arms = chunk.iter().map(|col| emit_set_field_arm(col));
            quote! {
                #[inline(never)]
                fn #fn_ident(
                    &mut self,
                    ctx: &EvalContext<'_, '_>,
                    field: &[u8],
                    val: EvalVal,
                ) -> Result<bool, LuacError> {
                    match field {
                        #(#arms,)*
                        _ => return Ok(false),
                    }
                    Ok(true)
                }
            }
        });
        let chunk_calls = (0..col_names.chunks(12).len()).map(|idx| {
            let fn_ident = format_ident!("set_field_{idx}");
            quote! { self.#fn_ident(ctx, field, val)? }
        });
        quote! {
            impl #row_ident {
                #(#chunk_fns)*
            }

            impl DataCfgRow for #row_ident {
                const TABLE_NAME: &'static str = #stem;

                #[inline(never)]
                fn set_field(
                    &mut self,
                    ctx: &EvalContext<'_, '_>,
                    field: &[u8],
                    val: EvalVal,
                ) -> Result<(), LuacError> {
                    if #(#chunk_calls)||* {
                        return Ok(());
                    }
                    Err(ctx.unknown_field(Self::TABLE_NAME, field))
                }
            }
        }
    } else {
        let arms = col_names.iter().map(|col| emit_set_field_arm(col));
        quote! {
            impl DataCfgRow for #row_ident {
                const TABLE_NAME: &'static str = #stem;

                #[inline(never)]
                fn set_field(
                    &mut self,
                    ctx: &EvalContext<'_, '_>,
                    field: &[u8],
                    val: EvalVal,
                ) -> Result<(), LuacError> {
                    match field {
                        #(#arms,)*
                        _ => return Err(ctx.unknown_field(Self::TABLE_NAME, field)),
                    }
                    Ok(())
                }
            }
        }
    }
}

fn emit_table_spec(spec: &LuaTableSpec) -> TokenStream {
    let helpers = spec.cols.iter().filter_map(|(col, ty)| {
        let (rec_ty, struct_name) = ty.nested_record(&spec.pascal, col)?;
        Some(emit_helper_struct(&spec.stem, col, &struct_name, rec_ty))
    });

    let row_ident = format_ident!("Config{}Row", spec.pascal);
    let table_ident = format_ident!("Config{}Table", spec.pascal);
    let parse_fn_ident = format_ident!("parse_{}", spec.stem);
    let row_derives = if spec.cols.values().any(ColType::has_float) {
        quote! { #[derive(Debug, Clone, PartialEq, Default)] }
    } else {
        quote! { #[derive(Debug, Clone, PartialEq, Eq, Default)] }
    };

    let mut ordered_cols: Vec<(&str, &ColType)> =
        spec.cols.iter().map(|(c, t)| (c.as_str(), t)).collect();
    ordered_cols.sort_by_key(|&(c, t)| (t.align_tier(), c));

    let field_defs = ordered_cols.iter().map(|&(col, ty)| {
        let f_ident = make_ident(&to_snake_case(col));
        let f_ty = ty.rust_type_tokens(&spec.pascal, col);
        quote! { pub #f_ident: #f_ty }
    });

    let row_impl = emit_row_impl(&row_ident, spec);
    let key_ty = spec.key.rust_type_tokens();
    let parse_call = if spec.is_3d {
        quote! { parse_luatable_datacfg_3d(buf, scratch) }
    } else {
        quote! { parse_luatable_datacfg(buf, scratch) }
    };

    let row_doc = format!(" Row record for `{}.out`.", spec.stem);
    let table_doc = format!(" Typed `DataTable` for `{}.out`.", spec.stem);
    let parse_doc = format!(" Parses `{}.out` into a [`{table_ident}`].", spec.stem);

    quote! {
        #(#helpers)*

        #[doc = #row_doc]
        #row_derives
        pub struct #row_ident {
            #(#field_defs,)*
        }

        #row_impl

        #[doc = #table_doc]
        pub type #table_ident = DataTable<#key_ty, #row_ident>;

        #[doc = #parse_doc]
        pub fn #parse_fn_ident(
            buf: &mut [u8],
            scratch: &mut DataCfgScratch,
        ) -> Result<#table_ident, LuacError> {
            #parse_call
        }
    }
}

fn emit_domain_verifier(domain: &str, specs: &[LuaTableSpec]) -> TokenStream {
    let verify_fn_ident = format_ident!("verify_{domain}_datacfg_by_stem");

    if specs.len() > 10 {
        let chunk_fns = specs.chunks(10).enumerate().map(|(idx, chunk)| {
            let chunk_ident = format_ident!("verify_{domain}_chunk_{idx}");
            let arms = chunk.iter().map(|spec| {
                let stem = &spec.stem;
                let parse_ident = format_ident!("parse_{stem}");
                quote! { #stem => Some(#parse_ident(buf, scratch).map(|t| t.len())) }
            });
            quote! {
                #[inline(never)]
                fn #chunk_ident(
                    stem: &str,
                    buf: &mut [u8],
                    scratch: &mut DataCfgScratch,
                ) -> Option<Result<u32, LuacError>> {
                    match stem {
                        #(#arms,)*
                        _ => None,
                    }
                }
            }
        });
        let chunk_calls = (0..specs.chunks(10).len()).map(|idx| {
            let chunk_ident = format_ident!("verify_{domain}_chunk_{idx}");
            quote! {
                if let Some(res) = #chunk_ident(stem, buf, scratch) {
                    return Some(res);
                }
            }
        });
        quote! {
            #(#chunk_fns)*

            pub(crate) fn #verify_fn_ident(
                stem: &str,
                buf: &mut [u8],
                scratch: &mut DataCfgScratch,
            ) -> Option<Result<u32, LuacError>> {
                #(#chunk_calls)*
                None
            }
        }
    } else {
        let arms = specs.iter().map(|spec| {
            let stem = &spec.stem;
            let parse_ident = format_ident!("parse_{stem}");
            quote! { #stem => Some(#parse_ident(buf, scratch).map(|t| t.len())) }
        });
        quote! {
            pub(crate) fn #verify_fn_ident(
                stem: &str,
                buf: &mut [u8],
                scratch: &mut DataCfgScratch,
            ) -> Option<Result<u32, LuacError>> {
                match stem {
                    #(#arms,)*
                    _ => None,
                }
            }
        }
    }
}

/// Generates or verifies a single `<domain>.rs` module in `out_dir`.
pub fn emit_domain_module(
    out_dir: &Path,
    domain: &str,
    specs: &[LuaTableSpec],
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let any_3d = specs.iter().any(|s| s.is_3d);
    let any_helper = specs.iter().any(|s| {
        s.cols
            .iter()
            .any(|(col, ty)| ty.nested_record(&s.pascal, col).is_some())
    });
    let any_fx_map = specs
        .iter()
        .any(|s| s.cols.values().any(ColType::uses_fx_hash_map));
    let any_int_or_list = specs
        .iter()
        .any(|s| s.cols.values().any(ColType::uses_int_or_list));

    let mut datacfg_names = vec![
        "DataCfgRow",
        "DataCfgScratch",
        "DataTable",
        "EvalContext",
        "EvalVal",
    ];
    if any_helper {
        datacfg_names.push("FromEvalVal");
    }
    datacfg_names.push("parse_luatable_datacfg");
    if any_3d {
        datacfg_names.push("parse_luatable_datacfg_3d");
    }
    let datacfg_idents: Vec<Ident> = datacfg_names.into_iter().map(make_ident).collect();
    let datacfg_prefix = quote! { crate::luac::datacfg };
    let datacfg_use = emit_use_group(&datacfg_prefix, &datacfg_idents);

    let int_or_list_use = if any_int_or_list {
        quote! { use crate::luac::types::IntOrIntList; }
    } else {
        TokenStream::new()
    };
    let fx_map_use = if any_fx_map {
        quote! { use rustc_hash::FxHashMap; }
    } else {
        TokenStream::new()
    };

    let mod_doc =
        format!(" Auto-generated `LuaTable/DataCfg` types and parsers for the `{domain}` domain.");
    let table_items = specs.iter().map(emit_table_spec);
    let verifier = emit_domain_verifier(domain, specs);

    let tokens = quote! {
        #![doc = #mod_doc]

        #datacfg_use
        use crate::luac::error::LuacError;
        #int_or_list_use
        #fx_map_use

        #(#table_items)*

        #verifier
    };
    write_or_check_tokens(&out_dir.join(format!("{domain}.rs")), tokens, mode)
}

fn emit_preload_item(p: &PreloadSpec) -> TokenStream {
    let alias_ident = format_ident!("{}Table", p.pascal);
    let parse_fn_ident = format_ident!("parse_{}", p.stem);
    let stem = &p.stem;

    let (doc_kind, rhs_tokens, parse_call) = match p.kind {
        PreloadKind::List(elem_ty) => {
            let et = elem_ty.rust_type_tokens();
            (
                "Typed preload list",
                quote! { Box<[#et]> },
                quote! { parse_preload_list(buf, scratch, #stem) },
            )
        }
        PreloadKind::Map1D(k_ty, v_ty) => {
            let kt = k_ty.rust_type_tokens();
            let vt = v_ty.rust_type_tokens();
            (
                "Typed preload table",
                quote! { PreloadTable<#kt, #vt> },
                quote! { parse_preload_map(buf, scratch, #stem) },
            )
        }
        PreloadKind::Map2D(k1_ty, k2_ty, v_ty) => {
            let k1 = k1_ty.rust_type_tokens();
            let k2 = k2_ty.rust_type_tokens();
            let vt = v_ty.rust_type_tokens();
            (
                "Typed 2D preload table",
                quote! { PreloadTable<(#k1, #k2), #vt> },
                quote! { parse_preload_map_2d(buf, scratch, #stem) },
            )
        }
    };

    let type_doc = format!(" {doc_kind} for `{stem}.out`.");
    let parse_doc = format!(" Parses `{stem}.out` into a [`{alias_ident}`].");

    quote! {
        #[doc = #type_doc]
        pub type #alias_ident = #rhs_tokens;

        #[doc = #parse_doc]
        pub fn #parse_fn_ident(
            buf: &mut [u8],
            scratch: &mut DataCfgScratch,
        ) -> Result<#alias_ident, LuacError> {
            #parse_call
        }
    }
}

fn emit_preload_verifier(preload_specs: &[PreloadSpec]) -> TokenStream {
    let chunk_fns = preload_specs.chunks(8).enumerate().map(|(idx, chunk)| {
        let chunk_ident = format_ident!("verify_preload_chunk_{idx}");
        let arms = chunk.iter().map(|p| {
            let stem = &p.stem;
            let parse_ident = format_ident!("parse_{stem}");
            if matches!(p.kind, PreloadKind::List(_)) {
                quote! { #stem => Some(#parse_ident(buf, scratch).map(|t| bounded_len_u32(t.len()))) }
            } else {
                quote! { #stem => Some(#parse_ident(buf, scratch).map(|t| t.len())) }
            }
        });
        quote! {
            #[inline(never)]
            fn #chunk_ident(
                stem: &str,
                buf: &mut [u8],
                scratch: &mut DataCfgScratch,
            ) -> Option<Result<u32, LuacError>> {
                match stem {
                    #(#arms,)*
                    _ => None,
                }
            }
        }
    });
    let chunk_calls = (0..preload_specs.chunks(8).len()).map(|idx| {
        let chunk_ident = format_ident!("verify_preload_chunk_{idx}");
        quote! {
            if let Some(res) = #chunk_ident(stem, buf, scratch) {
                return Some(res);
            }
        }
    });
    quote! {
        #(#chunk_fns)*

        pub(crate) fn verify_preload_datacfg_by_stem(
            stem: &str,
            buf: &mut [u8],
            scratch: &mut DataCfgScratch,
        ) -> Option<Result<u32, LuacError>> {
            #(#chunk_calls)*
            None
        }
    }
}

/// Generates or verifies `preload.rs` in `out_dir`.
pub fn emit_preload_module(
    out_dir: &Path,
    preload_specs: &[PreloadSpec],
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let int_or_list_use = if preload_specs.iter().any(|s| s.uses_int_or_list) {
        quote! { use crate::luac::types::IntOrIntList; }
    } else {
        TokenStream::new()
    };
    let items = preload_specs.iter().map(emit_preload_item);
    let verifier = emit_preload_verifier(preload_specs);

    let tokens = quote! {
        //! Auto-generated `LuaPreLoadTable/DataCfg` preload table types and parsers.

        use crate::bounded_len_u32;
        use crate::luac::datacfg::{
            DataCfgScratch, PreloadTable, parse_preload_list, parse_preload_map, parse_preload_map_2d,
        };
        use crate::luac::error::LuacError;
        #int_or_list_use

        #(#items)*

        #verifier
    };
    write_or_check_tokens(&out_dir.join("preload.rs"), tokens, mode)
}

/// Generates or verifies `mod.rs` in `out_dir`.
pub fn emit_mod_rs(
    out_dir: &Path,
    all_modules: &BTreeSet<&'static str>,
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let mod_idents: Vec<Ident> = all_modules.iter().map(|&m| make_ident(m)).collect();
    let count_lit = Literal::usize_unsuffixed(all_modules.len());
    let verifiers = all_modules.iter().map(|&m| {
        let dom_ident = make_ident(m);
        let fn_ident = format_ident!("verify_{m}_datacfg_by_stem");
        quote! { #dom_ident::#fn_ident }
    });

    let tokens = quote! {
        //! Strongly-typed `DataCfg` and `LuaPreLoadTable` parsers grouped into 23 domain modules.

        #(pub mod #mod_idents;)*

        #(pub use #mod_idents::*;)*

        use crate::luac::datacfg::DataCfgScratch;
        use crate::luac::error::{DataCfgError, LuacError};

        type DomainVerifier = fn(&str, &mut [u8], &mut DataCfgScratch) -> Option<Result<u32, LuacError>>;

        const DOMAIN_VERIFIERS: [DomainVerifier; #count_lit] = [
            #(#verifiers,)*
        ];

        /// Evaluates a `DataCfg` `.out` buffer by file stem (`config_*`) and returns its row/entry count.
        pub fn verify_all_datacfg_by_stem(
            stem: &str,
            buf: &mut [u8],
            scratch: &mut DataCfgScratch,
        ) -> Result<u32, LuacError> {
            for verifier in DOMAIN_VERIFIERS {
                if let Some(res) = verifier(stem, buf, scratch) {
                    return res;
                }
            }
            Err(DataCfgError::UnknownField {
                table: "<datacfg_corpus>",
                field: stem.into(),
            }
            .into())
        }
    };
    write_or_check_tokens(&out_dir.join("mod.rs"), tokens, mode)
}

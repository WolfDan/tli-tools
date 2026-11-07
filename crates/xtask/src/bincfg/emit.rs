//! Deterministic Rust AST code emitter for `crates/parsers/src/bin/tables/` using
//! `proc_macro2`, `quote`, `syn`, and `prettyplease`.

use crate::bincfg::domains::{MapKeyWire, SubOp, WireType};
use crate::bincfg::pe::ExtractedKeySchema;
use crate::datacfg::domains::{to_pascal_case, to_snake_case};
use crate::datacfg::emit::{CodegenMode, emit_use_group, make_ident, write_or_check_tokens};
use crate::error::XtaskError;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Final resolved specification for a single `.bin` table ready for Rust code emission.
#[derive(Debug, Clone)]
pub struct BinTableSpec {
    /// Canonical `config_*` snake_case name (e.g. `"config_item_base"`).
    pub config_stem: String,
    /// `PascalCase` name without `Config` prefix (e.g. `"ItemBase"`).
    pub pascal: String,
    /// `true` for uncompressed `Maps/<Map>/config_*.bin` tables; `false` for `*_ChunkA0.bin`.
    pub is_map_table: bool,
    /// When `Some(domain)`, the row struct and `Table` alias are already emitted in `domain`.
    pub shared_with_chunk_domain: Option<&'static str>,
    /// Primary key hierarchy schema.
    pub key_schema: ExtractedKeySchema,
    /// Columns in exact wire order: `(rust_field_name, wire_type, is_translate_str)`.
    pub cols: Vec<(String, WireType, bool)>,
}

impl BinTableSpec {
    #[must_use]
    pub fn has_float(&self) -> bool {
        self.cols.iter().any(|(_, k, _)| k.has_float())
    }

    #[must_use]
    pub fn uses_fx_hash_map(&self) -> bool {
        self.cols
            .iter()
            .any(|(_, k, _)| k.uses_direct_fx_hash_map())
    }

    #[must_use]
    pub fn uses_vec_dim(&self) -> bool {
        self.cols.iter().any(|(_, k, _)| k.uses_direct_vec_dim())
    }

    #[must_use]
    pub fn uses_affix(&self) -> bool {
        self.cols.iter().any(|(_, k, _)| k.uses_direct_affix())
    }

    #[must_use]
    pub fn uses_action_data(&self) -> bool {
        self.cols
            .iter()
            .any(|(_, k, _)| k.uses_direct_action_data())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StructUsageKind {
    Direct,
    InArray1d,
    InArray2d,
}

/// Synthesized compound sub-struct specification discovered from `WireType::Struct` nodes.
#[derive(Debug, Clone)]
pub struct SubStructSpec {
    /// Synthesized Rust struct identifier (e.g. `FConfigI32I32` or `FConfigMapMonsterPointMonsterStart`).
    pub name: String,
    /// Wire-order field types of this compound sub-struct.
    pub fields: Vec<WireType>,
    /// Whether `Self::read(cur)` is called directly.
    pub needs_read_single: bool,
    /// Whether `Self::read_array(cur)` is called for `<count: u32><Self[count]>`.
    pub needs_read_array: bool,
    /// Whether `Self::read_array_2d(cur)` is called for `<count: u32><Array<Self>[count]>`.
    pub needs_read_array_2d: bool,
    /// Distinct `(table_pascal, col_name)` pairs that reference this sub-struct.
    pub table_usages: BTreeSet<(String, String)>,
    /// Whether this sub-struct is referenced inside `ActionData`.
    pub used_in_action_data: bool,
}

/// Registry of all `WireType::Struct` compound types discovered across `.bin` schemas.
#[derive(Debug, Clone, Default)]
pub struct SubStructRegistry {
    specs: Vec<SubStructSpec>,
    by_fields: BTreeMap<Vec<WireType>, usize>,
}

impl SubStructRegistry {
    /// Collects and names all `WireType::Struct` compounds across `domains` and `action_table`.
    #[must_use]
    pub fn from_schemas(
        domains: &BTreeMap<&'static str, Vec<BinTableSpec>>,
        action_table: &[Vec<WireType>],
    ) -> Self {
        let mut reg = Self::default();
        for specs in domains.values() {
            for spec in specs {
                if spec.shared_with_chunk_domain.is_some() {
                    continue;
                }
                for (col_name, wt, _) in &spec.cols {
                    reg.visit_wire_type(
                        wt,
                        StructUsageKind::Direct,
                        Some((&spec.pascal, col_name)),
                        false,
                    );
                }
            }
        }
        for cols in action_table {
            for wt in cols {
                reg.visit_wire_type(wt, StructUsageKind::Direct, None, true);
            }
        }
        reg.assign_names();
        reg
    }

    fn visit_wire_type(
        &mut self,
        wt: &WireType,
        usage: StructUsageKind,
        table_col: Option<(&str, &str)>,
        in_action: bool,
    ) {
        match wt {
            WireType::Array(elem) => {
                if let WireType::Array(inner) = &**elem {
                    self.visit_wire_type(inner, StructUsageKind::InArray2d, table_col, in_action);
                } else {
                    self.visit_wire_type(elem, StructUsageKind::InArray1d, table_col, in_action);
                }
            }
            WireType::Map(_, val) => {
                self.visit_wire_type(val, StructUsageKind::Direct, table_col, in_action);
            }
            WireType::Struct(fields) => {
                for f in fields {
                    self.visit_wire_type(f, StructUsageKind::Direct, None, in_action);
                }
                let idx = if let Some(&existing) = self.by_fields.get(fields.as_slice()) {
                    existing
                } else {
                    let next = self.specs.len();
                    self.specs.push(SubStructSpec {
                        name: String::new(),
                        fields: fields.clone(),
                        needs_read_single: false,
                        needs_read_array: false,
                        needs_read_array_2d: false,
                        table_usages: BTreeSet::new(),
                        used_in_action_data: false,
                    });
                    self.by_fields.insert(fields.clone(), next);
                    next
                };
                let spec = &mut self.specs[idx];
                match usage {
                    StructUsageKind::Direct => spec.needs_read_single = true,
                    StructUsageKind::InArray1d => spec.needs_read_array = true,
                    StructUsageKind::InArray2d => spec.needs_read_array_2d = true,
                }
                if let Some((tp, col)) = table_col {
                    spec.table_usages.insert((tp.to_owned(), col.to_owned()));
                }
                if in_action {
                    spec.used_in_action_data = true;
                }
            }
            WireType::Bool
            | WireType::I32
            | WireType::F64
            | WireType::Str
            | WireType::Bytes
            | WireType::VecDim
            | WireType::AffixEntry
            | WireType::ActionData => {}
        }
    }

    fn assign_names(&mut self) {
        let mut used_names = BTreeSet::new();
        for spec in &mut self.specs {
            let structural = format!(
                "FConfig{}",
                WireType::Struct(spec.fields.clone()).type_token()
            );
            let candidate = if spec.fields.len() > 3
                && !spec.used_in_action_data
                && spec.table_usages.len() == 1
            {
                let (tp, col) = &spec
                    .table_usages
                    .iter()
                    .next()
                    .into_iter()
                    .collect::<Vec<_>>()[0];
                let clean_col = match col.strip_prefix("r#") {
                    Some(c) => c,
                    None => col.as_str(),
                };
                if clean_col.starts_with("col_") {
                    structural.clone()
                } else {
                    format!("FConfig{tp}{}", to_pascal_case(clean_col))
                }
            } else {
                structural.clone()
            };
            let chosen = if used_names.contains(&candidate) {
                structural
            } else {
                candidate
            };
            used_names.insert(chosen.clone());
            spec.name = chosen;
        }
    }

    /// Returns the synthesized struct name for `fields`.
    #[must_use]
    pub fn struct_name(&self, fields: &[WireType]) -> &str {
        let idx = self.by_fields[fields];
        &self.specs[idx].name
    }

    /// Returns the Rust type `TokenStream` for `wt`.
    #[must_use]
    pub fn rust_type_tokens(&self, wt: &WireType) -> TokenStream {
        match wt {
            WireType::Bool => quote! { bool },
            WireType::I32 => quote! { i32 },
            WireType::F64 => quote! { f64 },
            WireType::Str => quote! { Box<str> },
            WireType::Bytes => quote! { Box<[u8]> },
            WireType::VecDim => quote! { VecDim },
            WireType::AffixEntry => quote! { FConfigAffix },
            WireType::ActionData => quote! { ActionData },
            WireType::Array(elem) => {
                let inner = self.rust_type_tokens(elem);
                quote! { Box<[#inner]> }
            }
            WireType::Map(k, v) => {
                let kt = k.rust_type_tokens();
                let vt = self.rust_type_tokens(v);
                quote! { FxHashMap<#kt, #vt> }
            }
            WireType::Struct(fields) => {
                let ident = make_ident(self.struct_name(fields));
                quote! { #ident }
            }
        }
    }

    /// Returns the `WireCursor` read expression `TokenStream` for `wt` using cursor `cur_var`.
    #[must_use]
    pub fn read_expr_with_cursor(&self, wt: &WireType, cur_var: &Ident) -> TokenStream {
        match wt {
            WireType::Bool => quote! { #cur_var.read_bool()? },
            WireType::I32 => quote! { #cur_var.read_i32()? },
            WireType::F64 => quote! { #cur_var.read_f64()? },
            WireType::Str => quote! { #cur_var.read_str()? },
            WireType::Bytes => quote! { #cur_var.read_bytes()? },
            WireType::VecDim => quote! { #cur_var.read_vec_dim()? },
            WireType::AffixEntry => quote! { FConfigAffix::read(#cur_var)? },
            WireType::ActionData => quote! { ActionData::read(#cur_var)? },
            WireType::Struct(fields) => {
                let ident = make_ident(self.struct_name(fields));
                quote! { #ident::read(#cur_var)? }
            }
            WireType::Array(elem) => self.read_array_expr(elem, cur_var),
            WireType::Map(k, v) => self.read_map_expr(*k, v, cur_var),
        }
    }

    fn read_array_expr(&self, elem: &WireType, cur_var: &Ident) -> TokenStream {
        let c_ident = Ident::new("c", Span::call_site());
        match elem {
            WireType::I32 => quote! { #cur_var.read_array_i32()? },
            WireType::F64 => quote! { #cur_var.read_array_f64()? },
            WireType::Str => quote! { #cur_var.read_array_str()? },
            WireType::VecDim => quote! { #cur_var.read_array_vec_dim()? },
            WireType::AffixEntry => quote! { FConfigAffix::read_array(#cur_var)? },
            WireType::Struct(fields) => {
                let ident = make_ident(self.struct_name(fields));
                quote! { #ident::read_array(#cur_var)? }
            }
            WireType::Array(inner) => match &**inner {
                WireType::I32 => quote! { #cur_var.read_array_2d_i32()? },
                WireType::F64 => quote! { #cur_var.read_array_2d_f64()? },
                WireType::Struct(fields) => {
                    let ident = make_ident(self.struct_name(fields));
                    quote! { #ident::read_array_2d(#cur_var)? }
                }
                other @ (WireType::Bool
                | WireType::Str
                | WireType::Bytes
                | WireType::VecDim
                | WireType::AffixEntry
                | WireType::ActionData
                | WireType::Array(_)
                | WireType::Map(..)) => {
                    let inner_expr = self.read_array_expr(other, &c_ident);
                    quote! { #cur_var.read_array_with(|c| Ok(#inner_expr))? }
                }
            },
            other @ (WireType::Bool
            | WireType::Bytes
            | WireType::ActionData
            | WireType::Map(..)) => {
                let inner_expr = self.read_expr_with_cursor(other, &c_ident);
                quote! { #cur_var.read_array_with(|c| Ok(#inner_expr))? }
            }
        }
    }

    fn read_map_expr(&self, k: MapKeyWire, v: &WireType, cur_var: &Ident) -> TokenStream {
        let c_ident = Ident::new("c", Span::call_site());
        match (k, v) {
            (MapKeyWire::I32, WireType::I32) => quote! { #cur_var.read_map_i32_i32()? },
            (MapKeyWire::I32, WireType::Str) => quote! { #cur_var.read_map_i32_str()? },
            (MapKeyWire::I32, WireType::Array(inner)) if **inner == WireType::I32 => {
                quote! { #cur_var.read_map_i32_array_i32()? }
            }
            (MapKeyWire::I32, WireType::Array(inner)) if **inner == WireType::F64 => {
                quote! { #cur_var.read_map_i32_array_f64()? }
            }
            (MapKeyWire::Str, WireType::I32) => quote! { #cur_var.read_map_str_i32()? },
            (MapKeyWire::Str, WireType::F64) => quote! { #cur_var.read_map_str_f64()? },
            (MapKeyWire::Str, WireType::Str) => quote! { #cur_var.read_map_str_str()? },
            (MapKeyWire::Str, WireType::Array(inner)) if **inner == WireType::I32 => {
                quote! { #cur_var.read_map_str_array_i32()? }
            }
            (MapKeyWire::I32, other) => {
                let inner_expr = self.read_expr_with_cursor(other, &c_ident);
                quote! { #cur_var.read_map_i32_with(|c| Ok(#inner_expr))? }
            }
            (MapKeyWire::Str, other) => {
                let inner_expr = self.read_expr_with_cursor(other, &c_ident);
                quote! { #cur_var.read_map_str_with(|c| Ok(#inner_expr))? }
            }
        }
    }

    /// Returns the `WireCursor` read expression `TokenStream` for `wt` using `cur`.
    #[must_use]
    pub fn read_expr_tokens(&self, wt: &WireType) -> TokenStream {
        let cur = Ident::new("cur", Span::call_site());
        self.read_expr_with_cursor(wt, &cur)
    }

    /// Collects the sorted list of sub-struct `Ident`s directly referenced by `types`.
    #[must_use]
    pub fn collect_direct_idents<'a>(
        &self,
        types: impl Iterator<Item = &'a WireType>,
    ) -> Vec<Ident> {
        let mut names = BTreeSet::new();
        for wt in types {
            self.collect_direct_wt(wt, &mut names);
        }
        names.into_iter().map(|n| make_ident(&n)).collect()
    }

    fn collect_direct_wt(&self, wt: &WireType, out: &mut BTreeSet<String>) {
        match wt {
            WireType::Struct(fields) => {
                out.insert(self.struct_name(fields).to_owned());
            }
            WireType::Array(elem) | WireType::Map(_, elem) => {
                self.collect_direct_wt(elem, out);
            }
            WireType::Bool
            | WireType::I32
            | WireType::F64
            | WireType::Str
            | WireType::Bytes
            | WireType::VecDim
            | WireType::AffixEntry
            | WireType::ActionData => {}
        }
    }
}

fn emit_substruct_read_method(
    spec: &SubStructSpec,
    ordered: &[(usize, &WireType)],
    reg: &SubStructRegistry,
) -> TokenStream {
    let body = if spec.fields == [WireType::I32, WireType::I32] {
        quote! {
            let raw = cur.take::<RawPairI32>(WireContext::SubStructColumn)?;
            Ok(Self {
                f0: raw.a.get(),
                f1: raw.b.get(),
            })
        }
    } else if spec.fields == [WireType::I32, WireType::F64] {
        quote! {
            let raw = cur.take::<RawI32F64>(WireContext::SubStructColumn)?;
            Ok(Self {
                f1: raw.b.get(),
                f0: raw.a.get(),
            })
        }
    } else if spec.fields == [WireType::F64, WireType::F64, WireType::F64, WireType::F64] {
        quote! {
            let raw = cur.take::<RawLinearColorF64>(WireContext::SubStructColumn)?;
            Ok(Self {
                f0: raw.r.get(),
                f1: raw.g.get(),
                f2: raw.b.get(),
                f3: raw.a.get(),
            })
        }
    } else {
        let reads = spec.fields.iter().enumerate().map(|(i, wt)| {
            let f_ident = format_ident!("f{i}");
            let expr = reg.read_expr_tokens(wt);
            quote! { let #f_ident = #expr; }
        });
        let field_idents = ordered.iter().map(|&(i, _)| format_ident!("f{i}"));
        quote! {
            #(#reads)*
            Ok(Self {
                #(#field_idents),*
            })
        }
    };
    quote! {
        /// Decodes a single instance from `cur`.
        #[inline(never)]
        pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
            #body
        }
    }
}

fn emit_substruct_array_methods(spec: &SubStructSpec) -> TokenStream {
    let read_array = if spec.needs_read_array || spec.needs_read_array_2d {
        let loop_body = if spec.fields == [WireType::I32, WireType::I32] {
            quote! {
                let slice = cur.take_slice_elems::<RawPairI32>(count, WireContext::ArrayColumn)?;
                let mut out = Vec::with_capacity(u32_to_usize(count));
                for item in slice {
                    out.push(Self {
                        f0: item.a.get(),
                        f1: item.b.get(),
                    });
                }
                Ok(out.into_boxed_slice())
            }
        } else if spec.fields == [WireType::I32, WireType::F64] {
            quote! {
                let slice = cur.take_slice_elems::<RawI32F64>(count, WireContext::ArrayColumn)?;
                let mut out = Vec::with_capacity(u32_to_usize(count));
                for item in slice {
                    out.push(Self {
                        f1: item.b.get(),
                        f0: item.a.get(),
                    });
                }
                Ok(out.into_boxed_slice())
            }
        } else {
            quote! {
                let mut out = Vec::with_capacity(u32_to_usize(count));
                for _ in 0..count {
                    out.push(Self::read(cur)?);
                }
                Ok(out.into_boxed_slice())
            }
        };
        quote! {
            /// Decodes a length-prefixed array of `Self` from `cur`.
            #[inline(never)]
            pub fn read_array(cur: &mut WireCursor<'_>) -> Result<Box<[Self]>, BinError> {
                let count = cur.read_collection_count(WireContext::ArrayColumn)?;
                #loop_body
            }
        }
    } else {
        TokenStream::new()
    };

    let read_array_2d = if spec.needs_read_array_2d {
        quote! {
            /// Decodes a 2D length-prefixed array of `Self` from `cur`.
            #[inline(never)]
            pub fn read_array_2d(cur: &mut WireCursor<'_>) -> Result<Box<[Box<[Self]>]>, BinError> {
                let count = cur.read_collection_count(WireContext::ArrayColumn)?;
                let mut out = Vec::with_capacity(u32_to_usize(count));
                for _ in 0..count {
                    out.push(Self::read_array(cur)?);
                }
                Ok(out.into_boxed_slice())
            }
        }
    } else {
        TokenStream::new()
    };

    quote! {
        #read_array
        #read_array_2d
    }
}

fn emit_single_substruct(spec: &SubStructSpec, reg: &SubStructRegistry) -> TokenStream {
    let is_copy = spec.fields.iter().all(|f| {
        matches!(
            f,
            WireType::Bool | WireType::I32 | WireType::F64 | WireType::VecDim
        )
    });
    let has_flt = spec.fields.iter().any(WireType::has_float);
    let derives = match (is_copy, has_flt) {
        (true, true) => quote! { #[derive(Debug, Clone, Copy, PartialEq, Default)] },
        (true, false) => quote! { #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] },
        (false, true) => quote! { #[derive(Debug, Clone, PartialEq, Default)] },
        (false, false) => quote! { #[derive(Debug, Clone, PartialEq, Eq, Default)] },
    };
    let doc = format!(" Synthesized `{}` compound sub-struct.", spec.name);
    let struct_ident = make_ident(&spec.name);
    let mut ordered: Vec<(usize, &WireType)> = spec.fields.iter().enumerate().collect();
    ordered.sort_by_key(|&(i, wt)| (wt.align_tier(), i));
    let field_defs = ordered.iter().map(|&(i, wt)| {
        let f_ident = format_ident!("f{i}");
        let ty = reg.rust_type_tokens(wt);
        quote! { pub #f_ident: #ty }
    });
    let read_fn = emit_substruct_read_method(spec, &ordered, reg);
    let array_fns = emit_substruct_array_methods(spec);
    quote! {
        #[doc = #doc]
        #derives
        pub struct #struct_ident {
            #(#field_defs,)*
        }

        impl #struct_ident {
            #read_fn
            #array_fns
        }
    }
}

/// Synthesizes and writes or verifies `substructs.rs` from `reg`.
pub fn emit_substructs_rs(
    out_dir: &Path,
    reg: &SubStructRegistry,
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let affix_use = if reg
        .specs
        .iter()
        .any(|s| s.fields.iter().any(WireType::uses_direct_affix))
    {
        quote! { use crate::bin::affix::FConfigAffix; }
    } else {
        TokenStream::new()
    };
    let vec_dim_use = if reg
        .specs
        .iter()
        .any(|s| s.fields.iter().any(WireType::uses_direct_vec_dim))
    {
        quote! { use crate::bin::types::VecDim; }
    } else {
        TokenStream::new()
    };
    let usize_use = if reg
        .specs
        .iter()
        .any(|s| s.needs_read_array || s.needs_read_array_2d)
    {
        quote! { use crate::u32_to_usize; }
    } else {
        TokenStream::new()
    };
    let items = reg.specs.iter().map(|s| emit_single_substruct(s, reg));
    let tokens = quote! {
        //! Auto-generated compound `FConfig*` sub-structs synthesized from `.bin` schemas.

        #affix_use
        use crate::bin::error::{BinError, WireContext};
        #vec_dim_use
        use crate::bin::wire::{RawI32F64, RawLinearColorF64, RawPairI32, WireCursor};
        #usize_use

        #(#items)*
    };
    write_or_check_tokens(&out_dir.join("substructs.rs"), tokens, mode)
}

/// Generates or verifies `affix_ops.rs` from the extracted `op3_table`.
pub fn emit_affix_ops_rs(
    out_dir: &Path,
    op3_table: &[Vec<SubOp>],
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let max_idx = op3_table.len().saturating_sub(1);
    let max_lit = Literal::usize_unsuffixed(max_idx);
    let len_lit = Literal::usize_unsuffixed(op3_table.len());
    let max_doc = format!(
        " Maximum valid `TreeNode` tag-3 opcode index extracted from client (`0..={max_idx}`)."
    );

    let variants = (0..op3_table.len()).map(|idx| {
        let ident = format_ident!("Op{idx:02}");
        let disc = Literal::usize_unsuffixed(idx);
        quote! { #ident = #disc }
    });
    let code_entries = (0..op3_table.len()).map(|idx| {
        let ident = format_ident!("Op{idx:02}");
        quote! { AffixOp3Code::#ident }
    });
    let schema_entries = op3_table.iter().map(|ops| {
        let kinds = ops.iter().map(|s| match s {
            SubOp::FormulaOrF64 => quote! { AffixSubOpKind::FormulaOrF64 },
            SubOp::FormulaOrI32 => quote! { AffixSubOpKind::FormulaOrI32 },
            SubOp::FormulaOrBool => quote! { AffixSubOpKind::FormulaOrBool },
            SubOp::FormulaOrU8 => quote! { AffixSubOpKind::FormulaOrU8 },
            SubOp::ArrayFormulaOrI32 => quote! { AffixSubOpKind::ArrayFormulaOrI32 },
        });
        quote! { &[#(#kinds),*] }
    });

    let tokens = quote! {
        //! Auto-generated `AffixOp3Code` opcode table extracted from client.

        use crate::bin::affix::AffixSubOpKind;
        use crate::bin::error::BinError;

        #[doc = #max_doc]
        pub const AFFIX_OP3_INDEX_MAX: u8 = #max_lit;

        /// Strongly-typed tag-3 expression opcode (`0..=AFFIX_OP3_INDEX_MAX`).
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum AffixOp3Code {
            #(#variants,)*
        }

        const OP3_CODES: [AffixOp3Code; #len_lit] = [
            #(#code_entries,)*
        ];

        const OP3_SCHEMAS: [&[AffixSubOpKind]; #len_lit] = [
            #(#schema_entries,)*
        ];

        impl AffixOp3Code {
            /// Validates a raw tag-3 opcode byte (`0..=AFFIX_OP3_INDEX_MAX`).
            #[inline]
            pub fn from_raw(raw: u8) -> Result<Self, BinError> {
                OP3_CODES
                    .get(usize::from(raw))
                    .copied()
                    .ok_or(BinError::InvalidAffixOp3 { raw_op: raw })
            }

            /// Returns the operand schema for this opcode.
            #[inline]
            #[must_use]
            pub const fn schema(self) -> &'static [AffixSubOpKind] {
                OP3_SCHEMAS[self as usize]
            }
        }
    };
    write_or_check_tokens(&out_dir.join("affix_ops.rs"), tokens, mode)
}

fn emit_action_data_structs(
    action_table: &[Vec<WireType>],
    reg: &SubStructRegistry,
) -> TokenStream {
    let items = action_table.iter().enumerate().filter_map(|(idx, cols)| {
        if cols.is_empty() {
            return None;
        }
        let tag = 1000 + idx;
        let struct_ident = format_ident!("ActionData{tag}");
        let doc = format!(" Payload for `ActionData` tag `{tag}`.");
        let derives = if cols.iter().any(WireType::has_float) {
            quote! { #[derive(Debug, Clone, PartialEq, Default)] }
        } else {
            quote! { #[derive(Debug, Clone, PartialEq, Eq, Default)] }
        };
        let mut ordered: Vec<(usize, &WireType)> = cols.iter().enumerate().collect();
        ordered.sort_by_key(|&(i, k)| (k.align_tier(), i));
        let field_defs = ordered.iter().map(|&(i, k)| {
            let f_ident = format_ident!("f{i}");
            let ty = reg.rust_type_tokens(k);
            quote! { pub #f_ident: #ty }
        });
        let reads = cols.iter().enumerate().map(|(i, k)| {
            let f_ident = format_ident!("f{i}");
            let expr = reg.read_expr_tokens(k);
            quote! { let #f_ident = #expr; }
        });
        let field_idents = ordered.iter().map(|&(i, _)| format_ident!("f{i}"));
        Some(quote! {
            #[doc = #doc]
            #derives
            pub struct #struct_ident {
                #(#field_defs,)*
            }

            impl #struct_ident {
                #[inline(never)]
                pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
                    #(#reads)*
                    Ok(Self {
                        #(#field_idents),*
                    })
                }
            }
        })
    });
    quote! { #(#items)* }
}

fn emit_action_data_enum_and_dispatch(action_table: &[Vec<WireType>]) -> TokenStream {
    let variants = action_table.iter().enumerate().map(|(idx, cols)| {
        let tag = 1000 + idx;
        let var_ident = format_ident!("Tag{tag}");
        if cols.is_empty() {
            quote! { #var_ident }
        } else {
            let struct_ident = format_ident!("ActionData{tag}");
            quote! { #var_ident(#struct_ident) }
        }
    });

    let indices: Vec<usize> = (0..action_table.len()).collect();
    let chunk_fns = indices.chunks(16).enumerate().map(|(c_idx, chunk)| {
        let fn_ident = format_ident!("read_action_chunk_{c_idx}");
        let arms = chunk.iter().map(|&idx| {
            let tag = 1000 + idx;
            let tag_lit = Literal::usize_unsuffixed(tag);
            let var_ident = format_ident!("Tag{tag}");
            if action_table[idx].is_empty() {
                quote! { #tag_lit => Ok(ActionData::#var_ident) }
            } else {
                let struct_ident = format_ident!("ActionData{tag}");
                quote! { #tag_lit => #struct_ident::read(cur).map(ActionData::#var_ident) }
            }
        });
        quote! {
            #[inline(never)]
            fn #fn_ident(tag: i32, cur: &mut WireCursor<'_>) -> Option<Result<ActionData, BinError>> {
                let res = match tag {
                    #(#arms,)*
                    _ => return None,
                };
                Some(res)
            }
        }
    });

    let chunk_calls = (0..indices.chunks(16).len()).map(|c_idx| {
        let fn_ident = format_ident!("read_action_chunk_{c_idx}");
        quote! {
            if let Some(res) = #fn_ident(tag, cur) {
                return res;
            }
        }
    });

    quote! {
        /// Polymorphic gameplay action payload (`tag == 0` or `1000..=1000 + ACTION_DATA_INDEX_MAX`).
        #[derive(Debug, Clone, PartialEq, Default)]
        pub enum ActionData {
            #[default]
            None,
            #(#variants,)*
        }

        #(#chunk_fns)*

        impl ActionData {
            /// Decodes a tagged `ActionData` union from `cur`.
            #[inline(never)]
            pub fn read(cur: &mut WireCursor<'_>) -> Result<Self, BinError> {
                let tag = cur.read_i32_ctx(WireContext::ActionDataPayload)?;
                if tag == 0 {
                    return Ok(Self::None);
                }
                #(#chunk_calls)*
                Err(BinError::InvalidActionDataTag { raw_tag: tag })
            }
        }
    }
}

/// Generates or verifies `action_data.rs` from the extracted `action_table`.
pub fn emit_action_data_rs(
    out_dir: &Path,
    action_table: &[Vec<WireType>],
    reg: &SubStructRegistry,
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let max_idx = action_table.len().saturating_sub(1);
    let max_lit = Literal::usize_unsuffixed(max_idx);
    let max_doc = format!(" Maximum `ActionData` index offset from `1000` (`0..={max_idx}`).");

    let all_types: Vec<&WireType> = action_table.iter().flatten().collect();
    let affix_use = if all_types.iter().copied().any(WireType::uses_direct_affix) {
        quote! { use crate::bin::affix::FConfigAffix; }
    } else {
        TokenStream::new()
    };
    let substruct_idents = reg.collect_direct_idents(all_types.iter().copied());
    let substruct_prefix = quote! { crate::bin::tables::substructs };
    let substruct_use = emit_use_group(&substruct_prefix, &substruct_idents);
    let vec_dim_use = if all_types.iter().copied().any(WireType::uses_direct_vec_dim) {
        quote! { use crate::bin::types::VecDim; }
    } else {
        TokenStream::new()
    };
    let map_use = if all_types
        .iter()
        .copied()
        .any(WireType::uses_direct_fx_hash_map)
    {
        quote! { use rustc_hash::FxHashMap; }
    } else {
        TokenStream::new()
    };

    let structs = emit_action_data_structs(action_table, reg);
    let enum_and_dispatch = emit_action_data_enum_and_dispatch(action_table);

    let tokens = quote! {
        //! Auto-generated `ActionData` tagged union (`0` and `1000..=1042`) extracted from client.

        #affix_use
        use crate::bin::error::{BinError, WireContext};
        #substruct_use
        #vec_dim_use
        use crate::bin::wire::WireCursor;
        #map_use

        #[doc = #max_doc]
        pub const ACTION_DATA_INDEX_MAX: i32 = #max_lit;

        #structs
        #enum_and_dispatch
    };
    write_or_check_tokens(&out_dir.join("action_data.rs"), tokens, mode)
}

fn emit_row_fields_impl(
    row_ident: &Ident,
    spec: &BinTableSpec,
    reg: &SubStructRegistry,
) -> TokenStream {
    if spec.cols.len() > 12 {
        let chunk_fns = spec.cols.chunks(12).enumerate().map(|(c_idx, chunk)| {
            let fn_ident = format_ident!("read_fields_{c_idx}");
            let assigns = chunk.iter().map(|(fname, kind, _)| {
                let f_ident = make_ident(fname);
                let expr = reg.read_expr_tokens(kind);
                quote! { self.#f_ident = #expr; }
            });
            quote! {
                #[inline(never)]
                fn #fn_ident(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
                    #(#assigns)*
                    Ok(())
                }
            }
        });
        let chunk_calls = (0..spec.cols.chunks(12).len()).map(|c_idx| {
            let fn_ident = format_ident!("read_fields_{c_idx}");
            quote! { self.#fn_ident(cur)?; }
        });
        quote! {
            impl #row_ident {
                #(#chunk_fns)*
            }

            impl BinRow for #row_ident {
                #[inline(never)]
                fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
                    #(#chunk_calls)*
                    Ok(())
                }
            }
        }
    } else {
        let body = if spec.cols.is_empty() {
            quote! { let _ = cur; }
        } else {
            let assigns = spec.cols.iter().map(|(fname, kind, _)| {
                let f_ident = make_ident(fname);
                let expr = reg.read_expr_tokens(kind);
                quote! { self.#f_ident = #expr; }
            });
            quote! { #(#assigns)* }
        };
        quote! {
            impl BinRow for #row_ident {
                #[inline(never)]
                fn read_fields(&mut self, cur: &mut WireCursor<'_>) -> Result<(), BinError> {
                    #body
                    Ok(())
                }
            }
        }
    }
}

fn emit_parse_fn(spec: &BinTableSpec, table_ident: &Ident, fn_stem: &str) -> TokenStream {
    let fn_ident = format_ident!("parse_{fn_stem}");
    if spec.is_map_table {
        let doc = format!(
            " Parses `Maps/<Map>/{}.bin` into a [`{table_ident}`].",
            spec.config_stem
        );
        quote! {
            #[doc = #doc]
            pub fn #fn_ident(buf: &[u8], limits: BinLimits) -> Result<#table_ident, BinError> {
                parse_map_bin_table(buf, limits)
            }
        }
    } else {
        let doc = format!(
            " Parses `{}_ChunkA0.bin` into a [`{table_ident}`].",
            spec.config_stem
        );
        quote! {
            #[doc = #doc]
            pub fn #fn_ident(buf: &[u8], scratch: &mut BinScratch) -> Result<#table_ident, BinError> {
                parse_chunk_bin_table(buf, scratch)
            }
        }
    }
}

fn emit_row_struct_and_impl(spec: &BinTableSpec, reg: &SubStructRegistry) -> TokenStream {
    let row_ident = format_ident!("Config{}Row", spec.pascal);
    let table_ident = format_ident!("Config{}Table", spec.pascal);
    let fn_stem = to_snake_case(&spec.config_stem);
    if spec.shared_with_chunk_domain.is_some() {
        let fn_ident = format_ident!("parse_map_{fn_stem}");
        let doc = format!(
            " Parses `Maps/<Map>/{}.bin` into a [`{table_ident}`].",
            spec.config_stem
        );
        return quote! {
            #[doc = #doc]
            pub fn #fn_ident(buf: &[u8], limits: BinLimits) -> Result<#table_ident, BinError> {
                parse_map_bin_table(buf, limits)
            }
        };
    }

    let derives = if spec.has_float() {
        quote! { #[derive(Debug, Clone, PartialEq, Default)] }
    } else {
        quote! { #[derive(Debug, Clone, PartialEq, Eq, Default)] }
    };
    let ext = if spec.is_map_table {
        ".bin"
    } else {
        "_ChunkA0.bin"
    };
    let row_doc = format!(" Row record for `{}{ext}`.", spec.config_stem);
    let table_doc = format!(" Typed `BinTable` for `{}{ext}`.", spec.config_stem);

    let mut ordered_fields: Vec<(usize, &str, &WireType)> = spec
        .cols
        .iter()
        .enumerate()
        .map(|(i, (n, k, _))| (i, n.as_str(), k))
        .collect();
    ordered_fields.sort_by_key(|&(i, _, k)| (k.align_tier(), i));
    let field_defs = ordered_fields.iter().map(|&(_, fname, kind)| {
        let f_ident = make_ident(fname);
        let ty = reg.rust_type_tokens(kind);
        quote! { pub #f_ident: #ty }
    });

    let fields_impl = emit_row_fields_impl(&row_ident, spec, reg);
    let key_ty = spec.key_schema.rust_key_type_tokens();
    let parse_fn = emit_parse_fn(spec, &table_ident, &fn_stem);

    quote! {
        #[doc = #row_doc]
        #derives
        pub struct #row_ident {
            #(#field_defs,)*
        }

        #fields_impl

        #[doc = #table_doc]
        pub type #table_ident = BinTable<#key_ty, #row_ident>;

        #parse_fn
    }
}

fn emit_verify_arm(spec: &BinTableSpec, arg_ident: &Ident) -> TokenStream {
    let stem = &spec.config_stem;
    let fn_stem = to_snake_case(stem);
    let fn_ident = if spec.shared_with_chunk_domain.is_some() {
        format_ident!("parse_map_{fn_stem}")
    } else {
        format_ident!("parse_{fn_stem}")
    };
    quote! {
        #stem => Some(#fn_ident(buf, #arg_ident).map(|t| t.len()))
    }
}

fn emit_bin_domain_verifier(
    domain: &str,
    specs: &[BinTableSpec],
    is_map_domain: bool,
) -> TokenStream {
    let (arg_sig, arg_ident) = if is_map_domain {
        (
            quote! { limits: BinLimits },
            Ident::new("limits", Span::call_site()),
        )
    } else {
        (
            quote! { scratch: &mut BinScratch },
            Ident::new("scratch", Span::call_site()),
        )
    };
    let verify_fn_ident = format_ident!("verify_{domain}_bin_by_stem");

    if specs.len() > 10 {
        let chunk_fns = specs.chunks(10).enumerate().map(|(idx, chunk)| {
            let chunk_ident = format_ident!("verify_{domain}_chunk_{idx}");
            let arms = chunk.iter().map(|s| emit_verify_arm(s, &arg_ident));
            quote! {
                #[inline(never)]
                fn #chunk_ident(
                    stem: &str,
                    buf: &[u8],
                    #arg_sig,
                ) -> Option<Result<u32, BinError>> {
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
                if let Some(res) = #chunk_ident(stem, buf, #arg_ident) {
                    return Some(res);
                }
            }
        });
        quote! {
            #(#chunk_fns)*

            pub(crate) fn #verify_fn_ident(
                stem: &str,
                buf: &[u8],
                #arg_sig,
            ) -> Option<Result<u32, BinError>> {
                #(#chunk_calls)*
                None
            }
        }
    } else {
        let arms = specs.iter().map(|s| emit_verify_arm(s, &arg_ident));
        quote! {
            pub(crate) fn #verify_fn_ident(
                stem: &str,
                buf: &[u8],
                #arg_sig,
            ) -> Option<Result<u32, BinError>> {
                match stem {
                    #(#arms,)*
                    _ => None,
                }
            }
        }
    }
}

fn emit_bin_domain_table_imports(
    specs: &[BinTableSpec],
    owned_specs: &[&BinTableSpec],
    reg: &SubStructRegistry,
) -> TokenStream {
    let action_use = if owned_specs
        .iter()
        .copied()
        .any(BinTableSpec::uses_action_data)
    {
        quote! { use crate::bin::tables::action_data::ActionData; }
    } else {
        TokenStream::new()
    };

    let mut table_mod_imports: BTreeMap<&str, Vec<Ident>> = BTreeMap::new();
    let substructs = reg.collect_direct_idents(
        owned_specs
            .iter()
            .flat_map(|s| s.cols.iter().map(|(_, k, _)| k)),
    );
    if !substructs.is_empty() {
        table_mod_imports.insert("substructs", substructs);
    }
    for spec in specs {
        if let Some(chunk_dom) = spec.shared_with_chunk_domain {
            table_mod_imports
                .entry(chunk_dom)
                .or_default()
                .push(format_ident!("Config{}Table", spec.pascal));
        }
    }
    let mod_uses = table_mod_imports.iter().map(|(&submod, idents)| {
        let sub_ident = make_ident(submod);
        let prefix = quote! { crate::bin::tables::#sub_ident };
        emit_use_group(&prefix, idents)
    });

    quote! {
        #action_use
        #(#mod_uses)*
    }
}

fn emit_bin_domain_imports(
    specs: &[BinTableSpec],
    is_map_domain: bool,
    reg: &SubStructRegistry,
) -> TokenStream {
    let owned_specs: Vec<&BinTableSpec> = specs
        .iter()
        .filter(|s| s.shared_with_chunk_domain.is_none())
        .collect();
    let affix_use = if owned_specs.iter().copied().any(BinTableSpec::uses_affix) {
        quote! { use crate::bin::affix::FConfigAffix; }
    } else {
        TokenStream::new()
    };
    let container_use = if is_map_domain {
        if owned_specs.is_empty() {
            quote! { use crate::bin::container::parse_map_bin_table; }
        } else {
            quote! { use crate::bin::container::{BinRow, parse_map_bin_table}; }
        }
    } else {
        quote! { use crate::bin::container::{BinRow, BinScratch, parse_chunk_bin_table}; }
    };
    let table_imports = emit_bin_domain_table_imports(specs, &owned_specs, reg);

    let mut type_idents = Vec::new();
    if is_map_domain {
        type_idents.push(Ident::new("BinLimits", Span::call_site()));
    }
    if !owned_specs.is_empty() {
        type_idents.push(Ident::new("BinTable", Span::call_site()));
    }
    if owned_specs.iter().copied().any(BinTableSpec::uses_vec_dim) {
        type_idents.push(Ident::new("VecDim", Span::call_site()));
    }
    let types_prefix = quote! { crate::bin::types };
    let types_use = emit_use_group(&types_prefix, &type_idents);

    let wire_use = if !owned_specs.is_empty() {
        quote! { use crate::bin::wire::WireCursor; }
    } else {
        TokenStream::new()
    };
    let map_use = if owned_specs
        .iter()
        .copied()
        .any(BinTableSpec::uses_fx_hash_map)
    {
        quote! { use rustc_hash::FxHashMap; }
    } else {
        TokenStream::new()
    };

    quote! {
        #affix_use
        #container_use
        use crate::bin::error::BinError;
        #table_imports
        #types_use
        #wire_use
        #map_use
    }
}

/// Generates or verifies a single `<domain>.rs` module in `crates/parsers/src/bin/tables/`.
pub fn emit_bin_domain_module(
    out_dir: &Path,
    domain: &str,
    specs: &[BinTableSpec],
    reg: &SubStructRegistry,
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let is_map_domain = specs.first().is_some_and(|s| s.is_map_table);
    let mod_doc = format!(
        " Auto-generated `.bin` configuration table types and parsers for the `{domain}` domain."
    );
    let imports = emit_bin_domain_imports(specs, is_map_domain, reg);
    let rows = specs.iter().map(|s| emit_row_struct_and_impl(s, reg));
    let verifier = emit_bin_domain_verifier(domain, specs, is_map_domain);

    let tokens = quote! {
        #![doc = #mod_doc]

        #imports

        #(#rows)*

        #verifier
    };
    write_or_check_tokens(&out_dir.join(format!("{domain}.rs")), tokens, mode)
}

/// Generates or verifies `crates/parsers/src/bin/tables/mod.rs`.
pub fn emit_bin_mod_rs(
    out_dir: &Path,
    all_domains: &BTreeSet<&'static str>,
    mode: CodegenMode,
) -> Result<(), XtaskError> {
    let mut all_mods: BTreeSet<&str> = all_domains.iter().copied().collect();
    all_mods.insert("action_data");
    all_mods.insert("affix_ops");
    all_mods.insert("substructs");
    let mod_idents: Vec<Ident> = all_mods.iter().map(|&m| make_ident(m)).collect();

    let chunk_domains: Vec<&'static str> = all_domains
        .iter()
        .copied()
        .filter(|&d| d != "map_instance")
        .collect();
    let count_lit = Literal::usize_unsuffixed(chunk_domains.len());
    let verifiers = chunk_domains.iter().map(|&d| {
        let dom_ident = make_ident(d);
        let fn_ident = format_ident!("verify_{d}_bin_by_stem");
        quote! { #dom_ident::#fn_ident }
    });

    let tokens = quote! {
        //! Strongly-typed `.bin` configuration tables, compound `FConfig*` sub-structs,
        //! `AffixOp3Code` schemas, and `ActionData` variants.

        #(pub mod #mod_idents;)*

        #(pub use #mod_idents::*;)*

        use crate::bin::container::{BinScratch, normalize_chunk_bin_stem};
        use crate::bin::error::BinError;
        use crate::bin::types::BinLimits;

        type ChunkDomainVerifier = fn(&str, &[u8], &mut BinScratch) -> Option<Result<u32, BinError>>;

        const CHUNK_DOMAIN_VERIFIERS: [ChunkDomainVerifier; #count_lit] = [
            #(#verifiers,)*
        ];

        /// Parses a `*_ChunkA0.bin` buffer by file stem (`config_*`) and returns its row count.
        pub fn verify_all_chunk_bin_by_stem(
            raw_stem: &str,
            buf: &[u8],
            scratch: &mut BinScratch,
        ) -> Result<u32, BinError> {
            let stem = normalize_chunk_bin_stem(raw_stem);
            for verifier in CHUNK_DOMAIN_VERIFIERS {
                if let Some(res) = verifier(stem, buf, scratch) {
                    return res;
                }
            }
            Err(BinError::UnknownTableStem { stem: stem.into() })
        }

        /// Parses an uncompressed `Maps/<Map>/config_*.bin` buffer by file stem and returns its row count.
        pub fn verify_all_map_bin_by_stem(
            stem: &str,
            buf: &[u8],
            limits: BinLimits,
        ) -> Result<u32, BinError> {
            if let Some(res) = map_instance::verify_map_instance_bin_by_stem(stem, buf, limits) {
                return res;
            }
            Err(BinError::UnknownTableStem { stem: stem.into() })
        }
    };
    write_or_check_tokens(&out_dir.join("mod.rs"), tokens, mode)
}

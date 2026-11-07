//! Stratified, non-recursive type inference engine for `DataCfg` and `LuaPreLoadTable` schemas.

use crate::datacfg::domains::{strip_config_prefix, to_pascal_case};
use crate::datacfg::vm::{
    Cursor, TableObj, Val, decrypt_in_place, eval_root_slice, parse_root_proto,
};
use crate::error::XtaskError;
use crate::u32_to_usize;
use arrayvec::ArrayVec;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;

/// Inclusive integer range observed across all rows of a column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntBounds {
    pub min: i64,
    pub max: i64,
}

impl IntBounds {
    #[must_use]
    pub const fn single(v: i64) -> Self {
        Self { min: v, max: v }
    }

    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Returns the narrowest fixed-width Rust integer type name that fits `self.min..=self.max`.
    #[must_use]
    pub const fn rust_type(self) -> &'static str {
        if self.min >= 0 {
            if self.max <= u8::MAX as i64 {
                "u8"
            } else if self.max <= u16::MAX as i64 {
                "u16"
            } else if self.max <= u32::MAX as i64 {
                "u32"
            } else {
                "u64"
            }
        } else if self.min >= i8::MIN as i64 && self.max <= i8::MAX as i64 {
            "i8"
        } else if self.min >= i16::MIN as i64 && self.max <= i16::MAX as i64 {
            "i16"
        } else if self.min >= i32::MIN as i64 && self.max <= i32::MAX as i64 {
            "i32"
        } else {
            "i64"
        }
    }

    /// Returns the alignment tier (`0` = 8B, `1` = 4B, `2` = 2B, `3` = 1B) for this integer width.
    #[must_use]
    pub const fn align_tier(self) -> u8 {
        if self.min >= 0 {
            if self.max <= u8::MAX as i64 {
                3
            } else if self.max <= u16::MAX as i64 {
                2
            } else if self.max <= u32::MAX as i64 {
                1
            } else {
                0
            }
        } else if self.min >= i8::MIN as i64 && self.max <= i8::MAX as i64 {
            3
        } else if self.min >= i16::MIN as i64 && self.max <= i16::MAX as i64 {
            2
        } else if self.min >= i32::MIN as i64 && self.max <= i32::MAX as i64 {
            1
        } else {
            0
        }
    }

    /// Returns the `TokenStream` for the narrowest fixed-width Rust integer type.
    #[must_use]
    pub fn rust_type_tokens(self) -> TokenStream {
        let ident = format_ident!("{}", self.rust_type());
        quote! { #ident }
    }
}

/// Leaf scalar type inferred from `DataCfg` constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarType {
    EmptyTable,
    Bool,
    Int(IntBounds),
    Float,
    Str,
    IntOrList(IntBounds),
}

impl ScalarType {
    pub fn merge(a: Self, b: Self) -> Result<Self, XtaskError> {
        match (a, b) {
            (Self::EmptyTable, other) | (other, Self::EmptyTable) => Ok(other),
            (Self::Bool, Self::Bool) => Ok(Self::Bool),
            (Self::Int(ba), Self::Int(bb)) => Ok(Self::Int(ba.merge(bb))),
            (Self::Float, Self::Float)
            | (Self::Int(_), Self::Float)
            | (Self::Float, Self::Int(_)) => Ok(Self::Float),
            (Self::Str, Self::Str) => Ok(Self::Str),
            (Self::IntOrList(ba), Self::Int(bb) | Self::IntOrList(bb))
            | (Self::Int(ba), Self::IntOrList(bb)) => Ok(Self::IntOrList(ba.merge(bb))),
            (
                left @ (Self::Bool | Self::Int(_) | Self::Float | Self::Str | Self::IntOrList(_)),
                right,
            ) => Err(XtaskError::IncompatibleTypes {
                left: format!("{left:?}").into(),
                right: format!("{right:?}").into(),
            }),
        }
    }

    #[must_use]
    pub const fn is_copy(self) -> bool {
        match self {
            Self::EmptyTable | Self::Bool | Self::Int(_) | Self::Float => true,
            Self::Str | Self::IntOrList(_) => false,
        }
    }

    #[must_use]
    pub const fn has_float(self) -> bool {
        matches!(self, Self::Float)
    }

    #[must_use]
    pub const fn uses_int_or_list(self) -> bool {
        matches!(self, Self::IntOrList(_))
    }

    /// Returns the alignment tier (`0` = 8B, `1` = 4B, `2` = 2B, `3` = 1B, `4` = 0B) for struct layout.
    #[must_use]
    pub const fn align_tier(self) -> u8 {
        match self {
            Self::EmptyTable => 4,
            Self::Bool => 3,
            Self::Int(b) => b.align_tier(),
            Self::Float | Self::Str | Self::IntOrList(_) => 0,
        }
    }

    /// Returns the Rust type `TokenStream` for this scalar type.
    #[must_use]
    pub fn rust_type_tokens(self) -> TokenStream {
        match self {
            Self::EmptyTable => quote! { [u8; 0] },
            Self::Bool => quote! { bool },
            Self::Int(b) => b.rust_type_tokens(),
            Self::Float => quote! { f64 },
            Self::Str => quote! { Box<str> },
            Self::IntOrList(b) => {
                let inner = b.rust_type_tokens();
                quote! { IntOrIntList<#inner> }
            }
        }
    }
}

/// Scalar or 1D/2D boxed slice of scalars (`Copy`, zero heap allocation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarOrList {
    Scalar(ScalarType),
    List { elem: ScalarType, depth: u8 },
}

impl ScalarOrList {
    pub fn merge(a: Self, b: Self) -> Result<Self, XtaskError> {
        match (a, b) {
            (Self::Scalar(ScalarType::EmptyTable), other)
            | (other, Self::Scalar(ScalarType::EmptyTable)) => Ok(other),
            (
                Self::Scalar(ScalarType::Int(ba) | ScalarType::IntOrList(ba)),
                Self::List {
                    elem: ScalarType::Int(bb),
                    depth: 1,
                },
            )
            | (
                Self::List {
                    elem: ScalarType::Int(bb),
                    depth: 1,
                },
                Self::Scalar(ScalarType::Int(ba) | ScalarType::IntOrList(ba)),
            ) => Ok(Self::Scalar(ScalarType::IntOrList(ba.merge(bb)))),
            (Self::Scalar(sa), Self::Scalar(sb)) => Ok(Self::Scalar(ScalarType::merge(sa, sb)?)),
            (
                Self::List {
                    elem: ScalarType::EmptyTable,
                    depth: da,
                },
                Self::List {
                    elem: eb,
                    depth: db,
                },
            )
            | (
                Self::List {
                    elem: eb,
                    depth: db,
                },
                Self::List {
                    elem: ScalarType::EmptyTable,
                    depth: da,
                },
            ) if da <= db => Ok(Self::List {
                elem: eb,
                depth: db,
            }),
            (
                Self::List {
                    elem: ea,
                    depth: da,
                },
                Self::List {
                    elem: eb,
                    depth: db,
                },
            ) if da == db => Ok(Self::List {
                elem: ScalarType::merge(ea, eb)?,
                depth: da,
            }),
            (left, right) => Err(XtaskError::IncompatibleTypes {
                left: format!("{left:?}").into(),
                right: format!("{right:?}").into(),
            }),
        }
    }

    #[must_use]
    pub const fn is_copy(self) -> bool {
        match self {
            Self::Scalar(s) => s.is_copy(),
            Self::List { .. } => false,
        }
    }

    #[must_use]
    pub const fn has_float(self) -> bool {
        match self {
            Self::Scalar(s) | Self::List { elem: s, .. } => s.has_float(),
        }
    }

    #[must_use]
    pub const fn uses_int_or_list(self) -> bool {
        match self {
            Self::Scalar(s) | Self::List { elem: s, .. } => s.uses_int_or_list(),
        }
    }

    /// Returns the alignment tier (`0` = 8B, `1` = 4B, `2` = 2B, `3` = 1B, `4` = 0B) for struct layout.
    #[must_use]
    pub const fn align_tier(self) -> u8 {
        match self {
            Self::Scalar(s) => s.align_tier(),
            Self::List { .. } => 0,
        }
    }

    /// Returns the Rust type `TokenStream` for this scalar or boxed slice type.
    #[must_use]
    pub fn rust_type_tokens(self) -> TokenStream {
        match self {
            Self::Scalar(s) => s.rust_type_tokens(),
            Self::List { elem, depth } => {
                let mut out = elem.rust_type_tokens();
                for _ in 0..depth {
                    out = quote! { Box<[#out]> };
                }
                out
            }
        }
    }
}

/// Nested record type (`{ Key, Value }` or fixed string-keyed sub-record).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordType {
    KvRecord {
        key: ScalarOrList,
        val: ScalarOrList,
        tuple_elems: Option<ArrayVec<ScalarOrList, 4>>,
    },
    StringRecord(BTreeMap<String, ScalarOrList>),
}

impl RecordType {
    #[must_use]
    pub fn with_empty(self) -> Self {
        match self {
            Self::KvRecord { key, val, .. } => Self::KvRecord {
                key,
                val,
                tuple_elems: None,
            },
            Self::StringRecord(m) => Self::StringRecord(m),
        }
    }

    #[must_use]
    pub fn is_copy(&self) -> bool {
        match self {
            Self::KvRecord {
                key,
                val,
                tuple_elems,
            } => {
                key.is_copy()
                    && match tuple_elems {
                        Some(elems) => elems.iter().copied().all(ScalarOrList::is_copy),
                        None => val.is_copy(),
                    }
            }
            Self::StringRecord(m) => m.values().copied().all(ScalarOrList::is_copy),
        }
    }

    #[must_use]
    pub fn has_float(&self) -> bool {
        match self {
            Self::KvRecord {
                key,
                val,
                tuple_elems,
            } => {
                key.has_float()
                    || match tuple_elems {
                        Some(elems) => elems.iter().copied().any(ScalarOrList::has_float),
                        None => val.has_float(),
                    }
            }
            Self::StringRecord(m) => m.values().copied().any(ScalarOrList::has_float),
        }
    }

    #[must_use]
    pub fn uses_int_or_list(&self) -> bool {
        match self {
            Self::KvRecord { key, val, .. } => key.uses_int_or_list() || val.uses_int_or_list(),
            Self::StringRecord(m) => m.values().copied().any(ScalarOrList::uses_int_or_list),
        }
    }

    /// Returns the alignment tier (`0..=4`) of this nested record struct.
    #[must_use]
    pub fn align_tier(&self) -> u8 {
        match self {
            Self::KvRecord {
                key,
                val,
                tuple_elems,
            } => {
                let kt = key.align_tier();
                let vt = match tuple_elems {
                    Some(elems) => elems.iter().fold(4u8, |acc, &e| acc.min(e.align_tier())),
                    None => val.align_tier(),
                };
                kt.min(vt)
            }
            Self::StringRecord(m) => m.values().fold(4u8, |acc, &v| acc.min(v.align_tier())),
        }
    }
}

/// Top-level inferred type for a `DataCfg` table column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColType {
    Simple(ScalarOrList),
    Record(RecordType),
    RecordList(RecordType),
    Map {
        key: ScalarOrList,
        val: ScalarOrList,
    },
}

fn merge_kv_records(
    ka: ScalarOrList,
    va: ScalarOrList,
    ta: Option<ArrayVec<ScalarOrList, 4>>,
    kb: ScalarOrList,
    vb: ScalarOrList,
    tb: Option<ArrayVec<ScalarOrList, 4>>,
) -> Result<RecordType, XtaskError> {
    let merged_tuple = match (ta, tb) {
        (Some(va_list), Some(vb_list)) if va_list.len() == vb_list.len() => {
            let mut elems = ArrayVec::new();
            for (x, y) in va_list.into_iter().zip(vb_list) {
                let push_res = elems.try_push(ScalarOrList::merge(x, y)?);
                assert!(push_res.is_ok());
            }
            Some(elems)
        }
        _ => None,
    };
    Ok(RecordType::KvRecord {
        key: ScalarOrList::merge(ka, kb)?,
        val: ScalarOrList::merge(va, vb)?,
        tuple_elems: merged_tuple,
    })
}

fn merge_string_records_into_col(
    mut ma: BTreeMap<String, ScalarOrList>,
    mb: BTreeMap<String, ScalarOrList>,
) -> Result<ColType, XtaskError> {
    if ma.keys().eq(mb.keys()) {
        for (k, vb) in mb {
            if let Some(va) = ma.remove(&k) {
                ma.insert(k, ScalarOrList::merge(va, vb)?);
            }
        }
        Ok(ColType::Record(RecordType::StringRecord(ma)))
    } else {
        let mut val_ty = ScalarOrList::Scalar(ScalarType::EmptyTable);
        for (_, v) in ma.into_iter().chain(mb) {
            val_ty = ScalarOrList::merge(val_ty, v)?;
        }
        Ok(ColType::Map {
            key: ScalarOrList::Scalar(ScalarType::Str),
            val: val_ty,
        })
    }
}

fn merge_records_into_col(ra: RecordType, rb: RecordType) -> Result<ColType, XtaskError> {
    match (ra, rb) {
        (
            RecordType::KvRecord {
                key: ka,
                val: va,
                tuple_elems: ta,
            },
            RecordType::KvRecord {
                key: kb,
                val: vb,
                tuple_elems: tb,
            },
        ) => Ok(ColType::Record(merge_kv_records(ka, va, ta, kb, vb, tb)?)),
        (RecordType::StringRecord(ma), RecordType::StringRecord(mb)) => {
            merge_string_records_into_col(ma, mb)
        }
        (left, right) => Err(XtaskError::IncompatibleTypes {
            left: format!("{left:?}").into(),
            right: format!("{right:?}").into(),
        }),
    }
}

impl ColType {
    pub fn merge(a: Self, b: Self) -> Result<Self, XtaskError> {
        match (a, b) {
            (Self::Simple(ScalarOrList::Scalar(ScalarType::EmptyTable)), other)
            | (other, Self::Simple(ScalarOrList::Scalar(ScalarType::EmptyTable))) => match other {
                Self::Record(r) => Ok(Self::Record(r.with_empty())),
                other_col @ (Self::Simple(_) | Self::RecordList(_) | Self::Map { .. }) => {
                    Ok(other_col)
                }
            },
            (
                Self::Simple(ScalarOrList::List {
                    elem: ScalarType::EmptyTable,
                    depth: 1,
                }),
                Self::RecordList(r),
            )
            | (
                Self::RecordList(r),
                Self::Simple(ScalarOrList::List {
                    elem: ScalarType::EmptyTable,
                    depth: 1,
                }),
            ) => Ok(Self::RecordList(r.with_empty())),
            (Self::Simple(sa), Self::Simple(sb)) => Ok(Self::Simple(ScalarOrList::merge(sa, sb)?)),
            (Self::Record(ra), Self::Record(rb)) => merge_records_into_col(ra, rb),
            (Self::RecordList(ra), Self::RecordList(rb)) => match merge_records_into_col(ra, rb)? {
                Self::Record(r) => Ok(Self::RecordList(r)),
                other @ (Self::Simple(_) | Self::RecordList(_) | Self::Map { .. }) => {
                    Err(XtaskError::IncompatibleTypes {
                        left: "RecordList".into(),
                        right: format!("{other:?}").into(),
                    })
                }
            },
            (Self::Record(RecordType::StringRecord(m)), Self::Map { key, mut val })
            | (Self::Map { key, mut val }, Self::Record(RecordType::StringRecord(m))) => {
                for (_, mv) in m {
                    val = ScalarOrList::merge(val, mv)?;
                }
                Ok(Self::Map { key, val })
            }
            (Self::Map { key: ka, val: va }, Self::Map { key: kb, val: vb }) => Ok(Self::Map {
                key: ScalarOrList::merge(ka, kb)?,
                val: ScalarOrList::merge(va, vb)?,
            }),
            (left, right) => Err(XtaskError::IncompatibleTypes {
                left: format!("{left:?}").into(),
                right: format!("{right:?}").into(),
            }),
        }
    }

    #[must_use]
    pub fn has_float(&self) -> bool {
        match self {
            Self::Simple(s) => s.has_float(),
            Self::Record(r) | Self::RecordList(r) => r.has_float(),
            Self::Map { key, val } => key.has_float() || val.has_float(),
        }
    }

    #[must_use]
    pub const fn uses_fx_hash_map(&self) -> bool {
        matches!(self, Self::Map { .. })
    }

    #[must_use]
    pub fn uses_int_or_list(&self) -> bool {
        match self {
            Self::Simple(s) => s.uses_int_or_list(),
            Self::Record(r) | Self::RecordList(r) => r.uses_int_or_list(),
            Self::Map { key, val } => key.uses_int_or_list() || val.uses_int_or_list(),
        }
    }

    /// Returns the alignment tier (`0..=4`) of this column for struct field layout ordering.
    #[must_use]
    pub fn align_tier(&self) -> u8 {
        match self {
            Self::Simple(s) => s.align_tier(),
            Self::Record(r) => r.align_tier(),
            Self::RecordList(_) | Self::Map { .. } => 0,
        }
    }

    /// Returns `(record_type, helper_struct_name)` if this column requires a generated helper struct.
    #[must_use]
    pub fn nested_record(
        &self,
        table_pascal: &str,
        col_name: &str,
    ) -> Option<(&RecordType, String)> {
        match self {
            Self::Record(r) => {
                let col_pascal = to_pascal_case(col_name);
                Some((r, format!("{table_pascal}{col_pascal}")))
            }
            Self::RecordList(r) => {
                let col_pascal = to_pascal_case(col_name);
                Some((r, format!("{table_pascal}{col_pascal}Entry")))
            }
            Self::Simple(_) | Self::Map { .. } => None,
        }
    }

    /// Returns the Rust type `TokenStream` for this column on table `table_pascal`.
    #[must_use]
    pub fn rust_type_tokens(&self, table_pascal: &str, col_name: &str) -> TokenStream {
        match self {
            Self::Simple(s) => s.rust_type_tokens(),
            Self::Record(_) => {
                let ident = format_ident!("{table_pascal}{}", to_pascal_case(col_name));
                quote! { #ident }
            }
            Self::RecordList(_) => {
                let ident = format_ident!("{table_pascal}{}Entry", to_pascal_case(col_name));
                quote! { Box<[#ident]> }
            }
            Self::Map { key, val } => {
                let kt = key.rust_type_tokens();
                let vt = val.rust_type_tokens();
                quote! { FxHashMap<#kt, #vt> }
            }
        }
    }
}

fn get_table<'a>(tables: &'a [TableObj], tid: u32, stem: &str) -> Result<&'a TableObj, XtaskError> {
    tables
        .get(u32_to_usize(tid))
        .ok_or_else(|| XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "table id out of bounds",
        })
}

fn is_valid_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().enumerate().all(|(i, c)| {
            if i == 0 {
                c.is_ascii_alphabetic() || c == '_'
            } else {
                c.is_ascii_alphanumeric() || c == '_'
            }
        })
}

fn infer_scalar(val: &Val, tables: &[TableObj], stem: &str) -> Result<ScalarType, XtaskError> {
    match val {
        Val::Nil => Err(XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "unexpected nil value",
        }),
        Val::Bool => Ok(ScalarType::Bool),
        Val::Int(i) => Ok(ScalarType::Int(IntBounds::single(*i))),
        Val::Float => Ok(ScalarType::Float),
        Val::Str(_) => Ok(ScalarType::Str),
        Val::Table(tid) => {
            let t = get_table(tables, *tid, stem)?;
            if t.list.is_empty() && t.map.is_empty() {
                Ok(ScalarType::EmptyTable)
            } else {
                Err(XtaskError::MalformedBytecode {
                    stem: stem.into(),
                    context: "expected scalar or empty table",
                })
            }
        }
    }
}

fn infer_scalar_or_list_depth1(
    val: &Val,
    tables: &[TableObj],
    stem: &str,
) -> Result<ScalarOrList, XtaskError> {
    if let Val::Table(tid) = val {
        let t = get_table(tables, *tid, stem)?;
        if !t.list.is_empty() {
            let mut elem = infer_scalar(&t.list[0], tables, stem)?;
            for item in &t.list[1..] {
                elem = ScalarType::merge(elem, infer_scalar(item, tables, stem)?)?;
            }
            return Ok(ScalarOrList::List { elem, depth: 1 });
        }
    }
    Ok(ScalarOrList::Scalar(infer_scalar(val, tables, stem)?))
}

fn infer_map_or_record(
    t: &TableObj,
    tables: &[TableObj],
    stem: &str,
) -> Result<ColType, XtaskError> {
    let all_str_ident_keys = t
        .map
        .iter()
        .all(|(k, _)| matches!(k, Val::Str(s) if is_valid_ident(s)));
    if all_str_ident_keys {
        let mut field_map = BTreeMap::new();
        for (k, v) in &t.map {
            if let Val::Str(s) = k {
                field_map.insert(s.to_string(), infer_scalar_or_list_depth1(v, tables, stem)?);
            }
        }
        if field_map.len() == 2
            && let Some(k_ty) = field_map.remove("Key")
            && let Some(v_ty) = field_map.remove("Value")
        {
            let mut tuple_elems = None;
            for (k, v) in &t.map {
                if let (Val::Str(s), Val::Table(vtid)) = (k, v)
                    && &**s == "Value"
                {
                    let vt = get_table(tables, *vtid, stem)?;
                    if (2..=4).contains(&vt.list.len())
                        && vt.list.iter().all(|x| !matches!(x, Val::Table(_)))
                    {
                        let mut elems = ArrayVec::new();
                        for item in &vt.list {
                            let push_res =
                                elems.try_push(infer_scalar_or_list_depth1(item, tables, stem)?);
                            assert!(push_res.is_ok());
                        }
                        tuple_elems = Some(elems);
                    }
                }
            }
            return Ok(ColType::Record(RecordType::KvRecord {
                key: k_ty,
                val: v_ty,
                tuple_elems,
            }));
        }
        return Ok(ColType::Record(RecordType::StringRecord(field_map)));
    }
    let mut k_ty = infer_scalar_or_list_depth1(&t.map[0].0, tables, stem)?;
    let mut v_ty = infer_scalar_or_list_depth1(&t.map[0].1, tables, stem)?;
    for (k, v) in &t.map[1..] {
        k_ty = ScalarOrList::merge(k_ty, infer_scalar_or_list_depth1(k, tables, stem)?)?;
        v_ty = ScalarOrList::merge(v_ty, infer_scalar_or_list_depth1(v, tables, stem)?)?;
    }
    Ok(ColType::Map {
        key: k_ty,
        val: v_ty,
    })
}

fn lift_into_list(col: ColType, stem: &str) -> Result<ColType, XtaskError> {
    match col {
        ColType::Simple(ScalarOrList::Scalar(s)) => {
            Ok(ColType::Simple(ScalarOrList::List { elem: s, depth: 1 }))
        }
        ColType::Simple(ScalarOrList::List { elem, depth }) => {
            Ok(ColType::Simple(ScalarOrList::List {
                elem,
                depth: depth + 1,
            }))
        }
        ColType::Record(rec) => Ok(ColType::RecordList(rec)),
        ColType::RecordList(_) | ColType::Map { .. } => Err(XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "unsupported nested list element type",
        }),
    }
}

fn infer_elem_val(val: &Val, tables: &[TableObj], stem: &str) -> Result<ColType, XtaskError> {
    if let Val::Table(tid) = val {
        let t = get_table(tables, *tid, stem)?;
        if t.list.is_empty() && t.map.is_empty() {
            return Ok(ColType::Simple(ScalarOrList::Scalar(
                ScalarType::EmptyTable,
            )));
        }
        if !t.list.is_empty() {
            let mut elem = infer_scalar_or_list_depth1(&t.list[0], tables, stem)?;
            for item in &t.list[1..] {
                elem = ScalarOrList::merge(elem, infer_scalar_or_list_depth1(item, tables, stem)?)?;
            }
            return lift_into_list(ColType::Simple(elem), stem);
        }
        return infer_map_or_record(t, tables, stem);
    }
    Ok(ColType::Simple(ScalarOrList::Scalar(infer_scalar(
        val, tables, stem,
    )?)))
}

/// Infers the [`ColType`] of a column or preload entry value without recursion.
pub fn infer_col_val(val: &Val, tables: &[TableObj], stem: &str) -> Result<ColType, XtaskError> {
    if let Val::Table(tid) = val {
        let t = get_table(tables, *tid, stem)?;
        if !t.list.is_empty() {
            let mut elem = infer_elem_val(&t.list[0], tables, stem)?;
            for item in &t.list[1..] {
                elem = ColType::merge(elem, infer_elem_val(item, tables, stem)?)?;
            }
            return lift_into_list(elem, stem);
        }
    }
    infer_elem_val(val, tables, stem)
}

/// Inferred primary key type for a `LuaTable/DataCfg` table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaTableKey {
    /// Default `u32` key for empty tables.
    DefaultU32,
    /// Single scalar or 1D scalar list key (`K`).
    Single(ScalarOrList),
    /// Composite 3D key (`(K1, K2, K3)`).
    Triple(ScalarOrList, ScalarOrList, ScalarOrList),
}

impl LuaTableKey {
    /// Returns the Rust primary key type `TokenStream` for `DataTable<K, Row>`.
    #[must_use]
    pub fn rust_type_tokens(self) -> TokenStream {
        match self {
            Self::DefaultU32 => quote! { u32 },
            Self::Single(k) => k.rust_type_tokens(),
            Self::Triple(k1, k2, k3) => {
                let t1 = k1.rust_type_tokens();
                let t2 = k2.rust_type_tokens();
                let t3 = k3.rust_type_tokens();
                quote! { (#t1, #t2, #t3) }
            }
        }
    }
}

/// Schema specification for a standard `LuaTable/DataCfg` table.
#[derive(Debug)]
pub struct LuaTableSpec {
    pub stem: String,
    pub pascal: String,
    pub key: LuaTableKey,
    pub is_3d: bool,
    pub cols: BTreeMap<String, ColType>,
}

/// Shape of a `LuaPreLoadTable/DataCfg` preload table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreloadKind {
    List(ScalarOrList),
    Map1D(ScalarOrList, ScalarOrList),
    Map2D(ScalarOrList, ScalarOrList, ScalarOrList),
}

/// Schema specification for a `LuaPreLoadTable/DataCfg` preload table.
#[derive(Debug)]
pub struct PreloadSpec {
    pub stem: String,
    pub pascal: String,
    pub kind: PreloadKind,
    pub uses_int_or_list: bool,
}

fn merge_opt_scalar_or_list(
    slot: &mut Option<ScalarOrList>,
    next: ScalarOrList,
) -> Result<(), XtaskError> {
    *slot = Some(match slot.take() {
        Some(prev) => ScalarOrList::merge(prev, next)?,
        None => next,
    });
    Ok(())
}

fn collect_3d_keys_and_rows(
    stem: &str,
    main_tid: u32,
    tables: &[TableObj],
    row_tids: &mut Vec<u32>,
) -> Result<LuaTableKey, XtaskError> {
    let mut k1_ty = None;
    let mut k2_ty = None;
    let mut k3_ty = None;
    for (k1, v1) in &get_table(tables, main_tid, stem)?.map {
        merge_opt_scalar_or_list(&mut k1_ty, infer_scalar_or_list_depth1(k1, tables, stem)?)?;
        let Val::Table(tid1) = v1 else {
            return Err(XtaskError::MalformedBytecode {
                stem: stem.into(),
                context: "3d l1",
            });
        };
        for (k2, v2) in &get_table(tables, *tid1, stem)?.map {
            merge_opt_scalar_or_list(&mut k2_ty, infer_scalar_or_list_depth1(k2, tables, stem)?)?;
            let Val::Table(tid2) = v2 else {
                return Err(XtaskError::MalformedBytecode {
                    stem: stem.into(),
                    context: "3d l2",
                });
            };
            for (k3, v3) in &get_table(tables, *tid2, stem)?.map {
                merge_opt_scalar_or_list(
                    &mut k3_ty,
                    infer_scalar_or_list_depth1(k3, tables, stem)?,
                )?;
                if let Val::Table(rtid) = v3 {
                    row_tids.push(*rtid);
                }
            }
        }
    }
    let t1 = k1_ty.ok_or_else(|| XtaskError::MalformedBytecode {
        stem: stem.into(),
        context: "missing 3d k1",
    })?;
    let t2 = k2_ty.ok_or_else(|| XtaskError::MalformedBytecode {
        stem: stem.into(),
        context: "missing 3d k2",
    })?;
    let t3 = k3_ty.ok_or_else(|| XtaskError::MalformedBytecode {
        stem: stem.into(),
        context: "missing 3d k3",
    })?;
    Ok(LuaTableKey::Triple(t1, t2, t3))
}

fn detect_3d_table(main_tbl: &TableObj, tables: &[TableObj]) -> bool {
    main_tbl.map.first().is_some_and(|(_, v1)| {
        matches!(v1, Val::Table(t1) if tables.get(u32_to_usize(*t1)).is_some_and(|tbl1| {
            tbl1.map.first().is_some_and(|(k2, v2)| {
                matches!(k2, Val::Int(_))
                    && matches!(v2, Val::Table(t2) if tables.get(u32_to_usize(*t2)).is_some_and(|tbl2| {
                        tbl2.map.first().is_some_and(|(k3, _)| matches!(k3, Val::Int(_)))
                    }))
            })
        }))
    })
}

/// Decrypts, executes, and infers the [`LuaTableSpec`] for a `LuaTable/DataCfg` `.out` buffer.
pub fn analyze_luatable(stem: String, buf: &mut [u8]) -> Result<LuaTableSpec, XtaskError> {
    let pascal = to_pascal_case(strip_config_prefix(&stem));
    decrypt_in_place(buf);
    let mut cur = Cursor::new(buf, &stem);
    let proto = parse_root_proto(&mut cur)?;
    let mut regs: [Val; 256] = core::array::from_fn(|_| Val::Nil);
    let mut tables = Vec::new();
    eval_root_slice(&stem, &proto, &mut regs, &mut tables)?;

    let (Val::Table(main_tid), Val::Table(most_tid), Val::Table(def_tid)) =
        (&regs[0], &regs[1], &regs[2])
    else {
        return Err(XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "missing R[0..=2] tables",
        });
    };
    let (main_tid, most_tid, def_tid) = (*main_tid, *most_tid, *def_tid);
    let main_tbl = get_table(&tables, main_tid, &stem)?;
    let is_3d = detect_3d_table(main_tbl, &tables);

    let mut row_tids = vec![most_tid, def_tid];
    let key = if is_3d {
        collect_3d_keys_and_rows(&stem, main_tid, &tables, &mut row_tids)?
    } else {
        let mut k_ty = None;
        for (k, v) in &main_tbl.map {
            merge_opt_scalar_or_list(&mut k_ty, infer_scalar_or_list_depth1(k, &tables, &stem)?)?;
            if let Val::Table(rtid) = v {
                row_tids.push(*rtid);
            }
        }
        match k_ty {
            Some(kt) => LuaTableKey::Single(kt),
            None => LuaTableKey::DefaultU32,
        }
    };

    let mut cols: BTreeMap<String, ColType> = BTreeMap::new();
    for rtid in row_tids {
        for (k, v) in &get_table(&tables, rtid, &stem)?.map {
            let Val::Str(col_name) = k else { continue };
            let col_str = col_name.to_string();
            let inferred = infer_col_val(v, &tables, &stem)?;
            if let Some(existing) = cols.remove(&col_str) {
                cols.insert(col_str, ColType::merge(existing, inferred)?);
            } else {
                cols.insert(col_str, inferred);
            }
        }
    }
    Ok(LuaTableSpec {
        stem,
        pascal,
        key,
        is_3d,
        cols,
    })
}

struct InferredPreload {
    kind: PreloadKind,
    uses_int_or_list: bool,
}

fn infer_preload_2d(
    stem: &str,
    root_tbl: &TableObj,
    tables: &[TableObj],
) -> Result<InferredPreload, XtaskError> {
    let mut k1_ty = None;
    let mut k2_ty = None;
    let mut v_ty = None;
    for (k1, v1) in &root_tbl.map {
        merge_opt_scalar_or_list(&mut k1_ty, infer_scalar_or_list_depth1(k1, tables, stem)?)?;
        let Val::Table(sub_tid) = v1 else {
            return Err(XtaskError::MalformedBytecode {
                stem: stem.into(),
                context: "2d sub",
            });
        };
        for (k2, v2) in &get_table(tables, *sub_tid, stem)?.map {
            merge_opt_scalar_or_list(&mut k2_ty, infer_scalar_or_list_depth1(k2, tables, stem)?)?;
            merge_opt_scalar_or_list(&mut v_ty, infer_scalar_or_list_depth1(v2, tables, stem)?)?;
        }
    }
    let vt = v_ty.ok_or_else(|| XtaskError::MalformedBytecode {
        stem: stem.into(),
        context: "2d val",
    })?;
    let k1 = k1_ty.ok_or_else(|| XtaskError::MalformedBytecode {
        stem: stem.into(),
        context: "2d k1",
    })?;
    let k2 = k2_ty.ok_or_else(|| XtaskError::MalformedBytecode {
        stem: stem.into(),
        context: "2d k2",
    })?;
    Ok(InferredPreload {
        kind: PreloadKind::Map2D(k1, k2, vt),
        uses_int_or_list: vt.uses_int_or_list(),
    })
}

/// Decrypts, executes, and infers the [`PreloadSpec`] for a `LuaPreLoadTable/DataCfg` `.out` buffer.
pub fn analyze_preload(stem: String, buf: &mut [u8]) -> Result<PreloadSpec, XtaskError> {
    let pascal = to_pascal_case(&stem);
    decrypt_in_place(buf);
    let mut cur = Cursor::new(buf, &stem);
    let proto = parse_root_proto(&mut cur)?;
    let mut regs: [Val; 256] = core::array::from_fn(|_| Val::Nil);
    let mut tables = Vec::new();
    eval_root_slice(&stem, &proto, &mut regs, &mut tables)?;
    let Val::Table(main_tid) = regs[0] else {
        return Err(XtaskError::MalformedBytecode {
            stem: stem.into(),
            context: "missing preload R[0]",
        });
    };
    let root_tbl = get_table(&tables, main_tid, &stem)?;
    let inferred = if !root_tbl.list.is_empty() {
        let mut elem = infer_scalar_or_list_depth1(&root_tbl.list[0], &tables, &stem)?;
        for item in &root_tbl.list[1..] {
            elem = ScalarOrList::merge(elem, infer_scalar_or_list_depth1(item, &tables, &stem)?)?;
        }
        InferredPreload {
            kind: PreloadKind::List(elem),
            uses_int_or_list: false,
        }
    } else if root_tbl.map.first().is_some_and(|(_, v)| {
        matches!(v, Val::Table(tid) if tables.get(u32_to_usize(*tid)).is_some_and(|t| !t.map.is_empty()))
    }) {
        infer_preload_2d(&stem, root_tbl, &tables)?
    } else {
        let mut k_ty = None;
        let mut v_ty = None;
        for (k, v) in &root_tbl.map {
            merge_opt_scalar_or_list(&mut k_ty, infer_scalar_or_list_depth1(k, &tables, &stem)?)?;
            merge_opt_scalar_or_list(&mut v_ty, infer_scalar_or_list_depth1(v, &tables, &stem)?)?;
        }
        let kt = k_ty.ok_or_else(|| XtaskError::MalformedBytecode {
            stem: stem.as_str().into(),
            context: "1d key",
        })?;
        let vt = v_ty.ok_or_else(|| XtaskError::MalformedBytecode {
            stem: stem.as_str().into(),
            context: "1d val",
        })?;
        InferredPreload {
            kind: PreloadKind::Map1D(kt, vt),
            uses_int_or_list: vt.uses_int_or_list(),
        }
    };
    Ok(PreloadSpec {
        stem,
        pascal,
        kind: inferred.kind,
        uses_int_or_list: inferred.uses_int_or_list,
    })
}

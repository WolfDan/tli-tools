//! High-performance `DataCfg` static table evaluator and typed Rust struct decoder.

use crate::luac::crypto::decrypt_luac_in_place;
use crate::luac::error::{DataCfgError, LuacError};
use crate::luac::opcode::{LuaInstruction, LuaOpcode};
use crate::luac::transpile::{WireCursor, read_chunk_header};
use crate::luac::types::{ByteSpan, DataCfgLimits, IntOrIntList, LuacLimitKind, LuacLimits};
use crate::{FixedVec, bounded_len_u32, u32_to_usize};
use arrayvec::ArrayVec;
use core::borrow::Borrow;
use core::hash::Hash;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::collections::hash_map::{Iter, Keys, Values};

const OPEN_TABLES_MAX: u32 = 256;
const OPEN_TABLES_MAX_USIZE: usize = u32_to_usize(OPEN_TABLES_MAX);
const LFIELDS_PER_FLUSH: u32 = 50;

/// Strongly-typed `LuaTable/DataCfg` table backed by an `O(1)` [`FxHashMap<K, Row>`].
#[derive(Debug, Clone, PartialEq)]
pub struct DataTable<K: Eq + Hash, Row> {
    rows: FxHashMap<K, Row>,
    most_row: Row,
    default_row: Row,
    language_keys: Box<[Box<str>]>,
}

impl<K: Eq + Hash, Row> DataTable<K, Row> {
    /// Returns the number of rows in the table.
    #[inline]
    #[must_use]
    pub fn len(&self) -> u32 {
        bounded_len_u32(self.rows.len())
    }

    /// Returns `true` if the table contains no rows.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Returns a reference to the underlying `O(1)` [`FxHashMap<K, Row>`].
    #[inline]
    #[must_use]
    pub const fn rows(&self) -> &FxHashMap<K, Row> {
        &self.rows
    }

    /// Returns an iterator over the table's keys.
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, Row> {
        self.rows.keys()
    }

    /// Returns an iterator over the table's row structs.
    #[inline]
    pub fn values(&self) -> Values<'_, K, Row> {
        self.rows.values()
    }

    /// Returns the `most_value` (`R[1]`) shared default row.
    #[inline]
    #[must_use]
    pub const fn most_row(&self) -> &Row {
        &self.most_row
    }

    /// Returns the `default_value` (`R[2]`) fallback row.
    #[inline]
    #[must_use]
    pub const fn default_row(&self) -> &Row {
        &self.default_row
    }

    /// Returns the localized column names (`R[3]`).
    #[inline]
    #[must_use]
    pub fn language_keys(&self) -> &[Box<str>] {
        &self.language_keys
    }

    /// Looks up `key` in `O(1)` time and returns the corresponding `Row`.
    #[inline]
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&Row>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.rows.get(key)
    }

    /// Iterates over `(&K, &Row)` pairs in the table.
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, Row> {
        self.rows.iter()
    }
}

/// Strongly-typed `LuaPreLoadTable/DataCfg` map table backed by an `O(1)` [`FxHashMap<K, V>`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreloadTable<K: Eq + Hash, V> {
    entries: FxHashMap<K, V>,
}

impl<K: Eq + Hash, V> PreloadTable<K, V> {
    /// Returns the number of entries in the preload table.
    #[inline]
    #[must_use]
    pub fn len(&self) -> u32 {
        bounded_len_u32(self.entries.len())
    }

    /// Returns `true` if the preload table contains no entries.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns a reference to the underlying `O(1)` [`FxHashMap<K, V>`].
    #[inline]
    #[must_use]
    pub const fn entries(&self) -> &FxHashMap<K, V> {
        &self.entries
    }

    /// Returns an iterator over the preload table's keys.
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        self.entries.keys()
    }

    /// Returns an iterator over the preload table's values.
    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        self.entries.values()
    }

    /// Looks up `key` in `O(1)` time and returns the corresponding value.
    #[inline]
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.entries.get(key)
    }

    /// Iterates over `(&K, &V)` pairs in the preload table.
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.entries.iter()
    }
}

/// Evaluated value inside the `DataCfg` register machine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EvalVal {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(ByteSpan),
    OpenTable(u8),
    Table(u32),
}
const _: () = assert!(core::mem::size_of::<EvalVal>() == 16);

#[derive(Debug, Clone, Copy)]
pub(crate) enum ClosedTable {
    Empty,
    List { start: u32, len: u32 },
    Map { start: u32, len: u32 },
}
const _: () = assert!(core::mem::size_of::<ClosedTable>() == 12);

#[derive(Debug, Default)]
struct OpenTableSlot {
    list: Vec<EvalVal>,
    map: Vec<(EvalVal, EvalVal)>,
}

/// Reusable pre-allocated scratch buffers for `DataCfg` evaluation.
#[derive(Debug)]
pub struct DataCfgScratch {
    limits: DataCfgLimits,
    code: FixedVec<LuaInstruction>,
    consts: FixedVec<EvalVal>,
    open_slots: Box<[OpenTableSlot]>,
    free_open: ArrayVec<u8, OPEN_TABLES_MAX_USIZE>,
    closed_tables: FixedVec<ClosedTable>,
    closed_lists: FixedVec<EvalVal>,
    closed_maps: FixedVec<(EvalVal, EvalVal)>,
}

impl DataCfgScratch {
    /// Allocates a reusable `DataCfgScratch` bounded by `limits`.
    #[must_use]
    pub fn with_limits(limits: DataCfgLimits) -> Self {
        Self {
            limits,
            code: FixedVec::with_capacity(limits.luac().instructions_count_max()),
            consts: FixedVec::with_capacity(limits.luac().constants_count_max()),
            open_slots: (0..OPEN_TABLES_MAX)
                .map(|_| OpenTableSlot::default())
                .collect(),
            free_open: ArrayVec::new(),
            closed_tables: FixedVec::with_capacity(limits.tables_count_max()),
            closed_lists: FixedVec::with_capacity(limits.list_items_max()),
            closed_maps: FixedVec::with_capacity(limits.map_entries_max()),
        }
    }

    fn reset(&mut self) {
        self.code.clear();
        self.consts.clear();
        self.free_open.clear();
        for slot in (0..=u8::MAX).rev() {
            let slot_idx = u32_to_usize(u32::from(slot));
            self.open_slots[slot_idx].list.clear();
            self.open_slots[slot_idx].map.clear();
            let push_res = self.free_open.try_push(slot);
            assert!(push_res.is_ok());
        }
        self.closed_tables.clear();
        self.closed_lists.clear();
        self.closed_maps.clear();
    }

    fn alloc_open_table(&mut self) -> Result<u8, LuacError> {
        self.free_open.pop().ok_or(LuacError::LimitExceeded {
            kind: LuacLimitKind::DataCfgTablesCount,
            actual: u64::from(OPEN_TABLES_MAX + 1),
            max: u64::from(OPEN_TABLES_MAX),
        })
    }

    fn close_open_table(&mut self, slot: u8) -> Result<u32, LuacError> {
        let slot_idx = u32_to_usize(u32::from(slot));
        assert!(
            self.open_slots[slot_idx].list.is_empty() || self.open_slots[slot_idx].map.is_empty()
        );
        let closed = if !self.open_slots[slot_idx].list.is_empty() {
            let start = self.closed_lists.len();
            let len = bounded_len_u32(self.open_slots[slot_idx].list.len());
            for &item in &self.open_slots[slot_idx].list {
                self.closed_lists
                    .try_push(item)
                    .map_err(|_| LuacError::LimitExceeded {
                        kind: LuacLimitKind::DataCfgListItems,
                        actual: u64::from(self.closed_lists.len()) + 1,
                        max: u64::from(self.closed_lists.cap()),
                    })?;
            }
            self.open_slots[slot_idx].list.clear();
            ClosedTable::List { start, len }
        } else if !self.open_slots[slot_idx].map.is_empty() {
            let start = self.closed_maps.len();
            let len = bounded_len_u32(self.open_slots[slot_idx].map.len());
            for &entry in &self.open_slots[slot_idx].map {
                self.closed_maps
                    .try_push(entry)
                    .map_err(|_| LuacError::LimitExceeded {
                        kind: LuacLimitKind::DataCfgMapEntries,
                        actual: u64::from(self.closed_maps.len()) + 1,
                        max: u64::from(self.closed_maps.cap()),
                    })?;
            }
            self.open_slots[slot_idx].map.clear();
            ClosedTable::Map { start, len }
        } else {
            ClosedTable::Empty
        };

        let tid = self.closed_tables.len();
        self.closed_tables
            .try_push(closed)
            .map_err(|_| LuacError::LimitExceeded {
                kind: LuacLimitKind::DataCfgTablesCount,
                actual: u64::from(tid) + 1,
                max: u64::from(self.closed_tables.cap()),
            })?;
        let push_res = self.free_open.try_push(slot);
        assert!(push_res.is_ok());
        Ok(tid)
    }

    fn materialize_reg(
        &mut self,
        regs: &mut [EvalVal; 256],
        reg: u8,
    ) -> Result<EvalVal, LuacError> {
        let reg_idx = usize::from(reg);
        if let EvalVal::OpenTable(slot) = regs[reg_idx] {
            let tid = self.close_open_table(slot)?;
            regs[reg_idx] = EvalVal::Table(tid);
        }
        Ok(regs[reg_idx])
    }

    fn resolve_rk(
        &mut self,
        regs: &mut [EvalVal; 256],
        rk: u16,
        pc: u32,
    ) -> Result<EvalVal, LuacError> {
        if (rk & 0x100) != 0 {
            let idx = u32::from(rk & 0xFF);
            let slice = self.consts.as_slice();
            slice.get(u32_to_usize(idx)).copied().ok_or_else(|| {
                LuacError::from(DataCfgError::ConstantIndexOutOfBounds {
                    pc,
                    index: idx,
                    count: bounded_len_u32(slice.len()),
                })
            })
        } else {
            assert!(rk <= 0xFF);
            let reg = (rk & 0xFF) as u8;
            self.materialize_reg(regs, reg)
        }
    }
}

/// Read-only decoding context over a completed `DataCfgScratch` evaluation.
pub(crate) struct EvalContext<'a, 's> {
    buf: &'a [u8],
    scratch: &'s DataCfgScratch,
}

impl<'a, 's> EvalContext<'a, 's> {
    #[inline]
    pub(crate) fn table(&self, tid: u32) -> ClosedTable {
        self.scratch.closed_tables.as_slice()[u32_to_usize(tid)]
    }

    #[inline]
    pub(crate) fn list_slice(
        &self,
        tbl: ClosedTable,
        table: &'static str,
        field: &'static str,
    ) -> Result<&'s [EvalVal], LuacError> {
        match tbl {
            ClosedTable::Empty => Ok(&[]),
            ClosedTable::List { start, len } => {
                let s = u32_to_usize(start);
                let e = s + u32_to_usize(len);
                Ok(&self.scratch.closed_lists.as_slice()[s..e])
            }
            ClosedTable::Map { .. } => Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "list table",
            }
            .into()),
        }
    }

    #[inline]
    pub(crate) fn map_slice(
        &self,
        tbl: ClosedTable,
        context: &'static str,
    ) -> Result<&'s [(EvalVal, EvalVal)], LuacError> {
        match tbl {
            ClosedTable::Empty => Ok(&[]),
            ClosedTable::Map { start, len } => {
                let s = u32_to_usize(start);
                let e = s + u32_to_usize(len);
                Ok(&self.scratch.closed_maps.as_slice()[s..e])
            }
            ClosedTable::List { .. } => Err(DataCfgError::ExpectedTable { context }.into()),
        }
    }

    #[inline]
    pub(crate) fn str_bytes(&self, span: ByteSpan) -> &'a [u8] {
        let start = u32_to_usize(span.offset);
        let end = start + u32_to_usize(span.len);
        &self.buf[start..end]
    }

    #[inline]
    pub(crate) fn decode_str(&self, span: ByteSpan) -> Result<&'a str, LuacError> {
        core::str::from_utf8(self.str_bytes(span))
            .map_err(|source| LuacError::InvalidUtf8 { source })
    }

    /// Decodes a single field value into `T`.
    #[inline]
    pub(crate) fn decode_field<T: FromEvalVal>(
        &self,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<T, LuacError> {
        T::from_eval_val(self, val, table, field)
    }

    /// Constructs an `UnknownField` (or `InvalidUtf8`) error for an unrecognized column name.
    #[inline]
    #[must_use]
    pub(crate) fn unknown_field(&self, table: &'static str, raw_field: &[u8]) -> LuacError {
        match core::str::from_utf8(raw_field) {
            Ok(s) => DataCfgError::UnknownField {
                table,
                field: s.into(),
            }
            .into(),
            Err(source) => LuacError::InvalidUtf8 { source },
        }
    }

    /// Returns the map slice of a sub-table value `val`, or a `TypeMismatch` error.
    #[inline]
    pub(crate) fn table_map(
        &self,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<&'s [(EvalVal, EvalVal)], LuacError> {
        let EvalVal::Table(tid) = val else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "map sub-table",
            }
            .into());
        };
        match self.table(tid) {
            ClosedTable::Empty => Ok(&[]),
            ClosedTable::Map { start, len } => {
                let s = u32_to_usize(start);
                let e = s + u32_to_usize(len);
                Ok(&self.scratch.closed_maps.as_slice()[s..e])
            }
            ClosedTable::List { .. } => Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "map sub-table",
            }
            .into()),
        }
    }

    /// Returns the raw byte slice of a string key `k` in a sub-table, or a `TypeMismatch` error.
    #[inline]
    pub(crate) fn decode_str_key(
        &self,
        key: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<&'a [u8], LuacError> {
        let EvalVal::Str(k_span) = key else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "string field key",
            }
            .into());
        };
        Ok(self.str_bytes(k_span))
    }

    /// Decodes a `{ Key = K, Value = V }` Lua sub-table into `(K, V)`.
    pub(crate) fn decode_kv_pair<K: FromEvalVal + Default, V: FromEvalVal + Default>(
        &self,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<(K, V), LuacError> {
        let mut key = K::default();
        let mut value = V::default();
        for &(k, v) in self.table_map(val, table, field)? {
            match self.decode_str_key(k, table, field)? {
                b"Key" => key = K::from_eval_val(self, v, table, field)?,
                b"Value" => value = V::from_eval_val(self, v, table, field)?,
                _ => {
                    return Err(DataCfgError::TypeMismatch {
                        table,
                        field,
                        expected: "Key or Value field",
                    }
                    .into());
                }
            }
        }
        Ok((key, value))
    }
}

/// Trait implemented by field types that can be decoded from an `EvalVal`.
pub(crate) trait FromEvalVal: Sized {
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError>;
}

/// Trait implemented by generated `Config*Row` structs.
pub(crate) trait DataCfgRow: Clone + Default {
    const TABLE_NAME: &'static str;

    fn set_field(
        &mut self,
        ctx: &EvalContext<'_, '_>,
        field: &[u8],
        val: EvalVal,
    ) -> Result<(), LuacError>;
}

impl FromEvalVal for bool {
    #[inline]
    fn from_eval_val(
        _ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        match val {
            EvalVal::Bool(b) => Ok(b),
            EvalVal::Nil
            | EvalVal::Int(_)
            | EvalVal::Float(_)
            | EvalVal::Str(_)
            | EvalVal::OpenTable(_)
            | EvalVal::Table(_) => Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "bool",
            }
            .into()),
        }
    }
}

macro_rules! impl_from_eval_int {
    ($($ty:ty => $name:literal),* $(,)?) => {
        $(
            impl FromEvalVal for $ty {
                #[inline]
                fn from_eval_val(
                    _ctx: &EvalContext<'_, '_>,
                    val: EvalVal,
                    table: &'static str,
                    field: &'static str,
                ) -> Result<Self, LuacError> {
                    let EvalVal::Int(raw) = val else {
                        return Err(DataCfgError::TypeMismatch {
                            table,
                            field,
                            expected: $name,
                        }
                        .into());
                    };
                    <$ty>::try_from(raw).map_err(|_| {
                        DataCfgError::IntegerOverflow {
                            table,
                            field,
                            value: raw,
                            target_type: $name,
                        }
                        .into()
                    })
                }
            }
        )*
    };
}

impl_from_eval_int!(
    u8 => "u8",
    u16 => "u16",
    u32 => "u32",
    u64 => "u64",
    i8 => "i8",
    i16 => "i16",
    i32 => "i32",
    i64 => "i64",
);

#[inline]
#[expect(
    clippy::cast_precision_loss,
    reason = "Lua 5.3 Number columns store whole numbers as i64 and fractional numbers as f64"
)]
const fn i64_to_f64_lossy(x: i64) -> f64 {
    x as f64
}

impl FromEvalVal for f64 {
    #[inline]
    fn from_eval_val(
        _ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        match val {
            EvalVal::Float(f) => Ok(f),
            EvalVal::Int(i) => Ok(i64_to_f64_lossy(i)),
            EvalVal::Nil
            | EvalVal::Bool(_)
            | EvalVal::Str(_)
            | EvalVal::OpenTable(_)
            | EvalVal::Table(_) => Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "f64",
            }
            .into()),
        }
    }
}

impl FromEvalVal for Box<str> {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let EvalVal::Str(span) = val else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "Box<str>",
            }
            .into());
        };
        Ok(ctx.decode_str(span)?.into())
    }
}

impl FromEvalVal for [u8; 0] {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let EvalVal::Table(tid) = val else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "empty table",
            }
            .into());
        };
        if !matches!(ctx.table(tid), ClosedTable::Empty) {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "empty table",
            }
            .into());
        }
        Ok([])
    }
}

impl<T: FromEvalVal> FromEvalVal for Box<[T]> {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let EvalVal::Table(tid) = val else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "table for Box<[T]>",
            }
            .into());
        };
        let items = ctx.list_slice(ctx.table(tid), table, field)?;
        let mut out = Vec::with_capacity(items.len());
        for &item in items {
            out.push(T::from_eval_val(ctx, item, table, field)?);
        }
        Ok(out.into_boxed_slice())
    }
}

impl<K: FromEvalVal + Eq + Hash, V: FromEvalVal> FromEvalVal for FxHashMap<K, V> {
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let entries = ctx.table_map(val, table, field)?;
        let mut map = Self::with_capacity_and_hasher(entries.len(), FxBuildHasher);
        for &(k, v) in entries {
            let key = K::from_eval_val(ctx, k, table, field)?;
            let value = V::from_eval_val(ctx, v, table, field)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<A: FromEvalVal, B: FromEvalVal> FromEvalVal for (A, B) {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let EvalVal::Table(tid) = val else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "2-element list table",
            }
            .into());
        };
        let [a, b] = *ctx.list_slice(ctx.table(tid), table, field)? else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "2-element list table",
            }
            .into());
        };
        Ok((
            A::from_eval_val(ctx, a, table, field)?,
            B::from_eval_val(ctx, b, table, field)?,
        ))
    }
}

impl<A: FromEvalVal, B: FromEvalVal, C: FromEvalVal> FromEvalVal for (A, B, C) {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let EvalVal::Table(tid) = val else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "3-element list table",
            }
            .into());
        };
        let [a, b, c] = *ctx.list_slice(ctx.table(tid), table, field)? else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "3-element list table",
            }
            .into());
        };
        Ok((
            A::from_eval_val(ctx, a, table, field)?,
            B::from_eval_val(ctx, b, table, field)?,
            C::from_eval_val(ctx, c, table, field)?,
        ))
    }
}

impl<A: FromEvalVal, B: FromEvalVal, C: FromEvalVal, D: FromEvalVal> FromEvalVal for (A, B, C, D) {
    #[inline]
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        let EvalVal::Table(tid) = val else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "4-element list table",
            }
            .into());
        };
        let [a, b, c, d] = *ctx.list_slice(ctx.table(tid), table, field)? else {
            return Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "4-element list table",
            }
            .into());
        };
        Ok((
            A::from_eval_val(ctx, a, table, field)?,
            B::from_eval_val(ctx, b, table, field)?,
            C::from_eval_val(ctx, c, table, field)?,
            D::from_eval_val(ctx, d, table, field)?,
        ))
    }
}

impl<T: FromEvalVal> FromEvalVal for IntOrIntList<T> {
    fn from_eval_val(
        ctx: &EvalContext<'_, '_>,
        val: EvalVal,
        table: &'static str,
        field: &'static str,
    ) -> Result<Self, LuacError> {
        match val {
            EvalVal::Int(_) => Ok(Self::Int(T::from_eval_val(ctx, val, table, field)?)),
            EvalVal::Table(_) => Ok(Self::List(Box::<[T]>::from_eval_val(
                ctx, val, table, field,
            )?)),
            EvalVal::Nil
            | EvalVal::Bool(_)
            | EvalVal::Float(_)
            | EvalVal::Str(_)
            | EvalVal::OpenTable(_) => Err(DataCfgError::TypeMismatch {
                table,
                field,
                expected: "integer or integer list",
            }
            .into()),
        }
    }
}

fn load_root_proto_code(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    scratch: &mut DataCfgScratch,
) -> Result<(), LuacError> {
    let limits: LuacLimits = scratch.limits.luac();
    let _src_name = cur.read_string(is_xdi)?;
    let _meta = cur.take_bytes(11)?;

    let size_code = cur.read_u32()?;
    if size_code > limits.instructions_count_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::InstructionsCount,
            actual: u64::from(size_code),
            max: u64::from(limits.instructions_count_max()),
        });
    }
    for pc in 0..size_code {
        let raw_w = cur.read_u32()?;
        let inst = if is_xdi {
            LuaInstruction::from_xdi_word(raw_w).map_err(|raw_op| LuacError::InvalidXdiOpcode {
                proto_index: 0,
                pc,
                raw_op,
            })?
        } else {
            LuaInstruction::from_lua53_word(raw_w).map_err(|raw_op| {
                LuacError::InvalidLua53Opcode {
                    proto_index: 0,
                    pc,
                    raw_op,
                }
            })?
        };
        let push_res = scratch.code.try_push(inst);
        assert!(push_res.is_ok());
    }
    Ok(())
}

fn load_root_proto_into_scratch(
    cur: &mut WireCursor<'_>,
    is_xdi: bool,
    scratch: &mut DataCfgScratch,
) -> Result<(), LuacError> {
    load_root_proto_code(cur, is_xdi, scratch)?;
    let limits: LuacLimits = scratch.limits.luac();
    let size_k = cur.read_u32()?;
    if size_k > limits.constants_count_max() {
        return Err(LuacError::LimitExceeded {
            kind: LuacLimitKind::ConstantsCount,
            actual: u64::from(size_k),
            max: u64::from(limits.constants_count_max()),
        });
    }
    for const_index in 0..size_k {
        let tag_offset = cur.offset();
        let tag = cur.read_u8()?;
        let val = match tag {
            0x00 => EvalVal::Nil,
            0x01 => EvalVal::Bool(cur.read_u8()? != 0),
            0x03 => {
                let f = cur.read_f64()?;
                if !f.is_finite() {
                    return Err(LuacError::NonFiniteFloatConstant {
                        proto_index: 0,
                        const_index,
                        bits: f.to_bits(),
                    });
                }
                EvalVal::Float(f)
            }
            0x13 => EvalVal::Int(cur.read_i64()?),
            0x04 | 0x14 => {
                let span = cur
                    .read_string(is_xdi)?
                    .ok_or(LuacError::NullConstantString {
                        proto_index: 0,
                        const_index,
                        tag,
                    })?;
                EvalVal::Str(span)
            }
            other => {
                return Err(LuacError::InvalidConstantTag {
                    offset: tag_offset,
                    proto_index: 0,
                    const_index,
                    tag: other,
                });
            }
        };
        let push_res = scratch.consts.try_push(val);
        assert!(push_res.is_ok());
    }
    Ok(())
}

fn find_stop_pc(code: &[LuaInstruction], mode_readonly_reg: u8) -> Result<u32, LuacError> {
    let closure_idx = code
        .iter()
        .position(|inst| inst.opcode() == LuaOpcode::Closure)
        .ok_or(DataCfgError::MissingModeReadonlyBoundary)?;
    let prefix = &code[..closure_idx];
    let rel_idx = prefix
        .iter()
        .rposition(|inst| inst.opcode() == LuaOpcode::NewTable && inst.a() == mode_readonly_reg)
        .ok_or(DataCfgError::MissingModeReadonlyBoundary)?;
    if closure_idx - rel_idx > 2 {
        return Err(DataCfgError::MissingModeReadonlyBoundary.into());
    }
    Ok(bounded_len_u32(rel_idx))
}

fn resolve_setlist_batch(
    code: &[LuaInstruction],
    c: u16,
    pc: &mut u32,
    stop_pc: u32,
) -> Result<u32, LuacError> {
    if c != 0 {
        return Ok(u32::from(c));
    }
    let cur_pc = *pc;
    let next_pc = cur_pc + 1;
    if next_pc >= stop_pc {
        return Err(DataCfgError::MissingExtraArg { pc: cur_pc }.into());
    }
    let next_inst = code[u32_to_usize(next_pc)];
    if next_inst.opcode() != LuaOpcode::ExtraArg {
        return Err(DataCfgError::MissingExtraArg { pc: cur_pc }.into());
    }
    *pc = next_pc;
    Ok(next_inst.ax())
}

fn eval_setlist(
    scratch: &mut DataCfgScratch,
    regs: &mut [EvalVal; 256],
    inst: LuaInstruction,
    pc: &mut u32,
    stop_pc: u32,
) -> Result<(), LuacError> {
    let cur_pc = *pc;
    let a = inst.a();
    let b = inst.b();
    if b == 0 || u32::from(a) + u32::from(b) > 255 {
        return Err(DataCfgError::InvalidSetListCount { pc: cur_pc, a, b }.into());
    }
    let batch = resolve_setlist_batch(scratch.code.as_slice(), inst.c(), pc, stop_pc)?;
    let EvalVal::OpenTable(slot) = regs[u32_to_usize(u32::from(a))] else {
        return Err(DataCfgError::RegisterNotTable { pc: cur_pc, reg: a }.into());
    };
    let slot_idx = u32_to_usize(u32::from(slot));
    if !scratch.open_slots[slot_idx].map.is_empty() {
        return Err(DataCfgError::MixedTableKind {
            pc: cur_pc,
            table_id: u32::from(slot),
        }
        .into());
    }
    let actual_len = bounded_len_u32(scratch.open_slots[slot_idx].list.len());
    let expected_len = batch
        .checked_sub(1)
        .and_then(|z| z.checked_mul(LFIELDS_PER_FLUSH))
        .ok_or(DataCfgError::SetListBatchMismatch {
            pc: cur_pc,
            expected_len: u32::MAX,
            actual_len,
        })?;
    if actual_len != expected_len {
        return Err(DataCfgError::SetListBatchMismatch {
            pc: cur_pc,
            expected_len,
            actual_len,
        }
        .into());
    }
    let new_open_len = actual_len
        .checked_add(u32::from(b))
        .ok_or(LuacError::LimitExceeded {
            kind: LuacLimitKind::DataCfgListItems,
            actual: u64::from(actual_len) + u64::from(b),
            max: u64::from(scratch.closed_lists.cap()),
        })?;
    let _total_list_len = scratch
        .closed_lists
        .len()
        .checked_add(new_open_len)
        .filter(|&tot| tot <= scratch.closed_lists.cap())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::DataCfgListItems,
            actual: u64::from(scratch.closed_lists.len()) + u64::from(new_open_len),
            max: u64::from(scratch.closed_lists.cap()),
        })?;
    for off in 1..=b {
        let reg = ((u32::from(a) + u32::from(off)) & 0xFF) as u8;
        let item = scratch.materialize_reg(regs, reg)?;
        scratch.open_slots[slot_idx].list.push(item);
    }
    Ok(())
}

fn eval_settable(
    scratch: &mut DataCfgScratch,
    regs: &mut [EvalVal; 256],
    inst: LuaInstruction,
    cur_pc: u32,
) -> Result<(), LuacError> {
    let a = inst.a();
    let a_idx = u32_to_usize(u32::from(a));
    let key = scratch.resolve_rk(regs, inst.b(), cur_pc)?;
    let val = scratch.resolve_rk(regs, inst.c(), cur_pc)?;
    let EvalVal::OpenTable(slot) = regs[a_idx] else {
        return Err(DataCfgError::RegisterNotTable { pc: cur_pc, reg: a }.into());
    };
    let slot_idx = u32_to_usize(u32::from(slot));
    if !scratch.open_slots[slot_idx].list.is_empty() {
        return Err(DataCfgError::MixedTableKind {
            pc: cur_pc,
            table_id: u32::from(slot),
        }
        .into());
    }
    let cur_open_map_len = bounded_len_u32(scratch.open_slots[slot_idx].map.len());
    let new_open_map_len = cur_open_map_len
        .checked_add(1)
        .ok_or(LuacError::LimitExceeded {
            kind: LuacLimitKind::DataCfgMapEntries,
            actual: u64::from(cur_open_map_len) + 1,
            max: u64::from(scratch.closed_maps.cap()),
        })?;
    let _total_map_len = scratch
        .closed_maps
        .len()
        .checked_add(new_open_map_len)
        .filter(|&tot| tot <= scratch.closed_maps.cap())
        .ok_or_else(|| LuacError::LimitExceeded {
            kind: LuacLimitKind::DataCfgMapEntries,
            actual: u64::from(scratch.closed_maps.len()) + u64::from(new_open_map_len),
            max: u64::from(scratch.closed_maps.cap()),
        })?;
    scratch.open_slots[slot_idx].map.push((key, val));
    Ok(())
}

#[inline]
fn close_reg_if_open(
    scratch: &mut DataCfgScratch,
    regs: &[EvalVal; 256],
    a: u8,
) -> Result<(), LuacError> {
    if let EvalVal::OpenTable(old_slot) = regs[usize::from(a)] {
        let _closed_tid = scratch.close_open_table(old_slot)?;
    }
    Ok(())
}

fn eval_newtable(
    scratch: &mut DataCfgScratch,
    regs: &mut [EvalVal; 256],
    a: u8,
) -> Result<(), LuacError> {
    close_reg_if_open(scratch, regs, a)?;
    let slot = scratch.alloc_open_table()?;
    regs[usize::from(a)] = EvalVal::OpenTable(slot);
    Ok(())
}

fn eval_loadbool(
    scratch: &mut DataCfgScratch,
    regs: &mut [EvalVal; 256],
    inst: LuaInstruction,
    cur_pc: u32,
) -> Result<(), LuacError> {
    let a = inst.a();
    close_reg_if_open(scratch, regs, a)?;
    if inst.c() != 0 {
        return Err(DataCfgError::LoadBoolSkipNotAllowed {
            pc: cur_pc,
            c: inst.c(),
        }
        .into());
    }
    regs[usize::from(a)] = EvalVal::Bool(inst.b() != 0);
    Ok(())
}

fn eval_loadkx(
    scratch: &mut DataCfgScratch,
    regs: &mut [EvalVal; 256],
    a: u8,
    pc: &mut u32,
    stop_pc: u32,
) -> Result<(), LuacError> {
    close_reg_if_open(scratch, regs, a)?;
    let cur_pc = *pc;
    let next_pc = cur_pc + 1;
    if next_pc >= stop_pc {
        return Err(DataCfgError::MissingExtraArg { pc: cur_pc }.into());
    }
    let next_inst = scratch.code.as_slice()[u32_to_usize(next_pc)];
    if next_inst.opcode() != LuaOpcode::ExtraArg {
        return Err(DataCfgError::MissingExtraArg { pc: cur_pc }.into());
    }
    let idx = next_inst.ax();
    let val = scratch
        .consts
        .as_slice()
        .get(u32_to_usize(idx))
        .copied()
        .ok_or(DataCfgError::ConstantIndexOutOfBounds {
            pc: cur_pc,
            index: idx,
            count: scratch.consts.len(),
        })?;
    regs[usize::from(a)] = val;
    *pc = next_pc;
    Ok(())
}

fn eval_loadk(
    scratch: &mut DataCfgScratch,
    regs: &mut [EvalVal; 256],
    a: u8,
    idx: u32,
    cur_pc: u32,
) -> Result<(), LuacError> {
    close_reg_if_open(scratch, regs, a)?;
    let val = scratch
        .consts
        .as_slice()
        .get(u32_to_usize(idx))
        .copied()
        .ok_or(DataCfgError::ConstantIndexOutOfBounds {
            pc: cur_pc,
            index: idx,
            count: scratch.consts.len(),
        })?;
    regs[usize::from(a)] = val;
    Ok(())
}

fn step_datacfg_instruction(
    scratch: &mut DataCfgScratch,
    regs: &mut [EvalVal; 256],
    inst: LuaInstruction,
    pc: &mut u32,
    stop_pc: u32,
) -> Result<(), LuacError> {
    let cur_pc = *pc;
    let a = inst.a();
    match inst.opcode() {
        LuaOpcode::NewTable => eval_newtable(scratch, regs, a)?,
        LuaOpcode::LoadK => eval_loadk(scratch, regs, a, inst.bx(), cur_pc)?,
        LuaOpcode::LoadKx => eval_loadkx(scratch, regs, a, pc, stop_pc)?,
        LuaOpcode::LoadBool => eval_loadbool(scratch, regs, inst, cur_pc)?,
        LuaOpcode::SetList => eval_setlist(scratch, regs, inst, pc, stop_pc)?,
        LuaOpcode::SetTable => eval_settable(scratch, regs, inst, cur_pc)?,
        other @ (LuaOpcode::Move
        | LuaOpcode::LoadNil
        | LuaOpcode::GetUpval
        | LuaOpcode::GetTabUp
        | LuaOpcode::GetTable
        | LuaOpcode::SetTabUp
        | LuaOpcode::SetUpval
        | LuaOpcode::SelfOp
        | LuaOpcode::Add
        | LuaOpcode::Sub
        | LuaOpcode::Mul
        | LuaOpcode::Mod
        | LuaOpcode::Pow
        | LuaOpcode::Div
        | LuaOpcode::IDiv
        | LuaOpcode::BAnd
        | LuaOpcode::BOr
        | LuaOpcode::BXor
        | LuaOpcode::Shl
        | LuaOpcode::Shr
        | LuaOpcode::Unm
        | LuaOpcode::BNot
        | LuaOpcode::Not
        | LuaOpcode::Len
        | LuaOpcode::Concat
        | LuaOpcode::Jmp
        | LuaOpcode::Eq
        | LuaOpcode::Lt
        | LuaOpcode::Le
        | LuaOpcode::Test
        | LuaOpcode::TestSet
        | LuaOpcode::Call
        | LuaOpcode::TailCall
        | LuaOpcode::Return
        | LuaOpcode::ForLoop
        | LuaOpcode::ForPrep
        | LuaOpcode::TForCall
        | LuaOpcode::TForLoop
        | LuaOpcode::Closure
        | LuaOpcode::VarArg
        | LuaOpcode::ExtraArg) => {
            return Err(DataCfgError::UnsupportedInstruction {
                pc: cur_pc,
                opcode: other as u8,
            }
            .into());
        }
    }
    Ok(())
}

fn eval_datacfg_instructions(
    scratch: &mut DataCfgScratch,
    mode_readonly_reg: u8,
) -> Result<[EvalVal; 256], LuacError> {
    let stop_pc = find_stop_pc(scratch.code.as_slice(), mode_readonly_reg)?;
    let mut regs = [EvalVal::Nil; 256];
    let mut pc = 0_u32;

    while pc < stop_pc {
        let inst = scratch.code.as_slice()[u32_to_usize(pc)];
        step_datacfg_instruction(scratch, &mut regs, inst, &mut pc, stop_pc)?;
        pc += 1;
    }

    for reg in 0..=3_u8 {
        let _materialized = scratch.materialize_reg(&mut regs, reg)?;
    }
    Ok(regs)
}

fn apply_map_to_row<Row: DataCfgRow>(
    ctx: &EvalContext<'_, '_>,
    tid: u32,
    row: &mut Row,
) -> Result<(), LuacError> {
    let tbl = ctx.table(tid);
    for &(k, v) in ctx.map_slice(tbl, Row::TABLE_NAME)? {
        let EvalVal::Str(k_span) = k else {
            return Err(DataCfgError::TypeMismatch {
                table: Row::TABLE_NAME,
                field: "<row_field_key>",
                expected: "string column name",
            }
            .into());
        };
        row.set_field(ctx, ctx.str_bytes(k_span), v)?;
    }
    Ok(())
}

fn decode_language_keys(ctx: &EvalContext<'_, '_>, tid: u32) -> Result<Box<[Box<str>]>, LuacError> {
    let tbl = ctx.table(tid);
    let entries = ctx.map_slice(tbl, "language_key")?;
    let mut keys = Vec::with_capacity(entries.len());
    for &(k, _v) in entries {
        let EvalVal::Str(span) = k else {
            return Err(DataCfgError::TypeMismatch {
                table: "language_key",
                field: "key",
                expected: "string",
            }
            .into());
        };
        keys.push(Box::<str>::from(ctx.decode_str(span)?));
    }
    keys.sort();
    Ok(keys.into_boxed_slice())
}

/// Evaluates a single-level `LuaTable/DataCfg/config_*.out` buffer into `DataTable<K, Row>`.
pub(crate) fn parse_luatable_datacfg<K, Row>(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<DataTable<K, Row>, LuacError>
where
    K: FromEvalVal + Eq + Hash,
    Row: DataCfgRow,
{
    let _cipher = decrypt_luac_in_place(buf);
    scratch.reset();
    let mut cur = WireCursor::new(buf)?;
    let (is_xdi, _upvals) = read_chunk_header(&mut cur)?;
    load_root_proto_into_scratch(&mut cur, is_xdi, scratch)?;
    let regs = eval_datacfg_instructions(scratch, 4)?;

    let EvalVal::Table(r0_tid) = regs[0] else {
        return Err(DataCfgError::ExpectedTable { context: "R[0]" }.into());
    };
    let EvalVal::Table(r1_tid) = regs[1] else {
        return Err(DataCfgError::ExpectedTable { context: "R[1]" }.into());
    };
    let EvalVal::Table(r2_tid) = regs[2] else {
        return Err(DataCfgError::ExpectedTable { context: "R[2]" }.into());
    };
    let EvalVal::Table(r3_tid) = regs[3] else {
        return Err(DataCfgError::ExpectedTable { context: "R[3]" }.into());
    };

    let ctx = EvalContext { buf, scratch };
    let mut most_row = Row::default();
    apply_map_to_row(&ctx, r1_tid, &mut most_row)?;

    let mut default_row = most_row.clone();
    apply_map_to_row(&ctx, r2_tid, &mut default_row)?;

    let language_keys = decode_language_keys(&ctx, r3_tid)?;

    let r0_entries = ctx.map_slice(ctx.table(r0_tid), Row::TABLE_NAME)?;
    let mut rows = FxHashMap::with_capacity_and_hasher(r0_entries.len(), FxBuildHasher);
    for &(k_val, row_val) in r0_entries {
        let key = K::from_eval_val(&ctx, k_val, Row::TABLE_NAME, "<row_key>")?;
        let EvalVal::Table(row_tid) = row_val else {
            return Err(DataCfgError::ExpectedTable {
                context: Row::TABLE_NAME,
            }
            .into());
        };
        let mut row = most_row.clone();
        apply_map_to_row(&ctx, row_tid, &mut row)?;
        rows.insert(key, row);
    }

    Ok(DataTable {
        rows,
        most_row,
        default_row,
        language_keys,
    })
}

fn decode_3d_rows<K1, K2, K3, Row>(
    ctx: &EvalContext<'_, '_>,
    r0_tid: u32,
    most_row: &Row,
) -> Result<FxHashMap<(K1, K2, K3), Row>, LuacError>
where
    K1: FromEvalVal + Eq + Hash + Copy,
    K2: FromEvalVal + Eq + Hash + Copy,
    K3: FromEvalVal + Eq + Hash + Copy,
    Row: DataCfgRow,
{
    let l0_entries = ctx.map_slice(ctx.table(r0_tid), Row::TABLE_NAME)?;
    let mut rows = FxHashMap::with_capacity_and_hasher(l0_entries.len(), FxBuildHasher);
    for &(k1_val, l1_val) in l0_entries {
        let k1 = K1::from_eval_val(ctx, k1_val, Row::TABLE_NAME, "<key1>")?;
        let EvalVal::Table(l1_tid) = l1_val else {
            return Err(DataCfgError::ExpectedTable {
                context: Row::TABLE_NAME,
            }
            .into());
        };
        for &(k2_val, l2_val) in ctx.map_slice(ctx.table(l1_tid), Row::TABLE_NAME)? {
            let k2 = K2::from_eval_val(ctx, k2_val, Row::TABLE_NAME, "<key2>")?;
            let EvalVal::Table(l2_tid) = l2_val else {
                return Err(DataCfgError::ExpectedTable {
                    context: Row::TABLE_NAME,
                }
                .into());
            };
            for &(k3_val, row_val) in ctx.map_slice(ctx.table(l2_tid), Row::TABLE_NAME)? {
                let k3 = K3::from_eval_val(ctx, k3_val, Row::TABLE_NAME, "<key3>")?;
                let EvalVal::Table(row_tid) = row_val else {
                    return Err(DataCfgError::ExpectedTable {
                        context: Row::TABLE_NAME,
                    }
                    .into());
                };
                let mut row = most_row.clone();
                apply_map_to_row(ctx, row_tid, &mut row)?;
                rows.insert((k1, k2, k3), row);
            }
        }
    }
    Ok(rows)
}

/// Evaluates a 3-level `LuaTable/DataCfg` (`config_map_gameplay_season_doll_text_display.out`) into `DataTable<(K1, K2, K3), Row>`.
pub(crate) fn parse_luatable_datacfg_3d<K1, K2, K3, Row>(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
) -> Result<DataTable<(K1, K2, K3), Row>, LuacError>
where
    K1: FromEvalVal + Eq + Hash + Copy,
    K2: FromEvalVal + Eq + Hash + Copy,
    K3: FromEvalVal + Eq + Hash + Copy,
    Row: DataCfgRow,
{
    let _cipher = decrypt_luac_in_place(buf);
    scratch.reset();
    let mut cur = WireCursor::new(buf)?;
    let (is_xdi, _upvals) = read_chunk_header(&mut cur)?;
    load_root_proto_into_scratch(&mut cur, is_xdi, scratch)?;
    let regs = eval_datacfg_instructions(scratch, 4)?;

    let (
        EvalVal::Table(r0_tid),
        EvalVal::Table(r1_tid),
        EvalVal::Table(r2_tid),
        EvalVal::Table(r3_tid),
    ) = (regs[0], regs[1], regs[2], regs[3])
    else {
        return Err(DataCfgError::ExpectedTable {
            context: "R[0..=3]",
        }
        .into());
    };

    let ctx = EvalContext { buf, scratch };
    let mut most_row = Row::default();
    apply_map_to_row(&ctx, r1_tid, &mut most_row)?;
    let mut default_row = most_row.clone();
    apply_map_to_row(&ctx, r2_tid, &mut default_row)?;
    let language_keys = decode_language_keys(&ctx, r3_tid)?;
    let rows = decode_3d_rows(&ctx, r0_tid, &most_row)?;

    Ok(DataTable {
        rows,
        most_row,
        default_row,
        language_keys,
    })
}

/// Evaluates a 1-level `LuaPreLoadTable/DataCfg/config_preload_*.out` map buffer into `PreloadTable<K, V>`.
pub(crate) fn parse_preload_map<K, V>(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
    table_name: &'static str,
) -> Result<PreloadTable<K, V>, LuacError>
where
    K: FromEvalVal + Eq + Hash,
    V: FromEvalVal,
{
    let _cipher = decrypt_luac_in_place(buf);
    scratch.reset();
    let mut cur = WireCursor::new(buf)?;
    let (is_xdi, _upvals) = read_chunk_header(&mut cur)?;
    load_root_proto_into_scratch(&mut cur, is_xdi, scratch)?;
    let regs = eval_datacfg_instructions(scratch, 1)?;

    let EvalVal::Table(r0_tid) = regs[0] else {
        return Err(DataCfgError::ExpectedTable { context: "R[0]" }.into());
    };
    let ctx = EvalContext { buf, scratch };
    let raw_entries = ctx.map_slice(ctx.table(r0_tid), table_name)?;
    let mut entries = FxHashMap::with_capacity_and_hasher(raw_entries.len(), FxBuildHasher);
    for &(k_val, v_val) in raw_entries {
        let key = K::from_eval_val(&ctx, k_val, table_name, "<key>")?;
        let val = V::from_eval_val(&ctx, v_val, table_name, "<value>")?;
        entries.insert(key, val);
    }
    Ok(PreloadTable { entries })
}

/// Evaluates a 2-level `LuaPreLoadTable/DataCfg/config_preload_*.out` map buffer into `PreloadTable<(K1, K2), V>`.
pub(crate) fn parse_preload_map_2d<K1, K2, V>(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
    table_name: &'static str,
) -> Result<PreloadTable<(K1, K2), V>, LuacError>
where
    K1: FromEvalVal + Eq + Hash + Copy,
    K2: FromEvalVal + Eq + Hash + Copy,
    V: FromEvalVal,
{
    let _cipher = decrypt_luac_in_place(buf);
    scratch.reset();
    let mut cur = WireCursor::new(buf)?;
    let (is_xdi, _upvals) = read_chunk_header(&mut cur)?;
    load_root_proto_into_scratch(&mut cur, is_xdi, scratch)?;
    let regs = eval_datacfg_instructions(scratch, 1)?;

    let EvalVal::Table(r0_tid) = regs[0] else {
        return Err(DataCfgError::ExpectedTable { context: "R[0]" }.into());
    };
    let ctx = EvalContext { buf, scratch };
    let l0_entries = ctx.map_slice(ctx.table(r0_tid), table_name)?;
    let mut entries = FxHashMap::with_capacity_and_hasher(l0_entries.len(), FxBuildHasher);
    for &(k1_val, l1_val) in l0_entries {
        let k1 = K1::from_eval_val(&ctx, k1_val, table_name, "<key1>")?;
        let EvalVal::Table(l1_tid) = l1_val else {
            return Err(DataCfgError::ExpectedTable {
                context: table_name,
            }
            .into());
        };
        for &(k2_val, v_val) in ctx.map_slice(ctx.table(l1_tid), table_name)? {
            let k2 = K2::from_eval_val(&ctx, k2_val, table_name, "<key2>")?;
            let val = V::from_eval_val(&ctx, v_val, table_name, "<value>")?;
            entries.insert((k1, k2), val);
        }
    }
    Ok(PreloadTable { entries })
}

/// Evaluates a list-based `LuaPreLoadTable/DataCfg/config_preload_*.out` buffer into `Box<[T]>`.
pub(crate) fn parse_preload_list<T>(
    buf: &mut [u8],
    scratch: &mut DataCfgScratch,
    table_name: &'static str,
) -> Result<Box<[T]>, LuacError>
where
    T: FromEvalVal,
{
    let _cipher = decrypt_luac_in_place(buf);
    scratch.reset();
    let mut cur = WireCursor::new(buf)?;
    let (is_xdi, _upvals) = read_chunk_header(&mut cur)?;
    load_root_proto_into_scratch(&mut cur, is_xdi, scratch)?;
    let regs = eval_datacfg_instructions(scratch, 1)?;

    let ctx = EvalContext { buf, scratch };
    Box::<[T]>::from_eval_val(&ctx, regs[0], table_name, "<root_list>")
}

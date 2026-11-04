---
name: tigerstyle-rust
description: >-
  Write, refactor, and review Rust code following TigerStyle adapted natively for
  strict, cross-platform systems Rust (Safety > Performance > Developer Experience).
  Use this skill whenever writing or reviewing Rust systems, storage, networking,
  compute, or high-reliability code where TigerStyle is requested.
---

# TigerStyle for Strict Systems Rust

Apply TigerBeetle's engineering discipline (**Safety > Performance > Developer Experience**) natively to cross-platform Rust (**Linux, macOS, Windows** on `x86_64` and `aarch64`).

**Core Mandate:** Always operate in **Strict Systems Mode** (upfront/bounded memory, zero recursion, production assertions, deterministic layouts, batching) while writing **100% idiomatic Rust**. Never write C-in-Rust or Zig-in-Rust: leverage fat slices, move semantics, algebraic types, niche optimizations, and compile-time proofs instead of pointer chasing, `(_offset, _len)` plumbing, or gratuitous `unsafe`.

---

## 1. Anti-C & Anti-Pointer-Chasing Guardrails

Strict static bounds in Rust must **never** degenerate into C idioms:

1. **Fat Slices & Checked Sub-Slicing — Never `(_offset, _len, _cap)` Arguments**
   * Never pass sibling offset, length, or capacity integers alongside slices. Narrow windows at the call site so callees receive the exact `&[T]`, `&mut [T]`, or `&[T; N]` they own:
     * At **external or untrusted boundaries**, narrow fallibly with `.get(start..end)`, `.split_at_checked(mid)`, or `.split_at_mut_checked(mid)` mapped to `Err(...)` (never panicking `&buf[start..end]` or `split_at_mut`).
     * On **pre-validated internal tables, SoA columns, `[T; 256]` LUTs, ring buffers, and SIMD lane arrays**, direct `[idx]` indexing is idiomatic—in Rust, `slice[idx]` acts as a deterministic fail-fast `assert!`.
   * Advance wire/disk parsers by **shrinking the slice binding** with `zerocopy`'s `ref_from_prefix` (requiring `Unaligned`, which pairs naturally with `zerocopy::byteorder::little_endian::{U16, U32, U64, U128}` fields; use `try_ref_from_prefix` for checked `bool`/wire `enum`s, or `read_from_prefix` when copying by value into host-aligned structs):
     ```rust
     pub fn take<'a, T>(buf: &mut &'a [u8]) -> Result<&'a T, DecodeError>
     where
         T: zerocopy::FromBytes
             + zerocopy::KnownLayout
             + zerocopy::Immutable
             + zerocopy::Unaligned,
     {
         let (item, rest) =
             T::ref_from_prefix(buf).map_err(|_| DecodeError::InvalidLayout)?;
         *buf = rest;
         Ok(item)
     }
     ```

2. **Startup Memory & Encapsulated Fixed-Capacity Containers — Never Sibling `(items, items_len)` Fields**
   * Ban raw array + manual length sibling fields inside domain structs.
   * **Fixed length at startup** (I/O page blocks, ring-buffer backing arrays, dense `0..N` ID-indexed tables, `[T; 256]` LUTs): use **`Box<[T]>`** (`vec![init; len].into_boxed_slice()`). It is 16 bytes and statically incapable of `push`, `resize`, or `realloc`.
   * **Variable count `0..=CAP` (small/inline)**: use **`arrayvec::{ArrayVec, ArrayString}`**.
   * **Variable count `0..=CAP` (large startup heap pool)**: encapsulate `Vec<T>` with a **private explicit `cap: u32` field** (never rely on `Vec::capacity()` for logical bounds, because `std::alloc` permits over-allocation and ZST capacity is `usize::MAX`):
     ```rust
     pub struct FixedVec<T> {
         inner: Vec<T>,
         cap: u32,
     }

     impl<T> FixedVec<T> {
         #[must_use]
         pub fn with_capacity(cap: u32) -> Self {
             Self {
                 inner: Vec::with_capacity(u32_to_usize(cap)),
                 cap,
             }
         }

         pub fn try_push(&mut self, item: T) -> Result<(), T> {
             if self.inner.len() == u32_to_usize(self.cap) {
                 return Err(item);
             }
             let raw_cap_before = self.inner.capacity();
             self.inner.push(item);
             assert!(self.inner.capacity() == raw_cap_before);
             Ok(())
         }

         #[must_use]
         #[expect(clippy::cast_possible_truncation, reason = "Invariant: inner.len() <= cap (u32)")]
         pub fn len(&self) -> u32 {
             self.inner.len() as u32
         }

         #[must_use]
         pub fn is_empty(&self) -> bool {
             self.inner.is_empty()
         }

         pub fn clear(&mut self) {
             self.inner.clear();
         }

         #[must_use]
         pub fn as_slice(&self) -> &[T] { &self.inner }

         #[must_use]
         pub fn as_mut_slice(&mut self) -> &mut [T] { &mut self.inner }
     }
     ```
   * **Key-value hash maps & sets (`K -> V`) — Never Hand-Roll Linear Probing:**
     * Rust's `std::collections::HashMap` is `hashbrown`'s SwissTable (a single flat allocation with a dense 1-byte `ctrl` fingerprint prefix probed 16 slots at a time via SIMD, where failed probes never touch `(K, V)` cache lines).
     * For all self-contained keys (`u32`, `i32`, composite structs/tuples `(i32, i32, i32)`, and `Box<str>` queried zero-alloc via `&str` through `Borrow<str>`), use **`rustc_hash::{FxHashMap, FxHashSet, FxBuildHasher}`** directly—pre-sized upfront via `FxHashMap::with_capacity_and_hasher(u32_to_usize(cap), FxBuildHasher)` and bounded by an explicit `_MAX` entry limit when populated from external input (use `std`'s seeded `RandomState` only if keys come from unverified attacker-controlled network input).
     * Reserve **`hashbrown::HashTable<u32>`** strictly for index tables where slots store compact `u32` IDs or spans (`RowId(u32)`, `SymbolId(u32)`, `Span`) whose equality/hashing requires passing external context (`&[u8]` string arena, `&[Key]` slice) at lookup time that Rust's `Eq`/`Borrow` traits cannot accept. Never hand-roll scalar linear-probing loops over `Box<[Option<Slot>]>` or fall back to `O(log N)` `binary_search` on `Box<[K]>` + `Box<[V]>` for key-value lookups.

3. **Slice Iterators & Chunks — Never C-Style `for i in 0..len { arr[i] }`**
   * Iterate over pre-bounded slices via `.iter().zip(...)`, `.chunks_exact()`, or `.as_chunks::<N>()`. This hoists bounds checks, eliminates off-by-one indexing bugs, and unlocks LLVM autovectorization.
   * **Preserve slice totality & tail completeness:** `assert_eq!(a.len(), b.len())` before `.zip()`ing parallel SoA columns (since `.zip()` silently truncates mismatched lengths), always process or `assert!(rem.is_empty())` on both `.as_chunks::<N>()` and `.chunks_exact(N)` / `.chunks_exact_mut(N)` remainders, and keep batch/slice kernels total over empty slices (`&[]` = 0 work; never `assert!(!slice.is_empty())` unless non-emptiness is encoded in a wrapper type).

4. **Flat Contiguous Memory & Access-Temperature Layout — Zero Pointer/Index Chasing & Zero "Shred-and-Reassemble" SoA**
   * Ban `Box<Node>`, `Arc<Mutex<T>>` graphs, `Vec<Box<dyn Trait>>` vtable indirection, and **arena index-chasing graphs** (`slab[node.next_id]`), which all thrash CPU caches. Store records inline by value in contiguous slices (`Box<[T]>`, `FixedVec<T>`) ordered for linear hardware prefetching and use `enum`s for static dispatch. Reserve `SlotId(u32)` handles strictly for O(1) direct table lookups (e.g., active I/O or connection slots).
   * **Ban "Shred-and-Reassemble" on point-lookup tables:** If fields `A`, `B`, and `C` are always read or written at the exact same index `i` at the exact same time (e.g., inside `get(id)` / `meta(idx)`), they share the same access temperature and **must live in one contiguous `Box<[Row]>`**. Never split a record into `K` separate `Box<[Field]>` arrays if no loop in the crate scans those arrays independently and the accessor merely indexes `self.a[i], self.b[i], self.c[i]` to reconstruct a struct. Only split co-accessed fields by alignment tier (`u64` vs `u8`/`u16`/`u32`) if `size_of::<Row>()` wastes `>= 4` bytes of alignment padding per row even after optimal field ordering.
   * **Group searched tables by access temperature (Hot `Key` vs Cold `Payload`), not one array per primitive field:**
     * For standard `K -> Row` key-value tables with self-contained keys, use **`FxHashMap<K, Row>` directly** (SwissTable already separates its hot 1-byte `ctrl` SIMD metadata array from `(K, Row)` slots in a single flat allocation, and keeping `(K, Row)` colocated avoids a second cache-line fetch on hits; do **not** split self-contained `K -> Row` maps into `HashTable<u32>` + `Box<[K]>` + `Box<[Row]>` or sorted `Box<[K]>` + `binary_search`).
     * When a `0..count` positional table is scanned/binary-searched over contiguous ranges (`&keys[start..end]`) or indexed by an external-context `hashbrown::HashTable<u32>` (e.g., arena-backed symbol/catalog tables that are also indexed by `u32` ID), split into at most **two** parallel arrays based on *when* fields are touched:
       * `keys: Box<[Key]>` (**Hot** — touched on every probe/comparison): Pack all fields compared together during a lookup (such as `(parent_id, name_span, kind)`) into a single `Key` struct so verifying a candidate `idx` touches 1 cache line and 1 bounds check, not 3. Always verify full `Key` equality after any hash or SIMD fingerprint match.
       * `payloads: Box<[Payload]>` (**Cold** — touched only once after a match succeeds): Store result data (plus variable-length side slabs) in a parallel array so failed probes/scans never pull payload bytes into L1 cache.
   * **Never split fields beyond `Row` or `Key` + `Payload` into standalone `Box<[Field]>` columns without a selective scan or SIMD lane load:** A field may be split out of `Row`, `Key`, or `Payload` into its own standalone `Box<[Field]>` column *only* when:
     1. **Selective scan/search:** At least one function iterates or binary-searches `&col[start..end]` and skips sibling columns on non-matching elements, **or**
     2. **Contiguous SIMD lane loads:** A vectorized kernel iterates `col.as_chunks::<N>()` to load `N` contiguous scalar lanes into SIMD registers where an AoS struct (`Vec2 { x, y }`) would interleave fields across lanes.
   * **Top-down slice narrowing vs bottom-up inline parent links — never standalone `parent_ids: Box<[ParentId]>` columns:** When parent records partition child records into contiguous `start..end` ranges, top-down lookups (`(parent_id, child_key) -> child`) must narrow directly to `&children[start..end]` rather than probing a global table and checking `parents[idx] == parent_id`. If bottom-up traversal (`child_id -> parent_id`, such as path reconstruction from a bare ID) is also required, store `parent: ParentId` **inline inside the child's `Row` / `Payload` / `Key` struct**—never as a standalone `Box<[ParentId]>` column.

5. **Value Returns, Niche Types, & Explicit Fallible Teardown**
   * Never port Zig's `fn init(out: *Self)` into `&mut MaybeUninit<T>` or raw out-pointers. Return values directly (`T` or `Result<T, E>`), relying on Rust move semantics / RVO. Pass `&mut Scratch` strictly to reuse pre-allocated backing capacity across hot calls—never expose hidden-allocation convenience wrappers that call `Scratch::new()` internally.
   * Ban sentinel integers (`u32::MAX`, `-1`, `0` = null), sibling `is_valid: bool` flags, `(kind: Tag, ...)` "tag-plus-bag" structs with dummy unused fields, and reserved fallback/dummy table slots (`SlotId(0) = FALLBACK`). Keep 0-based table/SoA indices strictly bijective with `0..len` (`SlotId(u32)`, using `Option<SlotId>` or an outer `enum` for absent/fallback states), and use niche-optimized types (`Option<NonZeroU32>` for naturally `>= 1` counts, byte sizes, generations, and external IDs; `Option<&T>`) plus data-carrying `enum` variants so invalid states are unrepresentable.
   * **Never let `Drop` silently swallow teardown I/O errors:** Use `Drop` (RAII) for infallible cleanup (memory, locks, error-path fd reclamation), **and expose a consuming `pub fn close(self) -> Result<(), Error>` (calling `flush` / `sync_all`)** on any resource with fallible or durable teardown (writable files, `BufWriter`, WAL segments, transactions). For durable file replacement, write (flushing any `BufWriter`), `sync_all`, and close (or drop) the sibling temp file before atomic `rename`—and never write implicit cache or test artifacts into the current working directory.

---

## 2. Safety: Types, Bounds, Assertions, & Errors

1. **Explicitly Sized Types (`u32`, `u64`, `u128`) & Semantic Cast Discipline**
   * **Ban `usize` in structs, wire/disk formats, state machines, and domain APIs.** Allow `usize` strictly at local slice-indexing boundaries.
   * Use `T::from(...)` for lossless widening (`cast_lossless = "deny"`) and `TryFrom -> Result` (never `.unwrap_or(0)`) for fallible range narrowing (where out-of-range values are errors).
   * Define one canonical crate-root `u32_to_usize` bridge alongside a static pointer-width check. **`u32_to_usize` is the *only* permitted `as usize` cast** (`u64 -> usize` must always use `usize::try_from(x)` because `cast_possible_truncation` only flags `u64 as usize` on 32-bit targets):
     ```rust
     const _: () = assert!(usize::BITS >= 32);

     #[inline(always)]
     #[must_use]
     pub const fn u32_to_usize(x: u32) -> usize {
         x as usize
     }
     ```
   * Fieldless `#[repr(u8)]` / `#[repr(u16)]` / `#[repr(u32)]` `enum` discriminants may use `op as u8` directly.
   * For intentional bitfield or word-half extraction (`u64 -> u32`, `u32 -> u16`, `u32 -> u8`), apply an inline constant mask or right-shift to the target width in the cast expression (`((bits >> 6) & 0xFFFF) as u16`, `(x >> 32) as u32`, `(x & 0xFFFF_FFFF) as u32`) so Clippy statically proves the `as` cast in-range without any `#[expect]`—never write unreachable `u16::try_from(x & 0xFFFF)` error branches or `.to_le_bytes()` array-slicing hacks.
   * Rust's scalar `float as int` (`x as i32`, `x as u8`) is guaranteed saturating and maps `NaN -> 0` across all targets since Rust 1.45. When an intentional scalar float-to-int or int-to-float conversion is genuinely required, isolate it in a small named helper with a narrow `#[expect(clippy::cast_possible_truncation, reason = "...")]` (plus `clippy::cast_sign_loss` for unsigned targets like `u8`/`u32`, or `clippy::cast_precision_loss` for `u32 -> f32`)—never dodge semantic cast lints with `wide::*::splat` or bit-twiddling hacks.

2. **`#[repr(transparent)]` Newtypes, Private Invariants, & Big-Endian Naming**
   * Wrap distinct domain primitives (`AccountId(u128)`, `SlotId(u32)`, `ByteCount(u32)`) so mixing indices, counts, and byte sizes fails to compile.
   * **Forbid `pub` fields on any newtype or struct that carries an invariant** (non-zero, bounded range, float finiteness, alignment, cross-field consistency). Keep fields private and expose validated constructors (`try_new(...) -> Result<Self, Error>`) + read-only accessors. Constrained float constructors must explicitly check `x.is_finite()` alongside bounds (because `NaN` evaluates `x < MIN || x > MAX` to `false`, and half-open range `.contains(&x)` admits `inf`). Never derive or implement `Default` on any type whose zero/empty state violates its invariants or bypasses `try_new` / `with_capacity`. `pub` fields are allowed *only* on unconstrained Plain-Old-Data records where every combination of values is domain-valid.
   * Use big-endian suffixes for units and limits so related identifiers align visually: `read_timeout_ms_max`, `frame_bytes_max`, `batch_count_max`.

3. **Explicit Limits, Zero Recursion, & Non-Silent Loop Exhaustion**
   * Every loop, queue, buffer, and batch must have an explicit `_MAX` upper bound.
   * **Ban recursion** (direct or mutual); use an explicit bounded work stack (`ArrayVec` or `FixedVec`).
   * **Bounded loops must never silently fall off the end:** If a search, probe, retry, or convergence loop (`for _ in 0..STEPS_MAX`) reaches `STEPS_MAX` without satisfying its termination condition, it must explicitly return `Err(Error::StepLimitExceeded)` or fail an `assert!`—never continue silently with partial or unconverged state.

4. **Totality at External Boundaries vs Production `assert!` on Internal Invariants**
   * **Public/external boundaries MUST be total:** Functions accepting untrusted `&[u8]`, `&str`, or external numbers must validate bounds and use **fallible `.checked_add` / `.checked_sub` / `.checked_mul` / `.checked_div` mapped to `Err(...)`**—never `assert!`, `.saturating_*`, or bare `+ - * / %` on unvalidated external input (which turns malformed payloads into a `panic = "abort"` DoS or silent corruption). Prefer constructing a private-field validated wrapper (`BoundedBatch<'a, T>`) at the boundary so internal kernels receive pre-validated inputs by type.
   * **Keep `assert!` active in release** (`[profile.release] panic = "abort"`), never downgraded to `debug_assert!`, for internal contracts and state invariants.
   * **Split compound assertions:** Write `assert!(a); assert!(b);` instead of `assert!(a && b);`.
   * **Pair assertions across boundaries:** Assert invariants right before encoding/writing and immediately after reading/decoding.
   * **Ban tautology assertions ("assertion theater"):** Never write post-guard tautologies (`if x > MAX { return Err(...); } assert!(x <= MAX);`), post-assignment tautologies (`let x = 5; assert!(x == 5);`), `std`/fixed-type tautologies (`v.push(x); assert!(!v.is_empty());`, `[T; N].len() == N`, `x & MASK <= MASK`, `x.min(N) <= N`), loop-condition echoes (`while cond { assert!(cond); }`), or single-constant literal echoes (`const MAX: u32 = 16; const _: () = assert!(MAX == 16);`—as opposed to `size_of::<T>()` or cross-constant relationship checks). Every `assert!` must verify a **non-trivial invariant across independent state, conservation laws, monotonicity/ordering, or cross-representation round-trips**. Trivial accessors and pure type-checked mappers have no artificial assertion-count quota—never pad functions with unrelated assertions.

5. **Zero Silent Fallbacks & Strict Error Discipline**
   * **Ban `.unwrap()`, `.expect()`, and `panic!`** in production code (`unwrap_used`, `expect_used`, and `panic` are denied; `assert!` and `assert_eq!` remain allowed for internal invariants).
   * **Equally ban silent fallbacks and error-discarding control flow:**
     * Never use `unwrap_or`, `unwrap_or_default`, `unwrap_or_else`, `map_or`, `map_or_else`, `.ok()`, or `let _ = ...` to swallow fallible conversions, parses, lookups, or mutations (mechanically denied via `disallowed-methods` and `let_underscore_must_use`).
     * Never discard errors via `if let Ok(...)` (without handling `Err`) or `let Ok(...) = ... else { continue | break | return Ok(...) }`.
     * Never disguise a silent fallback as an explicit `match` / `if-else` returning dummy defaults (`0`, `""`, `&[]`, `Ok(Vec::new())`) on `Err`/`None`, clamp or bitmask-wrap invalid indices (`idx.min(len - 1)`, `slice[idx & mask]` outside power-of-two ring/hash tables), decode wire text via `from_utf8_lossy` / `from_utf16_lossy`, or add catch-all `Unknown(u8)` variants to domain enums to dodge boundary validation.
     * Every `Err` must propagate with `?`, record an explicit per-item failure into a batch output slot (`out[i] = Outcome::Corrupt`), or exhaustively `match` specific expected error variants (**no wildcard `_ =>` arms** on internal enums).
   * **Structured error hierarchy & causal chains:**
     * Define concrete error `enum`s with **`thiserror`** across all library, domain, and state-machine APIs (preserving `#[source]` / `#[from]` causal chains).
     * Keep internal and intra-workspace error enums exhaustive so new variants force compile-time handling across sibling workspace crates; apply `#[non_exhaustive]` *only* on externally published SemVer library crate boundaries, and wrap third-party upstream errors rather than leaking foreign error types via `#[error(transparent)]`.
     * Explicitly ban `Result<T, String>` and `From<Error> for String` (destroys causal chains), and never duplicate `{0}` in `#[error("...")]` on a variant carrying `#[source]` / `#[from]` (prevents duplicate printing when walking `.source()`).
     * Reserve `anyhow` strictly for binary/CLI crates (`src/main.rs` and CLI command drivers), attaching `.context(...)` / `.with_context(...)` (with file paths and attempted action) at every I/O boundary.
   * Place `#[must_use]` on **state-machine return types (`#[must_use] pub enum Action`, `#[must_use] pub enum Outcome`), pure constructors, and guard/ticket types** so emitted commands cannot be ignored (do not place bare `#[must_use]` on functions already returning `Result` or `Option`, which trips `clippy::double_must_use`).
   * Never wrap violated internal invariants (programmer bugs) in `Err(...)`—`assert!` and abort immediately. If `std::sync::Mutex` is used, define one crate-level `lock_unpoisoned` helper carrying `#[expect(clippy::expect_used, reason = "Lock poisoning is an unrecoverable invariant failure; release aborts on panic")]`; `PoisonError::into_inner()` is banned.

6. **Overflow & Binary Layout Safety**
   * Enable `overflow-checks = true` in `[profile.release]` and `[profile.bench]` and pick arithmetic operators/methods strictly by boundary:
     1. **Untrusted wire/file/caller boundary (`-> Result`):** Use `.checked_add` / `.checked_sub` / `.checked_mul` / `.checked_div` mapped to `Err(...)` when validating raw external offsets, lengths, or counts—never bare `+ - * / %` or `.saturating_*` on unvalidated external numbers.
     2. **Everything else (pre-validated internal indices, offsets, counts, loops):** Use normal `+`, `-`, `*` directly (backed by `overflow-checks = true` across all profiles)—never use `.saturating_*` on internal indices, offsets, or counters, and never use `.strict_*` (rewrite any existing `.strict_add` / `.strict_sub` / `.strict_mul` into plain `+`, `-`, `*`). *(Prefer passing `NonZeroU32` or returning `Result` via `NonZeroU32::new(x).ok_or(...)?`; when constructing `NonZeroU32` from a pre-validated `x > 0` inside an infallible `const fn`, `NonZeroU32::MIN.saturating_add(x - 1)` is an accepted trap-on-zero idiom.)*
     3. **Hashes, ciphers, and genuine clamping specs:** Use `.wrapping_*` for intentional modular hash/cipher math, and `.saturating_*` *only* when clamping at `0` or `MAX` is the genuine domain spec (such as pixel channels, audio samples, or heuristic initial capacity hints clamped to `_MAX`).
   * Wire/disk structs must use `#[repr(C)]` + `zerocopy` (`FromBytes` / `TryFromBytes`, `IntoBytes`, `Immutable`, `KnownLayout`, `Unaligned`, `little_endian::*`) + `const _: () = assert!(size_of::<Header>() == EXPECTED);` so any implicit padding fails compilation. Raw `core::mem::transmute` and `core::mem::forget` are banned.

---

## 3. Performance, SIMD, & Cross-Platform I/O

1. **Batching, Cache Alignment, & Pure State Machines**
   * Do napkin math upfront. Design hot APIs over slices (`fn apply_batch(&mut self, batch: BoundedBatch<'_, Item>, out: &mut [Outcome])`) rather than single items to amortize dispatch, syscalls, and cache misses.
   * Align hot contended or block-sized structs explicitly (`#[repr(C, align(64))]`) and verify size at compile time.
   * Keep core domain logic synchronous, allocation-free after startup, and decoupled from OS I/O (`fn step(&mut self, event: Event) -> Action`).

2. **Portable SIMD (`wide` + `as_chunks`)**
   * **Step 1 (Vertical maps & integers):** Branchless loops over `slice.as_chunks::<N>()` + `zip` autovectorize cleanly for integers and *lane-independent* vertical float maps (`out[i] = a[i] * scale + b[i]`).
   * **Step 2 (Explicit SIMD & float reductions):** IEEE-754 non-associativity prevents LLVM from autovectorizing horizontal float reductions (`sum`, dot products, norms). Accumulate float reductions explicitly into **`wide`** vector registers (`wide::f32x8::ZERO` + `.reduce_add()`), and use `wide` for explicit vector kernels:
     ```rust
     let (chunks, remainder) = values.as_chunks::<8>();
     let mut acc = wide::f32x8::ZERO;
     for chunk in chunks {
         acc += wide::f32x8::from(*chunk);
     }
     let mut total = acc.reduce_add();
     // MANDATORY: never drop `remainder` silently!
     for &val in remainder {
         total += val;
     }
     ```
   * **Anti-theater & float-to-int safety:**
     * Never wrap single scalars in `wide::*::splat(x)` to dodge scalar cast lints.
     * Always use portable saturating **`trunc_int()`** or **`round_int()`** (`NaN -> 0`, saturating integer bounds). **`fast_trunc_int()` and `fast_round_int()` are banned** across all float vector widths (on x86, `cvttps2dq` returns `i32::MIN` on `NaN` or positive overflow while ARM64 NEON saturates, and naive `i32::MAX as f32` clamping rounds up to `2147483648.0` and still overflows).
   * Document `target-cpu=x86-64-v3` (AVX2 + FMA) or `native` in build configs for `x86_64` release builds (`aarch64` enables NEON at baseline).
   * **Precision contract:** `wide` arithmetic (`+`, `-`, `*`, `/`) is IEEE-754 exact per lane; `wide` transcendentals (`sin`, `cos`, `exp`, `ln`, `atan2`) are fast SIMD polynomial approximations—assert transcendentals with an explicit epsilon, never `==`.

3. **Cross-Platform Completion I/O (`compio-driver` + `compio-buf`)**
   * Use `std::fs` / `std::net` for simple blocking tools. When non-blocking or high-throughput batched disk/network I/O is required across **Linux (`io_uring`), Windows (`IOCP`), and macOS (`kqueue`)**, use **`compio-driver` (`Proactor`) + `compio-buf`**.
   * Drive `Proactor::push` and `Proactor::poll` explicitly without pulling in an async runtime unless strictly isolated at an outer shell.
   * **Mandatory `Proactor` guardrails:**
     1. Explicitly bound `ProactorBuilder` at startup: set both `.capacity(IO_ENTRIES_MAX)` and `.thread_pool_limit(FALLBACK_THREADS_MAX)`.
     2. Pre-allocate a fixed pool of I/O buffers at startup, move a buffer into each submitted op (`IoBuf`/`IoBufMut`), and return it to the pool via `BufResult(res, buf)` on completion—**never allocate a fresh `Vec<u8>` per I/O submission**.

---

## 4. Developer Experience & Code Shape

* **Hard 70-line function limit:** Keep functions under 70 lines with an inverse-hourglass shape (few parameters, simple return type, clear linear control flow). Never pack parameters or coordinates into anonymous multi-field tuples or raw `[u32; 4]` arrays to dodge argument limits; group cohesive fields into a named struct (`Rect`, `WriteOp`). *(Sole exception to 70 lines: a single flat, exhaustive `match` dispatch over a large enum may carry `#[expect(clippy::too_many_lines, reason = "...")]` rather than splitting into `_ =>` wildcard helpers.)*
* **"Push `if`s up and `for`s down":** Centralize branching and validation in parent orchestrators; push pure, branch-free batch loops down into leaf functions taking pre-validated slices.
* **Minimal variable scopes & no deceptive shadowing:** Bind variables in the smallest possible block expression (`let cfg = { ... };`). Do not shadow variable names across type or unit transformations.
* **Prose "why" comments:** Write full-sentence comments (capitalized, proper punctuation) explaining *why* an invariant, bound, or layout exists—never restating *what* the syntax does.
* **Zero technical debt & boundary testing:** No `todo!()`, `unimplemented!()`, or placeholder hacks. Test with built-in `#[test]` covering positive space, negative space (`#[should_panic]` for violated internal preconditions, `Err(...)` for external boundaries), and exact boundary transitions (`0`, `MAX - 1`, `MAX`, `MAX + 1`). Tests and benchmarks must never silently return early on `Err` or missing fixtures (`let Ok(x) = setup() else { return; };` is banned; use `#[ignore = "reason"]` for environment-dependent tests).

---

## 5. Locked "One Crate Per Job" Toolkit

Every core systems capability in the table below has **strictly one** approved crate. Domain-specific codecs or CLI parsers outside this table must likewise pick one canonical crate per workspace, reject unknown inputs/flags, and be declared only in the member crates that directly import them. Pull a situational crate in **only** when the project exercises that boundary:

| Capability | Canonical Crate | Tier & Rules |
| :--- | :--- | :--- |
| **Zero-Copy Layout & Padding Proofs** | `zerocopy` | Core (`features = ["derive"]`; `Unaligned`, `TryFromBytes`, `little_endian::*`) |
| **Concrete Error Enums & `source()` Chains** | `thiserror` | Core (all library/domain/subsystem error enums) |
| **Inline Fixed-Capacity Buffers/Strings** | `arrayvec` | Core (0 deps; pair with `Box<[T]>` / `FixedVec<T>` on heap) |
| **Typed Fixed-Width Bitfields** | `bitflags` | Core (0 deps, 0 proc-macros, `#[repr(transparent)]`) |
| **O(1) SwissTable Hash Maps/Sets & Arena Index Tables** | `rustc-hash` (+ `hashbrown`) | Core `FxHashMap` / `FxHashSet` / `FxBuildHasher` for `K -> V` lookups (0 deps, 0 proc-macros); situational `hashbrown::HashTable<u32>` strictly when key `Eq`/`Hash` requires external arena/slice context |
| **Portable Cross-Platform SIMD** | `wide` | Situational (0 proc-macros; `as_chunks::<N>()` + `trunc_int()`) |
| **Cross-Platform Completion I/O** | `compio-driver` (+ `compio-buf`) | Situational (explicit `ProactorBuilder` bounds + buffer pool) |
| **Fast 32-Bit Checksums** | `crc32fast` | Situational (non-cryptographic frame/block integrity) |
| **Cryptographic Hashing** | `blake3` | Situational (256-bit cryptographic integrity) |
| **Inter-Thread Messaging** | `crossbeam-channel` | Situational (multi-threading only; **`bounded(CAP)` only**) |
| **Seeded Non-Crypto PRNG (Tests/Sampling)** | `fastrand` | Situational / `dev-dependencies` (0 deps; `Rng::with_seed`) |

### Explicitly Blocked Dependencies
* **No `serde` (`serde_json`, `bincode`, etc.)** -> zero carve-outs; prevents hidden allocations, recursive deserialization stack overflows, and derive sprawl.
* **No `anyhow` / `eyre` in library, domain, or state-machine crates** -> `anyhow` is permitted *only* in binary/CLI crates (`src/main.rs` and CLI command drivers).
* **No `tokio` / `async-trait` inside core domain state machines** -> prevents hidden `Box<dyn Future>` allocations, opaque state sizes, and cancellation hazards.
* **No `lazy_static` or hidden mutable globals** -> initialize state explicitly in `main()` and pass references down.
* **No heavy property-test frameworks (`proptest`, `quickcheck`)** -> use built-in `#[test]` boundary suites (plus `fastrand::Rng::with_seed` when randomized sequences are needed).

---

## 6. Mechanical Enforcement & Verification

Wire these checks directly into `Cargo.toml` and `clippy.toml`.
**Lint Suppression Policy:** `#[allow(...)]` is banned (`allow_attributes = "deny"`), and unused expectations fail the build (`unfulfilled_lint_expectations = "deny"`). Ban inner `#![expect(...)]` and `mod`-level `#[expect(...)]` (a single hit anywhere in a large file marks a file-wide `#![expect]` fulfilled and blinds the rest of the module); attach `#[expect]` strictly to the narrowest individual function, item, or statement. Any rare, legitimate suppression (such as an intentional scalar float cast helper, `FixedVec::len`, `lock_unpoisoned`, a flat exhaustive `match` over 70 lines, or an external derive-macro span quirk) **must** include a concrete `reason = "..."`. Never suppress `allow_attributes` or `allow_attributes_without_reason` on handwritten code.

### `Cargo.toml`
```toml
[profile.release]
panic = "abort"
overflow-checks = true

[profile.bench]
overflow-checks = true
# Note: Never set `panic = "abort"` in `[profile.dev]`, `[profile.test]`, or `[profile.bench]`;
# `libtest` requires unwinding and Cargo warns if `panic` is set on `bench`/`test`.

# In a multi-crate workspace, keep `[profile.*]` only in the root `Cargo.toml`, define
# `[workspace.lints.rust]` / `[workspace.lints.clippy]`, and inherit via `[lints] workspace = true`.
[lints.rust]
unsafe_code = "deny"
unsafe_op_in_unsafe_fn = "deny"
unused_must_use = "deny"
unused_comparisons = "deny"
unfulfilled_lint_expectations = "deny"

[lints.clippy]
todo = "forbid"
unimplemented = "forbid"
dbg_macro = "forbid"
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
disallowed_methods = "deny"
let_underscore_must_use = "deny"
assertions_on_constants = "deny"
infinite_loop = "deny"
multiple_unsafe_ops_per_block = "deny"
undocumented_unsafe_blocks = "deny"
mem_forget = "deny"
rc_buffer = "deny"
rc_mutex = "deny"

# Semantic cast safety (blocks narrowing, sign-loss, wrap, lossless `as` where `From` exists,
# precision-loss, and raw pointer `as`, while allowing `#[repr(u8)]` `enum as u8`, inline-masked/shifted bit-extractions, and `u32_to_usize`)
cast_possible_truncation = "deny"
cast_possible_wrap = "deny"
cast_sign_loss = "deny"
cast_lossless = "deny"
cast_precision_loss = "deny"
ptr_as_ptr = "deny"
ptr_cast_constness = "deny"
ref_as_ptr = "deny"
cast_ptr_alignment = "deny"
fn_to_numeric_cast_any = "deny"

wildcard_enum_match_arm = "deny"
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
large_stack_arrays = "deny"
too_many_lines = "deny"
```

### `clippy.toml`
```toml
too-many-lines-threshold = 70
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true

disallowed-methods = [
    { path = "core::mem::transmute", reason = "Use zerocopy traits/macros instead of raw transmute" },
    { path = "std::process::exit", reason = "Bypasses Drop and close() teardown; return Err from main() instead" },
    { path = "std::sync::PoisonError::into_inner", reason = "Never recover poisoned mutex state; abort via lock_unpoisoned" },
    { path = "std::sync::mpsc::channel", reason = "Unbounded channel; use crossbeam_channel::bounded" },
    { path = "crossbeam_channel::unbounded", reason = "All channels must have an explicit upper bound", allow-invalid = true },
    { path = "alloc::string::String::from_utf8_lossy", reason = "Silently replaces invalid UTF-8; validate with core::str::from_utf8" },
    { path = "alloc::string::String::from_utf16_lossy", reason = "Silently replaces invalid UTF-16; validate explicitly" },
    { path = "core::result::Result::ok", reason = "Silently discards the error cause; propagate with ? or match specific error variants" },
    { path = "core::result::Result::unwrap_or", reason = "No silent error fallbacks; propagate with ? (never rewrite into a match returning a dummy default)" },
    { path = "core::result::Result::unwrap_or_default", reason = "No silent error default fallbacks; propagate with ?" },
    { path = "core::result::Result::unwrap_or_else", reason = "No silent error fallback closures; propagate with ?" },
    { path = "core::result::Result::map_or", reason = "No silent error fallbacks; propagate with ?" },
    { path = "core::result::Result::map_or_else", reason = "No silent error fallbacks; propagate with ?" },
    { path = "core::option::Option::unwrap_or", reason = "No silent fallbacks; use ok_or/ok_or_else(?) or handle None explicitly without dummy defaults" },
    { path = "core::option::Option::unwrap_or_default", reason = "No silent default fallbacks; use ok_or/ok_or_else(?) or handle None explicitly" },
    { path = "core::option::Option::unwrap_or_else", reason = "No silent fallback closures; use ok_or_else(?) or handle None explicitly" },
    { path = "core::option::Option::map_or", reason = "No silent fallbacks; use ok_or(?) or handle None explicitly without dummy defaults" },
    { path = "core::option::Option::map_or_else", reason = "No silent fallbacks; use ok_or_else(?) or handle None explicitly without dummy defaults" },
    { path = "wide::f32x4::fast_trunc_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f32x4::fast_round_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f32x8::fast_trunc_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f32x8::fast_round_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f32x16::fast_trunc_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f32x16::fast_round_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f64x2::fast_trunc_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f64x2::fast_round_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f64x4::fast_trunc_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f64x4::fast_round_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f64x8::fast_trunc_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
    { path = "wide::f64x8::fast_round_int", reason = "Non-portable NaN/overflow behavior; use trunc_int() or round_int()", allow-invalid = true },
]
```

### Verification Checklist Before Completing Work
1. Verify `Cargo.toml` contains zero blocked dependencies or overlapping alternatives to Section 5's canonical toolkit, and that dependencies are declared only in member crates that directly import them.
2. Run `cargo test` and verify tests cover positive invariants, negative rejections (`Err(...)` on external boundaries, `#[should_panic]` on internal contract violations), and `_MAX` boundary edges (`0`, `MAX - 1`, `MAX`, `MAX + 1`) with zero silent early-`return` skips.
3. Run `cargo clippy --all-targets -- -D warnings` and resolve all lint errors without file/module-wide `#![expect]` or shortcut suppressions.
4. Audit every modified module to confirm:
   * Public/external functions are total (`Result`, `.checked_*` math, checked slice narrowing, empty-slice `&[]` totality, zero `assert!` / `.saturating_*` / bare `+ - * / %` on unvalidated untrusted input).
   * Invariant-bearing structs/newtypes have private fields (`is_finite()` enforced on constrained floats, no invalid `Default` impls), and resources with fallible/durable teardown expose `close(self) -> Result<(), Error>`.
   * Zero silent fallbacks (`unwrap_or*`, `map_or*`, `.ok()`, `let _ =`, bare `if let Ok`, error-discarding `let Ok(...) else`, dummy-default `match` arms, `.saturating_*` on internal indices/offsets/counters, redundant `.strict_*` calls (rewrite to `+ - *`), clamped/wrapped invalid indices, `from_utf8_lossy`, `Unknown(u8)` domain enum catch-alls, unreachable `try_from(x & MASK)` or `.to_le_bytes()` cast-dodge hacks, `Result<T, String>`, `PoisonError::into_inner`, ignored chunk remainders, unasserted parallel `.zip()` lengths, or silent bounded-loop exhaustion).
   * Zero tautology assertions or quota-padding checks; all `assert!`s verify real cross-state or paired boundary invariants.
   * Zero post-startup heap growth in hot paths (no hidden `Scratch::new()` wrappers), zero hand-rolled linear-probing hash tables or `binary_search` key-value maps (rewrite self-contained `K -> V` tables to `FxHashMap`/`FxHashSet`, and external-context/arena index tables to `hashbrown::HashTable<u32>`), zero "shred-and-reassemble" fake SoA (point-lookup fields share a `Row`; searched positional/arena tables split at most into hot `Box<[Key]>` + cold `Box<[Payload]>`; standalone `Box<[Field]>` columns exist only for selective slice scans or contiguous `as_chunks::<N>()` SIMD loads), zero standalone `parent_ids: Box<[ParentId]>` columns (use `&children[start..end]` slice narrowing for top-down lookups, and keep bottom-up `parent: ParentId` inline in the child struct), zero `(_offset, _len)` plumbing, and zero recursion.

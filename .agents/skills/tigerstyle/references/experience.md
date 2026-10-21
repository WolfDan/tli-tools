# TigerStyle Developer Experience Guide

> "Great names are the essence of great code, capturing what a thing is or does, for a crisp mental model. Optimize the total cost of software ownership, not for those who write it once, but for those who read and run it many times." — [TigerStyle](https://tigerstyle.dev/)

Developer Experience (DX) in TigerStyle is not about personal aesthetics; it is about cognitive efficiency, team alignment, reducing visual noise, and preventing misunderstandings.

---

## 1. Big-Endian Naming

In TigerStyle, variable names are structured hierarchically from the **most significant category/noun** to the **least significant qualifier or unit**:

```
[Category / Domain]_[Entity]_[Unit / Scope]_[Qualifier]
```

### Examples

| Bad (Little-Endian) | Good (TigerStyle Big-Endian) | Why |
| :--- | :--- | :--- |
| `max_connection_delay_ms` | `connection_delay_ms_max` | Groups all `connection_delay` metrics together in autocomplete and code |
| `min_connection_delay_ms` | `connection_delay_ms_min` | Lines up symmetrically with `_max` |
| `total_account_count` | `account_count_total` | Noun `account` comes first |
| `timeout_in_millis` | `timeout_ms` | Standard unit suffix |
| `batch_size_limit` | `batch_size_limit` or `batch_limit` | Clear hierarchy |

### Why Big-Endian Naming Matters
1. **Alphabetical Grouping**: When identifiers are sorted alphabetically, related concepts group together naturally.
2. **Visual Scanability**: Code reviews quickly spot missing symmetric counterparts (e.g. seeing `_min` immediately prompts checking for `_max`).

---

## 2. Symmetrical Variable Naming

When variables represent paired roles or dual endpoints, select names with the **exact same character length** so that columns and calculations align horizontally:

| Standard Names | Symmetrical TigerStyle Pairs | Character Count |
| :--- | :--- | :--- |
| `src` / `dest` | `source` / `target` | 6 chars == 6 chars |
| `in` / `output` | `input` / `output` | 5 chars vs 6 chars → align with padding |
| `head` / `tail` | `head` / `tail` | 4 chars == 4 chars |
| `min` / `max` | `min` / `max` | 3 chars == 3 chars |

```zig
// ✅ Visual Symmetry in memory operations
std.mem.copyForwards(u8, target_buffer[target_offset..], source_buffer[source_offset..]);
```

---

## 3. Style by the Numbers

```
┌─────────────────────────────────────────────────────────┐
│  • Max Function Length:  70 lines                       │
│  • Max Line Width:       100 columns                    │
│  • Indentation:          4 spaces (no tabs)             │
│  • Assertion Density:    >= 2 assertions per function   │
└─────────────────────────────────────────────────────────┘
```

### 3.1 70-Line Function Limit
* Fits on a single laptop screen without vertical scrolling.
* Forces separation between coordinator logic and pure leaf calculations.
* Keeps cognitive load minimal.

### 3.2 100-Column Hard Limit
* Allows two files to be viewed side-by-side on standard displays without horizontal scrolling.
* Use trailing commas in function calls and struct initializations to let code formatters wrap lines gracefully.

---

## 4. Top-Down Layout & Struct Anatomy

Code should read like a cohesive narrative from highest abstraction to lowest detail:

### 4.1 File Level Order
1. File doc comment (describing module purpose and architecture).
2. Primary public entry points / `main()` function.
3. Supporting public functions.
4. Private helper functions.
5. Unit tests.

### 4.2 Struct Level Order
```zig
pub const Replica = struct {
    // 1. FIELDS (State)
    cluster_id: u128,
    replica_id: u8,
    status: Status,

    // 2. TYPES & CONSTANTS (Definitions)
    pub const Status = enum { joining, active, recovering };
    const Self = @This();

    // 3. METHODS (Public API followed by private helpers)
    pub fn init(target: *Self, cluster_id: u128, replica_id: u8) !void { ... }
    pub fn tick(self: *Self) void { ... }
    fn handle_message(self: *Self, message: *const Message) void { ... }
};
```

---

## 5. Commenting: Always Say Why

> Code tells you *how*; comments tell you *why*.

1. **Explain the Rationale**: Document the non-obvious engineering decisions, mathematical constraints, and trade-offs.
2. **Prose Quality**: Write comments as complete English sentences starting with a capital letter and ending with a period.
3. **No Redundant Comments**: Do not repeat the identifier name in the comment.

```zig
// ✅ Explaining WHY:
// We align the WAL record buffer to 4096 bytes so that kernel direct I/O (O_DIRECT)
// bypasses the page cache without triggering alignment faults on NVMe controllers.
const wal_buffer_align = 4096;

// ❌ Redundant comment:
// The WAL buffer alignment variable.
const wal_buffer_align = 4096;
```

---

## 6. Commit Message Etiquette

Commit messages are permanent records stored in git history. Pull requests disappear or get detached, but commits remain forever in `git blame`.

* **Structure**:
  ```
  subsystem: concise summary in present tense (<= 72 chars)

  Explain the context, the problem being solved, and WHY this approach
  was chosen over alternatives. Mention performance or safety implications.
  ```
* Include quantitative benchmark numbers, assertion additions, or architectural invariants.

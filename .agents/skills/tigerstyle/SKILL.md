---
name: tigerstyle
description: >-
  Expert engineering methodology and coding standard based on TigerStyle (TigerBeetle).
  Enforces safety-critical principles (NASA Power of 10, >=2 assertions per function,
  static memory allocation at startup, explicit bounds on all loops and queues, zero technical debt),
  extreme performance (mechanical sympathy, primary colors, control/data plane separation, zero-copy,
  cache-line alignment), and developer experience (big-endian naming, 70-line function limit, 100-col rule,
  and explaining why). Use when designing, writing, refactoring, or reviewing mission-critical,
  high-performance, or safety-first software.
license: Apache-2.0
compatibility: Universal (Antigravity/AGY, OpenAI Codex, Claude Code, Cursor, Copilot, skills CLI)
metadata:
  version: 1.0.0
  author: TigerBeetle Community & AgentSkills
  source: https://tigerstyle.dev/
---

# TigerStyle Agent Skill

> "Our design goals are safety, performance, and developer experience. In that order. All three are important. Good style advances these goals." — [TigerStyle](https://tigerstyle.dev/)

TigerStyle is an engineering philosophy and coding standard born from [TigerBeetle](https://tigerbeetle.com), designed for mission-critical, deterministic, and high-throughput systems. It blends NASA safety standards with mechanical sympathy and artistic craftsmanship ("biodigital jazz").

---

## When to Apply This Skill

Activate TigerStyle when:
1. **Writing new code** in systems programming (Zig, C, C++, Rust, Go) or high-reliability applications.
2. **Refactoring legacy code** to eliminate technical debt, memory leaks, and latent bugs.
3. **Conducting code reviews & PR audits** to enforce assertions, bounds, naming, and function constraints.
4. **Architecting high-throughput / low-latency systems** where unpredictable GC pauses, dynamic allocations, or cache thrashing are unacceptable.

---

## Core Tenets: The Three Axioms

```
   ┌─────────────────────────────────────────────────────────┐
   │  1. SAFETY                                              │
   │     Defense-in-depth, runtime assertions, static alloc  │
   │                                                         │
   │  2. PERFORMANCE                                         │
   │     Mechanical sympathy, primary colors, zero-copy      │
   │                                                         │
   │  3. DEVELOPER EXPERIENCE                                │
   │     Big-endian naming, 70-line limit, explaining why    │
   └─────────────────────────────────────────────────────────┘
```

1. **Zero Technical Debt Policy**: "Code, like steel, is easier to change while it's hot." Solve showstoppers immediately in design or implementation. Never allow latent bugs or exponential complexity to pass.
2. **Zero External Dependencies**: Beyond the native compiler toolchain and standard libraries, eliminate third-party dependencies to guard against supply chain vulnerabilities, performance degradation, and build friction.
3. **Simplicity is the Hardest Revision**: Simplicity is not the first attempt. It is the result of multiple design passes to find the "super idea" that simultaneously resolves multiple constraints.

---

## Quick Reference Rules

### 1. Safety & Correctness
* **Explicit Bounds on Everything**: Every loop must have an asserted upper iteration bound. Every queue, buffer, and batch must have fixed maximum capacity.
* **Static Memory Allocation**: Allocate 100% of memory at startup during initialization. **Zero dynamic heap allocations (`malloc`, `free`, `realloc`) during steady state/data plane.**
* **Relentless Assertions (Minimum 2 per function)**:
  - Assert function arguments, return values, preconditions, postconditions, and invariants.
  - **Pair Assertions**: Assert state at both ends (e.g. before disk write and immediately after disk read).
  - **Assert Positive & Negative Space**: Assert what is expected AND explicitly what is forbidden.
  - **Assert Compile-Time Invariants**: Check struct sizes, alignments, and constant relationships at compile time.
  - **Split Compound Assertions**: Prefer `assert(a); assert(b);` over `assert(a and b);`.
  - **Implication Pattern**: Use single-line conditional asserts: `if (condition) assert(property);`.
* **Simple, Explicit Control Flow**: No recursion. Push `if` statements up to coordinator functions and `for` loops down into pure leaf workers.
* **Narrowest Variable Scope**: Declare variables at the smallest possible scope. Never keep unused variables in scope (eliminates Place-of-Check to Place-of-Use bugs).
* **70-Line Function Limit**: Hard limit of 70 lines per function. Split functions into centralized coordinator functions (handling control flow) and pure leaf helpers.
* **100% Error Handling**: Handle every error explicitly. Never swallow errors.

### 2. Extreme Performance
* **The 4 Primary Colors**: Model performance across Network, Storage, Memory, and Compute, evaluated by Bandwidth and Latency.
* **Bottleneck Hierarchy**: Optimize for the slowest resource first: Network → Storage → Memory → Compute.
* **Control Plane vs. Data Plane**:
  - *Control Plane*: Runs orchestration and extensive assertions.
  - *Data Plane*: Zero-copy, direct memory access, tight CPU loops, batching.
* **Zero-Copy & Zero-Deserialization**: Direct wire-to-memory layouts. Align structs to cache lines (64B) and sort fields from largest to smallest to avoid padding holes.
* **In-Place Initialization**: Construct large structs in place via out-pointers (`target: *Struct`) to eliminate stack copy/moves and prevent stack overflow.
* **Guard Against Buffer Bleeds**: Explicitly zero unused padding and buffers to prevent data leaks and maintain determinism.

### 3. Developer Experience & Formatting
* **Big-Endian Naming**: Most significant noun first, specific units/qualifiers last:
  - ✅ `latency_ms_max`, `latency_ms_min`, `connection_count_limit`
  - ❌ `max_latency_ms`, `min_latency_ms`, `max_connections`
* **Symmetrical Variable Names**: Equal length for paired names so source lines align visually (e.g. `source` / `target` instead of `src` / `dest`).
* **Snake_case**: Standard `snake_case` for functions, variables, and filenames.
* **Layout Structure**: Top-down readability. Entry point / `main()` first. For structs: `Fields` → `Types` → `Methods`.
* **Style by the Numbers**:
  - Maximum **100 columns** per line.
  - **4 spaces** indentation.
  - Explicit braces `{}` for multi-line control blocks.
* **Always Say Why**: Comments must explain the rationale (*why* and *how*), written in full, punctuated English sentences.

---

## Detailed References (Progressive Disclosure)

For in-depth explanations and domain deep-dives, consult:
* 🛡️ [Safety Guide](./references/safety.md): NASA Power of 10, assertion strategies, static memory, bounded loops.
* ⚡ [Performance Guide](./references/performance.md): Primary colors, mechanical sympathy, zero-copy, in-place init.
* 📐 [Developer Experience Guide](./references/experience.md): Big-endian naming, layout rules, function anatomy, 100 cols.
* ✅ [PR Review Checklist](./references/checklist.md): Step-by-step checklist for TigerStyle compliance.

---

## Reference Implementations

Explore canonical TigerStyle examples in multiple languages:
* [Zig Canonical Example](./examples/zig_canonical.zig)
* [C Canonical Example](./examples/c_canonical.c)
* [Rust Canonical Example](./examples/rust_canonical.rs)
* [TypeScript Canonical Example](./examples/ts_canonical.ts)

---

## Automated Verification Tools

Run the included verification scripts:
* **Lint codebase for TigerStyle compliance**:
  ```bash
  python3 scripts/tigerstyle_lint.py <path-to-source-files>
  ```
* **Review git diff / PR changes**:
  ```bash
  python3 scripts/tigerstyle_review.py
  ```

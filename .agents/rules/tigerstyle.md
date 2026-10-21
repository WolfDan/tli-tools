# TigerStyle Agent Guidelines & System Rules

This project enforces **TigerStyle** (TigerBeetle engineering methodology and coding philosophy). When generating, modifying, or reviewing code in this repository, you MUST strictly adhere to the following rules:

---

## 1. Safety & Correctness (Highest Priority)
- **Assertion Density**: Average at least 2 assertions per function. Assert all function arguments, return values, preconditions, postconditions, and internal state invariants.
- **Pair Assertions**: Assert invariants at both ends of a lifecycle (e.g. before write and after read).
- **Positive & Negative Space**: Assert what is expected AND explicitly what is forbidden.
- **Explicit Bounds**: Every loop must have an iteration counter with an asserted upper limit (`assert(iterations < LIMIT)`). Unbounded loops are strictly prohibited.
- **Static Memory Allocation**: Allocate memory at startup. Zero dynamic heap allocation (`malloc`, `free`, `realloc`) during steady state/data plane.
- **No Recursion**: Use simple, explicit iterative control flow. Push `if`s up to coordinator functions and `for` loops down into pure leaf helpers.
- **Narrowest Scope**: Declare variables at the smallest possible scope. Never keep unused variables in scope.
- **70-Line Function Limit**: Hard limit of 70 lines per function.
- **100% Error Handling**: Handle every error explicitly.

---

## 2. Performance & Hardware Sympathy
- **Primary Colors**: Optimize across Network, Storage, Memory, and Compute (Bandwidth and Latency).
- **Control Plane vs. Data Plane**: Keep complex branching and orchestration in the control plane; keep data plane zero-copy, direct memory, and sprintable.
- **Zero-Copy & Zero-Deserialization**: Direct binary layouts, cache-line aligned (64B), fields ordered largest to smallest.
- **In-Place Initialization**: Construct large structs via out-pointers (`target: *Struct`) to eliminate stack copy/moves.
- **Zero Buffer Bleeds**: Explicitly zero unused padding and buffers.

---

## 3. Developer Experience & Style
- **Big-Endian Naming**: Most significant noun first, unit/qualifier last (`latency_ms_max`, `latency_ms_min`, `connection_count_limit`).
- **Symmetrical Variable Names**: Equal character length for paired concepts (`source` / `target`).
- **100 Columns Max**: Hard limit of 100 columns per line.
- **4 Spaces Indent**: Use 4 spaces for indentation.
- **Always Say Why**: Comments must explain the rationale (*why* and *how*), written in complete punctuated English sentences.
- **Zero Technical Debt**: Fix showstoppers and bugs immediately. Do it right the first time.

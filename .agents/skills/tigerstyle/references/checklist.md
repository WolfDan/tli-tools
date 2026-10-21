# TigerStyle Code Review & PR Checklist

Use this checklist before opening a Pull Request or during code review to verify complete TigerStyle compliance.

---

## 1. Safety & Correctness

- [ ] **Assertion Density**: Does every function contain an average of $\ge 2$ assertions?
- [ ] **Argument & Return Checks**: Are all parameters and return values asserted for validity?
- [ ] **Pair Assertions**: Are critical invariants asserted at both ends (e.g. before write and after read)?
- [ ] **Positive & Negative Space**: Are positive expectations AND forbidden edge states explicitly asserted?
- [ ] **Compile-Time Checks**: Are struct sizes, alignments, and constant relationships asserted at compile-time (`comptime`)?
- [ ] **Loop Bounds**: Does every loop have a strict iteration counter with an asserted upper limit?
- [ ] **No Recursion**: Is control flow purely iterative and explicit?
- [ ] **Zero Dynamic Allocation**: Is all memory allocated at startup? Zero heap allocation in steady-state/data plane?
- [ ] **Narrowest Scope**: Are variables declared at the tightest possible scope?
- [ ] **100% Error Handling**: Are all errors caught, handled, or propagated explicitly? No swallowed errors.

---

## 2. Performance & Memory

- [ ] **Zero-Copy**: Is data passed via pointers/slices without redundant copying in the data plane?
- [ ] **Cache-Line Alignment**: Are high-throughput structs aligned to 64 bytes?
- [ ] **Struct Field Order**: Are fields ordered from largest to smallest to eliminate padding holes?
- [ ] **In-Place Initialization**: Are large structs initialized in place using out-pointers (`target: *Struct`)?
- [ ] **Buffer Bleed Prevention**: Are unused buffer slices and padding explicitly zeroed?
- [ ] **Batching**: Are operations batched to amortize syscall, network, and storage overhead?

---

## 3. Developer Experience & Formatting

- [ ] **Big-Endian Naming**: Are variables named from most significant noun to qualifier/unit (e.g. `timeout_ms_max`, `count_limit`)?
- [ ] **Symmetrical Names**: Do paired variables share identical character lengths (e.g. `source` / `target`)?
- [ ] **70-Line Function Limit**: Are all functions under 70 lines in length?
- [ ] **100-Column Limit**: Does every line fit strictly within 100 columns?
- [ ] **4-Space Indentation**: Is the code formatted with 4 spaces (no tabs)?
- [ ] **Explanatory Comments**: Do comments explain *why* non-obvious code was written, in complete punctuated sentences?
- [ ] **Zero Warnings**: Does the build compile cleanly with zero warnings under the strictest flags?
- [ ] **Descriptive Commit Message**: Does the commit message explain the rationale and trade-offs clearly?

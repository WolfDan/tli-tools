Don't run commands in sandbox mode, since this is NixOS they will always fail.

If you're running in NixOS, always use `nix develop` to run `cargo`.

Always activate and follow `.agents/skills/tigerstyle-rust/SKILL.md` whenever planning, writing, refactoring, or reviewing Rust code.

Write data-oriented Rust: use contiguous tables, slices, and Struct-of-Arrays for hot data planes, algebraic sum types (`enum`) and private-field newtypes for domain state so invalid states are unrepresentable at compile time, and SIMD (`as_chunks::<N>()` + `wide`) strictly on genuine bulk numeric or byte streams—never invent speculative framework wrappers or single-scalar SIMD theater.

Always diagnose and solve the root architectural or type-system cause of an issue; never apply surface-level syntactic patches, silent fallbacks, or file/module-wide lint suppressions.

Don't remove existing comments; you can modify or improve them, but don't remove them unless explicitly told to do so.

Avoid using emojis in code, comments, documentation, and responses.

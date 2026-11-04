Don't run commands in sandbox mode, since this is NixOS they will always fail.

If you're running in NixOS, always use `nix develop` to run `cargo`.

Always activate and follow `.agents/skills/tigerstyle-rust/SKILL.md` whenever planning, writing, refactoring, or reviewing Rust code.

Write data-oriented Rust: group contiguous tables by access temperature (`Box<[Row]>` for `0..N` point-lookup records, `FxHashMap<K, Row>` for `K -> Row` key-value lookups, hot `Box<[Key]>` + cold `Box<[Payload]>` for searched positional/arena tables, and standalone SoA columns strictly for selective slice scans or `as_chunks::<N>()` SIMD lane loads—never "shred-and-reassemble" co-accessed fields into separate arrays or hand-roll linear-probing hash tables), use algebraic sum types (`enum`) and private-field newtypes so invalid states are unrepresentable at compile time, and use SIMD (`as_chunks::<N>()` + `wide`) strictly on genuine bulk numeric or byte streams—never invent speculative framework wrappers or single-scalar SIMD theater.

Files under `crates/parsers/src/luac/tables/` are auto-generated: do not manually edit them during refactoring or style passes; if their generated code or signatures need to change for a valid architectural reason, update/re-run the generator rather than hand-patching individual table files.

Always diagnose and solve the root architectural or type-system cause of an issue; never apply surface-level syntactic patches, silent fallbacks, or file/module-wide lint suppressions.

Don't remove existing comments; you can modify or improve them, but don't remove them unless explicitly told to do so.

Avoid using emojis in code, comments, documentation, and responses.

# TLI Tools

Data-oriented, TigerStyle Rust workspace engineered for deterministic memory bounds, zero-copy binary layouts, and hardware-accelerated SIMD batch processing.

## Workspace Crates

| Crate | Package | Description |
| --- | --- | --- |
| [`crates/data`](crates/data) | `data` | Data-oriented domain crate for TLI Tools. |
| [`crates/pak`](crates/pak) | `pak` | Data-oriented TigerStyle Unreal Engine `.pak` archive reader (`v8A`–`v11` and Torchlight Infinite `v12`). |
| [`crates/pak_vfs`](crates/pak_vfs) | `pak_vfs` | High-performance, low-memory Virtual File System unifying ordered Unreal Engine and Torchlight Infinite `.pak` mounts. |
| [`crates/parsers`](crates/parsers) | `parsers` | TigerStyle Lua 5.3 / `\x01XDI` bytecode decompiler, strongly-typed `DataCfg` table evaluators, and `ReadBin` `.bin` table parsers. |
| [`crates/xtask`](crates/xtask) | `xtask` | Workspace automation and `DataCfg` / `LuaPreLoadTable` and PE64 `ReadBin` `.bin` schema extractors and code generators (`cargo codegen`). |
| [`crates/workspace-hack`](crates/workspace-hack) | `workspace-hack` | Unified cross-platform dependency feature set managed automatically by `cargo-hakari`. |

## Development & Verification (NixOS / Nix Flakes)

```bash
# Enter the development shell
nix develop

# Run tests across the workspace
nix develop --command cargo test --workspace

# Run strict TigerStyle Clippy verification
nix develop --command cargo clippy --workspace --all-targets -- -D warnings

# Regenerate and verify cargo-hakari workspace-hack after changing dependencies
nix run .#hakari

# Run the full Crane verification suite (build, hakari, clippy, rustdoc, rustfmt, taplo, audit, nextest)
nix flake check
```

## Local Game Archive Integration Tests & Codegen (`pak`, `pak_vfs`, `parsers`, and `xtask`)

Live `.pak`, `.out`, and `.bin` integration tests in [`crates/pak/tests/torchlight_infinite.rs`](crates/pak/tests/torchlight_infinite.rs), [`crates/pak_vfs/tests/torchlight_infinite.rs`](crates/pak_vfs/tests/torchlight_infinite.rs), [`crates/parsers/src/luac/tests/positive.rs`](crates/parsers/src/luac/tests/positive.rs), and [`crates/parsers/src/bin/tests/positive.rs`](crates/parsers/src/bin/tests/positive.rs) are marked `#[ignore]` so `cargo test` and `nix flake check` remain hermetic across machines without the game installed.

To run them or regenerate the typed `DataCfg` and `.bin` tables against a local Torchlight Infinite installation:

1. Copy [`.envrc.example`](.envrc.example) to `.envrc` (which is ignored by [`.gitignore`](.gitignore) so local paths and keys are never committed to source control):
   ```bash
   cp .envrc.example .envrc
   ```
2. Set `TLI_GAME_DIR` to your local Torchlight Infinite root directory and `TLI_AES_KEY` to the 256-bit hex AES key, then approve the file with `direnv`:
   ```bash
   direnv allow
   ```
3. Run the ignored integration suites or verify `DataCfg` and `.bin` codegen:
   ```bash
   direnv exec . nix develop --command cargo test -p pak -p pak_vfs -p parsers -- --ignored --nocapture
   direnv exec . nix develop --command cargo codegen --check
   ```

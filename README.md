# Tartarus Architect Overlay

Native Rust desktop overlay, Monte Carlo solver, computer-vision screenshot scanner, and `.pak` asset extractor for the **Tartarus** prison mode (S16) in *Torchlight: Infinite*.

## Workspace Crates

| Crate | Description |
| --- | --- |
| [`crates/tartarus-core`](crates/tartarus-core) | Zero-heap Data-Oriented simulation engine (`SimState`, 31 active Torture Instruments, 4 Warden's Gifts, 17 Prison Incidents, 5 Trial Objectives, 4 Start Events) and multi-threaded Rayon Monte Carlo Rollout / UCB1 solver. |
| [`crates/tartarus-scan`](crates/tartarus-scan) | Pure-Rust computer-vision scanner and stateful [`RunTracker`](crates/tartarus-scan/src/tracker.rs) that localizes the 16-cell board, reads beasts, tiers, locks, nests, marks, doors, relics, and decision prompts, and merges frames into the live board model. |
| [`crates/tartarus-overlay`](crates/tartarus-overlay) | Native `eframe` / `egui` transparent desktop overlay with live game log watching (`UE_game.log`), automatic burst capture (`xcap` / PipeWire / X11), interactive board editor, and system tray controls. |
| [`crates/tartarus-extract`](crates/tartarus-extract) | Pure-Rust Unreal Engine 4 `.pak` v9 virtual filesystem (`AES-256-ECB` + Oodle via `oozextract` + Zlib), Lua 5.3 bytecode disassembler (`config_s16_gameplay_*.out`), localization parser, and `UTexture2D` (`BC1`/`BC3`/`BC7`/`BGRA8`) decoder. |
| [`crates/tartarus-data`](crates/tartarus-data) | Authoritative S16 game database extractor, visual catalog renderer (`assets/s16/`), and static Rust CV template generator (`crates/tartarus-scan/src/cv_data.rs`). |

## Running & Building

With Nix Flakes enabled:

```bash
# Run the desktop overlay directly
nix run

# Enter the development shell
nix develop

# Run the regression test suite
nix develop --command cargo test --workspace --release
```

## Regenerating S16 Assets & CV Reference Data

To regenerate the visual catalog sheets in [`assets/s16/`](assets/s16) and the static CV reference tables in [`crates/tartarus-scan/src/cv_data.rs`](crates/tartarus-scan/src/cv_data.rs) directly from a local *Torchlight: Infinite* installation:

```bash
nix develop --command cargo run --release -p tartarus-data -- --pak-dir "/path/to/Torchlight Infinite/UE_game/Torchlight/Content/Paks"
```

To regenerate the 31-relic Monte Carlo synergy table in [`crates/tartarus-core/src/tool_combos.rs`](crates/tartarus-core/src/tool_combos.rs):

```bash
nix develop --command cargo run --release -p tartarus-core --example gen_tool_combos
```

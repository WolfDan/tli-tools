use std::fs;
use std::path::PathBuf;
use tartarus_core::{format_tool_combos_rs, simulate_tool_combos};

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_path = manifest_dir.join("src/tool_combos.rs");
    println!("Simulating 31-relic single & pair synergy tables (400 runs/combo)...");
    let (base, c1, c2) = simulate_tool_combos(400, 0x5160_C0B0);
    let code = format_tool_combos_rs(base, &c1, &c2);
    fs::write(&out_path, code).expect("Failed to write src/tool_combos.rs");
    println!("Wrote {} (COMBO_BASE = {:.4})", out_path.display(), base);
}

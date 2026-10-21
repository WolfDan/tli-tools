use crate::engine::{ALL_TOOLS, Beast, GameMode, PreyLink, SimState, ToolId, ToolMask};
use rayon::prelude::*;

use super::chain_dp::Context;
use super::heuristics::link_value;
use super::params::{REWARDS_PROFILE, SolverParams};
use super::search::playout;

include!("../tool_combos.rs");

pub fn tool_gains(st: &SimState, options: &[ToolId]) -> Option<[f64; 31]> {
    if st.mode == GameMode::Challenge {
        return None;
    }
    let held = st.tools_slice();
    let mut gains = [f64::NAN; 31];
    if held.is_empty() {
        for &id in options {
            let v = COMBO_1[id.idx()];
            if v.is_nan() {
                return None;
            }
            gains[id.idx()] = v - COMBO_BASE;
        }
    } else {
        for &id in options {
            let mut sum = 0.0;
            let mut n = 0usize;
            for &h in held {
                let a = COMBO_2[h.idx()][id.idx()];
                let b = COMBO_1[h.idx()];
                if a.is_nan() || b.is_nan() {
                    continue;
                }
                sum += a - b;
                n += 1;
            }
            if n == 0 {
                return None;
            }
            gains[id.idx()] = sum / (n as f64);
        }
    }

    let days_rem = st.max_days.saturating_sub(st.day) as f64;
    let horizon = (days_rem / 16.0).clamp(0.15, 1.15);
    let sealed_cnt = st.cells.iter().filter(|c| c.sealed).count() as f64;
    let open_inmates = st
        .cells
        .iter()
        .filter(|c| !c.sealed && c.inmate.is_some())
        .count() as f64;
    let lions = st
        .cells
        .iter()
        .filter(|c| !c.sealed && c.inmate.is_some_and(|i| i.beast == Beast::Lion))
        .count() as f64;
    let tusks = st.cells.iter().filter(|c| !c.sealed && c.ivory).count() as f64;

    for &id in options {
        let g = &mut gains[id.idx()];
        if g.is_nan() {
            continue;
        }
        let adj = match id {
            ToolId::Crown | ToolId::Coin => (horizon - 0.65) * 0.45,
            ToolId::Keys | ToolId::Shortchain | ToolId::Cage => {
                ((sealed_cnt - 6.0) / 10.0) * 0.25 + (horizon - 0.6) * 0.15
            }
            ToolId::Shackles | ToolId::Dogtag => (horizon - 0.55) * 0.25,
            ToolId::Trap => ((lions - 1.0) * 0.08 + (horizon - 0.5) * 0.20).clamp(-0.25, 0.35),
            ToolId::Guillotine | ToolId::Claws | ToolId::Maul | ToolId::Hook | ToolId::Rope => {
                ((open_inmates - 3.0) / 8.0).clamp(-0.1, 0.25)
            }
            ToolId::Scapegoat | ToolId::Maiden | ToolId::Stake => {
                (tusks * 0.08 + (horizon - 0.4) * 0.10).clamp(-0.15, 0.25)
            }
            ToolId::Blacklist => {
                if sealed_cnt <= 8.0 {
                    0.20
                } else {
                    -0.05
                }
            }
            _ => 0.0,
        };
        *g += adj;
    }

    Some(gains)
}

#[derive(Copy, Clone, Debug)]
pub struct RerollOdds {
    pub now: f64,
    pub next: f64,
}

pub fn reroll_odds(st: &SimState, options: &[ToolId]) -> Option<RerollOdds> {
    let mut exclude = ToolMask(0);
    for &t in options {
        exclude.insert(t);
    }
    let (others, olen) = st.unheld_tools(exclude);
    let (pool, plen) = if olen >= 3 {
        (others, olen)
    } else {
        st.unheld_tools(ToolMask(0))
    };
    if plen < 3 {
        return None;
    }
    let mut union = [ToolId::Revenge; 31];
    let mut ulen = 0usize;
    for &t in options.iter().chain(pool[..plen].iter()) {
        if !union[..ulen].contains(&t) {
            union[ulen] = t;
            ulen += 1;
        }
    }
    let gains = tool_gains(st, &union[..ulen])?;
    let mut sum = 0.0;
    let mut n = 0usize;
    for i in 0..plen {
        let gi = gains[pool[i].idx()];
        for j in (i + 1)..plen {
            let gj = gi.max(gains[pool[j].idx()]);
            for k in (j + 1)..plen {
                sum += gj.max(gains[pool[k].idx()]);
                n += 1;
            }
        }
    }
    let now = options
        .iter()
        .map(|&id| gains[id.idx()])
        .fold(f64::NEG_INFINITY, f64::max);
    Some(RerollOdds {
        now,
        next: sum / (n as f64),
    })
}

pub fn wants_reroll(st: &SimState, p: &SolverParams, options: &[ToolId]) -> bool {
    if !st.can_reroll() || !p.tool_sets {
        return false;
    }
    if let Some(odds) = reroll_odds(st, options) {
        odds.next > odds.now + p.reroll_margin
    } else {
        false
    }
}

pub fn pick_tool(st: &SimState, p: &SolverParams, options: &[ToolId]) -> ToolId {
    if p.tool_sets
        && let Some(gains) = tool_gains(st, options)
    {
        let mut best = options[0];
        for &id in &options[1..] {
            if gains[id.idx()] > gains[best.idx()] {
                best = id;
            }
        }
        return best;
    }
    for &t in &p.tool_rank {
        if options.contains(&t) {
            return t;
        }
    }
    options[0]
}

pub fn best_hunt(st: &SimState, ctx: &Context) -> Option<(u8, f64)> {
    if !st.can_hunt_now() {
        return None;
    }
    let lion = st.cells[st.position as usize].inmate?;
    let (targets, tlen) = st.hunt_targets();
    let mut best: Option<(u8, f64)> = None;
    for &n in &targets[..tlen] {
        let Some(meal) = st.cells[n as usize].inmate else {
            continue;
        };
        let value = 0.8
            * link_value(
                st,
                ctx,
                PreyLink {
                    lion: lion.id,
                    prey: meal.id,
                    prey_beast: meal.beast,
                    rounds: st.params.prey_rounds,
                    next_day: st.day + 1,
                },
            );
        if best.is_none_or(|(_, b_val)| value > b_val) {
            best = Some((n, value));
        }
    }
    best
}

/// Pure-Rust parallel Monte Carlo simulator for single-instrument and pair-synergy tables
/// across all 31 active S16 Torture Instruments (including `Claws` and `Trap`).
pub fn simulate_tool_combos(runs_per_combo: usize, seed: u32) -> (f64, [f64; 31], [[f64; 31]; 31]) {
    let runs = runs_per_combo.max(1);
    let p = &REWARDS_PROFILE;
    let run_set = |tools: &[ToolId]| -> f64 {
        let sum: u64 = (0..runs)
            .into_par_iter()
            .map(|r| {
                let r_seed = seed.wrapping_add((r as u32).wrapping_mul(0x9e3779b1));
                let mut st = SimState::new_drill_run(r_seed, tools);
                st.params.rewards = true;
                st.milestone = true;
                let mut roll_rng = r_seed ^ 0x5bd1e995;
                playout(&mut st, Some(p), &mut roll_rng);
                st.vault_len as u64
            })
            .sum();
        (sum as f64) / (runs as f64)
    };

    let base = run_set(&[]);
    let mut combo_1 = [0.0f64; 31];
    for t in ALL_TOOLS {
        combo_1[t.idx()] = run_set(&[t]);
    }
    let mut combo_2 = [[f64::NAN; 31]; 31];
    for i in 0..31 {
        for j in (i + 1)..31 {
            let v = run_set(&[ALL_TOOLS[i], ALL_TOOLS[j]]);
            combo_2[i][j] = v;
            combo_2[j][i] = v;
        }
    }
    (base, combo_1, combo_2)
}

pub fn format_tool_combos_rs(base: f64, combo_1: &[f64; 31], combo_2: &[[f64; 31]; 31]) -> String {
    let mut code = String::from(
        "// Auto-generated by `tartarus-data --emit-combos` (pure-Rust Rayon simulation).\n",
    );
    code.push_str(&format!("pub const COMBO_BASE: f64 = {base:.6};\n"));
    code.push_str("pub static COMBO_1: [f64; 31] = [\n");
    for x in combo_1 {
        code.push_str(&format!("    {x:.6},\n"));
    }
    code.push_str("];\n");
    code.push_str("pub static COMBO_2: [[f64; 31]; 31] = [\n");
    for row in combo_2 {
        code.push_str("    [");
        for (idx, x) in row.iter().enumerate() {
            if idx > 0 {
                code.push_str(", ");
            }
            if x.is_nan() {
                code.push_str("f64::NAN");
            } else {
                code.push_str(&format!("{x:.6}"));
            }
        }
        code.push_str("],\n");
    }
    code.push_str("];\n");
    code
}

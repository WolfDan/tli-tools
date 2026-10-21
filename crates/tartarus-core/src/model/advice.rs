use crate::solver::{
    InstantPick, OptionRank, SimAction, SolveGoal, SolverParams, candidates, chain_odds, decide,
    reason, solve_parallel_with_candidates,
};
use serde::{Deserialize, Serialize};

use super::board::{BoardModel, FixedVec, LIVE_MIN, TreasurePhase};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AdviceOptionRow {
    pub label: String,
    pub value_text: String,
    pub width_pct: f32,
    pub is_best: bool,
    pub is_policy: bool,
    pub sub_text: String,
    pub action: Option<SimAction>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AdviceSnapshot {
    pub working: bool,
    pub move_text: String,
    pub why: String,
    pub expect: String,
    pub source: String,
    pub lesson: String,
    pub status: String,
    pub note: String,
    pub problems: Vec<String>,
    pub checks: FixedVec<u8, 16>,
    pub treasure_wait: bool,
    pub tools_to_confirm: String,
    pub options: Vec<AdviceOptionRow>,
    pub best_action: Option<SimAction>,
    pub elapsed_ms: u64,
}

pub fn format_option_value(goal: SolveGoal, o: &OptionRank) -> String {
    if goal == SolveGoal::Contract {
        format!("{:.0}% pass \u{00b7} {:.0} pts", 100.0 * o.pass, o.points)
    } else if o.n > 1 {
        format!("{:.2} \u{00b1}{:.2}", o.mean, 1.96 * o.se)
    } else {
        format!("{:.2}", o.mean)
    }
}

pub fn format_expect_text(goal: SolveGoal, o: &OptionRank, max_days: u8) -> String {
    if goal == SolveGoal::Contract {
        format!(
            "Expected: {:.0}% chance to pass, {:.0} contract points.",
            100.0 * o.pass,
            o.points
        )
    } else {
        format!(
            "Expected: {:.2} Rainbows in the vault by the end of day {max_days}.",
            o.mean
        )
    }
}

impl BoardModel {
    /// Runs both the instant strategy pick and multi-threaded Monte Carlo playouts (`rayon`)
    /// to produce a complete `AdviceSnapshot` for the overlay.
    pub fn evaluate(
        &self,
        playout_budget_total: usize,
        budget_ms: u64,
        seed: u32,
    ) -> AdviceSnapshot {
        let t0 = std::time::Instant::now();
        let mut checks_vec = FixedVec::new();
        for n in 0..16u8 {
            if self.is_checked(n) {
                checks_vec.push(n);
            }
        }
        let tools_to_confirm = self.tools_prompt.format_for(&self.tools);

        if self.treasure == TreasurePhase::WaitingForBoardView {
            return AdviceSnapshot {
                working: false,
                move_text: "The Warden\u{2019}s Treasure: check the board first".to_string(),
                why:
                    "Press View Prison Status in the game, wait a second on the board, then return."
                        .to_string(),
                treasure_wait: true,
                tools_to_confirm,
                checks: checks_vec,
                elapsed_ms: t0.elapsed().as_millis() as u64,
                ..AdviceSnapshot::default()
            };
        }

        let (st, problems) = self.build_state(seed);
        if !problems.is_empty() {
            return AdviceSnapshot {
                working: false,
                move_text: String::new(),
                problems,
                checks: checks_vec,
                tools_to_confirm,
                elapsed_ms: t0.elapsed().as_millis() as u64,
                ..AdviceSnapshot::default()
            };
        }

        let p = SolverParams::for_state(&st);
        let cands = candidates(&st);
        let mut inst_rng = 1u32;
        let inst_action = if !cands.is_empty() {
            decide(&st, Some(p), &mut inst_rng)
        } else {
            None
        };
        let inst = inst_action.map(|action| {
            let label = cands
                .iter()
                .find(|c| c.action.same_action(&action))
                .map(|c| c.label.clone())
                .unwrap_or_else(|| action.type_name().to_string());
            let why = reason(&st, &action, Some(p));
            InstantPick { action, label, why }
        });

        let num_options = cands.len().max(1);
        let rollouts_per_option = (playout_budget_total / num_options).clamp(800, 32_000);
        let summary = solve_parallel_with_candidates(
            &st,
            cands,
            inst_action,
            rollouts_per_option,
            budget_ms,
            seed,
            true,
        );

        let ranked = &summary.options;
        let live = if ranked.len() > 1 && (summary.done || summary.rounds >= LIVE_MIN) {
            ranked.first()
        } else {
            None
        };
        let mine = inst
            .as_ref()
            .and_then(|pick| ranked.iter().find(|o| o.action.same_action(&pick.action)));

        let (rec_action, rec_label, rec_why, rec_live, rec_rounds, rec_opt) = if let Some(l) = live
        {
            if let Some(m) = mine {
                let p = inst.as_ref().unwrap();
                if m.action.same_action(&l.action) || l.mean - m.mean <= 1.96 * m.gap_se {
                    let why = if m.action.same_action(&l.action) {
                        p.why.clone()
                    } else {
                        format!("{} The live check rates it about equal to the best.", p.why)
                    };
                    (
                        Some(p.action),
                        m.label.clone(),
                        why,
                        true,
                        summary.rounds,
                        Some(m),
                    )
                } else {
                    let why = format!(
                        "The playouts found a better move than the strategy\u{2019}s pick ({}).",
                        p.label
                    );
                    (
                        Some(l.action),
                        l.label.clone(),
                        why,
                        true,
                        summary.rounds,
                        Some(l),
                    )
                }
            } else {
                let why =
                    "The playouts found a better move than the strategy\u{2019}s pick.".to_string();
                (
                    Some(l.action),
                    l.label.clone(),
                    why,
                    true,
                    summary.rounds,
                    Some(l),
                )
            }
        } else if let Some(p) = inst.as_ref() {
            (
                Some(p.action),
                p.label.clone(),
                p.why.clone(),
                false,
                0,
                mine,
            )
        } else if ranked.len() == 1 {
            let r0 = &ranked[0];
            (
                Some(r0.action),
                r0.label.clone(),
                "It\u{2019}s the only move here.".to_string(),
                true,
                summary.rounds,
                Some(r0),
            )
        } else {
            (
                None,
                "Nothing to decide here.".to_string(),
                String::new(),
                false,
                0,
                None,
            )
        };

        let expect = rec_opt
            .map(|o| format_expect_text(summary.goal, o, st.max_days))
            .unwrap_or_default();
        let source = if rec_action.is_some() {
            if rec_live && rec_rounds > 0 {
                format!("Checked by {rec_rounds} playouts per option")
            } else {
                "Instant pick from training".to_string()
            }
        } else {
            String::new()
        };

        let mut lesson = String::new();
        if let Some(SimAction::Devour { to }) = rec_action {
            let odds = chain_odds(&st, st.position, to);
            lesson = format!(
                "If the chain then goes on with the best meal at each bite: {:.1} bites on average, {:.0}% to reach Rainbow. Your game's Cancel can stop it at any bite.",
                odds.bites,
                100.0 * odds.rainbow
            );
        }

        let status = if ranked.len() <= 1 {
            if ranked.len() == 1 {
                "Only one move here.".to_string()
            } else {
                "Nothing to decide right now.".to_string()
            }
        } else {
            format!("Live check: {} playouts per option", summary.rounds)
        };

        let max_mean = ranked.iter().map(|o| o.mean).fold(0.01f64, f64::max);

        let mut option_rows = Vec::new();
        if ranked.len() > 1 {
            for o in ranked {
                let is_best = rec_action
                    .as_ref()
                    .is_some_and(|ra| o.action.same_action(ra));
                let width = if summary.goal == SolveGoal::Contract {
                    100.0 * o.pass
                } else {
                    (100.0 * o.mean) / max_mean
                };
                let sub_text = if let SimAction::Devour { to } = o.action {
                    let odds = chain_odds(&st, st.position, to);
                    format!(
                        "chain: {:.1} bites on average, {:.0}% to Rainbow",
                        odds.bites,
                        100.0 * odds.rainbow
                    )
                } else {
                    String::new()
                };
                option_rows.push(AdviceOptionRow {
                    label: o.label.clone(),
                    value_text: format_option_value(summary.goal, o),
                    width_pct: width.clamp(2.0, 100.0) as f32,
                    is_best,
                    is_policy: o.policy,
                    sub_text,
                    action: Some(o.action),
                });
            }
        }

        let close =
            ranked.len() >= 2 && (ranked[0].mean - ranked[1].mean) < 1.96 * ranked[1].gap_se;
        let note = if close {
            "The top moves are within the margin of error: either is fine.".to_string()
        } else {
            String::new()
        };

        AdviceSnapshot {
            working: false,
            move_text: rec_label,
            why: rec_why,
            expect,
            source,
            lesson,
            status,
            note,
            problems: Vec::new(),
            checks: checks_vec,
            treasure_wait: false,
            tools_to_confirm,
            options: option_rows,
            best_action: rec_action,
            elapsed_ms: t0.elapsed().as_millis() as u64,
        }
    }
}

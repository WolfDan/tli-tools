use crate::engine::SimState;
use crate::solver::{
    InstantPick, OptionRank, SimAction, SolveGoal, SolverParams, candidates, chain_odds, decide,
    reason, solve_parallel_with_candidates,
};
use serde::{Deserialize, Serialize};

use super::board::{BoardModel, FixedVec, LIVE_MIN, TreasurePhase};

const _: () = {
    assert!(LIVE_MIN >= 100);
    assert!(std::mem::size_of::<AdviceOptionRow>() > 0);
};

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

struct Recommendation<'a> {
    action: Option<SimAction>,
    label: String,
    why: String,
    live: bool,
    rounds: usize,
    opt: Option<&'a OptionRank>,
}

pub fn format_option_value(goal: SolveGoal, o: &OptionRank) -> String {
    assert!(o.mean.is_finite());
    assert!(o.se.is_finite() && o.pass.is_finite());
    let out = if goal == SolveGoal::Contract {
        format!("{:.0}% pass \u{00b7} {:.0} pts", 100.0 * o.pass, o.points)
    } else if o.n > 1 {
        format!("{:.2} \u{00b1}{:.2}", o.mean, 1.96 * o.se)
    } else {
        format!("{:.2}", o.mean)
    };
    assert!(!out.is_empty());
    out
}

pub fn format_expect_text(goal: SolveGoal, o: &OptionRank, days_max: u8) -> String {
    assert!(days_max > 0);
    assert!(o.mean.is_finite() && o.pass.is_finite());
    let out = if goal == SolveGoal::Contract {
        format!(
            "Expected: {:.0}% chance to pass, {:.0} contract points.",
            100.0 * o.pass,
            o.points
        )
    } else {
        format!(
            "Expected: {:.2} Rainbows in the vault by the end of day {days_max}.",
            o.mean
        )
    };
    assert!(!out.is_empty());
    out
}

fn recommend_from_live<'a>(
    l: &'a OptionRank,
    mine: Option<&'a OptionRank>,
    inst: Option<&InstantPick>,
    rounds: usize,
) -> Recommendation<'a> {
    assert!(l.mean.is_finite());
    assert!(rounds > 0);
    if let Some(m) = mine {
        let p = inst.expect("mine requires instant pick");
        if m.action.same_action(&l.action) || l.mean - m.mean <= 1.96 * m.gap_se {
            let why = if m.action.same_action(&l.action) {
                p.why.clone()
            } else {
                format!("{} The live check rates it about equal to the best.", p.why)
            };
            Recommendation {
                action: Some(p.action),
                label: m.label.clone(),
                why,
                live: true,
                rounds,
                opt: Some(m),
            }
        } else {
            let why = format!(
                "The playouts found a better move than the strategy\u{2019}s pick ({}).",
                p.label
            );
            Recommendation {
                action: Some(l.action),
                label: l.label.clone(),
                why,
                live: true,
                rounds,
                opt: Some(l),
            }
        }
    } else {
        let why = "The playouts found a better move than the strategy\u{2019}s pick.".to_string();
        Recommendation {
            action: Some(l.action),
            label: l.label.clone(),
            why,
            live: true,
            rounds,
            opt: Some(l),
        }
    }
}

fn choose_recommendation<'a>(
    ranked: &'a [OptionRank],
    done: bool,
    rounds: usize,
    inst: Option<&InstantPick>,
) -> Recommendation<'a> {
    assert!(ranked.len() <= 64);
    let live = if ranked.len() > 1 && (done || rounds >= LIVE_MIN) {
        ranked.first()
    } else {
        None
    };
    let mine = inst.and_then(|pick| ranked.iter().find(|o| o.action.same_action(&pick.action)));
    let rec = if let Some(l) = live {
        recommend_from_live(l, mine, inst, rounds)
    } else if let Some(p) = inst {
        Recommendation {
            action: Some(p.action),
            label: p.label.clone(),
            why: p.why.clone(),
            live: false,
            rounds: 0,
            opt: mine,
        }
    } else if ranked.len() == 1 {
        let r0 = &ranked[0];
        Recommendation {
            action: Some(r0.action),
            label: r0.label.clone(),
            why: "It\u{2019}s the only move here.".to_string(),
            live: true,
            rounds,
            opt: Some(r0),
        }
    } else {
        Recommendation {
            action: None,
            label: "Nothing to decide here.".to_string(),
            why: String::new(),
            live: false,
            rounds: 0,
            opt: None,
        }
    };
    assert!(!rec.label.is_empty());
    rec
}

fn format_devour_lesson(st: &SimState, rec_action: Option<SimAction>) -> String {
    assert!(st.position < 16);
    assert!(st.day >= 1);
    if let Some(SimAction::Devour { to }) = rec_action {
        assert!(to < 16);
        let odds = chain_odds(st, st.position, to);
        format!(
            "If the chain then goes on with the best meal at each bite: \
             {:.1} bites on average, {:.0}% to reach Rainbow. Your game's \
             Cancel can stop it at any bite.",
            odds.bites,
            100.0 * odds.rainbow
        )
    } else {
        String::new()
    }
}

fn build_option_rows(
    st: &SimState,
    goal: SolveGoal,
    ranked: &[OptionRank],
    rec_action: Option<SimAction>,
) -> Vec<AdviceOptionRow> {
    assert!(st.position < 16);
    if ranked.len() <= 1 {
        return Vec::new();
    }
    let mean_max = ranked.iter().map(|o| o.mean).fold(0.01f64, f64::max);
    assert!(mean_max.is_finite() && mean_max >= 0.01);
    let mut option_rows = Vec::with_capacity(ranked.len());
    for o in ranked {
        let is_best = rec_action
            .as_ref()
            .is_some_and(|ra| o.action.same_action(ra));
        let width = if goal == SolveGoal::Contract {
            100.0 * o.pass
        } else {
            (100.0 * o.mean) / mean_max
        };
        let sub_text = if let SimAction::Devour { to } = o.action {
            let odds = chain_odds(st, st.position, to);
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
            value_text: format_option_value(goal, o),
            width_pct: width.clamp(2.0, 100.0) as f32,
            is_best,
            is_policy: o.policy,
            sub_text,
            action: Some(o.action),
        });
    }
    assert_eq!(option_rows.len(), ranked.len());
    option_rows
}

fn format_source_and_status(
    rec: &Recommendation<'_>,
    ranked_len: usize,
    rounds: usize,
) -> (String, String) {
    assert!(ranked_len <= 64);
    let source = if rec.action.is_some() {
        if rec.live && rec.rounds > 0 {
            format!("Checked by {} playouts per option", rec.rounds)
        } else {
            "Instant pick from training".to_string()
        }
    } else {
        String::new()
    };
    let status = match ranked_len {
        0 => "Nothing to decide right now.".to_string(),
        1 => "Only one move here.".to_string(),
        _ => format!("Live check: {rounds} playouts per option"),
    };
    assert!(!status.is_empty());
    (source, status)
}

impl BoardModel {
    fn checked_cells(&self) -> FixedVec<u8, 16> {
        assert_eq!(self.cells.len(), 16);
        let mut checks_vec = FixedVec::new();
        for n in 0..16u8 {
            if self.is_checked(n) {
                checks_vec.push(n);
            }
        }
        assert!(checks_vec.len() <= 16);
        checks_vec
    }

    fn solve_state_snapshot(
        &self,
        st: &SimState,
        playout_budget_total: usize,
        budget_ms: u64,
        seed: u32,
    ) -> AdviceSnapshot {
        assert!(st.day >= 1 && st.day <= st.days_max);
        let p = SolverParams::for_state(st);
        let cands = candidates(st);
        let mut inst_rng = 1u32;
        let inst_action = (!cands.is_empty())
            .then(|| decide(st, Some(p), &mut inst_rng))
            .flatten();
        let inst = inst_action.map(|action| {
            let label = cands
                .iter()
                .find(|c| c.action.same_action(&action))
                .map(|c| c.label.clone())
                .unwrap_or_else(|| action.type_name().to_string());
            InstantPick {
                action,
                label,
                why: reason(st, &action, Some(p)),
            }
        });

        let rollouts_per_option = (playout_budget_total / cands.len().max(1)).clamp(800, 32_000);
        assert!((800..=32_000).contains(&rollouts_per_option));
        let summary = solve_parallel_with_candidates(
            st,
            cands,
            inst_action,
            rollouts_per_option,
            budget_ms,
            seed,
            true,
        );
        let ranked = &summary.options;
        let rec = choose_recommendation(ranked, summary.done, summary.rounds, inst.as_ref());
        let expect = rec
            .opt
            .map(|o| format_expect_text(summary.goal, o, st.days_max))
            .unwrap_or_default();
        let (source, status) = format_source_and_status(&rec, ranked.len(), summary.rounds);
        let close =
            ranked.len() >= 2 && (ranked[0].mean - ranked[1].mean) < 1.96 * ranked[1].gap_se;
        AdviceSnapshot {
            working: false,
            move_text: rec.label,
            why: rec.why,
            expect,
            source,
            lesson: format_devour_lesson(st, rec.action),
            status,
            note: if close {
                "The top moves are within the margin of error: either is fine.".to_string()
            } else {
                String::new()
            },
            options: build_option_rows(st, summary.goal, ranked, rec.action),
            best_action: rec.action,
            ..AdviceSnapshot::default()
        }
    }

    /// Runs both the instant strategy pick and multi-threaded Monte Carlo playouts (`rayon`)
    /// to produce a complete `AdviceSnapshot` for the overlay.
    pub fn evaluate(
        &self,
        playout_budget_total: usize,
        budget_ms: u64,
        seed: u32,
    ) -> AdviceSnapshot {
        assert_eq!(self.cells.len(), 16);
        assert!(self.days_max > 0);
        let t0 = std::time::Instant::now();
        let checks_vec = self.checked_cells();
        let tools_to_confirm = self.tools_prompt.format_for(&self.tools);

        if self.treasure == TreasurePhase::WaitingForBoardView {
            return AdviceSnapshot {
                working: false,
                move_text: "The Warden\u{2019}s Treasure: check the board first".to_string(),
                why: "Press View Prison Status in the game, wait a second on the board, \
                      then return."
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

        let mut snap = self.solve_state_snapshot(&st, playout_budget_total, budget_ms, seed);
        snap.checks = checks_vec;
        snap.tools_to_confirm = tools_to_confirm;
        snap.elapsed_ms = t0.elapsed().as_millis() as u64;
        snap
    }
}

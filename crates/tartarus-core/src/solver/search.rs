use crate::engine::{
    BEASTS, ChoiceId, EventKind, GameMode, INSTIGATE_EVENTS, OFFICE, ObjectiveId, OfficeChoice,
    PendingKind, Phase, RAINBOW, SUPREME, SimState, ToolId, cell_label, describe_inmate, next_rand,
    plural, roll_event_from_pool,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::chain_dp::{Context, best_meal, composition};
use super::convoy::{Candidate, ConvoyKind, SolveGoal, convoy_options, convoy_plan, wants_early};
use super::heuristics::{
    devour_scores, execution_value, office_values, patrol_scores_fast, pick_gift, swap_gain_here,
};
use super::params::{PASS_WEIGHT, SimAction, SolverParams, apply_action};
use super::tools::{best_hunt, pick_tool, wants_reroll};

pub fn decide(
    st: &SimState,
    params: Option<&'static SolverParams>,
    rng: &mut u32,
) -> Option<SimAction> {
    if st.phase == Phase::Ended {
        return None;
    }
    let p = params.unwrap_or_else(|| SolverParams::for_state(st));
    let mut ctx = Context::new(st, p);

    if st.pending_len > 0 {
        let choice = &st.pending[0];
        let opts = choice.slice();
        if choice.kind == PendingKind::Gift {
            return Some(SimAction::Choose(pick_gift(st, p, opts, rng)));
        }
        let mut tool_opts = [ToolId::Revenge; 4];
        let mut tlen = 0;
        for &c in opts {
            if let ChoiceId::Tool(t) = c {
                tool_opts[tlen] = t;
                tlen += 1;
            }
        }
        if choice.kind == PendingKind::Tool && wants_reroll(st, p, &tool_opts[..tlen]) {
            return Some(SimAction::Reroll);
        }
        let picked = pick_tool(st, p, &tool_opts[..tlen]);
        return Some(SimAction::Choose(ChoiceId::Tool(picked)));
    }

    if let Some(ch) = st.challenge.as_ref()
        && ch.window_open
    {
        if let Some(plan) = convoy_plan(st, &ctx) {
            return Some(SimAction::Deliver {
                cell_mask: plan.cell_mask,
                vault_ids: plan.vault_ids,
                vault_len: plan.vault_len,
                escort: plan.escort,
            });
        }
        return Some(if ch.window_early {
            SimAction::Close
        } else {
            SimAction::Skip
        });
    }

    if st.chaining_eater.is_some() {
        let (options, olen) = devour_scores(st, &mut ctx);
        let stop_thresh = if st.day >= st.max_days {
            0.0
        } else {
            p.stop_margin
        };
        if ctx.chain_stop && (olen == 0 || options[0].score < stop_thresh) {
            return Some(SimAction::StopChain);
        }
        return Some(if olen > 0 {
            SimAction::Chain {
                to: options[0].cell,
            }
        } else {
            SimAction::End
        });
    }

    if st.phase == Phase::Office {
        if let Some(ov) = p.office_override {
            return Some(SimAction::Office { choice: ov });
        }
        let (comp, _) = composition(st, -1);
        let office = office_values(st, &ctx, &comp);
        let choice = if office.bestow >= office.instigate {
            OfficeChoice::Bestow
        } else {
            OfficeChoice::Instigate
        };
        return Some(SimAction::Office { choice });
    }

    if st.phase == Phase::Event
        && let Some((kind, cell_n)) = st.active_event
    {
        if kind == EventKind::Chain {
            return Some(SimAction::Event {
                accept: p.accept_chain,
            });
        }
        if matches!(
            kind,
            EventKind::Appetite
                | EventKind::Convict
                | EventKind::Crow
                | EventKind::Freedom
                | EventKind::Usurp1
                | EventKind::Usurp2
                | EventKind::Usurp3
                | EventKind::Usurp4
                | EventKind::Ratway
                | EventKind::Hunt
                | EventKind::Portion
                | EventKind::Fortune
                | EventKind::Archenemy
                | EventKind::Transfer
        ) {
            return Some(SimAction::Event { accept: true });
        }
        let Some(exec) = st.cells[cell_n as usize].inmate else {
            return Some(SimAction::Event { accept: false });
        };
        if kind == EventKind::Trickster {
            return Some(SimAction::Event {
                accept: exec.tier < SUPREME,
            });
        }
        let (mut comp, crowd) = composition(st, cell_n as i16);
        let mut ev_val = execution_value(st, &ctx, exec.tier, exec.beast.idx() as u8, &comp);
        if ev_val != 0.0 && st.can_devour_now() && exec.tier < RAINBOW {
            let same = st.params.execution_same_tier;
            let prey = exec.beast.primary_prey().idx();
            let max_t = if same {
                exec.tier as i16
            } else {
                (exec.tier as i16) - 1
            };
            let mut comp_after = comp;
            let mut count = 0u8;
            for t in 0..=max_t {
                if same && t != (exec.tier as i16) {
                    continue;
                }
                let idx = (t as usize) * 4 + prey;
                count += comp_after[idx];
                comp_after[idx] = 0;
            }
            if count > 0 {
                let (bm, _) = best_meal(
                    &mut ctx,
                    exec.tier,
                    exec.beast.idx() as u8,
                    &mut comp,
                    crowd,
                );
                let to = (exec.tier + count).min(RAINBOW);
                let mut post_dev = 0.0;
                if to < RAINBOW {
                    let rem_crowd = crowd.saturating_sub(count as usize);
                    for b in 0..4u8 {
                        let (pbm, _) = best_meal(&mut ctx, to, b, &mut comp_after, rem_crowd);
                        if pbm > 0.0 {
                            post_dev += 0.25 * pbm;
                        }
                    }
                }
                ev_val += post_dev;
                if bm > 0.0 && ev_val < bm {
                    return Some(SimAction::Event { accept: false });
                }
            }
        }
        return Some(SimAction::Event {
            accept: ev_val > p.exec_margin,
        });
    }

    if st.phase == Phase::Patrol {
        let (scores, len) = patrol_scores_fast(st, &mut ctx);
        return Some(if len > 0 {
            SimAction::Patrol {
                cell: scores[0].cell,
            }
        } else {
            SimAction::End
        });
    }

    let swap_gain = swap_gain_here(st, &mut ctx);
    let hunt = best_hunt(st, &ctx);
    if st.can_devour_now() {
        let (devours, dlen) = devour_scores(st, &mut ctx);
        let min_dev = if st.day >= st.max_days {
            0.0
        } else {
            p.min_devour
        };
        if dlen > 0 && devours[0].score >= min_dev {
            if let Some((h_cell, h_val)) = hunt {
                // Under CageFight, Hunt is a 100% free action before Devour if the Hunt target isn't the meal being eaten!
                if st.cage_fight() && h_val > 0.0 && h_cell != devours[0].cell {
                    return Some(SimAction::Hunt { to: h_cell });
                }
                if h_val > devours[0].score && h_val > swap_gain {
                    return Some(SimAction::Hunt { to: h_cell });
                }
            }
            if swap_gain > 0.0 && swap_gain > devours[0].score + p.swap_margin {
                return Some(SimAction::Swap);
            }
            return Some(SimAction::Devour {
                to: devours[0].cell,
            });
        }
    }
    if let Some((h_cell, h_val)) = hunt
        && h_val > 0.0
        && h_val > swap_gain
    {
        return Some(SimAction::Hunt { to: h_cell });
    }
    if swap_gain > 0.0 {
        return Some(SimAction::Swap);
    }
    if st.challenge.is_some() && wants_early(st, &ctx) {
        return Some(SimAction::Early);
    }
    Some(SimAction::End)
}

pub fn playout(st: &mut SimState, params: Option<&'static SolverParams>, rng: &mut u32) {
    for _ in 0..2000 {
        if st.phase == Phase::Ended {
            break;
        }
        let Some(action) = decide(st, params, rng) else {
            break;
        };
        let ok = apply_action(st, &action);
        if !ok && !(action != SimAction::End && apply_action(st, &SimAction::End)) {
            break;
        }
    }
}

pub fn hide_future(st: &mut SimState, seed: u32) {
    let mut rng = seed;
    st.rng = (next_rand(&mut rng) * 4294967296.0) as u32;
    if st.objective_id == ObjectiveId::Other && !st.milestone && st.params.rewards {
        const S16_OBJS: [ObjectiveId; 5] = [
            ObjectiveId::King,
            ObjectiveId::Walk,
            ObjectiveId::Doors,
            ObjectiveId::Consume,
            ObjectiveId::Incident,
        ];
        let mut valid = [ObjectiveId::Walk; 5];
        let mut vlen = 0usize;
        for &obj in &S16_OBJS {
            let already_met = match obj {
                ObjectiveId::King => st.vault_len > 0 || st.stats.best >= RAINBOW,
                ObjectiveId::Walk => false,
                ObjectiveId::Doors => st.stats.opened >= (obj.need() as u16),
                ObjectiveId::Consume => (st.stats.devours + st.stats.chains) >= (obj.need() as u16),
                ObjectiveId::Incident => false,
                ObjectiveId::Other => false,
            };
            if !already_met {
                valid[vlen] = obj;
                vlen += 1;
            }
        }
        if vlen > 0 {
            let idx = (next_rand(&mut rng) * (vlen as f64)).floor() as usize;
            let picked = valid[idx.min(vlen - 1)];
            st.objective_id = picked;
            st.objective_progress = match picked {
                ObjectiveId::Doors => (st.stats.opened as u8).min(picked.need().saturating_sub(1)),
                ObjectiveId::Consume => ((st.stats.devours + st.stats.chains) as u8)
                    .min(picked.need().saturating_sub(1)),
                _ => st.objective_progress,
            };
        }
    }

    let pool_len = (st.event_pool_len as usize).min(st.event_pool.len());
    let mut pool = [EventKind::Chain; 17];
    pool[..pool_len].copy_from_slice(&st.event_pool[..pool_len]);

    for c in &mut st.cells {
        if c.event.is_some() && !c.event_known && pool_len > 0 {
            c.event = Some(roll_event_from_pool(&mut rng, &pool[..pool_len]));
        }
        if c.sealed && !c.seen && c.nest_due.is_none() {
            if next_rand(&mut rng) * 100.0 < st.params.event_chance && pool_len > 0 {
                c.event = Some(roll_event_from_pool(&mut rng, &pool[..pool_len]));
            } else {
                c.event = None;
            }
        }
        if !c.sealed || c.hidden.is_none() || c.known >= 2 {
            continue;
        }
        if c.known == 1 {
            if let Some(ref mut h) = c.hidden {
                let b_idx = (next_rand(&mut rng) * 4.0).floor() as usize;
                h.beast = BEASTS[b_idx.min(3)];
            }
        } else {
            c.hidden = None;
        }
    }
}

pub fn candidates(st: &SimState) -> Vec<Candidate> {
    let mut out = Vec::new();
    if st.phase == Phase::Ended {
        return out;
    }
    if st.pending_len > 0 {
        for &id in st.pending[0].slice() {
            out.push(Candidate {
                action: SimAction::Choose(id),
                label: format!("Take {}", id.name()),
                kind: None,
                points: None,
                extra: false,
            });
        }
        if st.can_reroll() {
            out.push(Candidate {
                action: SimAction::Reroll,
                label: "Re-roll the offer".to_string(),
                kind: None,
                points: None,
                extra: false,
            });
        }
        return out;
    }

    if let Some(ch) = st.challenge.as_ref()
        && ch.window_open
    {
        out.extend(convoy_options(st, None));
        if ch.window_early {
            out.push(Candidate {
                action: SimAction::Close,
                label: "Close the early window".to_string(),
                kind: None,
                points: None,
                extra: false,
            });
        } else {
            out.push(Candidate {
                action: SimAction::Skip,
                label: "Skip this contract".to_string(),
                kind: None,
                points: None,
                extra: false,
            });
        }
        return out;
    }

    if st.chaining_eater.is_some() {
        let (opts, olen) = st.chain_options();
        for &o in &opts[..olen] {
            let target = &st.cells[o.cell as usize];
            let meal = if o.scratch {
                "the scratched door".to_string()
            } else {
                format!("the {}", describe_inmate(target.inmate.unwrap()))
            };
            out.push(Candidate {
                action: SimAction::Chain { to: o.cell },
                label: format!("Chain on: eat {meal} in {}", cell_label(o.cell)),
                kind: None,
                points: None,
                extra: false,
            });
        }
        if st.can_stop_chain() {
            out.push(Candidate {
                action: SimAction::StopChain,
                label: "Stop the chain (Cancel)".to_string(),
                kind: None,
                points: None,
                extra: false,
            });
        }
        return out;
    }

    if st.phase == Phase::Office {
        out.push(Candidate {
            action: SimAction::Office {
                choice: OfficeChoice::Bestow,
            },
            label: "Bestow: 1 random inmate rises a tier".to_string(),
            kind: None,
            points: None,
            extra: false,
        });
        out.push(Candidate {
            action: SimAction::Office {
                choice: OfficeChoice::Instigate,
            },
            label: format!("Instigate: {INSTIGATE_EVENTS} new special events"),
            kind: None,
            points: None,
            extra: false,
        });
        return out;
    }

    if st.phase == Phase::Event
        && let Some((kind, _)) = st.active_event
    {
        let prefix = if matches!(kind, EventKind::Chain | EventKind::Execution) {
            "the "
        } else {
            ""
        };
        let name = format!("{prefix}{}", kind.name());
        if kind == EventKind::Hunt {
            out.push(Candidate {
                action: SimAction::Event { accept: true },
                label: format!("Confirm {name}"),
                kind: None,
                points: None,
                extra: false,
            });
            return out;
        }
        out.push(Candidate {
            action: SimAction::Event { accept: true },
            label: format!("Accept {name}"),
            kind: None,
            points: None,
            extra: false,
        });
        out.push(Candidate {
            action: SimAction::Event { accept: false },
            label: format!("Skip {name}"),
            kind: None,
            points: None,
            extra: false,
        });
        return out;
    }

    if st.phase == Phase::Patrol {
        if st.can_open_early() {
            out.push(Candidate {
                action: SimAction::Early,
                label: "Deliver the heavy contract early".to_string(),
                kind: None,
                points: None,
                extra: false,
            });
        }
        for &n in st.destinations_slice() {
            let c = &st.cells[n as usize];
            let what = if st.chain_armed {
                let (path, plen) = st.clockwise_path(n);
                let opens = path[..plen]
                    .iter()
                    .filter(|&&k| st.cells[k as usize].sealed)
                    .count();
                format!("the chain opens {}", plural(opens, "door"))
            } else if n == OFFICE {
                "Instigate or Bestow".to_string()
            } else if c.sealed {
                if c.nest_due.is_some() {
                    "the rat nest".to_string()
                } else if c.event.is_some() {
                    "a locked door with an event".to_string()
                } else {
                    "a locked door".to_string()
                }
            } else if let Some(inmate) = c.inmate {
                format!("the {}", describe_inmate(inmate))
            } else {
                "an empty cell".to_string()
            };
            let where_str = if n == OFFICE {
                "Interrogation Room".to_string()
            } else {
                format!("cell {}", cell_label(n))
            };
            out.push(Candidate {
                action: SimAction::Patrol { cell: n },
                label: format!("Go to {where_str} ({what})"),
                kind: None,
                points: None,
                extra: false,
            });
        }
        return out;
    }

    let here = &st.cells[st.position as usize];
    let (s_opts, s_len) = st.start_options();
    for &o in &s_opts[..s_len] {
        let target = &st.cells[o.cell as usize];
        let meal = if o.scratch {
            "the scratched door".to_string()
        } else {
            format!("the {}", describe_inmate(target.inmate.unwrap()))
        };
        out.push(Candidate {
            action: SimAction::Devour { to: o.cell },
            label: format!("Consume {meal} in {}", cell_label(o.cell)),
            kind: None,
            points: None,
            extra: false,
        });
    }
    if st.can_swap_now()
        && !here.sealed
        && let Some(inmate) = here.inmate
    {
        out.push(Candidate {
            action: SimAction::Swap,
            label: format!(
                "Replace the {} in {}",
                describe_inmate(inmate),
                cell_label(here.n)
            ),
            kind: None,
            points: None,
            extra: false,
        });
    }
    if st.can_hunt_now() {
        let (targets, tlen) = st.hunt_targets();
        for &n in &targets[..tlen] {
            if let Some(inmate) = st.cells[n as usize].inmate {
                out.push(Candidate {
                    action: SimAction::Hunt { to: n },
                    label: format!(
                        "Hunt: a Prey mark on the {} in {}",
                        describe_inmate(inmate),
                        cell_label(n)
                    ),
                    kind: None,
                    points: None,
                    extra: false,
                });
            }
        }
    }
    if st.can_open_early() {
        out.push(Candidate {
            action: SimAction::Early,
            label: "Deliver the heavy contract early".to_string(),
            kind: None,
            points: None,
            extra: false,
        });
    }
    let end_label = if st.mode == GameMode::Drill && st.day >= st.max_days {
        "Finish the run"
    } else {
        "End the day"
    };
    out.push(Candidate {
        action: SimAction::End,
        label: end_label.to_string(),
        kind: None,
        points: None,
        extra: false,
    });
    out
}

#[inline]
fn paired_se_from_stats(n: usize, sum: f64, sq: f64) -> f64 {
    if n < 2 {
        return f64::INFINITY;
    }
    let nf = n as f64;
    let m = sum / nf;
    ((sq / nf - m * m).max(0.0) / nf).sqrt()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptionRank {
    pub label: String,
    pub action: SimAction,
    pub kind: Option<ConvoyKind>,
    pub mean: f64,
    pub se: f64,
    pub gap_se: f64,
    pub n: usize,
    pub pass: f64,
    pub points: f64,
    pub best: f64,
    pub policy: bool,
    pub extra: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SolveSummary {
    pub done: bool,
    pub rounds: usize,
    pub goal: SolveGoal,
    pub options: Vec<OptionRank>,
}

pub struct PlayoutRow {
    pub label: String,
    pub action: SimAction,
    pub kind: Option<ConvoyKind>,
    pub extra: bool,
    pub policy: bool,
    pub n: usize,
    pub sum: f64,
    pub sq: f64,
    pub pairs: [(usize, f64, f64); 24],
    pub pass: usize,
    pub points: u64,
    pub best: u64,
}

pub fn summarize(
    rows: &[PlayoutRow],
    rounds: usize,
    target: usize,
    goal: SolveGoal,
) -> SolveSummary {
    let mut stats: Vec<(usize, f64, f64, usize)> = rows
        .iter()
        .enumerate()
        .map(|(idx, r)| {
            let n = r.n;
            let nf = n as f64;
            let mean = if n > 0 { r.sum / nf } else { 0.0 };
            let sd = if n > 1 {
                (r.sq / nf - mean * mean).max(0.0).sqrt()
            } else {
                0.0
            };
            let se = if n > 1 { sd / nf.sqrt() } else { 0.0 };
            (idx, mean, se, n)
        })
        .collect();

    stats.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| rows[b.0].best.cmp(&rows[a.0].best))
    });

    let top_idx = stats.first().map(|s| s.0).unwrap_or(0);
    let options = stats
        .into_iter()
        .map(|(idx, mean, se, n)| {
            let r = &rows[idx];
            let nf = n as f64;
            let gap_se = if idx == top_idx {
                0.0
            } else {
                let (pn, psum, psq) = rows[top_idx].pairs[idx];
                paired_se_from_stats(pn, psum, psq)
            };
            OptionRank {
                label: r.label.clone(),
                action: r.action,
                kind: r.kind,
                mean,
                se,
                gap_se,
                n,
                pass: if n > 0 { (r.pass as f64) / nf } else { 0.0 },
                points: if n > 0 { (r.points as f64) / nf } else { 0.0 },
                best: if n > 0 { (r.best as f64) / nf } else { 0.0 },
                policy: r.policy,
                extra: r.extra,
            }
        })
        .collect();

    SolveSummary {
        done: rounds >= target || rows.len() <= 1,
        rounds,
        goal,
        options,
    }
}

pub fn should_stop(result: &SolveSummary, min_rounds: usize) -> bool {
    let z = 3.0;
    if result.rounds < min_rounds || result.options.len() < 2 {
        return false;
    }
    let tie = if result.goal == SolveGoal::Contract {
        5.0
    } else {
        0.03
    };
    let lead = &result.options[0];
    let rest = &result.options[1..];
    rest.iter().all(|x| {
        let diff = lead.mean - x.mean;
        let ratio = diff / x.gap_se.max(1e-9);
        let is_clear = ratio >= z || (result.rounds >= 384 && ratio >= 2.0);
        let is_tied = diff + z * x.gap_se < tie
            || (diff <= 0.015 && x.gap_se < 0.02)
            || (result.rounds >= 384
                && (2.2 * x.gap_se < diff + tie
                    || (result.goal == SolveGoal::Rainbows
                        && x.gap_se <= 0.16
                        && diff <= 2.0 * x.gap_se)));
        is_clear || is_tied
    })
}

/// Zero-allocation early stopping check directly over `PlayoutRow` slice.
pub fn should_stop_rows(
    rows: &[PlayoutRow],
    rounds: usize,
    min_rounds: usize,
    goal: SolveGoal,
) -> bool {
    let z = 3.0;
    if rounds < min_rounds || rows.len() < 2 {
        return false;
    }
    let tie = if goal == SolveGoal::Contract {
        5.0
    } else {
        0.03
    };
    let mut top_idx = 0usize;
    let mut lead_mean = if rows[0].n > 0 {
        rows[0].sum / (rows[0].n as f64)
    } else {
        0.0
    };
    for (idx, r) in rows.iter().enumerate().skip(1) {
        let m = if r.n > 0 { r.sum / (r.n as f64) } else { 0.0 };
        let cmp = m
            .total_cmp(&lead_mean)
            .then_with(|| r.best.cmp(&rows[top_idx].best));
        if cmp.is_gt() {
            top_idx = idx;
            lead_mean = m;
        }
    }
    for (idx, r) in rows.iter().enumerate() {
        if idx == top_idx {
            continue;
        }
        let m = if r.n > 0 { r.sum / (r.n as f64) } else { 0.0 };
        let (pn, psum, psq) = rows[top_idx].pairs[idx];
        let gap_se = paired_se_from_stats(pn, psum, psq);
        let diff = lead_mean - m;
        let ratio = diff / gap_se.max(1e-9);
        let is_clear = ratio >= z || (rounds >= 384 && ratio >= 2.0);
        let is_tied = diff + z * gap_se < tie
            || (diff <= 0.015 && gap_se < 0.02)
            || (rounds >= 384
                && (2.2 * gap_se < diff + tie
                    || (goal == SolveGoal::Rainbows && gap_se <= 0.16 && diff <= 2.0 * gap_se)));
        if !is_clear && !is_tied {
            return false;
        }
    }
    true
}

/// High-speed multi-threaded Monte Carlo playout solver using `rayon`.
pub fn solve_parallel(
    st: &SimState,
    max_rollouts: usize,
    budget_ms: u64,
    seed: u32,
    early_stop: bool,
) -> SolveSummary {
    let list = candidates(st);
    let mut pick_rng = seed ^ 0x5bd1e995;
    let pick = if !list.is_empty() {
        decide(st, None, &mut pick_rng)
    } else {
        None
    };
    solve_parallel_with_candidates(st, list, pick, max_rollouts, budget_ms, seed, early_stop)
}

pub fn solve_parallel_with_candidates(
    st: &SimState,
    list: Vec<Candidate>,
    pick: Option<SimAction>,
    max_rollouts: usize,
    budget_ms: u64,
    seed: u32,
    early_stop: bool,
) -> SolveSummary {
    let base = *st;
    let is_challenge = base.mode == GameMode::Challenge;
    let goal = if is_challenge {
        SolveGoal::Contract
    } else {
        SolveGoal::Rainbows
    };

    let mut rows: Vec<PlayoutRow> = list
        .into_iter()
        .take(24)
        .map(|c| PlayoutRow {
            label: c.label,
            action: c.action,
            kind: c.kind,
            extra: c.extra,
            policy: pick.as_ref().is_some_and(|p| c.action.same_action(p)),
            n: 0,
            sum: 0.0,
            sq: 0.0,
            pairs: [(0, 0.0, 0.0); 24],
            pass: 0,
            points: 0,
            best: 0,
        })
        .collect();

    if rows.len() <= 1 || max_rollouts == 0 {
        return summarize(&rows, 0, max_rollouts, goal);
    }

    let num_rows = rows.len();
    let start = Instant::now();
    let batch_size = 128usize.min(max_rollouts);
    let mut rounds_done = 0usize;
    let mut stopped = false;
    let mut batch_results = [[(0.0f64, 0usize, 0u64, 0u64); 24]; 128];

    while rounds_done < max_rollouts && !stopped {
        let count = batch_size.min(max_rollouts - rounds_done);
        let round_base = rounds_done;

        // Run `count` rounds in parallel across all CPU cores with zero heap allocations!
        // Each round `r` uses deterministic common random numbers across all candidate actions,
        // and `hide_future` is computed once per round and shared across all candidates.
        batch_results[..count]
            .par_iter_mut()
            .enumerate()
            .for_each(|(offset, round_out)| {
                let r_idx = (round_base + offset) as u32;
                let mut seed_gen = seed.wrapping_add(r_idx.wrapping_mul(0x9e3779b1));
                let round_seed = (next_rand(&mut seed_gen) * 4294967296.0) as u32;
                let mut hidden_base = base;
                hide_future(&mut hidden_base, round_seed);

                // Couple exchangeable unrevealed sealed patrol targets so all locked-door candidates
                // reveal the exact same hidden inmate on today's step (dramatically reducing paired variance).
                let ref_sealed_cell = rows.iter().find_map(|r| match r.action {
                    SimAction::Patrol { cell: c } => {
                        let cell = &base.cells[c as usize];
                        if cell.sealed
                            && cell.known == 0
                            && !cell.scratch
                            && cell.nest_due.is_none()
                        {
                            Some(c as usize)
                        } else {
                            None
                        }
                    }
                    _ => None,
                });

                for (i, row) in rows.iter().enumerate() {
                    let mut trial = hidden_base;
                    if let (Some(ref_c), SimAction::Patrol { cell: c }) =
                        (ref_sealed_cell, row.action)
                    {
                        let c_idx = c as usize;
                        if c_idx != ref_c {
                            let tc = &base.cells[c_idx];
                            let rc = &base.cells[ref_c];
                            if tc.sealed
                                && tc.known == 0
                                && !tc.scratch
                                && tc.nest_due.is_none()
                                && tc.at_least == rc.at_least
                            {
                                let tmp = trial.cells[ref_c].hidden;
                                trial.cells[ref_c].hidden = trial.cells[c_idx].hidden;
                                trial.cells[c_idx].hidden = tmp;
                            }
                        }
                    }
                    if !apply_action(&mut trial, &row.action) {
                        round_out[i] = (f64::NAN, 0, 0, 0);
                        continue;
                    }
                    let mut roll_rng = round_seed ^ 0x9e3779b9;
                    playout(&mut trial, None, &mut roll_rng);
                    round_out[i] = if is_challenge {
                        let ch = trial.challenge.as_ref().unwrap();
                        let passed = ch.settled && ch.passed;
                        let val = (if passed { PASS_WEIGHT } else { 0.0 }) + (ch.score as f64);
                        (val, usize::from(passed), ch.score as u64, 0)
                    } else {
                        let val = trial.vault_len as f64;
                        let mut max_tier = if trial.vault_len > 0 { RAINBOW } else { 0 };
                        for c in &trial.cells {
                            if !c.sealed
                                && let Some(inm) = c.inmate
                                && inm.tier > max_tier
                            {
                                max_tier = inm.tier;
                            }
                        }
                        (val, 0, 0, max_tier as u64)
                    };
                }
            });

        for round_out in &batch_results[..count] {
            for i in 0..num_rows {
                let (val_i, pass, pts, best) = round_out[i];
                let row = &mut rows[i];
                row.pass += pass;
                row.points += pts;
                row.best += best;
                if !val_i.is_nan() {
                    row.n += 1;
                    row.sum += val_i;
                    row.sq += val_i * val_i;
                    for j in 0..num_rows {
                        if i == j {
                            continue;
                        }
                        let val_j = round_out[j].0;
                        let d = val_i - val_j;
                        if !d.is_nan() {
                            let p = &mut row.pairs[j];
                            p.0 += 1;
                            p.1 += d;
                            p.2 += d * d;
                        }
                    }
                }
            }
        }
        rounds_done += count;

        if early_stop
            && rounds_done < max_rollouts
            && should_stop_rows(&rows, rounds_done, 100, goal)
        {
            stopped = true;
        }
        if budget_ms > 0 && start.elapsed().as_millis() as u64 >= budget_ms {
            break;
        }
    }

    let mut res = summarize(&rows, rounds_done, max_rollouts, goal);
    if stopped {
        res.done = true;
    }
    res
}

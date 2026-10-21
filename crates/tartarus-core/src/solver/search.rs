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

const _: () = {
    assert!(PASS_WEIGHT > 0.0);
    assert!(RAINBOW > SUPREME);
    assert!((OFFICE as usize) < 13);
};

fn decide_pending(st: &SimState, p: &'static SolverParams, rng: &mut u32) -> SimAction {
    assert!(st.pending_len > 0);
    assert!((st.pending_len as usize) <= st.pending.len());
    let choice = &st.pending[0];
    let opts = choice.slice();
    assert!(!opts.is_empty());
    if choice.kind == PendingKind::Gift {
        return SimAction::Choose(pick_gift(st, p, opts, rng));
    }
    let mut tool_opts = [ToolId::Revenge; 4];
    let mut tlen = 0;
    for &c in opts {
        if let ChoiceId::Tool(t) = c {
            assert!(tlen < tool_opts.len());
            tool_opts[tlen] = t;
            tlen += 1;
        }
    }
    if choice.kind == PendingKind::Tool && wants_reroll(st, p, &tool_opts[..tlen]) {
        return SimAction::Reroll;
    }
    let picked = pick_tool(st, p, &tool_opts[..tlen]);
    SimAction::Choose(ChoiceId::Tool(picked))
}

fn decide_chain(st: &SimState, ctx: &mut Context, p: &SolverParams) -> SimAction {
    assert!(st.chaining_eater.is_some());
    assert!(p.stop_margin.is_finite());
    let (options, olen) = devour_scores(st, ctx);
    assert!(olen <= options.len());
    let stop_thresh = if st.day >= st.days_max {
        0.0
    } else {
        p.stop_margin
    };
    if ctx.chain_stop && (olen == 0 || options[0].score < stop_thresh) {
        return SimAction::StopChain;
    }
    if olen > 0 {
        SimAction::Chain {
            to: options[0].cell,
        }
    } else {
        SimAction::End
    }
}

fn decide_execution_event(
    st: &SimState,
    ctx: &mut Context,
    p: &SolverParams,
    cell_n: u8,
) -> SimAction {
    assert!((cell_n as usize) < st.cells.len());
    assert!(p.exec_margin.is_finite());
    let Some(exec) = st.cells[cell_n as usize].inmate else {
        return SimAction::Event { accept: false };
    };
    let (mut comp, crowd) = composition(st, cell_n as i16);
    let mut ev_val = execution_value(st, ctx, exec.tier, exec.beast.idx() as u8, &comp);
    if ev_val != 0.0 && st.can_devour_now() && exec.tier < RAINBOW {
        let same = st.params.execution_same_tier;
        let prey = exec.beast.primary_prey().idx();
        let t_max = if same {
            exec.tier as i16
        } else {
            (exec.tier as i16) - 1
        };
        let mut comp_after = comp;
        let mut count = 0u8;
        for t in 0..=t_max {
            if same && t != (exec.tier as i16) {
                continue;
            }
            let idx = (t as usize) * 4 + prey;
            assert!(idx < comp_after.len());
            count += comp_after[idx];
            comp_after[idx] = 0;
        }
        if count > 0 {
            let (bm, _) = best_meal(ctx, exec.tier, exec.beast.idx() as u8, &mut comp, crowd);
            let to = (exec.tier + count).min(RAINBOW);
            let mut post_dev = 0.0;
            if to < RAINBOW {
                let rem_crowd = crowd.saturating_sub(count as usize);
                for b in 0..4u8 {
                    let (pbm, _) = best_meal(ctx, to, b, &mut comp_after, rem_crowd);
                    if pbm > 0.0 {
                        post_dev += 0.25 * pbm;
                    }
                }
            }
            ev_val += post_dev;
            if bm > 0.0 && ev_val < bm {
                return SimAction::Event { accept: false };
            }
        }
    }
    SimAction::Event {
        accept: ev_val > p.exec_margin,
    }
}

fn decide_event(
    st: &SimState,
    ctx: &mut Context,
    p: &SolverParams,
    kind: EventKind,
    cell_n: u8,
) -> SimAction {
    assert_eq!(st.phase, Phase::Event);
    assert!((cell_n as usize) < st.cells.len());
    if kind == EventKind::Chain {
        return SimAction::Event {
            accept: p.accept_chain,
        };
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
        return SimAction::Event { accept: true };
    }
    let Some(exec) = st.cells[cell_n as usize].inmate else {
        return SimAction::Event { accept: false };
    };
    if kind == EventKind::Trickster {
        return SimAction::Event {
            accept: exec.tier < SUPREME,
        };
    }
    decide_execution_event(st, ctx, p, cell_n)
}

fn decide_action_phase(st: &SimState, ctx: &mut Context, p: &SolverParams) -> SimAction {
    assert!((st.position as usize) < st.cells.len());
    assert!(p.swap_margin.is_finite());
    let swap_gain = swap_gain_here(st, ctx);
    let hunt = best_hunt(st, ctx);
    if st.can_devour_now() {
        let (devours, dlen) = devour_scores(st, ctx);
        assert!(dlen <= devours.len());
        let dev_min = if st.day >= st.days_max {
            0.0
        } else {
            p.devour_min
        };
        if dlen > 0 && devours[0].score >= dev_min {
            if let Some((h_cell, h_val)) = hunt {
                // Under CageFight, Hunt is a 100% free action before Devour
                // if the Hunt target isn't the meal being eaten!
                if st.cage_fight() && h_val > 0.0 && h_cell != devours[0].cell {
                    return SimAction::Hunt { to: h_cell };
                }
                if h_val > devours[0].score && h_val > swap_gain {
                    return SimAction::Hunt { to: h_cell };
                }
            }
            if swap_gain > 0.0 && swap_gain > devours[0].score + p.swap_margin {
                return SimAction::Swap;
            }
            return SimAction::Devour {
                to: devours[0].cell,
            };
        }
    }
    if let Some((h_cell, h_val)) = hunt
        && h_val > 0.0
        && h_val > swap_gain
    {
        return SimAction::Hunt { to: h_cell };
    }
    if swap_gain > 0.0 {
        return SimAction::Swap;
    }
    if st.challenge.is_some() && wants_early(st, ctx) {
        return SimAction::Early;
    }
    SimAction::End
}

fn decide_office(st: &SimState, ctx: &Context, p: &SolverParams) -> SimAction {
    assert_eq!(st.phase, Phase::Office);
    assert!((st.position as usize) < st.cells.len());
    if let Some(ov) = p.office_override {
        return SimAction::Office { choice: ov };
    }
    let (comp, _) = composition(st, -1);
    let office = office_values(st, ctx, &comp);
    let choice = if office.bestow >= office.instigate {
        OfficeChoice::Bestow
    } else {
        OfficeChoice::Instigate
    };
    SimAction::Office { choice }
}

pub fn decide(
    st: &SimState,
    params: Option<&'static SolverParams>,
    rng: &mut u32,
) -> Option<SimAction> {
    assert!((st.position as usize) < st.cells.len());
    assert!((st.pending_len as usize) <= st.pending.len());
    if st.phase == Phase::Ended {
        return None;
    }
    let p = params.unwrap_or_else(|| SolverParams::for_state(st));
    let mut ctx = Context::new(st, p);

    if st.pending_len > 0 {
        return Some(decide_pending(st, p, rng));
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
        return Some(decide_chain(st, &mut ctx, p));
    }

    if st.phase == Phase::Office {
        return Some(decide_office(st, &ctx, p));
    }

    if st.phase == Phase::Event
        && let Some((kind, cell_n)) = st.active_event
    {
        return Some(decide_event(st, &mut ctx, p, kind, cell_n));
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

    Some(decide_action_phase(st, &mut ctx, p))
}

pub fn playout(st: &mut SimState, params: Option<&'static SolverParams>, rng: &mut u32) {
    const STEPS_MAX: usize = 2000;
    assert!((st.position as usize) < st.cells.len());
    assert!((st.pending_len as usize) <= st.pending.len());
    for step in 0..STEPS_MAX {
        assert!(step < STEPS_MAX);
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

fn hide_future_objective(st: &mut SimState, rng: &mut u32) {
    assert!((st.vault_len as usize) <= st.vault.len());
    assert!(st.stats.best <= RAINBOW);
    if st.objective_id != ObjectiveId::Other || st.milestone || !st.params.rewards {
        return;
    }
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
            assert!(vlen < valid.len());
            valid[vlen] = obj;
            vlen += 1;
        }
    }
    if vlen > 0 {
        let idx = (next_rand(rng) * (vlen as f64)).floor() as usize;
        let picked = valid[idx.min(vlen - 1)];
        st.objective_id = picked;
        st.objective_progress = match picked {
            ObjectiveId::Doors => (st.stats.opened as u8).min(picked.need().saturating_sub(1)),
            ObjectiveId::Consume => {
                ((st.stats.devours + st.stats.chains) as u8).min(picked.need().saturating_sub(1))
            }
            _ => st.objective_progress,
        };
    }
}

pub fn hide_future(st: &mut SimState, seed: u32) {
    assert!((st.position as usize) < st.cells.len());
    let mut rng = seed;
    st.rng = (next_rand(&mut rng) * 4294967296.0) as u32;
    hide_future_objective(st, &mut rng);

    let pool_len = (st.event_pool_len as usize).min(st.event_pool.len());
    let mut pool = [EventKind::Chain; 17];
    assert!(pool_len <= pool.len());
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
                assert!(b_idx.min(3) < BEASTS.len());
                h.beast = BEASTS[b_idx.min(3)];
            }
        } else {
            c.hidden = None;
        }
    }
}

fn candidates_pending(st: &SimState) -> Vec<Candidate> {
    assert!(st.pending_len > 0);
    assert!((st.pending_len as usize) <= st.pending.len());
    let mut out = Vec::new();
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
    out
}

fn candidates_convoy(st: &SimState, window_early: bool) -> Vec<Candidate> {
    assert!(st.challenge.is_some());
    assert!((st.vault_len as usize) <= st.vault.len());
    let mut out = convoy_options(st, None);
    if window_early {
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
    out
}

fn candidates_chain(st: &SimState) -> Vec<Candidate> {
    assert!(st.chaining_eater.is_some());
    let mut out = Vec::new();
    let (opts, olen) = st.chain_options();
    assert!(olen <= opts.len());
    for &o in &opts[..olen] {
        assert!((o.cell as usize) < st.cells.len());
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
    out
}

fn candidates_office() -> Vec<Candidate> {
    let out = vec![
        Candidate {
            action: SimAction::Office {
                choice: OfficeChoice::Bestow,
            },
            label: "Bestow: 1 random inmate rises a tier".to_string(),
            kind: None,
            points: None,
            extra: false,
        },
        Candidate {
            action: SimAction::Office {
                choice: OfficeChoice::Instigate,
            },
            label: format!("Instigate: {INSTIGATE_EVENTS} new special events"),
            kind: None,
            points: None,
            extra: false,
        },
    ];
    assert_eq!(out.len(), 2);
    assert!(!out[0].label.is_empty());
    out
}

fn candidates_event(kind: EventKind) -> Vec<Candidate> {
    let prefix = if matches!(kind, EventKind::Chain | EventKind::Execution) {
        "the "
    } else {
        ""
    };
    let name = format!("{prefix}{}", kind.name());
    assert!(!name.is_empty());
    let mut out = Vec::new();
    if kind == EventKind::Hunt {
        out.push(Candidate {
            action: SimAction::Event { accept: true },
            label: format!("Confirm {name}"),
            kind: None,
            points: None,
            extra: false,
        });
        assert_eq!(out.len(), 1);
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
    assert_eq!(out.len(), 2);
    out
}

fn candidates_patrol(st: &SimState) -> Vec<Candidate> {
    assert_eq!(st.phase, Phase::Patrol);
    assert!((st.position as usize) < st.cells.len());
    let mut out = Vec::new();
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
        assert!((n as usize) < st.cells.len());
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
    out
}

fn push_hunt_candidates(st: &SimState, out: &mut Vec<Candidate>) {
    assert!((st.position as usize) < st.cells.len());
    if !st.can_hunt_now() {
        return;
    }
    let (targets, tlen) = st.hunt_targets();
    assert!(tlen <= targets.len());
    for &n in &targets[..tlen] {
        assert!((n as usize) < st.cells.len());
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

fn candidates_action_phase(st: &SimState) -> Vec<Candidate> {
    assert!((st.position as usize) < st.cells.len());
    let mut out = Vec::new();
    let here = &st.cells[st.position as usize];
    let (s_opts, s_len) = st.start_options();
    assert!(s_len <= s_opts.len());
    for &o in &s_opts[..s_len] {
        assert!((o.cell as usize) < st.cells.len());
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
    push_hunt_candidates(st, &mut out);
    if st.can_open_early() {
        out.push(Candidate {
            action: SimAction::Early,
            label: "Deliver the heavy contract early".to_string(),
            kind: None,
            points: None,
            extra: false,
        });
    }
    let end_label = if st.mode == GameMode::Drill && st.day >= st.days_max {
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

pub fn candidates(st: &SimState) -> Vec<Candidate> {
    assert!((st.position as usize) < st.cells.len());
    assert!((st.pending_len as usize) <= st.pending.len());
    if st.phase == Phase::Ended {
        return Vec::new();
    }
    if st.pending_len > 0 {
        return candidates_pending(st);
    }
    if let Some(ch) = st.challenge.as_ref()
        && ch.window_open
    {
        return candidates_convoy(st, ch.window_early);
    }
    if st.chaining_eater.is_some() {
        return candidates_chain(st);
    }
    if st.phase == Phase::Office {
        return candidates_office();
    }
    if st.phase == Phase::Event
        && let Some((kind, _)) = st.active_event
    {
        return candidates_event(kind);
    }
    if st.phase == Phase::Patrol {
        return candidates_patrol(st);
    }
    candidates_action_phase(st)
}

#[inline]
fn paired_se_from_stats(n: usize, sum: f64, sq: f64) -> f64 {
    assert!(sum.is_finite());
    assert!(sq.is_finite());
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

fn build_option_rank(
    rows: &[PlayoutRow],
    top_idx: usize,
    stat: (usize, f64, f64, usize),
) -> OptionRank {
    let (idx, mean, se, n) = stat;
    assert!(idx < rows.len());
    assert!(top_idx < rows.len());
    let r = &rows[idx];
    let nf = n as f64;
    let gap_se = if idx == top_idx {
        0.0
    } else {
        assert!(idx < rows[top_idx].pairs.len());
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
}

pub fn summarize(
    rows: &[PlayoutRow],
    rounds: usize,
    target: usize,
    goal: SolveGoal,
) -> SolveSummary {
    assert!(rows.len() <= 24);
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
    let options: Vec<OptionRank> = stats
        .into_iter()
        .map(|stat| build_option_rank(rows, top_idx, stat))
        .collect();
    assert_eq!(options.len(), rows.len());

    SolveSummary {
        done: rounds >= target || rows.len() <= 1,
        rounds,
        goal,
        options,
    }
}

#[inline]
fn is_pair_resolved(diff: f64, gap_se: f64, rounds: usize, goal: SolveGoal) -> bool {
    assert!(diff.is_finite());
    assert!(!gap_se.is_nan());
    let z = 3.0;
    let tie = if goal == SolveGoal::Contract {
        5.0
    } else {
        0.03
    };
    let ratio = diff / gap_se.max(1e-9);
    let is_clear = ratio >= z || (rounds >= 384 && ratio >= 2.0);
    let is_tied = diff + z * gap_se < tie
        || (diff <= 0.015 && gap_se < 0.02)
        || (rounds >= 384
            && (2.2 * gap_se < diff + tie
                || (goal == SolveGoal::Rainbows && gap_se <= 0.16 && diff <= 2.0 * gap_se)));
    is_clear || is_tied
}

pub fn should_stop(result: &SolveSummary, rounds_min: usize) -> bool {
    if result.rounds < rounds_min || result.options.len() < 2 {
        return false;
    }
    assert!(result.rounds >= rounds_min);
    assert!(result.options.len() >= 2);
    let lead = &result.options[0];
    let rest = &result.options[1..];
    rest.iter()
        .all(|x| is_pair_resolved(lead.mean - x.mean, x.gap_se, result.rounds, result.goal))
}

/// Zero-allocation early stopping check directly over `PlayoutRow` slice.
pub fn should_stop_rows(
    rows: &[PlayoutRow],
    rounds: usize,
    rounds_min: usize,
    goal: SolveGoal,
) -> bool {
    if rounds < rounds_min || rows.len() < 2 {
        return false;
    }
    assert!(rounds >= rounds_min);
    assert!(rows.len() >= 2 && rows.len() <= 24);
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
        if !is_pair_resolved(lead_mean - m, gap_se, rounds, goal) {
            return false;
        }
    }
    true
}

/// High-speed multi-threaded Monte Carlo playout solver using `rayon`.
pub fn solve_parallel(
    st: &SimState,
    rollouts_max: usize,
    budget_ms: u64,
    seed: u32,
    early_stop: bool,
) -> SolveSummary {
    assert!((st.position as usize) < st.cells.len());
    assert!((st.pending_len as usize) <= st.pending.len());
    let list = candidates(st);
    let mut pick_rng = seed ^ 0x5bd1e995;
    let pick = if !list.is_empty() {
        decide(st, None, &mut pick_rng)
    } else {
        None
    };
    solve_parallel_with_candidates(st, list, pick, rollouts_max, budget_ms, seed, early_stop)
}

fn init_playout_rows(list: Vec<Candidate>, pick: Option<SimAction>) -> Vec<PlayoutRow> {
    let rows: Vec<PlayoutRow> = list
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
    assert!(rows.len() <= 24);
    assert!(rows.iter().all(|r| r.n == 0));
    rows
}

fn eval_trial_outcome(trial: &SimState, is_challenge: bool) -> (f64, usize, u64, u64) {
    assert!((trial.position as usize) < trial.cells.len());
    assert!((trial.vault_len as usize) <= trial.vault.len());
    if is_challenge {
        let ch = trial.challenge.as_ref().unwrap();
        let passed = ch.settled && ch.passed;
        let val = (if passed { PASS_WEIGHT } else { 0.0 }) + (ch.score as f64);
        (val, usize::from(passed), ch.score as u64, 0)
    } else {
        let val = trial.vault_len as f64;
        let mut tier_max = if trial.vault_len > 0 { RAINBOW } else { 0 };
        for c in &trial.cells {
            if !c.sealed
                && let Some(inm) = c.inmate
                && inm.tier > tier_max
            {
                tier_max = inm.tier;
            }
        }
        (val, 0, 0, tier_max as u64)
    }
}

fn run_playout_round(
    base: &SimState,
    rows: &[PlayoutRow],
    is_challenge: bool,
    seed: u32,
    r_idx: u32,
    round_out: &mut [(f64, usize, u64, u64); 24],
) {
    assert!(rows.len() <= round_out.len());
    let mut seed_gen = seed.wrapping_add(r_idx.wrapping_mul(0x9e3779b1));
    let round_seed = (next_rand(&mut seed_gen) * 4294967296.0) as u32;
    let mut hidden_base = *base;
    hide_future(&mut hidden_base, round_seed);

    // Couple exchangeable unrevealed sealed patrol targets so all locked-door
    // candidates reveal the exact same hidden inmate on today's step.
    let ref_sealed_cell = rows.iter().find_map(|r| match r.action {
        SimAction::Patrol { cell: c } => {
            let cell = &base.cells[c as usize];
            if cell.sealed && cell.known == 0 && !cell.scratch && cell.nest_due.is_none() {
                Some(c as usize)
            } else {
                None
            }
        }
        _ => None,
    });

    for (i, row) in rows.iter().enumerate() {
        let mut trial = hidden_base;
        if let (Some(ref_c), SimAction::Patrol { cell: c }) = (ref_sealed_cell, row.action) {
            let c_idx = c as usize;
            assert!(c_idx < trial.cells.len());
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
        round_out[i] = eval_trial_outcome(&trial, is_challenge);
    }
}

fn accumulate_batch(rows: &mut [PlayoutRow], batch: &[[(f64, usize, u64, u64); 24]]) {
    let num_rows = rows.len();
    assert!(num_rows <= 24);
    assert!(batch.len() <= 128);
    for round_out in batch {
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
                for (j, pair) in row.pairs.iter_mut().enumerate().take(num_rows) {
                    if i == j {
                        continue;
                    }
                    let val_j = round_out[j].0;
                    let d = val_i - val_j;
                    if !d.is_nan() {
                        pair.0 += 1;
                        pair.1 += d;
                        pair.2 += d * d;
                    }
                }
            }
        }
    }
}

pub fn solve_parallel_with_candidates(
    st: &SimState,
    list: Vec<Candidate>,
    pick: Option<SimAction>,
    rollouts_max: usize,
    budget_ms: u64,
    seed: u32,
    early_stop: bool,
) -> SolveSummary {
    assert!((st.position as usize) < st.cells.len());
    let base = *st;
    let is_challenge = base.mode == GameMode::Challenge;
    let goal = if is_challenge {
        SolveGoal::Contract
    } else {
        SolveGoal::Rainbows
    };

    let mut rows = init_playout_rows(list, pick);
    if rows.len() <= 1 || rollouts_max == 0 {
        return summarize(&rows, 0, rollouts_max, goal);
    }

    let start = Instant::now();
    let batch_size = 128usize.min(rollouts_max);
    let batches_max = rollouts_max.div_ceil(batch_size);
    assert!(batches_max > 0);
    let mut rounds_done = 0usize;
    let mut stopped = false;
    let mut batch_results = [[(0.0f64, 0usize, 0u64, 0u64); 24]; 128];

    for batch_idx in 0..batches_max {
        assert!(batch_idx < batches_max);
        if rounds_done >= rollouts_max || stopped {
            break;
        }
        let count = batch_size.min(rollouts_max - rounds_done);
        let round_base = rounds_done;

        batch_results[..count]
            .par_iter_mut()
            .enumerate()
            .for_each(|(offset, round_out)| {
                let r_idx = (round_base + offset) as u32;
                run_playout_round(&base, &rows, is_challenge, seed, r_idx, round_out);
            });

        accumulate_batch(&mut rows, &batch_results[..count]);
        rounds_done += count;

        if early_stop
            && rounds_done < rollouts_max
            && should_stop_rows(&rows, rounds_done, 100, goal)
        {
            stopped = true;
        }
        if budget_ms > 0 && start.elapsed().as_millis() as u64 >= budget_ms {
            break;
        }
    }

    let mut res = summarize(&rows, rounds_done, rollouts_max, goal);
    if stopped {
        res.done = true;
    }
    res
}

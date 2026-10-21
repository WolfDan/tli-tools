use crate::engine::{
    Beast, CELL_COUNT, CHALLENGE_DAYS, Contract, ContractId, EscortKind, PRESSURE, RAINBOW,
    SUPREME, SimState, cell_label, describe_inmate, plural,
};
use serde::{Deserialize, Serialize};

use super::chain_dp::Context;
use super::params::{SimAction, SolverParams};

const WALK_ITERATIONS_MAX: usize = 1_000_000;

const _: () = {
    assert!(CELL_COUNT == 16);
    assert!(RAINBOW == 5);
};

#[derive(Copy, Clone, Debug, Default)]
struct CargoRef {
    is_vault: bool,
    cell: u8,
    vault_id: u16,
    last: u8,
}

#[derive(Copy, Clone, Debug, Default)]
struct CargoGroup {
    tier: u8,
    beast: u8,
    refs: [CargoRef; 16],
    refs_len: u8,
}

fn push_cargo_ref(
    groups: &mut [CargoGroup; 24],
    glen: &mut usize,
    key_to_idx: &mut [u8; 24],
    tier: u8,
    beast: Beast,
    r: CargoRef,
) {
    assert!(tier <= RAINBOW);
    assert!(*glen <= 24);
    let key = (tier as usize) * 4 + beast.idx();
    assert!(key < 24);
    let g_idx = if key_to_idx[key] != u8::MAX {
        key_to_idx[key] as usize
    } else {
        let idx = *glen;
        *glen += 1;
        key_to_idx[key] = idx as u8;
        groups[idx].tier = tier;
        groups[idx].beast = beast.idx() as u8;
        idx
    };
    let g = &mut groups[g_idx];
    assert!((g.refs_len as usize) < g.refs.len());
    g.refs[g.refs_len as usize] = r;
    g.refs_len += 1;
}

fn cargo_groups(st: &SimState) -> ([CargoGroup; 24], usize) {
    assert!(st.cells.len() == CELL_COUNT);
    let mut groups = [CargoGroup::default(); 24];
    let mut glen = 0usize;
    let mut key_to_idx = [u8::MAX; 24];

    for c in &st.cells {
        if c.sealed {
            continue;
        }
        if let Some(inmate) = c.inmate {
            let last = u8::from(c.ivory || st.link_of(inmate.id).is_some());
            let r = CargoRef {
                is_vault: false,
                cell: c.n,
                vault_id: 0,
                last,
            };
            push_cargo_ref(
                &mut groups,
                &mut glen,
                &mut key_to_idx,
                inmate.tier,
                inmate.beast,
                r,
            );
        }
    }
    for v in st.vault_slice() {
        let r = CargoRef {
            is_vault: true,
            cell: 0,
            vault_id: v.id,
            last: 0,
        };
        push_cargo_ref(&mut groups, &mut glen, &mut key_to_idx, RAINBOW, v.beast, r);
    }
    for g in &mut groups[..glen] {
        g.refs[..g.refs_len as usize].sort_by_key(|r| r.last);
    }
    assert!(glen <= 24);
    (groups, glen)
}

#[derive(Copy, Clone, Debug)]
pub struct ConvoyPlan {
    pub escort: EscortKind,
    pub cost: u8,
    pub points: u16,
    pub pressure: u16,
    pub kept: f64,
    pub size: u8,
    pub score: f64,
    pub cell_mask: u16,
    pub vault_ids: [u16; 10],
    pub vault_len: u8,
}

struct ConvoyWalkCtx<'a, F> {
    glen: usize,
    groups: &'a [CargoGroup; 24],
    cap: u16,
    bounty: i8,
    contract: Contract,
    escorts: &'a [EscortKind],
    ctx: &'a Context,
    rate: &'a F,
}

struct ConvoyLeafTally {
    size: u8,
    mask: u8,
    elite: u8,
    low: u8,
    top: u8,
    leg: [u8; 4],
    below: [u8; 4],
    points: u16,
    press: u16,
    kept: f64,
}

fn tally_convoy_leaf<F>(w: &ConvoyWalkCtx<'_, F>, counts: &[u8; 24]) -> ConvoyLeafTally {
    assert!(w.glen <= 24);
    assert!((-1..4).contains(&w.bounty));
    let mut t = ConvoyLeafTally {
        size: 0,
        mask: 0,
        elite: 0,
        low: 0,
        top: 0,
        leg: [0; 4],
        below: [0; 4],
        points: 0,
        press: 0,
        kept: 0.0,
    };
    for idx in 0..w.glen {
        let k = counts[idx];
        if k == 0 {
            continue;
        }
        let g = &w.groups[idx];
        t.size += k;
        t.mask |= 1 << g.beast;
        if g.tier == 2 || g.tier == 3 {
            t.elite += k;
        }
        if g.tier <= 1 {
            t.low += k;
        }
        if g.tier >= SUPREME {
            t.top += k;
        }
        if g.tier == 3 {
            t.leg[g.beast as usize] += k;
        } else if g.tier < 3 {
            t.below[g.beast as usize] += k;
        }
        let bonus = if g.tier >= SUPREME && (g.beast as i8) == w.bounty {
            8u16
        } else {
            0u16
        };
        t.points += (k as u16) * ((1u16 << g.tier) + bonus);
        t.press += (k as u16) * PRESSURE[g.tier as usize];
        let v_t = if g.tier >= RAINBOW {
            w.ctx.r_val
        } else {
            w.ctx.v[g.tier as usize]
        };
        t.kept += (k as f64) * v_t;
    }
    t
}

fn contract_fits_tally(id: ContractId, t: &ConvoyLeafTally) -> bool {
    assert!(t.size > 0);
    let beasts = t.mask.count_ones();
    assert!(beasts <= 4);
    match id {
        ContractId::Mixed => t.size >= 3 && beasts >= 2,
        ContractId::Cohort => t.size >= 4,
        ContractId::Veteran => t.elite >= 1 && t.low >= 2,
        ContractId::Elite => t.elite >= 2 && beasts >= 2,
        ContractId::Kin => (0..4).any(|b| t.leg[b] >= 1 && t.below[b] >= 1),
        ContractId::Muster => t.elite >= 3,
        ContractId::Heavy | ContractId::Bounty => t.top >= 1,
    }
}

fn eval_convoy_leaf<F>(
    w: &ConvoyWalkCtx<'_, F>,
    counts: &[u8; 24],
    best: &mut Option<(ConvoyPlan, [u8; 24])>,
) where
    F: Fn(&ConvoyPlan) -> f64,
{
    assert!(w.glen <= 24);
    assert!(w.escorts.len() <= 2);
    let t = tally_convoy_leaf(w, counts);
    if t.size == 0 || !contract_fits_tally(w.contract.id, &t) {
        return;
    }
    for &escort in w.escorts {
        if t.press > escort.cap() {
            continue;
        }
        let mut plan = ConvoyPlan {
            escort,
            cost: escort.cost(),
            points: t.points,
            pressure: t.press,
            kept: t.kept,
            size: t.size,
            score: 0.0,
            cell_mask: 0,
            vault_ids: [0; 10],
            vault_len: 0,
        };
        let sc = (w.rate)(&plan);
        if sc > f64::NEG_INFINITY && (best.is_none() || sc > best.as_ref().unwrap().0.score) {
            plan.score = sc;
            *best = Some((plan, *counts));
        }
    }
}

fn walk_convoy_stack<F>(w: &ConvoyWalkCtx<'_, F>) -> Option<(ConvoyPlan, [u8; 24])>
where
    F: Fn(&ConvoyPlan) -> f64,
{
    assert!(w.glen <= 24);
    let mut counts = [0u8; 24];
    let mut next_k = [0u8; 24];
    let mut pressures = [0u16; 25];
    let mut best: Option<(ConvoyPlan, [u8; 24])> = None;
    let mut depth = 0usize;
    let mut iterations = 0usize;

    loop {
        iterations += 1;
        assert!(iterations <= WALK_ITERATIONS_MAX);
        if depth == w.glen {
            eval_convoy_leaf(w, &counts, &mut best);
            if depth == 0 {
                break;
            }
            depth -= 1;
            continue;
        }
        let g = &w.groups[depth];
        let k = next_k[depth];
        let next_p = pressures[depth] + (k as u16) * PRESSURE[g.tier as usize];
        if k <= g.refs_len && next_p <= w.cap {
            counts[depth] = k;
            next_k[depth] = k + 1;
            pressures[depth + 1] = next_p;
            depth += 1;
            if depth < w.glen {
                next_k[depth] = 0;
            }
        } else {
            counts[depth] = 0;
            next_k[depth] = 0;
            if depth == 0 {
                break;
            }
            depth -= 1;
        }
    }
    best
}

fn populate_convoy_refs(
    plan: &mut ConvoyPlan,
    groups: &[CargoGroup; 24],
    glen: usize,
    counts: &[u8; 24],
) {
    assert!(glen <= 24);
    assert!(plan.vault_len == 0);
    for (i, g) in groups[..glen].iter().enumerate() {
        let take = counts[i] as usize;
        for r in &g.refs[..take] {
            if r.is_vault {
                if (plan.vault_len as usize) < plan.vault_ids.len() {
                    plan.vault_ids[plan.vault_len as usize] = r.vault_id;
                    plan.vault_len += 1;
                }
            } else {
                plan.cell_mask |= 1u16 << r.cell;
            }
        }
    }
}

pub fn plan_cargo<F>(st: &SimState, ctx: &Context, rate: F) -> Option<ConvoyPlan>
where
    F: Fn(&ConvoyPlan) -> f64,
{
    assert!(st.cells.len() == CELL_COUNT);
    assert!(ctx.r_val.is_finite());
    let ch = st.challenge.as_ref()?;
    let contract = st.current_contract()?;
    if ch.vouchers < EscortKind::Light.cost() {
        return None;
    }
    let (groups, glen) = cargo_groups(st);
    let mut escorts = [EscortKind::Light; 2];
    let mut elen = 0;
    for e in [EscortKind::Light, EscortKind::Heavy] {
        if ch.vouchers >= e.cost() {
            escorts[elen] = e;
            elen += 1;
        }
    }
    let cap = escorts[..elen].iter().map(|e| e.cap()).max().unwrap_or(0);
    let bounty = contract.bounty.map(|b| b.idx() as i8).unwrap_or(-1);
    let w_ctx = ConvoyWalkCtx {
        glen,
        groups: &groups,
        cap,
        bounty,
        contract,
        escorts: &escorts[..elen],
        ctx,
        rate: &rate,
    };
    let (mut best_plan, best_counts) = walk_convoy_stack(&w_ctx)?;
    populate_convoy_refs(&mut best_plan, &groups, glen, &best_counts);
    Some(best_plan)
}

#[inline]
pub fn most_points(plan: &ConvoyPlan) -> f64 {
    assert!(plan.cost <= 10);
    assert!(plan.kept.is_finite());
    (plan.points as f64) - 0.01 * (plan.cost as f64)
}

pub fn convoy_plan(st: &SimState, ctx: &Context) -> Option<ConvoyPlan> {
    assert!(st.cells.len() == CELL_COUNT);
    assert!(ctx.r_val.is_finite());
    let ch = st.challenge.as_ref()?;
    let slot = ch.deliveries_len as usize;
    if slot >= 2 {
        return plan_cargo(st, ctx, most_points);
    }
    let p = ctx.p;
    let keep = if slot == 0 { p.keep1 } else { p.keep2 };
    let must = slot == 1 && !ch.deliveries[0].success;
    let need = if must {
        p.left_min
    } else {
        p.left_min.max(p.reserve + u8::from(slot == 0))
    };
    plan_cargo(st, ctx, |plan| {
        if ch.vouchers < plan.cost {
            return f64::NEG_INFINITY;
        }
        let left = ch.vouchers - plan.cost;
        if left < p.left_min || (plan.escort != EscortKind::Light && left < need) {
            return f64::NEG_INFINITY;
        }
        (plan.points as f64) - keep * plan.kept + 0.001 * (plan.points as f64)
            - 0.01 * (plan.cost as f64)
    })
}

pub fn wants_early(st: &SimState, ctx: &Context) -> bool {
    assert!(st.cells.len() == CELL_COUNT);
    assert!(ctx.p.early_points.is_finite());
    if !st.can_open_early() || st.day >= CHALLENGE_DAYS || ctx.p.early_points >= 1e9 {
        return false;
    }
    let Some(plan) = plan_cargo(st, ctx, most_points) else {
        return false;
    };
    if ctx.p.early_points >= 0.0 {
        return (plan.points as f64) >= ctx.p.early_points;
    }
    let ch = st.challenge.as_ref().unwrap();
    let cap = if ch.vouchers >= EscortKind::Heavy.cost() {
        EscortKind::Heavy.cap()
    } else {
        EscortKind::Light.cap()
    };
    cap.saturating_sub(plan.pressure) < PRESSURE[0]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvoyKind {
    Pick,
    Most,
    Light,
    Least,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolveGoal {
    Rainbows,
    Contract,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub action: SimAction,
    pub label: String,
    pub kind: Option<ConvoyKind>,
    pub points: Option<u16>,
    pub extra: bool,
}

pub fn cargo_label(
    st: &SimState,
    cell_mask: u16,
    vault_len: usize,
    escort: EscortKind,
    points: u16,
) -> String {
    assert!(st.cells.len() == CELL_COUNT);
    assert!(vault_len <= 16);
    let mut names = Vec::with_capacity((cell_mask.count_ones() as usize) + vault_len);
    for n in 0..16u8 {
        if (cell_mask & (1u16 << n)) != 0
            && let Some(inmate) = st.cells[n as usize].inmate
        {
            names.push(format!("{} {}", describe_inmate(inmate), cell_label(n)));
        }
    }
    for _ in 0..vault_len {
        names.push("a vault Rainbow".to_string());
    }
    format!(
        "Send {} by {} escort ({})",
        names.join(", "),
        escort.as_str(),
        plural(points as usize, "point")
    )
}

fn build_convoy_plan_variants(
    st: &SimState,
    ctx: &Context,
    vouchers: u8,
    heavy_next: bool,
) -> ([(ConvoyKind, Option<ConvoyPlan>); 4], usize) {
    assert!(st.cells.len() == CELL_COUNT);
    assert!(vouchers <= 20);
    let ok = |plan: &ConvoyPlan| !heavy_next || vouchers.saturating_sub(plan.cost) >= 1;
    let mut plans: [(ConvoyKind, Option<ConvoyPlan>); 4] = [
        (ConvoyKind::Pick, convoy_plan(st, ctx)),
        (ConvoyKind::Most, None),
        (ConvoyKind::Light, None),
        (ConvoyKind::Least, None),
    ];
    if !heavy_next {
        return (plans, 1);
    }
    plans[1] = (
        ConvoyKind::Most,
        plan_cargo(st, ctx, |pl| {
            if ok(pl) {
                most_points(pl)
            } else {
                f64::NEG_INFINITY
            }
        }),
    );
    plans[2] = (
        ConvoyKind::Light,
        plan_cargo(st, ctx, |pl| {
            if ok(pl) && pl.escort == EscortKind::Light {
                most_points(pl)
            } else {
                f64::NEG_INFINITY
            }
        }),
    );
    plans[3] = (
        ConvoyKind::Least,
        plan_cargo(st, ctx, |pl| {
            if ok(pl) && pl.escort == EscortKind::Light {
                0.001 * (pl.points as f64) - pl.kept
            } else {
                f64::NEG_INFINITY
            }
        }),
    );
    (plans, 4)
}

pub fn convoy_options(st: &SimState, params: Option<&'static SolverParams>) -> Vec<Candidate> {
    assert!(st.cells.len() == CELL_COUNT);
    let p = params.unwrap_or_else(|| SolverParams::for_state(st));
    assert!(p.tier_base.is_finite());
    let ctx = Context::new(st, p);
    let Some(ch) = st.challenge.as_ref() else {
        return Vec::new();
    };
    let heavy_next = ch.deliveries_len < 2;
    let (plans, plans_len) = build_convoy_plan_variants(st, &ctx, ch.vouchers, heavy_next);

    let mut out: Vec<Candidate> = Vec::with_capacity(plans_len);
    for &(kind, plan_opt) in &plans[..plans_len] {
        let Some(plan) = plan_opt else {
            continue;
        };
        let action = SimAction::Deliver {
            cell_mask: plan.cell_mask,
            vault_ids: plan.vault_ids,
            vault_len: plan.vault_len,
            escort: plan.escort,
        };
        if out.iter().any(|c| c.action.same_action(&action)) {
            continue;
        }
        let label = cargo_label(
            st,
            plan.cell_mask,
            plan.vault_len as usize,
            plan.escort,
            plan.points,
        );
        out.push(Candidate {
            action,
            label,
            kind: Some(kind),
            points: Some(plan.points),
            extra: false,
        });
    }
    out
}

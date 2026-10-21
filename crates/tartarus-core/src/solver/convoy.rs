use crate::engine::{
    Beast, CHALLENGE_DAYS, Contract, ContractId, EscortKind, PRESSURE, RAINBOW, SUPREME, SimState,
    cell_label, describe_inmate, plural,
};
use serde::{Deserialize, Serialize};

use super::chain_dp::Context;
use super::params::{SimAction, SolverParams};

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

fn cargo_groups(st: &SimState) -> ([CargoGroup; 24], usize) {
    let mut groups = [CargoGroup::default(); 24];
    let mut glen = 0usize;
    let mut key_to_idx = [u8::MAX; 24];

    let mut add = |tier: u8, beast: Beast, r: CargoRef| {
        let key = (tier as usize) * 4 + beast.idx();
        let g_idx = if key_to_idx[key] != u8::MAX {
            key_to_idx[key] as usize
        } else {
            let idx = glen;
            glen += 1;
            key_to_idx[key] = idx as u8;
            groups[idx].tier = tier;
            groups[idx].beast = beast.idx() as u8;
            idx
        };
        let g = &mut groups[g_idx];
        g.refs[g.refs_len as usize] = r;
        g.refs_len += 1;
    };

    for c in &st.cells {
        if c.sealed {
            continue;
        }
        if let Some(inmate) = c.inmate {
            let last = u8::from(c.ivory || st.link_of(inmate.id).is_some());
            add(
                inmate.tier,
                inmate.beast,
                CargoRef {
                    is_vault: false,
                    cell: c.n,
                    vault_id: 0,
                    last,
                },
            );
        }
    }
    for v in st.vault_slice() {
        add(
            RAINBOW,
            v.beast,
            CargoRef {
                is_vault: true,
                cell: 0,
                vault_id: v.id,
                last: 0,
            },
        );
    }
    for g in &mut groups[..glen] {
        g.refs[..g.refs_len as usize].sort_by_key(|r| r.last);
    }
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

pub fn plan_cargo<F>(st: &SimState, ctx: &Context, rate: F) -> Option<ConvoyPlan>
where
    F: Fn(&ConvoyPlan) -> f64,
{
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

    let mut counts = [0u8; 24];
    let mut best: Option<(ConvoyPlan, [u8; 24])> = None;

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

    fn walk<F>(
        w: &ConvoyWalkCtx<'_, F>,
        i: usize,
        pressure: u16,
        counts: &mut [u8; 24],
        best: &mut Option<(ConvoyPlan, [u8; 24])>,
    ) where
        F: Fn(&ConvoyPlan) -> f64,
    {
        if i == w.glen {
            let mut size = 0u8;
            let mut mask = 0u8;
            let mut elite = 0u8;
            let mut low = 0u8;
            let mut top = 0u8;
            let mut leg = [0u8; 4];
            let mut below = [0u8; 4];
            let mut points = 0u16;
            let mut press = 0u16;
            let mut kept = 0.0;

            for idx in 0..w.glen {
                let k = counts[idx];
                if k == 0 {
                    continue;
                }
                let g = &w.groups[idx];
                size += k;
                mask |= 1 << g.beast;
                if g.tier == 2 || g.tier == 3 {
                    elite += k;
                }
                if g.tier <= 1 {
                    low += k;
                }
                if g.tier >= SUPREME {
                    top += k;
                }
                if g.tier == 3 {
                    leg[g.beast as usize] += k;
                } else if g.tier < 3 {
                    below[g.beast as usize] += k;
                }
                let bonus = if g.tier >= SUPREME && (g.beast as i8) == w.bounty {
                    8u16
                } else {
                    0u16
                };
                points += (k as u16) * ((1u16 << g.tier) + bonus);
                press += (k as u16) * PRESSURE[g.tier as usize];
                let v_t = if g.tier >= RAINBOW {
                    w.ctx.r_val
                } else {
                    w.ctx.v[g.tier as usize]
                };
                kept += (k as f64) * v_t;
            }
            if size == 0 {
                return;
            }
            let beasts = mask.count_ones();
            let fits = match w.contract.id {
                ContractId::Mixed => size >= 3 && beasts >= 2,
                ContractId::Cohort => size >= 4,
                ContractId::Veteran => elite >= 1 && low >= 2,
                ContractId::Elite => elite >= 2 && beasts >= 2,
                ContractId::Kin => (0..4).any(|b| leg[b] >= 1 && below[b] >= 1),
                ContractId::Muster => elite >= 3,
                ContractId::Heavy | ContractId::Bounty => top >= 1,
            };
            if !fits {
                return;
            }
            for &escort in w.escorts {
                if press > escort.cap() {
                    continue;
                }
                let mut plan = ConvoyPlan {
                    escort,
                    cost: escort.cost(),
                    points,
                    pressure: press,
                    kept,
                    size,
                    score: 0.0,
                    cell_mask: 0,
                    vault_ids: [0; 10],
                    vault_len: 0,
                };
                let sc = (w.rate)(&plan);
                if sc > f64::NEG_INFINITY && (best.is_none() || sc > best.as_ref().unwrap().0.score)
                {
                    plan.score = sc;
                    *best = Some((plan, *counts));
                }
            }
            return;
        }
        let g = &w.groups[i];
        let p_step = PRESSURE[g.tier as usize];
        for k in 0..=g.refs_len {
            let next_p = pressure + (k as u16) * p_step;
            if next_p > w.cap {
                break;
            }
            counts[i] = k;
            walk(w, i + 1, next_p, counts, best);
        }
        counts[i] = 0;
    }

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
    walk(&w_ctx, 0, 0, &mut counts, &mut best);

    let (mut best_plan, best_counts) = best?;
    for (i, g) in groups[..glen].iter().enumerate() {
        let take = best_counts[i] as usize;
        for r in &g.refs[..take] {
            if r.is_vault {
                if (best_plan.vault_len as usize) < best_plan.vault_ids.len() {
                    best_plan.vault_ids[best_plan.vault_len as usize] = r.vault_id;
                    best_plan.vault_len += 1;
                }
            } else {
                best_plan.cell_mask |= 1u16 << r.cell;
            }
        }
    }
    Some(best_plan)
}

#[inline]
pub fn most_points(plan: &ConvoyPlan) -> f64 {
    (plan.points as f64) - 0.01 * (plan.cost as f64)
}

pub fn convoy_plan(st: &SimState, ctx: &Context) -> Option<ConvoyPlan> {
    let ch = st.challenge.as_ref()?;
    let slot = ch.deliveries_len as usize;
    if slot >= 2 {
        return plan_cargo(st, ctx, most_points);
    }
    let p = ctx.p;
    let keep = if slot == 0 { p.keep1 } else { p.keep2 };
    let must = slot == 1 && !ch.deliveries[0].success;
    let need = if must {
        p.min_left
    } else {
        p.min_left.max(p.reserve + u8::from(slot == 0))
    };
    plan_cargo(st, ctx, |plan| {
        if ch.vouchers < plan.cost {
            return f64::NEG_INFINITY;
        }
        let left = ch.vouchers - plan.cost;
        if left < p.min_left || (plan.escort != EscortKind::Light && left < need) {
            return f64::NEG_INFINITY;
        }
        (plan.points as f64) - keep * plan.kept + 0.001 * (plan.points as f64)
            - 0.01 * (plan.cost as f64)
    })
}

pub fn wants_early(st: &SimState, ctx: &Context) -> bool {
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

pub fn convoy_options(st: &SimState, params: Option<&'static SolverParams>) -> Vec<Candidate> {
    let p = params.unwrap_or_else(|| SolverParams::for_state(st));
    let ctx = Context::new(st, p);
    let Some(ch) = st.challenge.as_ref() else {
        return Vec::new();
    };
    let heavy_next = ch.deliveries_len < 2;
    let ok = |plan: &ConvoyPlan| !heavy_next || ch.vouchers.saturating_sub(plan.cost) >= 1;

    let mut plans: [(ConvoyKind, Option<ConvoyPlan>); 4] = [
        (ConvoyKind::Pick, convoy_plan(st, &ctx)),
        (ConvoyKind::Most, None),
        (ConvoyKind::Light, None),
        (ConvoyKind::Least, None),
    ];
    let mut plans_len = 1usize;
    if heavy_next {
        plans[1] = (
            ConvoyKind::Most,
            plan_cargo(st, &ctx, |pl| {
                if ok(pl) {
                    most_points(pl)
                } else {
                    f64::NEG_INFINITY
                }
            }),
        );
        plans[2] = (
            ConvoyKind::Light,
            plan_cargo(st, &ctx, |pl| {
                if ok(pl) && pl.escort == EscortKind::Light {
                    most_points(pl)
                } else {
                    f64::NEG_INFINITY
                }
            }),
        );
        plans[3] = (
            ConvoyKind::Least,
            plan_cargo(st, &ctx, |pl| {
                if ok(pl) && pl.escort == EscortKind::Light {
                    0.001 * (pl.points as f64) - pl.kept
                } else {
                    f64::NEG_INFINITY
                }
            }),
        );
        plans_len = 4;
    }

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

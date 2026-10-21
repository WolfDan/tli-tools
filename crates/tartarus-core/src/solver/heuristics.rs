use crate::engine::{
    APPETITE_ODDS, APPETITE_TRIES, BEASTS, Beast, CROW_CELLS, ChoiceId, Contract, ContractId,
    EventKind, FREEDOM_CELLS, GiftId, INSTIGATE_EVENTS, Inmate, OFFICE, ObjectiveId, PreyLink,
    RAINBOW, SUPREME, SimState, StartEventId, ToolId, describe_inmate, next_rand, plural,
    steps_between, tier_name,
};

use super::chain_dp::{Context, best_meal, composition, devour_delta, feed, floor_dist, swap_dist};
use super::params::SolverParams;

const _: () = {
    assert!(RAINBOW == 5);
    assert!(SUPREME == 4);
    assert!(OFFICE < 16);
};

#[inline]
pub fn stair(ctx: &Context, comp: &[u8; 24]) -> f64 {
    assert!(comp.len() == 24);
    assert!(ctx.v.len() >= RAINBOW as usize);
    let mut value = 0.0;
    for t in 0..(RAINBOW as usize) {
        let mut fed = 0;
        let eats = ctx.eats_at(t as u8);
        for eat in eats {
            if eat.slice().iter().any(|&f| comp[t * 4 + (f as usize)] > 0) {
                fed += 1;
            }
        }
        value += (fed as f64 / 4.0) * ctx.v[t];
    }
    assert!(value.is_finite());
    value
}

#[inline]
pub fn stair_delta(ctx: &Context, comp: &mut [u8; 24], tier: u8, beast: u8, sign: i8) -> f64 {
    assert!(beast < 4);
    assert!(sign == 1 || sign == -1);
    if ctx.p.w_stair == 0.0 || tier >= RAINBOW {
        return 0.0;
    }
    let before = stair(ctx, comp);
    let idx = (tier as usize) * 4 + (beast as usize);
    assert!(idx < comp.len());
    comp[idx] = (comp[idx] as i16 + sign as i16) as u8;
    let after = stair(ctx, comp);
    comp[idx] = (comp[idx] as i16 - sign as i16) as u8;
    ctx.p.w_stair * (after - before)
}

#[inline]
pub fn stair_fresh(ctx: &Context, dist: &[f64], comp: &mut [u8; 24]) -> f64 {
    assert!(!dist.is_empty());
    assert!(comp.len() == 24);
    if ctx.p.w_stair == 0.0 {
        return 0.0;
    }
    let before = stair(ctx, comp);
    let mut sum = 0.0;
    for (t, &p) in dist.iter().enumerate() {
        if p == 0.0 || (t as u8) >= RAINBOW {
            continue;
        }
        for b in 0..4 {
            comp[t * 4 + b] += 1;
            let after = stair(ctx, comp);
            comp[t * 4 + b] -= 1;
            sum += (p / 4.0) * (ctx.p.w_stair * (after - before));
        }
    }
    assert!(sum.is_finite());
    sum
}

pub fn maiden_bonus(st: &SimState, ctx: &Context, tier: u8) -> (f64, f64) {
    assert!(tier <= RAINBOW);
    assert!(st.day <= st.days_max);
    let Some(mut counts) = ctx.maiden else {
        return (0.0, 0.0);
    };
    let dist = swap_dist(st, ctx, tier);
    if (tier as usize) < 5 && counts[tier as usize] > 0 {
        counts[tier as usize] -= 1;
    }
    let mut chance = 0.0;
    let mut value = 0.0;
    for (t, &p) in dist.iter().enumerate() {
        if p == 0.0 {
            continue;
        }
        let mut above = 0u8;
        let mut below = 0u8;
        for (u, &n) in counts.iter().enumerate() {
            if u > t {
                above += n;
            } else if u < t {
                below += n;
            }
        }
        if above != below {
            continue;
        }
        chance += p;
        let promoted_val = if t + 3 >= (RAINBOW as usize) {
            ctx.r_val
        } else {
            ctx.v[t + 3]
        };
        value += p * (promoted_val - ctx.v[t]);
    }
    assert!(chance.is_finite() && value.is_finite());
    (chance, value)
}

#[inline]
pub fn coin_pair_ev(ctx: &Context, tier: u8) -> f64 {
    assert!(tier <= RAINBOW);
    assert!(ctx.r_val.is_finite());
    if !ctx.coin || tier >= RAINBOW {
        return 0.0;
    }
    let prom_v = if tier + 2 >= RAINBOW {
        ctx.r_val
    } else {
        ctx.v[(tier + 2) as usize]
    };
    0.6 * (prom_v - 2.0 * ctx.v[tier as usize]).max(0.0)
}

fn apply_swap_tail(ctx: &Context, dist: &[f64; 6], tier: u8, mut value: f64) -> f64 {
    assert!(tier <= RAINBOW);
    assert!(dist.len() == 6);
    if ctx.axe_doors > 0 {
        let mut higher_p = 0.0;
        for (t, &p) in dist.iter().enumerate() {
            if (t as u8) > tier {
                higher_p += p;
            }
        }
        value += higher_p * (ctx.axe_doors as f64) * ctx.p.w_open * ctx.open_value;
    }
    if let Some(loss) = ctx.scapegoat_loss {
        value -= loss;
    }
    assert!(value.is_finite());
    value
}

fn maiden_promoted_tier(t: usize, maiden_counts: Option<[u8; 5]>) -> u8 {
    assert!(t <= RAINBOW as usize);
    let mut eff_t = t as u8;
    if let Some(counts) = maiden_counts
        && t < 5
    {
        let mut above = 0u8;
        let mut below = 0u8;
        for (u, &n) in counts.iter().enumerate() {
            if u > t {
                above += n;
            } else if u < t {
                below += n;
            }
        }
        if above == below {
            eff_t = (eff_t + 3).min(RAINBOW);
        }
    }
    assert!(eff_t <= RAINBOW);
    eff_t
}

pub fn swap_value(st: &SimState, ctx: &Context, tier: u8) -> f64 {
    assert!((tier as usize) < ctx.v.len());
    assert!(st.day <= st.days_max);
    let dist = swap_dist(st, ctx, tier);
    let mut value = -ctx.v[tier as usize];
    for (t, &p) in dist.iter().enumerate() {
        let v_t = if (t as u8) >= RAINBOW {
            ctx.r_val
        } else {
            ctx.v[t]
        };
        value += p * v_t;
    }
    value = apply_swap_tail(ctx, &dist, tier, value);
    let out = value + maiden_bonus(st, ctx, tier).1;
    assert!(out.is_finite());
    out
}

pub fn swap_value_with_comp(
    st: &SimState,
    ctx: &mut Context,
    tier: u8,
    beast: u8,
    comp: &mut [u8; 24],
    crowd: usize,
) -> f64 {
    assert!((tier as usize) < ctx.v.len());
    assert!(beast < 4);
    if !st.used.devour_after_swap && !ctx.pestle {
        return swap_value(st, ctx, tier);
    }
    let dist = swap_dist(st, ctx, tier);
    let mut maiden_counts = ctx.maiden;
    if let Some(ref mut counts) = maiden_counts
        && (tier as usize) < 5
        && counts[tier as usize] > 0
    {
        counts[tier as usize] -= 1;
    }

    let beasts_slice: &[u8] = match beast {
        0 if ctx.pestle => &[0],
        1 if ctx.pestle => &[1],
        2 if ctx.pestle => &[2],
        3 if ctx.pestle => &[3],
        _ => &[0, 1, 2, 3],
    };
    let b_weight = 1.0 / (beasts_slice.len() as f64);
    let mut value = -ctx.v[tier as usize];

    for (t, &p) in dist.iter().enumerate() {
        if p == 0.0 {
            continue;
        }
        let eff_t = maiden_promoted_tier(t, maiden_counts);
        if eff_t >= RAINBOW {
            value += p * ctx.r_val;
            continue;
        }
        let mut tier_sum = 0.0;
        for &b in beasts_slice {
            let mut b_val = ctx.v[eff_t as usize];
            if st.used.devour_after_swap {
                let (bm, _) = best_meal(ctx, eff_t, b, comp, crowd);
                if bm > 0.0 {
                    b_val += bm;
                }
            } else if ctx.pestle && eff_t > tier {
                b_val += stair_delta(ctx, comp, eff_t, b, 1).max(0.0);
            }
            tier_sum += b_weight * b_val;
        }
        value += p * tier_sum;
    }

    let out = apply_swap_tail(ctx, &dist, tier, value);
    assert!(out.is_finite());
    out
}

#[inline]
pub fn trickster_value(ctx: &Context, dist: &[f64; 5]) -> f64 {
    assert!(dist.len() == 5);
    assert!((SUPREME as usize) < ctx.v.len());
    let mut sum = 0.0;
    for (t, &p) in dist.iter().enumerate() {
        let cur = ctx.v[t.min(SUPREME as usize)];
        sum += p * (ctx.v[SUPREME as usize] - cur).max(0.0);
    }
    assert!(sum.is_finite());
    sum
}

pub fn appetite_value(ctx: &Context, low: u8, dist: &[f64; 5]) -> f64 {
    assert!(dist.len() == 5);
    assert!((0.0..=1.0).contains(&APPETITE_ODDS));
    let n = APPETITE_TRIES;
    let q = APPETITE_ODDS;
    let val = |t: u8| -> f64 {
        if t >= RAINBOW {
            ctx.r_val
        } else {
            ctx.v[t as usize]
        }
    };
    let mut sum = 0.0;
    for (t, &p) in dist.iter().enumerate() {
        if p == 0.0 {
            continue;
        }
        let from = (t as u8).min(low);
        let mut choose = 1.0;
        for k in 0..=n {
            if k > 0 {
                choose = (choose * ((n - k + 1) as f64)) / (k as f64);
            }
            let prob = choose * q.powi(k as i32) * (1.0 - q).powi((n - k) as i32);
            sum += p * prob * (val((from + (k as u8)).min(RAINBOW)) - val(from));
        }
    }
    assert!(sum.is_finite());
    sum
}

pub fn hunt_pair_ev(st: &SimState, ctx: &Context, lion_tier: u8, prey_tier: u8) -> f64 {
    assert!((lion_tier as usize) < ctx.v.len());
    assert!((prey_tier as usize) < ctx.v.len());
    let eff_rounds = st
        .params
        .prey_rounds
        .min(st.days_max.saturating_sub(st.day));
    if eff_rounds == 0 {
        return 0.0;
    }
    let lion_to = (lion_tier + eff_rounds).min(RAINBOW);
    let lion_to_v = if lion_to >= RAINBOW {
        ctx.r_val
    } else {
        ctx.v[lion_to as usize]
    };
    let lion_gain = (lion_to_v - ctx.v[lion_tier as usize]).max(0.0);

    let mut dist = [0.0f64; 6];
    dist[(prey_tier as usize).min(5)] = 1.0;
    for _ in 0..eff_rounds {
        let mut next = [0.0f64; 6];
        next[5] += dist[5];
        for cur in 0..5usize {
            let p = dist[cur];
            if p == 0.0 {
                continue;
            }
            next[(cur + 1).min(5)] += 0.7 * p;
            next[(cur + 3).min(5)] += 0.3 * p;
        }
        dist = next;
    }
    let mut prey_ev = dist[5] * ctx.r_val;
    for (t_idx, &prob) in dist[..5].iter().enumerate() {
        prey_ev += prob * ctx.v[t_idx];
    }
    let prey_gain = (prey_ev - ctx.v[prey_tier as usize]).max(0.0);
    assert!(prey_gain.is_finite());
    0.8 * (lion_gain + prey_gain)
}

pub fn activation(
    st: &SimState,
    ctx: &mut Context,
    tier: u8,
    beast: u8,
    comp: &mut [u8; 24],
    crowd: usize,
    swap_left: bool,
) -> f64 {
    assert!(beast < 4);
    assert!(comp.len() == 24);
    let mut value = 0.0;
    if tier < RAINBOW {
        let (bm, _) = best_meal(ctx, tier, beast, comp, crowd);
        if bm > value {
            value = bm;
        }
    }
    let eff_swap_cap = if st.used.devour_after_swap && value <= 0.0 {
        ctx.swap_cap.max(2)
    } else {
        ctx.swap_cap
    };
    if swap_left && (tier as i8) <= eff_swap_cap {
        let sv = if st.used.devour_after_swap {
            swap_value_with_comp(st, ctx, tier, beast, comp, crowd)
        } else {
            swap_value(st, ctx, tier)
        };
        if sv > value {
            value = sv;
        }
    }
    assert!(value.is_finite());
    value
}

pub fn fresh_activation(
    st: &SimState,
    ctx: &mut Context,
    dist: &[f64; 5],
    comp: &mut [u8; 24],
    crowd: usize,
    swap_left: bool,
) -> f64 {
    assert!(dist.len() == 5);
    assert!(comp.len() == 24);
    let mut sum = 0.0;
    for (t, &p) in dist.iter().enumerate() {
        if p == 0.0 {
            continue;
        }
        for b in 0..4u8 {
            sum += (p / 4.0) * activation(st, ctx, t as u8, b, comp, crowd, swap_left);
        }
    }
    assert!(sum.is_finite());
    sum
}

#[inline]
pub fn tagged(ctx: &Context, dist: &[f64; 5]) -> [f64; 5] {
    assert!(dist.len() == 5);
    assert!(ctx.open_value.is_finite());
    if !ctx.dogtag {
        return *dist;
    }
    let mut out = [0.0; 5];
    for t in 0..5 {
        out[t] += 0.75 * dist[t];
        out[(t + 1).min(4)] += 0.25 * dist[t];
    }
    out
}

pub fn fresh_value(
    st: &SimState,
    ctx: &mut Context,
    dist: &[f64; 5],
    comp: &mut [u8; 24],
    crowd: usize,
    swap_left: bool,
) -> f64 {
    assert!(dist.len() == 5);
    assert!(comp.len() == 24);
    let mut worth = 0.0;
    for (t, &p) in dist.iter().enumerate() {
        worth += p * ctx.v[t];
    }
    assert!(worth.is_finite());
    ctx.p.w_open * worth
        + ctx.p.w_now * fresh_activation(st, ctx, dist, comp, crowd, swap_left)
        + stair_fresh(ctx, dist, comp)
}

pub fn execution_value(st: &SimState, ctx: &Context, tier: u8, beast: u8, comp: &[u8; 24]) -> f64 {
    assert!((tier as usize) < ctx.v.len());
    assert!((beast as usize) < BEASTS.len());
    let same = st.params.execution_same_tier;
    let prey = BEASTS[beast as usize].primary_prey().idx();
    let mut count = 0u8;
    let mut lost = 0.0;
    let t_max = if same { tier as i16 } else { (tier as i16) - 1 };
    for t in 0..=t_max {
        if same && t != (tier as i16) {
            continue;
        }
        let k = comp[(t as usize) * 4 + prey];
        count += k;
        lost += (k as f64) * ctx.v[t as usize];
    }
    assert!(lost.is_finite());
    if count == 0 {
        return 0.0;
    }
    let to = (tier + count).min(RAINBOW);
    let to_val = if to >= RAINBOW {
        ctx.r_val
    } else {
        ctx.v[to as usize]
    };
    to_val - ctx.v[tier as usize] - lost
}

pub fn event_worth_of(st: &SimState, ctx: &Context, comp: &[u8; 24], kind: EventKind) -> f64 {
    assert!(comp.len() == 24);
    assert!(st.day <= st.days_max);
    let p = &ctx.p;
    match kind {
        EventKind::Execution => {
            let mut exec = 0.0;
            for (t, &prob) in ctx.open_dist.iter().enumerate() {
                for b in 0..4u8 {
                    exec += (prob / 4.0)
                        * (execution_value(st, ctx, t as u8, b, comp) - p.exec_margin).max(0.0);
                }
            }
            exec
        }
        EventKind::Chain => p.w_chain,
        EventKind::Trickster => trickster_value(ctx, &ctx.open_dist),
        EventKind::Appetite => {
            let mut low = u8::MAX;
            for (i, &c_val) in comp.iter().enumerate().take(20) {
                if c_val > 0 {
                    low = (i / 4) as u8;
                    break;
                }
            }
            appetite_value(ctx, low, &ctx.open_dist)
        }
        EventKind::Convict => p.w_open * ctx.v[3],
        EventKind::Crow => p.w_crow,
        EventKind::Freedom => (FREEDOM_CELLS as f64) * p.w_open * ctx.open_value,
        EventKind::Usurp1 => ctx.lift + ctx.lift2 * p.w_instigate,
        EventKind::Usurp2 => ctx.lift2 + 2.0 * ctx.lift2 * p.w_instigate,
        EventKind::Usurp3 => 2.0 * ctx.lift2 + ctx.all_lift2 * p.w_instigate,
        EventKind::Usurp4 => ctx.all_lift2,
        EventKind::Ratway => p.w_open * ctx.open_value,
        EventKind::Hunt => p.w_crow * (1.0f64).max((ctx.sealed as f64) / (CROW_CELLS as f64)),
        EventKind::Portion => {
            let promotable: usize = comp[..20].iter().map(|&c| c as usize).sum();
            ((promotable + 1).min(3) as f64) * ctx.lift
        }
        EventKind::Fortune => 1.5 * ctx.lift,
        EventKind::Archenemy => {
            if ctx.sealed > 0 {
                p.w_open * ctx.open_value + ctx.lift
            } else {
                0.0
            }
        }
        EventKind::Transfer => {
            if ctx.sealed > 0 {
                p.w_open * ctx.open_value + 0.5 * ctx.lift
            } else {
                0.0
            }
        }
    }
}

fn fortune_cell_worth(st: &SimState, ctx: &Context, cell_n: u8, tier: u8, beast: u8) -> f64 {
    assert!((cell_n as usize) < st.cells.len());
    assert!(beast < 4);
    let (kcomp, _) = composition(st, cell_n as i16);
    let mut same = 0u8;
    for t in 0..5usize {
        same += kcomp[t * 4 + (beast as usize)];
    }
    if same == 0 {
        return 0.0;
    }
    let to = (tier + same).min(RAINBOW);
    let to_v = if to >= RAINBOW {
        ctx.r_val
    } else {
        ctx.v[to as usize]
    };
    (to_v - ctx.v[tier as usize]).max(0.0)
}

fn archenemy_cell_worth(
    st: &SimState,
    ctx: &mut Context,
    cell_n: u8,
    inm: Inmate,
    rem_sealed: usize,
) -> f64 {
    assert!((cell_n as usize) < st.cells.len());
    assert!(inm.tier <= RAINBOW);
    if rem_sealed == 0 {
        return 0.0;
    }
    let tier = inm.tier;
    let beast = inm.beast.idx() as u8;
    let m_tier = tier.min(SUPREME);
    let spawn_v = ctx.p.w_open * ctx.v[m_tier as usize];
    if st.can_devour_now() && tier < RAINBOW {
        let (mut kcomp, kcrowd) = composition(st, cell_n as i16);
        let food = inm.beast.primary_prey().idx() as u8;
        kcomp[(m_tier as usize) * 4 + (food as usize)] += 1;
        let dev = devour_delta(
            ctx,
            (tier, beast),
            (m_tier, food),
            false,
            &mut kcomp,
            kcrowd + 1,
            false,
        );
        return spawn_v + (ctx.p.w_now * dev).max(ctx.lift);
    }
    spawn_v + ctx.lift
}

fn event_worth_known_inmate(
    st: &SimState,
    ctx: &mut Context,
    cell_n: u8,
    inm: Inmate,
    kind: EventKind,
) -> Option<f64> {
    assert!((cell_n as usize) < st.cells.len());
    assert!(inm.tier <= RAINBOW);
    let c = &st.cells[cell_n as usize];
    let tier = inm.tier;
    let beast = inm.beast.idx() as u8;
    let rem_sealed = ctx.sealed.saturating_sub(usize::from(c.sealed));
    match kind {
        EventKind::Execution => {
            let (kcomp, _) = composition(st, cell_n as i16);
            Some((execution_value(st, ctx, tier, beast, &kcomp) - ctx.p.exec_margin).max(0.0))
        }
        EventKind::Trickster => {
            if tier >= SUPREME {
                Some(0.0)
            } else {
                Some((ctx.v[SUPREME as usize] - ctx.v[tier as usize]).max(0.0))
            }
        }
        EventKind::Fortune => Some(fortune_cell_worth(st, ctx, cell_n, tier, beast)),
        EventKind::Archenemy => Some(archenemy_cell_worth(st, ctx, cell_n, inm, rem_sealed)),
        EventKind::Transfer => {
            if rem_sealed == 0 {
                Some(0.0)
            } else {
                let base_v = ctx.p.w_open * ctx.v[tier.min(SUPREME) as usize];
                Some(base_v + 0.5 * ctx.lift + 0.5 * coin_pair_ev(ctx, tier))
            }
        }
        _ => None,
    }
}

pub fn event_worth_of_cell(
    st: &SimState,
    ctx: &mut Context,
    board_comp: &[u8; 24],
    cell_n: u8,
    kind: EventKind,
) -> f64 {
    assert!(board_comp.len() == 24);
    assert!((cell_n as usize) < st.cells.len());
    let c = &st.cells[cell_n as usize];
    let known_inmate = if !c.sealed {
        c.inmate
    } else if c.known >= 2 {
        c.hidden
    } else {
        None
    };

    if let Some(inm) = known_inmate {
        if let Some(val) = event_worth_known_inmate(st, ctx, cell_n, inm, kind) {
            return val;
        }
    } else if c.sealed
        && c.known == 1
        && let Some(h) = c.hidden
    {
        let tier = h.tier.min(SUPREME);
        match kind {
            EventKind::Trickster => {
                return if tier >= SUPREME {
                    0.0
                } else {
                    (ctx.v[SUPREME as usize] - ctx.v[tier as usize]).max(0.0)
                };
            }
            EventKind::Transfer => {
                return if ctx.sealed > 1 {
                    ctx.p.w_open * ctx.v[tier as usize] + 0.5 * ctx.lift
                } else {
                    0.0
                };
            }
            _ => {}
        }
    }

    event_worth_of(st, ctx, board_comp, kind)
}

pub fn expected_unknown_event_worth(st: &SimState, ctx: &Context, comp: &[u8; 24]) -> f64 {
    assert!(comp.len() == 24);
    assert!(st.day <= st.days_max);
    let pool = st.event_pool_slice();
    if pool.is_empty() {
        return 0.0;
    }
    let mut w_sum = 0.0;
    let mut v_sum = 0.0;
    for &id in pool {
        let w = id.weight() as f64;
        w_sum += w;
        v_sum += w * event_worth_of(st, ctx, comp, id);
    }
    assert!(w_sum >= 0.0 && v_sum.is_finite());
    if w_sum > 0.0 { v_sum / w_sum } else { 0.0 }
}

pub fn event_why(kind: EventKind) -> &'static str {
    let msg = match kind {
        EventKind::Execution => "an Execution waits behind the door",
        EventKind::Chain => "a Chain reaction waits behind the door",
        EventKind::Trickster => {
            "Behind Closed Doors waits there: whoever is inside becomes a Supreme"
        }
        EventKind::Appetite => {
            "Ravenous Appetite waits there: the lowest-rarity inmate gets 5 tries at +1 tier"
        }
        EventKind::Convict => "High-Risk Convict waits there: another cell opens on a Legendary",
        EventKind::Crow => {
            "Crow Whispers waits there: it shows the rarity behind the next 4 locked doors"
        }
        EventKind::Freedom => "Taste of Freedom waits there: 3 more locked cells open",
        EventKind::Usurp1 => {
            "Usurpation I waits there: a random inmate rises a tier, and Usurpation II appears"
        }
        EventKind::Usurp2 => "Usurpation II waits there: 1 random inmate rises 2 tiers",
        EventKind::Usurp3 => "Usurpation III waits there: 2 random inmates rise 2 tiers",
        EventKind::Usurp4 => "Usurpation IV waits there: every inmate rises 2 tiers",
        EventKind::Ratway => "the Sewer Ratway waits there: a free move to any cell but the office",
        EventKind::Hunt => concat!(
            "\"Treasure\" Hunt waits there: every locked door holding a Normal or Magic ",
            "shows its rarity, so the rest hold Rare or better"
        ),
        EventKind::Portion => "Extra Portion waits there: up to 3 random inmates rise a tier",
        EventKind::Fortune => concat!(
            "Shared Fortune waits there: this inmate rises a tier for each other ",
            "inmate of the same beast"
        ),
        EventKind::Archenemy => {
            "Archenemy waits there: a locked cell opens with a guaranteed meal for this inmate"
        }
        EventKind::Transfer => {
            "Transfer of Convicts waits there: this inmate is replicated into a random locked cell"
        }
    };
    assert!(!msg.is_empty());
    assert!(msg.len() > 10);
    msg
}

#[derive(Copy, Clone, Debug)]
pub struct OfficeValues {
    pub bestow: f64,
    pub instigate: f64,
    pub tyranny: f64,
}

pub fn office_values(st: &SimState, ctx: &Context, comp: &[u8; 24]) -> OfficeValues {
    assert!(comp.len() == 24);
    assert!(st.cells.len() == 16);
    let mut bestow = 0.0;
    let mut count = 0usize;
    for c in &st.cells {
        if c.sealed {
            continue;
        }
        if let Some(inmate) = c.inmate
            && inmate.tier < RAINBOW
        {
            let t = inmate.tier as usize;
            let next_v = if inmate.tier + 1 >= RAINBOW {
                ctx.r_val
            } else {
                ctx.v[t + 1]
            };
            bestow += next_v - ctx.v[t];
            count += 1;
        }
    }
    bestow /= (count.max(1)) as f64;
    let free = st
        .cells
        .iter()
        .filter(|c| c.sealed && c.event.is_none() && c.nest_due.is_none())
        .count();
    let each = expected_unknown_event_worth(st, ctx, comp);
    let days_rem = st.days_max.saturating_sub(st.day) as f64;
    let horizon_inst = (days_rem / 6.0).clamp(0.0, 1.0);
    let extra = if st.start_event == StartEventId::Tyranny {
        3usize
    } else {
        0usize
    };
    let tyranny = if extra > 0 {
        (extra as f64) * bestow
            + ((extra * INSTIGATE_EVENTS).min(free) as f64)
                * each
                * ctx.p.w_instigate
                * horizon_inst
    } else {
        0.0
    };
    let mut instigate =
        (INSTIGATE_EVENTS.min(free) as f64) * each * ctx.p.w_instigate * horizon_inst;
    if !st.milestone && st.params.rewards && st.objective_id == ObjectiveId::Incident && free > 0 {
        let need_prog = ObjectiveId::Incident.need();
        if st.objective_progress < need_prog {
            let rem = (need_prog - st.objective_progress) as f64;
            instigate += (0.40 / rem.max(1.0)) * ctx.lift;
        }
    }
    assert!(bestow.is_finite() && instigate.is_finite());
    OfficeValues {
        bestow,
        instigate,
        tyranny,
    }
}

pub fn contract_gap(contract: Contract, comp: &[u8; 24]) -> i32 {
    assert!(comp.len() == 24);
    let mut size = 0i32;
    let mut elite = 0i32;
    let mut low = 0i32;
    let mut beasts = 0i32;
    let mut kin = 2i32;
    for b in 0..4usize {
        let mut any = 0i32;
        let mut below = 0i32;
        for t in 0..(RAINBOW as usize) {
            let k = comp[t * 4 + b] as i32;
            if k == 0 {
                continue;
            }
            any += k;
            if t == 2 || t == 3 {
                elite += k;
            }
            if t <= 1 {
                low += k;
            }
            if t < 3 {
                below += k;
            }
        }
        size += any;
        if any > 0 {
            beasts += 1;
        }
        let leg = comp[12 + b] as i32;
        if leg > 0 && below > 0 {
            kin = 0;
        } else if leg > 0 && kin > 0 {
            kin = 1;
        }
    }
    let gap = match contract.id {
        ContractId::Mixed => (3 - size).max(0) + (2 - beasts).max(0),
        ContractId::Cohort => (4 - size).max(0),
        ContractId::Veteran => (1 - elite).max(0) + (2 - low).max(0),
        ContractId::Elite => (2 - elite).max(0) + (2 - beasts).max(0),
        ContractId::Kin => kin,
        ContractId::Muster => (3 - elite).max(0),
        ContractId::Heavy | ContractId::Bounty => 0,
    };
    assert!(gap >= 0);
    gap
}

pub fn need_value(
    ctx: &Context,
    comp: &mut [u8; 24],
    removes: &[(u8, u8)],
    add_dist: Option<&[f64]>,
) -> f64 {
    assert!(comp.len() == 24);
    assert!(removes.len() <= 2);
    let Some(need) = ctx.need else {
        return 0.0;
    };
    let before = contract_gap(need.contract, comp) as f64;
    assert!(before >= 0.0);
    for &(t, b) in removes {
        comp[(t as usize) * 4 + (b as usize)] -= 1;
    }
    let mut after = 0.0;
    if let Some(dist) = add_dist {
        for (t, &p) in dist.iter().enumerate() {
            if p == 0.0 || (t as u8) >= RAINBOW {
                continue;
            }
            for b in 0..4usize {
                comp[t * 4 + b] += 1;
                after += (p / 4.0) * (contract_gap(need.contract, comp) as f64);
                comp[t * 4 + b] -= 1;
            }
        }
        if dist.len() > (RAINBOW as usize) && dist[RAINBOW as usize] > 0.0 {
            after += dist[RAINBOW as usize] * (contract_gap(need.contract, comp) as f64);
        }
    } else {
        after = contract_gap(need.contract, comp) as f64;
    }
    for &(t, b) in removes {
        comp[(t as usize) * 4 + (b as usize)] += 1;
    }
    need.weight * (before - after)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PatrolCellScore {
    pub cell: u8,
    pub score: f64,
}

struct PatrolEvalState {
    board_comp: [u8; 24],
    board_crowd: usize,
    swap_left: bool,
    fresh_cached: Option<f64>,
}

fn patrol_chain_score(
    st: &SimState,
    ctx: &mut Context,
    state: &mut PatrolEvalState,
    n: u8,
    steps: u8,
    mut score: f64,
) -> f64 {
    assert!((n as usize) < st.cells.len());
    let c = &st.cells[n as usize];
    let p = ctx.p;
    let (path, plen) = if steps > 0 {
        st.clockwise_path(n)
    } else {
        let mut arr = [0u8; 16];
        arr[0] = n;
        (arr, 1)
    };
    assert!(plen <= path.len());
    let sealed_cnt = path[..plen]
        .iter()
        .filter(|&&k| st.cells[k as usize].sealed)
        .count();
    let dest_val = if c.sealed {
        if let Some(fv) = state.fresh_cached {
            fv
        } else {
            let tg = tagged(ctx, &ctx.open_dist);
            let fv = fresh_value(
                st,
                ctx,
                &tg,
                &mut state.board_comp,
                state.board_crowd + 1,
                state.swap_left,
            );
            state.fresh_cached = Some(fv);
            fv
        }
    } else if let Some(inmate) = c.inmate {
        let (mut kcomp, kcrowd) = composition(st, c.n as i16);
        p.w_walk
            * activation(
                st,
                ctx,
                inmate.tier,
                inmate.beast.idx() as u8,
                &mut kcomp,
                kcrowd,
                state.swap_left,
            )
    } else {
        0.0
    };
    score += p.w_open * ((sealed_cnt - (c.sealed as usize)) as f64) * ctx.open_value + dest_val;
    score
}

fn patrol_sealed_door_bonuses(
    st: &SimState,
    ctx: &mut Context,
    board_comp: &[u8; 24],
    n: u8,
    mut score: f64,
) -> f64 {
    assert!(board_comp.len() == 24);
    assert!((n as usize) < st.cells.len());
    let c = &st.cells[n as usize];
    let p = ctx.p;
    if ctx.blacklist && ctx.sealed.saturating_sub(1) <= 5 {
        score += 2.0 * ctx.lift;
    }
    if st.has_tool(ToolId::Keys) {
        let adj_sealed = [((n + 15) & 15), ((n + 1) & 15)]
            .into_iter()
            .filter(|&adj| st.cells[adj as usize].sealed)
            .count();
        score += (adj_sealed as f64) * 0.45 * p.w_open * ctx.open_value;
    }
    if ctx.ravenous && ctx.top_tier >= 3 {
        let adj_top = [((n + 15) & 15), ((n + 1) & 15)].into_iter().any(|adj| {
            let ac = &st.cells[adj as usize];
            !ac.sealed && ac.inmate.is_some_and(|i| i.tier == ctx.top_tier)
        });
        if adj_top {
            score += 0.25 * ctx.lift;
        }
    }
    let unk_ev = expected_unknown_event_worth(st, ctx, board_comp);
    if !c.seen {
        score += (st.params.event_chance / 100.0) * unk_ev;
    } else if c.event.is_some() && !c.event_known {
        score += unk_ev;
    } else if let Some(ev) = c.event {
        score += event_worth_of_cell(st, ctx, board_comp, n, ev);
    }
    score
}

fn patrol_sealed_score(
    st: &SimState,
    ctx: &mut Context,
    state: &mut PatrolEvalState,
    n: u8,
    mut score: f64,
) -> f64 {
    assert!((n as usize) < st.cells.len());
    let c = &st.cells[n as usize];
    assert!(c.sealed);
    let shown = if c.known > 0 {
        c.hidden.map(|h| {
            let mut arr = [0.0; 5];
            arr[(h.tier as usize).min(4)] = 1.0;
            arr
        })
    } else {
        None
    };
    let dist_opt = shown.or_else(|| {
        if c.at_least > 0 {
            Some(floor_dist(&ctx.open_dist, c.at_least))
        } else {
            None
        }
    });
    let open_dist_copy = ctx.open_dist;
    let need_dist = dist_opt.unwrap_or(open_dist_copy);
    let need = need_value(ctx, &mut state.board_comp, &[], Some(&need_dist));
    let fv = if let Some(d) = dist_opt {
        let tg = tagged(ctx, &d);
        fresh_value(
            st,
            ctx,
            &tg,
            &mut state.board_comp,
            state.board_crowd + 1,
            state.swap_left,
        )
    } else if let Some(cached) = state.fresh_cached {
        cached
    } else {
        let tg = tagged(ctx, &open_dist_copy);
        let val = fresh_value(
            st,
            ctx,
            &tg,
            &mut state.board_comp,
            state.board_crowd + 1,
            state.swap_left,
        );
        state.fresh_cached = Some(val);
        val
    };
    score += fv + need;
    patrol_sealed_door_bonuses(st, ctx, &state.board_comp, n, score)
}

fn patrol_inmate_score(
    st: &SimState,
    ctx: &mut Context,
    board_comp: &[u8; 24],
    n: u8,
    inmate: Inmate,
    mut score: f64,
) -> f64 {
    assert!(board_comp.len() == 24);
    assert!((n as usize) < st.cells.len());
    let c = &st.cells[n as usize];
    let swap_left = st.can_swap_now();
    let (mut kcomp, kcrowd) = composition(st, c.n as i16);
    let own_loss = link_loss(st, ctx, Some(inmate), false);
    let mut act_val = (activation(
        st,
        ctx,
        inmate.tier,
        inmate.beast.idx() as u8,
        &mut kcomp,
        kcrowd,
        swap_left,
    ) - own_loss)
        .max(0.0);
    if ctx.trap
        && inmate.beast == Beast::Lion
        && inmate.tier < RAINBOW
        && !st.used.hunt
        && st.link_of(inmate.id).is_none()
    {
        for oc in &st.cells {
            if oc.n != c.n
                && !oc.sealed
                && let Some(meal) = oc.inmate
                && st.can_eat(inmate, meal)
                && st.link_of(meal.id).is_none()
            {
                let hv = hunt_pair_ev(st, ctx, inmate.tier, meal.tier);
                if hv > act_val {
                    act_val = hv;
                }
            }
        }
    }
    score += ctx.p.w_walk * act_val;
    if let Some(ev) = c.event {
        if !c.event_known {
            score += expected_unknown_event_worth(st, ctx, board_comp);
        } else {
            score += event_worth_of_cell(st, ctx, board_comp, n, ev);
        }
    }
    score
}

fn patrol_step_and_objective_bonus(
    st: &SimState,
    ctx: &Context,
    n: u8,
    steps: u8,
    mut score: f64,
) -> f64 {
    assert!((n as usize) < st.cells.len());
    assert!(steps <= 16);
    let c = &st.cells[n as usize];
    let p = ctx.p;
    if ctx.shackles_lift > 0.0 && steps >= 4 {
        score += 0.4 * ctx.shackles_lift;
    }
    if ctx.shortchain && (1..=3).contains(&steps) && ctx.sealed > (c.sealed as usize) {
        score += p.w_short * 0.5 * p.w_open * ctx.open_value;
    }
    if ctx.goal_left > 0 {
        score += p.w_goal * (steps.min(ctx.goal_left) as f64);
    }
    if !st.milestone && st.params.rewards {
        let need_prog = st.objective_id.need();
        if st.objective_id == ObjectiveId::Doors && c.sealed && st.objective_progress < need_prog {
            let rem = (need_prog - st.objective_progress) as f64;
            score += (0.35 / rem.max(1.0)) * ctx.lift;
        } else if st.objective_id == ObjectiveId::Incident
            && c.event.is_some()
            && st.objective_progress < need_prog
        {
            let rem = (need_prog - st.objective_progress) as f64;
            score += (0.45 / rem.max(1.0)) * ctx.lift;
        }
    }
    score
}

pub fn patrol_scores_fast(st: &SimState, ctx: &mut Context) -> ([PatrolCellScore; 16], usize) {
    assert!(st.cells.len() == 16);
    assert!((st.position as usize) < st.cells.len());
    let p = ctx.p;
    let (board_comp, board_crowd) = composition(st, -1);
    let mut state = PatrolEvalState {
        board_comp,
        board_crowd,
        swap_left: st.can_swap_now(),
        fresh_cached: None,
    };

    let mut out = [PatrolCellScore::default(); 16];
    let mut len = 0usize;
    let range = st.params.patrol_range.min(15);

    for &n in st.destinations_slice() {
        let c = &st.cells[n as usize];
        let d_steps = steps_between(st.position, n);
        let steps = if !st.ratway && d_steps <= range {
            d_steps
        } else {
            0
        };
        let mut score = p.w_step * (steps as f64);

        if st.chain_armed && !st.ratway {
            score = patrol_chain_score(st, ctx, &mut state, n, steps, score);
        } else if c.sealed && c.nest_due.is_some() {
            score += p.w_nest;
        } else if c.sealed {
            score = patrol_sealed_score(st, ctx, &mut state, n, score);
        } else if n == OFFICE {
            let office = office_values(st, ctx, &state.board_comp);
            score += office.bestow.max(office.instigate) + office.tyranny;
        } else if let Some(inmate) = c.inmate {
            score = patrol_inmate_score(st, ctx, &state.board_comp, n, inmate, score);
        }

        score = patrol_step_and_objective_bonus(st, ctx, n, steps, score);
        if len < 16 {
            out[len] = PatrolCellScore { cell: n, score };
            len += 1;
        }
    }

    assert!(len <= out.len());
    out[..len].sort_by(|a, b| b.score.total_cmp(&a.score));
    (out, len)
}

fn patrol_sealed_cell_why(
    st: &SimState,
    ctx: &Context,
    board_comp: &mut [u8; 24],
    n: u8,
) -> String {
    assert!(board_comp.len() == 24);
    assert!((n as usize) < st.cells.len());
    let c = &st.cells[n as usize];
    if c.seen && c.event.is_some() && !c.event_known {
        return "a special event waits behind the door".to_string();
    }
    if let Some(ev) = c.event.filter(|_| c.seen) {
        return event_why(ev).to_string();
    }
    let shown = if c.known > 0 {
        c.hidden.map(|h| {
            let mut arr = [0.0; 5];
            arr[(h.tier as usize).min(4)] = 1.0;
            arr
        })
    } else {
        None
    };
    let dist_opt = shown.or_else(|| {
        if c.at_least > 0 {
            Some(floor_dist(&ctx.open_dist, c.at_least))
        } else {
            None
        }
    });
    let need_dist = dist_opt.unwrap_or(ctx.open_dist);
    let need = need_value(ctx, board_comp, &[], Some(&need_dist));
    let who = if shown.is_none() && c.at_least > 0 {
        format!(
            "a {} or better inmate (the \"Treasure\" Hunt left this door dark)",
            tier_name(c.at_least)
        )
    } else {
        "a new inmate".to_string()
    };
    if need > 0.0 {
        format!(
            "{who}, who may help meet \"{}\"",
            ctx.need.unwrap().contract.id.name()
        )
    } else {
        format!("{who}, who may consume today")
    }
}

fn patrol_open_cell_why(st: &SimState, ctx: &Context, board_comp: &[u8; 24], n: u8) -> String {
    assert!(board_comp.len() == 24);
    assert!((n as usize) < st.cells.len());
    let c = &st.cells[n as usize];
    if n == OFFICE {
        let office = office_values(st, ctx, board_comp);
        let mut why = if office.bestow >= office.instigate {
            "the office: Bestow raises 1 random inmate a tier".to_string()
        } else {
            format!("the office: Instigate brings {INSTIGATE_EVENTS} new special events")
        };
        if office.tyranny > 0.0 {
            why.push_str(", and Tyranny adds 3 of each");
        }
        why
    } else if let Some(inmate) = c.inmate {
        if let Some(ev) = c.event {
            if !c.event_known {
                format!(
                    "a special event waits in the cell with its {}",
                    describe_inmate(inmate)
                )
            } else {
                event_why(ev).replace("behind the door", "in the cell")
            }
        } else {
            format!("its {} can act today", describe_inmate(inmate))
        }
    } else {
        "an empty cell".to_string()
    }
}

pub fn patrol_cell_why(st: &SimState, ctx: &mut Context, n: u8) -> String {
    assert!((n as usize) < st.cells.len());
    assert!((st.position as usize) < st.cells.len());
    let c = &st.cells[n as usize];
    let range = st.params.patrol_range.min(15);
    let d_steps = steps_between(st.position, n);
    let steps = if !st.ratway && d_steps <= range {
        d_steps
    } else {
        0
    };
    let (mut board_comp, _) = composition(st, -1);

    if st.chain_armed && !st.ratway {
        let (path, plen) = if steps > 0 {
            st.clockwise_path(n)
        } else {
            let mut arr = [0u8; 16];
            arr[0] = n;
            (arr, 1)
        };
        let sealed_cnt = path[..plen]
            .iter()
            .filter(|&&k| st.cells[k as usize].sealed)
            .count();
        format!("the chain opens {} on the way", plural(sealed_cnt, "door"))
    } else if c.sealed && c.nest_due.is_some() {
        format!(
            "it destroys the rat nest due on day {}",
            c.nest_due.unwrap()
        )
    } else if c.sealed {
        patrol_sealed_cell_why(st, ctx, &mut board_comp, n)
    } else {
        patrol_open_cell_why(st, ctx, &board_comp, n)
    }
}

pub fn link_value(st: &SimState, ctx: &Context, link: PreyLink) -> f64 {
    assert!(st.day <= st.days_max);
    assert!(ctx.r_val.is_finite());
    let eff_rounds = link.rounds.min(st.days_max.saturating_sub(st.day));
    if eff_rounds == 0 {
        return 0.0;
    }
    let has_trap = st.has_tool(ToolId::Trap);
    let mut value = 0.0;
    for id in [link.lion, link.prey] {
        let Some(cn) = st.cell_of(id) else {
            continue;
        };
        let Some(inmate) = st.cells[cn as usize].inmate else {
            continue;
        };
        let t = inmate.tier;
        if has_trap && id == link.prey {
            let mut dist = [0.0f64; 6];
            dist[(t as usize).min(5)] = 1.0;
            for _ in 0..eff_rounds {
                let mut next = [0.0f64; 6];
                next[5] += dist[5];
                for cur in 0..5usize {
                    let p = dist[cur];
                    if p == 0.0 {
                        continue;
                    }
                    next[(cur + 1).min(5)] += 0.7 * p;
                    next[(cur + 3).min(5)] += 0.3 * p;
                }
                dist = next;
            }
            let mut ev = dist[5] * ctx.r_val;
            for (tier_idx, &prob) in dist[..5].iter().enumerate() {
                ev += prob * ctx.v[tier_idx];
            }
            value += (ev - ctx.v[t as usize]).max(0.0);
        } else {
            let to = (t + eff_rounds).min(RAINBOW);
            let to_v = if to >= RAINBOW {
                ctx.r_val
            } else {
                ctx.v[to as usize]
            };
            value += to_v - ctx.v[t as usize];
        }
    }
    assert!(value.is_finite());
    value
}

pub fn link_loss(st: &SimState, ctx: &Context, inmate: Option<Inmate>, certain: bool) -> f64 {
    assert!(st.day <= st.days_max);
    assert!(ctx.r_val.is_finite());
    let Some(inm) = inmate else {
        return 0.0;
    };
    let Some(link) = st.link_of(inm.id) else {
        return 0.0;
    };
    if certain {
        link_value(st, ctx, link)
    } else if st.cage_fight() {
        // Under CageFight, devouring does not reroll the eater's species, so the PreyLink survives!
        0.0
    } else {
        // Without CageFight, devouring rerolls the eater's species (75% chance to change species,
        // and >= 0.80 ensures a Lion that chose Hunt over Devour won't immediately Devour on
        // step 2).
        0.82 * link_value(st, ctx, link)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DevourScore {
    pub cell: u8,
    pub score: f64,
}

fn devour_need_bonus(
    ctx: &Context,
    whole: &mut Option<[u8; 24]>,
    eater: (u8, u8),
    meal: (u8, u8),
    scratch: bool,
    to: u8,
) -> f64 {
    assert!(eater.1 < 4 && meal.1 < 4);
    assert!(to <= RAINBOW);
    let Some(w_comp) = whole.as_mut() else {
        return 0.0;
    };
    let mut removes = [eater, (0, 0)];
    let rlen = if scratch {
        1
    } else {
        removes[1] = meal;
        2
    };
    let mut add_d = [0.0; 6];
    add_d[to as usize] = 1.0;
    need_value(ctx, w_comp, &removes[..rlen], Some(&add_d))
}

fn devour_static_bonuses(st: &SimState, ctx: &Context) -> (f64, f64) {
    assert!(st.day <= st.days_max);
    let cage = if ctx.cage {
        ctx.p.w_cage * 0.4 * ctx.p.w_open * ctx.open_value
    } else {
        0.0
    };
    let need_prog = st.objective_id.need();
    let obj_bonus = if !st.milestone
        && st.params.rewards
        && st.objective_id == ObjectiveId::Consume
        && st.objective_progress < need_prog
    {
        let rem = (need_prog - st.objective_progress) as f64;
        (0.35 / rem.max(1.0)) * ctx.lift
    } else {
        0.0
    };
    assert!(cage.is_finite() && obj_bonus.is_finite());
    (cage, obj_bonus)
}

fn devour_meal_penalties(
    st: &SimState,
    ctx: &Context,
    comp: &[u8; 24],
    eater: Inmate,
    meal: Inmate,
    scratch: bool,
) -> (f64, f64) {
    assert!(comp.len() == 24);
    let own = link_loss(st, ctx, Some(eater), false);
    let meal_link = if scratch {
        0.0
    } else {
        link_loss(st, ctx, Some(meal), true)
    };
    let same_link = own > 0.0
        && meal_link > 0.0
        && st.link_of(eater.id).map(|l| (l.lion, l.prey))
            == st.link_of(meal.id).map(|l| (l.lion, l.prey));
    let loss = if same_link {
        own.max(meal_link)
    } else {
        own + meal_link
    };
    let m_tier = if scratch { eater.tier } else { meal.tier };
    let coin_loss = if !scratch
        && ctx.coin
        && (2..RAINBOW).contains(&m_tier)
        && comp[(m_tier as usize) * 4 + meal.beast.idx()] == 1
    {
        0.40 * coin_pair_ev(ctx, m_tier)
    } else {
        0.0
    };
    assert!(loss.is_finite() && coin_loss.is_finite());
    (loss, coin_loss)
}

pub fn devour_scores(st: &SimState, ctx: &mut Context) -> ([DevourScore; 16], usize) {
    assert!(st.cells.len() == 16);
    assert!((st.position as usize) < st.cells.len());
    let mut out = [DevourScore {
        cell: 0,
        score: 0.0,
    }; 16];
    let here = &st.cells[st.position as usize];
    if here.sealed {
        return (out, 0);
    }
    let Some(eater) = here.inmate else {
        return (out, 0);
    };
    let (mut comp, crowd) = composition(st, st.position as i16);
    let tier = eater.tier;
    let beast = eater.beast.idx() as u8;
    let mut whole = if ctx.need.is_some() {
        Some(composition(st, -1).0)
    } else {
        None
    };
    let chained = st.chaining_eater.is_some();
    let (opts, olen) = if chained {
        st.chain_options()
    } else {
        st.start_options()
    };
    let (cage, obj_bonus) = devour_static_bonuses(st, ctx);

    for (idx, &o) in opts[..olen].iter().enumerate() {
        let target = &st.cells[o.cell as usize];
        let meal = if o.scratch {
            target.hidden.unwrap()
        } else {
            target.inmate.unwrap()
        };
        let (loss, coin_loss) = devour_meal_penalties(st, ctx, &comp, eater, meal, o.scratch);
        let stair_loss = if o.scratch {
            0.0
        } else {
            stair_delta(ctx, &mut comp, meal.tier, meal.beast.idx() as u8, -1)
        };
        let to = (tier + st.gain_for(st.position, o)).min(RAINBOW);
        let meal_pair = (meal.tier, meal.beast.idx() as u8);
        let need = devour_need_bonus(ctx, &mut whole, (tier, beast), meal_pair, o.scratch, to);
        let m_tier = if o.scratch { tier } else { meal.tier };
        let delta = devour_delta(
            ctx,
            (tier, beast),
            (m_tier, meal.beast.idx() as u8),
            o.scratch,
            &mut comp,
            crowd,
            chained,
        );
        out[idx] = DevourScore {
            cell: o.cell,
            score: delta - loss - coin_loss + stair_loss + need + cage + obj_bonus,
        };
    }

    out[..olen].sort_by(|a, b| b.score.total_cmp(&a.score));
    (out, olen)
}

fn ivory_swap_bonus(st: &SimState, ctx: &Context) -> f64 {
    assert!(st.cells.len() == 16);
    let mut board_len = 0usize;
    let mut lift_sum = 0.0;
    for c in &st.cells {
        if !c.sealed
            && let Some(i) = c.inmate
            && i.tier < RAINBOW
        {
            board_len += 1;
            let t = i.tier as usize;
            lift_sum += ctx.v[t + 1] - ctx.v[t];
        }
    }
    let lift = lift_sum / ((board_len.max(1)) as f64);
    let bonus = ((st.params.ivory_count as usize).min(board_len) as f64) * lift;
    assert!(bonus.is_finite());
    bonus
}

pub fn swap_gain_here(st: &SimState, ctx: &mut Context) -> f64 {
    assert!(st.cells.len() == 16);
    assert!((st.position as usize) < st.cells.len());
    let here = &st.cells[st.position as usize];
    if here.sealed || !st.can_swap_now() || ctx.swap_cap < 0 {
        return f64::NEG_INFINITY;
    }
    let Some(inmate) = here.inmate else {
        return f64::NEG_INFINITY;
    };
    let (mut kcomp, kcrowd) = composition(st, st.position as i16);
    let mut gain = swap_value_with_comp(
        st,
        ctx,
        inmate.tier,
        inmate.beast.idx() as u8,
        &mut kcomp,
        kcrowd,
    ) - link_loss(st, ctx, Some(inmate), true);
    if ctx.need.is_some() {
        let (mut w_comp, _) = composition(st, -1);
        let s_dist = swap_dist(st, ctx, inmate.tier);
        gain += need_value(
            ctx,
            &mut w_comp,
            &[(inmate.tier, inmate.beast.idx() as u8)],
            Some(&s_dist),
        );
    }
    if here.ivory {
        return gain + ivory_swap_bonus(st, ctx);
    }
    if (inmate.tier as i8) <= ctx.swap_cap
        || (inmate.tier == 2
            && st.can_devour_now()
            && st.used.devour_after_swap
            && gain > 0.08
            && best_meal(
                ctx,
                inmate.tier,
                inmate.beast.idx() as u8,
                &mut kcomp,
                kcrowd,
            )
            .0 <= 0.0)
    {
        gain
    } else {
        f64::NEG_INFINITY
    }
}

fn add_board_tool_bonuses(st: &SimState, ctx: &Context, comp: &[u8; 24], mut value: f64) -> f64 {
    assert!(comp.len() == 24);
    assert!(value.is_finite());
    if ctx.coin {
        for t in 2..(RAINBOW as usize) {
            for b in 0..4usize {
                if comp[t * 4 + b] >= 2 {
                    value += 0.45 * coin_pair_ev(ctx, t as u8);
                }
            }
        }
    }
    if ctx.ravenous && ctx.top_tier >= 2 && ctx.top_tier < RAINBOW {
        for c in &st.cells {
            if !c.sealed
                && let Some(top) = c.inmate.filter(|i| i.tier == ctx.top_tier)
            {
                let gain = 1
                    + u8::from(ctx.crown && top.beast == Beast::Lion)
                    + u8::from(ctx.skewer && top.beast == Beast::Rat);
                let to = (top.tier + gain).min(RAINBOW);
                let to_v = if to >= RAINBOW {
                    ctx.r_val
                } else {
                    ctx.v[to as usize]
                };
                let mut best_adj = 0.0_f64;
                for adj in [((c.n + 15) & 15), ((c.n + 1) & 15)] {
                    let ac = &st.cells[adj as usize];
                    if !ac.sealed
                        && let Some(victim) = ac.inmate.filter(|v| v.tier < top.tier)
                    {
                        let net = (to_v - ctx.v[top.tier as usize] - ctx.v[victim.tier as usize])
                            .max(0.0);
                        if net > best_adj {
                            best_adj = net;
                        }
                    }
                }
                value += 0.45 * best_adj;
            }
        }
    }
    value
}

pub fn board_value(st: &SimState, ctx: &mut Context) -> f64 {
    assert!(st.cells.len() == 16);
    assert!(ctx.r_val.is_finite());
    let mut value = (st.vault_len as f64) * ctx.r_val;
    let (comp, _) = composition(st, -1);
    let mut best_chain_gain = 0.0f64;
    for c in &st.cells {
        if c.sealed {
            continue;
        }
        if let Some(inmate) = c.inmate {
            let (_, olen) = st.devour_options(c.n);
            let t = inmate.tier as usize;
            let mut stag = 0.0;
            if t > 0 && t < (RAINBOW as usize) {
                let prey = inmate.beast.primary_prey().idx();
                if comp[(t - 1) * 4 + prey] > 0 {
                    stag += 0.12 * ctx.v[t];
                }
            }
            if olen > 0 {
                let mut extra = 0.0;
                if inmate.tier < SUPREME {
                    let (mut kcomp, kcrowd) = composition(st, c.n as i16);
                    let f_val = feed(
                        ctx,
                        inmate.tier,
                        inmate.beast.idx() as u8,
                        &mut kcomp,
                        kcrowd,
                    );
                    let excess = (f_val - ctx.v[t + 1]).max(0.0);
                    if excess > best_chain_gain {
                        best_chain_gain = excess;
                    }
                    extra = 0.12 * excess;
                }
                value += ctx.v[t] * 1.25 + extra + stag;
            } else {
                value += ctx.v[t] + stag;
            }
        }
    }
    value += 0.25 * best_chain_gain;
    value = add_board_tool_bonuses(st, ctx, &comp, value);
    for &link in &st.prey[..st.prey_len as usize] {
        value += 0.50 * link_value(st, ctx, link);
    }
    assert!(value.is_finite());
    value
}

pub fn pick_gift(
    st: &SimState,
    p: &'static SolverParams,
    options: &[ChoiceId],
    rng: &mut u32,
) -> ChoiceId {
    assert!(!options.is_empty());
    assert!(st.day <= st.days_max);
    let mut best_id = options[0];
    let mut best_val = f64::NEG_INFINITY;
    for &id in options {
        let ChoiceId::Gift(g) = id else {
            continue;
        };
        let samples = if matches!(g, GiftId::Ring | GiftId::Horn) {
            4
        } else {
            1
        };
        let mut val = 0.0;
        for _ in 0..samples {
            let mut trial = *st;
            trial.rng = (next_rand(rng) * 4294967296.0) as u32;
            if !trial.choose(id) {
                continue;
            }
            let mut t_ctx = Context::new(&trial, p);
            val += board_value(&trial, &mut t_ctx);
        }
        val /= samples as f64;
        let tie_bias = match g {
            GiftId::Brand => 1e-4,
            GiftId::Wine => 7e-5,
            GiftId::Horn => 4e-5,
            GiftId::Ring => 1e-5,
        };
        val += tie_bias;
        if val > best_val {
            best_val = val;
            best_id = id;
        }
    }
    assert!(best_val.is_finite() || best_val == f64::NEG_INFINITY);
    best_id
}

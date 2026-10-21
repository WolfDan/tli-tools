use crate::engine::{
    BEASTS, Beast, Contract, EscortKind, ObjectiveId, RAINBOW, SUPREME, SimState, ToolId,
};

use super::params::{CHAIN_ODDS_PROFILE, SolverParams};

/// Zero-allocation stack-resident memoization table for `feed`.
#[derive(Default)]
pub struct FeedMemo {
    map: heapless::FnvIndexMap<u128, f64, 64>,
}

impl FeedMemo {
    #[inline]
    pub fn new() -> Self {
        Self {
            map: heapless::FnvIndexMap::new(),
        }
    }

    #[inline]
    pub fn get(&self, key: u128) -> Option<f64> {
        self.map.get(&key).copied()
    }

    #[inline]
    pub fn insert(&mut self, key: u128, val: f64) {
        let _ = self.map.insert(key, val);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ContractNeed {
    pub contract: Contract,
    pub weight: f64,
}

#[derive(Copy, Clone, Debug)]
pub struct DietList {
    pub items: [u8; 4],
    pub len: u8,
}

impl DietList {
    #[inline]
    pub fn slice(&self) -> &[u8] {
        &self.items[..self.len as usize]
    }
}

pub struct Context {
    pub p: &'static SolverParams,
    pub eats_low: [DietList; 4],
    pub eats_high: [DietList; 4],
    pub counters_low: [DietList; 4],
    pub counters_high: [DietList; 4],
    pub double_from: u8,
    pub fight: bool,
    pub v: [f64; 6],
    pub r_val: f64,
    pub saw: bool,
    pub whip: bool,
    pub muzzle: bool,
    pub skewer: bool,
    pub crown: bool,
    pub maul: bool,
    pub guillotine: bool,
    pub collar: bool,
    pub revenge: bool,
    pub axe_doors: usize,
    pub dogtag: bool,
    pub cage: bool,
    pub blacklist: bool,
    pub shortchain: bool,
    pub pestle: bool,
    pub coin: bool,
    pub ravenous: bool,
    pub tongs: bool,
    pub fire: bool,
    pub forged: bool,
    pub scales: bool,
    pub trap: bool,
    pub top_tier: u8,
    pub maiden: Option<[u8; 5]>,
    pub scapegoat_loss: Option<f64>,
    pub stake: bool,
    pub swap_cap: i8,
    pub sealed: usize,
    pub lift: f64,
    pub lift2: f64,
    pub all_lift2: f64,
    pub shackles_lift: f64,
    pub goal_left: u8,
    pub scratch_extra: u8,
    pub open_dist: [f64; 5],
    pub open_value: f64,
    pub need: Option<ContractNeed>,
    pub chain_stop: bool,
    pub memo: FeedMemo,
}

#[inline]
pub fn composition(st: &SimState, exclude: i16) -> ([u8; 24], usize) {
    let mut comp = [0u8; 24];
    let mut crowd = 0usize;
    for c in &st.cells {
        if c.sealed {
            continue;
        }
        let Some(inmate) = c.inmate else {
            continue;
        };
        crowd += 1;
        if (c.n as i16) == exclude || inmate.tier >= RAINBOW {
            continue;
        }
        comp[(inmate.tier as usize) * 4 + inmate.beast.idx()] += 1;
    }
    (comp, crowd)
}

#[inline]
pub fn normalize5(weights: &[f64; 5]) -> [f64; 5] {
    let total: f64 = weights.iter().map(|w| w.max(0.0)).sum();
    let denom = if total > 0.0 { total } else { 1.0 };
    let mut out = [0.0; 5];
    for i in 0..5 {
        out[i] = weights[i].max(0.0) / denom;
    }
    out
}

#[inline]
pub fn floor_dist(dist: &[f64; 5], floor: u8) -> [f64; 5] {
    let mut kept = [0.0; 5];
    let mut any = false;
    for t in 0..5 {
        if (t as u8) >= floor {
            kept[t] = dist[t];
            if kept[t] > 0.0 {
                any = true;
            }
        }
    }
    if any { normalize5(&kept) } else { *dist }
}

#[inline]
pub fn rainbow_stuck(st: &SimState) -> bool {
    let Some(ch) = st.challenge.as_ref() else {
        return false;
    };
    !ch.settled && ch.deliveries_len < 3 && ch.vouchers < EscortKind::Heavy.cost()
}

#[inline]
pub fn contract_need(st: &SimState, p: &SolverParams) -> Option<ContractNeed> {
    let ch = st.challenge.as_ref()?;
    if ch.window_open || p.w_need == 0.0 {
        return None;
    }
    let slot = ch.deliveries_len as usize;
    if slot >= 2 {
        return None;
    }
    let contract = ch.contracts[slot];
    if (contract.deadline as i16) - (st.day as i16) >= (p.need_days as i16) {
        return None;
    }
    let must = slot == 1 && !ch.deliveries[0].success;
    Some(ContractNeed {
        contract,
        weight: p.w_need * if must { p.need_crit } else { 1.0 },
    })
}

impl Context {
    pub fn new(st: &SimState, p: &'static SolverParams) -> Self {
        let skewer = st.has_tool(ToolId::Skewer);
        let crown = st.has_tool(ToolId::Crown);
        let maul = st.has_tool(ToolId::Maul);

        let mut counters_low = [
            DietList {
                items: [1, 0, 0, 0],
                len: 1,
            },
            DietList {
                items: [2, 0, 0, 0],
                len: 1,
            },
            DietList {
                items: [3, 0, 0, 0],
                len: 1,
            },
            DietList {
                items: [0, 0, 0, 0],
                len: 1,
            },
        ];
        if skewer {
            counters_low[Beast::Rat.idx()] = DietList {
                items: [0, 3, 0, 0],
                len: 2,
            };
        }
        let mut eats_low = counters_low;
        if crown {
            eats_low[Beast::Lion.idx()] = DietList {
                items: [0, 1, 2, 3],
                len: 4,
            };
        }

        let add_second = |base: [DietList; 4]| -> [DietList; 4] {
            let mut out = base;
            for b in BEASTS {
                if let Some(sec) = b.second_meal() {
                    let s_idx = sec.idx() as u8;
                    let list = &mut out[b.idx()];
                    if !list.slice().contains(&s_idx) {
                        list.items[list.len as usize] = s_idx;
                        list.len += 1;
                    }
                }
            }
            out
        };

        let eats_high = add_second(eats_low);
        let counters_high = add_second(counters_low);

        let fight = st.cage_fight();
        let double_from = if fight { 0 } else { st.params.double_eat_tier };

        let mut v = [0.0; 6];
        for (t, slot) in v.iter_mut().enumerate() {
            *slot = p.tier_base.powi(t as i32);
        }
        let open_dist = normalize5(&st.params.tier_weights);
        let r_val = v[RAINBOW as usize]
            * p.rainbow_mult
            * if rainbow_stuck(st) {
                p.stuck_rainbow
            } else {
                1.0
            };

        let mut sealed = 0usize;
        let mut board_len = 0usize;
        let mut lift_sum = 0.0;
        let mut all_lift2 = 0.0;
        let mut tier_counts = [0u8; 5];
        let mut top_tier = 0u8;

        for c in &st.cells {
            if c.sealed {
                sealed += 1;
            } else if let Some(inmate) = c.inmate {
                if inmate.tier > top_tier {
                    top_tier = inmate.tier;
                }
                if inmate.tier < RAINBOW {
                    board_len += 1;
                    let t = inmate.tier as usize;
                    tier_counts[t] += 1;
                    let next1 = if inmate.tier + 1 >= RAINBOW {
                        r_val
                    } else {
                        v[t + 1]
                    };
                    let next2 = if inmate.tier + 2 >= RAINBOW {
                        r_val
                    } else {
                        v[t + 2]
                    };
                    lift_sum += next1 - v[t];
                    all_lift2 += next2 - v[t];
                }
            }
        }

        let denom = (board_len.max(1)) as f64;
        let lift = if board_len > 0 { lift_sum / denom } else { 0.0 };
        let lift2 = if board_len > 0 {
            all_lift2 / denom
        } else {
            0.0
        };

        let scapegoat_loss = if st.has_tool(ToolId::Scapegoat) && board_len >= 3 {
            let mut val_sum = 0.0;
            for c in &st.cells {
                if !c.sealed
                    && let Some(inmate) = c.inmate
                    && inmate.tier < RAINBOW
                {
                    val_sum += v[inmate.tier as usize];
                }
            }
            Some((2.0 * val_sum) / (board_len as f64))
        } else {
            None
        };

        let has_maiden = st.has_tool(ToolId::Maiden);
        let stake = st.has_tool(ToolId::Stake);
        let swap_cap = if p.swap_max_tier < 0 {
            -1
        } else if scapegoat_loss.is_some() {
            SUPREME as i8
        } else {
            let m_tier = if has_maiden {
                p.swap_max_tier.max(p.maiden_swap_tier)
            } else {
                p.swap_max_tier
            };
            m_tier.max(if stake { 2 } else { 0 })
        };

        let goal_left =
            if st.objective_id == ObjectiveId::Walk && !st.milestone && st.params.rewards {
                ObjectiveId::Walk
                    .need()
                    .saturating_sub(st.objective_progress)
            } else {
                0
            };

        let mut open_value = 0.0;
        for t in 0..5 {
            open_value += open_dist[t] * v[t];
        }

        Self {
            p,
            eats_low,
            eats_high,
            counters_low,
            counters_high,
            double_from,
            fight,
            v,
            r_val,
            saw: st.has_tool(ToolId::Saw),
            whip: st.has_tool(ToolId::Whip),
            muzzle: st.has_tool(ToolId::Muzzle),
            skewer,
            crown,
            maul,
            guillotine: st.has_tool(ToolId::Guillotine),
            collar: st.has_tool(ToolId::Collar),
            revenge: st.has_tool(ToolId::Revenge) && sealed > 0,
            axe_doors: if st.has_tool(ToolId::Axe) {
                sealed.min(2)
            } else {
                0
            },
            dogtag: st.has_tool(ToolId::Dogtag),
            cage: st.has_tool(ToolId::Cage),
            blacklist: st.has_tool(ToolId::Blacklist),
            shortchain: st.has_tool(ToolId::Shortchain),
            pestle: st.has_tool(ToolId::Pestle),
            coin: st.has_tool(ToolId::Coin),
            ravenous: st.has_tool(ToolId::Ravenous),
            tongs: st.has_tool(ToolId::Tongs),
            fire: st.has_tool(ToolId::Fire),
            forged: st.has_tool(ToolId::Forged),
            scales: st.has_tool(ToolId::Scales),
            trap: st.has_tool(ToolId::Trap),
            top_tier,
            maiden: if has_maiden { Some(tier_counts) } else { None },
            scapegoat_loss,
            stake,
            swap_cap,
            sealed,
            lift,
            lift2,
            all_lift2,
            shackles_lift: if st.has_tool(ToolId::Shackles) && board_len > 0 {
                lift
            } else {
                0.0
            },
            goal_left,
            scratch_extra: 1 + (st.params.scratch_stacks_base as u8),
            open_dist,
            open_value,
            need: contract_need(st, p),
            chain_stop: st.can_stop_chain(),
            memo: FeedMemo::new(),
        }
    }

    #[inline]
    pub fn eats_at(&self, t: u8) -> &[DietList; 4] {
        if t >= self.double_from {
            &self.eats_high
        } else {
            &self.eats_low
        }
    }

    #[inline]
    pub fn counters_at(&self, t: u8) -> &[DietList; 4] {
        if t >= self.double_from {
            &self.counters_high
        } else {
            &self.counters_low
        }
    }

    #[inline]
    pub fn min_meal(&self, t: u8, b: u8) -> u8 {
        if self.fight || (self.guillotine && b == (Beast::Wolf as u8)) {
            0
        } else {
            t
        }
    }

    #[inline]
    pub fn top_meal(&self, t: u8, b: u8) -> u8 {
        if self.maul && b == (Beast::Mammoth as u8) {
            RAINBOW - 1
        } else {
            t
        }
    }

    #[inline]
    pub fn meals_at(&self, t: u8, b: u8, mt: u8) -> DietList {
        if mt == t {
            self.eats_at(t)[b as usize]
        } else if mt < t && !self.fight && self.guillotine && b == (Beast::Wolf as u8) {
            DietList {
                items: [Beast::Rat as u8, 0, 0, 0],
                len: 1,
            }
        } else {
            self.counters_at(t)[b as usize]
        }
    }
}

#[inline]
pub(crate) fn pack_feed_key(t: u8, b: u8, crowd: usize, fight: bool, comp: &[u8; 24]) -> u128 {
    let mut k: u128 = (1u128 << 120) | (t as u128) | ((b as u128) << 4) | ((crowd as u128) << 8);
    let start = if fight { 0 } else { (t as usize) * 4 };
    let mut shift = 16;
    for &c in &comp[start..20] {
        k |= (c as u128) << shift;
        shift += 4;
    }
    k
}

pub fn swap_dist(st: &SimState, ctx: &Context, old_tier: u8) -> [f64; 6] {
    let mut w = st.params.swap_weights;
    if st.has_tool(ToolId::Rope) {
        w[0] = 0.0;
        w[1] = 0.0;
    }
    let mut dist = [0.0; 6];
    if ctx.scapegoat_loss.is_some() {
        let mut higher = [0.0; 5];
        let mut any = false;
        for t in 0..5 {
            if (t as u8) > old_tier {
                higher[t] = w[t];
                if higher[t] > 0.0 {
                    any = true;
                }
            }
        }
        if any {
            let norm = normalize5(&higher);
            dist[..5].copy_from_slice(&norm);
        } else {
            let target = (old_tier + 1).min(RAINBOW) as usize;
            dist[target] = 1.0;
        }
    } else {
        let norm = normalize5(&w);
        dist[..5].copy_from_slice(&norm);
    }
    if ctx.stake && old_tier < 3 {
        for p in &mut dist {
            *p *= 0.6;
        }
        dist[3] += 0.4;
    }
    dist
}

pub fn rise(ctx: &mut Context, t: u8, comp: &mut [u8; 24], crowd: usize, keep: i8) -> f64 {
    if t >= RAINBOW {
        return ctx.r_val;
    }
    let beasts_slice: &[u8] = match keep {
        0 => &[0],
        1 => &[1],
        2 => &[2],
        3 => &[3],
        _ => &[0, 1, 2, 3],
    };
    let mut sum = 0.0;
    for &b in beasts_slice {
        if ctx.revenge {
            let foods = ctx.eats_at(t)[b as usize];
            let mut spawned = 0.0;
            for &f in foods.slice() {
                let idx = (t as usize) * 4 + (f as usize);
                comp[idx] += 1;
                spawned += ctx.v[t as usize] + feed(ctx, t, b, comp, crowd + 1);
                comp[idx] -= 1;
            }
            sum += (0.3 * spawned) / (foods.len as f64) + 0.7 * feed(ctx, t, b, comp, crowd);
        } else {
            sum += feed(ctx, t, b, comp, crowd);
        }
    }
    sum / (beasts_slice.len() as f64)
}

pub fn feed(ctx: &mut Context, t: u8, b: u8, comp: &mut [u8; 24], crowd: usize) -> f64 {
    let key = pack_feed_key(
        t,
        b,
        if ctx.muzzle { crowd } else { 0 },
        ctx.fight || ctx.guillotine,
        comp,
    );
    if let Some(hit) = ctx.memo.get(key) {
        return hit;
    }
    let mut out = f64::NEG_INFINITY;
    let start_mt = ctx.min_meal(t, b);
    let end_mt = ctx.top_meal(t, b);
    for mt in start_mt..=end_mt {
        let list = ctx.meals_at(t, b, mt);
        for &f in list.slice() {
            let idx = (mt as usize) * 4 + (f as usize);
            if comp[idx] == 0 {
                continue;
            }
            comp[idx] -= 1;
            let val = bite(ctx, (t, b), comp, crowd, 0, true, mt);
            if val > out {
                out = val;
            }
            comp[idx] += 1;
        }
    }
    let base_v = ctx.v[t as usize];
    if out == f64::NEG_INFINITY || (ctx.chain_stop && out - base_v < ctx.p.stop_margin) {
        out = base_v;
    }
    ctx.memo.insert(key, out);
    out
}

pub fn bite(
    ctx: &mut Context,
    (t, b): (u8, u8),
    comp: &mut [u8; 24],
    crowd: usize,
    extra: u8,
    chained: bool,
    meal_tier: u8,
) -> f64 {
    let base_gain = if ctx.maul && b == (Beast::Mammoth as u8) {
        ((meal_tier as i16) + 2 - (t as i16)).max(1) as u8
    } else {
        1
    };
    let mut gain = base_gain + extra;
    if ctx.muzzle && crowd.saturating_sub(1) <= 4 {
        gain += 2;
    }
    if ctx.skewer && b == (Beast::Rat as u8) {
        gain += 1;
    }
    if ctx.crown && b == (Beast::Lion as u8) {
        gain += 2;
    }

    let keep = if ctx.fight { b as i8 } else { -1 };
    let next_crowd = crowd.saturating_sub(1);
    let s = if ctx.saw { 0.25 } else { 0.0 };
    let w = if ctx.whip { 0.35 } else { 0.0 };

    let sawed = |g: u8, ctx: &mut Context, comp: &mut [u8; 24]| -> f64 {
        if s > 0.0 || w > 0.0 {
            let r0 = rise(ctx, (t + g).min(RAINBOW), comp, next_crowd, keep);
            let r1 = rise(ctx, (t + g + 1).min(RAINBOW), comp, next_crowd, keep);
            let r2 = rise(ctx, (t + g + 2).min(RAINBOW), comp, next_crowd, keep);
            (1.0 - s) * (1.0 - w) * r0 + (s + w - 2.0 * s * w) * r1 + s * w * r2
        } else {
            rise(ctx, (t + g).min(RAINBOW), comp, next_crowd, keep)
        }
    };

    let after_collar = if chained && ctx.collar {
        0.2 * sawed(gain, ctx, comp) + 0.8 * sawed(gain + 1, ctx, comp)
    } else {
        sawed(gain, ctx, comp)
    };
    -ctx.v[meal_tier as usize] + after_collar
}

pub fn devour_delta(
    ctx: &mut Context,
    (tier, beast): (u8, u8),
    (meal_tier, meal): (u8, u8),
    scratch: bool,
    comp: &mut [u8; 24],
    crowd: usize,
    chained: bool,
) -> f64 {
    let i = (meal_tier as usize) * 4 + (meal as usize);
    if !scratch {
        comp[i] -= 1;
    }
    let c_arg = if scratch { crowd + 1 } else { crowd };
    let extra = if scratch { ctx.scratch_extra } else { 0 };
    let value =
        bite(ctx, (tier, beast), comp, c_arg, extra, chained, meal_tier) - ctx.v[tier as usize];
    if !scratch {
        comp[i] += 1;
    }
    value
}

pub fn best_meal(
    ctx: &mut Context,
    tier: u8,
    beast: u8,
    comp: &mut [u8; 24],
    crowd: usize,
) -> (f64, i8) {
    let mut best_val = f64::NEG_INFINITY;
    let mut best_m = -1i8;
    let start_mt = ctx.min_meal(tier, beast);
    let end_mt = ctx.top_meal(tier, beast);
    for mt in start_mt..=end_mt {
        let list = ctx.meals_at(tier, beast, mt);
        for &f in list.slice() {
            let idx = (mt as usize) * 4 + (f as usize);
            if comp[idx] == 0 {
                continue;
            }
            let val = devour_delta(ctx, (tier, beast), (mt, f), false, comp, crowd, false);
            if val > best_val {
                best_val = val;
                best_m = f as i8;
            }
        }
    }
    (best_val, best_m)
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ChainOdds {
    pub rainbow: f64,
    pub bites: f64,
}

pub fn chain_odds(st: &SimState, from: u8, to: u8) -> ChainOdds {
    let ctx = Context::new(st, &CHAIN_ODDS_PROFILE);
    let Some(eater) = st.cells[from as usize].inmate else {
        return ChainOdds::default();
    };
    let target = &st.cells[to as usize];
    let Some(meal) = (if target.sealed {
        target.hidden
    } else {
        target.inmate
    }) else {
        return ChainOdds::default();
    };
    let (mut comp, _) = composition(st, from as i16);
    if !target.sealed {
        let idx = (meal.tier as usize) * 4 + meal.beast.idx();
        if comp[idx] > 0 {
            comp[idx] -= 1;
        }
    }

    let mut memo_p = FeedMemo::new();
    let mut memo_n = FeedMemo::new();

    fn from_tier(
        ctx: &Context,
        t: u8,
        keep: i8,
        comp: &mut [u8; 24],
        memo_p: &mut FeedMemo,
        memo_n: &mut FeedMemo,
    ) -> (f64, f64) {
        if t >= RAINBOW {
            return (1.0, 0.0);
        }
        let beasts_slice: &[u8] = match keep {
            0 => &[0],
            1 => &[1],
            2 => &[2],
            3 => &[3],
            _ => &[0, 1, 2, 3],
        };
        let mut p_sum = 0.0;
        let mut n_sum = 0.0;
        for &b in beasts_slice {
            let (pb, nb) = eat(ctx, t, b, comp, memo_p, memo_n);
            p_sum += pb / (beasts_slice.len() as f64);
            n_sum += nb / (beasts_slice.len() as f64);
        }
        (p_sum, n_sum)
    }

    fn eat(
        ctx: &Context,
        t: u8,
        b: u8,
        comp: &mut [u8; 24],
        memo_p: &mut FeedMemo,
        memo_n: &mut FeedMemo,
    ) -> (f64, f64) {
        let key = pack_feed_key(t, b, 0, ctx.fight || ctx.guillotine, comp);
        if let (Some(p), Some(n)) = (memo_p.get(key), memo_n.get(key)) {
            return (p, n);
        }
        let mut best: Option<(f64, f64)> = None;
        let start_mt = ctx.min_meal(t, b);
        let end_mt = ctx.top_meal(t, b);
        for mt in start_mt..=end_mt {
            let list = ctx.meals_at(t, b, mt);
            for &f in list.slice() {
                let idx = (mt as usize) * 4 + (f as usize);
                if comp[idx] == 0 {
                    continue;
                }
                comp[idx] -= 1;
                let next_t = if ctx.maul && b == (Beast::Mammoth as u8) {
                    (t + 1).max(mt + 2)
                } else {
                    t + 1
                };
                let keep = if ctx.fight { b as i8 } else { -1 };
                let (pf, nf) = from_tier(ctx, next_t.min(RAINBOW), keep, comp, memo_p, memo_n);
                comp[idx] += 1;
                match best {
                    None => best = Some((pf, nf + 1.0)),
                    Some((bp, bn)) => {
                        if pf > bp + 1e-12 || ((pf - bp).abs() <= 1e-12 && nf + 1.0 > bn) {
                            best = Some((pf, nf + 1.0));
                        }
                    }
                }
            }
        }
        let (rp, rn) = best.unwrap_or((0.0, 0.0));
        memo_p.insert(key, rp);
        memo_n.insert(key, rn);
        (rp, rn)
    }

    let gain = if target.sealed && target.scratch {
        1 + ctx.scratch_extra
    } else if ctx.maul && eater.beast == Beast::Mammoth {
        ((meal.tier as i16) + 2 - (eater.tier as i16)).max(1) as u8
    } else {
        1
    };
    let keep = if ctx.fight {
        eater.beast.idx() as i8
    } else {
        -1
    };
    let (p_val, n_val) = from_tier(
        &ctx,
        (eater.tier + gain).min(RAINBOW),
        keep,
        &mut comp,
        &mut memo_p,
        &mut memo_n,
    );
    ChainOdds {
        rainbow: p_val,
        bites: 1.0 + n_val,
    }
}

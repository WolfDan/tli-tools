use crate::engine::{
    BEASTS, Beast, CELL_COUNT, Contract, EscortKind, ObjectiveId, RAINBOW, SUPREME, SimState,
    ToolId,
};

use super::params::{CHAIN_ODDS_PROFILE, SolverParams};

const DP_STACK_MAX: usize = 32;
const DP_ITERATIONS_MAX: usize = 1_000_000;

/// Zero-allocation stack-resident memoization table for `feed`.
#[repr(C, align(64))]
#[derive(Default)]
pub struct FeedMemo {
    map: heapless::FnvIndexMap<u128, f64, 64>,
}

const _: () = {
    assert!(std::mem::align_of::<FeedMemo>() == 64);
    assert!(RAINBOW == 5);
    assert!(CELL_COUNT == 16);
};

impl FeedMemo {
    #[inline]
    pub fn new() -> Self {
        let out = Self {
            map: heapless::FnvIndexMap::new(),
        };
        debug_assert!(out.map.is_empty());
        debug_assert!(out.map.capacity() == 64);
        out
    }

    #[inline]
    pub fn get(&self, key: u128) -> Option<f64> {
        debug_assert!(key != 0);
        debug_assert!(self.map.len() <= 64);
        self.map.get(&key).copied()
    }

    #[inline]
    pub fn insert(&mut self, key: u128, val: f64) {
        debug_assert!(key != 0);
        debug_assert!(val.is_finite());
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
        debug_assert!(self.len <= 4);
        debug_assert!((self.len as usize) <= self.items.len());
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
    assert!(st.cells.len() == CELL_COUNT);
    assert!((-1..(CELL_COUNT as i16)).contains(&exclude));
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
    assert!(crowd <= CELL_COUNT);
    (comp, crowd)
}

#[inline]
pub fn normalize5(weights: &[f64; 5]) -> [f64; 5] {
    assert!(weights.iter().all(|w| w.is_finite()));
    let weight_total: f64 = weights.iter().map(|w| w.max(0.0)).sum();
    let denom = if weight_total > 0.0 {
        weight_total
    } else {
        1.0
    };
    assert!(denom > 0.0);
    let mut out = [0.0; 5];
    for i in 0..5 {
        out[i] = weights[i].max(0.0) / denom;
    }
    out
}

#[inline]
pub fn floor_dist(dist: &[f64; 5], floor: u8) -> [f64; 5] {
    assert!(floor <= RAINBOW);
    assert!(dist.iter().all(|d| d.is_finite()));
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
    assert!(st.cells.len() == CELL_COUNT);
    assert!(EscortKind::Heavy.cost() > 0);
    let Some(ch) = st.challenge.as_ref() else {
        return false;
    };
    !ch.settled && ch.deliveries_len < 3 && ch.vouchers < EscortKind::Heavy.cost()
}

#[inline]
pub fn contract_need(st: &SimState, p: &SolverParams) -> Option<ContractNeed> {
    assert!(st.cells.len() == CELL_COUNT);
    assert!(p.w_need.is_finite());
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

struct DietTables {
    eats_low: [DietList; 4],
    eats_high: [DietList; 4],
    counters_low: [DietList; 4],
    counters_high: [DietList; 4],
}

fn add_second_meals(base: [DietList; 4]) -> [DietList; 4] {
    assert!(base.iter().all(|d| d.len <= 4));
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
    assert!(out.iter().all(|d| d.len <= 4));
    out
}

fn build_diet_tables(skewer: bool, crown: bool) -> DietTables {
    assert!(Beast::Rat.idx() < 4);
    assert!(Beast::Lion.idx() < 4);
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
    DietTables {
        eats_low,
        eats_high: add_second_meals(eats_low),
        counters_low,
        counters_high: add_second_meals(counters_low),
    }
}

struct BoardLiftStats {
    sealed: usize,
    board_len: usize,
    lift: f64,
    lift2: f64,
    all_lift2: f64,
    tier_counts: [u8; 5],
    top_tier: u8,
    scapegoat_loss: Option<f64>,
}

fn scan_board_lifts(st: &SimState, v: &[f64; 6], r_val: f64) -> BoardLiftStats {
    assert!(st.cells.len() == CELL_COUNT);
    assert!(r_val.is_finite());
    let mut sealed = 0usize;
    let mut board_len = 0usize;
    let mut lift_sum = 0.0;
    let mut all_lift2 = 0.0;
    let mut tier_counts = [0u8; 5];
    let mut top_tier = 0u8;
    let mut val_sum = 0.0;

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
                val_sum += v[t];
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
        Some((2.0 * val_sum) / (board_len as f64))
    } else {
        None
    };
    BoardLiftStats {
        sealed,
        board_len,
        lift,
        lift2,
        all_lift2,
        tier_counts,
        top_tier,
        scapegoat_loss,
    }
}

fn compute_swap_cap(p: &SolverParams, has_maiden: bool, stake: bool, scapegoat: bool) -> i8 {
    assert!(p.swap_max_tier >= -1);
    assert!(p.maiden_swap_tier >= -1);
    if p.swap_max_tier < 0 {
        -1
    } else if scapegoat {
        SUPREME as i8
    } else {
        let m_tier = if has_maiden {
            p.swap_max_tier.max(p.maiden_swap_tier)
        } else {
            p.swap_max_tier
        };
        m_tier.max(if stake { 2 } else { 0 })
    }
}

impl Context {
    fn base_from_diets(st: &SimState, p: &'static SolverParams, diets: DietTables) -> Self {
        assert!(st.cells.len() == CELL_COUNT);
        assert!(p.tier_base.is_finite());
        let fight = st.cage_fight();
        let double_from = if fight { 0 } else { st.params.double_eat_tier };
        let open_dist = normalize5(&st.params.tier_weights);
        Self {
            p,
            eats_low: diets.eats_low,
            eats_high: diets.eats_high,
            counters_low: diets.counters_low,
            counters_high: diets.counters_high,
            double_from,
            fight,
            v: [0.0; 6],
            r_val: 0.0,
            saw: st.has_tool(ToolId::Saw),
            whip: st.has_tool(ToolId::Whip),
            muzzle: st.has_tool(ToolId::Muzzle),
            skewer: st.has_tool(ToolId::Skewer),
            crown: st.has_tool(ToolId::Crown),
            maul: st.has_tool(ToolId::Maul),
            guillotine: st.has_tool(ToolId::Guillotine),
            collar: st.has_tool(ToolId::Collar),
            revenge: false,
            axe_doors: 0,
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
            top_tier: 0,
            maiden: None,
            scapegoat_loss: None,
            stake: st.has_tool(ToolId::Stake),
            swap_cap: 0,
            sealed: 0,
            lift: 0.0,
            lift2: 0.0,
            all_lift2: 0.0,
            shackles_lift: 0.0,
            goal_left: 0,
            scratch_extra: 1 + (st.params.scratch_stacks_base as u8),
            open_dist,
            open_value: 0.0,
            need: contract_need(st, p),
            chain_stop: st.can_stop_chain(),
            memo: FeedMemo::new(),
        }
    }

    pub fn new(st: &SimState, p: &'static SolverParams) -> Self {
        assert!(st.cells.len() == CELL_COUNT);
        assert!(p.tier_base.is_finite());
        let diets = build_diet_tables(st.has_tool(ToolId::Skewer), st.has_tool(ToolId::Crown));
        let mut ctx = Self::base_from_diets(st, p, diets);
        for (t, slot) in ctx.v.iter_mut().enumerate() {
            *slot = p.tier_base.powi(t as i32);
        }
        let stuck_mult = if rainbow_stuck(st) {
            p.stuck_rainbow
        } else {
            1.0
        };
        ctx.r_val = ctx.v[RAINBOW as usize] * p.rainbow_mult * stuck_mult;
        let stats = scan_board_lifts(st, &ctx.v, ctx.r_val);
        let has_maiden = st.has_tool(ToolId::Maiden);
        ctx.revenge = st.has_tool(ToolId::Revenge) && stats.sealed > 0;
        ctx.axe_doors = if st.has_tool(ToolId::Axe) {
            stats.sealed.min(2)
        } else {
            0
        };
        ctx.top_tier = stats.top_tier;
        ctx.maiden = if has_maiden {
            Some(stats.tier_counts)
        } else {
            None
        };
        ctx.scapegoat_loss = stats.scapegoat_loss;
        ctx.swap_cap = compute_swap_cap(p, has_maiden, ctx.stake, stats.scapegoat_loss.is_some());
        ctx.sealed = stats.sealed;
        ctx.lift = stats.lift;
        ctx.lift2 = stats.lift2;
        ctx.all_lift2 = stats.all_lift2;
        ctx.shackles_lift = if st.has_tool(ToolId::Shackles) && stats.board_len > 0 {
            stats.lift
        } else {
            0.0
        };
        ctx.goal_left =
            if st.objective_id == ObjectiveId::Walk && !st.milestone && st.params.rewards {
                ObjectiveId::Walk
                    .need()
                    .saturating_sub(st.objective_progress)
            } else {
                0
            };
        ctx.open_value = (0..5).map(|t| ctx.open_dist[t] * ctx.v[t]).sum();
        ctx
    }

    #[inline]
    pub fn eats_at(&self, t: u8) -> &[DietList; 4] {
        debug_assert!(t <= RAINBOW);
        debug_assert!(self.double_from <= RAINBOW + 1);
        if t >= self.double_from {
            &self.eats_high
        } else {
            &self.eats_low
        }
    }

    #[inline]
    pub fn counters_at(&self, t: u8) -> &[DietList; 4] {
        debug_assert!(t <= RAINBOW);
        debug_assert!(self.double_from <= RAINBOW + 1);
        if t >= self.double_from {
            &self.counters_high
        } else {
            &self.counters_low
        }
    }

    #[inline]
    pub fn meal_min(&self, t: u8, b: u8) -> u8 {
        debug_assert!(t <= RAINBOW);
        debug_assert!(b < 4);
        if self.fight || (self.guillotine && b == (Beast::Wolf as u8)) {
            0
        } else {
            t
        }
    }

    #[inline]
    pub fn top_meal(&self, t: u8, b: u8) -> u8 {
        debug_assert!(t < RAINBOW);
        debug_assert!(b < 4);
        if self.maul && b == (Beast::Mammoth as u8) {
            RAINBOW - 1
        } else {
            t
        }
    }

    #[inline]
    pub fn meals_at(&self, t: u8, b: u8, mt: u8) -> DietList {
        debug_assert!(t < RAINBOW);
        debug_assert!(b < 4 && mt < RAINBOW);
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
    debug_assert!(t < RAINBOW && b < 4);
    debug_assert!(crowd <= CELL_COUNT + 4);
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
    assert!(st.cells.len() == CELL_COUNT);
    assert!(old_tier <= RAINBOW);
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

#[derive(Copy, Clone)]
struct FeedNode {
    t: u8,
    b: u8,
    crowd: usize,
    started: bool,
    key: u128,
    meals: [(u8, u8); 20],
    meals_len: u8,
    meal_pos: u8,
    active_idx: Option<usize>,
    out: f64,
}

#[derive(Copy, Clone)]
struct BiteNode {
    meal_tier: u8,
    next_crowd: usize,
    keep: i8,
    collar_chain: bool,
    rise_tiers: [u8; 6],
    rise_len: u8,
    rise_pos: u8,
    rise_vals: [f64; 6],
    waiting: bool,
}

#[derive(Copy, Clone)]
struct RiseNode {
    t: u8,
    crowd: usize,
    beasts: [u8; 4],
    beasts_len: u8,
    beast_pos: u8,
    rev_pos: u8,
    active_rev_idx: Option<usize>,
    waiting: bool,
    spawned: f64,
    sum: f64,
}

#[derive(Copy, Clone)]
enum ChainFrame {
    Feed(FeedNode),
    Bite(BiteNode),
    Rise(RiseNode),
}

fn make_feed_node(t: u8, b: u8, crowd: usize) -> FeedNode {
    debug_assert!(t < RAINBOW);
    debug_assert!(b < 4);
    FeedNode {
        t,
        b,
        crowd,
        started: false,
        key: 0,
        meals: [(0, 0); 20],
        meals_len: 0,
        meal_pos: 0,
        active_idx: None,
        out: f64::NEG_INFINITY,
    }
}

fn make_rise_node(t: u8, crowd: usize, keep: i8) -> RiseNode {
    debug_assert!(t < RAINBOW);
    debug_assert!((-1..4).contains(&keep));
    let (beasts, beasts_len) = match keep {
        0..=3 => ([keep as u8, 0, 0, 0], 1u8),
        _ => ([0, 1, 2, 3], 4u8),
    };
    RiseNode {
        t,
        crowd,
        beasts,
        beasts_len,
        beast_pos: 0,
        rev_pos: 0,
        active_rev_idx: None,
        waiting: false,
        spawned: 0.0,
        sum: 0.0,
    }
}

fn make_bite_node(
    ctx: &Context,
    (t, b): (u8, u8),
    crowd: usize,
    extra: u8,
    chained: bool,
    meal_tier: u8,
) -> BiteNode {
    debug_assert!(t < RAINBOW && b < 4);
    debug_assert!(meal_tier < RAINBOW);
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
    let collar_chain = chained && ctx.collar;
    let mut rise_tiers = [0u8; 6];
    let rise_len = if ctx.saw || ctx.whip {
        rise_tiers[0] = (t + gain).min(RAINBOW);
        rise_tiers[1] = (t + gain + 1).min(RAINBOW);
        rise_tiers[2] = (t + gain + 2).min(RAINBOW);
        if collar_chain {
            rise_tiers[3] = (t + gain + 1).min(RAINBOW);
            rise_tiers[4] = (t + gain + 2).min(RAINBOW);
            rise_tiers[5] = (t + gain + 3).min(RAINBOW);
            6
        } else {
            3
        }
    } else {
        rise_tiers[0] = (t + gain).min(RAINBOW);
        if collar_chain {
            rise_tiers[1] = (t + gain + 1).min(RAINBOW);
            2
        } else {
            1
        }
    };
    BiteNode {
        meal_tier,
        next_crowd,
        keep,
        collar_chain,
        rise_tiers,
        rise_len,
        rise_pos: 0,
        rise_vals: [0.0; 6],
        waiting: false,
    }
}

fn step_feed_frame(
    ctx: &mut Context,
    comp: &mut [u8; 24],
    node: &mut FeedNode,
    last_val: &mut Option<f64>,
) -> Option<ChainFrame> {
    debug_assert!(node.t < RAINBOW);
    debug_assert!(node.b < 4);
    if !node.started {
        node.started = true;
        node.key = pack_feed_key(
            node.t,
            node.b,
            if ctx.muzzle { node.crowd } else { 0 },
            ctx.fight || ctx.guillotine,
            comp,
        );
        if let Some(hit) = ctx.memo.get(node.key) {
            *last_val = Some(hit);
            return None;
        }
        let start_mt = ctx.meal_min(node.t, node.b);
        let end_mt = ctx.top_meal(node.t, node.b);
        for mt in start_mt..=end_mt {
            let list = ctx.meals_at(node.t, node.b, mt);
            for &f in list.slice() {
                if comp[(mt as usize) * 4 + (f as usize)] > 0 {
                    node.meals[node.meals_len as usize] = (mt, f);
                    node.meals_len += 1;
                }
            }
        }
    }
    if let Some(idx) = node.active_idx.take() {
        let val = last_val.take().unwrap();
        if val > node.out {
            node.out = val;
        }
        comp[idx] += 1;
    }
    if node.meal_pos < node.meals_len {
        let (mt, f) = node.meals[node.meal_pos as usize];
        node.meal_pos += 1;
        let idx = (mt as usize) * 4 + (f as usize);
        comp[idx] -= 1;
        node.active_idx = Some(idx);
        let b_node = make_bite_node(ctx, (node.t, node.b), node.crowd, 0, true, mt);
        return Some(ChainFrame::Bite(b_node));
    }
    let base_v = ctx.v[node.t as usize];
    let mut out = node.out;
    if out == f64::NEG_INFINITY || (ctx.chain_stop && out - base_v < ctx.p.stop_margin) {
        out = base_v;
    }
    ctx.memo.insert(node.key, out);
    *last_val = Some(out);
    None
}

fn step_bite_frame(
    ctx: &Context,
    node: &mut BiteNode,
    last_val: &mut Option<f64>,
) -> Option<ChainFrame> {
    debug_assert!(node.meal_tier < RAINBOW);
    debug_assert!(node.rise_len <= 6);
    if node.waiting {
        node.rise_vals[node.rise_pos as usize] = last_val.take().unwrap();
        node.rise_pos += 1;
        node.waiting = false;
    }
    while node.rise_pos < node.rise_len {
        let rt = node.rise_tiers[node.rise_pos as usize];
        if rt >= RAINBOW {
            node.rise_vals[node.rise_pos as usize] = ctx.r_val;
            node.rise_pos += 1;
        } else {
            node.waiting = true;
            return Some(ChainFrame::Rise(make_rise_node(
                rt,
                node.next_crowd,
                node.keep,
            )));
        }
    }
    let s = if ctx.saw { 0.25 } else { 0.0 };
    let w = if ctx.whip { 0.35 } else { 0.0 };
    let eval_sawed = |r: &[f64]| -> f64 {
        if s > 0.0 || w > 0.0 {
            (1.0 - s) * (1.0 - w) * r[0] + (s + w - 2.0 * s * w) * r[1] + s * w * r[2]
        } else {
            r[0]
        }
    };
    let stride = if s > 0.0 || w > 0.0 { 3 } else { 1 };
    let after_collar = if node.collar_chain {
        0.2 * eval_sawed(&node.rise_vals[0..stride])
            + 0.8 * eval_sawed(&node.rise_vals[stride..2 * stride])
    } else {
        eval_sawed(&node.rise_vals[0..stride])
    };
    *last_val = Some(-ctx.v[node.meal_tier as usize] + after_collar);
    None
}

fn step_rise_frame(
    ctx: &Context,
    comp: &mut [u8; 24],
    node: &mut RiseNode,
    last_val: &mut Option<f64>,
) -> Option<ChainFrame> {
    debug_assert!(node.t < RAINBOW);
    debug_assert!((1..=4).contains(&node.beasts_len));
    if node.waiting {
        let val = last_val.take().unwrap();
        node.waiting = false;
        if let Some(r_idx) = node.active_rev_idx.take() {
            comp[r_idx] -= 1;
            node.spawned += ctx.v[node.t as usize] + val;
        } else {
            let b = node.beasts[node.beast_pos as usize];
            if ctx.revenge {
                let flen = ctx.eats_at(node.t)[b as usize].len as f64;
                node.sum += (0.3 * node.spawned) / flen + 0.7 * val;
            } else {
                node.sum += val;
            }
            node.beast_pos += 1;
            node.spawned = 0.0;
            node.rev_pos = 0;
        }
    }
    if node.beast_pos == node.beasts_len {
        *last_val = Some(node.sum / (node.beasts_len as f64));
        return None;
    }
    let b = node.beasts[node.beast_pos as usize];
    if ctx.revenge {
        let foods = ctx.eats_at(node.t)[b as usize];
        if node.rev_pos < foods.len {
            let rf = foods.items[node.rev_pos as usize];
            node.rev_pos += 1;
            let r_idx = (node.t as usize) * 4 + (rf as usize);
            comp[r_idx] += 1;
            node.active_rev_idx = Some(r_idx);
            node.waiting = true;
            return Some(ChainFrame::Feed(make_feed_node(node.t, b, node.crowd + 1)));
        }
    }
    node.waiting = true;
    Some(ChainFrame::Feed(make_feed_node(node.t, b, node.crowd)))
}

fn run_chain_stack(ctx: &mut Context, comp: &mut [u8; 24], root: ChainFrame) -> f64 {
    assert!(ctx.r_val.is_finite());
    let mut stack = [root; DP_STACK_MAX];
    let mut stack_len = 1usize;
    let mut last_val: Option<f64> = None;
    let mut iterations = 0usize;

    while stack_len > 0 {
        iterations += 1;
        assert!(iterations <= DP_ITERATIONS_MAX);
        let top_idx = stack_len - 1;
        let next_frame = match &mut stack[top_idx] {
            ChainFrame::Feed(node) => step_feed_frame(ctx, comp, node, &mut last_val),
            ChainFrame::Bite(node) => step_bite_frame(ctx, node, &mut last_val),
            ChainFrame::Rise(node) => step_rise_frame(ctx, comp, node, &mut last_val),
        };
        if let Some(child) = next_frame {
            assert!(stack_len < DP_STACK_MAX);
            stack[stack_len] = child;
            stack_len += 1;
        } else {
            stack_len -= 1;
        }
    }
    let out = last_val.unwrap();
    assert!(out.is_finite());
    out
}

pub fn rise(ctx: &mut Context, t: u8, comp: &mut [u8; 24], crowd: usize, keep: i8) -> f64 {
    assert!(t <= RAINBOW);
    assert!((-1..4).contains(&keep));
    if t >= RAINBOW {
        return ctx.r_val;
    }
    run_chain_stack(ctx, comp, ChainFrame::Rise(make_rise_node(t, crowd, keep)))
}

pub fn feed(ctx: &mut Context, t: u8, b: u8, comp: &mut [u8; 24], crowd: usize) -> f64 {
    assert!(t < RAINBOW);
    assert!(b < 4);
    run_chain_stack(ctx, comp, ChainFrame::Feed(make_feed_node(t, b, crowd)))
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
    assert!(t < RAINBOW && b < 4);
    assert!(meal_tier < RAINBOW);
    let node = make_bite_node(ctx, (t, b), crowd, extra, chained, meal_tier);
    run_chain_stack(ctx, comp, ChainFrame::Bite(node))
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
    assert!(tier < RAINBOW && beast < 4);
    assert!(meal_tier < RAINBOW && meal < 4);
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
    assert!(tier < RAINBOW);
    assert!(beast < 4);
    let mut best_val = f64::NEG_INFINITY;
    let mut best_m = -1i8;
    let start_mt = ctx.meal_min(tier, beast);
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

#[derive(Copy, Clone)]
struct OddsRiseNode {
    t: u8,
    beasts: [u8; 4],
    beasts_len: u8,
    beast_pos: u8,
    waiting: bool,
    p_sum: f64,
    n_sum: f64,
}

#[derive(Copy, Clone)]
struct OddsEatNode {
    t: u8,
    b: u8,
    started: bool,
    key: u128,
    meals: [(u8, u8); 20],
    meals_len: u8,
    meal_pos: u8,
    active_idx: Option<usize>,
    best: Option<(f64, f64)>,
}

#[derive(Copy, Clone)]
enum OddsFrame {
    Rise(OddsRiseNode),
    Eat(OddsEatNode),
}

fn make_odds_rise_node(t: u8, keep: i8) -> OddsRiseNode {
    debug_assert!(t <= RAINBOW);
    debug_assert!((-1..4).contains(&keep));
    let (beasts, beasts_len) = match keep {
        0..=3 => ([keep as u8, 0, 0, 0], 1u8),
        _ => ([0, 1, 2, 3], 4u8),
    };
    OddsRiseNode {
        t,
        beasts,
        beasts_len,
        beast_pos: 0,
        waiting: false,
        p_sum: 0.0,
        n_sum: 0.0,
    }
}

fn step_odds_rise_frame(
    node: &mut OddsRiseNode,
    last_val: &mut Option<(f64, f64)>,
) -> Option<OddsFrame> {
    debug_assert!(node.t <= RAINBOW);
    debug_assert!((1..=4).contains(&node.beasts_len));
    if node.t >= RAINBOW {
        *last_val = Some((1.0, 0.0));
        return None;
    }
    if node.waiting {
        let (pb, nb) = last_val.take().unwrap();
        node.waiting = false;
        node.p_sum += pb / (node.beasts_len as f64);
        node.n_sum += nb / (node.beasts_len as f64);
        node.beast_pos += 1;
    }
    if node.beast_pos == node.beasts_len {
        *last_val = Some((node.p_sum, node.n_sum));
        return None;
    }
    let b = node.beasts[node.beast_pos as usize];
    node.waiting = true;
    Some(OddsFrame::Eat(OddsEatNode {
        t: node.t,
        b,
        started: false,
        key: 0,
        meals: [(0, 0); 20],
        meals_len: 0,
        meal_pos: 0,
        active_idx: None,
        best: None,
    }))
}

fn step_odds_eat_frame(
    ctx: &Context,
    comp: &mut [u8; 24],
    memo_p: &mut FeedMemo,
    memo_n: &mut FeedMemo,
    node: &mut OddsEatNode,
    last_val: &mut Option<(f64, f64)>,
) -> Option<OddsFrame> {
    debug_assert!(node.t < RAINBOW);
    debug_assert!(node.b < 4);
    if !node.started {
        node.started = true;
        node.key = pack_feed_key(node.t, node.b, 0, ctx.fight || ctx.guillotine, comp);
        if let (Some(p), Some(n)) = (memo_p.get(node.key), memo_n.get(node.key)) {
            *last_val = Some((p, n));
            return None;
        }
        for mt in ctx.meal_min(node.t, node.b)..=ctx.top_meal(node.t, node.b) {
            let list = ctx.meals_at(node.t, node.b, mt);
            for &f in list.slice() {
                if comp[(mt as usize) * 4 + (f as usize)] > 0 {
                    node.meals[node.meals_len as usize] = (mt, f);
                    node.meals_len += 1;
                }
            }
        }
    }
    if let Some(idx) = node.active_idx.take() {
        let (pf, nf) = last_val.take().unwrap();
        comp[idx] += 1;
        match node.best {
            None => node.best = Some((pf, nf + 1.0)),
            Some((bp, bn)) => {
                if pf > bp + 1e-12 || ((pf - bp).abs() <= 1e-12 && nf + 1.0 > bn) {
                    node.best = Some((pf, nf + 1.0));
                }
            }
        }
    }
    if node.meal_pos < node.meals_len {
        let (mt, f) = node.meals[node.meal_pos as usize];
        node.meal_pos += 1;
        let idx = (mt as usize) * 4 + (f as usize);
        comp[idx] -= 1;
        node.active_idx = Some(idx);
        let next_t = if ctx.maul && node.b == (Beast::Mammoth as u8) {
            (node.t + 1).max(mt + 2)
        } else {
            node.t + 1
        };
        let keep = if ctx.fight { node.b as i8 } else { -1 };
        return Some(OddsFrame::Rise(make_odds_rise_node(
            next_t.min(RAINBOW),
            keep,
        )));
    }
    let (rp, rn) = node.best.unwrap_or((0.0, 0.0));
    memo_p.insert(node.key, rp);
    memo_n.insert(node.key, rn);
    *last_val = Some((rp, rn));
    None
}

fn eval_odds_from_tier(
    ctx: &Context,
    start_t: u8,
    start_keep: i8,
    comp: &mut [u8; 24],
) -> (f64, f64) {
    assert!(start_t <= RAINBOW);
    assert!((-1..4).contains(&start_keep));
    let mut memo_p = FeedMemo::new();
    let mut memo_n = FeedMemo::new();
    let root = OddsFrame::Rise(make_odds_rise_node(start_t, start_keep));
    let mut stack = [root; DP_STACK_MAX];
    let mut stack_len = 1usize;
    let mut last_val: Option<(f64, f64)> = None;
    let mut iterations = 0usize;

    while stack_len > 0 {
        iterations += 1;
        assert!(iterations <= DP_ITERATIONS_MAX);
        let top_idx = stack_len - 1;
        let next_frame = match &mut stack[top_idx] {
            OddsFrame::Rise(node) => step_odds_rise_frame(node, &mut last_val),
            OddsFrame::Eat(node) => {
                step_odds_eat_frame(ctx, comp, &mut memo_p, &mut memo_n, node, &mut last_val)
            }
        };
        if let Some(child) = next_frame {
            assert!(stack_len < DP_STACK_MAX);
            stack[stack_len] = child;
            stack_len += 1;
        } else {
            stack_len -= 1;
        }
    }
    last_val.unwrap()
}

pub fn chain_odds(st: &SimState, from: u8, to: u8) -> ChainOdds {
    assert!((from as usize) < CELL_COUNT);
    assert!((to as usize) < CELL_COUNT);
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
    let (p_val, n_val) =
        eval_odds_from_tier(&ctx, (eater.tier + gain).min(RAINBOW), keep, &mut comp);
    ChainOdds {
        rainbow: p_val,
        bites: 1.0 + n_val,
    }
}

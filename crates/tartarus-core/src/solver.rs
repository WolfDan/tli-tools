use crate::engine::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

include!("tool_combos.rs");

pub const PASS_WEIGHT: f64 = 1000.0;

pub const DEFAULT_TOOL_RANK: [ToolId; 31] = [
    ToolId::Trap,
    ToolId::Crown,
    ToolId::Rope,
    ToolId::Maul,
    ToolId::Claws,
    ToolId::Scapegoat,
    ToolId::Blacklist,
    ToolId::Ravenous,
    ToolId::Maiden,
    ToolId::Coin,
    ToolId::Revenge,
    ToolId::Shackles,
    ToolId::Muzzle,
    ToolId::Collar,
    ToolId::Stake,
    ToolId::Saw,
    ToolId::Hook,
    ToolId::Whip,
    ToolId::Skewer,
    ToolId::Dogtag,
    ToolId::Keys,
    ToolId::Guillotine,
    ToolId::Tongs,
    ToolId::Cage,
    ToolId::Fire,
    ToolId::Scales,
    ToolId::Shortchain,
    ToolId::Axe,
    ToolId::Pestle,
    ToolId::Forged,
    ToolId::Ironbox,
];

pub const CHALLENGE_TOOL_RANK: [ToolId; 31] = [
    ToolId::Rope,
    ToolId::Crown,
    ToolId::Maul,
    ToolId::Maiden,
    ToolId::Scapegoat,
    ToolId::Coin,
    ToolId::Shackles,
    ToolId::Ravenous,
    ToolId::Trap,
    ToolId::Skewer,
    ToolId::Claws,
    ToolId::Revenge,
    ToolId::Blacklist,
    ToolId::Collar,
    ToolId::Keys,
    ToolId::Cage,
    ToolId::Ironbox,
    ToolId::Muzzle,
    ToolId::Saw,
    ToolId::Whip,
    ToolId::Stake,
    ToolId::Guillotine,
    ToolId::Hook,
    ToolId::Tongs,
    ToolId::Dogtag,
    ToolId::Fire,
    ToolId::Axe,
    ToolId::Scales,
    ToolId::Pestle,
    ToolId::Shortchain,
    ToolId::Forged,
];

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SolverParams {
    pub tier_base: f64,
    pub rainbow_mult: f64,
    pub w_open: f64,
    pub w_now: f64,
    pub w_walk: f64,
    pub w_step: f64,
    pub w_nest: f64,
    pub w_chain: f64,
    pub w_goal: f64,
    pub w_instigate: f64,
    pub w_crow: f64,
    pub w_short: f64,
    pub w_cage: f64,
    pub reroll_margin: f64,
    pub w_stair: f64,
    pub min_devour: f64,
    pub stop_margin: f64,
    pub swap_max_tier: i8,
    pub swap_margin: f64,
    pub maiden_swap_tier: i8,
    pub exec_margin: f64,
    pub accept_chain: bool,
    pub tool_rank: [ToolId; 31],
    pub tool_sets: bool,
    pub office_override: Option<OfficeChoice>,
    pub keep1: f64,
    pub keep2: f64,
    pub reserve: u8,
    pub early_points: f64,
    pub stuck_rainbow: f64,
    pub min_left: u8,
    pub w_need: f64,
    pub need_days: u8,
    pub need_crit: f64,
}

pub const DEFAULT_SOLVER_PARAMS: SolverParams = SolverParams {
    tier_base: 3.0,
    rainbow_mult: 2.0,
    w_open: 1.0,
    w_now: 1.0,
    w_walk: 1.0,
    w_step: 0.0,
    w_nest: -6.0,
    w_chain: 4.0,
    w_goal: 1.0,
    w_instigate: 0.5,
    w_crow: 2.0,
    w_short: 0.1,
    w_cage: 1.0,
    reroll_margin: 0.02,
    w_stair: 0.0,
    min_devour: 0.0,
    stop_margin: 0.0,
    swap_max_tier: 0,
    swap_margin: 0.0,
    maiden_swap_tier: 3,
    exec_margin: 0.0,
    accept_chain: true,
    tool_rank: DEFAULT_TOOL_RANK,
    tool_sets: true,
    office_override: None,
    keep1: 0.0,
    keep2: 0.0,
    reserve: 1,
    early_points: 1e9,
    stuck_rainbow: 1.0,
    min_left: 1,
    w_need: 0.0,
    need_days: 3,
    need_crit: 2.0,
};

pub static REWARDS_PROFILE: SolverParams = SolverParams::rewards_profile();
pub static NOTOOLS_PROFILE: SolverParams = SolverParams::notools_profile();
pub static CHALLENGE_PROFILE: SolverParams = SolverParams::challenge_profile();
pub static CHAIN_ODDS_PROFILE: SolverParams = SolverParams {
    tier_base: 1.0,
    rainbow_mult: 1.0,
    ..DEFAULT_SOLVER_PARAMS
};

impl Default for SolverParams {
    fn default() -> Self {
        DEFAULT_SOLVER_PARAMS
    }
}

impl SolverParams {
    pub const fn rewards_profile() -> Self {
        Self {
            tier_base: 3.0,
            rainbow_mult: 1.5,
            w_open: 1.5,
            w_now: 0.844,
            w_walk: 0.75,
            w_step: 0.5,
            w_chain: 6.0,
            w_stair: 0.25,
            swap_max_tier: 3,
            min_devour: 0.0,
            exec_margin: 0.0,
            accept_chain: true,
            w_crow: 1.0,
            tool_rank: DEFAULT_TOOL_RANK,
            ..DEFAULT_SOLVER_PARAMS
        }
    }

    pub const fn notools_profile() -> Self {
        Self {
            tier_base: 2.75,
            rainbow_mult: 12.0,
            w_open: 2.25,
            w_now: 0.375,
            w_walk: 1.875,
            w_step: 0.5,
            w_chain: 24.0,
            w_stair: 0.125,
            swap_max_tier: 1,
            min_devour: 0.0,
            exec_margin: -5.0,
            accept_chain: false,
            w_crow: 1.0,
            tool_rank: DEFAULT_TOOL_RANK,
            ..DEFAULT_SOLVER_PARAMS
        }
    }

    pub const fn challenge_profile() -> Self {
        Self {
            tier_base: 2.25,
            rainbow_mult: 1.0,
            w_open: 2.0,
            w_now: 0.75,
            w_walk: 1.0,
            w_step: 0.5,
            w_chain: 8.0,
            w_stair: 0.5,
            swap_max_tier: 3,
            min_devour: -2.0,
            exec_margin: 5.0,
            accept_chain: true,
            tool_rank: CHALLENGE_TOOL_RANK,
            keep1: 1.0,
            keep2: 1.0,
            reserve: 3,
            early_points: 1e9,
            w_need: 40.0,
            need_days: 2,
            need_crit: 4.0,
            ..DEFAULT_SOLVER_PARAMS
        }
    }

    #[inline]
    pub fn for_state(st: &SimState) -> &'static Self {
        if st.mode == GameMode::Challenge {
            &CHALLENGE_PROFILE
        } else if st.params.rewards {
            &REWARDS_PROFILE
        } else {
            &NOTOOLS_PROFILE
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SimAction {
    Choose(ChoiceId),
    Reroll,
    Event {
        accept: bool,
    },
    Patrol {
        cell: u8,
    },
    Devour {
        to: u8,
    },
    Chain {
        to: u8,
    },
    StopChain,
    Hunt {
        to: u8,
    },
    Office {
        choice: OfficeChoice,
    },
    Swap,
    End,
    Deliver {
        cell_mask: u16,
        vault_ids: [u16; 10],
        vault_len: u8,
        escort: EscortKind,
    },
    Skip,
    Close,
    Early,
}

impl SimAction {
    pub fn type_name(&self) -> &'static str {
        match self {
            SimAction::Choose(_) => "choose",
            SimAction::Reroll => "reroll",
            SimAction::Event { .. } => "event",
            SimAction::Patrol { .. } => "patrol",
            SimAction::Devour { .. } => "devour",
            SimAction::Chain { .. } => "chain",
            SimAction::StopChain => "stopchain",
            SimAction::Hunt { .. } => "hunt",
            SimAction::Office { .. } => "office",
            SimAction::Swap => "swap",
            SimAction::End => "end",
            SimAction::Deliver { .. } => "deliver",
            SimAction::Skip => "skip",
            SimAction::Close => "close",
            SimAction::Early => "early",
        }
    }

    pub fn same_action(&self, other: &Self) -> bool {
        match (self, other) {
            (
                SimAction::Deliver {
                    cell_mask: m1,
                    vault_ids: v1,
                    vault_len: vl1,
                    escort: e1,
                },
                SimAction::Deliver {
                    cell_mask: m2,
                    vault_ids: v2,
                    vault_len: vl2,
                    escort: e2,
                },
            ) => {
                if e1 != e2 || m1 != m2 || vl1 != vl2 {
                    return false;
                }
                let vs1 = &v1[..*vl1 as usize];
                let vs2 = &v2[..*vl2 as usize];
                vs1.iter().all(|x| vs2.contains(x))
            }
            _ => self == other,
        }
    }
}

pub fn apply_action(st: &mut SimState, action: &SimAction) -> bool {
    match *action {
        SimAction::Choose(id) => st.choose(id),
        SimAction::Reroll => st.reroll_offer(),
        SimAction::Event { accept } => st.resolve_event(accept),
        SimAction::Patrol { cell } => st.patrol(cell),
        SimAction::Devour { to } => st.devour(st.position, to),
        SimAction::Chain { to } => st.chain_bite(to),
        SimAction::StopChain => st.stop_chain(),
        SimAction::Hunt { to } => st.hunt(to),
        SimAction::Office { choice } => st.office_act(choice),
        SimAction::Swap => st.swap(st.position),
        SimAction::End => st.end_day(),
        SimAction::Deliver {
            cell_mask,
            ref vault_ids,
            vault_len,
            escort,
        } => st.deliver_mask(
            cell_mask,
            &vault_ids[..vault_len as usize],
            escort,
        ),
        SimAction::Skip => st.skip_delivery(),
        SimAction::Close => st.cancel_early_delivery(),
        SimAction::Early => st.open_early_delivery(),
    }
}

/// Zero-allocation stack-resident memoization table for `feed`.
pub struct FeedMemo {
    table: [u8; 128],
    keys: [u128; 64],
    vals: [f64; 64],
    len: usize,
}

impl FeedMemo {
    #[inline]
    pub fn new() -> Self {
        Self {
            table: [0; 128],
            keys: [0; 64],
            vals: [0.0; 64],
            len: 0,
        }
    }

    #[inline]
    fn hash(key: u128) -> usize {
        let lo = key as u64;
        let hi = (key >> 64) as u64;
        let h = (lo ^ hi.rotate_left(29)).wrapping_mul(0x9e3779b97f4a7c15);
        (h >> 57) as usize
    }

    #[inline]
    pub fn get(&self, key: u128) -> Option<f64> {
        let mut idx = Self::hash(key);
        for _ in 0..8 {
            let slot = self.table[idx];
            if slot == 0 {
                return None;
            }
            let e = (slot - 1) as usize;
            if self.keys[e] == key {
                return Some(self.vals[e]);
            }
            idx = (idx + 1) & 127;
        }
        None
    }

    #[inline]
    pub fn insert(&mut self, key: u128, val: f64) {
        if self.len >= 64 {
            return;
        }
        let mut idx = Self::hash(key);
        for _ in 0..8 {
            let slot = self.table[idx];
            if slot == 0 {
                let e = self.len;
                self.len += 1;
                self.keys[e] = key;
                self.vals[e] = val;
                self.table[idx] = (e + 1) as u8;
                return;
            }
            let e = (slot - 1) as usize;
            if self.keys[e] == key {
                self.vals[e] = val;
                return;
            }
            idx = (idx + 1) & 127;
        }
    }
}

impl Default for FeedMemo {
    fn default() -> Self {
        Self::new()
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
    if any {
        normalize5(&kept)
    } else {
        *dist
    }
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
        for t in 0..6 {
            v[t] = p.tier_base.powi(t as i32);
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
                if !c.sealed {
                    if let Some(inmate) = c.inmate {
                        if inmate.tier < RAINBOW {
                            val_sum += v[inmate.tier as usize];
                        }
                    }
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
fn pack_feed_key(t: u8, b: u8, crowd: usize, fight: bool, comp: &[u8; 24]) -> u128 {
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
            let val = bite(ctx, t, b, comp, crowd, 0, true, mt);
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

#[allow(clippy::too_many_arguments)]
pub fn bite(
    ctx: &mut Context,
    t: u8,
    b: u8,
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

#[allow(clippy::too_many_arguments)]
pub fn devour_delta(
    ctx: &mut Context,
    tier: u8,
    beast: u8,
    meal: u8,
    scratch: bool,
    comp: &mut [u8; 24],
    crowd: usize,
    chained: bool,
    meal_tier: u8,
) -> f64 {
    let i = (meal_tier as usize) * 4 + (meal as usize);
    if !scratch {
        comp[i] -= 1;
    }
    let c_arg = if scratch { crowd + 1 } else { crowd };
    let extra = if scratch { ctx.scratch_extra } else { 0 };
    let value =
        bite(ctx, tier, beast, comp, c_arg, extra, chained, meal_tier) - ctx.v[tier as usize];
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
            let val = devour_delta(ctx, tier, beast, f, false, comp, crowd, false, mt);
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

#[inline]
pub fn stair(ctx: &Context, comp: &[u8; 24]) -> f64 {
    let mut value = 0.0;
    for t in 0..(RAINBOW as usize) {
        let mut fed = 0;
        let eats = ctx.eats_at(t as u8);
        for b in 0..4 {
            if eats[b].slice().iter().any(|&f| comp[t * 4 + (f as usize)] > 0) {
                fed += 1;
            }
        }
        value += (fed as f64 / 4.0) * ctx.v[t];
    }
    value
}

#[inline]
pub fn stair_delta(ctx: &Context, comp: &mut [u8; 24], tier: u8, beast: u8, sign: i8) -> f64 {
    if ctx.p.w_stair == 0.0 || tier >= RAINBOW {
        return 0.0;
    }
    let before = stair(ctx, comp);
    let idx = (tier as usize) * 4 + (beast as usize);
    comp[idx] = (comp[idx] as i16 + sign as i16) as u8;
    let after = stair(ctx, comp);
    comp[idx] = (comp[idx] as i16 - sign as i16) as u8;
    ctx.p.w_stair * (after - before)
}

#[inline]
pub fn stair_fresh(ctx: &Context, dist: &[f64], comp: &mut [u8; 24]) -> f64 {
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
    sum
}

pub fn maiden_bonus(st: &SimState, ctx: &Context, tier: u8) -> (f64, f64) {
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
    (chance, value)
}

#[inline]
pub fn coin_pair_ev(ctx: &Context, tier: u8) -> f64 {
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

pub fn swap_value(st: &SimState, ctx: &Context, tier: u8) -> f64 {
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
    value + maiden_bonus(st, ctx, tier).1
}

pub fn swap_value_with_comp(
    st: &SimState,
    ctx: &mut Context,
    tier: u8,
    beast: u8,
    comp: &mut [u8; 24],
    crowd: usize,
) -> f64 {
    if !st.used.devour_after_swap && !ctx.pestle {
        return swap_value(st, ctx, tier);
    }
    let dist = swap_dist(st, ctx, tier);
    let mut maiden_counts = ctx.maiden;
    if let Some(ref mut counts) = maiden_counts {
        if (tier as usize) < 5 && counts[tier as usize] > 0 {
            counts[tier as usize] -= 1;
        }
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
        let mut eff_t = t as u8;
        if let Some(ref counts) = maiden_counts {
            if t < 5 {
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
        }

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
    value
}

#[inline]
pub fn trickster_value(ctx: &Context, dist: &[f64; 5]) -> f64 {
    let mut sum = 0.0;
    for (t, &p) in dist.iter().enumerate() {
        let cur = ctx.v[t.min(SUPREME as usize)];
        sum += p * (ctx.v[SUPREME as usize] - cur).max(0.0);
    }
    sum
}

pub fn appetite_value(ctx: &Context, low: u8, dist: &[f64; 5]) -> f64 {
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
    sum
}

pub fn hunt_pair_ev(st: &SimState, ctx: &Context, lion_tier: u8, prey_tier: u8) -> f64 {
    let eff_rounds = st.params.prey_rounds.min(st.max_days.saturating_sub(st.day));
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
    let mut sum = 0.0;
    for (t, &p) in dist.iter().enumerate() {
        if p == 0.0 {
            continue;
        }
        for b in 0..4u8 {
            sum += (p / 4.0) * activation(st, ctx, t as u8, b, comp, crowd, swap_left);
        }
    }
    sum
}

#[inline]
pub fn tagged(ctx: &Context, dist: &[f64; 5]) -> [f64; 5] {
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
    let mut worth = 0.0;
    for (t, &p) in dist.iter().enumerate() {
        worth += p * ctx.v[t];
    }
    ctx.p.w_open * worth
        + ctx.p.w_now * fresh_activation(st, ctx, dist, comp, crowd, swap_left)
        + stair_fresh(ctx, dist, comp)
}

pub fn execution_value(st: &SimState, ctx: &Context, tier: u8, beast: u8, comp: &[u8; 24]) -> f64 {
    let same = st.params.execution_same_tier;
    let prey = BEASTS[beast as usize].primary_prey().idx();
    let mut count = 0u8;
    let mut lost = 0.0;
    let max_t = if same {
        tier as i16
    } else {
        (tier as i16) - 1
    };
    for t in 0..=max_t {
        if same && t != (tier as i16) {
            continue;
        }
        let k = comp[(t as usize) * 4 + prey];
        count += k;
        lost += (k as f64) * ctx.v[t as usize];
    }
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
            for i in 0..20 {
                if comp[i] > 0 {
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

pub fn event_worth_of_cell(
    st: &SimState,
    ctx: &mut Context,
    board_comp: &[u8; 24],
    cell_n: u8,
    kind: EventKind,
) -> f64 {
    let c = &st.cells[cell_n as usize];
    let known_inmate = if !c.sealed {
        c.inmate
    } else if c.known >= 2 {
        c.hidden
    } else {
        None
    };

    if let Some(inm) = known_inmate {
        let tier = inm.tier;
        let beast = inm.beast.idx() as u8;
        let rem_sealed = ctx.sealed.saturating_sub(usize::from(c.sealed));
        match kind {
            EventKind::Execution => {
                let (kcomp, _) = composition(st, cell_n as i16);
                return (execution_value(st, ctx, tier, beast, &kcomp) - ctx.p.exec_margin).max(0.0);
            }
            EventKind::Trickster => {
                if tier >= SUPREME {
                    return 0.0;
                }
                return (ctx.v[SUPREME as usize] - ctx.v[tier as usize]).max(0.0);
            }
            EventKind::Fortune => {
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
                return (to_v - ctx.v[tier as usize]).max(0.0);
            }
            EventKind::Archenemy => {
                if rem_sealed == 0 {
                    return 0.0;
                }
                let m_tier = tier.min(SUPREME);
                let spawn_v = ctx.p.w_open * ctx.v[m_tier as usize];
                if st.can_devour_now() && tier < RAINBOW {
                    let (mut kcomp, kcrowd) = composition(st, cell_n as i16);
                    let food = inm.beast.primary_prey().idx() as u8;
                    kcomp[(m_tier as usize) * 4 + (food as usize)] += 1;
                    let dev = devour_delta(
                        ctx,
                        tier,
                        beast,
                        food,
                        false,
                        &mut kcomp,
                        kcrowd + 1,
                        false,
                        m_tier,
                    );
                    return spawn_v + (ctx.p.w_now * dev).max(ctx.lift);
                }
                return spawn_v + ctx.lift;
            }
            EventKind::Transfer => {
                if rem_sealed == 0 {
                    return 0.0;
                }
                let base_v = ctx.p.w_open * ctx.v[tier.min(SUPREME) as usize];
                return base_v + 0.5 * ctx.lift + 0.5 * coin_pair_ev(ctx, tier);
            }
            _ => {}
        }
    } else if c.sealed && c.known == 1 {
        if let Some(h) = c.hidden {
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
    }

    event_worth_of(st, ctx, board_comp, kind)
}

pub fn expected_unknown_event_worth(st: &SimState, ctx: &Context, comp: &[u8; 24]) -> f64 {
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
    if w_sum > 0.0 {
        v_sum / w_sum
    } else {
        0.0
    }
}

pub fn event_why(kind: EventKind) -> &'static str {
    match kind {
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
        EventKind::Ratway => {
            "the Sewer Ratway waits there: a free move to any cell but the office"
        }
        EventKind::Hunt => {
            "\"Treasure\" Hunt waits there: every locked door holding a Normal or Magic shows its rarity, so the rest hold Rare or better"
        }
        EventKind::Portion => "Extra Portion waits there: up to 3 random inmates rise a tier",
        EventKind::Fortune => {
            "Shared Fortune waits there: this inmate rises a tier for each other inmate of the same beast"
        }
        EventKind::Archenemy => {
            "Archenemy waits there: a locked cell opens with a guaranteed meal for this inmate"
        }
        EventKind::Transfer => {
            "Transfer of Convicts waits there: this inmate is replicated into a random locked cell"
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OfficeValues {
    pub bestow: f64,
    pub instigate: f64,
    pub tyranny: f64,
}

pub fn office_values(st: &SimState, ctx: &Context, comp: &[u8; 24]) -> OfficeValues {
    let mut bestow = 0.0;
    let mut count = 0usize;
    for c in &st.cells {
        if c.sealed {
            continue;
        }
        if let Some(inmate) = c.inmate {
            if inmate.tier < RAINBOW {
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
    }
    bestow /= (count.max(1)) as f64;
    let free = st
        .cells
        .iter()
        .filter(|c| c.sealed && c.event.is_none() && c.nest_due.is_none())
        .count();
    let each = expected_unknown_event_worth(st, ctx, comp);
    let days_rem = st.max_days.saturating_sub(st.day) as f64;
    let horizon_inst = (days_rem / 6.0).clamp(0.0, 1.0);
    let extra = if st.start_event == StartEventId::Tyranny {
        3usize
    } else {
        0usize
    };
    let tyranny = if extra > 0 {
        (extra as f64) * bestow
            + ((extra * INSTIGATE_EVENTS).min(free) as f64) * each * ctx.p.w_instigate * horizon_inst
    } else {
        0.0
    };
    let mut instigate = (INSTIGATE_EVENTS.min(free) as f64) * each * ctx.p.w_instigate * horizon_inst;
    if !st.milestone && st.params.rewards && st.objective_id == ObjectiveId::Incident && free > 0 {
        let need_prog = ObjectiveId::Incident.need();
        if st.objective_progress < need_prog {
            let rem = (need_prog - st.objective_progress) as f64;
            instigate += (0.40 / rem.max(1.0)) * ctx.lift;
        }
    }
    OfficeValues {
        bestow,
        instigate,
        tyranny,
    }
}

pub fn contract_gap(contract: Contract, comp: &[u8; 24]) -> i32 {
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
    match contract.id {
        ContractId::Mixed => (3 - size).max(0) + (2 - beasts).max(0),
        ContractId::Cohort => (4 - size).max(0),
        ContractId::Veteran => (1 - elite).max(0) + (2 - low).max(0),
        ContractId::Elite => (2 - elite).max(0) + (2 - beasts).max(0),
        ContractId::Kin => kin,
        ContractId::Muster => (3 - elite).max(0),
        ContractId::Heavy | ContractId::Bounty => 0,
    }
}

pub fn need_value(
    ctx: &Context,
    comp: &mut [u8; 24],
    removes: &[(u8, u8)],
    add_dist: Option<&[f64]>,
) -> f64 {
    let Some(need) = ctx.need else {
        return 0.0;
    };
    let before = contract_gap(need.contract, comp) as f64;
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

pub fn patrol_scores_fast(st: &SimState, ctx: &mut Context) -> ([PatrolCellScore; 16], usize) {
    let p = ctx.p;
    let (mut board_comp, board_crowd) = composition(st, -1);
    let swap_left = st.can_swap_now();
    let mut fresh_cached: Option<f64> = None;

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
            let dest_val = if c.sealed {
                if let Some(fv) = fresh_cached {
                    fv
                } else {
                    let tg = tagged(ctx, &ctx.open_dist);
                    let fv = fresh_value(st, ctx, &tg, &mut board_comp, board_crowd + 1, swap_left);
                    fresh_cached = Some(fv);
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
                        swap_left,
                    )
            } else {
                0.0
            };
            score += p.w_open * ((sealed_cnt - (c.sealed as usize)) as f64) * ctx.open_value + dest_val;
        } else if c.sealed && c.nest_due.is_some() {
            score += p.w_nest;
        } else if c.sealed {
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
            let need = need_value(ctx, &mut board_comp, &[], Some(&need_dist));
            let fv = if let Some(d) = dist_opt {
                let tg = tagged(ctx, &d);
                fresh_value(st, ctx, &tg, &mut board_comp, board_crowd + 1, swap_left)
            } else if let Some(cached) = fresh_cached {
                cached
            } else {
                let tg = tagged(ctx, &open_dist_copy);
                let val = fresh_value(st, ctx, &tg, &mut board_comp, board_crowd + 1, swap_left);
                fresh_cached = Some(val);
                val
            };
            score += fv + need;
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
            let unk_ev = expected_unknown_event_worth(st, ctx, &board_comp);
            if !c.seen {
                score += (st.params.event_chance / 100.0) * unk_ev;
            } else if c.event.is_some() && !c.event_known {
                score += unk_ev;
            } else if let Some(ev) = c.event {
                score += event_worth_of_cell(st, ctx, &board_comp, n, ev);
            }
        } else if n == OFFICE {
            let office = office_values(st, ctx, &board_comp);
            score += office.bestow.max(office.instigate) + office.tyranny;
        } else if let Some(inmate) = c.inmate {
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
                    if oc.n != c.n && !oc.sealed {
                        if let Some(meal) = oc.inmate {
                            if st.can_eat(inmate, meal) && st.link_of(meal.id).is_none() {
                                let hv = hunt_pair_ev(st, ctx, inmate.tier, meal.tier);
                                if hv > act_val {
                                    act_val = hv;
                                }
                            }
                        }
                    }
                }
            }
            score += p.w_walk * act_val;
            if let Some(ev) = c.event {
                if !c.event_known {
                    score += expected_unknown_event_worth(st, ctx, &board_comp);
                } else {
                    score += event_worth_of_cell(st, ctx, &board_comp, n, ev);
                }
            }
        }

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
            if st.objective_id == ObjectiveId::Doors
                && c.sealed
                && st.objective_progress < need_prog
            {
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
        if len < 16 {
            out[len] = PatrolCellScore { cell: n, score };
            len += 1;
        }
    }

    out[..len].sort_by(|a, b| b.score.total_cmp(&a.score));
    (out, len)
}

pub fn patrol_cell_why(st: &SimState, ctx: &mut Context, n: u8) -> String {
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
        format!("it destroys the rat nest due on day {}", c.nest_due.unwrap())
    } else if c.sealed {
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
        let need = need_value(ctx, &mut board_comp, &[], Some(&need_dist));
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
    } else if n == OFFICE {
        let office = office_values(st, ctx, &board_comp);
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
        if c.event.is_some() {
            if !c.event_known {
                format!(
                    "a special event waits in the cell with its {}",
                    describe_inmate(inmate)
                )
            } else {
                event_why(c.event.unwrap()).replace("behind the door", "in the cell")
            }
        } else {
            format!("its {} can act today", describe_inmate(inmate))
        }
    } else {
        "an empty cell".to_string()
    }
}

pub fn link_value(st: &SimState, ctx: &Context, link: PreyLink) -> f64 {
    let eff_rounds = link.rounds.min(st.max_days.saturating_sub(st.day));
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
    value
}

pub fn link_loss(st: &SimState, ctx: &Context, inmate: Option<Inmate>, certain: bool) -> f64 {
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
        // and >= 0.80 ensures a Lion that chose Hunt over Devour won't immediately Devour on step 2).
        0.82 * link_value(st, ctx, link)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DevourScore {
    pub cell: u8,
    pub score: f64,
}

pub fn devour_scores(st: &SimState, ctx: &mut Context) -> ([DevourScore; 16], usize) {
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
    let own = link_loss(st, ctx, Some(eater), false);
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

    for (idx, &o) in opts[..olen].iter().enumerate() {
        let target = &st.cells[o.cell as usize];
        let meal = if o.scratch {
            target.hidden.unwrap()
        } else {
            target.inmate.unwrap()
        };
        let meal_link = if o.scratch {
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
        let stair_loss = if o.scratch {
            0.0
        } else {
            stair_delta(ctx, &mut comp, meal.tier, meal.beast.idx() as u8, -1)
        };
        let mut need = 0.0;
        if let Some(ref mut w_comp) = whole {
            let mut removes = [(tier, beast), (0, 0)];
            let rlen = if o.scratch {
                1
            } else {
                removes[1] = (meal.tier, meal.beast.idx() as u8);
                2
            };
            let to = (tier + st.gain_for(st.position, o)).min(RAINBOW);
            let mut add_d = [0.0; 6];
            add_d[to as usize] = 1.0;
            need = need_value(ctx, w_comp, &removes[..rlen], Some(&add_d));
        }
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
        let m_tier = if o.scratch { tier } else { meal.tier };
        let coin_loss = if !o.scratch
            && ctx.coin
            && m_tier >= 2
            && m_tier < RAINBOW
            && comp[(m_tier as usize) * 4 + meal.beast.idx()] == 1
        {
            0.40 * coin_pair_ev(ctx, m_tier)
        } else {
            0.0
        };
        let delta = devour_delta(
            ctx,
            tier,
            beast,
            meal.beast.idx() as u8,
            o.scratch,
            &mut comp,
            crowd,
            chained,
            m_tier,
        );
        out[idx] = DevourScore {
            cell: o.cell,
            score: delta - loss - coin_loss + stair_loss + need + cage + obj_bonus,
        };
    }

    out[..olen].sort_by(|a, b| b.score.total_cmp(&a.score));
    (out, olen)
}

pub fn swap_gain_here(st: &SimState, ctx: &mut Context) -> f64 {
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
        let mut board_len = 0usize;
        let mut lift_sum = 0.0;
        for c in &st.cells {
            if !c.sealed {
                if let Some(i) = c.inmate {
                    if i.tier < RAINBOW {
                        board_len += 1;
                        let t = i.tier as usize;
                        lift_sum += ctx.v[t + 1] - ctx.v[t];
                    }
                }
            }
        }
        let lift = lift_sum / ((board_len.max(1)) as f64);
        gain += ((st.params.ivory_count as usize).min(board_len) as f64) * lift;
        return gain;
    }
    if (inmate.tier as i8) <= ctx.swap_cap {
        gain
    } else if inmate.tier == 2
        && st.can_devour_now()
        && st.used.devour_after_swap
        && gain > 0.08
        && best_meal(ctx, inmate.tier, inmate.beast.idx() as u8, &mut kcomp, kcrowd).0 <= 0.0
    {
        gain
    } else {
        f64::NEG_INFINITY
    }
}

pub fn board_value(st: &SimState, ctx: &mut Context) -> f64 {
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
            if !c.sealed {
                if let Some(top) = c.inmate.filter(|i| i.tier == ctx.top_tier) {
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
                        if !ac.sealed {
                            if let Some(victim) = ac.inmate.filter(|v| v.tier < top.tier) {
                                let net =
                                    (to_v - ctx.v[top.tier as usize] - ctx.v[victim.tier as usize])
                                        .max(0.0);
                                if net > best_adj {
                                    best_adj = net;
                                }
                            }
                        }
                    }
                    value += 0.45 * best_adj;
                }
            }
        }
    }
    for &link in &st.prey[..st.prey_len as usize] {
        value += 0.50 * link_value(st, ctx, link);
    }
    value
}

pub fn pick_gift(
    st: &SimState,
    p: &'static SolverParams,
    options: &[ChoiceId],
    rng: &mut u32,
) -> ChoiceId {
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
    best_id
}

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

    #[allow(clippy::too_many_arguments)]
    fn walk<F>(
        i: usize,
        pressure: u16,
        glen: usize,
        groups: &[CargoGroup; 24],
        counts: &mut [u8; 24],
        cap: u16,
        bounty: i8,
        contract: Contract,
        escorts: &[EscortKind],
        ctx: &Context,
        rate: &F,
        best: &mut Option<(ConvoyPlan, [u8; 24])>,
    ) where
        F: Fn(&ConvoyPlan) -> f64,
    {
        if i == glen {
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

            for idx in 0..glen {
                let k = counts[idx];
                if k == 0 {
                    continue;
                }
                let g = &groups[idx];
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
                let bonus = if g.tier >= SUPREME && (g.beast as i8) == bounty {
                    8u16
                } else {
                    0u16
                };
                points += (k as u16) * ((1u16 << g.tier) + bonus);
                press += (k as u16) * PRESSURE[g.tier as usize];
                let v_t = if g.tier >= RAINBOW {
                    ctx.r_val
                } else {
                    ctx.v[g.tier as usize]
                };
                kept += (k as f64) * v_t;
            }
            if size == 0 {
                return;
            }
            let beasts = mask.count_ones();
            let fits = match contract.id {
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
            for &escort in escorts {
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
                let sc = rate(&plan);
                if sc > f64::NEG_INFINITY && (best.is_none() || sc > best.as_ref().unwrap().0.score)
                {
                    plan.score = sc;
                    *best = Some((plan, *counts));
                }
            }
            return;
        }
        let g = &groups[i];
        let p_step = PRESSURE[g.tier as usize];
        for k in 0..=g.refs_len {
            let next_p = pressure + (k as u16) * p_step;
            if next_p > cap {
                break;
            }
            counts[i] = k;
            walk(
                i + 1,
                next_p,
                glen,
                groups,
                counts,
                cap,
                bounty,
                contract,
                escorts,
                ctx,
                rate,
                best,
            );
        }
        counts[i] = 0;
    }

    walk(
        0,
        0,
        glen,
        &groups,
        &mut counts,
        cap,
        bounty,
        contract,
        &escorts[..elen],
        ctx,
        &rate,
        &mut best,
    );

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
    if p.tool_sets {
        if let Some(gains) = tool_gains(st, options) {
            let mut best = options[0];
            for &id in &options[1..] {
                if gains[id.idx()] > gains[best.idx()] {
                    best = id;
                }
            }
            return best;
        }
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

    if let Some(ch) = st.challenge.as_ref() {
        if ch.window_open {
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

    if st.phase == Phase::Event {
        if let Some((kind, cell_n)) = st.active_event {
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
    if let Some((h_cell, h_val)) = hunt {
        if h_val > 0.0 && h_val > swap_gain {
            return Some(SimAction::Hunt { to: h_cell });
        }
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
                ObjectiveId::Consume => {
                    (st.stats.devours + st.stats.chains) >= (obj.need() as u16)
                }
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
        if (cell_mask & (1u16 << n)) != 0 {
            if let Some(inmate) = st.cells[n as usize].inmate {
                names.push(format!("{} {}", describe_inmate(inmate), cell_label(n)));
            }
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

    if let Some(ch) = st.challenge.as_ref() {
        if ch.window_open {
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

    if st.phase == Phase::Event {
        if let Some((kind, _)) = st.active_event {
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
    if st.can_swap_now() && !here.sealed {
        if let Some(inmate) = here.inmate {
            out.push(Candidate {
                action: SimAction::Swap,
                label: format!("Replace the {} in {}", describe_inmate(inmate), cell_label(here.n)),
                kind: None,
                points: None,
                extra: false,
            });
        }
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

pub fn summarize(rows: &[PlayoutRow], rounds: usize, target: usize, goal: SolveGoal) -> SolveSummary {
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
                        let val =
                            (if passed { PASS_WEIGHT } else { 0.0 }) + (ch.score as f64);
                        (val, usize::from(passed), ch.score as u64, 0)
                    } else {
                        let val = trial.vault_len as f64;
                        let mut max_tier = if trial.vault_len > 0 { RAINBOW } else { 0 };
                        for c in &trial.cells {
                            if !c.sealed {
                                if let Some(inm) = c.inmate {
                                    if inm.tier > max_tier {
                                        max_tier = inm.tier;
                                    }
                                }
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

        if early_stop && rounds_done < max_rollouts && should_stop_rows(&rows, rounds_done, 100, goal)
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

#[derive(Clone, Debug)]
pub struct InstantPick {
    pub action: SimAction,
    pub label: String,
    pub why: String,
}

pub fn signed_fmt(v: f64) -> String {
    let sign = if v >= 0.0 { "+" } else { "\u{2212}" };
    format!("{sign}{:.2}", v.abs())
}

pub fn convoy_why(st: &SimState, cell_mask: u16, escort: EscortKind, p: &SolverParams) -> String {
    let Some(ch) = st.challenge.as_ref() else {
        return String::new();
    };
    let slot = ch.deliveries_len as usize;
    if slot >= 2 {
        return "The heavy contract ends the run, so it takes the most points your vouchers can escort.".to_string();
    }
    let keep = if slot == 0 { p.keep1 } else { p.keep2 };
    let mut kept_cells: Vec<(u8, Inmate)> = st
        .cells
        .iter()
        .filter_map(|c| {
            if !c.sealed && (cell_mask & (1u16 << c.n)) == 0 {
                c.inmate.map(|i| (c.n, i))
            } else {
                None
            }
        })
        .collect();
    kept_cells.sort_by(|a, b| b.1.tier.cmp(&a.1.tier));
    let kept_names: Vec<String> = kept_cells
        .into_iter()
        .take(2)
        .map(|(n, i)| format!("{} {}", describe_inmate(i), cell_label(n)))
        .collect();
    let left = ch.vouchers.saturating_sub(escort.cost());
    let next = if slot == 0 {
        "the next contracts"
    } else {
        "the heavy contract"
    };
    let keep_text = if !kept_names.is_empty() {
        format!(" Your {} stay for {next}.", kept_names.join(" and "))
    } else {
        String::new()
    };
    let voucher_text = if left >= EscortKind::Heavy.cost() {
        format!(
            " It leaves {}, so the heavy contract can still go by heavy escort.",
            plural(left as usize, "voucher")
        )
    } else {
        format!(" It leaves {}.", plural(left as usize, "voucher"))
    };
    let lead = if keep >= 1.0 {
        "Meets the contract with the inmates worth least to you."
    } else if keep > 0.0 {
        "Meets the contract with the most points it can spare."
    } else {
        "Meets the contract for the most points."
    };
    format!("{lead}{keep_text}{voucher_text}")
}

pub fn reason(
    st: &SimState,
    action: &SimAction,
    params: Option<&'static SolverParams>,
) -> String {
    let p = params.unwrap_or_else(|| SolverParams::for_state(st));
    let mut ctx = Context::new(st, p);
    let here = &st.cells[st.position as usize];
    match *action {
        SimAction::Patrol { cell } => {
            let why = patrol_cell_why(st, &mut ctx, cell);
            if let Some(first) = why.chars().next() {
                return format!("{}{}.", first.to_uppercase(), &why[first.len_utf8()..]);
            }
            String::new()
        }
        SimAction::Devour { to } => {
            let odds = chain_odds(st, st.position, to);
            let over = if swap_gain_here(st, &mut ctx) > 0.0 {
                " It beats a Replace here, and a day has one or the other."
            } else {
                ""
            };
            format!(
                "The best first meal: picking its later meals well, its chain averages {:.1} bites, with {:.0}% to reach Rainbow.{over}",
                odds.bites,
                100.0 * odds.rainbow
            )
        }
        SimAction::Chain { to } => {
            let odds = chain_odds(st, st.position, to);
            format!(
                "The best next meal: going on with the best meal each time, the chain averages {:.1} more bites, with {:.0}% to reach Rainbow.",
                odds.bites,
                100.0 * odds.rainbow
            )
        }
        SimAction::StopChain => {
            "No meal left is worth eating: each would cost more than it gains, so press Cancel and keep them on the block.".to_string()
        }
        SimAction::Hunt { to } => {
            let Some(prey) = st.cells[to as usize].inmate else {
                return String::new();
            };
            let (_, s_len) = st.start_options();
            let over = if s_len > 0 || swap_gain_here(st, &mut ctx) > 0.0 {
                " That beats today\u{2019}s consume or Replace, and a day has one of them."
            } else {
                ""
            };
            format!(
                "A Prey mark: this Lion and the {} each rise a tier every morning for {}, as long as both stay on the block and keep their beasts.{over}",
                describe_inmate(prey),
                plural(st.params.prey_rounds as usize, "morning")
            )
        }
        SimAction::Office { choice } => {
            if choice == OfficeChoice::Bestow {
                "A random inmate rises a tier now, which beats a couple of new event doors."
                    .to_string()
            } else {
                "The block is thin, so new event doors are worth more than raising it.".to_string()
            }
        }
        SimAction::Swap => {
            let (_, s_len) = st.start_options();
            let over = if s_len > 0 {
                " That beats today\u{2019}s best consume, and a day has one or the other."
            } else {
                ""
            };
            let tier = here.inmate.map(|i| i.tier).unwrap_or(0);
            let (m_chance, _) = maiden_bonus(st, &ctx, tier);
            let odds = format!("{:.0}% of the time", 100.0 * m_chance);
            if here.ivory {
                let m_str = if m_chance > 0.0 {
                    format!(", and the Severed Iron Maiden adds 3 tiers to the new inmate {odds}")
                } else {
                    String::new()
                };
                return format!("A Tusk cell: replacing its inmate raises other inmates a tier{m_str}.{over}");
            }
            if ctx.scapegoat_loss.is_some() {
                let chrom = if tier >= SUPREME { ", a Chromatic" } else { "" };
                return format!(
                    "The Scapegoat's Guillotine brings the new inmate in above {}{chrom}, worth more than the 2 random inmates it removes.{over}",
                    tier_name(tier)
                );
            }
            if m_chance > 0.0 {
                return format!(
                    "The Severed Iron Maiden adds 3 tiers to the new inmate {odds}, so a fresh roll beats this {} inmate on average.{over}",
                    tier_name(tier)
                );
            }
            if ctx.axe_doors > 0 {
                return format!(
                    "A {} inmate is worth less than a fresh roll on average, and a higher roll makes the Gaoler's Execution Axe open {}.{over}",
                    tier_name(tier),
                    plural(ctx.axe_doors, "door")
                );
            }
            format!(
                "A {} inmate is worth less than a fresh roll on average.{over}",
                tier_name(tier)
            )
        }
        SimAction::End => {
            let (s_opts, s_len) = st.start_options();
            if !st.can_devour_now() || s_len == 0 {
                return "Nothing left today is worth doing.".to_string();
            }
            let marked = here
                .inmate
                .is_some_and(|i| st.link_of(i.id).is_some())
                || s_opts[..s_len].iter().any(|o| {
                    st.cells[o.cell as usize]
                        .inmate
                        .is_some_and(|m| st.link_of(m.id).is_some())
                });
            if marked {
                return "Skip today\u{2019}s consume: it would end a prey mark, and the free tier each morning is worth more than one bite.".to_string();
            }
            "Every consume here would cost more than it gains today.".to_string()
        }
        SimAction::Event { accept } => {
            let Some((kind, cell_n)) = st.active_event else {
                return String::new();
            };
            match kind {
                EventKind::Chain => {
                    if accept {
                        "Your next walk opens every sealed door on the way.".to_string()
                    } else {
                        "More inmates would hurt this block today.".to_string()
                    }
                }
                EventKind::Trickster => {
                    if accept {
                        let who = st.cells[cell_n as usize].inmate.unwrap();
                        format!(
                            "The {} becomes a Supreme, one step from Rainbow.",
                            describe_inmate(who)
                        )
                    } else {
                        "It is already a Supreme; accepting would only change its beast."
                            .to_string()
                    }
                }
                EventKind::Appetite => format!(
                    "The lowest-rarity inmate gets {APPETITE_TRIES} tries at +1 tier, for {:.1} tiers on average.",
                    (APPETITE_TRIES as f64) * APPETITE_ODDS
                ),
                EventKind::Convict => {
                    "Another cell opens on a Legendary inmate, with no patrol needed.".to_string()
                }
                EventKind::Crow => format!(
                    "Free information: the rarity behind the next {CROW_CELLS} locked doors."
                ),
                EventKind::Freedom => {
                    format!("Free inmates: {FREEDOM_CELLS} more locked cells open.")
                }
                EventKind::Usurp1 => {
                    "A random inmate rises a tier, and Usurpation II joins the events.".to_string()
                }
                EventKind::Usurp2 => {
                    "One random inmate rises 2 tiers, and Usurpation III joins the events."
                        .to_string()
                }
                EventKind::Usurp3 => {
                    "Two random inmates rise 2 tiers each, and Usurpation IV joins the events."
                        .to_string()
                }
                EventKind::Usurp4 => "Every inmate on the block rises 2 tiers.".to_string(),
                EventKind::Ratway => {
                    "A free move to any cell but the office: the best door or inmate on the whole block, not just the next six cells.".to_string()
                }
                EventKind::Hunt => {
                    "Free information, and its card has only Confirm: every locked door holding a Normal or Magic shows its rarity, so the doors left dark hold Rare or better.".to_string()
                }
                EventKind::Portion => {
                    "Up to 3 random inmates on the block rise a tier.".to_string()
                }
                EventKind::Fortune => {
                    "This inmate rises a tier for each other inmate of the same beast.".to_string()
                }
                EventKind::Archenemy => {
                    "A locked cell opens with an inmate guaranteed to be a meal for this inmate."
                        .to_string()
                }
                EventKind::Transfer => {
                    "This inmate is replicated into a random locked cell.".to_string()
                }
                EventKind::Execution => {
                    if accept {
                        let exec = st.cells[cell_n as usize].inmate.unwrap();
                        let (_, vlen) = st.execution_victims(cell_n);
                        format!(
                            "The {} removes {} and rises as many tiers: it gains more than it removes.",
                            describe_inmate(exec),
                            plural(vlen, "inmate")
                        )
                    } else {
                        "It would remove more than it gains.".to_string()
                    }
                }
            }
        }
        SimAction::Choose(id) => {
            if st.pending_len == 0 {
                return String::new();
            }
            if st.pending[0].kind == PendingKind::Gift {
                return "Worth the most on this board right now.".to_string();
            }
            let ChoiceId::Tool(t_id) = id else {
                return String::new();
            };
            let mut tool_opts = [ToolId::Revenge; 4];
            let mut tlen = 0;
            for &c in st.pending[0].slice() {
                if let ChoiceId::Tool(t) = c {
                    tool_opts[tlen] = t;
                    tlen += 1;
                }
            }
            if ctx.p.tool_sets {
                if let Some(gains) = tool_gains(st, &tool_opts[..tlen]) {
                    let add = gains[t_id.idx()];
                    if !add.is_nan() {
                        let held_str = if st.tools_len == 1 {
                            format!("With the {}, it", st.tools_order[0].name())
                        } else if st.tools_len > 1 {
                            "With the instruments you hold, it".to_string()
                        } else {
                            "It".to_string()
                        };
                        let vs = if st.can_reroll() {
                            if let Some(odds) = reroll_odds(st, &tool_opts[..tlen]) {
                                format!(
                                    " A re-roll's best would add about {} on average.",
                                    signed_fmt(odds.next)
                                )
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };
                        return format!(
                            "{held_str} adds about {} Rainbows per run in simulated runs.{vs}",
                            signed_fmt(add)
                        );
                    }
                }
            }
            if let Some(rank) = ctx.p.tool_rank.iter().position(|&x| x == t_id) {
                return format!("Ranked #{} of {} tools in training.", rank + 1, ctx.p.tool_rank.len());
            }
            String::new()
        }
        SimAction::Reroll => {
            let mut tool_opts = [ToolId::Revenge; 4];
            let mut tlen = 0;
            if st.pending_len > 0 {
                for &c in st.pending[0].slice() {
                    if let ChoiceId::Tool(t) = c {
                        tool_opts[tlen] = t;
                        tlen += 1;
                    }
                }
            }
            if let Some(odds) = reroll_odds(st, &tool_opts[..tlen]) {
                format!(
                    "The best instrument on offer adds about {} Rainbows per run; a new offer's best adds about {} on average.",
                    signed_fmt(odds.now),
                    signed_fmt(odds.next)
                )
            } else {
                "A new offer is likely to hold a better instrument.".to_string()
            }
        }
        SimAction::Deliver {
            cell_mask,
            escort,
            ..
        } => convoy_why(st, cell_mask, escort, ctx.p),
        SimAction::Skip => "Nothing on the block meets this contract.".to_string(),
        SimAction::Close => "Nothing meets the heavy contract yet.".to_string(),
        SimAction::Early => {
            if ctx.p.early_points < 0.0 {
                "The heavy convoy is as full as your escort allows, so waiting can\u{2019}t add points. Shipping now locks in the pass.".to_string()
            } else if ctx.p.early_points < 1e9 {
                format!(
                    "The heavy convoy scores at least {} points now. Shipping locks in the pass.",
                    ctx.p.early_points
                )
            } else {
                "The heavy contract can be met now. Shipping it ends the run.".to_string()
            }
        }
    }
}

pub fn instant(st: &SimState, params: Option<&'static SolverParams>) -> Option<InstantPick> {
    let p = params.unwrap_or_else(|| SolverParams::for_state(st));
    let mut rng = 1u32;
    let action = decide(st, Some(p), &mut rng)?;
    let cands = candidates(st);
    let label = cands
        .into_iter()
        .find(|c| c.action.same_action(&action))
        .map(|c| c.label)
        .unwrap_or_else(|| action.type_name().to_string());
    let why = reason(st, &action, Some(p));
    Some(InstantPick { action, label, why })
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
    let mut code = String::from("// Auto-generated by `tartarus-data --emit-combos` (pure-Rust Rayon simulation).\n");
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


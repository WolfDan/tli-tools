use crate::engine::{ChoiceId, EscortKind, GameMode, OfficeChoice, SimState, ToolId};
use serde::{Deserialize, Serialize};

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
        } => st.deliver_mask(cell_mask, &vault_ids[..vault_len as usize], escort),
        SimAction::Skip => st.skip_delivery(),
        SimAction::Close => st.cancel_early_delivery(),
        SimAction::Early => st.open_early_delivery(),
    }
}

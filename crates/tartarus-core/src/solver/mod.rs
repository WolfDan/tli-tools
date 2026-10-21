//! Heuristic policy, zero-allocation chain DP (`FeedMemo`), convoy planner, and parallel
//! Monte Carlo playout solver.

pub mod chain_dp;
pub mod convoy;
pub mod explain;
pub mod heuristics;
pub mod params;
pub mod search;
pub mod tools;

pub use chain_dp::{
    ChainOdds, Context, ContractNeed, DietList, FeedMemo, best_meal, bite, chain_odds, composition,
    contract_need, devour_delta, feed, floor_dist, normalize5, rainbow_stuck, rise, swap_dist,
};
pub use convoy::{
    Candidate, ConvoyKind, ConvoyPlan, SolveGoal, cargo_label, convoy_options, convoy_plan,
    most_points, plan_cargo, wants_early,
};
pub use explain::{InstantPick, convoy_why, instant, reason, signed_fmt};
pub use heuristics::{
    DevourScore, OfficeValues, PatrolCellScore, activation, appetite_value, board_value,
    coin_pair_ev, contract_gap, devour_scores, event_why, event_worth_of, event_worth_of_cell,
    execution_value, expected_unknown_event_worth, fresh_activation, fresh_value, hunt_pair_ev,
    link_loss, link_value, maiden_bonus, need_value, office_values, patrol_cell_why,
    patrol_scores_fast, pick_gift, stair, stair_delta, stair_fresh, swap_gain_here, swap_value,
    swap_value_with_comp, tagged, trickster_value,
};
pub use params::{
    CHAIN_ODDS_PROFILE, CHALLENGE_PROFILE, CHALLENGE_TOOL_RANK, DEFAULT_SOLVER_PARAMS,
    DEFAULT_TOOL_RANK, NOTOOLS_PROFILE, PASS_WEIGHT, REWARDS_PROFILE, SimAction, SolverParams,
    apply_action,
};
pub use search::{
    OptionRank, PlayoutRow, SolveSummary, candidates, decide, hide_future, playout, should_stop,
    should_stop_rows, solve_parallel, solve_parallel_with_candidates, summarize,
};
pub use tools::{
    COMBO_1, COMBO_2, COMBO_BASE, RerollOdds, best_hunt, format_tool_combos_rs, pick_tool,
    reroll_odds, simulate_tool_combos, tool_gains, wants_reroll,
};

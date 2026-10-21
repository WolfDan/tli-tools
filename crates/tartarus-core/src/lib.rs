#![forbid(unsafe_code)]

pub mod engine;
pub mod model;
pub mod solver;

pub use engine::{
    ALL_GIFTS, ALL_TOOLS, APPETITE_ODDS, APPETITE_TRIES, BEASTS, Beast, CELL_COUNT, CELL_LABELS,
    CHALLENGE_DAYS, CROW_CELLS, CellState, ChallengeState, ChoiceId, Contract, ContractId,
    DeliveryRecord, DevourOption, EscortKind, EventKind, FREEDOM_CELLS, GameMode, GameParams,
    GiftId, HUNT_BELOW, INITIAL_EVENT_POOL, INSTIGATE_EVENTS, Inmate, OFFICE, ObjectiveId,
    OfficeChoice, PRESSURE, PendingChoice, PendingKind, Phase, PreyLink, RAINBOW, START_CELLS,
    START_VOUCHERS, SUPREME, SimState, SimStats, StartEventId, ToolId, ToolMask, UsedToday,
    VERSION, VaultEntry, WORTH, cell_label, describe_inmate, hash_seed, next_rand, pick_index,
    pick_one, plural, qualifies, refit_seed, roll_event_from_pool, shuffle_slice, steps_between,
    tier_name,
};
pub use model::{
    AdviceOptionRow, AdviceSnapshot, BoardModel, BoardRunStage, BoardStep, CellEvent, CellKind,
    CellModel, ChallengeModel, ChoiceModel, ContractStatus, FixedVec, LIVE_MIN, PreyMark, RunPhase,
    TOOL_SURE, ToolsPrompt, TreasureOrigin, TreasurePhase, format_expect_text, format_option_value,
};
pub use solver::{
    CHAIN_ODDS_PROFILE, CHALLENGE_PROFILE, CHALLENGE_TOOL_RANK, COMBO_1, COMBO_2, COMBO_BASE,
    Candidate, ChainOdds, Context, ContractNeed, ConvoyKind, ConvoyPlan, DEFAULT_SOLVER_PARAMS,
    DEFAULT_TOOL_RANK, DevourScore, DietList, FeedMemo, InstantPick, NOTOOLS_PROFILE, OfficeValues,
    OptionRank, PASS_WEIGHT, PatrolCellScore, PlayoutRow, REWARDS_PROFILE, RerollOdds, SimAction,
    SolveGoal, SolveSummary, SolverParams, activation, appetite_value, apply_action, best_hunt,
    best_meal, bite, board_value, candidates, cargo_label, chain_odds, coin_pair_ev, composition,
    contract_gap, contract_need, convoy_options, convoy_plan, convoy_why, decide, devour_delta,
    devour_scores, event_why, event_worth_of, event_worth_of_cell, execution_value,
    expected_unknown_event_worth, feed, floor_dist, format_tool_combos_rs, fresh_activation,
    fresh_value, hide_future, hunt_pair_ev, instant, link_loss, link_value, maiden_bonus,
    most_points, need_value, normalize5, office_values, patrol_cell_why, patrol_scores_fast,
    pick_gift, pick_tool, plan_cargo, playout, rainbow_stuck, reason, reroll_odds, rise,
    should_stop, should_stop_rows, signed_fmt, simulate_tool_combos, solve_parallel,
    solve_parallel_with_candidates, stair, stair_delta, stair_fresh, summarize, swap_dist,
    swap_gain_here, swap_value, swap_value_with_comp, tagged, tool_gains, trickster_value,
    wants_early, wants_reroll,
};

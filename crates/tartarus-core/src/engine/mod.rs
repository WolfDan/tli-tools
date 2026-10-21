//! Core game simulation types, RNG helpers, state transitions, and combat/event resolution.

pub mod combat;
pub mod rng;
pub mod rules;
pub mod state;
pub mod types;

pub use combat::qualifies;
pub use rng::{
    hash_seed, next_rand, pick_index, pick_one, refit_seed, roll_event_from_pool, shuffle_slice,
};
pub use state::SimState;
pub use types::{
    ALL_GIFTS, ALL_TOOLS, APPETITE_ODDS, APPETITE_TRIES, BEASTS, Beast, CELL_COUNT, CELL_LABELS,
    CHALLENGE_DAYS, CROW_CELLS, CellState, ChallengeState, ChoiceId, Contract, ContractId,
    DeliveryRecord, DevourOption, EscortKind, EventKind, FREEDOM_CELLS, GameMode, GameParams,
    GiftId, HUNT_BELOW, INITIAL_EVENT_POOL, INSTIGATE_EVENTS, Inmate, OFFICE, ObjectiveId,
    OfficeChoice, PRESSURE, PendingChoice, PendingKind, Phase, PreyLink, RAINBOW, START_CELLS,
    START_VOUCHERS, SUPREME, SimStats, StartEventId, ToolId, ToolMask, UsedToday, VERSION,
    VaultEntry, WORTH, cell_label, describe_inmate, plural, steps_between, tier_name,
};

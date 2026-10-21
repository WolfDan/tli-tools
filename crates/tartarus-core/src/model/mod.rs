//! High-level `BoardModel` state, `SimState` conversion, and `AdviceSnapshot` evaluation.

pub mod advice;
pub mod board;
pub mod builder;

pub use advice::{AdviceOptionRow, AdviceSnapshot, format_expect_text, format_option_value};
pub use board::{
    BoardModel, BoardRunStage, BoardStep, CellEvent, CellKind, CellModel, ChallengeModel,
    ChoiceModel, ContractStatus, FixedVec, LIVE_MIN, PreyMark, RunPhase, TOOL_SURE, ToolsPrompt,
    TreasureOrigin, TreasurePhase,
};

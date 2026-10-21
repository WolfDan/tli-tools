use crate::engine::{
    Beast, CELL_COUNT, ContractId, EventKind, GameMode, OFFICE, ObjectiveId, PendingKind,
    START_VOUCHERS, StartEventId, ToolId, tier_name,
};
use serde::{Deserialize, Serialize};

pub const LIVE_MIN: usize = 150;
pub const TOOL_SURE: f64 = 0.9;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BoardStep {
    Patrol,
    Manage,
    Office,
    Chain,
    Event,
    Ratway,
    Choice,
    Window,
}

impl BoardStep {
    pub const fn as_str(self) -> &'static str {
        match self {
            BoardStep::Patrol => "patrol",
            BoardStep::Manage => "manage",
            BoardStep::Office => "office",
            BoardStep::Chain => "chain",
            BoardStep::Event => "event",
            BoardStep::Ratway => "ratway",
            BoardStep::Choice => "choice",
            BoardStep::Window => "window",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            BoardStep::Patrol => "Select a Destination",
            BoardStep::Manage => "Make Your Decision",
            BoardStep::Office => "Instigate or Bestow",
            BoardStep::Chain => "Pick a meal",
            BoardStep::Event => "Special event",
            BoardStep::Ratway => "Sewer Ratway: any cell",
            BoardStep::Choice => "Instrument or gift",
            BoardStep::Window => "Delivery",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CellKind {
    Office,
    Sealed,
    Empty,
    Inmate,
    Scratch,
    Revealed,
    Scouted,
    Nest,
}

impl CellKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            CellKind::Office => "office",
            CellKind::Sealed => "sealed",
            CellKind::Empty => "empty",
            CellKind::Inmate => "inmate",
            CellKind::Scratch => "scratch",
            CellKind::Revealed => "revealed",
            CellKind::Scouted => "scouted",
            CellKind::Nest => "nest",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum CellEvent {
    #[default]
    None,
    Unknown,
    Known(EventKind),
}

impl CellEvent {
    pub const fn is_some(self) -> bool {
        !matches!(self, CellEvent::None)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PreyMark {
    pub cell: u8,
    pub rounds: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CellModel {
    pub kind: CellKind,
    pub beast: Beast,
    pub tier: u8,
    pub ivory: bool,
    pub prey: Option<PreyMark>,
    pub event: CellEvent,
    pub due: u8,
    pub at_least: u8,
}

impl CellModel {
    pub const fn office() -> Self {
        Self {
            kind: CellKind::Office,
            beast: Beast::Mammoth,
            tier: 0,
            ivory: false,
            prey: None,
            event: CellEvent::None,
            due: 0,
            at_least: 0,
        }
    }

    pub const fn sealed() -> Self {
        Self {
            kind: CellKind::Sealed,
            beast: Beast::Mammoth,
            tier: 0,
            ivory: false,
            prey: None,
            event: CellEvent::None,
            due: 0,
            at_least: 0,
        }
    }

    pub fn short_label(&self) -> String {
        match self.kind {
            CellKind::Office => "Office".to_string(),
            CellKind::Empty => "Empty".to_string(),
            CellKind::Nest => format!("Nest d{}", self.due),
            CellKind::Inmate => {
                let t = &tier_name(self.tier)[..3];
                let iv = if self.ivory { " \u{2666}" } else { "" };
                let ev = if self.event.is_some() { " !" } else { "" };
                format!("{t} {}{iv}{ev}", self.beast.name())
            }
            CellKind::Scratch => format!("{} scratch", &tier_name(self.tier)[..3]),
            CellKind::Revealed => format!("{} ?", &tier_name(self.tier)[..3]),
            CellKind::Scouted => {
                format!("{} {} ?", &tier_name(self.tier)[..3], self.beast.name())
            }
            CellKind::Sealed => {
                if self.at_least >= 2 {
                    "Locked Rare+".to_string()
                } else if self.event.is_some() {
                    "Locked !".to_string()
                } else {
                    "Locked".to_string()
                }
            }
        }
    }
}

pub type FixedVec<T, const N: usize> = tinyvec::ArrayVec<[T; N]>;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum ToolsPrompt {
    #[default]
    None,
    CountMismatch {
        panel_slots: u8,
    },
    Unexpected {
        off: FixedVec<ToolId, 8>,
    },
    UnknownRun {
        panel_slots: u8,
    },
}

impl ToolsPrompt {
    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::None;
    }

    pub fn format_for(&self, tools: &[ToolId]) -> String {
        match *self {
            Self::None => String::new(),
            Self::CountMismatch { panel_slots } => {
                let held_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
                format!(
                    "Your panel shows {} instrument{}, and the analyzer has {}: {}. Confirm your instruments, then press These are right.",
                    panel_slots,
                    if panel_slots == 1 { "" } else { "s" },
                    tools.len(),
                    if held_names.is_empty() {
                        "none".to_string()
                    } else {
                        held_names.join(", ")
                    }
                )
            }
            Self::Unexpected { ref off } => {
                let off_names: Vec<&str> = off.iter().map(|t| t.name()).collect();
                let held_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
                format!(
                    "An icon on your panel looks like {}, which isn't one the analyzer has for this run ({}). Check your instruments, then press These are right.",
                    off_names.join(" and "),
                    held_names.join(", ")
                )
            }
            Self::UnknownRun { panel_slots } => {
                let sure_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
                format!(
                    "The analyzer doesn't know this run's instruments yet. It read {} from your panel of {}. Confirm your instruments, then press These are right.",
                    if sure_names.is_empty() {
                        "none for sure".to_string()
                    } else {
                        sure_names.join(", ")
                    },
                    panel_slots
                )
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum RunPhase {
    /// Fresh launch, waiting for first prison board or instrument offer.
    #[default]
    WaitingForRun,
    /// Previous trial ended (`End of the trial`); waiting for next run.
    RunOver,
    /// Active run in progress with a valid `BoardModel` and `AdviceSnapshot`.
    Active { last_capture_matched: bool },
}

impl RunPhase {
    #[inline]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active { .. })
    }

    #[inline]
    pub const fn is_run_over(self) -> bool {
        matches!(self, Self::RunOver)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ContractStatus {
    Pending,
    Delivered,
    Skipped,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ChallengeModel {
    pub vouchers: u8,
    pub contracts: [ContractId; 3],
    pub bounty: Beast,
    pub status: [ContractStatus; 2],
    pub points: [u16; 2],
}

impl Default for ChallengeModel {
    fn default() -> Self {
        Self {
            vouchers: START_VOUCHERS,
            contracts: [ContractId::Mixed, ContractId::Elite, ContractId::Heavy],
            bounty: Beast::Wolf,
            status: [ContractStatus::Pending, ContractStatus::Pending],
            points: [0, 0],
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum TreasurePhase {
    #[default]
    Closed,
    WaitingForBoardView,
    ReadyToChoose,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum TreasureOrigin {
    View,
    Swap,
    Consume,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum BoardRunStage {
    #[default]
    InProgress,
    Opening,
    Ended,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ChoiceModel {
    pub kind: PendingKind,
    pub options: [ToolId; 4],
    pub options_len: u8,
    pub rerolls: u8,
}

impl Default for ChoiceModel {
    fn default() -> Self {
        Self {
            kind: PendingKind::Tool,
            options: [ToolId::Revenge; 4],
            options_len: 0,
            rerolls: 0,
        }
    }
}

impl ChoiceModel {
    #[inline]
    pub fn from_tools(kind: PendingKind, tools: &[ToolId], rerolls: u8) -> Self {
        let mut options = [ToolId::Revenge; 4];
        let len = tools.len().min(4);
        options[..len].copy_from_slice(&tools[..len]);
        Self {
            kind,
            options,
            options_len: len as u8,
            rerolls,
        }
    }

    #[inline]
    pub fn slice(&self) -> &[ToolId] {
        &self.options[..(self.options_len as usize).min(4)]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BoardModel {
    pub mode: GameMode,
    pub rewards: bool,
    pub day: u8,
    pub max_days: u8,
    pub step: BoardStep,
    pub warden: u8,
    /// Bitmask of cells (0..16) marked as today's patrol doors.
    pub dest_mask: u16,
    pub devour_used: bool,
    pub swap_used: bool,
    pub hunt_used: bool,
    pub consume_offered: bool,
    pub treasure: TreasurePhase,
    pub tools_prompt: ToolsPrompt,
    pub extra_slots: usize,
    pub tools_slots: usize,
    /// Quantized tool-icon scale step (`(scale * 20.0).round() as u16`, `0` = unset).
    pub tools_scale_step: u16,
    pub no_swap: bool,
    pub no_devour: bool,
    pub event_kind: EventKind,
    pub choice: ChoiceModel,
    pub window_early: bool,
    pub chain_armed: bool,
    pub objective: ObjectiveId,
    pub start_event: StartEventId,
    pub walk_tiles: u8,
    pub milestone: bool,
    pub stage: BoardRunStage,
    pub tools: FixedVec<ToolId, 32>,
    pub vault: FixedVec<Beast, 16>,
    pub cells: [CellModel; CELL_COUNT],
    pub ch: ChallengeModel,
    /// Bitmask of cells (0..16) flagged as `Check` by the screenshot scanner.
    pub checks: u16,
}

impl Default for BoardModel {
    fn default() -> Self {
        Self::blank()
    }
}

impl BoardModel {
    pub fn blank() -> Self {
        let mut cells = [CellModel::sealed(); CELL_COUNT];
        cells[OFFICE as usize] = CellModel::office();
        Self {
            mode: GameMode::Drill,
            rewards: true,
            day: 1,
            max_days: 20,
            step: BoardStep::Patrol,
            warden: OFFICE,
            dest_mask: 0,
            devour_used: false,
            swap_used: false,
            hunt_used: false,
            consume_offered: false,
            treasure: TreasurePhase::Closed,
            tools_prompt: ToolsPrompt::None,
            extra_slots: 0,
            tools_slots: 0,
            tools_scale_step: 0,
            no_swap: false,
            no_devour: false,
            event_kind: EventKind::Chain,
            choice: ChoiceModel::default(),
            window_early: false,
            chain_armed: false,
            objective: ObjectiveId::Other,
            start_event: StartEventId::None,
            walk_tiles: 0,
            milestone: false,
            stage: BoardRunStage::InProgress,
            tools: FixedVec::new(),
            vault: FixedVec::new(),
            cells,
            ch: ChallengeModel::default(),
            checks: 0,
        }
    }

    pub fn example() -> Self {
        let mut m = Self::blank();
        m.day = 6;
        m.warden = 4;
        m.step = BoardStep::Patrol;
        m.dest_mask = (1u16 << 5) | (1u16 << 6) | (1u16 << 8);
        m.milestone = false;

        let put = |cells: &mut [CellModel; 16], n: usize, beast: Beast, tier: u8| {
            cells[n] = CellModel {
                kind: CellKind::Inmate,
                beast,
                tier,
                ivory: false,
                prey: None,
                event: CellEvent::None,
                due: 0,
                at_least: 0,
            };
        };
        put(&mut m.cells, 1, Beast::Wolf, 1);
        put(&mut m.cells, 3, Beast::Lion, 2);
        put(&mut m.cells, 4, Beast::Rat, 0);
        put(&mut m.cells, 6, Beast::Mammoth, 2);
        put(&mut m.cells, 9, Beast::Rat, 1);
        m.cells[10].kind = CellKind::Empty;
        m.cells[11] = CellModel {
            kind: CellKind::Scratch,
            tier: 2,
            ..CellModel::sealed()
        };
        put(&mut m.cells, 12, Beast::Wolf, 3);
        m.cells[14] = CellModel {
            kind: CellKind::Nest,
            due: 7,
            ..CellModel::sealed()
        };
        put(&mut m.cells, 15, Beast::Wolf, 2);
        m.cells[5] = CellModel {
            kind: CellKind::Sealed,
            event: CellEvent::Unknown,
            ..CellModel::sealed()
        };
        m.tools = FixedVec::from_iter([ToolId::Rope]);
        m
    }

    #[inline]
    pub fn has_dest(&self, cell: u8) -> bool {
        cell < 16 && (self.dest_mask & (1u16 << cell)) != 0
    }

    #[inline]
    pub fn set_dest(&mut self, cell: u8, on: bool) {
        if cell < 16 {
            if on {
                self.dest_mask |= 1u16 << cell;
            } else {
                self.dest_mask &= !(1u16 << cell);
            }
        }
    }

    #[inline]
    pub fn toggle_dest(&mut self, cell: u8) {
        if cell < 16 {
            self.dest_mask ^= 1u16 << cell;
        }
    }

    #[inline]
    pub fn clear_dest(&mut self) {
        self.dest_mask = 0;
    }

    #[inline]
    pub fn is_checked(&self, cell: u8) -> bool {
        (self.checks & (1u16 << cell)) != 0
    }

    #[inline]
    pub fn set_checked(&mut self, cell: u8, on: bool) {
        if on {
            self.checks |= 1u16 << cell;
        } else {
            self.checks &= !(1u16 << cell);
        }
    }

    #[inline]
    pub fn checked_count(&self) -> usize {
        self.checks.count_ones() as usize
    }

    pub fn clean_prey(&mut self) {
        let mut copy = *self;
        copy.step = BoardStep::Manage;
        let (st, _) = copy.build_state(1);
        for n in 0..CELL_COUNT {
            if let Some(prey) = self.cells[n].prey {
                let lion_cell = &st.cells[n];
                let target_cell = st.cells.get(prey.cell as usize);
                let ok = self.cells[n].kind == CellKind::Inmate
                    && self.cells[n].beast == Beast::Lion
                    && lion_cell.inmate.is_some()
                    && target_cell.is_some_and(|t| {
                        t.inmate
                            .is_some_and(|ti| st.can_eat(lion_cell.inmate.unwrap(), ti))
                    });
                if !ok {
                    self.cells[n].prey = None;
                }
            }
        }
    }
}

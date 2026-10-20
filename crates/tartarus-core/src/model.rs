use crate::engine::*;
use crate::solver::*;
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

    pub fn build_state(&self, seed: u32) -> (SimState, Vec<String>) {
        let mut problems = Vec::new();
        let challenge = self.mode == GameMode::Challenge;
        let mut st = SimState::new_blank(self.mode, self.rewards, self.max_days);
        let mut rng = if seed == 0 { 0x1234_5678 } else { seed };
        st.rng = (next_rand(&mut rng) * 4294967296.0) as u32;
        st.day = self.day.clamp(1, st.max_days);
        st.next_id = 1;

        for n in 0..CELL_COUNT {
            let c = &self.cells[n];
            let mut cell = CellState {
                n: n as u8,
                sealed: n != (OFFICE as usize),
                inmate: None,
                hidden: None,
                known: 0,
                at_least: 0,
                scratch: false,
                nest_due: None,
                event: None,
                event_known: true,
                ivory: false,
                seen: false,
            };
            if n == (OFFICE as usize) {
                st.cells[n] = cell;
                continue;
            }
            let apply_ev = |cell: &mut CellState, ev: CellEvent| {
                cell.seen = ev.is_some();
                match ev {
                    CellEvent::None => {}
                    CellEvent::Unknown => {
                        cell.event = Some(EventKind::Chain);
                        cell.event_known = false;
                    }
                    CellEvent::Known(k) => {
                        cell.event = Some(k);
                        cell.event_known = true;
                    }
                }
            };

            match c.kind {
                CellKind::Office => {
                    cell.sealed = false;
                }
                CellKind::Empty => {
                    cell.sealed = false;
                }
                CellKind::Inmate => {
                    cell.sealed = false;
                    let id = st.next_id;
                    st.next_id += 1;
                    cell.inmate = Some(Inmate {
                        id,
                        beast: c.beast,
                        tier: c.tier.min(SUPREME),
                    });
                    cell.ivory = c.ivory;
                    apply_ev(&mut cell, c.event);
                }
                CellKind::Nest => {
                    cell.nest_due = Some(c.due.max(st.day + 1));
                }
                CellKind::Scouted => {
                    cell.known = 2;
                    let id = st.next_id;
                    st.next_id += 1;
                    cell.hidden = Some(Inmate {
                        id,
                        beast: c.beast,
                        tier: c.tier.min(SUPREME),
                    });
                    apply_ev(&mut cell, c.event);
                }
                CellKind::Scratch | CellKind::Revealed => {
                    cell.scratch = c.kind == CellKind::Scratch;
                    cell.known = 1;
                    let id = st.next_id;
                    st.next_id += 1;
                    let b_idx = (next_rand(&mut rng) * 4.0).floor() as usize;
                    cell.hidden = Some(Inmate {
                        id,
                        beast: BEASTS[b_idx.min(3)],
                        tier: c.tier.min(SUPREME),
                    });
                    apply_ev(&mut cell, c.event);
                }
                CellKind::Sealed => {
                    cell.at_least = c.at_least.min(SUPREME);
                    apply_ev(&mut cell, c.event);
                }
            }
            st.cells[n] = cell;
        }

        // Prey marks
        st.prey_len = 0;
        for (n, c) in self.cells.iter().enumerate() {
            if c.kind != CellKind::Inmate || c.beast != Beast::Lion {
                continue;
            }
            let Some(prey) = c.prey else {
                continue;
            };
            let Some(lion) = st.cells[n].inmate else {
                continue;
            };
            let target_inmate = st.cells.get(prey.cell as usize).and_then(|tc| tc.inmate);
            let Some(target) = target_inmate else {
                problems.push(format!(
                    "The prey mark from cell {} needs a same-rarity meal for the Lion in cell {}.",
                    cell_label(n as u8),
                    cell_label(prey.cell)
                ));
                continue;
            };
            if !st.can_eat(lion, target) {
                problems.push(format!(
                    "The prey mark from cell {} needs a same-rarity meal for the Lion in cell {}.",
                    cell_label(n as u8),
                    cell_label(prey.cell)
                ));
                continue;
            }
            let already = st.prey_slice().iter().any(|l| {
                l.lion == target.id || l.prey == target.id || l.lion == lion.id || l.prey == lion.id
            });
            if already {
                problems.push(format!(
                    "Cell {} already has a prey mark.",
                    cell_label(prey.cell)
                ));
                continue;
            }
            if (st.prey_len as usize) < st.prey.len() {
                st.prey[st.prey_len as usize] = PreyLink {
                    lion: lion.id,
                    prey: target.id,
                    prey_beast: target.beast,
                    rounds: prey.rounds.clamp(1, 9),
                    next_day: st.day + 1,
                };
                st.prey_len += 1;
            }
        }

        // Vault
        st.vault_len = 0;
        for &beast in &self.vault {
            if (st.vault_len as usize) < st.vault.len() {
                let id = st.next_id;
                st.next_id += 1;
                st.vault[st.vault_len as usize] = VaultEntry { id, beast, day: 1 };
                st.vault_len += 1;
            }
        }
        st.cache = st.vault_len;

        // Tools
        st.tools_mask = ToolMask(0);
        st.tools_len = 0;
        for &t in &self.tools {
            st.add_tool(t);
        }

        st.position = self.warden.min(15);
        st.chain_armed = self.chain_armed;
        let need = self.objective.need();
        st.objective_id = self.objective;
        st.objective_progress = if need > 0 {
            self.walk_tiles.min(need)
        } else {
            0
        };
        st.start_event = self.start_event;
        st.milestone = self.milestone || (need > 0 && st.objective_progress >= need);

        let mut max_tier = if st.vault_len > 0 { RAINBOW } else { 0 };
        for c in &st.cells {
            if let Some(inm) = c.inmate
                && inm.tier > max_tier
            {
                max_tier = inm.tier;
            }
        }
        st.stats.best = max_tier;
        st.used = UsedToday::default();
        st.dest_len = 0;
        st.active_event = None;
        st.pending_len = 0;
        st.phase = Phase::Manage;

        if challenge && let Some(ch) = st.challenge.as_mut() {
            ch.vouchers = self.ch.vouchers.min(START_VOUCHERS);
            let deadlines = [4u8, 8u8, 12u8];
            for slot in 0..3 {
                let cid = self.ch.contracts[slot];
                ch.contracts[slot] = Contract {
                    slot: slot as u8,
                    id: cid,
                    deadline: deadlines[slot],
                    heavy: slot == 2,
                    bounty: if cid == ContractId::Bounty {
                        Some(self.ch.bounty)
                    } else {
                        None
                    },
                };
            }
            ch.deliveries_len = 0;
            ch.score = 0;
            for slot in 0..2 {
                let st_s = self.ch.status[slot];
                if st_s == ContractStatus::Pending {
                    break;
                }
                let success = st_s == ContractStatus::Delivered;
                let pts = if success {
                    self.ch.points[slot].min(999)
                } else {
                    0
                };
                ch.deliveries[ch.deliveries_len as usize] = DeliveryRecord {
                    slot: slot as u8,
                    day: deadlines[slot],
                    success,
                    points: pts,
                    cost: 0,
                    pressure: 0,
                    escort: if success {
                        Some(EscortKind::Light)
                    } else {
                        None
                    },
                    early: false,
                };
                ch.deliveries_len += 1;
                ch.score += pts;
            }
            if self.ch.status[0] == ContractStatus::Pending
                && self.ch.status[1] != ContractStatus::Pending
            {
                problems.push("Contract 2 can only be settled after contract 1.".to_string());
            }
            if (ch.deliveries_len as usize) < 2 {
                let next = ch.contracts[ch.deliveries_len as usize];
                if st.day > next.deadline {
                    problems.push(format!(
                        "\"{}\" was due at the end of day {}. Mark it delivered or skipped.",
                        next.id.name(),
                        next.deadline
                    ));
                }
            }
            ch.window_open = false;
            ch.window_early = false;
        }

        let set_used = |st: &mut SimState| {
            st.used = UsedToday {
                swap: u8::from(self.swap_used),
                devour: u8::from(self.devour_used),
                hunt: self.hunt_used,
                devour_after_swap: self.consume_offered,
                no_swap: self.no_swap,
                no_devour: self.no_devour,
            };
        };

        match self.step {
            BoardStep::Patrol => {
                st.phase = Phase::Patrol;
                let mut dlen = 0usize;
                for step in 1..16u8 {
                    let n = (st.position + step) & 15;
                    if self.has_dest(n) {
                        st.destinations[dlen] = n;
                        dlen += 1;
                    }
                }
                st.dest_len = dlen as u8;
                if st.dest_len == 0 {
                    problems.push("Mark today's doors: tick them under \u{201c}Which door to patrol\u{201d}, or select a cell and press Today's door.".to_string());
                }
                let range = st.params.patrol_range;
                let reach = |w: u8, n: u8| {
                    let s = steps_between(w, n);
                    s >= 1 && s <= range
                };
                let mut outside: FixedVec<&'static str, 16> = FixedVec::new();
                for n in 0..16u8 {
                    if self.has_dest(n) && !reach(st.position, n) {
                        outside.push(cell_label(n));
                    }
                }
                const TUTORIAL_MASK: u16 = (1u16 << 5) | (1u16 << 7) | (1u16 << 13);
                let tutorial = st.position == 4 && self.dest_mask == TUTORIAL_MASK;
                if !outside.is_empty() && !tutorial {
                    let mut fits: FixedVec<&'static str, 16> = FixedVec::new();
                    for w in 0..16u8 {
                        if (0..16u8).all(|n| !self.has_dest(n) || reach(w, n)) {
                            fits.push(cell_label(w));
                        }
                    }
                    let where_str = if fits.len() > 1 {
                        format!(
                            "{} or {}",
                            fits[..fits.len() - 1].join(", "),
                            fits[fits.len() - 1]
                        )
                    } else if fits.len() == 1 {
                        fits[0].to_string()
                    } else {
                        String::new()
                    };
                    let out_str = if outside.len() == 1 {
                        format!("cell {} isn't", outside[0])
                    } else {
                        format!("cells {} aren't", outside.join(" and "))
                    };
                    let suffix = if !fits.is_empty() {
                        format!(
                            "Check the warden's cell: with these doors it's in cell {where_str}."
                        )
                    } else {
                        "No single cell has all these doors in reach, so check the doors too."
                            .to_string()
                    };
                    problems.push(format!(
                        "Doors are always among the {range} cells clockwise ahead of the warden, and {out_str}: the warden is set in cell {}. {suffix}",
                        cell_label(st.position)
                    ));
                }
            }
            BoardStep::Ratway => {
                st.phase = Phase::Patrol;
                st.ratway = true;
                for i in 1..16u8 {
                    st.destinations[(i - 1) as usize] = i;
                }
                st.dest_len = 15;
            }
            BoardStep::Event => {
                let here = &st.cells[st.position as usize];
                if here.sealed || here.inmate.is_none() {
                    problems.push("An event needs the inmate that came out of the door in the warden's cell: put the warden there and set its inmate.".to_string());
                }
                st.phase = Phase::Event;
                st.active_event = Some((self.event_kind, st.position));
            }
            BoardStep::Choice => {
                set_used(&mut st);
                let kind = self.choice.kind;
                let mut opts = [ChoiceId::Tool(ToolId::Revenge); 4];
                let mut olen = 0usize;
                if kind == PendingKind::Gift {
                    opts = [
                        ChoiceId::Gift(GiftId::Ring),
                        ChoiceId::Gift(GiftId::Horn),
                        ChoiceId::Gift(GiftId::Wine),
                        ChoiceId::Gift(GiftId::Brand),
                    ];
                    olen = 4;
                } else {
                    for &t in self.choice.slice() {
                        if !st.has_tool(t) && !opts[..olen].contains(&ChoiceId::Tool(t)) && olen < 4
                        {
                            opts[olen] = ChoiceId::Tool(t);
                            olen += 1;
                        }
                    }
                }
                if olen == 0 {
                    problems.push("Pick the tools on offer.".to_string());
                }
                if kind == PendingKind::Refit && !challenge {
                    problems.push("Refits only follow contract deliveries. Pick the contract challenge, or another kind of offer.".to_string());
                }
                let slot = if challenge {
                    st.challenge
                        .as_ref()
                        .map(|c| c.deliveries_len.saturating_sub(1))
                        .unwrap_or(0)
                } else {
                    0
                };
                st.pending[0] = PendingChoice {
                    kind,
                    options: opts,
                    options_len: olen as u8,
                    slot,
                    refreshed: false,
                    rerolls: if kind == PendingKind::Tool {
                        self.choice.rerolls
                    } else {
                        0
                    },
                };
                st.pending_len = 1;
            }
            BoardStep::Window => {
                if !challenge {
                    problems.push("Deliveries are part of the contract challenge. Switch the goal to Contract challenge.".to_string());
                } else if let Some(ch) = st.challenge.as_mut() {
                    let early = self.window_early;
                    if (ch.deliveries_len as usize) >= 3 {
                        problems.push("Every contract is already settled.".to_string());
                    } else {
                        let next = ch.contracts[ch.deliveries_len as usize];
                        if early && !next.heavy {
                            problems.push("The early window is only for the heavy contract, once the first two are settled.".to_string());
                        } else if early && st.day < 9 {
                            problems.push(
                                "The heavy contract can be delivered early from day 9.".to_string(),
                            );
                        }
                    }
                    ch.window_open = true;
                    ch.window_early = early;
                }
            }
            BoardStep::Chain => {
                set_used(&mut st);
                st.used.devour = 1;
                let here = &st.cells[st.position as usize];
                if let Some(inmate) = here.inmate.filter(|_| !here.sealed) {
                    st.chaining_eater = Some(inmate.id);
                    let (_, clen) = st.chain_options();
                    if clen == 0 {
                        problems.push("Your game asks for a meal only when the chaining inmate has one to eat. Check the board: it can't find a meal for it.".to_string());
                    }
                } else {
                    problems.push("A chain needs its eater in the warden's cell: put the warden there and set the inmate as it is now.".to_string());
                }
            }
            BoardStep::Office => {
                set_used(&mut st);
                if st.position != OFFICE {
                    problems.push("Instigate and Bestow come with the warden in the office: put the warden in cell 01.".to_string());
                }
                st.phase = Phase::Office;
            }
            BoardStep::Manage => {
                set_used(&mut st);
            }
        }

        (st, problems)
    }

    pub fn from_state(st: &SimState, prev: Option<&BoardModel>) -> Self {
        let mut m = Self::blank();
        m.mode = st.mode;
        m.rewards = st.params.rewards;
        m.day = st.day;
        m.max_days = if st.mode == GameMode::Challenge {
            prev.map(|p| p.max_days).unwrap_or(20)
        } else {
            st.max_days
        };
        m.warden = st.position;
        m.chain_armed = st.chain_armed;
        m.objective = st.objective_id;
        m.start_event = st.start_event;
        m.walk_tiles = st.objective_progress;
        m.milestone = st.milestone;
        m.tools = st.tools_slice().iter().copied().take(32).collect();
        m.vault = st.vault_slice().iter().map(|v| v.beast).take(16).collect();

        for n in 0..CELL_COUNT {
            if n == (OFFICE as usize) {
                m.cells[n] = CellModel::office();
                continue;
            }
            let c = &st.cells[n];
            let before = prev.map(|p| &p.cells[n]);
            let event = if c.event.is_some() && c.seen {
                if !c.event_known {
                    CellEvent::Unknown
                } else if let Some(b) = before {
                    if b.event.is_some() {
                        b.event
                    } else {
                        CellEvent::Unknown
                    }
                } else {
                    CellEvent::Unknown
                }
            } else {
                CellEvent::None
            };

            if c.sealed {
                if let Some(due) = c.nest_due {
                    m.cells[n] = CellModel {
                        kind: CellKind::Nest,
                        due,
                        ..CellModel::sealed()
                    };
                } else if let Some(h) = c.hidden
                    && c.scratch
                    && c.known > 0
                {
                    m.cells[n] = CellModel {
                        kind: CellKind::Scratch,
                        tier: h.tier.min(SUPREME),
                        event,
                        ..CellModel::sealed()
                    };
                } else if let Some(h) = c.hidden
                    && c.known == 1
                {
                    m.cells[n] = CellModel {
                        kind: CellKind::Revealed,
                        tier: h.tier.min(SUPREME),
                        event,
                        ..CellModel::sealed()
                    };
                } else if let Some(h) = c.hidden
                    && c.known >= 2
                {
                    m.cells[n] = CellModel {
                        kind: CellKind::Scouted,
                        beast: h.beast,
                        tier: h.tier.min(SUPREME),
                        event,
                        ..CellModel::sealed()
                    };
                } else {
                    m.cells[n] = CellModel {
                        kind: CellKind::Sealed,
                        event,
                        at_least: c.at_least,
                        ..CellModel::sealed()
                    };
                }
            } else if let Some(inm) = c.inmate {
                let link = st.link_of(inm.id);
                let prey = link.and_then(|l| {
                    if l.lion == inm.id {
                        st.cell_of(l.prey).map(|pc| PreyMark {
                            cell: pc,
                            rounds: l.rounds,
                        })
                    } else {
                        None
                    }
                });
                m.cells[n] = CellModel {
                    kind: CellKind::Inmate,
                    beast: inm.beast,
                    tier: inm.tier.min(SUPREME),
                    ivory: c.ivory,
                    prey,
                    event,
                    due: 0,
                    at_least: 0,
                };
            } else {
                m.cells[n] = CellModel {
                    kind: CellKind::Empty,
                    ..CellModel::sealed()
                };
            }
        }

        if st.pending_len > 0 {
            m.step = BoardStep::Choice;
            let p0 = &st.pending[0];
            let mut opts = [ToolId::Revenge; 4];
            let mut olen = 0usize;
            for &c in p0.slice() {
                if let ChoiceId::Tool(t) = c
                    && olen < 4
                {
                    opts[olen] = t;
                    olen += 1;
                }
            }
            m.choice = ChoiceModel::from_tools(p0.kind, &opts[..olen], p0.rerolls);
        } else if let Some(ch) = st.challenge.as_ref().filter(|c| c.window_open) {
            m.step = BoardStep::Window;
            m.window_early = ch.window_early;
        } else if let (Phase::Event, Some((kind, _))) = (st.phase, st.active_event) {
            m.step = BoardStep::Event;
            m.event_kind = kind;
        } else if st.chaining_eater.is_some() {
            m.step = BoardStep::Chain;
        } else if st.phase == Phase::Patrol {
            m.step = if st.ratway {
                BoardStep::Ratway
            } else {
                BoardStep::Patrol
            };
        } else {
            m.step = BoardStep::Manage;
        }

        m.dest_mask = 0;
        if st.phase == Phase::Patrol && !st.ratway {
            for &d in st.destinations_slice() {
                m.set_dest(d, true);
            }
        }
        m.devour_used = st.used.devour > 0;
        m.swap_used = st.used.swap > 0;
        m.hunt_used = st.used.hunt;
        m.consume_offered = st.used.devour_after_swap;
        m.no_swap = st.used.no_swap;
        m.no_devour = st.used.no_devour;

        if let Some(ch) = st.challenge.as_ref() {
            m.ch.vouchers = ch.vouchers;
            m.ch.contracts = [ch.contracts[0].id, ch.contracts[1].id, ch.contracts[2].id];
            if let Some(b) = ch.contracts.iter().find_map(|c| c.bounty) {
                m.ch.bounty = b;
            }
            for slot in 0..2 {
                if (slot as u8) < ch.deliveries_len {
                    let d = ch.deliveries[slot];
                    m.ch.status[slot] = if d.success {
                        ContractStatus::Delivered
                    } else {
                        ContractStatus::Skipped
                    };
                    m.ch.points[slot] = if d.success { d.points } else { 0 };
                } else {
                    m.ch.status[slot] = ContractStatus::Pending;
                    m.ch.points[slot] = 0;
                }
            }
        }
        m.stage = if st.phase == Phase::Ended {
            BoardRunStage::Ended
        } else {
            BoardRunStage::InProgress
        };
        m
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AdviceOptionRow {
    pub label: String,
    pub value_text: String,
    pub width_pct: f32,
    pub is_best: bool,
    pub is_policy: bool,
    pub sub_text: String,
    pub action: Option<SimAction>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AdviceSnapshot {
    pub working: bool,
    pub move_text: String,
    pub why: String,
    pub expect: String,
    pub source: String,
    pub lesson: String,
    pub status: String,
    pub note: String,
    pub problems: Vec<String>,
    pub checks: FixedVec<u8, 16>,
    pub treasure_wait: bool,
    pub tools_to_confirm: String,
    pub options: Vec<AdviceOptionRow>,
    pub best_action: Option<SimAction>,
    pub elapsed_ms: u64,
}

pub fn format_option_value(goal: SolveGoal, o: &OptionRank) -> String {
    if goal == SolveGoal::Contract {
        format!("{:.0}% pass \u{00b7} {:.0} pts", 100.0 * o.pass, o.points)
    } else if o.n > 1 {
        format!("{:.2} \u{00b1}{:.2}", o.mean, 1.96 * o.se)
    } else {
        format!("{:.2}", o.mean)
    }
}

pub fn format_expect_text(goal: SolveGoal, o: &OptionRank, max_days: u8) -> String {
    if goal == SolveGoal::Contract {
        format!(
            "Expected: {:.0}% chance to pass, {:.0} contract points.",
            100.0 * o.pass,
            o.points
        )
    } else {
        format!(
            "Expected: {:.2} Rainbows in the vault by the end of day {max_days}.",
            o.mean
        )
    }
}

impl BoardModel {
    /// Runs both the instant strategy pick and multi-threaded Monte Carlo playouts (`rayon`)
    /// to produce a complete `AdviceSnapshot` for the overlay.
    pub fn evaluate(
        &self,
        playout_budget_total: usize,
        budget_ms: u64,
        seed: u32,
    ) -> AdviceSnapshot {
        let t0 = std::time::Instant::now();
        let mut checks_vec = FixedVec::new();
        for n in 0..16u8 {
            if self.is_checked(n) {
                checks_vec.push(n);
            }
        }
        let tools_to_confirm = self.tools_prompt.format_for(&self.tools);

        if self.treasure == TreasurePhase::WaitingForBoardView {
            return AdviceSnapshot {
                working: false,
                move_text: "The Warden\u{2019}s Treasure: check the board first".to_string(),
                why:
                    "Press View Prison Status in the game, wait a second on the board, then return."
                        .to_string(),
                treasure_wait: true,
                tools_to_confirm,
                checks: checks_vec,
                elapsed_ms: t0.elapsed().as_millis() as u64,
                ..AdviceSnapshot::default()
            };
        }

        let (st, problems) = self.build_state(seed);
        if !problems.is_empty() {
            return AdviceSnapshot {
                working: false,
                move_text: String::new(),
                problems,
                checks: checks_vec,
                tools_to_confirm,
                elapsed_ms: t0.elapsed().as_millis() as u64,
                ..AdviceSnapshot::default()
            };
        }

        let p = SolverParams::for_state(&st);
        let cands = candidates(&st);
        let mut inst_rng = 1u32;
        let inst_action = if !cands.is_empty() {
            decide(&st, Some(p), &mut inst_rng)
        } else {
            None
        };
        let inst = inst_action.map(|action| {
            let label = cands
                .iter()
                .find(|c| c.action.same_action(&action))
                .map(|c| c.label.clone())
                .unwrap_or_else(|| action.type_name().to_string());
            let why = reason(&st, &action, Some(p));
            InstantPick { action, label, why }
        });

        let num_options = cands.len().max(1);
        let rollouts_per_option = (playout_budget_total / num_options).clamp(800, 32_000);
        let summary = solve_parallel_with_candidates(
            &st,
            cands,
            inst_action,
            rollouts_per_option,
            budget_ms,
            seed,
            true,
        );

        let ranked = &summary.options;
        let live = if ranked.len() > 1 && (summary.done || summary.rounds >= LIVE_MIN) {
            ranked.first()
        } else {
            None
        };
        let mine = inst
            .as_ref()
            .and_then(|pick| ranked.iter().find(|o| o.action.same_action(&pick.action)));

        let (rec_action, rec_label, rec_why, rec_live, rec_rounds, rec_opt) = if let Some(l) = live
        {
            if let Some(m) = mine {
                let p = inst.as_ref().unwrap();
                if m.action.same_action(&l.action) || l.mean - m.mean <= 1.96 * m.gap_se {
                    let why = if m.action.same_action(&l.action) {
                        p.why.clone()
                    } else {
                        format!("{} The live check rates it about equal to the best.", p.why)
                    };
                    (
                        Some(p.action),
                        m.label.clone(),
                        why,
                        true,
                        summary.rounds,
                        Some(m),
                    )
                } else {
                    let why = format!(
                        "The playouts found a better move than the strategy\u{2019}s pick ({}).",
                        p.label
                    );
                    (
                        Some(l.action),
                        l.label.clone(),
                        why,
                        true,
                        summary.rounds,
                        Some(l),
                    )
                }
            } else {
                let why =
                    "The playouts found a better move than the strategy\u{2019}s pick.".to_string();
                (
                    Some(l.action),
                    l.label.clone(),
                    why,
                    true,
                    summary.rounds,
                    Some(l),
                )
            }
        } else if let Some(p) = inst.as_ref() {
            (
                Some(p.action),
                p.label.clone(),
                p.why.clone(),
                false,
                0,
                mine,
            )
        } else if ranked.len() == 1 {
            let r0 = &ranked[0];
            (
                Some(r0.action),
                r0.label.clone(),
                "It\u{2019}s the only move here.".to_string(),
                true,
                summary.rounds,
                Some(r0),
            )
        } else {
            (
                None,
                "Nothing to decide here.".to_string(),
                String::new(),
                false,
                0,
                None,
            )
        };

        let expect = rec_opt
            .map(|o| format_expect_text(summary.goal, o, st.max_days))
            .unwrap_or_default();
        let source = if rec_action.is_some() {
            if rec_live && rec_rounds > 0 {
                format!("Checked by {rec_rounds} playouts per option")
            } else {
                "Instant pick from training".to_string()
            }
        } else {
            String::new()
        };

        let mut lesson = String::new();
        if let Some(SimAction::Devour { to }) = rec_action {
            let odds = chain_odds(&st, st.position, to);
            lesson = format!(
                "If the chain then goes on with the best meal at each bite: {:.1} bites on average, {:.0}% to reach Rainbow. Your game's Cancel can stop it at any bite.",
                odds.bites,
                100.0 * odds.rainbow
            );
        }

        let status = if ranked.len() <= 1 {
            if ranked.len() == 1 {
                "Only one move here.".to_string()
            } else {
                "Nothing to decide right now.".to_string()
            }
        } else {
            format!("Live check: {} playouts per option", summary.rounds)
        };

        let max_mean = ranked.iter().map(|o| o.mean).fold(0.01f64, f64::max);

        let mut option_rows = Vec::new();
        if ranked.len() > 1 {
            for o in ranked {
                let is_best = rec_action
                    .as_ref()
                    .is_some_and(|ra| o.action.same_action(ra));
                let width = if summary.goal == SolveGoal::Contract {
                    100.0 * o.pass
                } else {
                    (100.0 * o.mean) / max_mean
                };
                let sub_text = if let SimAction::Devour { to } = o.action {
                    let odds = chain_odds(&st, st.position, to);
                    format!(
                        "chain: {:.1} bites on average, {:.0}% to Rainbow",
                        odds.bites,
                        100.0 * odds.rainbow
                    )
                } else {
                    String::new()
                };
                option_rows.push(AdviceOptionRow {
                    label: o.label.clone(),
                    value_text: format_option_value(summary.goal, o),
                    width_pct: width.clamp(2.0, 100.0) as f32,
                    is_best,
                    is_policy: o.policy,
                    sub_text,
                    action: Some(o.action),
                });
            }
        }

        let close =
            ranked.len() >= 2 && (ranked[0].mean - ranked[1].mean) < 1.96 * ranked[1].gap_se;
        let note = if close {
            "The top moves are within the margin of error: either is fine.".to_string()
        } else {
            String::new()
        };

        AdviceSnapshot {
            working: false,
            move_text: rec_label,
            why: rec_why,
            expect,
            source,
            lesson,
            status,
            note,
            problems: Vec::new(),
            checks: checks_vec,
            treasure_wait: false,
            tools_to_confirm,
            options: option_rows,
            best_action: rec_action,
            elapsed_ms: t0.elapsed().as_millis() as u64,
        }
    }
}

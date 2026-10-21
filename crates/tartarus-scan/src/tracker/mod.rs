//! Run state tracker and frame processing lifecycle (`RunTracker`).

pub mod reconcile;

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use tartarus_core::{
    BoardModel, BoardRunStage, BoardStep, CHALLENGE_DAYS, CellKind, CellModel, ChoiceId,
    ChoiceModel, FixedVec, GameMode, GameParams, ObjectiveId, PendingKind, RunPhase, SimAction,
    StartEventId, ToolId, ToolsPrompt, TreasureOrigin, TreasurePhase,
};

use crate::log_watcher::{ActiveUiView, LogEvent};
use crate::ui_reader::{
    BoardRead, FrameView, ReadBoardOptions, RelicModalRead, TitleModalRead, ToolSlotRead,
    UiReadProfile, UiReader, UmgViewport,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureTrigger {
    Board { treasure_view: bool },
    Instruments { fresh: bool },
    Objective,
    Manual,
}

impl CaptureTrigger {
    #[inline]
    pub const fn rank(self) -> u8 {
        let r = match self {
            Self::Board { .. } => 1,
            Self::Instruments { .. } => 2,
            Self::Objective => 3,
            Self::Manual => 7,
        };
        assert!(r >= 1);
        assert!(r <= 7);
        r
    }
}

pub(crate) fn pick_is_confident(pick: &RelicModalRead) -> bool {
    assert_eq!(pick.cards.len(), 3);
    assert!(pick.cards[0].score.is_finite());
    if pick.cards.iter().any(|c| c.score < 0.80) {
        return false;
    }
    let mut scores: [f32; 3] = [
        pick.cards[0].score,
        pick.cards[1].score,
        pick.cards[2].score,
    ];
    scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    if scores[1] < 0.84 {
        return false;
    }
    let mut confs: [f32; 3] = [pick.cards[0].conf, pick.cards[1].conf, pick.cards[2].conf];
    confs.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    confs[1] >= 0.50
}

#[derive(Clone, Debug)]
pub struct ScanReading {
    pub model: BoardModel,
    pub checks: u16,
    pub notes: Vec<String>,
    pub chromatic: usize,
    pub tools: FixedVec<ToolSlotRead, 6>,
    pub chain_targets: Option<FixedVec<u8, 16>>,
}

#[derive(Clone, Debug)]
pub struct RunTracker {
    pub model: BoardModel,
    pub phase: RunPhase,
    pub last_board_at: Option<Instant>,
    pub hint: Option<UmgViewport>,
    pub run_objective: Option<ObjectiveId>,
    pub run_start_event: Option<StartEventId>,
    pub phantom_slots: BTreeMap<u16, usize>,
    pub last_advised_tool: Option<ToolId>,
    pub last_notes: Vec<String>,
    /// Ground-truth active `runId` from `[S16BGM] board Begin runId=<u64> IsSettled=<bool>`
    pub log_run_id: Option<u64>,
    /// Ground-truth `(cur_day, days_max)` from `[S16LastDay] check cur=<u8> max=<u8>`
    pub log_days: Option<(u8, u8)>,
    /// Ground-truth whether `S16BoardRelicPop` (3-card Instrument Pick modal) is currently open
    pub log_relic_pop_open: Option<bool>,
    /// Ground-truth active UI view from `UE_game.log`
    pub log_active_view: ActiveUiView,
    /// Whether the last ingested board frame had visible bottom action cards
    pub last_board_had_cards: bool,
}

impl Default for RunTracker {
    fn default() -> Self {
        let tracker = Self::new();
        assert_eq!(tracker.phase, RunPhase::WaitingForRun);
        assert!(tracker.last_board_at.is_none());
        tracker
    }
}

impl RunTracker {
    pub fn new() -> Self {
        let tracker = Self {
            model: BoardModel::blank(),
            phase: RunPhase::WaitingForRun,
            last_board_at: None,
            hint: None,
            run_objective: None,
            run_start_event: None,
            phantom_slots: BTreeMap::new(),
            last_advised_tool: None,
            last_notes: Vec::new(),
            log_run_id: None,
            log_days: None,
            log_relic_pop_open: None,
            log_active_view: ActiveUiView::Unknown,
            last_board_had_cards: false,
        };
        assert_eq!(tracker.phase, RunPhase::WaitingForRun);
        assert!(!tracker.last_board_had_cards);
        tracker
    }

    pub fn clear_run(&mut self) {
        self.model = BoardModel::blank();
        self.run_objective = None;
        self.run_start_event = None;
        self.last_advised_tool = None;
        self.log_days = None;
        self.last_board_had_cards = false;
        self.last_notes.clear();
        assert!(self.run_objective.is_none());
        assert!(self.last_notes.is_empty());
    }

    pub fn on_run_over(&mut self) -> bool {
        self.log_days = None;
        self.log_relic_pop_open = Some(false);
        self.log_active_view = ActiveUiView::CheckOutRiot;
        let transitioned = if self.phase != RunPhase::RunOver {
            self.phase = RunPhase::RunOver;
            self.last_board_at = None;
            self.clear_run();
            true
        } else {
            false
        };
        assert_eq!(self.phase, RunPhase::RunOver);
        assert!(self.log_days.is_none());
        transitioned
    }

    pub fn on_run_new(&mut self) -> bool {
        let transitioned = if self.phase.is_run_over() {
            self.phase = RunPhase::WaitingForRun;
            true
        } else {
            false
        };
        assert!(!self.phase.is_run_over());
        assert!(transitioned || self.phase != RunPhase::RunOver);
        transitioned
    }

    pub fn on_log_run_begin(&mut self, run_id: u64, is_settled: bool) -> bool {
        if is_settled {
            let res = self.on_run_over();
            assert_eq!(self.phase, RunPhase::RunOver);
            assert!(self.log_days.is_none());
            return res;
        }
        let changed = self.log_run_id.is_some_and(|prev| prev != run_id);
        self.log_run_id = Some(run_id);
        assert_eq!(self.log_run_id, Some(run_id));
        if changed {
            self.last_board_at = None;
            self.clear_run();
            self.phase = RunPhase::WaitingForRun;
            assert_eq!(self.phase, RunPhase::WaitingForRun);
            true
        } else {
            self.on_run_new()
        }
    }

    pub fn on_log_day_check(&mut self, cur: u8, max: u8) -> bool {
        if max == 0 {
            self.log_days = None;
            assert!(self.log_days.is_none());
            return false;
        }
        self.log_days = Some((cur, max));
        let gt_left = max.saturating_sub(cur.max(1));
        let prev_day = self.model.day;
        let prev_max = self.model.days_max;
        if self.model.mode == GameMode::Challenge {
            self.model.day = CHALLENGE_DAYS
                .saturating_sub(gt_left)
                .clamp(1, CHALLENGE_DAYS);
        } else {
            self.model.days_max = max.clamp(1, 60);
            self.model.day = self.model.days_max.saturating_sub(gt_left).max(1);
        }
        assert!(self.model.day >= 1);
        assert!(self.model.days_max >= 1);
        self.phase.is_active() && (self.model.day != prev_day || self.model.days_max != prev_max)
    }

    /// Applies a single `LogEvent` from `LogWatcher` and updates `log_active_view` / `log_days`.
    pub fn apply_log_event(&mut self, event: LogEvent) -> bool {
        let changed = match event {
            LogEvent::RunBegin { run_id, is_settled } => self.on_log_run_begin(run_id, is_settled),
            LogEvent::DayCheck { cur, max } => self.on_log_day_check(cur, max),
            LogEvent::BoardTrigger { .. } => {
                self.log_active_view = ActiveUiView::BoardMain;
                self.on_run_new()
            }
            LogEvent::InstrumentsPopup { fresh } => {
                self.log_active_view = ActiveUiView::RelicPop;
                self.log_relic_pop_open = Some(true);
                if fresh { self.on_run_new() } else { false }
            }
            LogEvent::RelicPopState(open) => {
                self.log_relic_pop_open = Some(open);
                if open {
                    self.log_active_view = ActiveUiView::RelicPop;
                } else if self.log_active_view == ActiveUiView::RelicPop {
                    self.log_active_view = ActiveUiView::BoardMain;
                }
                false
            }
            LogEvent::ObjectivePopup => {
                self.log_active_view = ActiveUiView::TaskPop;
                self.on_run_new()
            }
            LogEvent::TreasurePopup { .. } => {
                self.log_active_view = ActiveUiView::ColoredPop;
                false
            }
            LogEvent::SkipPending => {
                self.log_active_view = ActiveUiView::NewDayPop;
                false
            }
            LogEvent::SpeedFlip => false,
            LogEvent::Riot(on) => {
                if on {
                    self.log_active_view = ActiveUiView::CheckOutRiot;
                }
                false
            }
            LogEvent::RunOver => self.on_run_over(),
            LogEvent::RunNew => {
                self.log_active_view = ActiveUiView::Lobby;
                self.on_run_new()
            }
        };
        assert!(self.model.days_max >= 1);
        assert!(self.model.cells.len() == 16);
        changed
    }

    pub fn set_model(&mut self, new_model: BoardModel) {
        self.model = new_model;
        self.model.clean_prey();
        self.phase = RunPhase::Active {
            last_capture_matched: true,
        };
        assert!(self.phase.is_active());
        assert!(self.model.cells.len() == 16);
    }

    pub fn load_example(&mut self) {
        self.model = BoardModel::example();
        self.phase = RunPhase::Active {
            last_capture_matched: true,
        };
        assert!(self.phase.is_active());
        assert!(self.model.days_max >= 1);
    }

    pub fn set_start_event(&mut self, id: StartEventId) {
        self.model.start_event = id;
        if self.model.mode != GameMode::Challenge {
            let left = self.model.days_max.saturating_sub(self.model.day);
            let days = if id == StartEventId::IronLaw {
                30
            } else if self.model.days_max == 30 && id == StartEventId::None {
                GameParams::default().drill_days
            } else {
                self.model.days_max
            };
            self.model.days_max = days;
            self.model.day = days.saturating_sub(left).clamp(1, days);
        }
        self.model.clean_prey();
        assert_eq!(self.model.start_event, id);
        assert!(self.model.days_max >= 1);
    }

    pub fn use_title(&mut self, title: TitleModalRead) -> String {
        if let RunPhase::Active {
            last_capture_matched,
        } = &mut self.phase
        {
            *last_capture_matched = true;
        }
        let msg = match title {
            TitleModalRead::StartEvent(ev) => {
                self.run_start_event = Some(ev);
                self.set_start_event(ev);
                format!(
                    "This run's card: {}. It's set under Start of run.",
                    ev.name()
                )
            }
            TitleModalRead::Objective(obj) => {
                self.run_objective = Some(obj);
                self.model.objective = obj;
                self.model.walk_tiles = 0;
                self.model.milestone = false;
                self.model.clean_prey();
                format!(
                    "This run's objective: {}. It's set under Objective.",
                    obj.name()
                )
            }
        };
        assert!(!msg.is_empty());
        assert!(self.run_start_event.is_some() || self.run_objective.is_some());
        msg
    }

    fn reset_fresh_pick_model(&mut self) {
        let again =
            self.model.stage == BoardRunStage::Opening && self.model.step == BoardStep::Choice;
        let mode = self.model.mode;
        let rewards = self.model.rewards;
        let days_max = self.model.days_max;
        let objective = self.run_objective.unwrap_or(if again {
            self.model.objective
        } else {
            ObjectiveId::Other
        });
        let walk_tiles = if again { self.model.walk_tiles } else { 0 };
        let start_event = self.run_start_event.unwrap_or(if again {
            self.model.start_event
        } else {
            StartEventId::None
        });
        self.run_objective = None;
        self.run_start_event = None;

        let mut m = BoardModel::blank();
        m.mode = mode;
        m.rewards = rewards;
        m.days_max = days_max;
        m.stage = BoardRunStage::Opening;
        m.objective = objective;
        m.walk_tiles = walk_tiles;
        m.start_event = start_event;
        if start_event == StartEventId::IronLaw && m.mode != GameMode::Challenge {
            m.days_max = 30;
        }
        m.clean_prey();
        self.model = m;
        assert_eq!(self.model.stage, BoardRunStage::Opening);
        assert!(self.run_objective.is_none());
    }

    pub fn use_pick(&mut self, read: &RelicModalRead, fresh: bool) -> String {
        assert_eq!(read.cards.len(), 3);
        if fresh {
            self.reset_fresh_pick_model();
        }

        let card_tools = [read.cards[0].sim, read.cards[1].sim, read.cards[2].sim];
        self.model.step = BoardStep::Choice;
        self.model.choice =
            ChoiceModel::from_tools(PendingKind::Tool, &card_tools, u8::from(read.reroll));
        self.model.checks = 0;
        self.model.clean_prey();
        self.phase = RunPhase::Active {
            last_capture_matched: true,
        };
        assert!(self.phase.is_active());

        let names: [&str; 3] = [
            read.cards[0].sim.name(),
            read.cards[1].sim.name(),
            read.cards[2].sim.name(),
        ];
        format!(
            "{} {}{}.",
            if fresh {
                "A fresh run: the instrument offer is"
            } else {
                "An instrument offer:"
            },
            names.join(", "),
            if read.reroll {
                ", with a re-roll left"
            } else {
                ""
            }
        )
    }

    pub fn use_treasure(&mut self, after: Option<TreasureOrigin>) -> String {
        let set_gift = |m: &mut BoardModel| {
            m.step = BoardStep::Choice;
            m.choice = ChoiceModel::from_tools(PendingKind::Gift, &[], 0);
        };

        self.phase = RunPhase::Active {
            last_capture_matched: true,
        };
        assert!(self.phase.is_active());

        if after == Some(TreasureOrigin::View) {
            set_gift(&mut self.model);
            self.model.treasure = TreasurePhase::ReadyToChoose;
            self.model.clean_prey();
            assert_eq!(self.model.treasure, TreasurePhase::ReadyToChoose);
            return "Board seen behind the Warden's Treasure. Which gift to take is below."
                .to_string();
        }

        if self.model.step == BoardStep::Choice
            && self.model.choice.kind == PendingKind::Gift
            && self.model.treasure == TreasurePhase::ReadyToChoose
        {
            return "The Warden's Treasure is open.".to_string();
        }

        let mut how = String::new();
        let w = self.model.warden as usize;
        assert!(w < 16);
        if after == Some(TreasureOrigin::Swap) && self.model.cells[w].kind == CellKind::Inmate {
            let beast = self.model.cells[w].beast;
            self.model.cells[w] = if GameParams::default().reusable_cells {
                CellModel::sealed()
            } else {
                let mut e = CellModel::sealed();
                e.kind = CellKind::Empty;
                e
            };
            self.model.vault.try_push(beast);
            self.model.swap_used = true;
            self.model.consume_offered = true;
            how = format!(
                " It came from your Replace in cell {:02}: the newcomer was a Rainbow, \
                 now in the vault, and the cell locked again.",
                w + 1
            );
        } else if after == Some(TreasureOrigin::Consume) {
            self.model.devour_used = true;
            how = " It came from your consume: the next picture shows the board after it."
                .to_string();
        }

        set_gift(&mut self.model);
        self.model.treasure = TreasurePhase::WaitingForBoardView;
        self.model.clean_prey();
        format!(
            "The Warden's Treasure is open.{how} To advise a gift on the board as it is now, \
             press View Prison Status in the game, wait a second on the board, then return to \
             the Treasure."
        )
    }

    pub fn acknowledge_treasure_now(&mut self) {
        self.model.treasure = TreasurePhase::ReadyToChoose;
        self.model.clean_prey();
        self.phase = RunPhase::Active {
            last_capture_matched: true,
        };
        assert_eq!(self.model.treasure, TreasurePhase::ReadyToChoose);
        assert!(self.phase.is_active());
    }

    pub fn confirm_tools(&mut self) {
        let extra = self
            .model
            .tools_slots
            .saturating_sub(self.model.tools.len());
        self.model.extra_slots = extra;
        if self.model.tools_scale_step > 0 {
            if extra > 0 {
                self.phantom_slots
                    .insert(self.model.tools_scale_step, extra);
            } else {
                self.phantom_slots.remove(&self.model.tools_scale_step);
            }
        }
        self.model.tools_slots = 0;
        self.model.tools_prompt = ToolsPrompt::None;
        self.model.clean_prey();
        assert_eq!(self.model.tools_slots, 0);
        assert_eq!(self.model.tools_prompt, ToolsPrompt::None);
    }

    pub fn ingest(&mut self, board: &BoardRead) -> bool {
        assert!(!board.ok || board.cells.is_some());
        if let Some(vp) = board.viewport
            && board.ok
        {
            self.hint = Some(vp);
        }

        if !board.ok {
            return false;
        }

        let behind_treasure = self.model.treasure == TreasurePhase::WaitingForBoardView;
        self.last_board_had_cards = behind_treasure
            || board.doors.as_ref().is_some_and(|d| !d.cells.is_empty())
            || board.decision.is_some()
            || board.chain.is_some();
        let reading = self.scan_to_model(board);
        self.last_notes = reading.notes;
        self.model = reading.model;
        self.model.checks = reading.checks;
        self.model.clean_prey();
        self.last_board_at = Some(Instant::now());
        self.phase = RunPhase::Active {
            last_capture_matched: true,
        };
        assert!(self.phase.is_active());

        if behind_treasure {
            self.use_treasure(Some(TreasureOrigin::View));
        }

        true
    }

    fn try_pre_modal_read(
        &mut self,
        frame: &FrameView<'_>,
        trigger: CaptureTrigger,
        reader: &mut UiReader,
        t_start: Instant,
    ) -> Result<(bool, Option<String>, UiReadProfile), (bool, f64)> {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        let mut pick_already_checked = false;
        let mut pre_modal_ms = 0.0;

        if trigger == CaptureTrigger::Objective {
            let t_modal = Instant::now();
            let title_opt = reader.read_title_modal(frame);
            let modal_ms = t_modal.elapsed().as_secs_f64() * 1000.0;
            if let Some(title) = title_opt {
                let status = Some(self.use_title(title));
                return Ok((
                    true,
                    status,
                    UiReadProfile {
                        viewport_ms: 0.0,
                        board_ms: 0.0,
                        hud_ms: 0.0,
                        modal_ms,
                        ms_total: t_start.elapsed().as_secs_f64() * 1000.0,
                    },
                ));
            }
            pre_modal_ms += modal_ms;
        }

        if let CaptureTrigger::Instruments { fresh: fresh_hint } = trigger {
            pick_already_checked = true;
            let t_modal = Instant::now();
            let pick_opt = reader.read_relic_modal(frame);
            let modal_ms = t_modal.elapsed().as_secs_f64() * 1000.0;
            if let Some(pick) = pick_opt
                && pick_is_confident(&pick)
            {
                let is_fresh = fresh_hint
                    || !self.phase.is_active()
                    || self
                        .last_board_at
                        .is_none_or(|t| t.elapsed() > Duration::from_secs(300));
                let status = Some(self.use_pick(&pick, is_fresh));
                return Ok((
                    true,
                    status,
                    UiReadProfile {
                        viewport_ms: 0.0,
                        board_ms: 0.0,
                        hud_ms: 0.0,
                        modal_ms,
                        ms_total: t_start.elapsed().as_secs_f64() * 1000.0,
                    },
                ));
            }
            pre_modal_ms += modal_ms;
        }

        Err((pick_already_checked, pre_modal_ms))
    }

    fn try_fallback_modal_read(
        &mut self,
        frame: &FrameView<'_>,
        trigger: CaptureTrigger,
        reader: &mut UiReader,
        pick_already_checked: bool,
        profile: &mut UiReadProfile,
    ) -> Option<String> {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        let skip_pick = self.log_relic_pop_open == Some(false) && trigger != CaptureTrigger::Manual;
        if !pick_already_checked && !skip_pick {
            let t_modal = Instant::now();
            let pick_opt = reader.read_relic_modal(frame);
            profile.modal_ms += t_modal.elapsed().as_secs_f64() * 1000.0;
            if let Some(pick) = pick_opt
                && pick_is_confident(&pick)
            {
                let fresh_hint = matches!(trigger, CaptureTrigger::Instruments { fresh: true });
                let is_fresh = fresh_hint
                    || !self.phase.is_active()
                    || self
                        .last_board_at
                        .is_none_or(|t| t.elapsed() > Duration::from_secs(300));
                return Some(self.use_pick(&pick, is_fresh));
            }
        }

        if trigger != CaptureTrigger::Objective {
            let t_modal = Instant::now();
            let title_opt = reader.read_title_modal(frame);
            profile.modal_ms += t_modal.elapsed().as_secs_f64() * 1000.0;
            if let Some(title) = title_opt {
                return Some(self.use_title(title));
            }
        }
        None
    }

    /// Unified frame-processing pipeline used by BOTH automatic log-triggered captures
    /// and manual `"Picture"` / hotkey captures.
    pub fn process_frame(
        &mut self,
        frame: &FrameView<'_>,
        trigger: CaptureTrigger,
        reader: &mut UiReader,
    ) -> (bool, Option<String>, UiReadProfile) {
        assert!(frame.width() > 0);
        assert!(frame.height() > 0);
        let t_start = Instant::now();
        let (pick_already_checked, pre_modal_ms) =
            match self.try_pre_modal_read(frame, trigger, reader, t_start) {
                Ok(res) => return res,
                Err(pair) => pair,
            };

        let opts = ReadBoardOptions {
            read_days: !self.log_days.is_some_and(|(_, max)| max > 0),
        };
        let mut board_res = reader.read_board(frame, self.hint, opts);
        board_res.profile.modal_ms += pre_modal_ms;
        if board_res.ok {
            let recognized = self.ingest(&board_res);
            if recognized
                && self.model.objective == ObjectiveId::Other
                && !self.model.milestone
                && let Some(vp) = board_res.viewport.as_ref()
                && let Some(obj) = reader.read_hud_objective(frame, vp)
            {
                self.model.objective = obj;
            }
            let treasure_view = matches!(
                trigger,
                CaptureTrigger::Board {
                    treasure_view: true
                }
            );
            let custom_status = (recognized && treasure_view)
                .then(|| self.use_treasure(Some(TreasureOrigin::View)));
            board_res.profile.ms_total = t_start.elapsed().as_secs_f64() * 1000.0;
            return (recognized, custom_status, board_res.profile);
        }

        if let Some(status) = self.try_fallback_modal_read(
            frame,
            trigger,
            reader,
            pick_already_checked,
            &mut board_res.profile,
        ) {
            board_res.profile.ms_total = t_start.elapsed().as_secs_f64() * 1000.0;
            return (true, Some(status), board_res.profile);
        }

        if let RunPhase::Active {
            last_capture_matched,
        } = &mut self.phase
        {
            *last_capture_matched = false;
        }

        board_res.profile.ms_total = t_start.elapsed().as_secs_f64() * 1000.0;
        (false, None, board_res.profile)
    }

    pub fn record_advice(&mut self, best_action: Option<SimAction>) {
        if let Some(SimAction::Choose(ChoiceId::Tool(t))) = best_action {
            self.last_advised_tool = Some(t);
            assert_eq!(self.last_advised_tool, Some(t));
        }
        assert!(self.model.cells.len() == 16);
    }
}

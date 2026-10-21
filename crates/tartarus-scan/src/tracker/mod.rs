//! Run state tracker and frame processing lifecycle.

pub mod reconcile;

use crate::cv::{
    BoardLoc, ImageView, PickScreenRead, ScanArena, ScanOptions, ScanProfile, ScanResult,
    ScannedTool, TitleRead, read_hud_objective_with_slot, read_objective_title_with_slot,
    scan_image_with_options_and_arena, scan_pick_with_slot,
};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use tartarus_core::{
    BoardModel, BoardRunStage, BoardStep, CHALLENGE_DAYS, CellKind, CellModel, ChoiceId,
    ChoiceModel, FixedVec, GameMode, GameParams, ObjectiveId, PendingKind, RunPhase, SimAction,
    StartEventId, ToolId, ToolsPrompt, TreasureOrigin, TreasurePhase,
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
        match self {
            Self::Board { .. } => 1,
            Self::Instruments { .. } => 2,
            Self::Objective => 3,
            Self::Manual => 7,
        }
    }
}

pub(crate) fn pick_is_confident(pick: &PickScreenRead) -> bool {
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
    pub tools: FixedVec<ScannedTool, 6>,
    pub chain_targets: Option<FixedVec<u8, 16>>,
}

#[derive(Clone, Debug)]
pub struct RunTracker {
    pub model: BoardModel,
    pub phase: RunPhase,
    pub last_board_at: Option<Instant>,
    pub hint: Option<BoardLoc>,
    pub run_objective: Option<ObjectiveId>,
    pub run_start_event: Option<StartEventId>,
    pub phantom_slots: BTreeMap<u16, usize>,
    pub last_advised_tool: Option<ToolId>,
    pub last_notes: Vec<String>,
    /// Ground-truth active `runId` from `[S16BGM] board Begin runId=<u64> IsSettled=<bool>`
    pub log_run_id: Option<u64>,
    /// Ground-truth `(cur_day, max_days)` from `[S16LastDay] check cur=<u8> max=<u8>`
    pub log_days: Option<(u8, u8)>,
    /// Ground-truth whether `S16BoardRelicPop` (3-card Instrument Pick modal) is currently open
    pub log_relic_pop_open: Option<bool>,
    /// Whether the last ingested board frame had visible bottom action cards (`doors`, `decision`, or `chain`)
    pub last_board_had_cards: bool,
}

impl Default for RunTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl RunTracker {
    pub fn new() -> Self {
        Self {
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
            last_board_had_cards: false,
        }
    }

    pub fn clear_run(&mut self) {
        self.model = BoardModel::blank();
        self.run_objective = None;
        self.run_start_event = None;
        self.last_advised_tool = None;
        self.log_days = None;
        self.last_board_had_cards = false;
        self.last_notes.clear();
    }

    pub fn on_run_over(&mut self) -> bool {
        self.log_days = None;
        self.log_relic_pop_open = Some(false);
        if self.phase != RunPhase::RunOver {
            self.phase = RunPhase::RunOver;
            self.last_board_at = None;
            self.clear_run();
            true
        } else {
            false
        }
    }

    pub fn on_run_new(&mut self) -> bool {
        if self.phase.is_run_over() {
            self.phase = RunPhase::WaitingForRun;
            true
        } else {
            false
        }
    }

    pub fn on_log_run_begin(&mut self, run_id: u64, is_settled: bool) -> bool {
        if is_settled {
            return self.on_run_over();
        }
        let changed = self.log_run_id.is_some_and(|prev| prev != run_id);
        self.log_run_id = Some(run_id);
        if changed {
            self.last_board_at = None;
            self.clear_run();
            self.phase = RunPhase::WaitingForRun;
            true
        } else {
            self.on_run_new()
        }
    }

    pub fn on_log_day_check(&mut self, cur: u8, max: u8) -> bool {
        if max == 0 {
            self.log_days = None;
            return false;
        }
        self.log_days = Some((cur, max));
        let gt_left = max.saturating_sub(cur.max(1));
        let prev_day = self.model.day;
        let prev_max = self.model.max_days;
        if self.model.mode == GameMode::Challenge {
            self.model.day = CHALLENGE_DAYS
                .saturating_sub(gt_left)
                .clamp(1, CHALLENGE_DAYS);
        } else {
            self.model.max_days = max.clamp(1, 60);
            self.model.day = self.model.max_days.saturating_sub(gt_left).max(1);
        }
        self.phase.is_active() && (self.model.day != prev_day || self.model.max_days != prev_max)
    }

    pub fn set_model(&mut self, new_model: BoardModel) {
        self.model = new_model;
        self.model.clean_prey();
        self.phase = RunPhase::Active {
            last_capture_matched: true,
        };
    }

    pub fn load_example(&mut self) {
        self.model = BoardModel::example();
        self.phase = RunPhase::Active {
            last_capture_matched: true,
        };
    }

    pub fn set_start_event(&mut self, id: StartEventId) {
        self.model.start_event = id;
        if self.model.mode != GameMode::Challenge {
            let left = self.model.max_days.saturating_sub(self.model.day);
            let days = if id == StartEventId::IronLaw {
                30
            } else if self.model.max_days == 30 && id == StartEventId::None {
                GameParams::default().drill_days
            } else {
                self.model.max_days
            };
            self.model.max_days = days;
            self.model.day = days.saturating_sub(left).clamp(1, days);
        }
        self.model.clean_prey();
    }

    pub fn use_title(&mut self, title: TitleRead) -> String {
        if let RunPhase::Active {
            last_capture_matched,
        } = &mut self.phase
        {
            *last_capture_matched = true;
        }
        match title {
            TitleRead::StartEvent(ev) => {
                self.run_start_event = Some(ev);
                self.set_start_event(ev);
                format!(
                    "This run's card: {}. It's set under Start of run.",
                    ev.name()
                )
            }
            TitleRead::Objective(obj) => {
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
        }
    }

    pub fn use_pick(&mut self, read: &PickScreenRead, fresh: bool) -> String {
        if fresh {
            let again =
                self.model.stage == BoardRunStage::Opening && self.model.step == BoardStep::Choice;
            let mode = self.model.mode;
            let rewards = self.model.rewards;
            let max_days = self.model.max_days;
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
            m.max_days = max_days;
            m.stage = BoardRunStage::Opening;
            m.objective = objective;
            m.walk_tiles = walk_tiles;
            m.start_event = start_event;
            if start_event == StartEventId::IronLaw && m.mode != GameMode::Challenge {
                m.max_days = 30;
            }
            m.clean_prey();
            self.model = m;
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

        if after == Some(TreasureOrigin::View) {
            set_gift(&mut self.model);
            self.model.treasure = TreasurePhase::ReadyToChoose;
            self.model.clean_prey();
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
                " It came from your Replace in cell {:02}: the newcomer was a Rainbow, now in the vault, and the cell locked again.",
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
            "The Warden's Treasure is open.{how} To advise a gift on the board as it is now, press View Prison Status in the game, wait a second on the board, then return to the Treasure."
        )
    }

    pub fn acknowledge_treasure_now(&mut self) {
        self.model.treasure = TreasurePhase::ReadyToChoose;
        self.model.clean_prey();
        self.phase = RunPhase::Active {
            last_capture_matched: true,
        };
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
    }

    pub fn ingest(&mut self, scan: &ScanResult) -> bool {
        if let Some(loc) = scan.loc
            && scan.ok
        {
            self.hint = Some(loc);
        }

        if let Some(title) = scan.title {
            self.use_title(title);
        }

        if !scan.ok {
            if let Some(ref pick) = scan.pick
                && pick_is_confident(pick)
            {
                let fresh = self.model.day >= self.model.max_days
                    || (self.model.tools.is_empty() && self.model.day <= 1);
                self.use_pick(pick, fresh);
                return true;
            }
            return false;
        }

        let behind_treasure = self.model.treasure == TreasurePhase::WaitingForBoardView;
        self.last_board_had_cards = behind_treasure
            || scan.doors.as_ref().is_some_and(|d| !d.cells.is_empty())
            || scan.decision.is_some()
            || scan.chain.is_some();
        let reading = self.scan_to_model(scan);
        self.last_notes = reading.notes;
        self.model = reading.model;
        self.model.checks = reading.checks;
        self.model.clean_prey();
        self.last_board_at = Some(Instant::now());
        self.phase = RunPhase::Active {
            last_capture_matched: true,
        };

        if behind_treasure {
            self.use_treasure(Some(TreasureOrigin::View));
        }

        true
    }

    /// Unified frame-processing pipeline used by BOTH automatic log-triggered captures
    /// and manual `"Picture"` / hotkey captures so state transitions are 100% identical.
    pub fn process_frame(
        &mut self,
        view: &ImageView<'_>,
        trigger: CaptureTrigger,
        arena: &mut ScanArena,
    ) -> (bool, Option<String>, ScanProfile) {
        let t_total = Instant::now();
        let mut custom_status: Option<String> = None;
        let mut pick_already_checked = false;
        let mut pre_pick_ms = 0.0;

        // 1. If triggered by an Objective/Start-Event popup log, check objective title first
        if trigger == CaptureTrigger::Objective {
            let t_pick = Instant::now();
            let title_opt = read_objective_title_with_slot(view, &mut arena.slots[0]);
            let pick_ms = t_pick.elapsed().as_secs_f64() * 1000.0;
            if let Some(title) = title_opt {
                custom_status = Some(self.use_title(title));
                return (
                    true,
                    custom_status,
                    ScanProfile {
                        loc_ms: 0.0,
                        cells_ms: 0.0,
                        ocr_ms: 0.0,
                        pick_ms,
                        total_ms: t_total.elapsed().as_secs_f64() * 1000.0,
                    },
                );
            }
            pre_pick_ms += pick_ms;
        }

        // 2. If triggered by an Instruments popup log, check the 3-card Pick screen first
        if let CaptureTrigger::Instruments { fresh: fresh_hint } = trigger {
            pick_already_checked = true;
            let t_pick = Instant::now();
            let pick_opt = scan_pick_with_slot(view, &mut arena.slots[0]);
            let pick_ms = t_pick.elapsed().as_secs_f64() * 1000.0;
            if let Some(pick) = pick_opt
                && pick_is_confident(&pick)
            {
                let is_fresh = fresh_hint
                    || !self.phase.is_active()
                    || self
                        .last_board_at
                        .is_none_or(|t| t.elapsed() > Duration::from_secs(300));
                custom_status = Some(self.use_pick(&pick, is_fresh));
                return (
                    true,
                    custom_status,
                    ScanProfile {
                        loc_ms: 0.0,
                        cells_ms: 0.0,
                        ocr_ms: 0.0,
                        pick_ms,
                        total_ms: t_total.elapsed().as_secs_f64() * 1000.0,
                    },
                );
            }
            pre_pick_ms += pick_ms;
        }

        // 3. Try scanning the 4x4 prison board
        let opts = ScanOptions {
            skip_days: self.log_days.is_some_and(|(_, max)| max > 0),
        };
        let mut scan_res = scan_image_with_options_and_arena(view, self.hint, opts, arena);
        scan_res.profile.pick_ms += pre_pick_ms;
        if scan_res.ok {
            let recognized = self.ingest(&scan_res);
            if recognized
                && self.model.objective == ObjectiveId::Other
                && !self.model.milestone
                && let Some(loc) = scan_res.loc.as_ref()
                && let Some(obj) = read_hud_objective_with_slot(view, loc, &mut arena.slots[0])
            {
                self.model.objective = obj;
            }
            let treasure_view = matches!(
                trigger,
                CaptureTrigger::Board {
                    treasure_view: true
                }
            );
            if recognized && treasure_view {
                custom_status = Some(self.use_treasure(Some(TreasureOrigin::View)));
            }
            scan_res.profile.total_ms = t_total.elapsed().as_secs_f64() * 1000.0;
            return (recognized, custom_status, scan_res.profile);
        }

        // 4. Try scanning the 3-card Instrument Pick screen
        let skip_pick = self.log_relic_pop_open == Some(false) && trigger != CaptureTrigger::Manual;
        if !pick_already_checked && !skip_pick {
            let t_pick = Instant::now();
            let pick_opt = scan_pick_with_slot(view, &mut arena.slots[0]);
            scan_res.profile.pick_ms += t_pick.elapsed().as_secs_f64() * 1000.0;
            if let Some(pick) = pick_opt
                && pick_is_confident(&pick)
            {
                let fresh_hint = matches!(trigger, CaptureTrigger::Instruments { fresh: true });
                let is_fresh = fresh_hint
                    || !self.phase.is_active()
                    || self
                        .last_board_at
                        .is_none_or(|t| t.elapsed() > Duration::from_secs(300));
                custom_status = Some(self.use_pick(&pick, is_fresh));
                scan_res.profile.total_ms = t_total.elapsed().as_secs_f64() * 1000.0;
                return (true, custom_status, scan_res.profile);
            }
        }

        // 5. Try scanning an Objective / Start-Event scroll popup (e.g. Manual picture or fallback)
        if trigger != CaptureTrigger::Objective {
            let t_pick = Instant::now();
            let title_opt = read_objective_title_with_slot(view, &mut arena.slots[0]);
            scan_res.profile.pick_ms += t_pick.elapsed().as_secs_f64() * 1000.0;
            if let Some(title) = title_opt {
                custom_status = Some(self.use_title(title));
                scan_res.profile.total_ms = t_total.elapsed().as_secs_f64() * 1000.0;
                return (true, custom_status, scan_res.profile);
            }
        }

        if let RunPhase::Active {
            last_capture_matched,
        } = &mut self.phase
        {
            *last_capture_matched = false;
        }

        scan_res.profile.total_ms = t_total.elapsed().as_secs_f64() * 1000.0;
        (false, custom_status, scan_res.profile)
    }

    pub fn record_advice(&mut self, best_action: Option<SimAction>) {
        if let Some(SimAction::Choose(ChoiceId::Tool(t))) = best_action {
            self.last_advised_tool = Some(t);
        }
    }
}

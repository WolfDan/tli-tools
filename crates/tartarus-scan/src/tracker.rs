use crate::cv::{
    BoardLoc, ImageView, PickScreenRead, ScanArena, ScanOptions, ScanProfile, ScanResult,
    ScannedCellKind, ScannedTool, TitleRead, read_hud_objective_with_slot,
    read_objective_title_with_slot, scan_image_with_options_and_arena, scan_pick_with_slot,
};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use tartarus_core::{
    BEASTS, Beast, BoardModel, BoardRunStage, BoardStep, CHALLENGE_DAYS, CellEvent, CellKind,
    CellModel, ChoiceId, ChoiceModel, FixedVec, GameMode, GameParams, OFFICE, ObjectiveId,
    PendingKind, RunPhase, SUPREME, SimAction, StartEventId, TOOL_SURE, ToolId, ToolsPrompt,
    TreasureOrigin, TreasurePhase,
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

fn pick_is_confident(pick: &PickScreenRead) -> bool {
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
            self.model.vault.push(beast);
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
        if let Some(loc) = scan.loc {
            if scan.ok {
                self.hint = Some(loc);
            }
        }

        if let Some(title) = scan.title {
            self.use_title(title);
        }

        if !scan.ok {
            if let Some(ref pick) = scan.pick {
                if pick_is_confident(pick) {
                    let fresh = self.model.day >= self.model.max_days
                        || (self.model.tools.is_empty() && self.model.day <= 1);
                    self.use_pick(pick, fresh);
                    return true;
                }
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
            if let Some(pick) = pick_opt {
                if pick_is_confident(&pick) {
                    let is_fresh = fresh_hint
                        || !self.phase.is_active()
                        || self
                            .last_board_at
                            .map_or(true, |t| t.elapsed() > Duration::from_secs(300));
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
            if recognized && self.model.objective == ObjectiveId::Other && !self.model.milestone {
                if let Some(loc) = scan_res.loc.as_ref() {
                    if let Some(obj) = read_hud_objective_with_slot(view, loc, &mut arena.slots[0])
                    {
                        self.model.objective = obj;
                    }
                }
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
            if let Some(pick) = pick_opt {
                if pick_is_confident(&pick) {
                    let fresh_hint = matches!(trigger, CaptureTrigger::Instruments { fresh: true });
                    let is_fresh = fresh_hint
                        || !self.phase.is_active()
                        || self
                            .last_board_at
                            .map_or(true, |t| t.elapsed() > Duration::from_secs(300));
                    custom_status = Some(self.use_pick(&pick, is_fresh));
                    scan_res.profile.total_ms = t_total.elapsed().as_secs_f64() * 1000.0;
                    return (true, custom_status, scan_res.profile);
                }
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

    #[inline]
    fn scale_step(scale: Option<f64>) -> u16 {
        scale
            .map(|s| (s * 20.0).round().clamp(0.0, u16::MAX as f64) as u16)
            .unwrap_or(0)
    }

    fn kept_tusk(base: &BoardModel, n: usize, beast: Beast, tier: u8) -> bool {
        let was = &base.cells[n];
        was.kind == CellKind::Inmate && was.ivory && was.beast == beast && tier >= was.tier
    }

    fn run_tools(
        &self,
        read: &[ScannedTool],
        base: &BoardModel,
        m: &mut BoardModel,
        notes: &mut Vec<String>,
        scale: Option<f64>,
    ) -> FixedVec<ScannedTool, 6> {
        let same_run = !(m.day < base.day);
        let known: &[ToolId] = if same_run { &base.tools } else { &[] };
        let offer: &[ToolId] = if same_run
            && base.step == BoardStep::Choice
            && base.choice.kind == PendingKind::Tool
        {
            base.choice.slice()
        } else {
            &[]
        };

        let mut out: FixedVec<ScannedTool, 6> = known
            .iter()
            .map(|&sim| ScannedTool {
                id: sim.as_str(),
                sim,
                name: sim.name(),
                score: 1.0,
                conf: 1.0,
            })
            .collect();

        let has = |list: &[ScannedTool], id: ToolId| list.iter().any(|t| t.sim == id);
        let slots = read.len();

        m.tools_prompt = ToolsPrompt::None;
        m.tools_slots = 0;
        m.tools_scale_step = Self::scale_step(scale);
        let phantom = self
            .phantom_slots
            .get(&m.tools_scale_step)
            .copied()
            .unwrap_or(0);
        let extra = m.extra_slots.max(phantom);

        if same_run
            && (!known.is_empty() || !offer.is_empty() || base.stage == BoardRunStage::Opening)
        {
            // Self-heal any existing slot if a much higher confidence read disagrees
            // (e.g. when a transient overlay occlusion caused an initial misread).
            for i in 0..out.len().min(read.len()) {
                if read[i].sim != out[i].sim
                    && read[i].conf >= 0.95
                    && read[i].score >= 0.94
                    && !has(&out, read[i].sim)
                {
                    out[i] = read[i];
                }
            }

            if !offer.is_empty() && slots > known.len() && !offer.iter().any(|&id| has(&out, id)) {
                let mut seen: FixedVec<ScannedTool, 6> = read
                    .iter()
                    .copied()
                    .filter(|t| offer.contains(&t.sim))
                    .collect();
                seen.as_mut_slice()
                    .sort_by(|a, b| b.conf.partial_cmp(&a.conf).unwrap());
                let advised = self.last_advised_tool.filter(|id| offer.contains(id));
                let take = seen.first().map(|t| t.sim).or(advised);
                if let Some(t_id) = take {
                    out.push(ScannedTool {
                        id: t_id.as_str(),
                        sim: t_id,
                        name: t_id.name(),
                        score: 1.0,
                        conf: 1.0,
                    });
                    if seen.is_empty() {
                        notes.push(format!(
                            "Took it you chose {}, as advised. If you took another, change it under Torture Instrument.",
                            t_id.name()
                        ));
                    }
                } else {
                    let offer_names: Vec<&str> = offer.iter().map(|id| id.name()).collect();
                    notes.push(format!(
                        "Your instruments show one more than before, from the offer of {}: set which you took under Torture Instrument.",
                        offer_names.join(", ")
                    ));
                }
            } else if offer.is_empty() && slots > out.len() {
                // Mid-run milestone / treasure instrument acquired without a prior Pick screen capture:
                // accept any newly scanned high-confidence tool slots directly.
                for t in &read[out.len()..] {
                    if !has(&out, t.sim) && (t.conf as f64) >= TOOL_SURE && t.score >= 0.88 {
                        out.push(*t);
                    }
                }
            }

            let off_tools: FixedVec<ToolId, 8> = read
                .iter()
                .filter(|t| (t.conf as f64) >= TOOL_SURE && !has(&out, t.sim))
                .map(|t| t.sim)
                .collect();
            if slots > out.len() + extra {
                m.tools_slots = slots;
                m.tools_prompt = ToolsPrompt::CountMismatch {
                    panel_slots: slots as u8,
                };
            } else if !off_tools.is_empty() {
                m.tools_prompt = ToolsPrompt::Unexpected { off: off_tools };
            }
            return out;
        }

        for t in read {
            if !has(&out, t.sim) && (t.conf as f64) >= TOOL_SURE {
                out.push(*t);
            }
        }
        if !read.is_empty() {
            m.tools_slots = read.len();
            m.tools_prompt = ToolsPrompt::UnknownRun {
                panel_slots: read.len() as u8,
            };
        }
        out
    }

    fn chain_to_model(&self, r: &ScanResult, base: &BoardModel) -> ScanReading {
        let mut m = *base;
        let mut checks = 0u16;
        let mut notes = Vec::new();
        let cells = r.cells.as_ref().unwrap();
        let chain = r.chain.as_ref().unwrap();

        let warden = r.warden.map_or(base.warden, |w| w.cell);
        let after = base.warden == warden
            && (base.step == BoardStep::Manage || base.step == BoardStep::Chain);
        let walked = base.step == BoardStep::Patrol && base.has_dest(warden);

        if !after && !walked {
            for (n, c) in cells.iter().enumerate() {
                if n == (OFFICE as usize) || n == (warden as usize) {
                    continue;
                }
                match c.kind {
                    ScannedCellKind::Locked => {
                        m.cells[n] = CellModel::sealed();
                    }
                    ScannedCellKind::Inmate => {
                        let tier = c.tier.min(4);
                        let mut cm = CellModel::sealed();
                        cm.kind = CellKind::Inmate;
                        cm.beast = c.beast;
                        cm.tier = tier;
                        cm.ivory = Self::kept_tusk(base, n, c.beast, tier);
                        m.cells[n] = cm;
                    }
                    _ => {
                        let mut cm = CellModel::sealed();
                        cm.kind = CellKind::Empty;
                        m.cells[n] = cm;
                    }
                }
                if c.kind != ScannedCellKind::Locked {
                    checks |= 1u16 << n;
                }
            }
            notes.push("A consume's meal pick darkens the board, so its rarities were guessed: check the marked cells.".to_string());
        }

        m.warden = warden;
        let w_idx = warden as usize;
        let was = base.cells[w_idx];
        let eater = &cells[w_idx];
        if eater.kind == ScannedCellKind::Inmate {
            let mut cm = CellModel::sealed();
            cm.kind = CellKind::Inmate;
            cm.beast = eater.beast;
            cm.tier = eater.tier.min(4);
            cm.ivory = was.ivory;
            cm.prey = if was.kind == CellKind::Inmate {
                was.prey
            } else {
                None
            };
            m.cells[w_idx] = cm;
        } else {
            checks |= 1u16 << w_idx;
            notes.push(
                "The eater in the warden's cell wasn't read: set it as it is now.".to_string(),
            );
        }

        let tier = if m.cells[w_idx].kind == CellKind::Inmate {
            m.cells[w_idx].tier
        } else {
            0
        };

        let mut eaten = 0usize;
        if after || walked {
            for (n, c) in cells.iter().enumerate() {
                if n == (OFFICE as usize)
                    || n == w_idx
                    || c.kind != ScannedCellKind::Locked
                    || m.cells[n].kind != CellKind::Inmate
                {
                    continue;
                }
                m.cells[n] = CellModel::sealed();
                eaten += 1;
            }
        }

        for &target_n in &chain.targets {
            let n = target_n as usize;
            let c = m.cells[n];
            if c.kind == CellKind::Scratch {
                continue;
            }
            if c.kind == CellKind::Inmate && c.tier <= tier && c.beast == cells[n].beast {
                continue;
            }
            let mut cm = CellModel::sealed();
            cm.kind = CellKind::Inmate;
            cm.beast = cells[n].beast;
            cm.tier = tier;
            cm.ivory = c.kind == CellKind::Inmate && c.ivory;
            m.cells[n] = cm;
            checks |= 1u16 << n;
        }

        let _ = (eaten, walked);
        m.step = BoardStep::Chain;
        m.devour_used = true;
        m.no_devour = false;
        m.no_swap = false;
        m.treasure = TreasurePhase::Closed;
        m.clear_dest();
        m.chain_armed = false;
        m.stage = BoardRunStage::InProgress;

        let tools: FixedVec<ScannedTool, 6> = m
            .tools
            .iter()
            .map(|&id| ScannedTool {
                id: id.as_str(),
                sim: id,
                name: id.name(),
                score: 1.0,
                conf: 1.0,
            })
            .collect();

        ScanReading {
            chromatic: m.vault.len(),
            model: m,
            checks,
            notes,
            tools,
            chain_targets: Some(chain.targets),
        }
    }

    pub fn scan_to_model(&self, r: &ScanResult) -> ScanReading {
        let base = &self.model;
        if r.chain.is_some() {
            return self.chain_to_model(r, base);
        }

        let mut m = *base;
        m.stage = BoardRunStage::InProgress;
        let mut checks = 0u16;
        let mut notes = Vec::new();
        let cells = r.cells.as_ref().unwrap();

        if let Some((cur, max)) = self.log_days.filter(|&(_, max)| max > 0) {
            let gt_left = max.saturating_sub(cur.max(1));
            if m.mode == GameMode::Challenge {
                m.day = CHALLENGE_DAYS
                    .saturating_sub(gt_left)
                    .clamp(1, CHALLENGE_DAYS);
            } else {
                m.max_days = max.clamp(1, 60);
                m.day = m.max_days.saturating_sub(gt_left).max(1);
            }
        } else if let Some((left, _conf)) = r.days {
            if m.mode == GameMode::Challenge {
                m.day = CHALLENGE_DAYS.saturating_sub(left).clamp(1, CHALLENGE_DAYS);
            } else {
                if left >= m.max_days {
                    m.max_days = (left + 1).min(60);
                }
                m.day = m.max_days.saturating_sub(left).max(1);
            }
        } else {
            notes.push("The days until the riot weren't read: check the day.".to_string());
        }

        for (n, c) in cells.iter().enumerate() {
            if n == (OFFICE as usize) {
                continue;
            }
            match c.kind {
                ScannedCellKind::Locked => {
                    if c.nest {
                        let mut cm = CellModel::sealed();
                        cm.kind = CellKind::Nest;
                        cm.due = m.day + 2;
                        m.cells[n] = cm;
                        checks |= 1u16 << n;
                    } else {
                        let was = base.cells[n];
                        let event = if c.event {
                            CellEvent::Unknown
                        } else {
                            CellEvent::None
                        };
                        if was.kind == CellKind::Revealed {
                            let mut cm = CellModel::sealed();
                            cm.kind = CellKind::Revealed;
                            cm.tier = was.tier;
                            cm.event = event;
                            m.cells[n] = cm;
                        } else if was.kind == CellKind::Sealed && was.at_least > 0 {
                            let mut cm = CellModel::sealed();
                            cm.event = event;
                            cm.at_least = was.at_least;
                            m.cells[n] = cm;
                        } else {
                            let mut cm = CellModel::sealed();
                            cm.event = event;
                            m.cells[n] = cm;
                        }
                    }
                    if c.conf < 0.60 {
                        checks |= 1u16 << n;
                    }
                }
                ScannedCellKind::Inmate => {
                    let tier = c.tier.min(4);
                    let mut cm = CellModel::sealed();
                    cm.kind = CellKind::Inmate;
                    cm.beast = c.beast;
                    cm.tier = tier;
                    cm.ivory = Self::kept_tusk(base, n, c.beast, tier);
                    cm.event = if c.event {
                        CellEvent::Unknown
                    } else {
                        CellEvent::None
                    };
                    m.cells[n] = cm;
                    if c.conf < 0.60 || c.ghost {
                        checks |= 1u16 << n;
                    }
                }
                _ => {
                    let mut cm = CellModel::sealed();
                    cm.kind = CellKind::Empty;
                    m.cells[n] = cm;
                    checks |= 1u16 << n;
                }
            }
        }

        if let Some(w) = r.warden {
            m.warden = w.cell;
            if w.conf < 0.60 {
                checks |= 1u16 << (w.cell as usize);
            }
        } else {
            notes.push("The warden's arrow wasn't found: set the warden.".to_string());
        }

        if let Some(ref doors) = r.doors {
            if !doors.cells.is_empty() {
                m.step = BoardStep::Patrol;
                m.clear_dest();
                for &n in &doors.cells {
                    if n != m.warden {
                        m.set_dest(n, true);
                    }
                }
                let two_door_tool =
                    |id: ToolId| matches!(id, ToolId::Forged | ToolId::Whip | ToolId::Stake);
                let has_two_door_limit = base.tools.iter().copied().any(two_door_tool)
                    || r.tools
                        .iter()
                        .any(|t| (t.conf as f64) >= TOOL_SURE && two_door_tool(t.sim));
                let expected_doors = if has_two_door_limit { 2 } else { 3 };
                if doors.conf < 0.60 || doors.cells.len() < expected_doors {
                    notes.push("Today's doors were hard to read: check them.".to_string());
                }
            } else {
                m.step = BoardStep::Manage;
                m.clear_dest();
                m.devour_used = false;
                m.swap_used = false;
                m.hunt_used = false;
            }
        } else {
            m.step = BoardStep::Manage;
            m.clear_dest();
            m.devour_used = false;
            m.swap_used = false;
            m.hunt_used = false;
        }

        m.chain_armed = false;
        m.consume_offered = false;
        m.treasure = TreasurePhase::Closed;
        m.no_swap = false;
        m.no_devour = false;

        let same_visit = m.day == base.day
            && m.warden == base.warden
            && matches!(
                base.step,
                BoardStep::Manage | BoardStep::Choice | BoardStep::Event
            );
        if r.decision.is_none() && same_visit && m.step == BoardStep::Manage {
            m.swap_used = base.swap_used;
            m.devour_used = base.devour_used;
            m.hunt_used = base.hunt_used;
            m.no_swap = base.no_swap;
            m.no_devour = base.no_devour;
            m.consume_offered = base.consume_offered;
            let w_idx = m.warden as usize;
            let was = base.cells[w_idx];
            let now = m.cells[w_idx];
            if !m.devour_used
                && was.kind == CellKind::Inmate
                && now.kind == CellKind::Inmate
                && (was.beast != now.beast || was.tier != now.tier)
            {
                m.swap_used = true;
                m.consume_offered = true;
                notes.push(
                    "The warden's inmate changed since the board before, so today's Replace is taken as done."
                        .to_string(),
                );
                m.cells[w_idx].ivory = false;
            }
        }

        if let Some(dec) = r.decision {
            if dec.office {
                m.step = BoardStep::Office;
                m.clear_dest();
            } else if dec.ratway {
                m.step = BoardStep::Ratway;
                m.clear_dest();
            } else if m.warden == OFFICE && !dec.end {
                m.step = BoardStep::Office;
                m.clear_dest();
            } else {
                m.swap_used = false;
                m.devour_used = false;
                m.hunt_used = false;
                if !same_visit && m.cells[m.warden as usize].kind == CellKind::Inmate {
                    // On the first decision upon entering an Inmate cell, Replace is always offered by game rules.
                    m.no_swap = false;
                    if !dec.replace || !dec.end {
                        notes.push(
                            "Today's decision cards were still appearing: check them.".to_string(),
                        );
                    }
                } else if !dec.end {
                    // Mid-animation frame where "End" hasn't faded in yet: preserve prior no_swap state and re-probe.
                    m.no_swap = base.no_swap;
                    notes.push(
                        "Today's decision cards were still appearing: check them.".to_string(),
                    );
                } else {
                    m.no_swap = !dec.replace;
                }
                m.no_devour = !dec.consume;
                m.consume_offered = dec.consume;
            }
        } else if m.warden == OFFICE && m.step == BoardStep::Manage {
            let already_acted_in_office =
                m.day == base.day && base.warden == OFFICE && base.step == BoardStep::Manage;
            if !already_acted_in_office {
                m.step = BoardStep::Office;
                m.clear_dest();
            }
        }

        let mut tools = self.run_tools(&r.tools, base, &mut m, &mut notes, r.loc.map(|l| l.s));
        if r.decision.is_some_and(|d| d.hunt)
            && !tools.iter().any(|t| t.sim == ToolId::Trap)
            && tools.len() < 6
        {
            tools.push(ScannedTool {
                id: ToolId::Trap.as_str(),
                sim: ToolId::Trap,
                name: ToolId::Trap.name(),
                score: 1.0,
                conf: 1.0,
            });
        }
        m.tools = tools.iter().map(|t| t.sim).collect();
        if tools.iter().any(|t| (t.conf as f64) < TOOL_SURE) {
            notes.push(
                "Some instruments were hard to tell apart: check them under Torture Instrument."
                    .to_string(),
            );
        }

        let same_run = !(m.day < base.day);
        let chromatic = r
            .chromatic
            .unwrap_or_else(|| if same_run { base.vault.len() } else { 0 });
        m.vault = FixedVec::from_elem(Beast::Wolf, chromatic);
        let highest = m
            .cells
            .iter()
            .filter(|c| c.kind == CellKind::Inmate)
            .map(|c| c.tier)
            .max()
            .unwrap_or(0);
        if m.objective == ObjectiveId::King {
            m.milestone = highest >= SUPREME || chromatic > 0;
        }

        let has_target = |state: &tartarus_core::SimState, n: usize| -> bool {
            !state.cells[n].sealed
                && state.cells[n].inmate.is_some()
                && state.devour_options(n as u8).1 > 0
        };

        let mut trial_final = m;
        trial_final.step = BoardStep::Manage;
        let (mut final_st, _) = trial_final.build_state(1);

        let mut ghosts = [0usize; 16];
        let mut glen = 0usize;
        for (n, c) in cells.iter().enumerate() {
            if c.ghost && m.cells[n].kind == CellKind::Inmate {
                ghosts[glen] = n;
                glen += 1;
            }
        }

        if (1..=3).contains(&glen) {
            let combos = 1usize << (2 * glen);
            let mut best_combo: Option<([Beast; 3], usize, usize)> = None;
            for mask in 0..combos {
                let mut assign = [Beast::Mammoth; 3];
                let mut kept_orig = 0usize;
                for i in 0..glen {
                    let g = ghosts[i];
                    let b = BEASTS[(mask >> (2 * i)) & 3];
                    assign[i] = b;
                    if b == cells[g].beast {
                        kept_orig += 1;
                    }
                    if let Some(inm) = final_st.cells[g].inmate.as_mut() {
                        inm.beast = b;
                    }
                }
                let fit = cells
                    .iter()
                    .enumerate()
                    .filter(|&(k, d)| {
                        d.kind == ScannedCellKind::Inmate && has_target(&final_st, k) == d.jaws
                    })
                    .count();
                if best_combo.map_or(true, |(_, b_fit, b_orig)| {
                    fit > b_fit || (fit == b_fit && kept_orig > b_orig)
                }) {
                    best_combo = Some((assign, fit, kept_orig));
                }
            }
            if let Some((assign, _, _)) = best_combo {
                for i in 0..glen {
                    let g = ghosts[i];
                    m.cells[g].beast = assign[i];
                    if let Some(inm) = final_st.cells[g].inmate.as_mut() {
                        inm.beast = assign[i];
                    }
                }
            }
        } else if glen > 3 {
            for &n in &ghosts[..glen] {
                let mut best: Option<(Beast, usize)> = None;
                for beast in BEASTS {
                    if let Some(inm) = final_st.cells[n].inmate.as_mut() {
                        inm.beast = beast;
                    }
                    let fit = cells
                        .iter()
                        .enumerate()
                        .filter(|&(k, d)| {
                            d.kind == ScannedCellKind::Inmate && has_target(&final_st, k) == d.jaws
                        })
                        .count();
                    if best.map_or(true, |b| fit > b.1) {
                        best = Some((beast, fit));
                    }
                }
                if let Some((best_beast, _)) = best {
                    m.cells[n].beast = best_beast;
                    if let Some(inm) = final_st.cells[n].inmate.as_mut() {
                        inm.beast = best_beast;
                    }
                }
            }
        }

        let mut odd = 0usize;
        for (n, c) in cells.iter().enumerate() {
            if c.kind != ScannedCellKind::Inmate || m.cells[n].kind != CellKind::Inmate {
                continue;
            }
            if c.tier < SUPREME && has_target(&final_st, n) != c.jaws {
                checks |= 1u16 << n;
                odd += 1;
            }
        }
        if odd > 0 {
            notes.push(if odd == 1 {
                "1 inmate's jaws marker doesn't fit what was read: check it and the inmates around."
                    .to_string()
            } else {
                format!("{odd} inmates' jaws markers don't fit what was read: check them and the inmates around.")
            });
        }

        ScanReading {
            model: m,
            checks,
            notes,
            chromatic,
            tools,
            chain_targets: None,
        }
    }
}

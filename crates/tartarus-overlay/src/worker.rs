use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use tartarus_core::AdviceSnapshot;
use tartarus_scan::{CaptureTrigger, ImageView, RunTracker, ScanArena, ScanProfile};

use crate::capture::{GameWindowPlacement, StillnessTracker, WindowCapturer};
use crate::log_watcher::{LogEvent, LogWatcher};
use crate::state::{OverlaySnapshot, UiToWorker, WorkerToUi, read_process_rss_mb};

const PLAYOUT_BUDGET: usize = 96_000;
const SOLVE_BUDGET_MS: u64 = 85;
const PROBE_INTERVAL_MS: u64 = 40;
const IDLE_POLL_MS: u64 = 50;
const WIN_POLL_INTERVAL_MS: u64 = 1500;
const UI_CMDS_PER_STEP_MAX: usize = 256;

const _: () = {
    assert!(PLAYOUT_BUDGET > 0);
    assert!(SOLVE_BUDGET_MS > 0);
    assert!(PROBE_INTERVAL_MS > 0);
    assert!(IDLE_POLL_MS >= PROBE_INTERVAL_MS);
};

#[derive(Clone, Copy, Debug)]
struct PendingBurst {
    t0: Instant,
    last_probe: Option<Instant>,
    settle_ms: u64,
    earliest_ms: u64,
    deadline_ms: u64,
    kind: CaptureTrigger,
}

fn schedule_burst(
    pending_capture: &mut Option<PendingBurst>,
    stillness: &mut StillnessTracker,
    new_kind: CaptureTrigger,
    base_settle_ms: u64,
    pace: f64,
) {
    assert!(pace > 0.0 && pace.is_finite());
    if pending_capture
        .as_ref()
        .is_some_and(|cur| cur.kind == CaptureTrigger::Manual)
    {
        return;
    }
    stillness.reset();
    let now = Instant::now();
    let settle = (((base_settle_ms as f64) * pace) as u64).max(200);
    let earliest = 250_u64.max((400.0 * pace) as u64).min(settle);
    let extra = (1000.0 * pace) as u64;
    assert!(earliest <= settle);
    if let Some(cur) = pending_capture {
        cur.t0 = now;
        cur.last_probe = None;
        cur.settle_ms = cur.settle_ms.max(settle);
        cur.earliest_ms = cur.earliest_ms.max(earliest);
        cur.deadline_ms = cur.settle_ms + extra;
        if new_kind.rank() > cur.kind.rank() {
            cur.kind = new_kind;
        } else if let (
            CaptureTrigger::Instruments { fresh: cur_fresh },
            CaptureTrigger::Instruments { fresh: new_fresh },
        ) = (cur.kind, new_kind)
        {
            cur.kind = CaptureTrigger::Instruments {
                fresh: cur_fresh || new_fresh,
            };
        }
    } else {
        *pending_capture = Some(PendingBurst {
            t0: now,
            last_probe: None,
            settle_ms: settle,
            earliest_ms: earliest,
            deadline_ms: settle + extra,
            kind: new_kind,
        });
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SnapshotMetrics {
    riot: bool,
    game_connected: bool,
    log_connected: bool,
    cap_ms: f64,
    scan_ms: f64,
    scan_profile: ScanProfile,
    solve_ms: f64,
}

fn send_snapshot(
    worker_tx: &Sender<WorkerToUi>,
    tracker: &RunTracker,
    advice: AdviceSnapshot,
    metrics: SnapshotMetrics,
    status_text: String,
) {
    assert!(!status_text.is_empty());
    assert!(metrics.cap_ms >= 0.0 && metrics.scan_ms >= 0.0 && metrics.solve_ms >= 0.0);
    let _ = worker_tx.send(WorkerToUi::Snapshot(Box::new(OverlaySnapshot {
        model: tracker.model,
        advice,
        phase: tracker.phase,
        riot: metrics.riot,
        notes: tracker.last_notes.clone(),
        game_connected: metrics.game_connected,
        log_connected: metrics.log_connected,
        cap_ms: metrics.cap_ms,
        scan_ms: metrics.scan_ms,
        scan_profile: metrics.scan_profile,
        solve_ms: metrics.solve_ms,
        rss_mb: read_process_rss_mb(),
        status_text,
    })));
}

#[derive(Default)]
struct StepFlags {
    recheck_only: bool,
    state_push_only: bool,
    status_override: Option<String>,
}

struct WorkerRuntime {
    worker_tx: Sender<WorkerToUi>,
    capturer: WindowCapturer,
    arena: ScanArena,
    log_watcher: LogWatcher,
    tracker: RunTracker,
    stillness: StillnessTracker,
    riot: bool,
    speed_on: bool,
    pending_capture: Option<PendingBurst>,
    queued_cmd: Option<UiToWorker>,
    last_win_poll: Instant,
    last_placement: Option<GameWindowPlacement>,
    game_connected: bool,
    seed_counter: u32,
    last_advice: AdviceSnapshot,
}

struct SettledScanOutcome {
    saved_tracker: RunTracker,
    recognized: bool,
    custom_status: Option<String>,
    metrics: SnapshotMetrics,
}

impl WorkerRuntime {
    fn new(worker_tx: Sender<WorkerToUi>) -> Self {
        let capturer = WindowCapturer::new_with_buffer();
        let arena = ScanArena::new();
        let mut log_watcher = LogWatcher::new();
        let mut tracker = RunTracker::new();
        let mut riot = false;

        for &ev in log_watcher.poll() {
            match ev {
                LogEvent::Riot(on) => {
                    riot = on;
                    let _ = worker_tx.send(WorkerToUi::RiotState(on));
                }
                LogEvent::RunOver => {
                    tracker.on_run_over();
                }
                LogEvent::RunNew => {
                    tracker.on_run_new();
                }
                LogEvent::RunBegin { run_id, is_settled } => {
                    tracker.on_log_run_begin(run_id, is_settled);
                }
                LogEvent::DayCheck { cur, max } => {
                    tracker.on_log_day_check(cur, max);
                }
                LogEvent::RelicPopState(open) => {
                    tracker.log_relic_pop_open = Some(open);
                }
                _ => {}
            }
        }

        let rt = Self {
            worker_tx,
            capturer,
            arena,
            log_watcher,
            tracker,
            stillness: StillnessTracker::new(),
            riot,
            speed_on: false,
            pending_capture: None,
            queued_cmd: None,
            last_win_poll: Instant::now(),
            last_placement: None,
            game_connected: false,
            seed_counter: 0x9E37_79B9,
            last_advice: AdviceSnapshot::default(),
        };
        assert_ne!(rt.seed_counter, 0);
        assert!(rt.pending_capture.is_none());
        rt
    }

    fn next_seed(&mut self) -> u32 {
        self.seed_counter = self.seed_counter.wrapping_add(0x9E37_79B9);
        let seed = if self.seed_counter == 0 {
            1
        } else {
            self.seed_counter
        };
        assert_ne!(seed, 0);
        assert!(self.seed_counter != 0 || seed == 1);
        seed
    }

    fn clear_non_manual_pending(&mut self) {
        if self
            .pending_capture
            .as_ref()
            .is_none_or(|b| b.kind != CaptureTrigger::Manual)
        {
            self.pending_capture = None;
        }
        assert!(
            self.pending_capture
                .as_ref()
                .is_none_or(|b| b.kind == CaptureTrigger::Manual)
        );
        assert_ne!(self.seed_counter, 0);
    }

    fn handle_single_log_event(&mut self, ev: LogEvent, pace: f64, step: &mut StepFlags) {
        assert!(pace > 0.0 && pace.is_finite());
        assert_ne!(self.seed_counter, 0);
        match ev {
            LogEvent::Riot(on) => {
                if self.riot != on {
                    self.riot = on;
                    let _ = self.worker_tx.send(WorkerToUi::RiotState(on));
                    step.state_push_only = true;
                }
            }
            LogEvent::RunOver => {
                self.clear_non_manual_pending();
                if self.tracker.on_run_over() {
                    self.last_advice = AdviceSnapshot::default();
                    step.status_override = Some("Run over · Ready for the next run".into());
                    step.state_push_only = true;
                }
            }
            LogEvent::RunNew => {
                if self.tracker.on_run_new() {
                    step.state_push_only = true;
                }
            }
            LogEvent::RunBegin { run_id, is_settled } => {
                if self.tracker.on_log_run_begin(run_id, is_settled) {
                    if is_settled {
                        self.last_advice = AdviceSnapshot::default();
                        step.status_override = Some("Run over · Ready for the next run".into());
                    }
                    step.state_push_only = true;
                }
            }
            LogEvent::DayCheck { cur, max } => {
                if self.tracker.on_log_day_check(cur, max) {
                    if max == 0 {
                        self.last_advice = AdviceSnapshot::default();
                        step.status_override = Some("Run over · Ready for the next run".into());
                        step.state_push_only = true;
                    } else if self.tracker.phase.is_active() {
                        step.recheck_only = true;
                    } else {
                        step.state_push_only = true;
                    }
                }
            }
            LogEvent::RelicPopState(open) => {
                self.tracker.log_relic_pop_open = Some(open);
            }
            LogEvent::SpeedFlip => {
                self.speed_on = !self.speed_on;
            }
            other => {
                self.handle_log_trigger_event(other, pace, step);
            }
        }
    }

    fn handle_log_trigger_event(&mut self, ev: LogEvent, pace: f64, step: &mut StepFlags) {
        assert!(pace > 0.0 && pace.is_finite());
        assert_ne!(self.seed_counter, 0);
        match ev {
            LogEvent::SkipPending => {
                self.clear_non_manual_pending();
            }
            LogEvent::TreasurePopup { after } => {
                self.clear_non_manual_pending();
                let msg = self.tracker.use_treasure(after);
                step.recheck_only = true;
                step.status_override = Some(msg);
            }
            LogEvent::ObjectivePopup => {
                schedule_burst(
                    &mut self.pending_capture,
                    &mut self.stillness,
                    CaptureTrigger::Objective,
                    1000,
                    pace,
                );
            }
            LogEvent::InstrumentsPopup { fresh } => {
                schedule_burst(
                    &mut self.pending_capture,
                    &mut self.stillness,
                    CaptureTrigger::Instruments { fresh },
                    1000,
                    pace,
                );
            }
            LogEvent::BoardTrigger {
                settle_ms,
                treasure_view,
            } => {
                schedule_burst(
                    &mut self.pending_capture,
                    &mut self.stillness,
                    CaptureTrigger::Board { treasure_view },
                    settle_ms,
                    pace,
                );
            }
            _ => {}
        }
    }

    fn poll_log_events(&mut self, step: &mut StepFlags) {
        let pace = if self.speed_on { 0.45 } else { 1.0 };
        assert!(pace > 0.0);
        let events: Vec<LogEvent> = self.log_watcher.poll().to_vec();
        for ev in events {
            self.handle_single_log_event(ev, pace, step);
        }
        assert_ne!(self.seed_counter, 0);
    }

    fn handle_ui_cmd(&mut self, cmd: UiToWorker, step: &mut StepFlags) {
        assert_ne!(self.seed_counter, 0);
        match cmd {
            UiToWorker::CaptureNow => {
                self.stillness.reset();
                let now = Instant::now();
                self.pending_capture = Some(PendingBurst {
                    t0: now,
                    last_probe: None,
                    settle_ms: 0,
                    earliest_ms: 0,
                    deadline_ms: 200,
                    kind: CaptureTrigger::Manual,
                });
                assert!(self.pending_capture.is_some());
            }
            UiToWorker::ConfirmTools => {
                self.tracker.confirm_tools();
                step.recheck_only = self.tracker.phase.is_active();
                step.status_override = Some("Instruments confirmed".into());
                assert!(step.status_override.is_some());
            }
            UiToWorker::AckTreasureNow => {
                self.tracker.acknowledge_treasure_now();
                step.recheck_only = true;
                step.status_override = Some("Advising Warden's Treasure on current board".into());
                assert!(step.recheck_only);
            }
            UiToWorker::UpdateModelAndRecheck(new_model) => {
                self.tracker.set_model(*new_model);
                step.recheck_only = true;
                step.status_override = Some("Rechecked edited board".into());
                assert!(step.recheck_only);
            }
            UiToWorker::LoadExample => {
                self.tracker.load_example();
                step.recheck_only = true;
                step.status_override = Some("Loaded example board (Day 6)".into());
                assert!(step.recheck_only);
            }
        }
    }

    fn drain_ui_commands(&mut self, worker_rx: &Receiver<UiToWorker>, step: &mut StepFlags) {
        assert_ne!(self.seed_counter, 0);
        if let Some(cmd) = self.queued_cmd.take() {
            self.handle_ui_cmd(cmd, step);
        }
        let mut cmd_iters = 0usize;
        while cmd_iters < UI_CMDS_PER_STEP_MAX {
            let Ok(cmd) = worker_rx.try_recv() else {
                break;
            };
            cmd_iters += 1;
            self.handle_ui_cmd(cmd, step);
        }
        assert!(cmd_iters <= UI_CMDS_PER_STEP_MAX);
    }

    fn poll_window_placement(&mut self, step: &mut StepFlags) {
        assert_ne!(self.seed_counter, 0);
        if self.last_win_poll.elapsed() < Duration::from_millis(WIN_POLL_INTERVAL_MS) {
            return;
        }
        self.last_win_poll = Instant::now();
        let cur_placement = self.capturer.game_window_placement();
        let now_conn = cur_placement.is_some() || self.capturer.has_game_window();
        if now_conn != self.game_connected {
            self.game_connected = now_conn;
            step.state_push_only = true;
        }
        if let Some(p) = cur_placement {
            assert!(p.width > 0 && p.height > 0);
            let monitor_changed = self.last_placement.as_ref().is_none_or(|prev| {
                prev.monitor_index != p.monitor_index
                    || prev.monitor_spatial_rank != p.monitor_spatial_rank
                    || prev.monitor != p.monitor
            });
            if monitor_changed {
                let _ = self.worker_tx.send(WorkerToUi::GamePlacement(p.clone()));
            }
            self.last_placement = Some(p);
        }
    }

    fn flush_step_updates(&mut self, step: StepFlags) {
        assert_ne!(self.seed_counter, 0);
        if step.recheck_only && self.tracker.phase.is_active() {
            let t0 = Instant::now();
            let seed = self.next_seed();
            let advice = self
                .tracker
                .model
                .evaluate(PLAYOUT_BUDGET, SOLVE_BUDGET_MS, seed);
            self.tracker.record_advice(advice.best_action);
            self.last_advice = advice.clone();
            let solve_ms = t0.elapsed().as_secs_f64() * 1000.0;
            assert!(solve_ms >= 0.0);
            send_snapshot(
                &self.worker_tx,
                &self.tracker,
                advice,
                SnapshotMetrics {
                    riot: self.riot,
                    game_connected: self.game_connected,
                    log_connected: self.log_watcher.is_connected(),
                    solve_ms,
                    ..SnapshotMetrics::default()
                },
                step.status_override
                    .unwrap_or_else(|| format!("Solved in {solve_ms:.1}ms")),
            );
        } else if step.state_push_only {
            let advice = if self.tracker.phase.is_active() {
                self.last_advice.clone()
            } else {
                AdviceSnapshot::default()
            };
            let status_text = step.status_override.unwrap_or_else(|| {
                if self.tracker.phase.is_run_over() {
                    "Run over · Ready for the next run".into()
                } else {
                    "Waiting for the next move".into()
                }
            });
            assert!(!status_text.is_empty());
            send_snapshot(
                &self.worker_tx,
                &self.tracker,
                advice,
                SnapshotMetrics {
                    riot: self.riot,
                    game_connected: self.game_connected,
                    log_connected: self.log_watcher.is_connected(),
                    ..SnapshotMetrics::default()
                },
                status_text,
            );
        }
    }

    fn finalize_recognized_scan(
        &mut self,
        recognized: bool,
        is_manual: bool,
        custom_status: Option<String>,
        metrics: SnapshotMetrics,
    ) {
        assert!(metrics.scan_ms >= 0.0);
        self.pending_capture = None;
        let (advice, solve_ms) = if recognized && self.tracker.phase.is_active() {
            let t_solve = Instant::now();
            let seed = self.next_seed();
            let a = self
                .tracker
                .model
                .evaluate(PLAYOUT_BUDGET, SOLVE_BUDGET_MS, seed);
            self.tracker.record_advice(a.best_action);
            self.last_advice = a.clone();
            (a, t_solve.elapsed().as_secs_f64() * 1000.0)
        } else if self.tracker.phase.is_active() {
            (self.last_advice.clone(), 0.0)
        } else {
            (AdviceSnapshot::default(), 0.0)
        };
        assert!(solve_ms >= 0.0);

        if recognized || is_manual || self.tracker.phase.is_active() {
            let scan_ms = metrics.scan_ms;
            let status_text = custom_status.unwrap_or_else(|| {
                if !recognized {
                    if self.tracker.phase.is_active() {
                        format!("No board in picture ({scan_ms:.2}ms) · Showing previous board")
                    } else {
                        format!("No board in picture ({scan_ms:.2}ms)")
                    }
                } else {
                    format!("Scan {scan_ms:.2}ms · Solve {solve_ms:.1}ms")
                }
            });

            send_snapshot(
                &self.worker_tx,
                &self.tracker,
                advice,
                SnapshotMetrics {
                    solve_ms,
                    ..metrics
                },
                status_text,
            );
        }
    }

    fn capture_and_scan_if_settled(
        &mut self,
        burst: PendingBurst,
        elapsed_ms: u64,
        is_manual: bool,
    ) -> Option<Result<SettledScanOutcome, ()>> {
        let t_cap = Instant::now();
        let Some(frame) = self.capturer.capture() else {
            return Some(Err(()));
        };
        assert!(frame.width > 0 && frame.height > 0);
        self.game_connected = true;
        self.stillness.push(&frame);
        let cap_ms = t_cap.elapsed().as_secs_f64() * 1000.0;
        assert!(cap_ms >= 0.0);
        let settled_early = elapsed_ms >= burst.earliest_ms && self.stillness.settled();
        let settle_done = elapsed_ms >= burst.settle_ms
            && (!self.stillness.moving() || elapsed_ms >= burst.deadline_ms);
        if !(is_manual || settled_early || settle_done) {
            return None;
        }

        let t_scan = Instant::now();
        let view = ImageView {
            width: frame.width,
            height: frame.height,
            data: frame.rgba,
        };
        let saved_tracker = self.tracker.clone();
        let (recognized, custom_status, scan_profile) =
            self.tracker
                .process_frame(&view, burst.kind, &mut self.arena);
        let scan_ms = t_scan.elapsed().as_secs_f64() * 1000.0;
        let metrics = SnapshotMetrics {
            riot: self.riot,
            game_connected: true,
            log_connected: self.log_watcher.is_connected(),
            cap_ms,
            scan_ms,
            scan_profile,
            solve_ms: 0.0,
        };
        Some(Ok(SettledScanOutcome {
            saved_tracker,
            recognized,
            custom_status,
            metrics,
        }))
    }

    fn probe_pending_capture(&mut self) {
        let Some(burst) = self.pending_capture else {
            return;
        };
        let now = Instant::now();
        let elapsed_ms = now.duration_since(burst.t0).as_millis() as u64;
        let is_manual = burst.kind == CaptureTrigger::Manual;
        let due_probe = is_manual
            || burst
                .last_probe
                .map_or(elapsed_ms >= PROBE_INTERVAL_MS, |lp| {
                    now.duration_since(lp) >= Duration::from_millis(PROBE_INTERVAL_MS)
                })
            || elapsed_ms >= burst.settle_ms;
        if !due_probe {
            return;
        }

        if let Some(mut b) = self.pending_capture {
            b.last_probe = Some(now);
            self.pending_capture = Some(b);
        }
        assert!(self.pending_capture.is_some());
        assert_ne!(self.seed_counter, 0);

        match self.capture_and_scan_if_settled(burst, elapsed_ms, is_manual) {
            Some(Ok(outcome)) => {
                self.handle_scan_outcome(burst, elapsed_ms, outcome);
            }
            Some(Err(())) if is_manual || elapsed_ms >= burst.deadline_ms => {
                self.handle_missing_window();
            }
            _ => {}
        }
    }

    fn handle_scan_outcome(
        &mut self,
        burst: PendingBurst,
        elapsed_ms: u64,
        outcome: SettledScanOutcome,
    ) {
        assert!(outcome.metrics.cap_ms >= 0.0 && outcome.metrics.scan_ms >= 0.0);
        let is_manual = burst.kind == CaptureTrigger::Manual;
        let missing_cards = matches!(
            burst.kind,
            CaptureTrigger::Board {
                treasure_view: false
            }
        ) && !self.tracker.last_board_had_cards;
        let has_checks =
            self.tracker.model.checks != 0 || !self.tracker.last_notes.is_empty() || missing_cards;
        let keep_trying =
            (!outcome.recognized || has_checks) && !is_manual && elapsed_ms < burst.deadline_ms;

        if keep_trying && outcome.recognized {
            let hint = self.tracker.hint;
            self.tracker = outcome.saved_tracker;
            self.tracker.hint = hint;
        }

        assert!(!is_manual || !keep_trying);
        if !keep_trying {
            self.finalize_recognized_scan(
                outcome.recognized,
                is_manual,
                outcome.custom_status,
                outcome.metrics,
            );
        }
    }

    fn handle_missing_window(&mut self) {
        self.pending_capture = None;
        self.game_connected = false;
        assert!(!self.game_connected);
        assert!(self.pending_capture.is_none());
        let advice = if self.tracker.phase.is_active() {
            self.last_advice.clone()
        } else {
            AdviceSnapshot::default()
        };
        send_snapshot(
            &self.worker_tx,
            &self.tracker,
            advice,
            SnapshotMetrics {
                riot: self.riot,
                game_connected: false,
                log_connected: self.log_watcher.is_connected(),
                ..SnapshotMetrics::default()
            },
            "Torchlight: Infinite window not found".into(),
        );
    }

    fn run_loop(&mut self, worker_rx: Receiver<UiToWorker>) {
        self.last_placement = self.capturer.game_window_placement();
        self.game_connected = self.last_placement.is_some() || self.capturer.has_game_window();
        if let Some(ref p) = self.last_placement {
            let _ = self.worker_tx.send(WorkerToUi::GamePlacement(p.clone()));
        }

        let status_text: String = if self.tracker.phase.is_run_over() {
            "Run over · Ready for the next run".into()
        } else {
            "Ready · Waiting for the next move or press Picture".into()
        };
        assert!(!status_text.is_empty());
        assert_ne!(self.seed_counter, 0);
        send_snapshot(
            &self.worker_tx,
            &self.tracker,
            self.last_advice.clone(),
            SnapshotMetrics {
                riot: self.riot,
                game_connected: self.game_connected,
                log_connected: self.log_watcher.is_connected(),
                ..SnapshotMetrics::default()
            },
            status_text,
        );

        loop {
            let mut step = StepFlags::default();
            self.poll_log_events(&mut step);
            self.drain_ui_commands(&worker_rx, &mut step);
            self.poll_window_placement(&mut step);
            self.flush_step_updates(step);
            self.probe_pending_capture();

            let wait_ms = if self.pending_capture.is_some() {
                PROBE_INTERVAL_MS / 2
            } else {
                IDLE_POLL_MS
            };
            if let Ok(cmd) = worker_rx.recv_timeout(Duration::from_millis(wait_ms)) {
                self.queued_cmd = Some(cmd);
            }
        }
    }
}

pub fn spawn_worker() -> (Sender<UiToWorker>, Receiver<WorkerToUi>) {
    let (ui_tx, worker_rx) = mpsc::channel::<UiToWorker>();
    let (worker_tx, ui_rx) = mpsc::channel::<WorkerToUi>();

    let mut runtime = WorkerRuntime::new(worker_tx);
    assert_ne!(runtime.seed_counter, 0);
    assert!(runtime.pending_capture.is_none());

    std::thread::Builder::new()
        .name("tartarus-worker".into())
        .spawn(move || {
            runtime.run_loop(worker_rx);
        })
        .expect("Failed to spawn worker thread");

    (ui_tx, ui_rx)
}

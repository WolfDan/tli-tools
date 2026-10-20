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

fn send_snapshot(
    worker_tx: &Sender<WorkerToUi>,
    tracker: &RunTracker,
    advice: AdviceSnapshot,
    riot: bool,
    game_connected: bool,
    log_connected: bool,
    cap_ms: f64,
    scan_ms: f64,
    scan_profile: ScanProfile,
    solve_ms: f64,
    status_text: String,
) {
    let _ = worker_tx.send(WorkerToUi::Snapshot(OverlaySnapshot {
        model: tracker.model,
        advice,
        phase: tracker.phase,
        riot,
        notes: tracker.last_notes.clone(),
        game_connected,
        log_connected,
        cap_ms,
        scan_ms,
        scan_profile,
        solve_ms,
        rss_mb: read_process_rss_mb(),
        status_text,
    }));
}

pub fn spawn_worker() -> (Sender<UiToWorker>, Receiver<WorkerToUi>) {
    let (ui_tx, worker_rx) = mpsc::channel::<UiToWorker>();
    let (worker_tx, ui_rx) = mpsc::channel::<WorkerToUi>();

    let mut capturer = WindowCapturer::new_with_buffer();
    let mut arena = ScanArena::new();
    let mut log_watcher = LogWatcher::new();
    let mut tracker = RunTracker::new();
    let mut riot = false;

    // Read initial state flags from the tail of UE_game.log before opening the window
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

    std::thread::Builder::new()
        .name("tartarus-worker".into())
        .spawn(move || {
            let mut stillness = StillnessTracker::new();
            let mut speed_on = false;
            let mut pending_capture: Option<PendingBurst> = None;
            let mut queued_cmd: Option<UiToWorker> = None;
            let mut last_win_poll = Instant::now();
            let mut last_placement: Option<GameWindowPlacement> = capturer.game_window_placement();
            let mut game_connected = last_placement.is_some() || capturer.has_game_window();
            let mut seed_counter: u32 = 0x9E37_79B9;
            let mut last_advice = AdviceSnapshot::default();

            if let Some(ref p) = last_placement {
                let _ = worker_tx.send(WorkerToUi::GamePlacement(p.clone()));
            }

            let mut next_seed = || -> u32 {
                seed_counter = seed_counter.wrapping_add(0x9E37_79B9);
                if seed_counter == 0 {
                    1
                } else {
                    seed_counter
                }
            };

            {
                let status_text = if tracker.phase.is_run_over() {
                    "Run over · Ready for the next run".into()
                } else {
                    "Ready · Waiting for the next move or press Picture".into()
                };
                send_snapshot(
                    &worker_tx,
                    &tracker,
                    last_advice.clone(),
                    riot,
                    game_connected,
                    log_watcher.is_connected(),
                    0.0,
                    0.0,
                    ScanProfile::default(),
                    0.0,
                    status_text,
                );
            }

            loop {
                let mut recheck_only = false;
                let mut state_push_only = false;
                let mut status_override: Option<String> = None;

                // 1. Poll UE_game.log on every wakeup (fast stat check on persistent File handle)
                let pace = if speed_on { 0.45 } else { 1.0 };
                for &ev in log_watcher.poll() {
                    match ev {
                        LogEvent::Riot(on) => {
                            if riot != on {
                                riot = on;
                                let _ = worker_tx.send(WorkerToUi::RiotState(on));
                                state_push_only = true;
                            }
                        }
                        LogEvent::RunOver => {
                            if pending_capture
                                .as_ref()
                                .is_none_or(|b| b.kind != CaptureTrigger::Manual)
                            {
                                pending_capture = None;
                            }
                            if tracker.on_run_over() {
                                last_advice = AdviceSnapshot::default();
                                status_override =
                                    Some("Run over · Ready for the next run".into());
                                state_push_only = true;
                            }
                        }
                        LogEvent::RunNew => {
                            if tracker.on_run_new() {
                                state_push_only = true;
                            }
                        }
                        LogEvent::RunBegin { run_id, is_settled } => {
                            if tracker.on_log_run_begin(run_id, is_settled) {
                                if is_settled {
                                    last_advice = AdviceSnapshot::default();
                                    status_override =
                                        Some("Run over · Ready for the next run".into());
                                }
                                state_push_only = true;
                            }
                        }
                        LogEvent::DayCheck { cur, max } => {
                            if tracker.on_log_day_check(cur, max) {
                                if max == 0 {
                                    last_advice = AdviceSnapshot::default();
                                    status_override =
                                        Some("Run over · Ready for the next run".into());
                                    state_push_only = true;
                                } else if tracker.phase.is_active() {
                                    recheck_only = true;
                                } else {
                                    state_push_only = true;
                                }
                            }
                        }
                        LogEvent::RelicPopState(open) => {
                            tracker.log_relic_pop_open = Some(open);
                        }
                        LogEvent::SpeedFlip => {
                            speed_on = !speed_on;
                        }
                        LogEvent::SkipPending => {
                            if pending_capture
                                .as_ref()
                                .is_none_or(|b| b.kind != CaptureTrigger::Manual)
                            {
                                pending_capture = None;
                            }
                        }
                        LogEvent::TreasurePopup { after } => {
                            if pending_capture
                                .as_ref()
                                .is_none_or(|b| b.kind != CaptureTrigger::Manual)
                            {
                                pending_capture = None;
                            }
                            let msg = tracker.use_treasure(after);
                            recheck_only = true;
                            status_override = Some(msg);
                        }
                        LogEvent::ObjectivePopup => {
                            schedule_burst(
                                &mut pending_capture,
                                &mut stillness,
                                CaptureTrigger::Objective,
                                1000,
                                pace,
                            );
                        }
                        LogEvent::InstrumentsPopup { fresh } => {
                            schedule_burst(
                                &mut pending_capture,
                                &mut stillness,
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
                                &mut pending_capture,
                                &mut stillness,
                                CaptureTrigger::Board { treasure_view },
                                settle_ms,
                                pace,
                            );
                        }
                    }
                }

                // 2. Handle UI commands (woken immediately in 0ms by recv_timeout)
                let mut handle_cmd = |cmd: UiToWorker| match cmd {
                    UiToWorker::CaptureNow => {
                        stillness.reset();
                        let now = Instant::now();
                        pending_capture = Some(PendingBurst {
                            t0: now,
                            last_probe: None,
                            settle_ms: 0,
                            earliest_ms: 0,
                            deadline_ms: 200,
                            kind: CaptureTrigger::Manual,
                        });
                    }
                    UiToWorker::ConfirmTools => {
                        tracker.confirm_tools();
                        recheck_only = tracker.phase.is_active();
                        status_override = Some("Instruments confirmed".into());
                    }
                    UiToWorker::AckTreasureNow => {
                        tracker.acknowledge_treasure_now();
                        recheck_only = true;
                        status_override =
                            Some("Advising Warden's Treasure on current board".into());
                    }
                    UiToWorker::UpdateModelAndRecheck(new_model) => {
                        tracker.set_model(new_model);
                        recheck_only = true;
                        status_override = Some("Rechecked edited board".into());
                    }
                    UiToWorker::LoadExample => {
                        tracker.load_example();
                        recheck_only = true;
                        status_override = Some("Loaded example board (Day 6)".into());
                    }
                };

                if let Some(cmd) = queued_cmd.take() {
                    handle_cmd(cmd);
                }
                while let Ok(cmd) = worker_rx.try_recv() {
                    handle_cmd(cmd);
                }

                // Periodically refresh game window connection status and monitor placement when idle
                if last_win_poll.elapsed() >= Duration::from_millis(1500) {
                    last_win_poll = Instant::now();
                    let cur_placement = capturer.game_window_placement();
                    let now_conn = cur_placement.is_some() || capturer.has_game_window();
                    if now_conn != game_connected {
                        game_connected = now_conn;
                        state_push_only = true;
                    }
                    if let Some(p) = cur_placement {
                        let monitor_changed = last_placement.as_ref().is_none_or(|prev| {
                            prev.monitor_index != p.monitor_index
                                || prev.monitor_spatial_rank != p.monitor_spatial_rank
                                || prev.monitor != p.monitor
                        });
                        if monitor_changed {
                            let _ = worker_tx.send(WorkerToUi::GamePlacement(p.clone()));
                        }
                        last_placement = Some(p);
                    }
                }

                if recheck_only && tracker.phase.is_active() {
                    let t0 = Instant::now();
                    let advice =
                        tracker.model.evaluate(PLAYOUT_BUDGET, SOLVE_BUDGET_MS, next_seed());
                    tracker.record_advice(advice.best_action);
                    last_advice = advice.clone();
                    let solve_ms = t0.elapsed().as_secs_f64() * 1000.0;
                    send_snapshot(
                        &worker_tx,
                        &tracker,
                        advice,
                        riot,
                        game_connected,
                        log_watcher.is_connected(),
                        0.0,
                        0.0,
                        ScanProfile::default(),
                        solve_ms,
                        status_override
                            .unwrap_or_else(|| format!("Solved in {solve_ms:.1}ms")),
                    );
                } else if state_push_only {
                    let advice = if tracker.phase.is_active() {
                        last_advice.clone()
                    } else {
                        AdviceSnapshot::default()
                    };
                    send_snapshot(
                        &worker_tx,
                        &tracker,
                        advice,
                        riot,
                        game_connected,
                        log_watcher.is_connected(),
                        0.0,
                        0.0,
                        ScanProfile::default(),
                        0.0,
                        status_override.unwrap_or_else(|| {
                            if tracker.phase.is_run_over() {
                                "Run over · Ready for the next run".into()
                            } else {
                                "Waiting for the next move".into()
                            }
                        }),
                    );
                }

                // 3. Probe stillness every 40ms and execute capture when settled; retry until deadline_ms if premature
                if let Some(burst) = pending_capture {
                    let now = Instant::now();
                    let elapsed_ms = now.duration_since(burst.t0).as_millis() as u64;
                    let is_manual = burst.kind == CaptureTrigger::Manual;
                    let due_probe = is_manual
                        || burst.last_probe.map_or(elapsed_ms >= PROBE_INTERVAL_MS, |lp| {
                            now.duration_since(lp) >= Duration::from_millis(PROBE_INTERVAL_MS)
                        })
                        || elapsed_ms >= burst.settle_ms;

                    if due_probe {
                        if let Some(mut b) = pending_capture {
                            b.last_probe = Some(now);
                            pending_capture = Some(b);
                        }
                        let t_cap = Instant::now();
                        if let Some(frame) = capturer.capture() {
                            game_connected = true;
                            stillness.push(&frame);
                            let cap_ms = t_cap.elapsed().as_secs_f64() * 1000.0;
                            let settled_early =
                                elapsed_ms >= burst.earliest_ms && stillness.settled();
                            let settle_done = elapsed_ms >= burst.settle_ms
                                && (!stillness.moving() || elapsed_ms >= burst.deadline_ms);

                            if is_manual || settled_early || settle_done {
                                let t_scan = Instant::now();
                                let view = ImageView {
                                    width: frame.width,
                                    height: frame.height,
                                    data: frame.rgba,
                                };
                                let saved_tracker = tracker.clone();
                                let (recognized, custom_status, scan_profile) =
                                    tracker.process_frame(&view, burst.kind, &mut arena);
                                let scan_ms = t_scan.elapsed().as_secs_f64() * 1000.0;

                                // Self-healing burst: if an automatic trigger scanned early before the
                                // popup/board finished appearing, before the bottom action cards slid up,
                                // or while a cell card is mid-flip, keep probing until deadline_ms!
                                let missing_cards = matches!(
                                    burst.kind,
                                    CaptureTrigger::Board {
                                        treasure_view: false
                                    }
                                ) && !tracker.last_board_had_cards;
                                let has_checks = tracker.model.checks != 0
                                    || !tracker.last_notes.is_empty()
                                    || missing_cards;
                                let keep_trying = (!recognized || has_checks)
                                    && !is_manual
                                    && elapsed_ms < burst.deadline_ms;

                                if keep_trying && recognized {
                                    let hint = tracker.hint;
                                    tracker = saved_tracker;
                                    tracker.hint = hint;
                                }

                                if !keep_trying {
                                    pending_capture = None;
                                    let (advice, solve_ms) = if recognized && tracker.phase.is_active()
                                    {
                                        let t_solve = Instant::now();
                                        let a = tracker.model.evaluate(
                                            PLAYOUT_BUDGET,
                                            SOLVE_BUDGET_MS,
                                            next_seed(),
                                        );
                                        tracker.record_advice(a.best_action);
                                        last_advice = a.clone();
                                        (a, t_solve.elapsed().as_secs_f64() * 1000.0)
                                    } else if tracker.phase.is_active() {
                                        (last_advice.clone(), 0.0)
                                    } else {
                                        (AdviceSnapshot::default(), 0.0)
                                    };

                                    if recognized || is_manual || tracker.phase.is_active() {
                                        let status_text = custom_status.unwrap_or_else(|| {
                                            if !recognized {
                                                if tracker.phase.is_active() {
                                                    format!(
                                                        "No board in picture ({scan_ms:.2}ms) · Showing previous board"
                                                    )
                                                } else {
                                                    format!("No board in picture ({scan_ms:.2}ms)")
                                                }
                                            } else {
                                                format!("Scan {scan_ms:.2}ms · Solve {solve_ms:.1}ms")
                                            }
                                        });

                                        send_snapshot(
                                            &worker_tx,
                                            &tracker,
                                            advice,
                                            riot,
                                            true,
                                            log_watcher.is_connected(),
                                            cap_ms,
                                            scan_ms,
                                            scan_profile,
                                            solve_ms,
                                            status_text,
                                        );
                                    }
                                }
                            }
                        } else if is_manual || elapsed_ms >= burst.deadline_ms {
                            pending_capture = None;
                            game_connected = false;
                            let advice = if tracker.phase.is_active() {
                                last_advice.clone()
                            } else {
                                AdviceSnapshot::default()
                            };
                            send_snapshot(
                                &worker_tx,
                                &tracker,
                                advice,
                                riot,
                                false,
                                log_watcher.is_connected(),
                                0.0,
                                0.0,
                                ScanProfile::default(),
                                0.0,
                                "Torchlight: Infinite window not found".into(),
                            );
                        }
                    }
                }

                let wait_ms = if pending_capture.is_some() {
                    PROBE_INTERVAL_MS / 2
                } else {
                    IDLE_POLL_MS
                };
                if let Ok(cmd) = worker_rx.recv_timeout(Duration::from_millis(wait_ms)) {
                    queued_cmd = Some(cmd);
                }
            }
        })
        .expect("Failed to spawn worker thread");

    (ui_tx, ui_rx)
}

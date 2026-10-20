use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui::{
    Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point, Render,
    SharedString, Styled, Window, div, px, rgb, rgba,
};
use tartarus_core::{
    ALL_TOOLS, BEASTS, BoardStep, CHALLENGE_DAYS, CellEvent, CellKind, CellModel, FixedVec,
    GameMode, OFFICE, RunPhase, cell_label, tier_name,
};
use tartarus_scan::RING;

use crate::state::{
    DisplayTarget, OverlaySnapshot, OverlayWindowState, PanelLayout, SharedWorkerChannels,
    UiToWorker, WorkerToUi, read_process_rss_mb,
};
use crate::tray::ControlCommand;

pub struct OverlayView {
    pub worker_tx: Sender<UiToWorker>,
    pub shared_rx: Arc<Mutex<SharedWorkerChannels>>,
    pub snap: OverlaySnapshot,
    pub visible: bool,
    pub layout: PanelLayout,
    pub selected_cell: u8,
    pub display_target: DisplayTarget,
    pub is_layer_shell: bool,
    pub margin_top: f32,
    pub margin_left: f32,
    pub pending_reopen: bool,
    pub drag_start: Option<Point<Pixels>>,
}

impl OverlayView {
    pub fn from_state(state: OverlayWindowState, cx: &mut Context<Self>) -> Self {
        let shared_rx = state.shared_rx.clone();

        cx.spawn(async move |this, cx| {
            let mut tick_count: u32 = 0;
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(45))
                    .await;
                tick_count = tick_count.wrapping_add(1);
                let mut changed = false;
                let mut should_quit = false;

                let (cmds, worker_msgs) = {
                    let Ok(guard) = shared_rx.lock() else {
                        continue;
                    };
                    let cmds = guard.control_hub.poll();
                    let mut msgs = Vec::new();
                    while let Ok(msg) = guard.worker_rx.try_recv() {
                        msgs.push(msg);
                    }
                    (cmds, msgs)
                };

                // Refresh RSS right after first-frame GPUI paint (ticks 1..=3) and every ~1s thereafter
                let check_rss = tick_count <= 3 || tick_count % 22 == 0;
                if cmds.is_empty() && worker_msgs.is_empty() && !check_rss {
                    continue;
                }

                let res = this.update(cx, |view, cx| {
                    if check_rss {
                        let live_rss = read_process_rss_mb();
                        if (live_rss - view.snap.rss_mb).abs() >= 0.05 {
                            view.snap.rss_mb = live_rss;
                            changed = true;
                        }
                    }

                    for cmd in cmds {
                        match cmd {
                            ControlCommand::ToggleOverlay => {
                                view.visible = !view.visible;
                                changed = true;
                            }
                            ControlCommand::ToggleInspector => {
                                view.visible = true;
                                view.layout.toggle_inspector();
                                changed = true;
                            }
                            ControlCommand::CaptureNow => {
                                view.snap.status_text = "Capturing picture…".into();
                                changed = true;
                            }
                            ControlCommand::ToggleFold => {
                                view.layout.toggle_fold();
                                changed = true;
                            }
                            ControlCommand::Quit => {
                                should_quit = true;
                            }
                        }
                    }

                    for msg in worker_msgs {
                        match msg {
                            WorkerToUi::Snapshot(s) => {
                                if let Some(&first_check) = s.advice.checks.first() {
                                    view.selected_cell = first_check;
                                } else if s.model.warden < 16 {
                                    view.selected_cell = s.model.warden;
                                }
                                view.snap = s;
                                changed = true;
                            }
                            WorkerToUi::RiotState(on) => {
                                if view.layout.apply_riot(on) {
                                    changed = true;
                                }
                            }
                            WorkerToUi::GamePlacement(placement) => {
                                if let DisplayTarget::Auto(cur_id) = view.display_target {
                                    if let Some(target_id) =
                                        crate::resolve_game_display_id(cx, Some(&placement))
                                    {
                                        if cur_id != Some(target_id) {
                                            view.display_target =
                                                DisplayTarget::Auto(Some(target_id));
                                            view.pending_reopen = true;
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if should_quit {
                        cx.quit();
                    } else if changed {
                        cx.notify();
                    }
                });

                if res.is_err() || should_quit {
                    break;
                }
            }
        })
        .detach();

        Self {
            worker_tx: state.worker_tx,
            shared_rx: state.shared_rx,
            snap: state.snap,
            visible: state.visible,
            layout: state.layout,
            selected_cell: state.selected_cell,
            display_target: state.display_target,
            is_layer_shell: state.is_layer_shell,
            margin_top: state.margin_top,
            margin_left: state.margin_left,
            pending_reopen: false,
            drag_start: None,
        }
    }

    pub fn to_window_state(&self) -> OverlayWindowState {
        OverlayWindowState {
            worker_tx: self.worker_tx.clone(),
            shared_rx: self.shared_rx.clone(),
            snap: self.snap.clone(),
            visible: self.visible,
            layout: self.layout,
            selected_cell: self.selected_cell,
            display_target: self.display_target,
            is_layer_shell: self.is_layer_shell,
            margin_top: self.margin_top,
            margin_left: self.margin_left,
        }
    }

    fn days_until_riot(&self) -> u8 {
        let m = &self.snap.model;
        let max = if m.mode == GameMode::Challenge {
            CHALLENGE_DAYS
        } else {
            m.max_days
        };
        max.saturating_sub(m.day)
    }

    fn folded_one_line(&self) -> SharedString {
        if !self.snap.log_connected {
            return "Game log not found".into();
        }
        if self.snap.riot {
            return "Riot: fight on".into();
        }
        match self.snap.phase {
            RunPhase::RunOver => "Run over: ready for the next".into(),
            RunPhase::WaitingForRun => {
                if !self.snap.game_connected {
                    "Waiting for the game".into()
                } else {
                    "Waiting for the next move".into()
                }
            }
            RunPhase::Active { .. } => {
                let a = &self.snap.advice;
                if a.treasure_wait {
                    "Treasure: press View Prison Status".into()
                } else if !a.tools_to_confirm.is_empty() {
                    "Confirm your instruments".into()
                } else if !a.problems.is_empty() {
                    "The board needs a fix".into()
                } else if !a.move_text.is_empty() {
                    let checks_n = a.checks.len();
                    if checks_n > 0 {
                        SharedString::from(format!("{} ({checks_n} to check)", a.move_text))
                    } else {
                        SharedString::from(a.move_text.clone())
                    }
                } else {
                    "Waiting for the next move".into()
                }
            }
        }
    }

    fn finish_drag(&mut self, pos: Point<Pixels>, win: &mut Window, cx: &mut Context<Self>) {
        if let Some(start) = self.drag_start.take() {
            let dx = f32::from(pos.x - start.x);
            let dy = f32::from(pos.y - start.y);
            if self.is_layer_shell && (dx.abs() > 8.0 || dy.abs() > 8.0) {
                self.margin_left = (self.margin_left + dx).max(0.0);
                self.margin_top = (self.margin_top + dy).max(0.0);
                let state = self.to_window_state();
                crate::open_overlay_window(cx, state);
                win.remove_window();
            }
        }
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let folded = self.layout.is_folded();
        let subtitle = if folded {
            self.folded_one_line()
        } else if !self.snap.log_connected {
            "Game log not found".into()
        } else if self.snap.riot {
            "Riot in progress".into()
        } else {
            match self.snap.phase {
                RunPhase::RunOver => "Run over · Ready for next run".into(),
                RunPhase::WaitingForRun => {
                    if !self.snap.game_connected {
                        "Waiting for the game".into()
                    } else {
                        "No active run · Waiting for prison".into()
                    }
                }
                RunPhase::Active { .. } => {
                    let riot_days = self.days_until_riot();
                    SharedString::from(format!(
                        "{riot_days} {} until Riot · {}",
                        if riot_days == 1 { "day" } else { "days" },
                        self.snap.model.step.label()
                    ))
                }
            }
        };

        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .px(px(10.0))
            .py(px(if folded { 4.0 } else { 6.0 }))
            .bg(rgba(0x171b21e8))
            .border_b_1()
            .border_color(rgb(0x353c47))
            .child(
                div()
                    .id("header-drag")
                    .flex()
                    .flex_1()
                    .items_center()
                    .gap(px(8.0))
                    .overflow_hidden()
                    .cursor_move()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, ev: &gpui::MouseDownEvent, win, _cx| {
                            this.drag_start = Some(ev.position);
                            win.start_window_move();
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, ev: &gpui::MouseUpEvent, win, cx| {
                            this.finish_drag(ev.position, win, cx);
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, ev: &gpui::MouseUpEvent, win, cx| {
                            this.finish_drag(ev.position, win, cx);
                        }),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(0x5fc4b8))
                            .child("TARTARUS"),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(if folded { rgb(0xeadfc5) } else { rgb(0x959dab) })
                            .child(subtitle),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        div()
                            .id("btn-monitor")
                            .w(px(22.0))
                            .h(px(22.0))
                            .rounded(px(4.0))
                            .bg(rgb(0x1f242c))
                            .hover(|s| s.bg(rgb(0x353c47)))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(11.5))
                            .text_color(rgb(0x959dab))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _ev, win, cx| {
                                    let displays = cx.displays();
                                    if displays.len() > 1 {
                                        let cur_id = this
                                            .display_target
                                            .id()
                                            .or_else(|| win.display(cx).map(|d| d.id()));
                                        let cur_idx = cur_id
                                            .and_then(|id| {
                                                displays.iter().position(|d| d.id() == id)
                                            })
                                            .unwrap_or(0);
                                        let next_idx = (cur_idx + 1) % displays.len();
                                        this.display_target =
                                            DisplayTarget::Manual(displays[next_idx].id());
                                    } else {
                                        this.margin_left = if this.margin_left < 200.0 {
                                            720.0
                                        } else {
                                            24.0
                                        };
                                    }
                                    let state = this.to_window_state();
                                    crate::open_overlay_window(cx, state);
                                    win.remove_window();
                                }),
                            )
                            .child("⇄"),
                    )
                    .child(
                        div()
                            .id("btn-fold")
                            .w(px(22.0))
                            .h(px(22.0))
                            .rounded(px(4.0))
                            .bg(rgb(0x1f242c))
                            .hover(|s| s.bg(rgb(0x353c47)))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(13.0))
                            .text_color(rgb(0xeadfc5))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _ev, _win, cx| {
                                    this.layout.toggle_fold();
                                    cx.notify();
                                }),
                            )
                            .child(if folded { "+" } else { "–" }),
                    )
                    .child(
                        div()
                            .id("btn-quit")
                            .w(px(22.0))
                            .h(px(22.0))
                            .rounded(px(4.0))
                            .bg(rgb(0x2b191b))
                            .hover(|s| s.bg(rgb(0x5c2327)))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(12.0))
                            .text_color(rgb(0xe0675f))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_this, _ev, _win, cx| {
                                    cx.quit();
                                }),
                            )
                            .child("×"),
                    ),
            )
    }

    fn render_banners(&self, cx: &mut Context<Self>) -> Option<impl IntoElement + use<>> {
        if !self.snap.phase.is_active() {
            return None;
        }
        let a = &self.snap.advice;
        if a.tools_to_confirm.is_empty()
            && !a.treasure_wait
            && a.problems.is_empty()
            && a.checks.is_empty()
        {
            return None;
        }

        let mut col = div().flex().flex_col().gap(px(6.0));

        if !a.tools_to_confirm.is_empty() {
            let msg = SharedString::from(a.tools_to_confirm.clone());
            col = col.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(5.0))
                    .p(px(8.0))
                    .rounded(px(4.0))
                    .bg(rgba(0x2d2612dd))
                    .border_1()
                    .border_color(rgb(0xf1cd48))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(rgb(0xf1cd48))
                            .child(msg),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .id("btn-tools-ok")
                                    .px(px(8.0))
                                    .py(px(3.0))
                                    .rounded(px(4.0))
                                    .bg(rgb(0x243b39))
                                    .border_1()
                                    .border_color(rgb(0x5fc4b8))
                                    .cursor_pointer()
                                    .text_size(px(11.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(0xeadfc5))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _ev, _win, _cx| {
                                            let _ = this.worker_tx.send(UiToWorker::ConfirmTools);
                                        }),
                                    )
                                    .child("These are right"),
                            )
                            .child(
                                div()
                                    .id("btn-tools-edit")
                                    .px(px(8.0))
                                    .py(px(3.0))
                                    .rounded(px(4.0))
                                    .bg(rgb(0x1f242c))
                                    .border_1()
                                    .border_color(rgb(0x545e6d))
                                    .cursor_pointer()
                                    .text_size(px(11.0))
                                    .text_color(rgb(0xe3e6eb))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _ev, _win, cx| {
                                            this.layout.open_inspector(true);
                                            cx.notify();
                                        }),
                                    )
                                    .child("Instruments"),
                            ),
                    ),
            );
        }

        if a.treasure_wait {
            col = col.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .p(px(8.0))
                    .rounded(px(4.0))
                    .bg(rgba(0x2d2612dd))
                    .border_1()
                    .border_color(rgb(0xf1cd48))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(rgb(0xf1cd48))
                            .child("The Warden's Treasure is open. Press View Prison Status in the game, wait 1s, then return."),
                    )
                    .child(
                        div()
                            .id("btn-treasure-now")
                            .px(px(8.0))
                            .py(px(3.0))
                            .rounded(px(4.0))
                            .bg(rgb(0x243b39))
                            .border_1()
                            .border_color(rgb(0x5fc4b8))
                            .cursor_pointer()
                            .text_size(px(11.0))
                            .text_color(rgb(0xeadfc5))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _ev, _win, _cx| {
                                    let _ = this.worker_tx.send(UiToWorker::AckTreasureNow);
                                }),
                            )
                            .child("Advise anyway"),
                    ),
            );
        }

        if let Some(first_prob) = a.problems.first() {
            let text = SharedString::from(format!("The board needs a fix: {first_prob}"));
            col = col.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .p(px(8.0))
                    .rounded(px(4.0))
                    .bg(rgba(0x321818dd))
                    .border_1()
                    .border_color(rgb(0xe0675f))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(rgb(0xf3a49e))
                            .child(text),
                    )
                    .child(
                        div()
                            .id("btn-fix-prob")
                            .px(px(8.0))
                            .py(px(3.0))
                            .rounded(px(4.0))
                            .bg(rgb(0x243b39))
                            .border_1()
                            .border_color(rgb(0x5fc4b8))
                            .cursor_pointer()
                            .text_size(px(11.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(0xeadfc5))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _ev, _win, cx| {
                                    this.layout.open_inspector(false);
                                    cx.notify();
                                }),
                            )
                            .child("Fix"),
                    ),
            );
        }

        if !a.checks.is_empty() {
            let n = a.checks.len();
            let labels: FixedVec<&'static str, 16> =
                a.checks.iter().map(|&c| cell_label(c)).collect();
            let text = SharedString::from(format!(
                "{n} {} to check ({})",
                if n == 1 { "cell" } else { "cells" },
                labels.join(", ")
            ));
            col = col.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .p(px(6.0))
                    .rounded(px(4.0))
                    .bg(rgba(0x2d2612dd))
                    .border_1()
                    .border_color(rgb(0xf1cd48))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(rgb(0xf1cd48))
                            .child(text),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .id("btn-fix-checks")
                                    .px(px(7.0))
                                    .py(px(2.0))
                                    .rounded(px(4.0))
                                    .bg(rgb(0x1f242c))
                                    .border_1()
                                    .border_color(rgb(0x545e6d))
                                    .cursor_pointer()
                                    .text_size(px(11.0))
                                    .text_color(rgb(0xe3e6eb))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _ev, _win, cx| {
                                            this.layout.open_inspector(false);
                                            if let Some(&c0) = this.snap.advice.checks.first() {
                                                this.selected_cell = c0;
                                            }
                                            cx.notify();
                                        }),
                                    )
                                    .child("Fix"),
                            )
                            .child(
                                div()
                                    .id("btn-clear-checks")
                                    .px(px(7.0))
                                    .py(px(2.0))
                                    .rounded(px(4.0))
                                    .bg(rgb(0x243b39))
                                    .border_1()
                                    .border_color(rgb(0x5fc4b8))
                                    .cursor_pointer()
                                    .text_size(px(11.0))
                                    .text_color(rgb(0xeadfc5))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _ev, _win, _cx| {
                                            this.snap.model.checks = 0;
                                            let _ = this.worker_tx.send(
                                                UiToWorker::UpdateModelAndRecheck(
                                                    this.snap.model.clone(),
                                                ),
                                            );
                                        }),
                                    )
                                    .child("Recheck"),
                            ),
                    ),
            );
        }

        Some(col)
    }

    fn render_advice_card(&self) -> impl IntoElement + use<> {
        if !self.snap.phase.is_active() {
            let idle_msg: &'static str = if !self.snap.log_connected {
                "Couldn't find your game log (UE_game.log)."
            } else if !self.snap.game_connected {
                "Waiting for Torchlight: Infinite…"
            } else if self.snap.riot {
                "Riot: fight on. Advice resumes on the next prison screen."
            } else if self.snap.phase.is_run_over() {
                "Run over. Ready for the next run."
            } else {
                "Advice comes with the next move. Or take a picture now."
            };
            return div()
                .p(px(8.0))
                .rounded(px(4.0))
                .bg(rgb(0x171b21))
                .border_1()
                .border_color(rgb(0x353c47))
                .text_size(px(12.5))
                .text_color(rgb(0x959dab))
                .child(idle_msg);
        }

        let a = &self.snap.advice;
        if a.move_text.is_empty() {
            return div()
                .p(px(8.0))
                .rounded(px(4.0))
                .bg(rgb(0x171b21))
                .border_1()
                .border_color(rgb(0x353c47))
                .text_size(px(12.5))
                .text_color(rgb(0x959dab))
                .child("Nothing to decide on this screen, or fix the board problem above.");
        }

        let move_str = SharedString::from(a.move_text.clone());
        let why_str = SharedString::from(a.why.clone());

        let clean_expect = a
            .expect
            .trim_start_matches("Expected: ")
            .trim_end_matches('.');
        let clean_status = a
            .status
            .trim_start_matches("Live check: ")
            .replace(" playouts per option", " playouts each");
        let meta_parts: Vec<&str> = [clean_expect, clean_status.as_str()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();
        let meta_str = SharedString::from(meta_parts.join(" · "));

        let mut card = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_size(px(10.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0x5fc4b8))
                    .child("RECOMMENDED"),
            )
            .child(
                div()
                    .text_size(px(18.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0xeadfc5))
                    .child(move_str),
            );

        if !a.why.is_empty() {
            card = card.child(
                div()
                    .text_size(px(12.5))
                    .text_color(rgb(0xe3e6eb))
                    .child(why_str),
            );
        }

        if !meta_str.is_empty() {
            card = card.child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(0x959dab))
                    .child(meta_str),
            );
        }

        if !a.lesson.is_empty() {
            let lesson_str = SharedString::from(a.lesson.clone());
            card = card.child(
                div()
                    .text_size(px(11.5))
                    .text_color(rgb(0x86c497))
                    .child(lesson_str),
            );
        }

        if !a.note.is_empty() {
            let note_str = SharedString::from(a.note.clone());
            card = card.child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(0x959dab))
                    .child(note_str),
            );
        }

        if !a.options.is_empty() {
            let mut opts_col = div().flex().flex_col().gap(px(3.0)).mt(px(4.0));
            for opt in a.options.iter().take(5) {
                let lbl = SharedString::from(opt.label.clone());
                let val = SharedString::from(opt.value_text.clone());
                opts_col = opts_col.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(7.0))
                        .py(px(3.0))
                        .rounded(px(4.0))
                        .bg(if opt.is_best {
                            rgba(0x243b39cc)
                        } else {
                            rgba(0x171b21cc)
                        })
                        .border_1()
                        .border_color(if opt.is_best {
                            rgb(0x5fc4b8)
                        } else {
                            rgb(0x353c47)
                        })
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(if opt.is_best {
                                    gpui::FontWeight::BOLD
                                } else {
                                    gpui::FontWeight::NORMAL
                                })
                                .text_color(if opt.is_best {
                                    rgb(0xeadfc5)
                                } else {
                                    rgb(0xe3e6eb)
                                })
                                .child(lbl),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(rgb(0x959dab))
                                .child(val),
                        ),
                );
            }
            card = card.child(opts_col);
        }

        card
    }

    fn render_inspector(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        const GRID_TO_RING: [[Option<u8>; 6]; 4] = {
            let mut g = [[None; 6]; 4];
            let mut i = 0;
            while i < RING.len() {
                let (u, v) = RING[i];
                g[v as usize][u as usize] = Some(i as u8);
                i += 1;
            }
            g
        };

        let model = &self.snap.model;
        let sel = self.selected_cell.min(15);

        // 6 cols x 4 rows perimeter grid matching RING
        let mut grid_rows = div().flex().flex_col().gap(px(3.0));
        for row in 0..4u8 {
            let mut r_div = div().flex().gap(px(3.0));
            for col in 0..6u8 {
                let ring_pos = GRID_TO_RING[row as usize][col as usize];

                if let Some(n) = ring_pos {
                    let cell = &model.cells[n as usize];
                    let is_warden = model.warden == n;
                    let is_door = model.step == BoardStep::Patrol && model.has_dest(n);
                    let is_sel = sel == n;
                    let is_check = model.is_checked(n);

                    let mut badges = String::new();
                    if is_warden {
                        badges.push_str(" W");
                    }
                    if is_door {
                        badges.push_str(" D");
                    }
                    if is_check {
                        badges.push_str(" ?");
                    }
                    let top_line = SharedString::from(format!("{:02}{badges}", n + 1));
                    let mid_line = SharedString::from(cell.short_label());

                    let border_col = if is_sel {
                        rgb(0x5fc4b8)
                    } else if is_check {
                        rgb(0xf1cd48)
                    } else if is_warden {
                        rgb(0xf28c36)
                    } else if is_door {
                        rgb(0x86c497)
                    } else {
                        rgb(0x353c47)
                    };

                    let bg_col = if is_sel {
                        rgb(0x243b39)
                    } else if is_check {
                        rgb(0x2d2612)
                    } else {
                        rgb(0x171b21)
                    };

                    r_div = r_div.child(
                        div()
                            .id(("ring-cell", n as usize))
                            .w(px(54.0))
                            .h(px(36.0))
                            .px(px(3.0))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .bg(bg_col)
                            .border_1()
                            .border_color(border_col)
                            .cursor_pointer()
                            .flex()
                            .flex_col()
                            .justify_between()
                            .overflow_hidden()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _ev, _win, cx| {
                                    this.selected_cell = n;
                                    cx.notify();
                                }),
                            )
                            .child(
                                div()
                                    .text_size(px(8.5))
                                    .text_color(if is_check {
                                        rgb(0xf1cd48)
                                    } else if is_warden {
                                        rgb(0xf28c36)
                                    } else {
                                        rgb(0x959dab)
                                    })
                                    .child(top_line),
                            )
                            .child(
                                div()
                                    .text_size(px(9.5))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(0xeadfc5))
                                    .child(mid_line),
                            ),
                    );
                } else {
                    // Interior court cell (cols 1..=4, rows 1..=2)
                    let info_label = match (col, row) {
                        (1, 1) => SharedString::from(format!("Day {}", model.day)),
                        (2, 1) => SharedString::from(model.step.as_str().to_uppercase()),
                        (3, 1) => SharedString::from(format!("Vault {}", model.vault.len())),
                        (4, 1) => SharedString::from(format!("Tools {}", model.tools.len())),
                        (1, 2) => SharedString::from(model.objective.as_str().to_uppercase()),
                        (2, 2) => SharedString::from(if model.swap_used {
                            "Swapped"
                        } else {
                            "Swap OK"
                        }),
                        (3, 2) => {
                            SharedString::from(if model.devour_used { "Ate" } else { "Eat OK" })
                        }
                        (4, 2) => SharedString::from(model.start_event.as_str().to_uppercase()),
                        _ => SharedString::from(""),
                    };
                    r_div = r_div.child(
                        div()
                            .w(px(54.0))
                            .h(px(36.0))
                            .p(px(2.0))
                            .rounded(px(3.0))
                            .bg(rgba(0x0f121688))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_size(px(8.5))
                                    .text_color(rgb(0x545e6d))
                                    .child(info_label),
                            ),
                    );
                }
            }
            grid_rows = grid_rows.child(r_div);
        }

        // Selected cell editor controls
        let sel_cell = model.cells[sel as usize];
        let edit_title =
            SharedString::from(format!("Cell {:02} ({})", sel + 1, sel_cell.short_label()));

        let pos_row = div()
            .flex()
            .flex_wrap()
            .gap(px(4.0))
            .child(
                div()
                    .id("ed-set-warden")
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(if model.warden == sel {
                        rgb(0x5c3614)
                    } else {
                        rgb(0x1f242c)
                    })
                    .border_1()
                    .border_color(rgb(0xf28c36))
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xeadfc5))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _win, cx| {
                            this.snap.model.warden = sel;
                            this.snap.model.set_dest(sel, false);
                            cx.notify();
                        }),
                    )
                    .child("Warden Here"),
            )
            .child(
                div()
                    .id("ed-toggle-door")
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(if model.has_dest(sel) {
                        rgb(0x243b39)
                    } else {
                        rgb(0x1f242c)
                    })
                    .border_1()
                    .border_color(rgb(0x5fc4b8))
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xe3e6eb))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _win, cx| {
                            if sel != this.snap.model.warden {
                                this.snap.model.step = BoardStep::Patrol;
                                this.snap.model.toggle_dest(sel);
                                cx.notify();
                            }
                        }),
                    )
                    .child("Door"),
            )
            .child(
                div()
                    .id("ed-locked")
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(rgb(0x1f242c))
                    .border_1()
                    .border_color(rgb(0x545e6d))
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xe3e6eb))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _win, cx| {
                            if sel != OFFICE {
                                this.snap.model.cells[sel as usize] = CellModel::sealed();
                                this.snap.model.set_checked(sel, false);
                                cx.notify();
                            }
                        }),
                    )
                    .child("Locked"),
            )
            .child(
                div()
                    .id("ed-empty")
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(rgb(0x1f242c))
                    .border_1()
                    .border_color(rgb(0x545e6d))
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xe3e6eb))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _win, cx| {
                            if sel != OFFICE {
                                let mut cm = CellModel::sealed();
                                cm.kind = CellKind::Empty;
                                this.snap.model.cells[sel as usize] = cm;
                                this.snap.model.set_checked(sel, false);
                                cx.notify();
                            }
                        }),
                    )
                    .child("Empty"),
            )
            .child(
                div()
                    .id("ed-nest")
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(rgb(0x1f242c))
                    .border_1()
                    .border_color(rgb(0x545e6d))
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xe3e6eb))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _win, cx| {
                            if sel != OFFICE {
                                let mut cm = CellModel::sealed();
                                cm.kind = CellKind::Nest;
                                cm.due = this.snap.model.day + 2;
                                this.snap.model.cells[sel as usize] = cm;
                                this.snap.model.set_checked(sel, false);
                                cx.notify();
                            }
                        }),
                    )
                    .child("Nest"),
            )
            .child(
                div()
                    .id("ed-event")
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(if sel_cell.event.is_some() {
                        rgb(0x5c3614)
                    } else {
                        rgb(0x1f242c)
                    })
                    .border_1()
                    .border_color(rgb(0x545e6d))
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xe3e6eb))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _win, cx| {
                            if sel != OFFICE {
                                let c = &mut this.snap.model.cells[sel as usize];
                                c.event = if c.event.is_some() {
                                    CellEvent::None
                                } else {
                                    CellEvent::Unknown
                                };
                                cx.notify();
                            }
                        }),
                    )
                    .child("Event !"),
            )
            .child(
                div()
                    .id("ed-ivory")
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(if sel_cell.ivory {
                        rgb(0x243b39)
                    } else {
                        rgb(0x1f242c)
                    })
                    .border_1()
                    .border_color(rgb(0x545e6d))
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xe3e6eb))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _win, cx| {
                            if sel != OFFICE {
                                let c = &mut this.snap.model.cells[sel as usize];
                                c.ivory = !c.ivory;
                                cx.notify();
                            }
                        }),
                    )
                    .child("Ivory ♦"),
            );

        // Beast & Tier row
        let mut beast_row = div().flex().flex_wrap().gap(px(4.0));
        for (idx, beast) in BEASTS.into_iter().enumerate() {
            let active = sel_cell.kind == CellKind::Inmate && sel_cell.beast == beast;
            beast_row = beast_row.child(
                div()
                    .id(("ed-beast", idx))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(if active { rgb(0x243b39) } else { rgb(0x1f242c) })
                    .border_1()
                    .border_color(if active { rgb(0x5fc4b8) } else { rgb(0x545e6d) })
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xeadfc5))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _win, cx| {
                            if sel != OFFICE {
                                let c = &mut this.snap.model.cells[sel as usize];
                                c.kind = CellKind::Inmate;
                                c.beast = beast;
                                this.snap.model.set_checked(sel, false);
                                cx.notify();
                            }
                        }),
                    )
                    .child(beast.name()),
            );
        }
        for tier in 0..=4u8 {
            let active = sel_cell.kind == CellKind::Inmate && sel_cell.tier == tier;
            let short = &tier_name(tier)[..3];
            let lbl = SharedString::from(format!("{short}"));
            beast_row = beast_row.child(
                div()
                    .id(("ed-tier", tier as usize))
                    .px(px(5.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(if active { rgb(0x243b39) } else { rgb(0x171b21) })
                    .border_1()
                    .border_color(if active { rgb(0x5fc4b8) } else { rgb(0x353c47) })
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xe3e6eb))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _win, cx| {
                            if sel != OFFICE {
                                let c = &mut this.snap.model.cells[sel as usize];
                                if !matches!(
                                    c.kind,
                                    CellKind::Scratch | CellKind::Revealed | CellKind::Scouted
                                ) {
                                    c.kind = CellKind::Inmate;
                                }
                                c.tier = tier;
                                this.snap.model.set_checked(sel, false);
                                cx.notify();
                            }
                        }),
                    )
                    .child(lbl),
            );
        }

        // Step & Day controls
        let mut step_row = div().flex().flex_wrap().gap(px(4.0));
        for (idx, (st, lbl)) in [
            (BoardStep::Patrol, "Patrol"),
            (BoardStep::Manage, "Manage"),
            (BoardStep::Chain, "Chain"),
            (BoardStep::Office, "Office"),
            (BoardStep::Ratway, "Ratway"),
        ]
        .into_iter()
        .enumerate()
        {
            let active = model.step == st;
            step_row = step_row.child(
                div()
                    .id(("ed-step", idx))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(if active { rgb(0x243b39) } else { rgb(0x171b21) })
                    .border_1()
                    .border_color(if active { rgb(0x5fc4b8) } else { rgb(0x353c47) })
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xe3e6eb))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _win, cx| {
                            this.snap.model.step = st;
                            if st == BoardStep::Office {
                                this.snap.model.warden = OFFICE;
                            }
                            cx.notify();
                        }),
                    )
                    .child(lbl),
            );
        }

        let tools_drawer_open = self.layout.is_tools_drawer_open();
        let day_lbl = SharedString::from(format!("Day {} (+1)", model.day));
        step_row = step_row
            .child(
                div()
                    .id("ed-day-inc")
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(rgb(0x1f242c))
                    .border_1()
                    .border_color(rgb(0x545e6d))
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xeadfc5))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _ev, _win, cx| {
                            let max = this.snap.model.max_days.max(1);
                            this.snap.model.day = (this.snap.model.day % max) + 1;
                            cx.notify();
                        }),
                    )
                    .child(day_lbl),
            )
            .child(
                div()
                    .id("ed-tools-drawer")
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(if tools_drawer_open {
                        rgb(0x243b39)
                    } else {
                        rgb(0x1f242c)
                    })
                    .border_1()
                    .border_color(rgb(0x545e6d))
                    .cursor_pointer()
                    .text_size(px(10.0))
                    .text_color(rgb(0xe3e6eb))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _ev, _win, cx| {
                            this.layout.toggle_tools_drawer();
                            cx.notify();
                        }),
                    )
                    .child("Tools…"),
            );

        let mut container = div()
            .flex()
            .flex_col()
            .gap(px(5.0))
            .p(px(8.0))
            .rounded(px(6.0))
            .bg(rgba(0x171b21f2))
            .border_1()
            .border_color(rgb(0x353c47))
            .child(grid_rows)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(0x5fc4b8))
                            .child(edit_title),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .id("btn-example")
                                    .px(px(6.0))
                                    .py(px(2.0))
                                    .rounded(px(4.0))
                                    .bg(rgb(0x1f242c))
                                    .border_1()
                                    .border_color(rgb(0x545e6d))
                                    .cursor_pointer()
                                    .text_size(px(10.0))
                                    .text_color(rgb(0x959dab))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _ev, _win, _cx| {
                                            let _ = this.worker_tx.send(UiToWorker::LoadExample);
                                        }),
                                    )
                                    .child("Example"),
                            )
                            .child(
                                div()
                                    .id("btn-recheck-inspector")
                                    .px(px(8.0))
                                    .py(px(2.0))
                                    .rounded(px(4.0))
                                    .bg(rgb(0x243b39))
                                    .border_1()
                                    .border_color(rgb(0x5fc4b8))
                                    .cursor_pointer()
                                    .text_size(px(10.5))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(0xeadfc5))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _ev, _win, _cx| {
                                            this.snap.model.checks = 0;
                                            let _ = this.worker_tx.send(
                                                UiToWorker::UpdateModelAndRecheck(
                                                    this.snap.model.clone(),
                                                ),
                                            );
                                        }),
                                    )
                                    .child("Recheck"),
                            ),
                    ),
            )
            .child(pos_row)
            .child(beast_row)
            .child(step_row);

        if tools_drawer_open {
            let mut tools_wrap = div().flex().flex_wrap().gap(px(3.0)).pt(px(4.0));
            for (idx, tool) in ALL_TOOLS.into_iter().enumerate() {
                let held = model.tools.contains(&tool);
                let short_name = tool.name().split(':').next().unwrap_or(tool.name());
                let lbl = SharedString::from(short_name.to_string());
                tools_wrap = tools_wrap.child(
                    div()
                        .id(("ed-tool", idx))
                        .px(px(5.0))
                        .py(px(2.0))
                        .rounded(px(3.0))
                        .bg(if held { rgb(0x243b39) } else { rgb(0x0f1216) })
                        .border_1()
                        .border_color(if held { rgb(0x5fc4b8) } else { rgb(0x353c47) })
                        .cursor_pointer()
                        .text_size(px(9.5))
                        .text_color(if held { rgb(0xeadfc5) } else { rgb(0x959dab) })
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _ev, _win, cx| {
                                if let Some(pos) =
                                    this.snap.model.tools.iter().position(|&t| t == tool)
                                {
                                    this.snap.model.tools.remove(pos);
                                } else {
                                    this.snap.model.tools.push(tool);
                                }
                                cx.notify();
                            }),
                        )
                        .child(lbl),
                );
            }
            container = container.child(tools_wrap);
        }

        container
    }

    fn render_status_dot(label: &'static str, active: bool) -> impl IntoElement {
        let dot_color = if active { rgb(0x86c497) } else { rgb(0xe0675f) };
        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .child(div().w(px(7.0)).h(px(7.0)).rounded(px(4.0)).bg(dot_color))
            .child(
                div()
                    .text_size(px(10.5))
                    .text_color(rgb(0x959dab))
                    .child(label),
            )
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let inspector_open = self.layout.is_inspector_open();
        let status_str = SharedString::from(self.snap.status_text.clone());
        let live_rss = read_process_rss_mb().max(self.snap.rss_mb);
        let rss_str = SharedString::from(format!("{live_rss:.1} MB"));
        let p = self.snap.scan_profile;
        let cap_ms = self.snap.cap_ms;
        let scan_ms = self.snap.scan_ms;
        let solve_ms = self.snap.solve_ms;

        let perf_line = if p.cells_ms > 0.0 || p.ocr_ms > 0.0 {
            Some(SharedString::from(format!(
                "cap {cap_ms:.1}ms · scan {scan_ms:.2}ms [loc {:.2} · cells {:.2} · ocr {:.2}] · solve {solve_ms:.1}ms",
                p.loc_ms, p.cells_ms, p.ocr_ms
            )))
        } else if scan_ms > 0.0 {
            if solve_ms > 0.0 {
                Some(SharedString::from(format!(
                    "cap {cap_ms:.1}ms · scan {scan_ms:.2}ms [loc {:.2} · pick {:.2}] · solve {solve_ms:.1}ms",
                    p.loc_ms, p.pick_ms
                )))
            } else {
                Some(SharedString::from(format!(
                    "cap {cap_ms:.1}ms · scan {scan_ms:.2}ms [loc {:.2} · pick {:.2}]",
                    p.loc_ms, p.pick_ms
                )))
            }
        } else if solve_ms > 0.0 {
            Some(SharedString::from(format!("solve {solve_ms:.1}ms")))
        } else {
            None
        };

        let mut footer = div()
            .flex()
            .flex_col()
            .gap(px(3.0))
            .px(px(10.0))
            .py(px(6.0))
            .border_t_1()
            .border_color(rgb(0x353c47))
            .bg(rgba(0x171b21d8))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .id("btn-foot-board")
                                    .px(px(9.0))
                                    .py(px(3.0))
                                    .rounded(px(4.0))
                                    .bg(if inspector_open {
                                        rgb(0x243b39)
                                    } else {
                                        rgb(0x1f242c)
                                    })
                                    .border_1()
                                    .border_color(if inspector_open {
                                        rgb(0x5fc4b8)
                                    } else {
                                        rgb(0x545e6d)
                                    })
                                    .hover(|s| s.border_color(rgb(0x5fc4b8)))
                                    .cursor_pointer()
                                    .text_size(px(12.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(0xe3e6eb))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _ev, _win, cx| {
                                            this.layout.toggle_inspector();
                                            cx.notify();
                                        }),
                                    )
                                    .child("Board"),
                            )
                            .child(
                                div()
                                    .id("btn-foot-picture")
                                    .px(px(9.0))
                                    .py(px(3.0))
                                    .rounded(px(4.0))
                                    .bg(rgb(0x1f242c))
                                    .border_1()
                                    .border_color(rgb(0x545e6d))
                                    .hover(|s| s.border_color(rgb(0x5fc4b8)))
                                    .cursor_pointer()
                                    .text_size(px(12.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(0xe3e6eb))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _ev, _win, cx| {
                                            this.snap.status_text = "Capturing picture…".into();
                                            let _ = this.worker_tx.send(UiToWorker::CaptureNow);
                                            cx.notify();
                                        }),
                                    )
                                    .child("Picture"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .px(px(5.0))
                                    .py(px(1.0))
                                    .rounded(px(3.0))
                                    .bg(rgb(0x0f1216))
                                    .border_1()
                                    .border_color(rgb(0x28303b))
                                    .text_size(px(9.5))
                                    .text_color(rgb(0x5fc4b8))
                                    .child(rss_str),
                            )
                            .child(Self::render_status_dot("Log", self.snap.log_connected))
                            .child(Self::render_status_dot("Window", self.snap.game_connected)),
                    ),
            )
            .child(
                div()
                    .text_size(px(9.5))
                    .text_color(rgb(0x959dab))
                    .child(status_str),
            );

        if let Some(perf) = perf_line {
            footer = footer.child(
                div()
                    .text_size(px(9.0))
                    .text_color(rgb(0x545e6d))
                    .child(perf),
            );
        }

        footer
    }
}

impl Render for OverlayView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.pending_reopen {
            self.pending_reopen = false;
            let cur_win_display = window.display(cx).map(|d| d.id());
            if self.display_target.id().is_some() && cur_win_display != self.display_target.id() {
                let state = self.to_window_state();
                crate::open_overlay_window(cx, state);
                window.remove_window();
                return div().size_full().flex().flex_col().items_start();
            }
        }

        if !self.visible {
            return div().size_full().flex().flex_col().items_start();
        }

        let header = self.render_header(cx);

        if self.layout.is_folded() {
            let card = div()
                .w(px(360.0))
                .rounded(px(6.0))
                .bg(rgba(0x0f1216eb))
                .border_1()
                .border_color(rgb(0x446e6c))
                .overflow_hidden()
                .flex()
                .flex_col()
                .child(header);
            return div()
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .child(card);
        }

        let mut body = div()
            .px(px(10.0))
            .py(px(8.0))
            .flex()
            .flex_col()
            .gap(px(8.0));

        if let Some(banners) = self.render_banners(cx) {
            body = body.child(banners);
        }

        body = body.child(self.render_advice_card());

        if self.layout.is_inspector_open() {
            body = body.child(self.render_inspector(cx));
        }

        let footer = self.render_footer(cx);

        let card = div()
            .w(px(360.0))
            .rounded(px(6.0))
            .bg(rgba(0x0f1216eb))
            .border_1()
            .border_color(rgb(0x446e6c))
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(header)
            .child(body)
            .child(footer);

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_start()
            .child(card)
    }
}

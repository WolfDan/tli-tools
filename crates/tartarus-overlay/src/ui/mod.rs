//! GPUI overlay view, header bar, status footer, and top-level `Render` implementation.

mod advice;
mod inspector;

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui::{
    Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point, Render,
    SharedString, Styled, Window, div, px, rgb, rgba,
};
use tartarus_core::{CHALLENGE_DAYS, GameMode, RunPhase};

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
                                view.snap = *s;
                                changed = true;
                            }
                            WorkerToUi::RiotState(on) => {
                                if view.layout.apply_riot(on) {
                                    changed = true;
                                }
                            }
                            WorkerToUi::GamePlacement(placement) => {
                                if let DisplayTarget::Auto(cur_id) = view.display_target
                                    && let Some(target_id) =
                                        crate::resolve_game_display_id(cx, Some(&placement))
                                    && cur_id != Some(target_id)
                                {
                                    view.display_target = DisplayTarget::Auto(Some(target_id));
                                    view.pending_reopen = true;
                                    changed = true;
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

    pub(crate) fn days_until_riot(&self) -> u8 {
        let m = &self.snap.model;
        let max = if m.mode == GameMode::Challenge {
            CHALLENGE_DAYS
        } else {
            m.max_days
        };
        max.saturating_sub(m.day)
    }

    pub(crate) fn folded_one_line(&self) -> SharedString {
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

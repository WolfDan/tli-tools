//! Actionable alert banners and ranked solver recommendation cards.

use gpui::{
    Context, InteractiveElement, IntoElement, MouseButton, ParentElement, SharedString, Styled,
    div, px, rgb, rgba,
};
use tartarus_core::{FixedVec, cell_label};

use crate::state::UiToWorker;

use super::OverlayView;

const OPTIONS_SHOWN_MAX: usize = 5;

const _: () = {
    assert!(OPTIONS_SHOWN_MAX > 0);
    assert!(OPTIONS_SHOWN_MAX <= 16);
};

impl OverlayView {
    fn render_tools_confirm_buttons(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        assert!(self.selected_cell < 16);
        assert!(!self.snap.advice.tools_to_confirm.is_empty());
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
            )
    }

    fn render_tools_confirm_banner(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let msg = SharedString::from(self.snap.advice.tools_to_confirm.clone());
        assert!(!msg.is_empty());
        assert!(self.selected_cell < 16);
        let buttons = self.render_tools_confirm_buttons(cx);
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
            .child(buttons)
    }

    fn render_treasure_wait_banner(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        assert!(self.snap.advice.treasure_wait);
        assert!(self.selected_cell < 16);
        let msg = "The Warden's Treasure is open. Press View Prison Status in the game, \
                   wait 1s, then return.";
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
                    .child(msg),
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
            )
    }

    fn render_problem_banner(
        &self,
        first_prob: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        assert!(!first_prob.is_empty());
        assert!(self.selected_cell < 16);
        let text = SharedString::from(format!("The board needs a fix: {first_prob}"));
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
            )
    }

    fn render_checks_banner_buttons(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        assert!(!self.snap.advice.checks.is_empty());
        assert!(self.selected_cell < 16);
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
                                this.selected_cell = c0.min(15);
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
                            let _ = this
                                .worker_tx
                                .send(UiToWorker::UpdateModelAndRecheck(Box::new(this.snap.model)));
                        }),
                    )
                    .child("Recheck"),
            )
    }

    fn render_checks_banner(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let a = &self.snap.advice;
        let n = a.checks.len();
        assert!(n > 0 && n <= 16);
        let labels: FixedVec<&'static str, 16> = a.checks.iter().map(|&c| cell_label(c)).collect();
        let text = SharedString::from(format!(
            "{n} {} to check ({})",
            if n == 1 { "cell" } else { "cells" },
            labels.join(", ")
        ));
        assert!(!text.is_empty());
        let buttons = self.render_checks_banner_buttons(cx);
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
            .child(buttons)
    }

    pub(crate) fn render_banners(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement + use<>> {
        assert!(self.selected_cell < 16);
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
            col = col.child(self.render_tools_confirm_banner(cx));
        }
        if a.treasure_wait {
            col = col.child(self.render_treasure_wait_banner(cx));
        }
        if let Some(first_prob) = a.problems.first() {
            let prob_clone = first_prob.clone();
            col = col.child(self.render_problem_banner(&prob_clone, cx));
        }
        if !a.checks.is_empty() {
            col = col.child(self.render_checks_banner(cx));
        }
        assert!(self.snap.phase.is_active());
        Some(col)
    }

    fn idle_advice_message(&self) -> &'static str {
        assert!(!self.snap.phase.is_active());
        let msg = if !self.snap.log_connected {
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
        assert!(!msg.is_empty());
        msg
    }

    fn format_advice_meta(&self) -> SharedString {
        let a = &self.snap.advice;
        assert!(self.selected_cell < 16);
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
        assert!(meta_parts.len() <= 2);
        SharedString::from(meta_parts.join(" · "))
    }

    fn render_advice_options(&self) -> impl IntoElement + use<> {
        let a = &self.snap.advice;
        assert!(!a.options.is_empty());
        assert!(self.selected_cell < 16);
        let mut opts_col = div().flex().flex_col().gap(px(3.0)).mt(px(4.0));
        for opt in a.options.iter().take(OPTIONS_SHOWN_MAX) {
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
        opts_col
    }

    fn render_idle_advice_box(msg: &'static str) -> gpui::Div {
        assert!(!msg.is_empty());
        assert!(msg.len() <= 128);
        div()
            .p(px(8.0))
            .rounded(px(4.0))
            .bg(rgb(0x171b21))
            .border_1()
            .border_color(rgb(0x353c47))
            .text_size(px(12.5))
            .text_color(rgb(0x959dab))
            .child(msg)
    }

    fn append_advice_details(&self, mut card: gpui::Div) -> gpui::Div {
        assert!(self.selected_cell < 16);
        let a = &self.snap.advice;
        assert!(!a.move_text.is_empty());
        let why_str = SharedString::from(a.why.clone());
        let meta_str = self.format_advice_meta();

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
            card = card.child(self.render_advice_options());
        }
        card
    }

    pub(crate) fn render_advice_card(&self) -> impl IntoElement + use<> {
        assert!(self.selected_cell < 16);
        if !self.snap.phase.is_active() {
            return Self::render_idle_advice_box(self.idle_advice_message());
        }

        let a = &self.snap.advice;
        if a.move_text.is_empty() {
            return Self::render_idle_advice_box(
                "Nothing to decide on this screen, or fix the board problem above.",
            );
        }

        let move_str = SharedString::from(a.move_text.clone());
        assert!(!move_str.is_empty());

        let card = div()
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

        self.append_advice_details(card)
    }
}

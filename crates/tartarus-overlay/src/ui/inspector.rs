//! Interactive 6x4 ring board inspector and cell/tool state editor.

use gpui::{
    Context, InteractiveElement, IntoElement, MouseButton, ParentElement, SharedString, Styled,
    div, px, rgb, rgba,
};
use tartarus_core::{
    ALL_TOOLS, BEASTS, BoardStep, CellEvent, CellKind, CellModel, OFFICE, tier_name,
};
use tartarus_scan::RING;

use crate::state::UiToWorker;

use super::OverlayView;

impl OverlayView {
    pub(crate) fn render_inspector(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
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
            let lbl = SharedString::from(short.to_string());
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
                                                UiToWorker::UpdateModelAndRecheck(Box::new(
                                                    this.snap.model,
                                                )),
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
}

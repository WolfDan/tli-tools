#![feature(portable_simd)]

pub mod capture;
pub mod log_watcher;
pub mod state;
pub mod tray;
pub mod ui;
pub mod worker;

use std::sync::{Arc, Mutex};

use capture::{GameWindowPlacement, WindowCapturer};
use gpui::{
    App, AppContext, Bounds, DisplayId, Point, TitlebarOptions, Window, WindowBackgroundAppearance,
    WindowBounds, WindowDecorations, WindowKind, WindowOptions,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
    px, size,
};
use state::{
    DisplayTarget, OverlayWindowState, PanelLayout, SharedWorkerChannels, initial_snapshot,
};
use tray::ControlHub;
use ui::OverlayView;
use worker::spawn_worker;

pub fn resolve_game_display_id(
    cx: &App,
    placement: Option<&GameWindowPlacement>,
) -> Option<DisplayId> {
    let displays = cx.displays();
    if displays.is_empty() {
        return None;
    }
    if displays.len() == 1 {
        return Some(displays[0].id());
    }
    let p = placement?;
    let wcx = (p.x + (p.width as i32) / 2) as f32;
    let wcy = (p.y + (p.height as i32) / 2) as f32;

    let has_distinct_origins = displays.iter().any(|d| {
        let b = d.bounds();
        f32::from(b.origin.x).abs() > 1.0 || f32::from(b.origin.y).abs() > 1.0
    });

    if has_distinct_origins {
        for d in &displays {
            let b = d.bounds();
            let ox = f32::from(b.origin.x);
            let oy = f32::from(b.origin.y);
            let ow = f32::from(b.size.width);
            let oh = f32::from(b.size.height);
            if wcx >= ox && wcx < ox + ow && wcy >= oy && wcy < oy + oh {
                return Some(d.id());
            }
        }
        if let Some(rank) = p.monitor_spatial_rank {
            let mut sorted: Vec<_> = displays.iter().collect();
            sorted.sort_by_key(|d| {
                let b = d.bounds();
                (f32::from(b.origin.x) as i32, f32::from(b.origin.y) as i32)
            });
            if let Some(d) = sorted.get(rank) {
                return Some(d.id());
            }
        }
    }

    if let Some(mon) = p.monitor {
        let size_matches: Vec<_> = displays
            .iter()
            .filter(|d| {
                let b = d.bounds();
                (f32::from(b.size.width) as i32 - mon.width as i32).abs() <= 2
                    && (f32::from(b.size.height) as i32 - mon.height as i32).abs() <= 2
            })
            .collect();
        if size_matches.len() == 1 {
            return Some(size_matches[0].id());
        }
    }

    if let Some(idx) = p.monitor_index
        && let Some(d) = displays.get(idx)
    {
        return Some(d.id());
    }
    None
}

pub fn open_overlay_window(cx: &mut App, mut state: OverlayWindowState) {
    let bounds = Bounds {
        origin: Point {
            x: px(state.margin_left),
            y: px(state.margin_top),
        },
        size: size(px(380.0), px(740.0)),
    };

    let use_layer_shell = std::env::var("WAYLAND_DISPLAY").is_ok()
        && std::env::var("TARTARUS_NO_LAYER_SHELL").is_err();

    if use_layer_shell {
        let state_for_layer = state.clone();
        let layer_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            focus: false,
            show: true,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "tartarus-overlay".into(),
                layer: Layer::Overlay,
                anchor: Anchor::TOP | Anchor::LEFT,
                exclusive_zone: None,
                exclusive_edge: None,
                margin: Some((
                    px(state.margin_top),
                    px(0.0),
                    px(0.0),
                    px(state.margin_left),
                )),
                keyboard_interactivity: KeyboardInteractivity::None,
            }),
            is_movable: true,
            is_resizable: false,
            display_id: state.display_target.id(),
            window_background: WindowBackgroundAppearance::Transparent,
            window_decorations: Some(WindowDecorations::Client),
            app_id: Some("tartarus-overlay".into()),
            ..Default::default()
        };

        match cx.open_window(layer_options, move |window: &mut Window, cx: &mut App| {
            let mut s = state_for_layer;
            s.is_layer_shell = true;
            s.display_target
                .ensure_default(window.display(cx).map(|d| d.id()));
            cx.new(|cx| OverlayView::from_state(s, cx))
        }) {
            Ok(_) => {
                eprintln!(
                    "[tartarus-overlay] opened Wayland LayerShell overlay window (display_id={:?}, margin=({}, {}))",
                    state.display_target.id(),
                    state.margin_left,
                    state.margin_top
                );
                return;
            }
            Err(err) => {
                eprintln!(
                    "[tartarus-overlay] LayerShell window unavailable ({err:#}), falling back to Floating window"
                );
            }
        }
    }

    state.is_layer_shell = false;
    let state_for_float = state.clone();
    let floating_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Tartarus Overlay".into()),
            appears_transparent: true,
            ..Default::default()
        }),
        focus: false,
        show: true,
        kind: WindowKind::Floating,
        is_movable: true,
        is_resizable: true,
        display_id: state.display_target.id(),
        window_background: WindowBackgroundAppearance::Transparent,
        window_decorations: Some(WindowDecorations::Client),
        app_id: Some("tartarus-overlay".into()),
        ..Default::default()
    };

    match cx.open_window(
        floating_options,
        move |window: &mut Window, cx: &mut App| {
            window.activate_window();
            let mut s = state_for_float;
            s.display_target
                .ensure_default(window.display(cx).map(|d| d.id()));
            cx.new(|cx| OverlayView::from_state(s, cx))
        },
    ) {
        Ok(_) => {
            eprintln!(
                "[tartarus-overlay] opened Floating overlay window (display_id={:?})",
                state.display_target.id()
            );
        }
        Err(err) => {
            eprintln!(
                "[tartarus-overlay] Floating window failed ({err:#}), falling back to Normal window"
            );
            let normal_options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Tartarus Overlay".into()),
                    appears_transparent: false,
                    ..Default::default()
                }),
                focus: true,
                show: true,
                kind: WindowKind::Normal,
                is_movable: true,
                is_resizable: true,
                display_id: state.display_target.id(),
                window_background: WindowBackgroundAppearance::Opaque,
                window_decorations: Some(WindowDecorations::Server),
                app_id: Some("tartarus-overlay".into()),
                ..Default::default()
            };
            if let Err(err2) =
                cx.open_window(normal_options, move |window: &mut Window, cx: &mut App| {
                    window.activate_window();
                    let mut s = state;
                    s.display_target
                        .ensure_default(window.display(cx).map(|d| d.id()));
                    cx.new(|cx| OverlayView::from_state(s, cx))
                })
            {
                eprintln!("[tartarus-overlay] FATAL: failed to open any window: {err2:#}");
            } else {
                eprintln!("[tartarus-overlay] opened Normal fallback window");
            }
        }
    }
}

pub fn run_overlay() {
    #[cfg(target_os = "linux")]
    unsafe {
        // Lock glibc M_MMAP_THRESHOLD to 128 KB so dynamic threshold adjustment
        // is disabled and temporary large buffers are never retained in thread arenas.
        libc::mallopt(libc::M_MMAP_THRESHOLD, 128 * 1024);
    }
    rayon::broadcast(|_| {});

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .without_time()
        .init();

    eprintln!(
        "[tartarus-overlay] starting (WAYLAND_DISPLAY={:?}, DISPLAY={:?}, XDG_SESSION_TYPE={:?})",
        std::env::var("WAYLAND_DISPLAY").ok(),
        std::env::var("DISPLAY").ok(),
        std::env::var("XDG_SESSION_TYPE").ok(),
    );

    gpui_platform::application().run(|cx: &mut App| {
        let mut init_capturer = WindowCapturer::new();
        let initial_placement = init_capturer.game_window_placement();
        let target_display_id = resolve_game_display_id(cx, initial_placement.as_ref());

        let (worker_tx, worker_rx) = spawn_worker();
        let control_hub = ControlHub::init(worker_tx.clone());
        let shared_rx = Arc::new(Mutex::new(SharedWorkerChannels {
            worker_rx,
            control_hub,
        }));

        let state = OverlayWindowState {
            worker_tx,
            shared_rx,
            snap: initial_snapshot(),
            visible: true,
            layout: PanelLayout::Compact,
            selected_cell: 4,
            display_target: DisplayTarget::Auto(target_display_id),
            is_layer_shell: false,
            margin_top: 24.0,
            margin_left: 24.0,
        };

        open_overlay_window(cx, state);
    });
}

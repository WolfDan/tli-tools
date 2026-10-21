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
    App, AppContext, Bounds, DisplayId, Pixels, Point, TitlebarOptions, Window,
    WindowBackgroundAppearance, WindowBounds, WindowDecorations, WindowKind, WindowOptions,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
    px, size,
};
use state::{
    DisplayTarget, OverlayWindowState, PanelLayout, SharedWorkerChannels, initial_snapshot,
};
use tray::ControlHub;
use ui::OverlayView;
use worker::spawn_worker;

const OVERLAY_WIDTH_PX: f32 = 380.0;
const OVERLAY_HEIGHT_PX: f32 = 740.0;
const MMAP_THRESHOLD_BYTES: i32 = 128 * 1024;

const _: () = {
    assert!(OVERLAY_WIDTH_PX > 0.0);
    assert!(OVERLAY_HEIGHT_PX > 0.0);
    assert!(MMAP_THRESHOLD_BYTES == 131_072);
};

fn match_display_by_size(
    displays: &[std::rc::Rc<dyn gpui::PlatformDisplay>],
    mon_width: u32,
    mon_height: u32,
) -> Option<DisplayId> {
    assert!(mon_width > 0);
    assert!(mon_height > 0);
    let size_matches: Vec<_> = displays
        .iter()
        .filter(|d| {
            let b = d.bounds();
            (f32::from(b.size.width) as i32 - mon_width as i32).abs() <= 2
                && (f32::from(b.size.height) as i32 - mon_height as i32).abs() <= 2
        })
        .collect();
    if size_matches.len() == 1 {
        Some(size_matches[0].id())
    } else {
        None
    }
}

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
    assert!(p.width > 0 && p.height > 0);
    let wcx = (p.x + (p.width as i32) / 2) as f32;
    let wcy = (p.y + (p.height as i32) / 2) as f32;
    assert!(wcx.is_finite() && wcy.is_finite());

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

    if let Some(mon) = p.monitor
        && let Some(id) = match_display_by_size(&displays, mon.width, mon.height)
    {
        return Some(id);
    }

    if let Some(idx) = p.monitor_index
        && let Some(d) = displays.get(idx)
    {
        return Some(d.id());
    }
    None
}

fn try_open_layer_shell_window(
    cx: &mut App,
    state: OverlayWindowState,
    bounds: Bounds<Pixels>,
) -> bool {
    assert!(state.margin_left.is_finite());
    assert!(state.margin_top.is_finite());
    let display_id = state.display_target.id();
    let margin_left = state.margin_left;
    let margin_top = state.margin_top;
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
            margin: Some((px(margin_top), px(0.0), px(0.0), px(margin_left))),
            keyboard_interactivity: KeyboardInteractivity::None,
        }),
        is_movable: true,
        is_resizable: false,
        display_id,
        window_background: WindowBackgroundAppearance::Transparent,
        window_decorations: Some(WindowDecorations::Client),
        app_id: Some("tartarus-overlay".into()),
        ..Default::default()
    };

    match cx.open_window(layer_options, move |window: &mut Window, cx: &mut App| {
        let mut s = state;
        s.is_layer_shell = true;
        s.display_target
            .ensure_default(window.display(cx).map(|d| d.id()));
        cx.new(|cx| OverlayView::from_state(s, cx))
    }) {
        Ok(_) => {
            eprintln!(
                "[tartarus-overlay] opened Wayland LayerShell window (display={display_id:?}, \
                 margin=({margin_left}, {margin_top}))",
            );
            true
        }
        Err(err) => {
            eprintln!(
                "[tartarus-overlay] LayerShell unavailable ({err:#}), falling back to Floating"
            );
            false
        }
    }
}

fn try_open_floating_window(
    cx: &mut App,
    state: OverlayWindowState,
    bounds: Bounds<Pixels>,
) -> bool {
    assert!(!state.is_layer_shell);
    assert!(state.selected_cell < 16);
    let display_id = state.display_target.id();
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
        display_id,
        window_background: WindowBackgroundAppearance::Transparent,
        window_decorations: Some(WindowDecorations::Client),
        app_id: Some("tartarus-overlay".into()),
        ..Default::default()
    };

    match cx.open_window(
        floating_options,
        move |window: &mut Window, cx: &mut App| {
            window.activate_window();
            let mut s = state;
            s.display_target
                .ensure_default(window.display(cx).map(|d| d.id()));
            cx.new(|cx| OverlayView::from_state(s, cx))
        },
    ) {
        Ok(_) => {
            eprintln!(
                "[tartarus-overlay] opened Floating overlay window (display_id={display_id:?})"
            );
            true
        }
        Err(err) => {
            eprintln!(
                "[tartarus-overlay] Floating window failed ({err:#}), falling back to Normal"
            );
            false
        }
    }
}

fn open_normal_fallback_window(cx: &mut App, state: OverlayWindowState, bounds: Bounds<Pixels>) {
    assert!(!state.is_layer_shell);
    assert!(state.selected_cell < 16);
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
    if let Err(err2) = cx.open_window(normal_options, move |window: &mut Window, cx: &mut App| {
        window.activate_window();
        let mut s = state;
        s.display_target
            .ensure_default(window.display(cx).map(|d| d.id()));
        cx.new(|cx| OverlayView::from_state(s, cx))
    }) {
        eprintln!("[tartarus-overlay] FATAL: failed to open any window: {err2:#}");
    } else {
        eprintln!("[tartarus-overlay] opened Normal fallback window");
    }
}

pub fn open_overlay_window(cx: &mut App, mut state: OverlayWindowState) {
    assert!(state.margin_left.is_finite() && state.margin_top.is_finite());
    assert!(state.selected_cell < 16);

    let bounds = Bounds {
        origin: Point {
            x: px(state.margin_left),
            y: px(state.margin_top),
        },
        size: size(px(OVERLAY_WIDTH_PX), px(OVERLAY_HEIGHT_PX)),
    };

    let use_layer_shell = std::env::var("WAYLAND_DISPLAY").is_ok()
        && std::env::var("TARTARUS_NO_LAYER_SHELL").is_err();

    if use_layer_shell && try_open_layer_shell_window(cx, state.clone(), bounds) {
        return;
    }

    state.is_layer_shell = false;
    if !try_open_floating_window(cx, state.clone(), bounds) {
        open_normal_fallback_window(cx, state, bounds);
    }
}

pub fn run_overlay() {
    let init_rss = state::read_process_rss_mb();
    assert!(init_rss >= 0.0);
    assert!(init_rss.is_finite());

    #[cfg(target_os = "linux")]
    unsafe {
        // Lock glibc M_MMAP_THRESHOLD to 128 KB so dynamic threshold adjustment
        // is disabled and temporary large buffers are never retained in thread arenas.
        libc::mallopt(libc::M_MMAP_THRESHOLD, MMAP_THRESHOLD_BYTES);
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
        assert!(state.selected_cell < 16);
        assert!(state.margin_top >= 0.0 && state.margin_left >= 0.0);

        open_overlay_window(cx, state);
    });
}

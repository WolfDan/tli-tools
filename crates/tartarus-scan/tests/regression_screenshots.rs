mod common;

use common::{fixture_path, load_png_rgba8};
use tartarus_core::{BoardStep, CellKind, ObjectiveId, StartEventId, ToolId};
use tartarus_scan::{
    CaptureTrigger, ImageView, RunTracker, ScanArena, ScanOptions, ScannedCellKind, TitleRead,
    read_objective_title_with_slot, scan_image_with_arena, scan_image_with_options_and_arena,
    scan_pick_with_slot,
};

#[test]
fn test_board_cell06_and_cell10_warden_localization() {
    let mut arena = ScanArena::new();

    // Cell 06 (0-indexed 5)
    let (w6, h6, rgba6) = load_png_rgba8(&fixture_path("board_cell06_warden.png"));
    let view6 = ImageView {
        width: w6,
        height: h6,
        data: &rgba6,
    };
    let res6 = scan_image_with_arena(&view6, None, &mut arena);
    assert!(
        res6.ok,
        "board must be localized when warden is at cell 06 (idx 5)"
    );
    assert_eq!(
        res6.warden.map(|w| w.cell),
        Some(5),
        "warden must be detected at cell 06 (0-indexed 5)"
    );

    // Cell 10 (0-indexed 9)
    let (w10, h10, rgba10) = load_png_rgba8(&fixture_path("board_cell10_warden.png"));
    let view10 = ImageView {
        width: w10,
        height: h10,
        data: &rgba10,
    };
    let res10 = scan_image_with_arena(&view10, None, &mut arena);
    assert!(
        res10.ok,
        "board must be localized when warden is at cell 10 (idx 9)"
    );
    assert_eq!(
        res10.warden.map(|w| w.cell),
        Some(9),
        "warden must be detected at cell 10 (0-indexed 9)"
    );
}

#[test]
fn test_chain_consume_screens_never_recommend_end_day() {
    let mut arena = ScanArena::new();
    for filename in [
        "chain_consume_targets.png",
        "chain_consume_end_day_prompt.png",
    ] {
        let (w, h, rgba) = load_png_rgba8(&fixture_path(filename));
        let view = ImageView {
            width: w,
            height: h,
            data: &rgba,
        };
        let mut tracker = RunTracker::new();
        let (ok, _, _) = tracker.process_frame(&view, CaptureTrigger::Manual, &mut arena);
        assert!(
            ok,
            "{filename}: should recognize board / chain consume screen"
        );
        assert!(
            tracker.model.step == BoardStep::Chain || tracker.model.step == BoardStep::Manage,
            "{filename}: expected Chain or Manage step, got {:?}",
            tracker.model.step
        );
        let snap = tracker.model.evaluate(4000, 80, 42);
        assert!(
            !snap.move_text.to_lowercase().contains("end the day"),
            "{filename}: chain screen must never recommend 'End the day', got '{}'",
            snap.move_text
        );
    }
}

fn assert_pick_three_relics(arena: &mut ScanArena) {
    let (w1, h1, rgba1) =
        load_png_rgba8(&fixture_path("pick_three_relics_pestle_dogtag_noose.png"));
    let view1 = ImageView {
        width: w1,
        height: h1,
        data: &rgba1,
    };
    let pick1 = scan_pick_with_slot(&view1, &mut arena.slots[0]).expect("pick1 detected");
    let tools1: Vec<ToolId> = pick1.cards.iter().map(|c| c.sim).collect();
    assert_eq!(
        tools1,
        vec![ToolId::Pestle, ToolId::Dogtag, ToolId::Rope],
        "expected [Iron Pestle, Felon's Dog Tag, Hangman's Noose]"
    );

    let mut tracker1 = RunTracker::new();
    let (ok1, _, _) =
        tracker1.process_frame(&view1, CaptureTrigger::Instruments { fresh: true }, arena);
    assert!(ok1, "pick1 should be processed by RunTracker");
    let snap1 = tracker1.model.evaluate(4000, 80, 42);
    assert!(
        !snap1.move_text.contains("Skewered Rat Heads"),
        "must never recommend an unoffered relic, got '{}'",
        snap1.move_text
    );
    assert!(
        snap1.move_text.contains("Iron Pestle")
            || snap1.move_text.contains("Felon's Dog Tag")
            || snap1.move_text.contains("Hangman's Noose")
            || snap1.move_text.to_lowercase().contains("reroll"),
        "expected one of the 3 offered relics, got '{}'",
        snap1.move_text
    );
}

fn assert_scroll_popups(arena: &mut ScanArena) {
    let (w2, h2, rgba2) = load_png_rgba8(&fixture_path("start_event_tyranny_popup.png"));
    let view2 = ImageView {
        width: w2,
        height: h2,
        data: &rgba2,
    };
    let title = read_objective_title_with_slot(&view2, &mut arena.slots[0]);
    assert!(
        matches!(title, Some(TitleRead::StartEvent(StartEventId::Tyranny))),
        "expected StartEvent(Tyranny) from start_event_tyranny_popup.png, got {:?}",
        title
    );
    let mut tracker2 = RunTracker::new();
    let (ok2, status2, _) = tracker2.process_frame(&view2, CaptureTrigger::Manual, arena);
    assert!(
        ok2,
        "Manual Picture capture on Tyranny popup must be recognized"
    );
    assert_eq!(tracker2.model.start_event, StartEventId::Tyranny);
    assert!(
        status2.as_deref().unwrap_or("").contains("Tyranny"),
        "expected status to mention Tyranny, got {:?}",
        status2
    );

    let (w3, h3, rgba3) = load_png_rgba8(&fixture_path("objective_behind_cell_doors_popup.png"));
    let view3 = ImageView {
        width: w3,
        height: h3,
        data: &rgba3,
    };
    let title3 = read_objective_title_with_slot(&view3, &mut arena.slots[0]);
    assert!(
        matches!(title3, Some(TitleRead::Objective(ObjectiveId::Doors))),
        "expected Objective(Doors) from objective_behind_cell_doors_popup.png, got {:?}",
        title3
    );
    let mut tracker3 = RunTracker::new();
    let (ok3, status3, _) = tracker3.process_frame(&view3, CaptureTrigger::Manual, arena);
    assert!(
        ok3,
        "Manual Picture capture on Behind Cell Doors popup must be recognized"
    );
    assert_eq!(tracker3.model.objective, ObjectiveId::Doors);
    assert!(
        status3
            .as_deref()
            .unwrap_or("")
            .contains("Behind Cell Doors"),
        "expected status to mention Behind Cell Doors, got {:?}",
        status3
    );
}

#[test]
fn test_pick_three_relics_and_start_event_popup() {
    let mut arena = ScanArena::new();
    assert!(!arena.slots.is_empty());
    assert_pick_three_relics(&mut arena);
    assert_scroll_popups(&mut arena);
    assert_eq!(arena.slots.len(), 6);
}

fn assert_day5_and_popup_step(tracker: &mut RunTracker, arena: &mut ScanArena) {
    let (w5, h5, rgba5) = load_png_rgba8(&fixture_path("board_day5_supreme_lion_no_jaws.png"));
    let (wp, hp, rgbap) = load_png_rgba8(&fixture_path("start_event_tyranny_popup.png"));
    let view5 = ImageView {
        width: w5,
        height: h5,
        data: &rgba5,
    };
    let viewp = ImageView {
        width: wp,
        height: hp,
        data: &rgbap,
    };

    let scan5 = scan_image_with_arena(&view5, None, arena);
    assert!(scan5.ok, "Day 5 board must localize");
    assert_eq!(
        scan5.days.map(|(d, _)| d),
        Some(15),
        "Day 5 screenshot shows '15 day(s) until Riot'"
    );
    assert_eq!(
        scan5.chromatic,
        Some(0),
        "Day 5 screenshot shows 0 Chromatic Prisoners"
    );
    let cells5 = scan5.cells.expect("Day 5 cells");
    assert_eq!(cells5[9].kind, ScannedCellKind::Inmate);
    assert_eq!(cells5[9].tier, 4, "Cell 10 (idx 9) is a Supreme Lion");

    let (ok5, _, _) = tracker.process_frame(
        &view5,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        arena,
    );
    assert!(ok5);
    assert_eq!(
        tracker.model.checks, 0,
        "Day 5 board should have 0 cells to check, got mask {:#06x}",
        tracker.model.checks
    );
    let (_, probs5) = tracker.model.build_state(42);
    assert!(
        probs5.is_empty(),
        "Day 5 board should have 0 validation problems, got {:?}",
        probs5
    );

    let (okp, _, _) = tracker.process_frame(&viewp, CaptureTrigger::Objective, arena);
    assert!(okp);
    assert_eq!(tracker.model.start_event, StartEventId::Tyranny);
    assert_eq!(
        tracker.model.checks, 0,
        "Start event popup must not corrupt underlying board cells"
    );
}

fn assert_day13_step(tracker: &mut RunTracker, arena: &mut ScanArena) {
    let (w13, h13, rgba13) =
        load_png_rgba8(&fixture_path("board_day13_supreme_jaws_chromatic7.png"));
    let view13 = ImageView {
        width: w13,
        height: h13,
        data: &rgba13,
    };

    tracker.hint = None;
    let scan13 = scan_image_with_arena(&view13, None, arena);
    assert!(scan13.ok, "Day 13 board must localize");
    assert_eq!(
        scan13.days.map(|(d, _)| d),
        Some(7),
        "Day 13 screenshot shows '7 day(s) until Riot'"
    );
    assert_eq!(
        scan13.chromatic,
        Some(7),
        "Day 13 screenshot shows 7 Chromatic Prisoners"
    );
    let cells13 = scan13.cells.expect("Day 13 cells");
    assert!(
        cells13[3].jaws,
        "Cell 04 (idx 3) on Day 13 has a jaws badge"
    );
    assert_eq!(cells13[9].tier, 4);
    assert!(
        cells13[9].jaws,
        "Cell 10 (idx 9) Supreme Lion on Day 13 has a jaws badge"
    );
    assert_eq!(cells13[14].tier, 4);
    assert!(
        !cells13[14].jaws,
        "Cell 15 (idx 14) Supreme Wolf on Day 13 does NOT have a jaws badge"
    );

    let (ok13, _, _) = tracker.process_frame(
        &view13,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        arena,
    );
    assert!(ok13);
    assert_eq!(tracker.model.cells[3].kind, CellKind::Inmate);
    assert_eq!(tracker.model.cells[9].kind, CellKind::Inmate);
    assert_eq!(tracker.model.cells[9].tier, 4);
    assert_eq!(tracker.model.cells[14].kind, CellKind::Inmate);
    assert_eq!(tracker.model.cells[14].tier, 4);
    assert_eq!(
        tracker.model.checks, 0,
        "Day 13 board should have 0 cells to check, got mask {:#06x}",
        tracker.model.checks
    );
    let (_, probs13) = tracker.model.build_state(42);
    assert!(
        probs13.is_empty(),
        "Day 13 board should have 0 validation problems, got {:?}",
        probs13
    );
}

#[test]
fn test_day5_and_day13_supreme_jaws_chromatic_and_sequence() {
    let mut arena = ScanArena::new();
    let mut tracker = RunTracker::new();
    assert_day5_and_popup_step(&mut tracker, &mut arena);
    assert_day13_step(&mut tracker, &mut arena);
    assert_eq!(tracker.model.vault.len(), 7);
    assert_eq!(tracker.model.checks, 0);
}

#[test]
fn test_phantom_tool_slot_regression() {
    let (w, h, rgba) = load_png_rgba8(&fixture_path("phantom_slot.png"));
    assert!(w > 0 && h > 0);
    let view = ImageView {
        width: w,
        height: h,
        data: &rgba,
    };
    let mut arena = ScanArena::new();
    let res = scan_image_with_arena(&view, None, &mut arena);
    assert!(
        res.tools.len() <= 2,
        "phantom_slot.png must not detect phantom tool slots, got {:?}",
        res.tools
    );
}

#[test]
fn test_cell06_above_supreme_jaws_cell07() {
    let (w, h, rgba) = load_png_rgba8(&fixture_path("board_day2_cell06_check.png"));
    let view = ImageView {
        width: w,
        height: h,
        data: &rgba,
    };
    let mut arena = ScanArena::new();
    let scan = scan_image_with_arena(&view, None, &mut arena);
    assert!(scan.ok, "board_day2_cell06_check.png must localize");
    let cells = scan.cells.expect("cells");
    assert_eq!(
        cells[5].kind,
        ScannedCellKind::Locked,
        "Cell 06 (idx 5) must be Locked, got {:?}",
        cells[5]
    );
    assert!(
        !cells[5].nest,
        "Cell 06 (idx 5) must not have a false nest mark, got {:?}",
        cells[5]
    );
    assert!(
        cells[5].conf >= 0.85,
        "Cell 06 (idx 5) lock confidence must be >= 0.85, got {:?}",
        cells[5]
    );
    assert_eq!(cells[6].kind, ScannedCellKind::Inmate);
    assert_eq!(cells[6].tier, 4, "Cell 07 (idx 6) is a Supreme Mammoth");
    assert!(cells[6].jaws, "Cell 07 (idx 6) has a red jaws badge");

    let mut tracker = RunTracker::new();
    let (ok, _, _) = tracker.process_frame(
        &view,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        &mut arena,
    );
    assert!(ok);
    assert_eq!(
        tracker.model.checks, 0,
        "board_day2_cell06_check.png must have 0 cells to check, got mask {:#06x}",
        tracker.model.checks
    );
}

fn assert_office_full_frame(rgba: &[u8], w: usize, h: usize, arena: &mut ScanArena) {
    let view = ImageView {
        width: w,
        height: h,
        data: rgba,
    };
    let scan = scan_image_with_arena(&view, None, arena);
    assert!(
        scan.ok,
        "board_day4_office_instigate_bestow.png must localize"
    );
    assert_eq!(scan.warden.map(|w| w.cell), Some(0));
    assert_eq!(
        scan.decision.map(|d| d.office),
        Some(true),
        "expected office=true on Instigate/Bestow screen, got {:?}",
        scan.decision
    );

    let mut tracker = RunTracker::new();
    let (ok, _, _) = tracker.process_frame(
        &view,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        arena,
    );
    assert!(ok);
    assert!(tracker.last_board_had_cards);
    assert_eq!(tracker.model.warden, 0);
    assert_eq!(tracker.model.step, BoardStep::Office);
    assert_eq!(tracker.model.checks, 0);
    let advice = tracker.model.evaluate(4000, 80, 42);
    assert!(
        advice.move_text.contains("Instigate") || advice.move_text.contains("Bestow"),
        "expected Instigate or Bestow recommendation at Office, got '{}'",
        advice.move_text
    );
}

fn assert_office_early_frame(rgba: &[u8], w: usize, h: usize, arena: &mut ScanArena) {
    let mut early_rgba = rgba.to_vec();
    let y_cut = (h * 70) / 100;
    for y in y_cut..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            early_rgba[idx] /= 4;
            early_rgba[idx + 1] /= 4;
            early_rgba[idx + 2] /= 4;
        }
    }
    let early_view = ImageView {
        width: w,
        height: h,
        data: &early_rgba,
    };
    let early_scan = scan_image_with_arena(&early_view, None, arena);
    assert!(early_scan.ok);
    assert!(
        early_scan.decision.is_none(),
        "buttonless Make Your Decision header must return decision=None, got {:?}",
        early_scan.decision
    );
    let mut early_tracker = RunTracker::new();
    let (ok_early, _, _) = early_tracker.process_frame(
        &early_view,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        arena,
    );
    assert!(ok_early);
    assert!(
        !early_tracker.last_board_had_cards,
        "early frame without decision cards must set last_board_had_cards=false"
    );
    assert_eq!(
        early_tracker.model.step,
        BoardStep::Office,
        "arriving at OFFICE without decision cards must default to BoardStep::Office"
    );
    let early_advice = early_tracker.model.evaluate(4000, 80, 42);
    assert!(
        early_advice.move_text.contains("Instigate") || early_advice.move_text.contains("Bestow"),
        "even on an early frame at OFFICE, must recommend Instigate or Bestow, got '{}'",
        early_advice.move_text
    );
}

#[test]
fn test_office_instigate_bestow_decision() {
    let (w, h, rgba) = load_png_rgba8(&fixture_path("board_day4_office_instigate_bestow.png"));
    assert!(w > 0 && h > 0);
    let mut arena = ScanArena::new();
    assert_office_full_frame(&rgba, w, h, &mut arena);
    assert_office_early_frame(&rgba, w, h, &mut arena);
    assert_eq!(arena.slots.len(), 6);
}

fn bench_single_fixture(name: &str, arena: &mut ScanArena) -> [f64; 6] {
    let (fw, fh, frgba) = load_png_rgba8(&fixture_path(name));
    let fview = ImageView {
        width: fw,
        height: fh,
        data: &frgba,
    };
    let warm = scan_image_with_arena(&fview, None, arena);
    assert!(warm.ok, "Fixture {name} should scan ok");
    let hint = warm.loc;
    assert!(hint.is_some());

    let iters = 5usize;
    let mut loc_ms = 0.0;
    let mut cells_ms = 0.0;
    let mut ocr_ms = 0.0;
    let mut ms_total = 0.0;
    let mut ocr_skip_ms = 0.0;
    let mut skip_ms_total = 0.0;
    for _ in 0..iters {
        let res = scan_image_with_arena(&fview, hint, arena);
        loc_ms += res.profile.loc_ms;
        cells_ms += res.profile.cells_ms;
        ocr_ms += res.profile.ocr_ms;
        ms_total += res.profile.ms_total;

        let res_skip =
            scan_image_with_options_and_arena(&fview, hint, ScanOptions { skip_days: true }, arena);
        ocr_skip_ms += res_skip.profile.ocr_ms;
        skip_ms_total += res_skip.profile.ms_total;
    }
    let k = iters as f64;
    let out = [
        loc_ms / k,
        cells_ms / k,
        ocr_ms / k,
        ms_total / k,
        ocr_skip_ms / k,
        skip_ms_total / k,
    ];
    println!(
        "Scan [{name:<30}]: loc={:5.2}ms cells={:5.2}ms ocr={:5.2}ms \
         (skip_days={:5.2}ms) total={:5.2}ms (skip_days={:5.2}ms)",
        out[0], out[1], out[2], out[4], out[3], out[5]
    );
    out
}

#[test]
fn test_hud_objective_and_scan_latency_benchmark() {
    let mut arena = ScanArena::new();

    let (w, h, rgba) = load_png_rgba8(&fixture_path("board_cell06_warden.png"));
    let view = ImageView {
        width: w,
        height: h,
        data: &rgba,
    };
    let mut tracker = RunTracker::new();
    let (ok, _, _) = tracker.process_frame(
        &view,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        &mut arena,
    );
    assert!(ok);
    assert_eq!(
        tracker.model.objective,
        ObjectiveId::Walk,
        "Expected HUD objective fallback to recover ObjectiveId::Walk"
    );

    let fixtures = [
        "board_cell06_warden.png",
        "board_cell10_warden.png",
        "board_day5_supreme_lion_no_jaws.png",
        "board_day13_supreme_jaws_chromatic7.png",
        "board_day2_cell06_check.png",
        "board_day4_office_instigate_bestow.png",
        "chain_consume_targets.png",
        "chain_consume_end_day_prompt.png",
    ];
    let mut sums = [0.0_f64; 6];
    for name in fixtures {
        let m = bench_single_fixture(name, &mut arena);
        for i in 0..6 {
            sums[i] += m[i];
        }
    }
    let n = fixtures.len() as f64;
    println!(
        "Scan Average (8 board fixtures): loc={:.2}ms cells={:.2}ms ocr={:.2}ms \
         (skip_days={:.2}ms) total={:.2}ms (skip_days={:.2}ms)",
        sums[0] / n,
        sums[1] / n,
        sums[2] / n,
        sums[4] / n,
        sums[3] / n,
        sums[5] / n
    );
}

fn inspect_single_user_screenshot(name: &str, arena: &mut ScanArena) {
    let path = fixture_path(name);
    let (w, h, rgba) = load_png_rgba8(&path);
    assert!(w > 0 && h > 0);
    let view = ImageView {
        width: w,
        height: h,
        data: &rgba,
    };
    let res = scan_image_with_arena(&view, None, arena);
    assert!(res.ok);
    println!("\n=== {name} ({w}x{h}) ===");
    println!(
        "profile: loc={:.2}ms cells={:.2}ms ocr={:.2}ms total={:.2}ms",
        res.profile.loc_ms, res.profile.cells_ms, res.profile.ocr_ms, res.profile.ms_total
    );
    println!(
        "warden={:?} days={:?} chromatic={:?}",
        res.warden, res.days, res.chromatic
    );
    if let Some(loc) = res.loc {
        let (t_list, lbl) = tartarus_scan::read_tools_with_slot(&view, &loc, &mut arena.slots[0]);
        println!("read_tools label={:?} tools={:?}", lbl, t_list.as_slice());
    }
    if let Some(ref cells) = res.cells {
        for (idx, c) in cells.iter().enumerate() {
            if c.kind != ScannedCellKind::Locked {
                println!(
                    "  cell {:02} (idx {idx}): kind={:?} beast={:?} tier={} \
                     conf={:.2} jaws={} ghost={} event={}",
                    idx + 1,
                    c.kind,
                    c.beast,
                    c.tier,
                    c.conf,
                    c.jaws,
                    c.ghost,
                    c.event
                );
            }
        }
    }
    let mut tracker = RunTracker::new();
    tracker.ingest(&res);
    let snap = tracker.model.evaluate(96_000, 85, 42);
    println!(
        "move_text='{}' elapsed={}ms",
        snap.move_text, snap.elapsed_ms
    );
}

fn assert_seq_day3_and_day4(seq_tracker: &mut RunTracker, arena: &mut ScanArena) {
    let scan_limit_ms = if cfg!(debug_assertions) { 50.0 } else { 5.0 };
    let solve_limit_ms = if cfg!(debug_assertions) { 500 } else { 50 };
    seq_tracker.log_days = Some((3, 20));
    let (w1, h1, rgba1) = load_png_rgba8(&fixture_path("board_day3_doors_7_11_12.png"));
    let view1 = ImageView {
        width: w1,
        height: h1,
        data: &rgba1,
    };
    let (ok1, _, prof1) = seq_tracker.process_frame(
        &view1,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        arena,
    );
    assert!(ok1);
    assert_eq!(seq_tracker.model.tools.as_slice(), &[ToolId::Crown]);
    assert!(
        prof1.ms_total < scan_limit_ms,
        "expected scan < {scan_limit_ms}ms, got {:.2}ms",
        prof1.ms_total
    );
    let snap1 = seq_tracker.model.evaluate(96_000, 85, 42);
    assert!(
        snap1.elapsed_ms < solve_limit_ms,
        "expected solve < {solve_limit_ms}ms, got {}ms",
        snap1.elapsed_ms
    );
    assert_eq!(snap1.options.len(), 3);

    seq_tracker.log_days = Some((4, 20));
    let (w2, h2, rgba2) = load_png_rgba8(&fixture_path("board_day4_doors_15_16_office.png"));
    let view2 = ImageView {
        width: w2,
        height: h2,
        data: &rgba2,
    };
    let (ok2, _, prof2) = seq_tracker.process_frame(
        &view2,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        arena,
    );
    assert!(ok2);
    assert_eq!(seq_tracker.model.tools.as_slice(), &[ToolId::Crown]);
    assert!(
        prof2.ms_total < scan_limit_ms,
        "expected scan < {scan_limit_ms}ms, got {:.2}ms",
        prof2.ms_total
    );
    let snap2 = seq_tracker.model.evaluate(96_000, 85, 42);
    assert_eq!(snap2.options.len(), 3);
    assert!(
        snap2
            .options
            .iter()
            .any(|o| o.label.contains("Interrogation Room")),
        "expected Interrogation Room in options: {:?}",
        snap2.options
    );
    assert!(snap2.move_text.contains("Interrogation Room"));
}

fn assert_seq_day11(seq_tracker: &mut RunTracker, arena: &mut ScanArena) {
    let scan_limit_ms = if cfg!(debug_assertions) { 50.0 } else { 5.0 };
    seq_tracker.log_days = Some((11, 20));
    let (w3, h3, rgba3) = load_png_rgba8(&fixture_path("board_day11_cell11_lion_hunt_replace.png"));
    let view3 = ImageView {
        width: w3,
        height: h3,
        data: &rgba3,
    };
    let (ok3, _, prof3) = seq_tracker.process_frame(
        &view3,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        arena,
    );
    assert!(ok3);
    assert_eq!(
        seq_tracker.model.tools.as_slice(),
        &[ToolId::Crown, ToolId::Trap],
        "expected mid-run Trap instrument to be recognized alongside Crown"
    );
    assert!(
        !seq_tracker.model.no_swap,
        "expected Replace to be available"
    );
    assert!(
        !seq_tracker.model.no_devour,
        "expected Consume to be available"
    );
    assert!(
        prof3.ms_total < scan_limit_ms,
        "expected scan < {scan_limit_ms}ms, got {:.2}ms",
        prof3.ms_total
    );
    let snap3 = seq_tracker.model.evaluate(96_000, 85, 42);
    assert_eq!(
        snap3.options.len(),
        4,
        "expected Hunt, Replace, Consume, and End options"
    );
    assert!(snap3.options.iter().any(|o| o.label.starts_with("Hunt:")));
    assert!(
        snap3
            .options
            .iter()
            .any(|o| o.label.starts_with("Replace "))
    );
    assert!(
        snap3
            .options
            .iter()
            .any(|o| o.label.starts_with("Consume "))
    );
    assert!(snap3.options.iter().any(|o| o.label == "End the day"));
}

#[test]
fn test_user_three_screenshots_office_door_and_lion_hunt_replace() {
    let names = [
        "board_day3_doors_7_11_12.png",
        "board_day4_doors_15_16_office.png",
        "board_day11_cell11_lion_hunt_replace.png",
    ];
    let mut arena = ScanArena::new();
    for name in names {
        inspect_single_user_screenshot(name, &mut arena);
    }

    let mut seq_tracker = RunTracker::new();
    assert_seq_day3_and_day4(&mut seq_tracker, &mut arena);
    assert_seq_day11(&mut seq_tracker, &mut arena);
    assert_eq!(seq_tracker.model.day, 11);
    assert_eq!(seq_tracker.model.tools.len(), 2);
}

mod common;

use common::{fixture_path, load_png_rgba8};
use tartarus_core::{BoardStep, CellKind, ObjectiveId, StartEventId, ToolId};
use tartarus_scan::{
    CaptureTrigger, CellKindRead, FrameView, ReadBoardOptions, RunTracker, TitleModalRead, UiReader,
};

fn make_frame(w: usize, h: usize, rgba: &[u8]) -> FrameView<'_> {
    assert!(w > 0 && h > 0);
    assert_eq!(rgba.len(), w * h * 4);
    FrameView::new(rgba, w as u32, h as u32)
}

#[test]
fn test_board_cell06_and_cell10_warden_localization() {
    let mut reader = UiReader::new();

    // Cell 06 (0-indexed 5)
    let (w6, h6, rgba6) = load_png_rgba8(&fixture_path("board_cell06_warden.png"));
    let view6 = make_frame(w6, h6, &rgba6);
    let res6 = reader.read_board(&view6, None, ReadBoardOptions::default());
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
    let view10 = make_frame(w10, h10, &rgba10);
    let res10 = reader.read_board(&view10, None, ReadBoardOptions::default());
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
    let mut reader = UiReader::new();
    for filename in [
        "chain_consume_targets.png",
        "chain_consume_end_day_prompt.png",
    ] {
        let (w, h, rgba) = load_png_rgba8(&fixture_path(filename));
        let view = make_frame(w, h, &rgba);
        let mut tracker = RunTracker::new();
        let (ok, _, _) = tracker.process_frame(&view, CaptureTrigger::Manual, &mut reader);
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

fn assert_pick_three_relics(reader: &mut UiReader) {
    let (w1, h1, rgba1) =
        load_png_rgba8(&fixture_path("pick_three_relics_pestle_dogtag_noose.png"));
    let view1 = make_frame(w1, h1, &rgba1);
    let pick1 = reader.read_relic_modal(&view1).expect("pick1 detected");
    let tools1: Vec<ToolId> = pick1.cards.iter().map(|c| c.sim).collect();
    assert_eq!(
        tools1,
        vec![ToolId::Pestle, ToolId::Dogtag, ToolId::Rope],
        "expected [Iron Pestle, Felon's Dog Tag, Hangman's Noose]"
    );

    let mut tracker1 = RunTracker::new();
    let (ok1, _, _) =
        tracker1.process_frame(&view1, CaptureTrigger::Instruments { fresh: true }, reader);
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

    let (w2, h2, rgba2) = load_png_rgba8(&fixture_path("pick_three_relics_dogtag_cage_ratbox.png"));
    let view2 = make_frame(w2, h2, &rgba2);
    let t_pick2 = std::time::Instant::now();
    let pick2 = reader.read_relic_modal(&view2).expect("pick2 detected");
    let pick2_ms = t_pick2.elapsed().as_secs_f64() * 1000.0;
    let tools2: Vec<ToolId> = pick2.cards.iter().map(|c| c.sim).collect();
    assert_eq!(
        tools2,
        vec![ToolId::Dogtag, ToolId::Cage, ToolId::Ironbox],
        "expected [Felon's Dog Tag, Crowded Beastcage, Squirming Iron Box: Colossal Rat]"
    );
    println!("pick2 ({w2}x{h2}): tools={tools2:?} ms={pick2_ms:.2}ms");
}

fn assert_scroll_popups(reader: &mut UiReader) {
    let (w2, h2, rgba2) = load_png_rgba8(&fixture_path("start_event_tyranny_popup.png"));
    let view2 = make_frame(w2, h2, &rgba2);
    let title = reader.read_title_modal(&view2);
    assert!(
        matches!(
            title,
            Some(TitleModalRead::StartEvent(StartEventId::Tyranny))
        ),
        "expected StartEvent(Tyranny) from start_event_tyranny_popup.png, got {:?}",
        title
    );
    let mut tracker2 = RunTracker::new();
    let (ok2, status2, _) = tracker2.process_frame(&view2, CaptureTrigger::Manual, reader);
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
    let view3 = make_frame(w3, h3, &rgba3);
    let title3 = reader.read_title_modal(&view3);
    assert!(
        matches!(title3, Some(TitleModalRead::Objective(ObjectiveId::Doors))),
        "expected Objective(Doors) from objective_behind_cell_doors_popup.png, got {:?}",
        title3
    );
    let mut tracker3 = RunTracker::new();
    let (ok3, status3, _) = tracker3.process_frame(&view3, CaptureTrigger::Manual, reader);
    assert!(
        ok3,
        "Manual Picture capture on Behind Cell Doors popup must be recognized"
    );
    assert_eq!(tracker3.model.objective, ObjectiveId::Doors);
    assert!(
        status3
            .as_deref()
            .unwrap_or("")
            .contains("Behind Cell Doors")
    );
    let (w4, h4, rgba4) =
        load_png_rgba8(&fixture_path("objective_survival_of_the_fittest_popup.png"));
    let view4 = make_frame(w4, h4, &rgba4);
    let t_title4 = std::time::Instant::now();
    let title4 = reader.read_title_modal(&view4);
    let title4_ms = t_title4.elapsed().as_secs_f64() * 1000.0;
    assert!(
        matches!(
            title4,
            Some(TitleModalRead::Objective(ObjectiveId::Consume))
        ),
        "expected Objective(Consume) from objective_survival_of_the_fittest_popup.png, got {:?}",
        title4
    );
    println!("title4 ({w4}x{h4}): {title4:?} ms={title4_ms:.2}ms");
}

#[test]
fn test_pick_three_relics_and_start_event_popup() {
    let mut reader = UiReader::new();
    assert_eq!(std::mem::align_of::<UiReader>(), 64);
    assert_pick_three_relics(&mut reader);
    assert_scroll_popups(&mut reader);
    assert!(std::mem::size_of_val(&reader) > 0);
}

fn assert_day5_and_popup_step(tracker: &mut RunTracker, reader: &mut UiReader) {
    let (w5, h5, rgba5) = load_png_rgba8(&fixture_path("board_day5_supreme_lion_no_jaws.png"));
    let (wp, hp, rgbap) = load_png_rgba8(&fixture_path("start_event_tyranny_popup.png"));
    let view5 = make_frame(w5, h5, &rgba5);
    let viewp = make_frame(wp, hp, &rgbap);

    let scan5 = reader.read_board(&view5, None, ReadBoardOptions::default());
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
    assert_eq!(cells5[9].kind, CellKindRead::Inmate);
    assert_eq!(cells5[9].tier, 4, "Cell 10 (idx 9) is a Supreme Lion");

    let (ok5, _, _) = tracker.process_frame(
        &view5,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        reader,
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

    let (okp, _, _) = tracker.process_frame(&viewp, CaptureTrigger::Objective, reader);
    assert!(okp);
    assert_eq!(tracker.model.start_event, StartEventId::Tyranny);
    assert_eq!(
        tracker.model.checks, 0,
        "Start event popup must not corrupt underlying board cells"
    );
}

fn assert_day13_step(tracker: &mut RunTracker, reader: &mut UiReader) {
    let (w13, h13, rgba13) =
        load_png_rgba8(&fixture_path("board_day13_supreme_jaws_chromatic7.png"));
    let view13 = make_frame(w13, h13, &rgba13);

    tracker.hint = None;
    let scan13 = reader.read_board(&view13, None, ReadBoardOptions::default());
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
        reader,
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
    let mut reader = UiReader::new();
    let mut tracker = RunTracker::new();
    assert_day5_and_popup_step(&mut tracker, &mut reader);
    assert_day13_step(&mut tracker, &mut reader);
    assert_eq!(tracker.model.vault.len(), 7);
    assert_eq!(tracker.model.checks, 0);
}

#[test]
fn test_phantom_tool_slot_regression() {
    let (w, h, rgba) = load_png_rgba8(&fixture_path("phantom_slot.png"));
    assert!(w > 0 && h > 0);
    let view = make_frame(w, h, &rgba);
    let mut reader = UiReader::new();
    let res = reader.read_board(&view, None, ReadBoardOptions::default());
    assert!(
        res.tools.len() <= 2,
        "phantom_slot.png must not detect phantom tool slots, got {:?}",
        res.tools
    );
}

#[test]
fn test_cell06_above_supreme_jaws_cell07() {
    let (w, h, rgba) = load_png_rgba8(&fixture_path("board_day2_cell06_check.png"));
    let view = make_frame(w, h, &rgba);
    let mut reader = UiReader::new();
    let scan = reader.read_board(&view, None, ReadBoardOptions::default());
    assert!(scan.ok, "board_day2_cell06_check.png must localize");
    let cells = scan.cells.expect("cells");
    assert_eq!(
        cells[5].kind,
        CellKindRead::Locked,
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
    assert_eq!(cells[6].kind, CellKindRead::Inmate);
    assert_eq!(cells[6].tier, 4, "Cell 07 (idx 6) is a Supreme Mammoth");
    assert!(cells[6].jaws, "Cell 07 (idx 6) has a red jaws badge");

    let mut tracker = RunTracker::new();
    let (ok, _, _) = tracker.process_frame(
        &view,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        &mut reader,
    );
    assert!(ok);
    assert_eq!(
        tracker.model.checks, 0,
        "board_day2_cell06_check.png must have 0 cells to check, got mask {:#06x}",
        tracker.model.checks
    );
}

fn assert_office_full_frame(rgba: &[u8], w: usize, h: usize, reader: &mut UiReader) {
    let view = make_frame(w, h, rgba);
    let scan = reader.read_board(&view, None, ReadBoardOptions::default());
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
        reader,
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

fn assert_office_early_frame(rgba: &[u8], w: usize, h: usize, reader: &mut UiReader) {
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
    let early_view = make_frame(w, h, &early_rgba);
    let early_scan = reader.read_board(&early_view, None, ReadBoardOptions::default());
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
        reader,
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
    let mut reader = UiReader::new();
    assert_office_full_frame(&rgba, w, h, &mut reader);
    assert_office_early_frame(&rgba, w, h, &mut reader);
    assert_eq!(std::mem::align_of::<UiReader>(), 64);
}

fn bench_single_fixture(name: &str, reader: &mut UiReader) -> [f64; 6] {
    let (fw, fh, frgba) = load_png_rgba8(&fixture_path(name));
    let fview = make_frame(fw, fh, &frgba);
    let warm = reader.read_board(&fview, None, ReadBoardOptions::default());
    assert!(warm.ok, "Fixture {name} should read ok");
    let hint = warm.viewport;
    assert!(hint.is_some());

    let iters = 9usize;
    let mut vp_ms = f64::INFINITY;
    let mut board_ms = f64::INFINITY;
    let mut hud_ms = f64::INFINITY;
    let mut ms_total = f64::INFINITY;
    let mut hud_skip_ms = f64::INFINITY;
    let mut skip_ms_total = f64::INFINITY;
    for _ in 0..iters {
        let res = reader.read_board(&fview, hint, ReadBoardOptions { read_days: true });
        vp_ms = vp_ms.min(res.profile.viewport_ms);
        board_ms = board_ms.min(res.profile.board_ms);
        hud_ms = hud_ms.min(res.profile.hud_ms);
        ms_total = ms_total.min(res.profile.ms_total);

        let res_skip = reader.read_board(&fview, hint, ReadBoardOptions { read_days: false });
        hud_skip_ms = hud_skip_ms.min(res_skip.profile.hud_ms);
        skip_ms_total = skip_ms_total.min(res_skip.profile.ms_total);
    }
    let out = [
        vp_ms,
        board_ms,
        hud_ms,
        ms_total,
        hud_skip_ms,
        skip_ms_total,
    ];
    assert!(out[0].is_finite() && out[5].is_finite());
    out
}

#[test]
fn test_hud_objective_and_scan_latency_benchmark() {
    let mut reader = UiReader::new();

    let (w, h, rgba) = load_png_rgba8(&fixture_path("board_cell06_warden.png"));
    let view = make_frame(w, h, &rgba);
    let mut tracker = RunTracker::new();
    let (ok, _, _) = tracker.process_frame(
        &view,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        &mut reader,
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
    let mut best = [[f64::INFINITY; 6]; 8];
    for _pass in 0..2 {
        for (idx, name) in fixtures.iter().enumerate() {
            let m = bench_single_fixture(name, &mut reader);
            for i in 0..6 {
                best[idx][i] = best[idx][i].min(m[i]);
            }
        }
    }
    let mut sums = [0.0_f64; 6];
    for (idx, name) in fixtures.iter().enumerate() {
        let out = best[idx];
        println!(
            "UiRead [{name:<30}]: vp={:5.2}ms board={:5.2}ms hud={:5.2}ms \
             (no_days={:5.2}ms) total={:5.2}ms (no_days={:5.2}ms)",
            out[0], out[1], out[2], out[4], out[3], out[5]
        );
        for i in 0..6 {
            sums[i] += out[i];
        }
    }
    let n = fixtures.len() as f64;
    let avg_skip_days = sums[5] / n;
    println!(
        "UiRead Average (8 board fixtures): vp={:.2}ms board={:.2}ms hud={:.2}ms \
         (no_days={:.2}ms) total={:.2}ms (no_days={:.2}ms)",
        sums[0] / n,
        sums[1] / n,
        sums[2] / n,
        sums[4] / n,
        sums[3] / n,
        avg_skip_days
    );
    let limit_ms = if cfg!(debug_assertions) { 25.0 } else { 1.0 };
    assert!(
        avg_skip_days < limit_ms,
        "expected average log-assisted board read < {limit_ms}ms, got {avg_skip_days:.3}ms"
    );
}

fn inspect_single_user_screenshot(name: &str, reader: &mut UiReader) {
    let path = fixture_path(name);
    let (w, h, rgba) = load_png_rgba8(&path);
    assert!(w > 0 && h > 0);
    let view = make_frame(w, h, &rgba);
    let res = reader.read_board(&view, None, ReadBoardOptions::default());
    assert!(res.ok);
    println!("\n=== {name} ({w}x{h}) ===");
    println!(
        "profile: vp={:.2}ms board={:.2}ms hud={:.2}ms total={:.2}ms",
        res.profile.viewport_ms, res.profile.board_ms, res.profile.hud_ms, res.profile.ms_total
    );
    println!(
        "warden={:?} days={:?} chromatic={:?}",
        res.warden, res.days, res.chromatic
    );
    if let Some(ref cells) = res.cells {
        for (idx, c) in cells.iter().enumerate() {
            if c.kind != CellKindRead::Locked {
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
    assert!(tracker.hint.is_some());
}

fn assert_seq_day3_and_day4(seq_tracker: &mut RunTracker, reader: &mut UiReader) {
    let scan_limit_ms = if cfg!(debug_assertions) { 50.0 } else { 25.0 };
    let solve_limit_ms = if cfg!(debug_assertions) { 500 } else { 140 };
    seq_tracker.log_days = Some((3, 20));
    let (w1, h1, rgba1) = load_png_rgba8(&fixture_path("board_day3_doors_7_11_12.png"));
    let view1 = make_frame(w1, h1, &rgba1);
    let (ok1, _, prof1) = seq_tracker.process_frame(
        &view1,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        reader,
    );
    assert!(ok1);
    assert_eq!(seq_tracker.model.tools.as_slice(), &[ToolId::Crown]);
    assert!(
        prof1.ms_total < scan_limit_ms,
        "expected read < {scan_limit_ms}ms, got {:.2}ms",
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
    let view2 = make_frame(w2, h2, &rgba2);
    let (ok2, _, prof2) = seq_tracker.process_frame(
        &view2,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        reader,
    );
    assert!(ok2);
    assert_eq!(seq_tracker.model.tools.as_slice(), &[ToolId::Crown]);
    assert!(
        prof2.ms_total < scan_limit_ms,
        "expected read < {scan_limit_ms}ms, got {:.2}ms",
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

fn assert_seq_day11(seq_tracker: &mut RunTracker, reader: &mut UiReader) {
    let scan_limit_ms = if cfg!(debug_assertions) { 50.0 } else { 5.0 };
    seq_tracker.log_days = Some((11, 20));
    let (w3, h3, rgba3) = load_png_rgba8(&fixture_path("board_day11_cell11_lion_hunt_replace.png"));
    let view3 = make_frame(w3, h3, &rgba3);
    let (ok3, _, prof3) = seq_tracker.process_frame(
        &view3,
        CaptureTrigger::Board {
            treasure_view: false,
        },
        reader,
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
        "expected read < {scan_limit_ms}ms, got {:.2}ms",
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
    let mut reader = UiReader::new();
    for name in names {
        inspect_single_user_screenshot(name, &mut reader);
    }

    let mut seq_tracker = RunTracker::new();
    assert_seq_day3_and_day4(&mut seq_tracker, &mut reader);
    assert_seq_day11(&mut seq_tracker, &mut reader);
    assert_eq!(seq_tracker.model.day, 11);
    assert_eq!(seq_tracker.model.tools.len(), 2);
}

fn resize_rgba_bilinear(
    src: &[u8],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
) -> Vec<u8> {
    assert!(src_w > 0 && src_h > 0);
    assert!(dst_w > 0 && dst_h > 0);
    let mut out = vec![0u8; dst_w * dst_h * 4];
    for dy in 0..dst_h {
        let fy = ((dy as f32 + 0.5) * (src_h as f32) / (dst_h as f32) - 0.5)
            .clamp(0.0, (src_h - 1) as f32);
        let y0 = fy.floor() as usize;
        let y1 = (y0 + 1).min(src_h - 1);
        let wy = fy - y0 as f32;
        for dx in 0..dst_w {
            let fx = ((dx as f32 + 0.5) * (src_w as f32) / (dst_w as f32) - 0.5)
                .clamp(0.0, (src_w - 1) as f32);
            let x0 = fx.floor() as usize;
            let x1 = (x0 + 1).min(src_w - 1);
            let wx = fx - x0 as f32;
            let dst_idx = (dy * dst_w + dx) * 4;
            for c in 0..4 {
                let p00 = f32::from(src[(y0 * src_w + x0) * 4 + c]);
                let p10 = f32::from(src[(y0 * src_w + x1) * 4 + c]);
                let p01 = f32::from(src[(y1 * src_w + x0) * 4 + c]);
                let p11 = f32::from(src[(y1 * src_w + x1) * 4 + c]);
                let top = p00 + (p10 - p00) * wx;
                let bot = p01 + (p11 - p01) * wx;
                out[dst_idx + c] = (top + (bot - top) * wy).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

fn assert_modals_at_resolution(
    reader: &mut UiReader,
    (target_w, target_h): (usize, usize),
    p1: (usize, usize, &[u8]),
    p2: (usize, usize, &[u8]),
    t1: (usize, usize, &[u8]),
) {
    assert!(target_w > 0 && target_h > 0);
    assert!(!p1.2.is_empty() && !p2.2.is_empty());
    let scaled_p = resize_rgba_bilinear(p1.2, p1.0, p1.1, target_w, target_h);
    let p_view = make_frame(target_w, target_h, &scaled_p);
    let pick = reader
        .read_relic_modal(&p_view)
        .expect("Relic modal must be read at scaled resolution");
    let tools: Vec<ToolId> = pick.cards.iter().map(|c| c.sim).collect();
    assert_eq!(
        tools,
        vec![ToolId::Pestle, ToolId::Dogtag, ToolId::Rope],
        "Relic modal cards must match at {target_w}x{target_h}"
    );

    let scaled_p2 = resize_rgba_bilinear(p2.2, p2.0, p2.1, target_w, target_h);
    let p2_view = make_frame(target_w, target_h, &scaled_p2);
    let t_p2 = std::time::Instant::now();
    let pick2 = reader
        .read_relic_modal(&p2_view)
        .expect("Relic modal 2 must be read at scaled resolution");
    let p2_ms = t_p2.elapsed().as_secs_f64() * 1000.0;
    let tools2: Vec<ToolId> = pick2.cards.iter().map(|c| c.sim).collect();
    assert_eq!(
        tools2,
        vec![ToolId::Dogtag, ToolId::Cage, ToolId::Ironbox],
        "Relic modal 2 cards must match at {target_w}x{target_h}"
    );

    let scaled_t = resize_rgba_bilinear(t1.2, t1.0, t1.1, target_w, target_h);
    let t_view = make_frame(target_w, target_h, &scaled_t);
    let t_title = std::time::Instant::now();
    let title = reader.read_title_modal(&t_view);
    let title_ms = t_title.elapsed().as_secs_f64() * 1000.0;
    assert!(
        matches!(title, Some(TitleModalRead::Objective(ObjectiveId::Consume))),
        "Objective popup must match at {target_w}x{target_h}, got {title:?}"
    );
    println!(
        "MultiRes [{target_w}x{target_h}]: relic_modal={p2_ms:.2}ms title_modal={title_ms:.2}ms"
    );
}

#[test]
fn test_multi_resolution_scaling_board_and_modals() {
    let mut reader = UiReader::new();
    let (bw, bh, brgba) = load_png_rgba8(&fixture_path("board_day13_supreme_jaws_chromatic7.png"));
    let (pw, ph, prgba) =
        load_png_rgba8(&fixture_path("pick_three_relics_pestle_dogtag_noose.png"));
    let (p2w, p2h, p2rgba) =
        load_png_rgba8(&fixture_path("pick_three_relics_dogtag_cage_ratbox.png"));
    let (tw, th, trgba) =
        load_png_rgba8(&fixture_path("objective_survival_of_the_fittest_popup.png"));
    assert!(bw > 0 && pw > 0);
    assert!(p2w > 0 && tw > 0);

    for (target_w, target_h) in [(1280, 720), (1920, 1080), (2560, 1440), (3840, 2160)] {
        let scaled_b = resize_rgba_bilinear(&brgba, bw, bh, target_w, target_h);
        let b_view = make_frame(target_w, target_h, &scaled_b);
        let b_res = reader.read_board(&b_view, None, ReadBoardOptions { read_days: false });
        assert!(
            b_res.ok,
            "Board must localize at {target_w}x{target_h}, got {:?}",
            b_res.viewport
        );
        let cells = b_res.cells.expect("scaled cells");
        assert_eq!(
            cells[9].tier, 4,
            "Supreme Lion tier at {target_w}x{target_h}"
        );
        assert!(cells[9].jaws, "Supreme Lion jaws at {target_w}x{target_h}");
        assert_modals_at_resolution(
            &mut reader,
            (target_w, target_h),
            (pw, ph, &prgba),
            (p2w, p2h, &p2rgba),
            (tw, th, &trgba),
        );
    }
}

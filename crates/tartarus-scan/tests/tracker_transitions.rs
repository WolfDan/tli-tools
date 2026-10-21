use tartarus_core::{
    Beast, BoardRunStage, BoardStep, CellKind, FixedVec, GameMode, ObjectiveId, PendingKind,
    RunPhase, StartEventId, ToolId, TreasureOrigin, TreasurePhase,
};
use tartarus_scan::{
    ActiveUiView, BoardRead, CellKindRead, CellRead, DoorsRead, LogEvent, LogWatcher,
    RelicCardRead, RelicModalRead, RunTracker, TitleModalRead, UiReadProfile, UmgViewport,
    WardenRead,
};

#[test]
fn test_umg_viewport_roundtrip_and_aspect_scaling() {
    let vp_16_9 = UmgViewport::from_window_size(1920, 1080);
    assert!((vp_16_9.s - 1.430_463_6).abs() < 1e-4);
    assert!((vp_16_9.tx - 419.999_97).abs() < 1e-3);
    assert!((vp_16_9.ty - 328.623_3).abs() < 1e-2);

    let (x, y) = vp_16_9.at(2.0, 1.0);
    let (u, v) = vp_16_9.grid_of(x, y);
    assert!((u - 2.0).abs() < 1e-6);
    assert!((v - 1.0).abs() < 1e-6);

    let vp_21_9 = UmgViewport::from_window_size(3440, 1440);
    assert!((vp_21_9.s - 1.907_284_7).abs() < 1e-4);
    assert!(vp_21_9.tx > 900.0);
}

#[test]
fn test_tracker_default() {
    let mut tracker = RunTracker::new();
    let msg = tracker.use_title(TitleModalRead::Objective(ObjectiveId::King));
    assert!(msg.contains("King"));
    assert_eq!(tracker.model.objective, ObjectiveId::King);
}

#[test]
fn test_tracker_log_lifecycle_and_day_check() {
    let mut tracker = RunTracker::new();
    assert_eq!(tracker.phase, RunPhase::WaitingForRun);

    // Begin run 100
    assert!(!tracker.on_log_run_begin(100, false));
    assert_eq!(tracker.log_run_id, Some(100));

    // Activate run via example board
    tracker.load_example();
    assert!(tracker.phase.is_active());

    // Ground-truth day check updates model day and days_max
    assert!(tracker.on_log_day_check(5, 20));
    assert_eq!(tracker.model.days_max, 20);
    assert_eq!(tracker.model.day, 5);

    // Settled run ends the run
    assert!(tracker.on_log_run_begin(100, true));
    assert_eq!(tracker.phase, RunPhase::RunOver);
    assert_eq!(tracker.log_days, None);

    // Starting a new runId resets to WaitingForRun
    assert!(tracker.on_log_run_begin(101, false));
    assert_eq!(tracker.phase, RunPhase::WaitingForRun);
    assert_eq!(tracker.log_run_id, Some(101));
}

#[test]
fn test_log_watcher_active_view_tracking() {
    let mut lw = LogWatcher::new_isolated();
    let mut evs = Vec::new();
    assert_eq!(lw.active_view(), ActiveUiView::Unknown);

    lw.process_line("[2026.10.20] S16BoardTaskPop Run", &mut evs);
    assert_eq!(lw.active_view(), ActiveUiView::TaskPop);
    assert!(evs.contains(&LogEvent::ObjectivePopup));

    evs.clear();
    lw.process_line("[2026.10.20] S16BoardRelicPop Run", &mut evs);
    assert_eq!(lw.active_view(), ActiveUiView::RelicPop);
    assert!(evs.contains(&LogEvent::RelicPopState(true)));

    evs.clear();
    lw.process_line("[2026.10.20] S16BoardRelicPop Destory", &mut evs);
    assert_eq!(lw.active_view(), ActiveUiView::BoardMain);
    assert!(evs.contains(&LogEvent::RelicPopState(false)));

    evs.clear();
    lw.process_line("[2026.10.20] S16BoardColoredPop Run", &mut evs);
    assert_eq!(lw.active_view(), ActiveUiView::ColoredPop);

    evs.clear();
    lw.process_line("[2026.10.20] S16GamePlayCheckOut Run", &mut evs);
    assert_eq!(lw.active_view(), ActiveUiView::CheckOutRiot);
}

#[test]
fn test_tracker_pick_and_treasure_transitions() {
    let mut tracker = RunTracker::new();
    tracker.use_title(TitleModalRead::StartEvent(StartEventId::IronLaw));
    tracker.use_title(TitleModalRead::Objective(ObjectiveId::Consume));

    let pick = RelicModalRead {
        cards: [
            RelicCardRead {
                sim: ToolId::Crown,
                score: 0.95,
                conf: 0.90,
            },
            RelicCardRead {
                sim: ToolId::Rope,
                score: 0.94,
                conf: 0.88,
            },
            RelicCardRead {
                sim: ToolId::Guillotine,
                score: 0.92,
                conf: 0.85,
            },
        ],
        reroll: true,
    };

    let msg = tracker.use_pick(&pick, true);
    assert!(msg.contains("A fresh run"));
    assert_eq!(tracker.model.stage, BoardRunStage::Opening);
    assert_eq!(tracker.model.step, BoardStep::Choice);
    assert_eq!(tracker.model.choice.kind, PendingKind::Tool);
    assert_eq!(tracker.model.days_max, 30);
    assert_eq!(tracker.model.objective, ObjectiveId::Consume);

    // Open treasure after Consume
    let t_msg = tracker.use_treasure(Some(TreasureOrigin::Consume));
    assert!(t_msg.contains("Warden's Treasure is open"));
    assert_eq!(tracker.model.treasure, TreasurePhase::WaitingForBoardView);
    assert!(tracker.model.devour_used);

    // View board behind treasure transitions to ReadyToChoose
    let v_msg = tracker.use_treasure(Some(TreasureOrigin::View));
    assert!(v_msg.contains("Board seen behind"));
    assert_eq!(tracker.model.treasure, TreasurePhase::ReadyToChoose);
}

fn mock_scan_cells() -> [CellRead; 16] {
    let mut cells = [CellRead::default(); 16];
    for (i, cell) in cells.iter_mut().enumerate() {
        cell.n = i as u8;
        cell.kind = if i == 0 {
            CellKindRead::Office
        } else {
            CellKindRead::Locked
        };
        cell.conf = 0.95;
    }
    cells[1].kind = CellKindRead::Inmate;
    cells[1].beast = Beast::Lion;
    cells[1].tier = 2;
    cells[1].jaws = true;

    cells[2].kind = CellKindRead::Inmate;
    cells[2].beast = Beast::Wolf;
    cells[2].tier = 2;
    cells[2].jaws = false;
    assert_eq!(cells[0].kind, CellKindRead::Office);
    assert_eq!(cells[1].beast, Beast::Lion);
    cells
}

#[test]
fn test_tracker_scan_to_model_reconciliation() {
    let mut tracker = RunTracker::new();
    tracker.model.mode = GameMode::Drill;
    tracker.model.days_max = 20;

    let mut door_cells = FixedVec::new();
    door_cells.push(2);
    door_cells.push(5);
    door_cells.push(8);

    let board = BoardRead {
        ok: true,
        viewport: Some(UmgViewport {
            s: 1.0,
            tx: 0.0,
            ty: 0.0,
            score: 0.95,
            w: 1920,
            h: 1080,
            by_labels: false,
            cells_matched: 15,
        }),
        cells: Some(mock_scan_cells()),
        warden: Some(WardenRead {
            cell: 1,
            conf: 0.95,
            box_coords: [0, 0, 10, 10],
        }),
        doors: Some(DoorsRead {
            cells: door_cells,
            conf: 0.92,
            candidates: FixedVec::new(),
            office_card: false,
        }),
        days: Some((15, 0.90)),
        tools: FixedVec::new(),
        chromatic: Some(1),
        chain: None,
        decision: None,
        profile: UiReadProfile::default(),
    };

    assert!(tracker.ingest(&board));
    assert_eq!(tracker.model.day, 5);
    assert_eq!(tracker.model.warden, 1);
    assert_eq!(tracker.model.step, BoardStep::Patrol);
    assert!(tracker.model.has_dest(2));
    assert!(tracker.model.has_dest(5));
    assert!(tracker.model.has_dest(8));
    assert_eq!(tracker.model.cells[1].kind, CellKind::Inmate);
    assert_eq!(tracker.model.cells[1].beast, Beast::Lion);
    assert_eq!(tracker.model.vault.len(), 1);
    assert_eq!(tracker.model.checks, 0);
}

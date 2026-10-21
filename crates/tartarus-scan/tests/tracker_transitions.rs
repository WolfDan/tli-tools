use tartarus_core::{
    Beast, BoardRunStage, BoardStep, CellKind, FixedVec, GameMode, ObjectiveId, PendingKind,
    RunPhase, StartEventId, ToolId, TreasureOrigin, TreasurePhase,
};
use tartarus_scan::{
    BoardLoc, CARD_MEDALLIONS, DIGITS, DoorsRead, FACES_LION, FACES_MAMMOTH, FACES_RAT, FACES_WOLF,
    PREFIX_GLYPHS, PickCardRead, PickScreenRead, REF_BOARD_HEIGHT, REF_BOARD_POINTS,
    REF_BOARD_WIDTH, REF_PICK_TOOLS, REF_TOOLS, ROOM_BG_BOXES, RunTracker, S16_RELIC_META,
    ScanProfile, ScanResult, ScannedCell, ScannedCellKind, TitleRead, WardenRead,
};

#[test]
fn test_scan_data_loaded() {
    assert_eq!(S16_RELIC_META.len(), 31);
    assert_eq!(ROOM_BG_BOXES.len(), 16);
    assert_eq!(CARD_MEDALLIONS.len(), 8);
    assert_eq!(REF_BOARD_WIDTH, 755.0);
    assert_eq!(REF_BOARD_HEIGHT, 424.0);
    assert_eq!(REF_BOARD_POINTS.len(), 1002);
    assert_eq!(FACES_LION.len(), 54);
    assert_eq!(FACES_MAMMOTH.len(), 54);
    assert_eq!(FACES_RAT.len(), 54);
    assert_eq!(FACES_WOLF.len(), 54);
    assert_eq!(DIGITS[0].len(), 54);
    assert_eq!(PREFIX_GLYPHS.len(), 54);
    assert_eq!(REF_TOOLS.len(), 217);
    assert_eq!(REF_PICK_TOOLS.len(), 372);
}

#[test]
fn test_homography_roundtrip() {
    let loc = BoardLoc {
        s: 1.5,
        tx: 100.0,
        ty: 50.0,
        score: 0.95,
        w: 1920,
        h: 1080,
        by_labels: false,
        cells_matched: 15,
    };
    let (x, y) = loc.at(2.0, 1.0);
    let (u, v) = loc.grid_of(x, y);
    assert!((u - 2.0).abs() < 1e-6);
    assert!((v - 1.0).abs() < 1e-6);
}

#[test]
fn test_tracker_default() {
    let mut tracker = RunTracker::new();
    let msg = tracker.use_title(TitleRead::Objective(ObjectiveId::King));
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
fn test_tracker_pick_and_treasure_transitions() {
    let mut tracker = RunTracker::new();
    tracker.use_title(TitleRead::StartEvent(StartEventId::IronLaw));
    tracker.use_title(TitleRead::Objective(ObjectiveId::Consume));

    let pick = PickScreenRead {
        cards: [
            PickCardRead {
                sim: ToolId::Crown,
                score: 0.95,
                conf: 0.90,
            },
            PickCardRead {
                sim: ToolId::Rope,
                score: 0.94,
                conf: 0.88,
            },
            PickCardRead {
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

fn mock_scan_cells() -> [ScannedCell; 16] {
    let mut cells = [ScannedCell::default(); 16];
    for (i, cell) in cells.iter_mut().enumerate() {
        cell.n = i as u8;
        cell.kind = if i == 0 {
            ScannedCellKind::Office
        } else {
            ScannedCellKind::Locked
        };
        cell.conf = 0.95;
    }
    cells[1].kind = ScannedCellKind::Inmate;
    cells[1].beast = Beast::Lion;
    cells[1].tier = 2;
    cells[1].jaws = true;

    cells[2].kind = ScannedCellKind::Inmate;
    cells[2].beast = Beast::Wolf;
    cells[2].tier = 2;
    cells[2].jaws = false;
    assert_eq!(cells[0].kind, ScannedCellKind::Office);
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

    let scan = ScanResult {
        ok: true,
        loc: Some(BoardLoc {
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
        pick: None,
        title: None,
        profile: ScanProfile::default(),
    };

    assert!(tracker.ingest(&scan));
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

use tartarus_overlay::capture::{CapturedFrame, StillnessTracker, bgra_to_rgba_in_place};
use tartarus_overlay::log_watcher::{LogEvent, LogWatcher};
use tartarus_overlay::state::PanelLayout;

#[test]
fn test_s16_log_triggers() {
    let mut lw = LogWatcher::new_isolated();
    let mut evs = Vec::new();
    lw.process_line("[2026.10.17] S16BoardTaskPop Run", &mut evs);
    assert!(evs.contains(&LogEvent::ObjectivePopup));
    assert!(evs.contains(&LogEvent::RunNew));

    evs.clear();
    lw.process_line("[2026.10.17] [S16LastDay] check cur=0 max=12", &mut evs);
    assert!(evs.contains(&LogEvent::DayCheck { cur: 0, max: 12 }));
    lw.process_line("[2026.10.17] S16BoardRelicPop Run", &mut evs);
    assert!(evs.contains(&LogEvent::InstrumentsPopup { fresh: true }));
    assert!(evs.contains(&LogEvent::RelicPopState(true)));

    evs.clear();
    lw.process_line(
        "[2026.10.17] [S16BGM] board Begin runId=425a86a4-c923-11f1-8fcd-00000000005b IsSettled=false Result=-1",
        &mut evs,
    );
    assert!(evs.iter().any(|e| matches!(
        e,
        LogEvent::RunBegin {
            is_settled: false,
            ..
        }
    )));

    evs.clear();
    lw.process_line("[2026.10.17] S16BoardNewDayPop Destory", &mut evs);
    assert!(evs.contains(&LogEvent::BoardTrigger {
        settle_ms: 450,
        treasure_view: false
    }));

    evs.clear();
    lw.process_line("[2026.10.17] S16GamePlayCheckOut Run", &mut evs);
    assert!(evs.contains(&LogEvent::Riot(true)));
    assert!(evs.contains(&LogEvent::RelicPopState(false)));

    evs.clear();
    lw.process_line(
        "[2026.10.17] Btn_S16GamePlay_CheckOut_AbandonChallenge",
        &mut evs,
    );
    assert!(evs.contains(&LogEvent::RunOver));
}

#[test]
fn test_bgra_to_rgba_in_place() {
    let mut pixels = vec![0u8; 40];
    for (i, px) in pixels.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        px[0] = (i as u8) * 10 + 1; // B
        px[1] = (i as u8) * 10 + 2; // G
        px[2] = (i as u8) * 10 + 3; // R
        px[3] = 0; // A (should become 255)
    }
    bgra_to_rgba_in_place(&mut pixels);
    for (i, px) in pixels.as_chunks::<4>().0.iter().enumerate() {
        assert_eq!(px[0], (i as u8) * 10 + 3); // R
        assert_eq!(px[1], (i as u8) * 10 + 2); // G
        assert_eq!(px[2], (i as u8) * 10 + 1); // B
        assert_eq!(px[3], 255); // A
    }
}

#[test]
fn test_stillness_tracker_convergence() {
    let mut tracker = StillnessTracker::new();
    let frame_a_buf = vec![20u8; 64 * 36 * 4];
    let frame_b_buf = vec![200u8; 64 * 36 * 4];
    let frame_a = CapturedFrame {
        width: 64,
        height: 36,
        rgba: &frame_a_buf,
    };
    let frame_b = CapturedFrame {
        width: 64,
        height: 36,
        rgba: &frame_b_buf,
    };

    tracker.push(&frame_a);
    assert!(!tracker.moved);
    assert!(!tracker.settled());

    // Push a very different frame -> moved = true, moving() = true
    tracker.push(&frame_b);
    assert!(tracker.moved);
    assert!(tracker.moving());
    assert!(!tracker.settled());

    // Hold frame_b still for STILL_LOOKS frames -> settled() = true
    for _ in 0..StillnessTracker::STILL_LOOKS {
        assert!(!tracker.settled());
        tracker.push(&frame_b);
    }
    assert!(tracker.settled());
    assert!(!tracker.moving());
}

#[test]
fn test_panel_layout_transitions() {
    let mut layout = PanelLayout::default();
    assert_eq!(layout, PanelLayout::Compact);
    assert!(!layout.is_folded());
    assert!(!layout.is_inspector_open());

    // Open inspector with tools drawer
    layout.open_inspector(true);
    assert!(layout.is_inspector_open());
    assert!(layout.is_tools_drawer_open());

    // Auto-fold for Riot and restore previous inspector + tools state
    assert!(layout.apply_riot(true));
    assert!(layout.is_folded());
    assert!(layout.apply_riot(false));
    assert_eq!(layout, PanelLayout::Inspector { tools_drawer: true });

    // Toggle fold manually
    layout.toggle_fold();
    assert_eq!(layout, PanelLayout::FoldedManual);
    // Riot does not override manual fold
    assert!(!layout.apply_riot(true));
    layout.toggle_fold();
    assert_eq!(layout, PanelLayout::Compact);
}

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use tartarus_data::{
    DecodedAssetImage, map_event_id_to_sim, map_gift_id_to_sim, map_relic_id_to_sim,
    map_task_id_to_sim, pick_art_bbox_on_rgba, synthesize_icon_feature,
};
use tartarus_extract::strip_rich_text;

#[test]
fn test_strip_rich_text() {
    let raw = "Requires <Red>1</> fewer beast; grants +50% 'Rainbow' [145]!";
    let clean = strip_rich_text(raw);
    assert_eq!(
        clean,
        "Requires 1 fewer beast; grants +50% 'Rainbow' [145]!"
    );
}

#[test]
fn test_all_31_active_relics_have_unique_sim_keys() {
    let mut sims = BTreeSet::new();
    for id in [
        101, 102, 103, 104, 105, 106, 107, 111, 112, 113, 114, 115, 116, 117, 121, 122, 123, 124,
        125, 126, 127, 131, 132, 141, 142, 143, 144, 145, 146, 147, 148,
    ] {
        let sim = map_relic_id_to_sim(id);
        assert_ne!(
            sim, "unknown",
            "Active relic {id} must map to a known sim key"
        );
        assert!(sims.insert(sim), "Duplicate sim key {sim} for relic {id}");
    }
    assert_eq!(sims.len(), 31);
}

#[test]
fn test_all_gifts_tasks_and_events_have_unique_sim_keys() {
    let mut gifts = BTreeSet::new();
    for id in 1..=4 {
        let sim = map_gift_id_to_sim(id);
        assert_ne!(sim, "unknown");
        assert!(gifts.insert(sim));
    }
    assert_eq!(gifts.len(), 4);

    let mut tasks = BTreeSet::new();
    for id in 1..=5 {
        let sim = map_task_id_to_sim(id);
        assert_ne!(sim, "other", "Task {id} must not map to 'other'");
        assert!(
            tasks.insert(sim),
            "Duplicate task sim key {sim} for task {id}"
        );
    }
    assert_eq!(tasks.len(), 5);
    assert_eq!(map_task_id_to_sim(4), "incident");

    let mut events = BTreeSet::new();
    for id in [
        201, 202, 203, 211, 212, 213, 221, 222, 231, 232, 241, 242, 251, 252, 253, 254, 255,
    ] {
        let sim = map_event_id_to_sim(id);
        assert_ne!(sim, "unknown", "Event {id} must map to a known sim key");
        assert!(
            events.insert(sim),
            "Duplicate event sim key {sim} for event {id}"
        );
    }
    assert_eq!(events.len(), 17);
}

#[test]
fn test_synthesize_icon_feature_and_art_bbox() {
    let w = 16usize;
    let h = 16usize;
    let mut rgba8 = vec![0u8; w * h * 4];
    for y in 2..14 {
        for x in 2..14 {
            let idx = (y * w + x) * 4;
            rgba8[idx..idx + 4].copy_from_slice(&[220, 80, 40, 255]);
        }
    }
    let asset = DecodedAssetImage {
        virtual_path: "test/icon.uasset".to_string(),
        short_name: "icon".to_string(),
        width: w,
        height: h,
        rgba8: rgba8.clone(),
    };
    let (grid, hist) = synthesize_icon_feature(&asset, (20.0, 20.0, 20.0));
    assert!(grid.iter().any(|&v| v > 100.0));
    assert!(hist.iter().any(|&v| v > 0.0));

    let bbox = pick_art_bbox_on_rgba(&rgba8, (w, h), (0, 0, w, h), 100, 0.3);
    assert_eq!(bbox, Some((2, 2, 14, 14)));
}

#[test]
fn test_cv_data_uses_tool_id_enum() {
    let cv_data_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../tartarus-scan/src/cv_data.rs");
    let content = fs::read_to_string(&cv_data_path).expect("read cv_data.rs");
    assert!(!content.contains("pub sim: &'static str,"));
    assert!(content.contains("pub sim: ToolId,"));
}

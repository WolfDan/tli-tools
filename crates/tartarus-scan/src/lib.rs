#![feature(portable_simd)]
#![forbid(unsafe_code)]

pub mod cv;
pub mod tracker;

pub use cv::*;
pub use tracker::*;

#[cfg(test)]
mod tests {
    use super::*;

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
        let msg = tracker.use_title(TitleRead::Objective(tartarus_core::ObjectiveId::King));
        assert!(msg.contains("King"));
        assert_eq!(tracker.model.objective, tartarus_core::ObjectiveId::King);
    }
}

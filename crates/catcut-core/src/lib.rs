pub mod capcut;
pub mod media;
pub mod model;
pub mod timeline;

pub use capcut::{export_to_capcut_json, import_from_capcut_file};
pub use media::{
    detect_scenes, detect_silence, export_project, extract_frame_base64, extract_voice_intervals,
    get_transcript, probe_media, ExportOptions, SceneCut, SilenceInterval, TranscriptEntry,
    VoiceInterval,
};
pub use model::*;
pub use timeline::{TimelineEngine, TimelineSummary};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_crud_and_split() {
        let mut engine = TimelineEngine::new("Test Project", 1920, 1080, 30.0);

        let mat = MediaMaterial::new(
            MaterialType::Video,
            "/tmp/dummy.mp4".to_string(),
            "dummy.mp4".to_string(),
            10_000_000, // 10s
            1920,
            1080,
            30.0,
            true,
        );
        let mat_id = mat.id.clone();
        engine.add_material(mat).unwrap();

        // Add 5s clip starting at 0
        let clip = engine
            .add_clip(None, &mat_id, 0, Some(0), Some(5_000_000), Some(1.0))
            .unwrap();

        assert_eq!(clip.target_timerange.duration, 5_000_000);
        assert_eq!(engine.project.duration(), 5_000_000);

        // Split clip at 2s (2_000_000 us)
        let (part1, part2) = engine.split_clip(&clip.id, 2_000_000).unwrap();
        assert_eq!(part1.target_timerange.duration, 2_000_000);
        assert_eq!(part2.target_timerange.start, 2_000_000);
        assert_eq!(part2.target_timerange.duration, 3_000_000);

        // Ripple delete part1
        let removed = engine.remove_clip(&part1.id, true).unwrap();
        assert_eq!(removed.id, part1.id);

        // Part 2 should have shifted to 0
        let remaining = &engine.project.tracks[0].segments[0];
        assert_eq!(remaining.id, part2.id);
        assert_eq!(remaining.target_timerange.start, 0);

        // Test Undo
        assert!(engine.undo().unwrap());
        // After undo, both parts should exist again
        assert_eq!(engine.project.tracks[0].segments.len(), 2);
    }

    #[test]
    fn test_capcut_json_export() {
        let mut engine = TimelineEngine::new("CapCut Export Test", 1920, 1080, 30.0);
        let mat = MediaMaterial::new(
            MaterialType::Video,
            "C:/videos/test.mp4".to_string(),
            "test.mp4".to_string(),
            6_000_000,
            1920,
            1080,
            30.0,
            true,
        );
        let mat_id = mat.id.clone();
        engine.add_material(mat).unwrap();
        engine
            .add_clip(None, &mat_id, 0, Some(1_000_000), Some(4_000_000), Some(1.0))
            .unwrap();

        let json_str = export_to_capcut_json(&engine.project).unwrap();
        assert!(json_str.contains("CapCut Export Test"));
        assert!(json_str.contains("C:/videos/test.mp4"));
    }

    #[test]
    fn test_capcut_import_from_real_or_mock() {
        let real_path = std::path::Path::new("/mnt/c/Users/Marc/AppData/Local/CapCut/User Data/Projects/com.lveditor.draft/0131/draft_content.json");
        if real_path.exists() {
            let project = import_from_capcut_file(real_path).unwrap();
            assert!(!project.tracks.is_empty());
            println!("Successfully imported CapCut draft: {} tracks, {} materials", project.tracks.len(), project.materials.len());
        }
    }
}

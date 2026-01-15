use otio_rs::*;
use tempfile::NamedTempFile;

/// Test metadata on Timeline.
#[test]
fn test_timeline_metadata() {
    let mut timeline = Timeline::new("Metadata Test");

    // Set metadata
    timeline.set_metadata("project_id", "proj_12345");
    timeline.set_metadata("created_by", "unit_test");

    // Get metadata
    assert_eq!(timeline.get_metadata("project_id"), Some("proj_12345".to_string()));
    assert_eq!(timeline.get_metadata("created_by"), Some("unit_test".to_string()));
    assert_eq!(timeline.get_metadata("nonexistent"), None);
}

/// Test metadata on Track.
#[test]
fn test_track_metadata() {
    let mut timeline = Timeline::new("Track Metadata Test");
    let mut track = timeline.add_video_track("V1");

    // Set metadata
    track.set_metadata("track_id", "track_001");
    track.set_metadata("color", "red");

    // Get metadata
    assert_eq!(track.get_metadata("track_id"), Some("track_001".to_string()));
    assert_eq!(track.get_metadata("color"), Some("red".to_string()));
    assert_eq!(track.get_metadata("nonexistent"), None);
}

/// Test metadata on Clip.
#[test]
fn test_clip_metadata() {
    let range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    );
    let mut clip = Clip::new("Test Clip", range);

    // Set metadata
    clip.set_metadata("external_id", "clip_abc123");
    clip.set_metadata("seq:channel_mask", "0x0003");

    // Get metadata
    assert_eq!(clip.get_metadata("external_id"), Some("clip_abc123".to_string()));
    assert_eq!(clip.get_metadata("seq:channel_mask"), Some("0x0003".to_string()));
    assert_eq!(clip.get_metadata("nonexistent"), None);
}

/// Test metadata on Gap.
#[test]
fn test_gap_metadata() {
    let mut gap = Gap::new(RationalTime::new(24.0, 24.0));

    // Set metadata
    gap.set_metadata("gap_reason", "scene_transition");
    gap.set_metadata("gap_id", "gap_001");

    // Get metadata
    assert_eq!(gap.get_metadata("gap_reason"), Some("scene_transition".to_string()));
    assert_eq!(gap.get_metadata("gap_id"), Some("gap_001".to_string()));
    assert_eq!(gap.get_metadata("nonexistent"), None);
}

/// Test metadata on Stack.
#[test]
fn test_stack_metadata() {
    let mut stack = Stack::new("Test Stack");

    // Set metadata
    stack.set_metadata("stack_type", "compositing");
    stack.set_metadata("layer_count", "3");

    // Get metadata
    assert_eq!(stack.get_metadata("stack_type"), Some("compositing".to_string()));
    assert_eq!(stack.get_metadata("layer_count"), Some("3".to_string()));
    assert_eq!(stack.get_metadata("nonexistent"), None);
}

/// Test metadata on ExternalReference.
#[test]
fn test_external_ref_metadata() {
    let mut ext_ref = ExternalReference::new("/path/to/media.mov");

    // Set metadata
    ext_ref.set_metadata("codec", "ProRes422HQ");
    ext_ref.set_metadata("resolution", "1920x1080");

    // Get metadata
    assert_eq!(ext_ref.get_metadata("codec"), Some("ProRes422HQ".to_string()));
    assert_eq!(ext_ref.get_metadata("resolution"), Some("1920x1080".to_string()));
    assert_eq!(ext_ref.get_metadata("nonexistent"), None);
}

/// Test metadata roundtrip through file write/read.
#[test]
fn test_metadata_roundtrip() {
    let mut timeline = Timeline::new("Roundtrip Metadata Test");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0));

    // Set timeline metadata
    timeline.set_metadata("project_id", "proj_roundtrip");
    timeline.set_metadata("version", "1.0.0");

    let mut track = timeline.add_video_track("V1");
    track.set_metadata("track_color", "blue");

    let range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    );

    let mut clip = Clip::new("Test Clip", range);
    clip.set_metadata("external_id", "clip_001");
    clip.set_metadata("notes", "This is a test clip");
    track.append_clip(clip).unwrap();

    // Write to file
    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline.write_to_file(temp_file.path()).expect("Failed to write");

    // Verify JSON contains metadata
    let contents = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(contents.contains("project_id"));
    assert!(contents.contains("proj_roundtrip"));
    assert!(contents.contains("external_id"));
    assert!(contents.contains("clip_001"));
    assert!(contents.contains("track_color"));
    assert!(contents.contains("blue"));

    // Read back
    let _reloaded = Timeline::read_from_file(temp_file.path()).expect("Failed to read");
}

/// Test overwriting metadata values.
#[test]
fn test_metadata_overwrite() {
    let range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    );
    let mut clip = Clip::new("Test Clip", range);

    // Set initial value
    clip.set_metadata("status", "draft");
    assert_eq!(clip.get_metadata("status"), Some("draft".to_string()));

    // Overwrite
    clip.set_metadata("status", "approved");
    assert_eq!(clip.get_metadata("status"), Some("approved".to_string()));
}

/// Test metadata with special characters.
#[test]
fn test_metadata_special_chars() {
    let range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    );
    let mut clip = Clip::new("Test Clip", range);

    // Test with special characters
    clip.set_metadata("description", "Scene with 'quotes' and \"double quotes\"");
    clip.set_metadata("path", "/path/to/file with spaces/media.mov");

    assert_eq!(
        clip.get_metadata("description"),
        Some("Scene with 'quotes' and \"double quotes\"".to_string())
    );
    assert_eq!(
        clip.get_metadata("path"),
        Some("/path/to/file with spaces/media.mov".to_string())
    );
}

/// Test standalone track metadata.
#[test]
fn test_standalone_track_metadata() {
    let mut track = Track::new_video("Standalone Track");

    track.set_metadata("track_id", "standalone_001");
    track.set_metadata("kind", "video");

    assert_eq!(track.get_metadata("track_id"), Some("standalone_001".to_string()));
    assert_eq!(track.get_metadata("kind"), Some("video".to_string()));
}

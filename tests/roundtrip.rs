use otio_rs::*;
use tempfile::NamedTempFile;

#[test]
fn test_create_simple_timeline() {
    let mut timeline = Timeline::new("Test Timeline");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0));

    let mut video_track = timeline.add_video_track("V1");

    let source_range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0), // 2 seconds
    );

    let mut clip = Clip::new("Test Clip", source_range);
    let mut media_ref = ExternalReference::new("/path/to/media.mov");
    media_ref.set_available_range(TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(240.0, 24.0), // 10 seconds total
    ));
    clip.set_media_reference(media_ref);

    video_track.append_clip(clip).expect("Failed to append clip");

    // Write and read back
    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline
        .write_to_file(temp_file.path())
        .expect("Failed to write");

    let _reloaded = Timeline::read_from_file(temp_file.path()).expect("Failed to read");
}

#[test]
fn test_gap_creation() {
    let mut timeline = Timeline::new("Gap Test");
    let mut track = timeline.add_video_track("V1");

    let gap = Gap::new(RationalTime::new(24.0, 24.0)); // 1 second
    track.append_gap(gap).expect("Failed to append gap");

    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline
        .write_to_file(temp_file.path())
        .expect("Failed to write");
}

#[test]
fn test_audio_track() {
    let mut timeline = Timeline::new("Audio Test");
    let mut audio_track = timeline.add_audio_track("A1");

    let source_range = TimeRange::new(
        RationalTime::new(0.0, 48000.0),
        RationalTime::new(48000.0, 48000.0), // 1 second at 48kHz
    );

    let mut clip = Clip::new("Audio Clip", source_range);
    clip.set_metadata("seq:channel_mask", "0x0003"); // Stereo

    audio_track.append_clip(clip).expect("Failed to append");

    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline
        .write_to_file(temp_file.path())
        .expect("Failed to write");
}

#[test]
fn test_multiple_tracks() {
    let mut timeline = Timeline::new("Multi-track");

    let mut v1 = timeline.add_video_track("V1");
    let mut v2 = timeline.add_video_track("V2");
    let mut a1 = timeline.add_audio_track("A1");

    let range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(24.0, 24.0),
    );

    v1.append_clip(Clip::new("V1 Clip", range)).unwrap();
    v2.append_clip(Clip::new("V2 Clip", range)).unwrap();
    a1.append_clip(Clip::new("A1 Clip", range)).unwrap();

    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline
        .write_to_file(temp_file.path())
        .expect("Failed to write");

    // Verify file is valid JSON
    let contents = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(contents.contains("OTIO_SCHEMA"));
    assert!(contents.contains("V1 Clip"));
}

#[test]
fn test_rational_time() {
    let rt = RationalTime::new(48.0, 24.0);
    assert!((rt.to_seconds() - 2.0).abs() < f64::EPSILON);

    let rt2 = RationalTime::from_seconds(2.0, 24.0);
    assert!((rt2.value - 48.0).abs() < f64::EPSILON);
}

#[test]
fn test_time_range() {
    let tr = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    );

    let end = tr.end_time();
    assert!((end.value - 48.0).abs() < f64::EPSILON);
}

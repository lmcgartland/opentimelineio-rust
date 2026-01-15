//! Tests for extended OTIO features added in this phase.

// Allow exact float comparisons in tests - values are known exactly
#![allow(clippy::float_cmp)]
// Allow similar names in tests for clarity
#![allow(clippy::similar_names)]

use otio_rs::{
    generator_reference::kinds as gen_kinds,
    image_sequence_reference::MissingFramePolicy,
    marker::colors,
    Clip, Effect, ExternalReference, FreezeFrame, Gap, GeneratorReference,
    ImageSequenceReference, LinearTimeWarp, Marker, MissingReference, RationalTime, Stack,
    TimeRange, Timeline, Track, TrackKind,
};

// ============================================================================
// MissingReference tests
// ============================================================================

#[test]
fn test_missing_reference_create() {
    let missing = MissingReference::new();
    // MissingReference doesn't have many properties, just metadata
    drop(missing);
}

#[test]
fn test_missing_reference_default() {
    let missing = MissingReference::default();
    drop(missing);
}

#[test]
fn test_clip_with_missing_reference() {
    let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0));
    let mut clip = Clip::new("Offline Clip", range);

    let missing = MissingReference::new();
    clip.set_missing_reference(missing).unwrap();
}

// ============================================================================
// GeneratorReference tests
// ============================================================================

#[test]
fn test_generator_reference_create() {
    let gen = GeneratorReference::new("Color Bars", gen_kinds::SMPTE_BARS);
    assert_eq!(gen.name(), "Color Bars");
    assert_eq!(gen.generator_kind(), gen_kinds::SMPTE_BARS);
}

#[test]
fn test_generator_reference_black() {
    let gen = GeneratorReference::black("Black Video");
    assert_eq!(gen.name(), "Black Video");
    assert_eq!(gen.generator_kind(), gen_kinds::BLACK);
}

#[test]
fn test_generator_reference_smpte_bars() {
    let gen = GeneratorReference::smpte_bars("Bars and Tone");
    assert_eq!(gen.generator_kind(), gen_kinds::SMPTE_BARS);
}

#[test]
fn test_generator_reference_set_kind() {
    let mut gen = GeneratorReference::new("Test", gen_kinds::BLACK);
    gen.set_generator_kind(gen_kinds::SOLID_COLOR);
    assert_eq!(gen.generator_kind(), gen_kinds::SOLID_COLOR);
}

#[test]
fn test_generator_reference_available_range() {
    let mut gen = GeneratorReference::new("Test", gen_kinds::BLACK);
    let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(240.0, 24.0));
    gen.set_available_range(range).unwrap();

    let retrieved = gen.available_range().unwrap();
    assert_eq!(retrieved.duration.value, 240.0);
}

#[test]
fn test_clip_with_generator_reference() {
    let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(240.0, 24.0));
    let mut clip = Clip::new("Generated Clip", range);

    let mut gen = GeneratorReference::smpte_bars("Bars");
    gen.set_available_range(range).unwrap();
    clip.set_generator_reference(gen).unwrap();
}

// ============================================================================
// LinearTimeWarp tests
// ============================================================================

#[test]
fn test_linear_time_warp_create() {
    let effect = LinearTimeWarp::new("Speed Up", 2.0);
    assert_eq!(effect.name(), "Speed Up");
    assert!((effect.time_scalar() - 2.0).abs() < 0.001);
}

#[test]
fn test_linear_time_warp_slow_motion() {
    let effect = LinearTimeWarp::slow_motion("Slow Mo", 0.5);
    assert!((effect.time_scalar() - 0.5).abs() < 0.001);
}

#[test]
fn test_linear_time_warp_reverse() {
    let effect = LinearTimeWarp::reverse("Reverse");
    assert!((effect.time_scalar() - (-1.0)).abs() < 0.001);
}

#[test]
fn test_linear_time_warp_fast_forward() {
    let effect = LinearTimeWarp::fast_forward("4x Speed", 4.0);
    assert!((effect.time_scalar() - 4.0).abs() < 0.001);
}

#[test]
fn test_linear_time_warp_set_scalar() {
    let mut effect = LinearTimeWarp::new("Speed", 1.0);
    effect.set_time_scalar(3.0);
    assert!((effect.time_scalar() - 3.0).abs() < 0.001);
}

// ============================================================================
// FreezeFrame tests
// ============================================================================

#[test]
fn test_freeze_frame_create() {
    let effect = FreezeFrame::new("Hold Frame");
    assert_eq!(effect.name(), "Hold Frame");
}

// ============================================================================
// Clip Marker/Effect attachment tests
// ============================================================================

#[test]
fn test_clip_add_marker() {
    let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0));
    let mut clip = Clip::new("Test Clip", range);

    let marker_range = TimeRange::new(RationalTime::new(10.0, 24.0), RationalTime::new(5.0, 24.0));
    let marker = Marker::new("Review", marker_range, colors::RED);

    assert_eq!(clip.markers_count(), 0);
    clip.add_marker(marker).unwrap();
    assert_eq!(clip.markers_count(), 1);
}

#[test]
fn test_clip_add_multiple_markers() {
    let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(100.0, 24.0));
    let mut clip = Clip::new("Test Clip", range);

    for i in 0..5 {
        let marker_range =
            TimeRange::new(RationalTime::new(f64::from(i) * 10.0, 24.0), RationalTime::new(5.0, 24.0));
        let marker = Marker::new(&format!("Marker {i}"), marker_range, colors::GREEN);
        clip.add_marker(marker).unwrap();
    }

    assert_eq!(clip.markers_count(), 5);
}

#[test]
fn test_clip_add_effect() {
    let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0));
    let mut clip = Clip::new("Test Clip", range);

    let effect = Effect::new("Color Grade", "ColorCorrection");

    assert_eq!(clip.effects_count(), 0);
    clip.add_effect(effect).unwrap();
    assert_eq!(clip.effects_count(), 1);
}

#[test]
fn test_clip_add_linear_time_warp() {
    let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0));
    let mut clip = Clip::new("Slow Motion Clip", range);

    let effect = LinearTimeWarp::slow_motion("Slo Mo", 0.5);

    assert_eq!(clip.effects_count(), 0);
    clip.add_linear_time_warp(effect).unwrap();
    assert_eq!(clip.effects_count(), 1);
}

// ============================================================================
// Track marker tests
// ============================================================================

#[test]
fn test_track_add_marker() {
    let mut track = Track::new_video("V1");

    let marker_range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0));
    let marker = Marker::new("Track Marker", marker_range, colors::BLUE);

    assert_eq!(track.markers_count(), 0);
    track.add_marker(marker).unwrap();
    assert_eq!(track.markers_count(), 1);
}

// ============================================================================
// TrackKind tests
// ============================================================================

#[test]
fn test_track_kind_video() {
    let track = Track::new_video("V1");
    assert_eq!(track.kind(), TrackKind::Video);
}

#[test]
fn test_track_kind_audio() {
    let track = Track::new_audio("A1");
    assert_eq!(track.kind(), TrackKind::Audio);
}

#[test]
fn test_track_set_kind() {
    let mut track = Track::new_video("Track");
    assert_eq!(track.kind(), TrackKind::Video);

    track.set_kind(TrackKind::Audio);
    assert_eq!(track.kind(), TrackKind::Audio);
}

// ============================================================================
// Time transform tests
// ============================================================================

#[test]
fn test_track_range_of_child_at_index() {
    let mut track = Track::new_video("V1");

    // Add a clip (48 frames)
    let clip1_range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0));
    let clip1 = Clip::new("Clip 1", clip1_range);
    track.append_clip(clip1).unwrap();

    // Add a gap (24 frames)
    let gap = Gap::new(RationalTime::new(24.0, 24.0));
    track.append_gap(gap).unwrap();

    // Add another clip (72 frames)
    let clip2_range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(72.0, 24.0));
    let clip2 = Clip::new("Clip 2", clip2_range);
    track.append_clip(clip2).unwrap();

    // Check ranges
    let range0 = track.range_of_child_at_index(0).unwrap();
    assert_eq!(range0.start_time.value, 0.0);
    assert_eq!(range0.duration.value, 48.0);

    let range1 = track.range_of_child_at_index(1).unwrap();
    assert_eq!(range1.start_time.value, 48.0);
    assert_eq!(range1.duration.value, 24.0);

    let range2 = track.range_of_child_at_index(2).unwrap();
    assert_eq!(range2.start_time.value, 72.0);
    assert_eq!(range2.duration.value, 72.0);
}

#[test]
fn test_track_trimmed_range() {
    let mut track = Track::new_video("V1");

    // Add clips totaling 144 frames
    let clip1_range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0));
    let clip1 = Clip::new("Clip 1", clip1_range);
    track.append_clip(clip1).unwrap();

    let gap = Gap::new(RationalTime::new(24.0, 24.0));
    track.append_gap(gap).unwrap();

    let clip2_range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(72.0, 24.0));
    let clip2 = Clip::new("Clip 2", clip2_range);
    track.append_clip(clip2).unwrap();

    let trimmed = track.trimmed_range().unwrap();
    assert_eq!(trimmed.duration.value, 144.0); // 48 + 24 + 72
}

#[test]
fn test_stack_range_of_child_at_index() {
    let mut stack = Stack::new("Layers");

    // Add two tracks with different durations
    let mut track1 = Track::new_video("V1");
    let clip1 = Clip::new(
        "Clip 1",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track1.append_clip(clip1).unwrap();
    stack.append_track(track1).unwrap();

    let mut track2 = Track::new_video("V2");
    let clip2 = Clip::new(
        "Clip 2",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(72.0, 24.0)),
    );
    track2.append_clip(clip2).unwrap();
    stack.append_track(track2).unwrap();

    // In a stack, each child starts at 0
    let range0 = stack.range_of_child_at_index(0).unwrap();
    assert_eq!(range0.start_time.value, 0.0);
    assert_eq!(range0.duration.value, 48.0);

    let range1 = stack.range_of_child_at_index(1).unwrap();
    assert_eq!(range1.start_time.value, 0.0);
    assert_eq!(range1.duration.value, 72.0);
}

#[test]
fn test_stack_trimmed_range() {
    let mut stack = Stack::new("Layers");

    // Add tracks with different durations - trimmed range should be the max
    let mut track1 = Track::new_video("V1");
    let clip1 = Clip::new(
        "Clip 1",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track1.append_clip(clip1).unwrap();
    stack.append_track(track1).unwrap();

    let mut track2 = Track::new_video("V2");
    let clip2 = Clip::new(
        "Clip 2",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(72.0, 24.0)),
    );
    track2.append_clip(clip2).unwrap();
    stack.append_track(track2).unwrap();

    let trimmed = stack.trimmed_range().unwrap();
    assert_eq!(trimmed.duration.value, 72.0); // max of 48 and 72
}

// ============================================================================
// Timeline name and duration tests
// ============================================================================

#[test]
fn test_timeline_name() {
    let timeline = Timeline::new("My Timeline");
    assert_eq!(timeline.name(), "My Timeline");
}

#[test]
fn test_timeline_global_start_time() {
    let mut timeline = Timeline::new("Test");
    timeline
        .set_global_start_time(RationalTime::new(86400.0, 24.0))
        .unwrap(); // 1 hour at 24fps

    let start = timeline.global_start_time().unwrap();
    assert_eq!(start.value, 86400.0);
    assert_eq!(start.rate, 24.0);
}

#[test]
fn test_timeline_duration() {
    let mut timeline = Timeline::new("Test");

    let mut track = timeline.add_video_track("V1");
    let clip = Clip::new(
        "Clip",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(240.0, 24.0)),
    );
    track.append_clip(clip).unwrap();

    let duration = timeline.duration().unwrap();
    assert_eq!(duration.value, 240.0);
}

// ============================================================================
// ExternalReference additional accessor tests
// ============================================================================

#[test]
fn test_external_ref_target_url() {
    let ext_ref = ExternalReference::new("/path/to/video.mov");
    assert_eq!(ext_ref.target_url(), "/path/to/video.mov");
}

#[test]
fn test_external_ref_available_range() {
    let mut ext_ref = ExternalReference::new("/path/to/video.mov");
    let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(1000.0, 24.0));
    ext_ref.set_available_range(range).unwrap();

    let retrieved = ext_ref.available_range().unwrap();
    assert_eq!(retrieved.duration.value, 1000.0);
}

// ============================================================================
// Integration tests
// ============================================================================

#[test]
fn test_full_timeline_with_new_features() {
    let mut timeline = Timeline::new("Feature Demo");
    timeline
        .set_global_start_time(RationalTime::new(0.0, 24.0))
        .unwrap();

    // Add video track with markers
    let mut v1 = timeline.add_video_track("V1");

    // Add marker to track
    let track_marker_range =
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0));
    let track_marker = Marker::new("Chapter 1", track_marker_range, colors::CYAN);
    v1.add_marker(track_marker).unwrap();

    // Add clip with effects
    let clip_range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(240.0, 24.0));
    let mut clip = Clip::new("Main Shot", clip_range);

    // Add marker to clip
    let clip_marker_range =
        TimeRange::new(RationalTime::new(100.0, 24.0), RationalTime::new(10.0, 24.0));
    let clip_marker = Marker::new("Review Point", clip_marker_range, colors::RED);
    clip.add_marker(clip_marker).unwrap();

    // Add effect to clip
    let effect = Effect::new("Grade", "ColorCorrection");
    clip.add_effect(effect).unwrap();

    // Add speed ramp
    let speed = LinearTimeWarp::new("Speed Ramp", 1.5);
    clip.add_linear_time_warp(speed).unwrap();

    assert_eq!(clip.markers_count(), 1);
    assert_eq!(clip.effects_count(), 2); // Effect + LinearTimeWarp

    v1.append_clip(clip).unwrap();

    // Add a gap
    let gap = Gap::new(RationalTime::new(24.0, 24.0));
    v1.append_gap(gap).unwrap();

    // Add clip with generator reference
    let gen_clip_range =
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(120.0, 24.0));
    let mut gen_clip = Clip::new("Color Bars", gen_clip_range);
    let mut gen_ref = GeneratorReference::smpte_bars("Bars");
    gen_ref.set_available_range(gen_clip_range).unwrap();
    gen_clip.set_generator_reference(gen_ref).unwrap();
    v1.append_clip(gen_clip).unwrap();

    // Verify timeline structure
    assert_eq!(v1.markers_count(), 1);
    assert_eq!(v1.children_count(), 3); // clip + gap + clip
    assert_eq!(v1.kind(), TrackKind::Video);

    // Test time transforms
    let range0 = v1.range_of_child_at_index(0).unwrap();
    assert_eq!(range0.duration.value, 240.0);

    let range1 = v1.range_of_child_at_index(1).unwrap();
    assert_eq!(range1.start_time.value, 240.0);
    assert_eq!(range1.duration.value, 24.0);

    let trimmed = v1.trimmed_range().unwrap();
    assert_eq!(trimmed.duration.value, 384.0); // 240 + 24 + 120

    // Timeline duration
    let duration = timeline.duration().unwrap();
    assert_eq!(duration.value, 384.0);
}

// ============================================================================
// Parent navigation tests
// ============================================================================

#[test]
fn test_clip_parent_track() {
    use otio_rs::{Composable, ParentRef};

    let mut timeline = Timeline::new("Parent Test");
    let mut track = timeline.add_video_track("V1");

    let clip = Clip::new(
        "Test Clip",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track.append_clip(clip).unwrap();

    // Iterate and check parent
    for child in track.children() {
        if let Composable::Clip(clip_ref) = child {
            let parent = clip_ref.parent();
            assert!(parent.is_some());
            if let Some(ParentRef::Track(track_ref)) = parent {
                assert_eq!(track_ref.name(), "V1");
            } else {
                panic!("Expected Track parent");
            }
        }
    }
}

#[test]
fn test_gap_parent_track() {
    use otio_rs::{Composable, ParentRef};

    let mut timeline = Timeline::new("Parent Test");
    let mut track = timeline.add_video_track("V1");

    let gap = Gap::new(RationalTime::new(24.0, 24.0));
    track.append_gap(gap).unwrap();

    // Iterate and check parent
    for child in track.children() {
        if let Composable::Gap(gap_ref) = child {
            let parent = gap_ref.parent();
            assert!(parent.is_some());
            if let Some(ParentRef::Track(track_ref)) = parent {
                assert_eq!(track_ref.name(), "V1");
            } else {
                panic!("Expected Track parent");
            }
        }
    }
}

#[test]
fn test_track_parent_stack() {
    use otio_rs::Composable;

    let mut timeline = Timeline::new("Parent Test");
    let _ = timeline.add_video_track("V1");

    // Access through the timeline's tracks
    let tracks = timeline.tracks();
    for child in tracks.children() {
        if let Composable::Track(track_ref) = child {
            let parent = track_ref.parent();
            assert!(parent.is_some());
            // The parent should be the root stack of the timeline
        }
    }
}

// ============================================================================
// find_clips search tests
// ============================================================================

#[test]
fn test_track_find_clips() {
    let mut track = Track::new_video("V1");

    let clip1 = Clip::new(
        "Clip A",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track.append_clip(clip1).unwrap();

    let gap = Gap::new(RationalTime::new(24.0, 24.0));
    track.append_gap(gap).unwrap();

    let clip2 = Clip::new(
        "Clip B",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(72.0, 24.0)),
    );
    track.append_clip(clip2).unwrap();

    let clips: Vec<_> = track.find_clips().collect();
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[0].name(), "Clip A");
    assert_eq!(clips[1].name(), "Clip B");
}

#[test]
fn test_track_find_clips_count() {
    let mut track = Track::new_video("V1");

    for i in 0..5 {
        let clip = Clip::new(
            &format!("Clip {i}"),
            TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0)),
        );
        track.append_clip(clip).unwrap();
    }

    let iter = track.find_clips();
    assert_eq!(iter.count(), 5);
}

#[test]
fn test_stack_find_clips_recursive() {
    let mut stack = Stack::new("Root");

    // Add a track with clips
    let mut track1 = Track::new_video("V1");
    let clip1 = Clip::new(
        "V1 Clip",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track1.append_clip(clip1).unwrap();
    stack.append_track(track1).unwrap();

    // Add another track with clips
    let mut track2 = Track::new_audio("A1");
    let clip2 = Clip::new(
        "A1 Clip 1",
        TimeRange::new(RationalTime::new(0.0, 48000.0), RationalTime::new(48000.0, 48000.0)),
    );
    let clip3 = Clip::new(
        "A1 Clip 2",
        TimeRange::new(RationalTime::new(0.0, 48000.0), RationalTime::new(96000.0, 48000.0)),
    );
    track2.append_clip(clip2).unwrap();
    track2.append_clip(clip3).unwrap();
    stack.append_track(track2).unwrap();

    // Find all clips recursively
    let clips: Vec<_> = stack.find_clips().collect();
    assert_eq!(clips.len(), 3);

    let names: Vec<_> = clips.iter().map(otio_rs::ClipRef::name).collect();
    assert!(names.contains(&"V1 Clip".to_string()));
    assert!(names.contains(&"A1 Clip 1".to_string()));
    assert!(names.contains(&"A1 Clip 2".to_string()));
}

#[test]
fn test_timeline_find_clips() {
    let mut timeline = Timeline::new("Search Test");

    // Add video track with clips
    let mut v1 = timeline.add_video_track("V1");
    let clip1 = Clip::new(
        "Video Clip 1",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    let clip2 = Clip::new(
        "Video Clip 2",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(72.0, 24.0)),
    );
    v1.append_clip(clip1).unwrap();
    v1.append_clip(clip2).unwrap();

    // Add audio track with a clip
    let mut a1 = timeline.add_audio_track("A1");
    let clip3 = Clip::new(
        "Audio Clip",
        TimeRange::new(RationalTime::new(0.0, 48000.0), RationalTime::new(48000.0, 48000.0)),
    );
    a1.append_clip(clip3).unwrap();

    // Find all clips in timeline
    let clips: Vec<_> = timeline.find_clips().collect();
    assert_eq!(clips.len(), 3);
}

#[test]
fn test_find_clips_iterator_reset() {
    let mut track = Track::new_video("V1");

    let clip1 = Clip::new(
        "Clip 1",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    let clip2 = Clip::new(
        "Clip 2",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();

    let mut iter = track.find_clips();

    // Consume once
    let first_pass: Vec<_> = iter.by_ref().collect();
    assert_eq!(first_pass.len(), 2);

    // Reset and consume again
    iter.reset();
    let second_pass: Vec<_> = iter.collect();
    assert_eq!(second_pass.len(), 2);
}

#[test]
fn test_find_clips_empty_track() {
    let track = Track::new_video("Empty");
    let clips: Vec<_> = track.find_clips().collect();
    assert!(clips.is_empty());
}

#[test]
fn test_find_clips_empty_timeline() {
    let timeline = Timeline::new("Empty");
    let clips: Vec<_> = timeline.find_clips().collect();
    assert!(clips.is_empty());
}

// ============================================================================
// String serialization tests
// ============================================================================

#[test]
fn test_timeline_to_json_string() {
    let timeline = Timeline::new("JSON Test");
    let json = timeline.to_json_string().unwrap();

    // Verify it's valid JSON with expected content
    assert!(json.contains("JSON Test"));
    assert!(json.contains("OTIO_SCHEMA"));
    assert!(json.contains("Timeline"));
}

#[test]
fn test_timeline_from_json_string_roundtrip() {
    // Create a timeline with content
    let mut timeline = Timeline::new("Roundtrip Test");
    timeline
        .set_global_start_time(RationalTime::new(86400.0, 24.0))
        .unwrap();

    let mut track = timeline.add_video_track("V1");
    let clip = Clip::new(
        "Test Clip",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track.append_clip(clip).unwrap();

    // Serialize to JSON
    let json = timeline.to_json_string().unwrap();

    // Deserialize back
    let restored = Timeline::from_json_string(&json).unwrap();

    // Verify content
    assert_eq!(restored.name(), "Roundtrip Test");
    let clips: Vec<_> = restored.find_clips().collect();
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].name(), "Test Clip");
}

#[test]
fn test_timeline_from_json_string_with_metadata() {
    use otio_rs::HasMetadata;

    let mut timeline = Timeline::new("Metadata Test");
    timeline.set_metadata("key1", "value1");
    timeline.set_metadata("project", "My Project");

    let json = timeline.to_json_string().unwrap();
    let restored = Timeline::from_json_string(&json).unwrap();

    assert_eq!(restored.get_metadata("key1"), Some("value1".to_string()));
    assert_eq!(
        restored.get_metadata("project"),
        Some("My Project".to_string())
    );
}

#[test]
fn test_timeline_from_json_string_invalid() {
    // Invalid JSON
    let result = Timeline::from_json_string("not valid json");
    assert!(result.is_err());

    // Valid JSON but not a timeline
    let result = Timeline::from_json_string(r#"{"foo": "bar"}"#);
    assert!(result.is_err());
}

#[test]
fn test_timeline_json_complex_structure() {
    let mut timeline = Timeline::new("Complex Test");

    // Add multiple tracks
    let mut v1 = timeline.add_video_track("V1");
    let mut v2 = timeline.add_video_track("V2");
    let mut a1 = timeline.add_audio_track("A1");

    // Add clips to each
    for i in 0..3 {
        let clip = Clip::new(
            &format!("V1 Clip {i}"),
            TimeRange::new(
                RationalTime::new(f64::from(i * 48), 24.0),
                RationalTime::new(48.0, 24.0),
            ),
        );
        v1.append_clip(clip).unwrap();
    }

    let clip = Clip::new(
        "V2 Clip",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(96.0, 24.0)),
    );
    v2.append_clip(clip).unwrap();

    let clip = Clip::new(
        "A1 Clip",
        TimeRange::new(
            RationalTime::new(0.0, 48000.0),
            RationalTime::new(96000.0, 48000.0),
        ),
    );
    a1.append_clip(clip).unwrap();

    // Roundtrip
    let json = timeline.to_json_string().unwrap();
    let restored = Timeline::from_json_string(&json).unwrap();

    // Verify all clips are present
    let clips: Vec<_> = restored.find_clips().collect();
    assert_eq!(clips.len(), 5);
}

// ============================================================================
// ImageSequenceReference tests
// ============================================================================

#[test]
fn test_image_sequence_reference_create() {
    let seq = ImageSequenceReference::new(
        "/path/to/render/",
        "shot_",
        ".exr",
        1,    // start_frame
        1,    // frame_step
        24.0, // rate
        4,    // frame_zero_padding
    );

    assert_eq!(seq.target_url_base(), "/path/to/render/");
    assert_eq!(seq.name_prefix(), "shot_");
    assert_eq!(seq.name_suffix(), ".exr");
    assert_eq!(seq.start_frame(), 1);
    assert_eq!(seq.frame_step(), 1);
    assert!((seq.rate() - 24.0).abs() < 0.001);
    assert_eq!(seq.frame_zero_padding(), 4);
}

#[test]
fn test_image_sequence_reference_setters() {
    let mut seq = ImageSequenceReference::new("/base/", "prefix_", ".dpx", 1, 1, 24.0, 4);

    seq.set_target_url_base("/new/path/");
    seq.set_name_prefix("new_prefix_");
    seq.set_name_suffix(".exr");
    seq.set_start_frame(100);
    seq.set_frame_step(2);
    seq.set_rate(30.0);
    seq.set_frame_zero_padding(6);

    assert_eq!(seq.target_url_base(), "/new/path/");
    assert_eq!(seq.name_prefix(), "new_prefix_");
    assert_eq!(seq.name_suffix(), ".exr");
    assert_eq!(seq.start_frame(), 100);
    assert_eq!(seq.frame_step(), 2);
    assert!((seq.rate() - 30.0).abs() < 0.001);
    assert_eq!(seq.frame_zero_padding(), 6);
}

#[test]
fn test_image_sequence_reference_missing_frame_policy() {
    let mut seq = ImageSequenceReference::new("/base/", "shot_", ".exr", 1, 1, 24.0, 4);

    // Default should be Error
    assert_eq!(seq.missing_frame_policy(), MissingFramePolicy::Error);

    seq.set_missing_frame_policy(MissingFramePolicy::Hold);
    assert_eq!(seq.missing_frame_policy(), MissingFramePolicy::Hold);

    seq.set_missing_frame_policy(MissingFramePolicy::Black);
    assert_eq!(seq.missing_frame_policy(), MissingFramePolicy::Black);

    seq.set_missing_frame_policy(MissingFramePolicy::Error);
    assert_eq!(seq.missing_frame_policy(), MissingFramePolicy::Error);
}

#[test]
fn test_image_sequence_reference_available_range() {
    let mut seq = ImageSequenceReference::new("/base/", "shot_", ".exr", 1, 1, 24.0, 4);

    // Initially no available range
    assert!(seq.available_range().is_none());

    // Set available range (100 frames at 24fps)
    seq.set_available_range(TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(100.0, 24.0),
    ))
    .unwrap();

    let range = seq.available_range().expect("Should have range");
    assert_eq!(range.start_time.value, 0.0);
    assert_eq!(range.duration.value, 100.0);
}

#[test]
fn test_image_sequence_reference_number_of_images() {
    let mut seq = ImageSequenceReference::new("/base/", "shot_", ".exr", 1, 1, 24.0, 4);

    seq.set_available_range(TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(100.0, 24.0),
    ))
    .unwrap();

    assert_eq!(seq.number_of_images(), 100);
}

#[test]
fn test_image_sequence_reference_target_url_for_image_number() {
    let mut seq = ImageSequenceReference::new("/base/", "shot_", ".exr", 1, 1, 24.0, 4);

    seq.set_available_range(TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(100.0, 24.0),
    ))
    .unwrap();

    // Get URL for first image
    let url = seq.target_url_for_image_number(0).unwrap();
    assert!(url.contains("/base/"));
    assert!(url.contains("shot_"));
    assert!(url.contains(".exr"));
}

#[test]
fn test_image_sequence_reference_end_frame() {
    let mut seq = ImageSequenceReference::new("/base/", "shot_", ".exr", 1, 1, 24.0, 4);

    seq.set_available_range(TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(100.0, 24.0),
    ))
    .unwrap();

    // End frame should be start + (number_of_images - 1) * step
    let end = seq.end_frame();
    assert!(end > seq.start_frame());
}

#[test]
fn test_clip_with_image_sequence_reference() {
    let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(100.0, 24.0));
    let mut clip = Clip::new("VFX Shot", range);

    let mut seq = ImageSequenceReference::new("/renders/", "comp_", ".exr", 1001, 1, 24.0, 4);
    seq.set_available_range(range).unwrap();

    clip.set_image_sequence_reference(seq).unwrap();
}

#[test]
fn test_image_sequence_reference_metadata() {
    use otio_rs::HasMetadata;

    let mut seq = ImageSequenceReference::new("/base/", "shot_", ".exr", 1, 1, 24.0, 4);

    seq.set_metadata("colorspace", "ACEScg");
    seq.set_metadata("resolution", "4K");

    assert_eq!(seq.get_metadata("colorspace"), Some("ACEScg".to_string()));
    assert_eq!(seq.get_metadata("resolution"), Some("4K".to_string()));
}

#[test]
fn test_image_sequence_reference_frame_step() {
    // Test with frame step of 2 (every other frame)
    let mut seq = ImageSequenceReference::new("/base/", "shot_", ".exr", 1, 2, 24.0, 4);

    seq.set_available_range(TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(100.0, 24.0),
    ))
    .unwrap();

    // With step of 2, should have half as many images
    let num_images = seq.number_of_images();
    assert_eq!(num_images, 50);
}

// ============================================================================
// Time coordinate transform tests
// ============================================================================

#[test]
fn test_clip_range_in_parent() {
    use otio_rs::Composable;

    let mut timeline = Timeline::new("Time Transform Test");
    let mut track = timeline.add_video_track("V1");

    // Add a 48-frame clip
    let clip = Clip::new(
        "Test Clip",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track.append_clip(clip).unwrap();

    // Now iterate to get the ClipRef
    for child in track.children() {
        if let Composable::Clip(clip_ref) = child {
            let range = clip_ref.range_in_parent().unwrap();
            // First clip in track starts at 0
            assert_eq!(range.start_time.value, 0.0);
            assert_eq!(range.duration.value, 48.0);
        }
    }
}

#[test]
fn test_clip_range_in_parent_with_gap() {
    use otio_rs::Composable;

    let mut timeline = Timeline::new("Time Transform Test");
    let mut track = timeline.add_video_track("V1");

    // Add a gap first (24 frames)
    let gap = Gap::new(RationalTime::new(24.0, 24.0));
    track.append_gap(gap).unwrap();

    // Add a 48-frame clip
    let clip = Clip::new(
        "Test Clip",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track.append_clip(clip).unwrap();

    // Now iterate to get the ClipRef
    let mut clip_found = false;
    for child in track.children() {
        if let Composable::Clip(clip_ref) = child {
            let range = clip_ref.range_in_parent().unwrap();
            // Clip starts after the 24-frame gap
            assert_eq!(range.start_time.value, 24.0);
            assert_eq!(range.duration.value, 48.0);
            clip_found = true;
        }
    }
    assert!(clip_found);
}

#[test]
fn test_gap_range_in_parent() {
    use otio_rs::Composable;

    let mut timeline = Timeline::new("Time Transform Test");
    let mut track = timeline.add_video_track("V1");

    // Add a clip first (48 frames)
    let clip = Clip::new(
        "Test Clip",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track.append_clip(clip).unwrap();

    // Add a gap (24 frames)
    let gap = Gap::new(RationalTime::new(24.0, 24.0));
    track.append_gap(gap).unwrap();

    // Now iterate to get the GapRef
    let mut gap_found = false;
    for child in track.children() {
        if let Composable::Gap(gap_ref) = child {
            let range = gap_ref.range_in_parent().unwrap();
            // Gap starts after the 48-frame clip
            assert_eq!(range.start_time.value, 48.0);
            assert_eq!(range.duration.value, 24.0);
            gap_found = true;
        }
    }
    assert!(gap_found);
}

#[test]
fn test_multiple_clips_range_in_parent() {
    use otio_rs::Composable;

    let mut timeline = Timeline::new("Time Transform Test");
    let mut track = timeline.add_video_track("V1");

    // Add three clips of 24 frames each
    for i in 0..3 {
        let clip = Clip::new(
            &format!("Clip {i}"),
            TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0)),
        );
        track.append_clip(clip).unwrap();
    }

    // Verify each clip's position in parent
    let mut expected_start = 0.0;
    for child in track.children() {
        if let Composable::Clip(clip_ref) = child {
            let range = clip_ref.range_in_parent().unwrap();
            assert_eq!(range.start_time.value, expected_start);
            assert_eq!(range.duration.value, 24.0);
            expected_start += 24.0;
        }
    }
}

// ============================================================================
// Edit Algorithm tests
// ============================================================================

#[test]
fn test_track_slice_at_time() {
    let mut timeline = Timeline::new("Slice Test");
    let mut track = timeline.add_video_track("V1");

    // Add a 48-frame clip
    let clip = Clip::new(
        "Original Clip",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    );
    track.append_clip(clip).unwrap();

    // Slice in the middle (at frame 24)
    track
        .slice_at_time(RationalTime::new(24.0, 24.0), true)
        .unwrap();

    // Should now have 2 clips
    let clips: Vec<_> = track.find_clips().collect();
    assert_eq!(clips.len(), 2);
}

#[test]
fn test_track_remove_at_time() {
    let mut timeline = Timeline::new("Remove Test");
    let mut track = timeline.add_video_track("V1");

    // Add three clips
    for i in 0..3 {
        let clip = Clip::new(
            &format!("Clip {i}"),
            TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0)),
        );
        track.append_clip(clip).unwrap();
    }

    // Remove the middle clip (at frame 24+12 = 36)
    track
        .remove_at_time(RationalTime::new(36.0, 24.0), true)
        .unwrap();

    // Should now have 2 clips + 1 gap (since fill_with_gap=true)
    let clips: Vec<_> = track.find_clips().collect();
    assert_eq!(clips.len(), 2);
}

#[test]
fn test_track_remove_without_fill() {
    let mut timeline = Timeline::new("Remove No Fill Test");
    let mut track = timeline.add_video_track("V1");

    // Add two clips
    for i in 0..2 {
        let clip = Clip::new(
            &format!("Clip {i}"),
            TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0)),
        );
        track.append_clip(clip).unwrap();
    }

    // Total duration should be 48 frames
    let range = track.trimmed_range().unwrap();
    assert_eq!(range.duration.value, 48.0);

    // Remove the first clip without filling
    track
        .remove_at_time(RationalTime::new(12.0, 24.0), false)
        .unwrap();

    // Should now have 1 clip and shorter duration
    let clips: Vec<_> = track.find_clips().collect();
    assert_eq!(clips.len(), 1);

    let range = track.trimmed_range().unwrap();
    assert_eq!(range.duration.value, 24.0);
}

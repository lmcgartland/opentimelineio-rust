use otio_rs::*;
use tempfile::NamedTempFile;

/// Test stack composition with multiple clips (clip stack analog).
#[test]
fn test_stack_composition() {
    let mut timeline = Timeline::new("Stack Test Timeline");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0)).unwrap();

    // Create a stack with multiple clips (like a clip stack for compositing)
    let mut stack = Stack::new("Clip Stack");

    let range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0), // 2 seconds
    );

    // Add multiple clips to the stack (simulating layers)
    let clip1 = Clip::new("Background Layer", range);
    let clip2 = Clip::new("Foreground Layer", range);
    let gap = Gap::new(RationalTime::new(24.0, 24.0));

    stack.append_clip(clip1).unwrap();
    stack.append_clip(clip2).unwrap();
    stack.append_gap(gap).unwrap();

    // Add the stack to a track
    let mut track = timeline.add_video_track("V1");
    track.append_stack(stack).unwrap();

    // Write to file
    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline
        .write_to_file(temp_file.path())
        .expect("Failed to write stack composition");

    // Verify JSON contains Stack.1 schema
    let contents = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(contents.contains("\"OTIO_SCHEMA\": \"Stack.1\""));
    assert!(contents.contains("Clip Stack"));
    assert!(contents.contains("Background Layer"));
    assert!(contents.contains("Foreground Layer"));
    assert!(contents.contains("\"OTIO_SCHEMA\": \"Gap.1\""));

    // Read back
    let _reloaded = Timeline::read_from_file(temp_file.path()).expect("Failed to read stack composition");
}

/// Test deep nesting using stacks: Stack > Track > Stack > Clip (multiple levels).
#[test]
fn test_deep_stack_nesting() {
    let mut timeline = Timeline::new("Deep Nesting Timeline");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0)).unwrap();

    // Create innermost clips
    let inner_range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(24.0, 24.0), // 1 second
    );

    // Level 2: Inner stack with clips
    let mut inner_stack = Stack::new("Inner Stack");
    inner_stack.append_clip(Clip::new("Inner Clip A", inner_range)).unwrap();
    inner_stack.append_clip(Clip::new("Inner Clip B", inner_range)).unwrap();

    // Level 1: Track containing the inner stack
    let mut inner_track = Track::new_video("Inner Track");
    inner_track.append_stack(inner_stack).unwrap();
    inner_track.append_clip(Clip::new("Track Clip", inner_range)).unwrap();

    // Level 0: Outer stack containing the track
    let mut outer_stack = Stack::new("Outer Stack");
    outer_stack.append_track(inner_track).unwrap();

    // Add outer stack to main track
    let mut main_track = timeline.add_video_track("Main Track");
    main_track.append_stack(outer_stack).unwrap();

    // Write to file
    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline
        .write_to_file(temp_file.path())
        .expect("Failed to write deeply nested structure");

    // Verify all levels are present
    let contents = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(contents.contains("Deep Nesting Timeline"));
    assert!(contents.contains("Inner Stack"));
    assert!(contents.contains("Outer Stack"));
    assert!(contents.contains("Inner Track"));
    assert!(contents.contains("Inner Clip A"));
    assert!(contents.contains("Inner Clip B"));
    assert!(contents.contains("Track Clip"));

    // Count Stack.1 occurrences (root + outer + inner = at least 3)
    let stack_count = contents.matches("\"OTIO_SCHEMA\": \"Stack.1\"").count();
    assert!(stack_count >= 3, "Expected at least 3 Stack.1 schemas, found {stack_count}");

    // Read back
    let _reloaded = Timeline::read_from_file(temp_file.path()).expect("Failed to read deeply nested structure");
}

/// Test track containing stack items (for versioning/alternatives).
#[test]
fn test_stack_in_track() {
    let mut timeline = Timeline::new("Versioning Timeline");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0)).unwrap();

    let mut track = timeline.add_video_track("Main Track");

    // First: a regular clip
    let clip_range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(24.0, 24.0), // 1 second
    );
    let intro_clip = Clip::new("Intro", clip_range);
    track.append_clip(intro_clip).unwrap();

    // Second: a stack representing alternative versions of a segment
    let mut version_stack = Stack::new("Version Alternatives");

    let alt_range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0), // 2 seconds
    );

    let version_a = Clip::new("Version A - Director's Cut", alt_range);
    let version_b = Clip::new("Version B - Theatrical", alt_range);
    let version_c = Clip::new("Version C - Extended", alt_range);

    version_stack.append_clip(version_a).unwrap();
    version_stack.append_clip(version_b).unwrap();
    version_stack.append_clip(version_c).unwrap();

    track.append_stack(version_stack).unwrap();

    // Third: another regular clip
    let outro_clip = Clip::new("Outro", clip_range);
    track.append_clip(outro_clip).unwrap();

    // Write to file
    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline
        .write_to_file(temp_file.path())
        .expect("Failed to write versioning timeline");

    // Verify structure
    let contents = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(contents.contains("Versioning Timeline"));
    assert!(contents.contains("Version Alternatives"));
    assert!(contents.contains("Version A - Director's Cut"));
    assert!(contents.contains("Version B - Theatrical"));
    assert!(contents.contains("Version C - Extended"));
    assert!(contents.contains("Intro"));
    assert!(contents.contains("Outro"));

    // Count Stack.1 occurrences (root stack + version alternatives stack)
    let stack_count = contents.matches("\"OTIO_SCHEMA\": \"Stack.1\"").count();
    assert!(stack_count >= 2, "Expected at least 2 Stack.1 schemas, found {stack_count}");

    // Read back
    let _reloaded = Timeline::read_from_file(temp_file.path()).expect("Failed to read versioning timeline");
}

/// Test creating standalone tracks and adding them to a stack.
#[test]
fn test_standalone_tracks_in_stack() {
    let mut timeline = Timeline::new("Standalone Tracks Test");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0)).unwrap();

    // Create a stack to hold multiple tracks
    let mut main_stack = Stack::new("Main Stack");

    // Create standalone video track
    let mut video_track = Track::new_video("Standalone Video");
    let video_range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    );
    let video_clip = Clip::new("Video Content", video_range);
    video_track.append_clip(video_clip).unwrap();

    // Create standalone audio track
    let mut audio_track = Track::new_audio("Standalone Audio");
    let audio_range = TimeRange::new(
        RationalTime::new(0.0, 48000.0),
        RationalTime::new(96000.0, 48000.0), // 2 seconds
    );
    let audio_clip = Clip::new("Audio Content", audio_range);
    audio_track.append_clip(audio_clip).unwrap();

    // Add tracks to the stack
    main_stack.append_track(video_track).unwrap();
    main_stack.append_track(audio_track).unwrap();

    // Add the stack as the main content
    let mut root_track = timeline.add_video_track("Root");
    root_track.append_stack(main_stack).unwrap();

    // Write to file
    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline
        .write_to_file(temp_file.path())
        .expect("Failed to write standalone tracks test");

    // Verify structure
    let contents = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(contents.contains("Standalone Video"));
    assert!(contents.contains("Standalone Audio"));
    assert!(contents.contains("Video Content"));
    assert!(contents.contains("Audio Content"));
    assert!(contents.contains("\"kind\": \"Video\""));
    assert!(contents.contains("\"kind\": \"Audio\""));

    // Read back
    let _reloaded = Timeline::read_from_file(temp_file.path()).expect("Failed to read standalone tracks test");
}

/// Test nested stacks (stack containing stacks).
#[test]
fn test_nested_stacks() {
    let mut timeline = Timeline::new("Nested Stacks Timeline");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0)).unwrap();

    let range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(24.0, 24.0),
    );

    // Create child stacks
    let mut child_stack_1 = Stack::new("Child Stack 1");
    child_stack_1.append_clip(Clip::new("Child 1 Clip A", range)).unwrap();
    child_stack_1.append_clip(Clip::new("Child 1 Clip B", range)).unwrap();

    let mut child_stack_2 = Stack::new("Child Stack 2");
    child_stack_2.append_clip(Clip::new("Child 2 Clip A", range)).unwrap();
    child_stack_2.append_gap(Gap::new(RationalTime::new(12.0, 24.0))).unwrap();

    // Create parent stack containing child stacks
    let mut parent_stack = Stack::new("Parent Stack");
    parent_stack.append_stack(child_stack_1).unwrap();
    parent_stack.append_stack(child_stack_2).unwrap();

    // Add to track
    let mut track = timeline.add_video_track("V1");
    track.append_stack(parent_stack).unwrap();

    // Write to file
    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline
        .write_to_file(temp_file.path())
        .expect("Failed to write nested stacks");

    // Verify structure
    let contents = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(contents.contains("Parent Stack"));
    assert!(contents.contains("Child Stack 1"));
    assert!(contents.contains("Child Stack 2"));
    assert!(contents.contains("Child 1 Clip A"));
    assert!(contents.contains("Child 2 Clip A"));

    // Count Stack.1 occurrences (root + parent + 2 children = at least 4)
    let stack_count = contents.matches("\"OTIO_SCHEMA\": \"Stack.1\"").count();
    assert!(stack_count >= 4, "Expected at least 4 Stack.1 schemas, found {stack_count}");

    // Read back
    let _reloaded = Timeline::read_from_file(temp_file.path()).expect("Failed to read nested stacks");
}

/// Test timeline tracks accessor.
#[test]
fn test_timeline_tracks_accessor() {
    let mut timeline = Timeline::new("Tracks Accessor Test");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0)).unwrap();

    // Add some tracks
    let mut v1 = timeline.add_video_track("V1");
    let range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(24.0, 24.0),
    );
    v1.append_clip(Clip::new("Test Clip", range)).unwrap();

    let _ = timeline.add_audio_track("A1");

    // Access the tracks stack
    let tracks = timeline.tracks();

    // Verify we can get the pointer (basic sanity check)
    assert!(!tracks.as_ptr().is_null(), "Tracks stack pointer should not be null");

    // Write to file to verify the timeline is still valid
    let temp_file = NamedTempFile::with_suffix(".otio").unwrap();
    timeline
        .write_to_file(temp_file.path())
        .expect("Failed to write after accessing tracks");

    let contents = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(contents.contains("V1"));
    assert!(contents.contains("A1"));
    assert!(contents.contains("Test Clip"));
}

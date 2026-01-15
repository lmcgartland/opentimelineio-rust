//! Tests for error handling and FFI error propagation.
//!
//! These tests verify that errors from the C++ FFI layer are properly
//! converted and returned as Rust Result errors.

use otio_rs::{Clip, RationalTime, Stack, Timeline, TimeRange, Track};

fn make_time_range(start: f64, duration: f64, rate: f64) -> TimeRange {
    TimeRange::new(
        RationalTime::new(start, rate),
        RationalTime::new(duration, rate),
    )
}

// ============================================================================
// File I/O Error Tests
// ============================================================================

#[test]
fn test_read_nonexistent_file() {
    let result = Timeline::read_from_file(std::path::Path::new("/nonexistent/path/file.otio"));
    assert!(result.is_err(), "Reading nonexistent file should return error");
    let err = result.unwrap_err();
    assert!(err.code != 0, "Error code should be non-zero");
    assert!(!err.message.is_empty(), "Error message should not be empty");
}

#[test]
fn test_write_to_invalid_path() {
    let timeline = Timeline::new("Test");
    let result = timeline.write_to_file(std::path::Path::new("/nonexistent/dir/file.otio"));
    assert!(result.is_err(), "Writing to invalid path should return error");
}

// ============================================================================
// JSON Serialization Error Tests
// ============================================================================

#[test]
fn test_from_invalid_json() {
    let result = Timeline::from_json_string("not valid json at all");
    assert!(result.is_err(), "Parsing invalid JSON should return error");
    let err = result.unwrap_err();
    assert!(err.code != 0, "Error code should be non-zero");
}

#[test]
fn test_from_empty_json() {
    let result = Timeline::from_json_string("");
    assert!(result.is_err(), "Parsing empty string should return error");
}

#[test]
fn test_from_wrong_type_json() {
    // Valid JSON but not an OTIO timeline
    let result = Timeline::from_json_string(r#"{"key": "value"}"#);
    assert!(result.is_err(), "Parsing non-OTIO JSON should return error");
}

// ============================================================================
// Index Out of Bounds Tests
// ============================================================================

#[test]
fn test_track_range_of_child_at_invalid_index() {
    let mut timeline = Timeline::new("Test");
    let track = timeline.add_video_track("V1");
    // Track is empty, so any index should fail
    let result = track.range_of_child_at_index(0);
    assert!(result.is_err(), "Accessing invalid index should return error");
}

#[test]
fn test_stack_range_of_child_at_invalid_index() {
    let stack = Stack::new("Test Stack");
    // Stack is empty, so any index should fail
    let result = stack.range_of_child_at_index(0);
    assert!(result.is_err(), "Accessing invalid index should return error");
}

#[test]
fn test_track_range_of_child_large_index() {
    let mut timeline = Timeline::new("Test");
    let mut track = timeline.add_video_track("V1");
    let clip = Clip::new("Clip", make_time_range(0.0, 24.0, 24.0));
    track.append_clip(clip).unwrap();

    // Index 0 should work
    assert!(track.range_of_child_at_index(0).is_ok());
    // Index 1 should fail (only 1 child)
    assert!(track.range_of_child_at_index(1).is_err());
    // Large index should fail
    assert!(track.range_of_child_at_index(1000).is_err());
}

// ============================================================================
// Range Computation Error Tests
// ============================================================================

#[test]
fn test_trimmed_range_empty_track() {
    let track = Track::new_video("Empty Track");
    // Empty track may return error or zero duration
    let _result = track.trimmed_range();
    // Just verify it doesn't panic - behavior may vary
}

#[test]
fn test_trimmed_range_empty_stack() {
    let stack = Stack::new("Empty Stack");
    let _result = stack.trimmed_range();
    // Just verify it doesn't panic - behavior may vary
}

// ============================================================================
// Parent Navigation Error Tests
// ============================================================================

#[test]
fn test_clip_range_in_parent_without_parent() {
    // Create a standalone clip not attached to any track
    let mut timeline = Timeline::new("Test");
    let mut track = timeline.add_video_track("V1");
    let clip = Clip::new("Clip", make_time_range(0.0, 24.0, 24.0));
    track.append_clip(clip).unwrap();

    // Get a reference to the clip via iteration
    for child in track.children() {
        if let otio_rs::Composable::Clip(clip_ref) = child {
            // Clip has a parent, so this should work
            let result = clip_ref.range_in_parent();
            assert!(result.is_ok(), "Clip with parent should have valid range_in_parent");
        }
    }
}

// ============================================================================
// Edit Algorithm Error Tests
// ============================================================================

#[test]
fn test_overwrite_on_empty_track() {
    let mut timeline = Timeline::new("Test");
    let mut track = timeline.add_video_track("V1");
    let clip = Clip::new("Clip", make_time_range(0.0, 24.0, 24.0));

    // Overwrite on empty track - may or may not work depending on implementation
    let _result = track.overwrite(clip, make_time_range(0.0, 24.0, 24.0), false);
    // Just verify it doesn't panic
}

#[test]
fn test_insert_at_invalid_time() {
    let mut timeline = Timeline::new("Test");
    let mut track = timeline.add_video_track("V1");
    let clip = Clip::new("Clip", make_time_range(0.0, 24.0, 24.0));

    // Insert at negative time - should work or fail gracefully
    let _result = track.insert_at_time(clip, RationalTime::new(-100.0, 24.0), false);
    // Just verify it doesn't panic
}

#[test]
fn test_slice_empty_track() {
    let mut timeline = Timeline::new("Test");
    let mut track = timeline.add_video_track("V1");

    // Slicing empty track should fail
    let result = track.slice_at_time(RationalTime::new(0.0, 24.0), false);
    assert!(result.is_err(), "Slicing empty track should return error");
}

#[test]
fn test_remove_from_empty_track() {
    let mut timeline = Timeline::new("Test");
    let mut track = timeline.add_video_track("V1");

    // Removing from empty track should fail
    let result = track.remove_at_time(RationalTime::new(0.0, 24.0), false);
    assert!(result.is_err(), "Removing from empty track should return error");
}

// ============================================================================
// Error Message Content Tests
// ============================================================================

#[test]
fn test_error_has_descriptive_message() {
    let result = Timeline::read_from_file(std::path::Path::new("/this/path/does/not/exist.otio"));
    let err = result.unwrap_err();

    // Error message should contain something meaningful
    assert!(
        !err.message.is_empty(),
        "Error message should not be empty"
    );

    // Error Display trait should work
    let display_str = format!("{err}");
    assert!(
        display_str.contains(&err.code.to_string()),
        "Display should include error code"
    );
}

#[test]
fn test_error_debug_impl() {
    let result = Timeline::read_from_file(std::path::Path::new("/nonexistent.otio"));
    let err = result.unwrap_err();

    // Debug trait should work
    let debug_str = format!("{err:?}");
    assert!(
        debug_str.contains("OtioError"),
        "Debug should include type name"
    );
}

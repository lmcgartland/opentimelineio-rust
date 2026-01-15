//! Tests for Marker, Effect, and Transition types.

use otio_rs::{
    marker, transition, Clip, Composable, Effect, HasMetadata, Marker, RationalTime, TimeRange,
    Timeline, Transition,
};

fn make_time_range(start: f64, duration: f64, rate: f64) -> TimeRange {
    TimeRange::new(
        RationalTime::new(start, rate),
        RationalTime::new(duration, rate),
    )
}

// ============ Marker Tests ============

#[test]
fn test_marker_create() {
    let range = make_time_range(100.0, 24.0, 24.0);
    let marker = Marker::new("Test Marker", range, marker::colors::RED);

    assert_eq!(marker.name(), "Test Marker");
    assert_eq!(marker.color(), "RED");
}

#[test]
fn test_marker_default_color() {
    let range = make_time_range(0.0, 48.0, 24.0);
    let marker = Marker::with_default_color("Green Marker", range);

    assert_eq!(marker.name(), "Green Marker");
    assert_eq!(marker.color(), "GREEN");
}

#[test]
fn test_marker_set_color() {
    let range = make_time_range(0.0, 24.0, 24.0);
    let mut marker = Marker::new("Marker", range, marker::colors::GREEN);

    marker.set_color(marker::colors::BLUE);
    assert_eq!(marker.color(), "BLUE");

    marker.set_color(marker::colors::YELLOW);
    assert_eq!(marker.color(), "YELLOW");
}

#[test]
fn test_marker_marked_range() {
    let range = make_time_range(100.0, 50.0, 24.0);
    let marker = Marker::new("Test", range, marker::colors::RED);

    let retrieved = marker.marked_range();
    assert!((retrieved.start_time.value - 100.0).abs() < f64::EPSILON);
    assert!((retrieved.duration.value - 50.0).abs() < f64::EPSILON);
}

#[test]
fn test_marker_set_marked_range() {
    let range = make_time_range(0.0, 24.0, 24.0);
    let mut marker = Marker::new("Test", range, marker::colors::RED);

    let new_range = make_time_range(50.0, 100.0, 24.0);
    marker.set_marked_range(new_range).unwrap();

    let retrieved = marker.marked_range();
    assert!((retrieved.start_time.value - 50.0).abs() < f64::EPSILON);
    assert!((retrieved.duration.value - 100.0).abs() < f64::EPSILON);
}

#[test]
fn test_marker_comment() {
    let range = make_time_range(0.0, 24.0, 24.0);
    let mut marker = Marker::new("Test", range, marker::colors::RED);

    assert_eq!(marker.comment(), "");

    marker.set_comment("This is a test comment");
    assert_eq!(marker.comment(), "This is a test comment");
}

#[test]
fn test_marker_metadata() {
    let range = make_time_range(0.0, 24.0, 24.0);
    let mut marker = Marker::new("Test", range, marker::colors::RED);

    marker.set_metadata("key1", "value1");
    marker.set_metadata("key2", "value2");

    assert_eq!(marker.get_metadata("key1"), Some("value1".to_string()));
    assert_eq!(marker.get_metadata("key2"), Some("value2".to_string()));
    assert_eq!(marker.get_metadata("nonexistent"), None);
}

// ============ Effect Tests ============

#[test]
fn test_effect_create() {
    let effect = Effect::new("My Effect", "ColorCorrection");

    assert_eq!(effect.name(), "My Effect");
    assert_eq!(effect.effect_name(), "ColorCorrection");
}

#[test]
fn test_effect_set_effect_name() {
    let mut effect = Effect::new("Effect", "Blur");

    effect.set_effect_name("Sharpen");
    assert_eq!(effect.effect_name(), "Sharpen");
}

#[test]
fn test_effect_metadata() {
    let mut effect = Effect::new("Effect", "Test");

    effect.set_metadata("param1", "value1");
    effect.set_metadata("intensity", "0.5");

    assert_eq!(effect.get_metadata("param1"), Some("value1".to_string()));
    assert_eq!(effect.get_metadata("intensity"), Some("0.5".to_string()));
}

// ============ Transition Tests ============

#[test]
fn test_transition_create() {
    let in_offset = RationalTime::new(12.0, 24.0);
    let out_offset = RationalTime::new(12.0, 24.0);
    let transition = Transition::new("Dissolve", transition::types::SMPTE_DISSOLVE, in_offset, out_offset);

    assert_eq!(transition.name(), "Dissolve");
    assert_eq!(transition.transition_type(), "SMPTE_Dissolve");
}

#[test]
fn test_transition_dissolve_helper() {
    let in_offset = RationalTime::new(6.0, 24.0);
    let out_offset = RationalTime::new(6.0, 24.0);
    let transition = Transition::dissolve("Quick Dissolve", in_offset, out_offset);

    assert_eq!(transition.name(), "Quick Dissolve");
    assert_eq!(transition.transition_type(), "SMPTE_Dissolve");
}

#[test]
fn test_transition_offsets() {
    let in_offset = RationalTime::new(12.0, 24.0);
    let out_offset = RationalTime::new(18.0, 24.0);
    let transition = Transition::dissolve("Test", in_offset, out_offset);

    let retrieved_in = transition.in_offset();
    let retrieved_out = transition.out_offset();

    assert!((retrieved_in.value - 12.0).abs() < f64::EPSILON);
    assert!((retrieved_out.value - 18.0).abs() < f64::EPSILON);
}

#[test]
fn test_transition_set_offsets() {
    let in_offset = RationalTime::new(0.0, 24.0);
    let out_offset = RationalTime::new(0.0, 24.0);
    let mut transition = Transition::dissolve("Test", in_offset, out_offset);

    transition.set_in_offset(RationalTime::new(10.0, 24.0));
    transition.set_out_offset(RationalTime::new(15.0, 24.0));

    assert!((transition.in_offset().value - 10.0).abs() < f64::EPSILON);
    assert!((transition.out_offset().value - 15.0).abs() < f64::EPSILON);
}

#[test]
fn test_transition_set_type() {
    let in_offset = RationalTime::new(12.0, 24.0);
    let out_offset = RationalTime::new(12.0, 24.0);
    let mut transition = Transition::dissolve("Test", in_offset, out_offset);

    transition.set_transition_type(transition::types::CUSTOM);
    assert_eq!(transition.transition_type(), "Custom_Transition");
}

#[test]
fn test_transition_metadata() {
    let in_offset = RationalTime::new(12.0, 24.0);
    let out_offset = RationalTime::new(12.0, 24.0);
    let mut transition = Transition::dissolve("Test", in_offset, out_offset);

    transition.set_metadata("easing", "ease-in-out");
    assert_eq!(
        transition.get_metadata("easing"),
        Some("ease-in-out".to_string())
    );
}

// ============ Integration Tests ============

#[test]
fn test_track_with_transitions() {
    let mut timeline = Timeline::new("Test Timeline");
    let mut track = timeline.add_video_track("V1");

    // Add clip, transition, clip
    let clip1 = Clip::new("Clip 1", make_time_range(0.0, 48.0, 24.0));
    let transition = Transition::dissolve(
        "Dissolve",
        RationalTime::new(12.0, 24.0),
        RationalTime::new(12.0, 24.0),
    );
    let clip2 = Clip::new("Clip 2", make_time_range(0.0, 48.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_transition(transition).unwrap();
    track.append_clip(clip2).unwrap();

    assert_eq!(track.children_count(), 3);

    // Verify types
    let children: Vec<_> = track.children().collect();
    assert!(matches!(children[0], Composable::Clip(_)));
    assert!(matches!(children[1], Composable::Transition(_)));
    assert!(matches!(children[2], Composable::Clip(_)));

    // Verify transition properties
    if let Composable::Transition(t_ref) = &children[1] {
        assert_eq!(t_ref.name(), "Dissolve");
        assert_eq!(t_ref.transition_type(), "SMPTE_Dissolve");
    } else {
        panic!("Expected Transition");
    }
}

#[test]
fn test_insert_transition() {
    let mut timeline = Timeline::new("Test");
    let mut track = timeline.add_video_track("V1");

    let clip1 = Clip::new("A", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("B", make_time_range(0.0, 24.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();

    // Insert transition between clips
    let transition = Transition::dissolve(
        "Middle Dissolve",
        RationalTime::new(6.0, 24.0),
        RationalTime::new(6.0, 24.0),
    );
    track.insert_transition(1, transition).unwrap();

    assert_eq!(track.children_count(), 3);

    let children: Vec<_> = track.children().collect();
    assert!(matches!(children[0], Composable::Clip(_)));
    assert!(matches!(children[1], Composable::Transition(_)));
    assert!(matches!(children[2], Composable::Clip(_)));
}

#[test]
fn test_all_marker_colors() {
    let range = make_time_range(0.0, 24.0, 24.0);

    let colors = [
        (marker::colors::PINK, "PINK"),
        (marker::colors::RED, "RED"),
        (marker::colors::ORANGE, "ORANGE"),
        (marker::colors::YELLOW, "YELLOW"),
        (marker::colors::GREEN, "GREEN"),
        (marker::colors::CYAN, "CYAN"),
        (marker::colors::BLUE, "BLUE"),
        (marker::colors::PURPLE, "PURPLE"),
        (marker::colors::MAGENTA, "MAGENTA"),
        (marker::colors::BLACK, "BLACK"),
        (marker::colors::WHITE, "WHITE"),
    ];

    for (color, expected) in colors {
        let marker = Marker::new("Test", range, color);
        assert_eq!(marker.color(), expected);
    }
}

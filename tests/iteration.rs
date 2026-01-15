use otio_rs::{Clip, Composable, Gap, RationalTime, Stack, TimeRange, Timeline, Track};

fn make_time_range(start: f64, duration: f64, rate: f64) -> TimeRange {
    TimeRange::new(
        RationalTime::new(start, rate),
        RationalTime::new(duration, rate),
    )
}

#[test]
fn test_track_children_count_empty() {
    let mut tl = Timeline::new("test");
    let track = tl.add_video_track("V1");
    assert_eq!(track.children_count(), 0);
}

#[test]
fn test_track_children_count_with_clips() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip1 = Clip::new("clip1", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("clip2", make_time_range(0.0, 48.0, 24.0));
    let clip3 = Clip::new("clip3", make_time_range(0.0, 12.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();
    track.append_clip(clip3).unwrap();

    assert_eq!(track.children_count(), 3);
}

#[test]
fn test_track_children_count_mixed() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip = Clip::new("clip", make_time_range(0.0, 24.0, 24.0));
    let gap = Gap::new(RationalTime::new(12.0, 24.0));

    track.append_clip(clip).unwrap();
    track.append_gap(gap).unwrap();

    assert_eq!(track.children_count(), 2);
}

#[test]
fn test_track_iterate_clips() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip1 = Clip::new("clip1", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("clip2", make_time_range(0.0, 48.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();

    let children: Vec<_> = track.children().collect();
    assert_eq!(children.len(), 2);

    // Verify they are clips
    assert!(matches!(children[0], Composable::Clip(_)));
    assert!(matches!(children[1], Composable::Clip(_)));

    // Verify names
    if let Composable::Clip(clip_ref) = &children[0] {
        assert_eq!(clip_ref.name(), "clip1");
    }
    if let Composable::Clip(clip_ref) = &children[1] {
        assert_eq!(clip_ref.name(), "clip2");
    }
}

#[test]
fn test_track_iterate_gaps() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let gap1 = Gap::new(RationalTime::new(12.0, 24.0));
    let gap2 = Gap::new(RationalTime::new(24.0, 24.0));

    track.append_gap(gap1).unwrap();
    track.append_gap(gap2).unwrap();

    let children: Vec<_> = track.children().collect();
    assert_eq!(children.len(), 2);

    assert!(matches!(children[0], Composable::Gap(_)));
    assert!(matches!(children[1], Composable::Gap(_)));
}

#[test]
fn test_track_iterate_mixed() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip = Clip::new("clip", make_time_range(0.0, 24.0, 24.0));
    let gap = Gap::new(RationalTime::new(12.0, 24.0));
    let clip2 = Clip::new("clip2", make_time_range(0.0, 48.0, 24.0));

    track.append_clip(clip).unwrap();
    track.append_gap(gap).unwrap();
    track.append_clip(clip2).unwrap();

    let children: Vec<_> = track.children().collect();
    assert_eq!(children.len(), 3);

    assert!(matches!(children[0], Composable::Clip(_)));
    assert!(matches!(children[1], Composable::Gap(_)));
    assert!(matches!(children[2], Composable::Clip(_)));
}

#[test]
fn test_track_iterate_with_stack() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip = Clip::new("clip", make_time_range(0.0, 24.0, 24.0));
    let stack = Stack::new("nested_stack");

    track.append_clip(clip).unwrap();
    track.append_stack(stack).unwrap();

    let children: Vec<_> = track.children().collect();
    assert_eq!(children.len(), 2);

    assert!(matches!(children[0], Composable::Clip(_)));
    assert!(matches!(children[1], Composable::Stack(_)));

    if let Composable::Stack(stack_ref) = &children[1] {
        assert_eq!(stack_ref.name(), "nested_stack");
    }
}

#[test]
fn test_stack_children_count_empty() {
    let stack = Stack::new("test_stack");
    assert_eq!(stack.children_count(), 0);
}

#[test]
fn test_stack_children_count_with_tracks() {
    let mut stack = Stack::new("test_stack");

    let track1 = Track::new_video("V1");
    let track2 = Track::new_audio("A1");

    stack.append_track(track1).unwrap();
    stack.append_track(track2).unwrap();

    assert_eq!(stack.children_count(), 2);
}

#[test]
fn test_stack_iterate_tracks() {
    let mut stack = Stack::new("test_stack");

    let track1 = Track::new_video("V1");
    let track2 = Track::new_video("V2");

    stack.append_track(track1).unwrap();
    stack.append_track(track2).unwrap();

    let children: Vec<_> = stack.children().collect();
    assert_eq!(children.len(), 2);

    assert!(matches!(children[0], Composable::Track(_)));
    assert!(matches!(children[1], Composable::Track(_)));

    if let Composable::Track(track_ref) = &children[0] {
        assert_eq!(track_ref.name(), "V1");
    }
    if let Composable::Track(track_ref) = &children[1] {
        assert_eq!(track_ref.name(), "V2");
    }
}

#[test]
fn test_stack_iterate_mixed() {
    let mut stack = Stack::new("test_stack");

    let track = Track::new_video("V1");
    let clip = Clip::new("clip", make_time_range(0.0, 24.0, 24.0));
    let gap = Gap::new(RationalTime::new(12.0, 24.0));
    let nested_stack = Stack::new("nested");

    stack.append_track(track).unwrap();
    stack.append_clip(clip).unwrap();
    stack.append_gap(gap).unwrap();
    stack.append_stack(nested_stack).unwrap();

    let children: Vec<_> = stack.children().collect();
    assert_eq!(children.len(), 4);

    assert!(matches!(children[0], Composable::Track(_)));
    assert!(matches!(children[1], Composable::Clip(_)));
    assert!(matches!(children[2], Composable::Gap(_)));
    assert!(matches!(children[3], Composable::Stack(_)));
}

#[test]
fn test_timeline_tracks_iteration() {
    let mut tl = Timeline::new("test");
    let _ = tl.add_video_track("V1");
    let _ = tl.add_video_track("V2");
    let _ = tl.add_audio_track("A1");

    let tracks_stack = tl.tracks();
    assert_eq!(tracks_stack.children_count(), 3);

    let children: Vec<_> = tracks_stack.children().collect();
    assert_eq!(children.len(), 3);

    // All should be tracks
    for child in &children {
        assert!(matches!(child, Composable::Track(_)));
    }
}

#[test]
#[allow(clippy::float_cmp)]
fn test_clip_ref_source_range() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let source_range = make_time_range(10.0, 50.0, 24.0);
    let clip = Clip::new("clip", source_range);
    track.append_clip(clip).unwrap();

    let children: Vec<_> = track.children().collect();
    if let Composable::Clip(clip_ref) = &children[0] {
        let range = clip_ref.source_range();
        assert_eq!(range.start_time.value, 10.0);
        assert_eq!(range.duration.value, 50.0);
        assert_eq!(range.start_time.rate, 24.0);
    } else {
        panic!("Expected Clip");
    }
}

#[test]
fn test_iterate_empty_track() {
    let mut tl = Timeline::new("test");
    let track = tl.add_video_track("V1");

    let children: Vec<_> = track.children().collect();
    assert!(children.is_empty());
}

#[test]
fn test_iterate_empty_stack() {
    let stack = Stack::new("empty");

    let children: Vec<_> = stack.children().collect();
    assert!(children.is_empty());
}

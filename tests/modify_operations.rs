use otio_rs::{Clip, Composable, Gap, RationalTime, Stack, TimeRange, Timeline, Track};

fn make_time_range(start: f64, duration: f64, rate: f64) -> TimeRange {
    TimeRange::new(
        RationalTime::new(start, rate),
        RationalTime::new(duration, rate),
    )
}

// ============ Track Insert Operations ============

#[test]
fn test_track_insert_clip_at_beginning() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip1 = Clip::new("clip1", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("clip2", make_time_range(0.0, 24.0, 24.0));
    let clip_new = Clip::new("new_clip", make_time_range(0.0, 24.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();
    track.insert_clip(0, clip_new).unwrap();

    assert_eq!(track.children_count(), 3);

    let children: Vec<_> = track.children().collect();
    if let Composable::Clip(clip_ref) = &children[0] {
        assert_eq!(clip_ref.name(), "new_clip");
    } else {
        panic!("Expected Clip");
    }
}

#[test]
fn test_track_insert_clip_in_middle() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip1 = Clip::new("clip1", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("clip2", make_time_range(0.0, 24.0, 24.0));
    let clip_middle = Clip::new("middle", make_time_range(0.0, 24.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();
    track.insert_clip(1, clip_middle).unwrap();

    assert_eq!(track.children_count(), 3);

    let children: Vec<_> = track.children().collect();
    if let Composable::Clip(clip_ref) = &children[1] {
        assert_eq!(clip_ref.name(), "middle");
    } else {
        panic!("Expected Clip");
    }
}

#[test]
fn test_track_insert_gap() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip1 = Clip::new("clip1", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("clip2", make_time_range(0.0, 24.0, 24.0));
    let gap = Gap::new(RationalTime::new(12.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();
    track.insert_gap(1, gap).unwrap();

    assert_eq!(track.children_count(), 3);

    let children: Vec<_> = track.children().collect();
    assert!(matches!(children[0], Composable::Clip(_)));
    assert!(matches!(children[1], Composable::Gap(_)));
    assert!(matches!(children[2], Composable::Clip(_)));
}

#[test]
fn test_track_insert_stack() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip = Clip::new("clip", make_time_range(0.0, 24.0, 24.0));
    let stack = Stack::new("inserted_stack");

    track.append_clip(clip).unwrap();
    track.insert_stack(0, stack).unwrap();

    assert_eq!(track.children_count(), 2);

    let children: Vec<_> = track.children().collect();
    assert!(matches!(children[0], Composable::Stack(_)));
    assert!(matches!(children[1], Composable::Clip(_)));
}

// ============ Track Remove Operations ============

#[test]
fn test_track_remove_child_first() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip1 = Clip::new("clip1", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("clip2", make_time_range(0.0, 24.0, 24.0));
    let clip3 = Clip::new("clip3", make_time_range(0.0, 24.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();
    track.append_clip(clip3).unwrap();

    track.remove_child(0).unwrap();

    assert_eq!(track.children_count(), 2);

    let children: Vec<_> = track.children().collect();
    if let Composable::Clip(clip_ref) = &children[0] {
        assert_eq!(clip_ref.name(), "clip2");
    }
}

#[test]
fn test_track_remove_child_middle() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip1 = Clip::new("clip1", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("clip2", make_time_range(0.0, 24.0, 24.0));
    let clip3 = Clip::new("clip3", make_time_range(0.0, 24.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();
    track.append_clip(clip3).unwrap();

    track.remove_child(1).unwrap();

    assert_eq!(track.children_count(), 2);

    let children: Vec<_> = track.children().collect();
    if let Composable::Clip(clip_ref) = &children[0] {
        assert_eq!(clip_ref.name(), "clip1");
    }
    if let Composable::Clip(clip_ref) = &children[1] {
        assert_eq!(clip_ref.name(), "clip3");
    }
}

#[test]
fn test_track_remove_child_last() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip1 = Clip::new("clip1", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("clip2", make_time_range(0.0, 24.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();

    track.remove_child(1).unwrap();

    assert_eq!(track.children_count(), 1);

    let children: Vec<_> = track.children().collect();
    if let Composable::Clip(clip_ref) = &children[0] {
        assert_eq!(clip_ref.name(), "clip1");
    }
}

// ============ Track Clear Operations ============

#[test]
fn test_track_clear_children() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    let clip1 = Clip::new("clip1", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("clip2", make_time_range(0.0, 24.0, 24.0));
    let gap = Gap::new(RationalTime::new(12.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_gap(gap).unwrap();
    track.append_clip(clip2).unwrap();

    assert_eq!(track.children_count(), 3);

    track.clear_children().unwrap();

    assert_eq!(track.children_count(), 0);
}

#[test]
fn test_track_clear_empty() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    // Should not error on empty track
    track.clear_children().unwrap();
    assert_eq!(track.children_count(), 0);
}

// ============ Stack Insert Operations ============

#[test]
fn test_stack_insert_track_at_beginning() {
    let mut stack = Stack::new("test_stack");

    let track1 = Track::new_video("V1");
    let track2 = Track::new_video("V2");
    let track_new = Track::new_video("new_track");

    stack.append_track(track1).unwrap();
    stack.append_track(track2).unwrap();
    stack.insert_track(0, track_new).unwrap();

    assert_eq!(stack.children_count(), 3);

    let children: Vec<_> = stack.children().collect();
    if let Composable::Track(track_ref) = &children[0] {
        assert_eq!(track_ref.name(), "new_track");
    } else {
        panic!("Expected Track");
    }
}

#[test]
fn test_stack_insert_clip() {
    let mut stack = Stack::new("test_stack");

    let track = Track::new_video("V1");
    let clip = Clip::new("inserted_clip", make_time_range(0.0, 24.0, 24.0));

    stack.append_track(track).unwrap();
    stack.insert_clip(0, clip).unwrap();

    assert_eq!(stack.children_count(), 2);

    let children: Vec<_> = stack.children().collect();
    assert!(matches!(children[0], Composable::Clip(_)));
    assert!(matches!(children[1], Composable::Track(_)));
}

#[test]
fn test_stack_insert_gap() {
    let mut stack = Stack::new("test_stack");

    let track = Track::new_video("V1");
    let gap = Gap::new(RationalTime::new(12.0, 24.0));

    stack.append_track(track).unwrap();
    stack.insert_gap(1, gap).unwrap();

    assert_eq!(stack.children_count(), 2);

    let children: Vec<_> = stack.children().collect();
    assert!(matches!(children[0], Composable::Track(_)));
    assert!(matches!(children[1], Composable::Gap(_)));
}

#[test]
fn test_stack_insert_nested_stack() {
    let mut stack = Stack::new("outer");

    let track = Track::new_video("V1");
    let nested = Stack::new("nested");

    stack.append_track(track).unwrap();
    stack.insert_stack(0, nested).unwrap();

    assert_eq!(stack.children_count(), 2);

    let children: Vec<_> = stack.children().collect();
    assert!(matches!(children[0], Composable::Stack(_)));
    if let Composable::Stack(stack_ref) = &children[0] {
        assert_eq!(stack_ref.name(), "nested");
    }
}

// ============ Stack Remove Operations ============

#[test]
fn test_stack_remove_child() {
    let mut stack = Stack::new("test_stack");

    let track1 = Track::new_video("V1");
    let track2 = Track::new_video("V2");
    let track3 = Track::new_video("V3");

    stack.append_track(track1).unwrap();
    stack.append_track(track2).unwrap();
    stack.append_track(track3).unwrap();

    stack.remove_child(1).unwrap();

    assert_eq!(stack.children_count(), 2);

    let children: Vec<_> = stack.children().collect();
    if let Composable::Track(track_ref) = &children[0] {
        assert_eq!(track_ref.name(), "V1");
    }
    if let Composable::Track(track_ref) = &children[1] {
        assert_eq!(track_ref.name(), "V3");
    }
}

// ============ Stack Clear Operations ============

#[test]
fn test_stack_clear_children() {
    let mut stack = Stack::new("test_stack");

    let track = Track::new_video("V1");
    let clip = Clip::new("clip", make_time_range(0.0, 24.0, 24.0));

    stack.append_track(track).unwrap();
    stack.append_clip(clip).unwrap();

    assert_eq!(stack.children_count(), 2);

    stack.clear_children().unwrap();

    assert_eq!(stack.children_count(), 0);
}

#[test]
fn test_stack_clear_empty() {
    let mut stack = Stack::new("empty");

    stack.clear_children().unwrap();
    assert_eq!(stack.children_count(), 0);
}

// ============ Complex Scenarios ============

#[test]
fn test_insert_and_remove_sequence() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    // Build up
    let clip1 = Clip::new("A", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("B", make_time_range(0.0, 24.0, 24.0));
    let clip3 = Clip::new("C", make_time_range(0.0, 24.0, 24.0));

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();
    track.append_clip(clip3).unwrap();
    // Order: A, B, C

    // Insert D at position 1
    let clip4 = Clip::new("D", make_time_range(0.0, 24.0, 24.0));
    track.insert_clip(1, clip4).unwrap();
    // Order: A, D, B, C

    // Remove position 2 (B)
    track.remove_child(2).unwrap();
    // Order: A, D, C

    assert_eq!(track.children_count(), 3);

    let names: Vec<_> = track
        .children()
        .map(|c| {
            if let Composable::Clip(clip_ref) = c {
                clip_ref.name()
            } else {
                String::new()
            }
        })
        .collect();

    assert_eq!(names, vec!["A", "D", "C"]);
}

#[test]
fn test_rebuild_track_after_clear() {
    let mut tl = Timeline::new("test");
    let mut track = tl.add_video_track("V1");

    // Initial content
    let clip1 = Clip::new("old1", make_time_range(0.0, 24.0, 24.0));
    let clip2 = Clip::new("old2", make_time_range(0.0, 24.0, 24.0));
    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();

    // Clear and rebuild
    track.clear_children().unwrap();

    let new_clip1 = Clip::new("new1", make_time_range(0.0, 48.0, 24.0));
    let new_clip2 = Clip::new("new2", make_time_range(0.0, 48.0, 24.0));
    track.append_clip(new_clip1).unwrap();
    track.append_clip(new_clip2).unwrap();

    assert_eq!(track.children_count(), 2);

    let names: Vec<_> = track
        .children()
        .map(|c| {
            if let Composable::Clip(clip_ref) = c {
                clip_ref.name()
            } else {
                String::new()
            }
        })
        .collect();

    assert_eq!(names, vec!["new1", "new2"]);
}

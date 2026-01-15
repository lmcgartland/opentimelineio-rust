//! Memory leak stress tests.
//!
//! These tests are designed to be run with memory analysis tools like Valgrind
//! or `AddressSanitizer` to detect memory leaks in the FFI bindings.
//!
//! Run with: `cargo test --test memory -- --ignored --test-threads=1`
//! Run with Valgrind: `./scripts/check_memory.sh`

// Allow exact float comparisons in tests - values are known exactly
#![allow(clippy::float_cmp)]
// Intentional drops to test memory cleanup
#![allow(clippy::drop_non_drop)]

use otio_rs::{
    marker, Clip, Gap, HasMetadata, ImageSequenceReference, Marker, RationalTime, Stack,
    Timeline, TimeRange, Track,
};

/// Stress test: Create and drop many timelines.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_timeline_creation() {
    for i in 0..10_000 {
        let timeline = Timeline::new(&format!("Timeline {i}"));
        assert!(!timeline.name().is_empty());
        drop(timeline);
    }
}

/// Stress test: Create complex timelines with many tracks and clips.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_complex_timeline() {
    for iteration in 0..100 {
        let mut timeline = Timeline::new(&format!("Complex {iteration}"));

        // Add multiple video tracks
        for t in 0..5 {
            let mut track = timeline.add_video_track(&format!("V{}", t + 1));

            // Add clips to each track
            for c in 0..20 {
                let clip = Clip::new(
                    &format!("Clip_{t}_{c}"),
                    TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0)),
                );
                track.append_clip(clip).unwrap();
            }
        }

        // Verify structure
        assert_eq!(timeline.tracks().children_count(), 5);
        assert_eq!(timeline.find_clips().count(), 100);

        // All dropped here
    }
}

/// Stress test: Create and drop clips with markers.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_clips_with_markers() {
    for iteration in 0..1_000 {
        let mut clip = Clip::new(
            &format!("Clip {iteration}"),
            TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
        );

        // Add multiple markers
        for m in 0..10 {
            let marker = Marker::new(
                &format!("Marker {m}"),
                TimeRange::new(
                    RationalTime::new(f64::from(m), 24.0),
                    RationalTime::new(1.0, 24.0),
                ),
                marker::colors::RED,
            );
            clip.add_marker(marker).unwrap();
        }

        assert_eq!(clip.markers_count(), 10);
    }
}

/// Stress test: Create and drop gaps.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_gaps() {
    for _ in 0..10_000 {
        let gap = Gap::new(RationalTime::new(24.0, 24.0));
        // Gap is created and dropped
        drop(gap);
    }
}

/// Stress test: Create and drop tracks with mixed content.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_tracks_with_gaps() {
    for iteration in 0..500 {
        let mut track = Track::new_video(&format!("Track {iteration}"));

        for i in 0..50 {
            if i % 2 == 0 {
                let clip = Clip::new(
                    &format!("Clip {i}"),
                    TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0)),
                );
                track.append_clip(clip).unwrap();
            } else {
                let gap = Gap::new(RationalTime::new(12.0, 24.0));
                track.append_gap(gap).unwrap();
            }
        }

        assert_eq!(track.children_count(), 50);
    }
}

/// Stress test: Serialization roundtrip.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_serialization_roundtrip() {
    for iteration in 0..500 {
        // Create timeline
        let mut timeline = Timeline::new(&format!("Roundtrip {iteration}"));
        let mut track = timeline.add_video_track("V1");

        for i in 0..10 {
            let clip = Clip::new(
                &format!("Clip {i}"),
                TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
            );
            track.append_clip(clip).unwrap();
        }
        drop(track); // Explicitly drop to release borrow

        // Serialize
        let json = timeline.to_json_string().unwrap();
        assert!(!json.is_empty());

        // Deserialize
        let restored = Timeline::from_json_string(&json).unwrap();
        // Verify at least some clips were restored
        assert!(restored.find_clips().count() > 0);
    }
}

/// Stress test: `find_clips` iterator.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_find_clips_iterator() {
    // Create a large timeline
    let mut timeline = Timeline::new("Large");
    let mut track = timeline.add_video_track("V1");

    for i in 0..1000 {
        let clip = Clip::new(
            &format!("Clip {i}"),
            TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0)),
        );
        track.append_clip(clip).unwrap();
    }
    drop(track);

    // Iterate many times
    for _ in 0..1000 {
        let clips: Vec<_> = timeline.find_clips().collect();
        assert_eq!(clips.len(), 1000);

        // Access each clip
        for clip in &clips {
            let _ = clip.name();
            let _ = clip.source_range();
        }
    }
}

/// Stress test: Stack operations.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_stacks() {
    for iteration in 0..500 {
        let mut stack = Stack::new(&format!("Stack {iteration}"));

        for t in 0..5 {
            let mut track = Track::new_video(&format!("Track {t}"));

            for c in 0..10 {
                let clip = Clip::new(
                    &format!("Clip_{t}_{c}"),
                    TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0)),
                );
                track.append_clip(clip).unwrap();
            }

            stack.append_track(track).unwrap();
        }

        assert_eq!(stack.children_count(), 5);
    }
}

/// Stress test: `ImageSequenceReference` creation.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_image_sequence_reference() {
    for iteration in 0..5_000 {
        let mut seq = ImageSequenceReference::new(
            &format!("/path/to/sequence_{iteration}/"),
            "frame_",
            ".exr",
            1001,
            1,
            24.0,
            4,
        );

        seq.set_available_range(TimeRange::new(
            RationalTime::new(0.0, 24.0),
            RationalTime::new(100.0, 24.0),
        ))
        .unwrap();

        assert_eq!(seq.start_frame(), 1001);
        assert_eq!(seq.frame_step(), 1);
        assert_eq!(seq.rate(), 24.0);
        assert_eq!(seq.frame_zero_padding(), 4);
        assert_eq!(seq.number_of_images(), 100);

        // Test URL generation
        let url = seq.target_url_for_image_number(0).unwrap();
        assert!(url.contains("1001"));
    }
}

/// Stress test: Markers on tracks.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_track_markers() {
    for iteration in 0..500 {
        let mut track = Track::new_video(&format!("Track {iteration}"));

        for m in 0..50 {
            let marker = Marker::new(
                &format!("Marker {m}"),
                TimeRange::new(
                    RationalTime::new(f64::from(m) * 24.0, 24.0),
                    RationalTime::new(1.0, 24.0),
                ),
                marker::colors::GREEN,
            );
            track.add_marker(marker).unwrap();
        }

        assert_eq!(track.markers_count(), 50);
    }
}

/// Stress test: String operations (name getting).
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_string_operations() {
    for i in 0..10_000 {
        let name = format!("Timeline with a longer name {i}");
        let timeline = Timeline::new(&name);
        let retrieved = timeline.name();
        assert_eq!(retrieved, name);
    }
}

/// Stress test: Metadata operations.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_metadata() {
    for iteration in 0..1_000 {
        let mut clip = Clip::new(
            &format!("Clip {iteration}"),
            TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
        );

        // Set many metadata keys
        for k in 0..20 {
            clip.set_metadata(&format!("key_{k}"), &format!("value_{k}"));
        }

        // Read them back
        for k in 0..20 {
            let value = clip.get_metadata(&format!("key_{k}"));
            assert!(value.is_some());
            assert_eq!(value.unwrap(), format!("value_{k}"));
        }
    }
}

/// Stress test: `RationalTime` creation.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_rational_time() {
    for _ in 0..100_000 {
        let t1 = RationalTime::new(1.0, 24.0);
        let t2 = RationalTime::new(2.0, 24.0);

        // Basic operations
        let _ = t1.to_seconds();
        let _ = t2.to_seconds();
        let _ = RationalTime::from_seconds(1.0, 24.0);

        // Create time ranges
        let _ = TimeRange::new(t1, t2);
    }
}

/// Stress test: `TimeRange` operations.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_time_range() {
    for _ in 0..100_000 {
        let r1 = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0));

        let r2 = TimeRange::new(RationalTime::new(12.0, 24.0), RationalTime::new(24.0, 24.0));

        let _ = r1.start_time;
        let _ = r1.duration;
        let _ = r1.end_time();
        let _ = r2.start_time;
        let _ = r2.duration;
    }
}

/// Stress test: Rapid timeline destruction during iteration.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_early_drop() {
    for _ in 0..1_000 {
        let mut timeline = Timeline::new("DropTest");
        let mut track = timeline.add_video_track("V1");

        for i in 0..100 {
            let clip = Clip::new(
                &format!("Clip {i}"),
                TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0)),
            );
            track.append_clip(clip).unwrap();
        }
        drop(track);

        // Start iterating but drop early
        let mut clips = timeline.find_clips();
        let _ = clips.next();
        let _ = clips.next();
        // Drop iterator and timeline together
    }
}

/// Stress test: Audio tracks.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_audio_tracks() {
    for iteration in 0..500 {
        let mut timeline = Timeline::new(&format!("Audio Timeline {iteration}"));

        for t in 0..3 {
            let mut track = timeline.add_audio_track(&format!("A{}", t + 1));

            for c in 0..30 {
                let clip = Clip::new(
                    &format!("Audio_{t}_{c}"),
                    TimeRange::new(RationalTime::new(0.0, 48000.0), RationalTime::new(48000.0, 48000.0)),
                );
                track.append_clip(clip).unwrap();
            }
        }

        assert_eq!(timeline.tracks().children_count(), 3);
        assert_eq!(timeline.find_clips().count(), 90);
    }
}

/// Stress test: Mixed video and audio tracks.
#[test]
#[ignore = "Run with memory tools: cargo test --test memory -- --ignored"]
fn stress_test_mixed_tracks() {
    for iteration in 0..200 {
        let mut timeline = Timeline::new(&format!("Mixed {iteration}"));

        // Add video tracks
        for v in 0..2 {
            let mut track = timeline.add_video_track(&format!("V{}", v + 1));
            for c in 0..25 {
                let clip = Clip::new(
                    &format!("Video_{v}_{c}"),
                    TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0)),
                );
                track.append_clip(clip).unwrap();
            }
        }

        // Add audio tracks
        for a in 0..4 {
            let mut track = timeline.add_audio_track(&format!("A{}", a + 1));
            for c in 0..25 {
                let clip = Clip::new(
                    &format!("Audio_{a}_{c}"),
                    TimeRange::new(RationalTime::new(0.0, 48000.0), RationalTime::new(48000.0, 48000.0)),
                );
                track.append_clip(clip).unwrap();
            }
        }

        assert_eq!(timeline.tracks().children_count(), 6);
        assert_eq!(timeline.find_clips().count(), 150);
    }
}

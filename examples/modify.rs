//! Example demonstrating remove and modify operations on tracks.

use otio_rs::{Clip, Composable, Gap, RationalTime, TimeRange, Timeline};

fn main() -> otio_rs::Result<()> {
    // Create a timeline with a video track
    let mut timeline = Timeline::new("Modify Demo");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0))?;

    let mut v1 = timeline.add_video_track("V1");

    // Add some clips
    let clip1 = Clip::new("Clip_A", TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(24.0, 24.0),
    ));
    v1.append_clip(clip1)?;

    let clip2 = Clip::new("Clip_B", TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    ));
    v1.append_clip(clip2)?;

    let clip3 = Clip::new("Clip_C", TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(72.0, 24.0),
    ));
    v1.append_clip(clip3)?;

    println!("Initial track contents ({} children):", v1.children_count());
    for (i, child) in v1.children().enumerate() {
        if let Composable::Clip(clip) = child {
            println!("  [{}] {}", i, clip.name());
        }
    }

    // Insert a gap at position 1
    let gap = Gap::new(RationalTime::new(12.0, 24.0));
    v1.insert_gap(1, gap)?;

    println!("\nAfter inserting gap at position 1 ({} children):", v1.children_count());
    for (i, child) in v1.children().enumerate() {
        match child {
            Composable::Clip(clip) => println!("  [{}] Clip: {}", i, clip.name()),
            Composable::Gap(_) => println!("  [{}] Gap", i),
            _ => {}
        }
    }

    // Remove the item at position 2 (should be Clip_B)
    v1.remove_child(2)?;

    println!("\nAfter removing child at position 2 ({} children):", v1.children_count());
    for (i, child) in v1.children().enumerate() {
        match child {
            Composable::Clip(clip) => println!("  [{}] Clip: {}", i, clip.name()),
            Composable::Gap(_) => println!("  [{}] Gap", i),
            _ => {}
        }
    }

    // Insert a new clip at the beginning
    let new_clip = Clip::new("Intro", TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(36.0, 24.0),
    ));
    v1.insert_clip(0, new_clip)?;

    println!("\nAfter inserting Intro at position 0 ({} children):", v1.children_count());
    for (i, child) in v1.children().enumerate() {
        match child {
            Composable::Clip(clip) => println!("  [{}] Clip: {}", i, clip.name()),
            Composable::Gap(_) => println!("  [{}] Gap", i),
            _ => {}
        }
    }

    // Clear all children
    v1.clear_children()?;
    println!("\nAfter clearing all children ({} children):", v1.children_count());

    println!("\nModification operations complete!");
    Ok(())
}

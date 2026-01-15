//! Example demonstrating iteration over timeline tracks and children.

use otio_rs::{
    Clip, Composable, ExternalReference, Gap, RationalTime, TimeRange, Timeline,
};
use std::path::Path;

fn main() -> otio_rs::Result<()> {
    // Create a timeline with tracks and clips
    let mut timeline = Timeline::new("Iteration Demo");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0))?;

    // Add video track with clips and gaps
    let mut v1 = timeline.add_video_track("V1");

    let mut clip1 = Clip::new("Intro", TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    ));
    let mut ref1 = ExternalReference::new("/media/intro.mov");
    ref1.set_available_range(TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(720.0, 24.0),
    ))?;
    clip1.set_media_reference(ref1)?;
    v1.append_clip(clip1)?;

    let gap = Gap::new(RationalTime::new(24.0, 24.0));
    v1.append_gap(gap)?;

    let clip2 = Clip::new("Main", TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(72.0, 24.0),
    ));
    v1.append_clip(clip2)?;

    // Add audio track
    let mut a1 = timeline.add_audio_track("A1");
    let audio_clip = Clip::new("Music", TimeRange::new(
        RationalTime::new(0.0, 48_000.0),
        RationalTime::new(288_000.0, 48_000.0),
    ));
    a1.append_clip(audio_clip)?;

    // Write to file
    let output_path = Path::new("/tmp/iterate_demo.otio");
    timeline.write_to_file(output_path)?;
    println!("Created timeline at: {}", output_path.display());

    // Read back and iterate
    let timeline = Timeline::read_from_file(output_path)?;

    println!("\nTimeline tracks:");
    let tracks = timeline.tracks();
    println!("  Root stack has {} children", tracks.children_count());

    for (i, child) in tracks.children().enumerate() {
        match child {
            Composable::Track(track_ref) => {
                println!("\n  Track {}: {}", i, track_ref.name());
                println!("    Children count: {}", track_ref.children_count());

                for (j, item) in track_ref.children().enumerate() {
                    match item {
                        Composable::Clip(clip_ref) => {
                            let range = clip_ref.source_range();
                            println!("      [{}] Clip: {} (start: {}, duration: {})",
                                j,
                                clip_ref.name(),
                                range.start_time.value,
                                range.duration.value);
                        }
                        Composable::Gap(gap_ref) => {
                            println!("      [{}] Gap: {}", j, gap_ref.name());
                        }
                        Composable::Stack(stack_ref) => {
                            println!("      [{}] Nested Stack: {}", j, stack_ref.name());
                        }
                        Composable::Track(nested_track) => {
                            println!("      [{}] Nested Track: {}", j, nested_track.name());
                        }
                        Composable::Transition(transition_ref) => {
                            println!("      [{}] Transition: {} (type: {})",
                                j,
                                transition_ref.name(),
                                transition_ref.transition_type());
                        }
                    }
                }
            }
            Composable::Clip(clip) => {
                println!("  [{}] Clip in stack: {}", i, clip.name());
            }
            Composable::Gap(gap) => {
                println!("  [{}] Gap in stack: {}", i, gap.name());
            }
            Composable::Stack(stack) => {
                println!("  [{}] Nested Stack: {}", i, stack.name());
            }
            Composable::Transition(transition) => {
                println!("  [{}] Transition in stack: {}", i, transition.name());
            }
        }
    }

    println!("\nIteration complete!");
    Ok(())
}

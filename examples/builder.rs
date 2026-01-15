//! Example demonstrating the builder pattern for OTIO types.

use otio_rs::{
    ClipBuilder, ExternalReferenceBuilder, HasMetadata, RationalTime, TimeRange,
    TimelineBuilder,
};
use std::path::Path;

fn main() -> otio_rs::Result<()> {
    // Create a timeline using the builder pattern
    let mut timeline = TimelineBuilder::new("Builder Demo")
        .global_start_time(RationalTime::new(0.0, 24.0))
        .metadata("author", "Jane Doe")
        .metadata("project", "Demo Project")
        .build();

    // Add a video track
    let mut v1 = timeline.add_video_track("V1");

    // Create clips using the builder pattern
    let clip1 = ClipBuilder::new(
        "Interview_A",
        TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
    )
    .media_reference(
        ExternalReferenceBuilder::new("/media/interview_a.mov")
            .available_range(TimeRange::new(
                RationalTime::new(0.0, 24.0),
                RationalTime::new(720.0, 24.0),
            ))
            .metadata("codec", "ProRes")
            .metadata("resolution", "1920x1080")
            .build(),
    )
    .metadata("speaker", "John Smith")
    .metadata("take", "3")
    .build();

    v1.append_clip(clip1)?;

    // Create another clip using the convenience method on Clip
    let clip2 = otio_rs::Clip::builder(
        "Broll_001",
        TimeRange::new(RationalTime::new(120.0, 24.0), RationalTime::new(72.0, 24.0)),
    )
    .media_reference(
        otio_rs::ExternalReference::builder("/media/broll_001.mp4")
            .available_range(TimeRange::new(
                RationalTime::new(0.0, 24.0),
                RationalTime::new(480.0, 24.0),
            ))
            .build(),
    )
    .metadata("camera", "GoPro Hero 12")
    .build();

    v1.append_clip(clip2)?;

    // Write to file
    let output_path = Path::new("/tmp/builder_demo.otio");
    timeline.write_to_file(output_path)?;

    // Read back and display metadata
    let timeline = otio_rs::Timeline::read_from_file(output_path)?;

    println!("Timeline: {}", timeline.get_metadata("author").unwrap_or_default());
    println!("Project: {}", timeline.get_metadata("project").unwrap_or_default());

    for child in timeline.tracks().children() {
        if let otio_rs::Composable::Track(track) = child {
            println!("\nTrack: {}", track.name());
            for item in track.children() {
                if let otio_rs::Composable::Clip(clip) = item {
                    println!("  Clip: {}", clip.name());
                    if let Some(speaker) = clip.get_metadata("speaker") {
                        println!("    Speaker: {speaker}");
                    }
                    if let Some(take) = clip.get_metadata("take") {
                        println!("    Take: {take}");
                    }
                    if let Some(camera) = clip.get_metadata("camera") {
                        println!("    Camera: {camera}");
                    }
                }
            }
        }
    }

    println!("\nBuilder pattern demo complete!");
    Ok(())
}

use otio_rs::{Clip, ExternalReference, Gap, HasMetadata, RationalTime, TimeRange, Timeline};
use std::path::Path;

fn main() -> otio_rs::Result<()> {
    // Create a timeline
    let mut timeline = Timeline::new("My Project");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0))?;

    // Add video track with clips
    let mut v1 = timeline.add_video_track("V1");

    // First clip: 2 seconds
    let mut clip1 = Clip::new("Interview_A", TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    ));
    let mut ref1 = ExternalReference::new("/media/interview_a.mov");
    ref1.set_available_range(TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(720.0, 24.0), // 30 seconds
    ))?;
    clip1.set_media_reference(ref1)?;
    v1.append_clip(clip1)?;

    // Gap: 1 second
    let gap = Gap::new(RationalTime::new(24.0, 24.0));
    v1.append_gap(gap)?;

    // Second clip: 3 seconds
    let mut clip2 = Clip::new("Broll_001", TimeRange::new(
        RationalTime::new(120.0, 24.0), // Start at 5 seconds into source
        RationalTime::new(72.0, 24.0),  // 3 seconds duration
    ));
    let mut ref2 = ExternalReference::new("/media/broll_001.mp4");
    ref2.set_available_range(TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(480.0, 24.0), // 20 seconds
    ))?;
    clip2.set_media_reference(ref2)?;
    v1.append_clip(clip2)?;

    // Add audio track
    let mut a1 = timeline.add_audio_track("A1");
    let mut audio_clip = Clip::new("Music_Track", TimeRange::new(
        RationalTime::new(0.0, 48000.0),
        RationalTime::new(288000.0, 48000.0), // 6 seconds at 48kHz
    ));
    audio_clip.set_metadata("author", "Composer Name");
    a1.append_clip(audio_clip)?;

    // Write to file
    let output_path = Path::new("/tmp/dummy_timeline.otio");
    timeline.write_to_file(output_path)?;

    // Read and print the JSON
    let json = std::fs::read_to_string(output_path).expect("Failed to read output file");
    println!("{json}");

    Ok(())
}

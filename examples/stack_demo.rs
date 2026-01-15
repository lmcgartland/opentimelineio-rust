use otio_rs::*;
use std::path::Path;

fn main() -> otio_rs::Result<()> {
    let mut timeline = Timeline::new("Stack Composition Demo");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0));

    let mut track = timeline.add_video_track("V1");

    // Regular clip first
    let intro = Clip::new("Intro", TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(24.0, 24.0),
    ));
    track.append_clip(intro)?;

    // Stack with multiple clip versions
    let mut version_stack = Stack::new("Scene_01_Versions");
    
    let range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    );
    
    version_stack.append_clip(Clip::new("Scene_01_TakeA", range))?;
    version_stack.append_clip(Clip::new("Scene_01_TakeB", range))?;
    
    track.append_stack(version_stack)?;

    // Write to file
    timeline.write_to_file(Path::new("/tmp/stack_demo.otio"))?;
    
    let json = std::fs::read_to_string("/tmp/stack_demo.otio").unwrap();
    println!("{json}");
    Ok(())
}

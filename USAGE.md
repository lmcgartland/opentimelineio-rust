# Using otio-rs

This guide covers how to use the `otio-rs` crate in your Rust project.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
otio-rs = { git = "https://github.com/lukemcgartland/opentimelineio-rust" }
```

**Note:** The first build will take several minutes as it compiles the OpenTimelineIO C++ library.

## Quick Start

```rust
use otio_rs::{Timeline, Clip, RationalTime, TimeRange, ExternalReference};
use std::path::Path;

fn main() -> otio_rs::Result<()> {
    // Create a timeline
    let mut timeline = Timeline::new("My Project");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0));

    // Add a video track
    let mut track = timeline.add_video_track("V1");

    // Create a clip (2 seconds at 24fps)
    let clip = Clip::new("Shot_001", TimeRange::new(
        RationalTime::new(0.0, 24.0),   // start
        RationalTime::new(48.0, 24.0),  // duration (48 frames = 2 sec)
    ));
    track.append_clip(clip)?;

    // Save to file
    timeline.write_to_file(Path::new("output.otio"))?;

    Ok(())
}
```

## Core Types

### RationalTime

Represents time as a rational number (value/rate):

```rust
use otio_rs::RationalTime;

// 48 frames at 24fps = 2 seconds
let time = RationalTime::new(48.0, 24.0);

// Create from seconds
let time = RationalTime::from_seconds(2.0, 24.0);

// Convert to seconds
let seconds = time.to_seconds(); // 2.0
```

### TimeRange

Represents a span of time with start and duration:

```rust
use otio_rs::{TimeRange, RationalTime};

let range = TimeRange::new(
    RationalTime::new(0.0, 24.0),   // start_time
    RationalTime::new(48.0, 24.0),  // duration
);

let end = range.end_time(); // start + duration
```

## Building Timelines

### Timeline

The top-level container:

```rust
use otio_rs::{Timeline, RationalTime};
use std::path::Path;

let mut timeline = Timeline::new("Project Name");
timeline.set_global_start_time(RationalTime::new(0.0, 24.0));

// Add tracks
let mut video = timeline.add_video_track("V1");
let mut audio = timeline.add_audio_track("A1");

// Access the root stack
let tracks = timeline.tracks();

// Save/load
timeline.write_to_file(Path::new("output.otio"))?;
let loaded = Timeline::read_from_file(Path::new("output.otio"))?;
```

### Track

Tracks contain clips, gaps, and stacks:

```rust
use otio_rs::{Track, Clip, Gap, Stack, TimeRange, RationalTime};

// Tracks from timeline (non-owning)
let mut track = timeline.add_video_track("V1");

// Standalone tracks (owning) - for use with Stack
let mut standalone = Track::new_video("Standalone V1");
let mut audio_track = Track::new_audio("Standalone A1");

// Add content
track.append_clip(clip)?;
track.append_gap(gap)?;
track.append_stack(stack)?;  // for versioning/alternatives
```

### Clip

Represents a segment of media:

```rust
use otio_rs::{Clip, TimeRange, RationalTime, ExternalReference};

let mut clip = Clip::new("Shot_001", TimeRange::new(
    RationalTime::new(100.0, 24.0),  // start at frame 100
    RationalTime::new(48.0, 24.0),   // 2 seconds duration
));

// Set media reference
let mut media_ref = ExternalReference::new("/path/to/media.mov");
media_ref.set_available_range(TimeRange::new(
    RationalTime::new(0.0, 24.0),
    RationalTime::new(1000.0, 24.0),  // 41.6 seconds available
));
clip.set_media_reference(media_ref);

// Set metadata
clip.set_metadata("author", "Editor Name");
clip.set_metadata("notes", "Final take");
```

### Gap

Empty space in a track:

```rust
use otio_rs::{Gap, RationalTime};

// 1 second gap at 24fps
let gap = Gap::new(RationalTime::new(24.0, 24.0));
track.append_gap(gap)?;
```

### Stack

Compositions for layering and versioning:

```rust
use otio_rs::{Stack, Clip, Track, Gap, TimeRange, RationalTime};

let mut stack = Stack::new("Version Alternatives");

// Add clips (layered)
stack.append_clip(Clip::new("Take A", range))?;
stack.append_clip(Clip::new("Take B", range))?;

// Add tracks
let mut track = Track::new_video("Nested Track");
track.append_clip(Clip::new("Nested Clip", range))?;
stack.append_track(track)?;

// Add gaps
stack.append_gap(Gap::new(RationalTime::new(24.0, 24.0)))?;

// Nest stacks
let mut child_stack = Stack::new("Child");
child_stack.append_clip(clip)?;
stack.append_stack(child_stack)?;
```

## Common Patterns

### Video with Audio

```rust
let mut timeline = Timeline::new("Video Project");
timeline.set_global_start_time(RationalTime::new(0.0, 24.0));

// Video track
let mut video = timeline.add_video_track("V1");
video.append_clip(Clip::new("Video", TimeRange::new(
    RationalTime::new(0.0, 24.0),
    RationalTime::new(240.0, 24.0),  // 10 seconds
)))?;

// Audio track (48kHz)
let mut audio = timeline.add_audio_track("A1");
audio.append_clip(Clip::new("Audio", TimeRange::new(
    RationalTime::new(0.0, 48000.0),
    RationalTime::new(480000.0, 48000.0),  // 10 seconds at 48kHz
)))?;
```

### Clip with Gap

```rust
let mut track = timeline.add_video_track("V1");

// First clip
track.append_clip(Clip::new("Intro", TimeRange::new(
    RationalTime::new(0.0, 24.0),
    RationalTime::new(48.0, 24.0),
)))?;

// Gap (1 second)
track.append_gap(Gap::new(RationalTime::new(24.0, 24.0)))?;

// Second clip
track.append_clip(Clip::new("Main", TimeRange::new(
    RationalTime::new(0.0, 24.0),
    RationalTime::new(72.0, 24.0),
)))?;
```

### Version Alternatives (Stack in Track)

```rust
let mut track = timeline.add_video_track("V1");

// Regular clip
track.append_clip(Clip::new("Intro", range))?;

// Stack with alternative versions
let mut versions = Stack::new("Scene 5 Versions");
versions.append_clip(Clip::new("Scene5_TakeA", range))?;
versions.append_clip(Clip::new("Scene5_TakeB", range))?;
versions.append_clip(Clip::new("Scene5_TakeC", range))?;
track.append_stack(versions)?;

// Continue with regular clips
track.append_clip(Clip::new("Outro", range))?;
```

### Nested Composition

```rust
// Create inner stack with tracks
let mut inner_stack = Stack::new("Composite");

let mut fg_track = Track::new_video("Foreground");
fg_track.append_clip(Clip::new("FG Element", range))?;

let mut bg_track = Track::new_video("Background");
bg_track.append_clip(Clip::new("BG Element", range))?;

inner_stack.append_track(fg_track)?;
inner_stack.append_track(bg_track)?;

// Add to main track
let mut main = timeline.add_video_track("V1");
main.append_stack(inner_stack)?;
```

## Error Handling

All fallible operations return `otio_rs::Result<T>`:

```rust
use otio_rs::{Timeline, OtioError};
use std::path::Path;

fn load_timeline(path: &str) -> otio_rs::Result<Timeline> {
    Timeline::read_from_file(Path::new(path))
}

fn main() {
    match load_timeline("project.otio") {
        Ok(timeline) => println!("Loaded timeline"),
        Err(e) => eprintln!("Error: {} (code {})", e.message, e.code),
    }
}
```

## Frame Rate Reference

Common frame rates:

| Rate | Frames/sec | Use Case |
|------|------------|----------|
| 24.0 | 24 | Film |
| 25.0 | 25 | PAL TV |
| 29.97 | 29.97 | NTSC TV |
| 30.0 | 30 | Web video |
| 48000.0 | 48000 | Audio (48kHz) |
| 44100.0 | 44100 | Audio (CD quality) |

Example conversions:
- 2 seconds at 24fps = `RationalTime::new(48.0, 24.0)`
- 5 seconds at 48kHz = `RationalTime::new(240000.0, 48000.0)`
- 1 frame at 29.97fps = `RationalTime::new(1.0, 29.97)`

## Output Format

The `.otio` files are JSON. Example structure:

```json
{
    "OTIO_SCHEMA": "Timeline.1",
    "name": "My Project",
    "tracks": {
        "OTIO_SCHEMA": "Stack.1",
        "children": [
            {
                "OTIO_SCHEMA": "Track.1",
                "name": "V1",
                "kind": "Video",
                "children": [
                    {
                        "OTIO_SCHEMA": "Clip.2",
                        "name": "Shot_001",
                        "source_range": { ... }
                    }
                ]
            }
        ]
    }
}
```

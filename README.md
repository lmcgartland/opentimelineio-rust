# otio-rs

Rust bindings to [OpenTimelineIO](https://opentimeline.io/) - an open-source API and interchange format for editorial timeline information.

[![Test](https://github.com/lukemcgartland/opentimelineio-rust/actions/workflows/test.yml/badge.svg)](https://github.com/lukemcgartland/opentimelineio-rust/actions/workflows/test.yml)

## Status

This crate provides FFI bindings to the C++ OpenTimelineIO library (v0.17.0) via a thin C shim layer.

## Features

- **Timeline creation and manipulation** - Create timelines, tracks, clips, gaps, and stacks
- **Iteration support** - Iterate over children of tracks and stacks with type-safe `Composable` enum
- **Builder pattern** - Fluent API for constructing clips, timelines, and references
- **Metadata support** - Get/set string metadata on all OTIO objects via `HasMetadata` trait
- **Remove/modify operations** - Insert, remove, and clear children from tracks and stacks
- **File I/O** - Read and write `.otio` JSON files
- **Flexible linking** - Use vendored (bundled) or system-installed OpenTimelineIO

## Prerequisites

### All Platforms
- **Rust 1.70+** - Install via [rustup](https://rustup.rs/)
- **CMake 3.18+** - Required for building OpenTimelineIO
- **C++17 compiler** - clang or gcc

### macOS
```bash
# Install Xcode command line tools (includes clang)
xcode-select --install

# Install CMake via Homebrew
brew install cmake
```

### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y cmake clang libstdc++-12-dev
```

### Windows
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with C++ workload
- Install [CMake](https://cmake.org/download/)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
otio-rs = { git = "https://github.com/lukemcgartland/opentimelineio-rust" }
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `vendored` | Yes | Build and link bundled OpenTimelineIO from source |
| `system` | No | Link against system-installed OpenTimelineIO via pkg-config |

To use system-installed OpenTimelineIO instead of vendored:
```toml
[dependencies]
otio-rs = { git = "https://github.com/lukemcgartland/opentimelineio-rust", default-features = false, features = ["system"] }
```

## Quick Start

```rust
use otio_rs::{Timeline, Clip, RationalTime, TimeRange, ExternalReference};
use std::path::Path;

fn main() -> otio_rs::Result<()> {
    // Create a new timeline
    let mut timeline = Timeline::new("My Timeline");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0))?;

    // Add a video track
    let mut video_track = timeline.add_video_track("V1");

    // Create a clip with a 2-second source range at 24fps
    let source_range = TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(48.0, 24.0),
    );
    let mut clip = Clip::new("My Clip", source_range);

    // Set media reference
    let mut media_ref = ExternalReference::new("/path/to/media.mov");
    media_ref.set_available_range(TimeRange::new(
        RationalTime::new(0.0, 24.0),
        RationalTime::new(240.0, 24.0), // 10 seconds
    ))?;
    clip.set_media_reference(media_ref)?;

    // Add clip to track
    video_track.append_clip(clip)?;

    // Write to file
    timeline.write_to_file(Path::new("output.otio"))?;

    // Read back
    let _loaded = Timeline::read_from_file(Path::new("output.otio"))?;

    Ok(())
}
```

## Builder Pattern

Use builders for a fluent construction API:

```rust
use otio_rs::{ClipBuilder, TimelineBuilder, ExternalReferenceBuilder, RationalTime, TimeRange};

let timeline = TimelineBuilder::new("My Project")
    .global_start_time(RationalTime::new(0.0, 24.0))
    .metadata("author", "Jane Doe")
    .build();

let clip = ClipBuilder::new(
    "Interview",
    TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0)),
)
.media_reference(
    ExternalReferenceBuilder::new("/media/interview.mov")
        .available_range(TimeRange::new(
            RationalTime::new(0.0, 24.0),
            RationalTime::new(720.0, 24.0),
        ))
        .metadata("codec", "ProRes")
        .build(),
)
.metadata("speaker", "John Smith")
.build();
```

## Iteration

Iterate over track and stack children:

```rust
use otio_rs::{Timeline, Composable};

let timeline = Timeline::read_from_file(std::path::Path::new("input.otio"))?;

for child in timeline.tracks().children() {
    match child {
        Composable::Track(track_ref) => {
            println!("Track: {}", track_ref.name());
            for item in track_ref.children() {
                match item {
                    Composable::Clip(clip) => println!("  Clip: {}", clip.name()),
                    Composable::Gap(gap) => println!("  Gap: {}", gap.name()),
                    Composable::Stack(stack) => println!("  Stack: {}", stack.name()),
                    Composable::Track(track) => println!("  Track: {}", track.name()),
                }
            }
        }
        _ => {}
    }
}
```

## Metadata

All OTIO objects support string metadata via the `HasMetadata` trait:

```rust
use otio_rs::{Clip, RationalTime, TimeRange, HasMetadata};

let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0));
let mut clip = Clip::new("My Clip", range);

clip.set_metadata("external_id", "abc123");
clip.set_metadata("notes", "Director's preferred take");

assert_eq!(clip.get_metadata("external_id"), Some("abc123".to_string()));
```

## Modify Operations

Insert, remove, and clear children:

```rust
use otio_rs::{Track, Clip, Gap, RationalTime, TimeRange};

let mut track = Track::new_video("V1");
let range = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(24.0, 24.0));

// Append items
track.append_clip(Clip::new("Clip A", range))?;
track.append_clip(Clip::new("Clip B", range))?;

// Insert at specific index
track.insert_gap(1, Gap::new(RationalTime::new(12.0, 24.0)))?;

// Remove by index
track.remove_child(0)?;

// Clear all children
track.clear_children()?;
```

## Building from Source

### 1. Clone the Repository

```bash
# Clone with submodules (required!)
git clone --recursive https://github.com/lukemcgartland/opentimelineio-rust
cd opentimelineio-rust
```

If you already cloned without `--recursive`:
```bash
git submodule update --init --recursive
```

### 2. Build

```bash
# Debug build
cargo build

# Release build (recommended for production)
cargo build --release
```

The first build will take several minutes as it compiles the OpenTimelineIO C++ library.

### 3. Run Tests

```bash
cargo test
```

### 4. Run Examples

```bash
cargo run --example dummy      # Basic timeline creation
cargo run --example iterate    # Iteration demo
cargo run --example modify     # Insert/remove operations
cargo run --example builder    # Builder pattern demo
```

## Architecture

```
otio-rs/
├── Cargo.toml          # Rust package manifest
├── build.rs            # Build script (CMake + bindgen)
├── src/
│   ├── lib.rs          # Safe Rust wrappers
│   ├── types.rs        # Type aliases (Result)
│   ├── traits.rs       # HasMetadata trait
│   ├── iterators.rs    # Iteration support
│   └── builders.rs     # Builder pattern
├── shim/
│   ├── otio_shim.h     # C interface header
│   ├── otio_shim.cpp   # C++ implementation
│   └── CMakeLists.txt  # CMake build configuration
├── vendor/
│   └── OpenTimelineIO/ # Git submodule (v0.17.0)
├── examples/
│   ├── dummy.rs        # Basic usage
│   ├── iterate.rs      # Iteration example
│   ├── modify.rs       # Modify operations
│   └── builder.rs      # Builder pattern
└── tests/
    ├── roundtrip.rs    # File I/O tests
    ├── metadata.rs     # Metadata tests
    └── nested.rs       # Nested structure tests
```

## Troubleshooting

### "could not find native static library"
Ensure CMake completed successfully. Check the build output for CMake errors.

### "opentimelineio/timeline.h not found"
Make sure submodules are initialized:
```bash
git submodule update --init --recursive
```

### Build takes too long
The first build compiles the entire OpenTimelineIO C++ library. Subsequent builds are incremental and much faster. Use `cargo build --release` for optimized builds.

### macOS: "xcrun: error: invalid active developer path"
Install Xcode command line tools:
```bash
xcode-select --install
```

### Using system OpenTimelineIO
If using the `system` feature, ensure OpenTimelineIO is installed and discoverable via pkg-config:
```bash
pkg-config --modversion opentimelineio
```

## License

Licensed under the Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>).

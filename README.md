# otio-rs

Rust bindings to [OpenTimelineIO](https://opentimeline.io/) - an open-source API and interchange format for editorial timeline information.

[![Test](https://github.com/lukemcgartland/opentimelineio-rust/actions/workflows/test.yml/badge.svg)](https://github.com/lukemcgartland/opentimelineio-rust/actions/workflows/test.yml)

## Status

This crate provides FFI bindings to the C++ OpenTimelineIO library (v0.17.0) via a thin C shim layer.

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

### 4. Verify Installation

```bash
# Check that clippy passes
cargo clippy

# Run the doc tests
cargo test --doc
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
otio-rs = { git = "https://github.com/lukemcgartland/opentimelineio-rust" }
```

## Usage

```rust
use otio_rs::{Timeline, Clip, RationalTime, TimeRange, ExternalReference};
use std::path::Path;

fn main() -> otio_rs::Result<()> {
    // Create a new timeline
    let mut timeline = Timeline::new("My Timeline");
    timeline.set_global_start_time(RationalTime::new(0.0, 24.0));

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
    ));
    clip.set_media_reference(media_ref);

    // Add clip to track
    video_track.append_clip(clip)?;

    // Write to file
    timeline.write_to_file(Path::new("output.otio"))?;

    // Read back
    let _loaded = Timeline::read_from_file(Path::new("output.otio"))?;

    Ok(())
}
```

## Architecture

```
otio-rs/
├── Cargo.toml          # Rust package manifest
├── build.rs            # Build script (CMake + bindgen)
├── src/
│   ├── lib.rs          # Safe Rust wrappers
│   └── types.rs        # Type aliases (Result)
├── shim/
│   ├── otio_shim.h     # C interface header
│   ├── otio_shim.cpp   # C++ implementation
│   └── CMakeLists.txt  # CMake build configuration
├── vendor/
│   └── OpenTimelineIO/ # Git submodule (v0.17.0)
└── tests/
    └── roundtrip.rs    # Integration tests
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

## License

Licensed under the Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>).

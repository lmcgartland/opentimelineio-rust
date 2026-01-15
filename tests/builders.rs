use otio_rs::{Clip, ExternalReference, HasMetadata, RationalTime, TimeRange, Timeline};

fn make_time_range(start: f64, duration: f64, rate: f64) -> TimeRange {
    TimeRange::new(
        RationalTime::new(start, rate),
        RationalTime::new(duration, rate),
    )
}

// ============ ClipBuilder Tests ============

#[test]
fn test_clip_builder_basic() {
    let source_range = make_time_range(0.0, 48.0, 24.0);
    let _clip = Clip::builder("test_clip", source_range).build();
    // Clip was created successfully
}

#[test]
fn test_clip_builder_with_metadata() {
    let source_range = make_time_range(0.0, 24.0, 24.0);
    let clip = Clip::builder("clip", source_range)
        .metadata("author", "John")
        .metadata("description", "A test clip")
        .build();

    assert_eq!(clip.get_metadata("author"), Some("John".to_string()));
    assert_eq!(
        clip.get_metadata("description"),
        Some("A test clip".to_string())
    );
}

#[test]
fn test_clip_builder_with_media_reference() {
    let source_range = make_time_range(0.0, 100.0, 24.0);
    let media_ref = ExternalReference::new("/path/to/media.mp4");

    let _clip = Clip::builder("clip", source_range)
        .media_reference(media_ref)
        .build();
    // Clip with media reference was created successfully
}

#[test]
fn test_clip_builder_full_chain() {
    let source_range = make_time_range(10.0, 50.0, 30.0);
    let media_ref = ExternalReference::new("https://example.com/video.mov");

    let clip = Clip::builder("full_clip", source_range)
        .media_reference(media_ref)
        .metadata("project", "Test Project")
        .metadata("scene", "1")
        .metadata("take", "3")
        .build();

    assert_eq!(clip.get_metadata("project"), Some("Test Project".to_string()));
    assert_eq!(clip.get_metadata("scene"), Some("1".to_string()));
    assert_eq!(clip.get_metadata("take"), Some("3".to_string()));
}

// ============ TimelineBuilder Tests ============

#[test]
fn test_timeline_builder_basic() {
    let _tl = Timeline::builder("my_timeline").build();
    // Timeline was created successfully
}

#[test]
fn test_timeline_builder_with_global_start_time() {
    let start_time = RationalTime::new(3600.0, 24.0);
    let _tl = Timeline::builder("timeline")
        .global_start_time(start_time)
        .build();
    // Global start time is set (we can verify through serialization later)
}

#[test]
fn test_timeline_builder_with_metadata() {
    let tl = Timeline::builder("project_timeline")
        .metadata("project_name", "Big Movie")
        .metadata("editor", "Jane Doe")
        .metadata("version", "2")
        .build();

    assert_eq!(tl.get_metadata("project_name"), Some("Big Movie".to_string()));
    assert_eq!(tl.get_metadata("editor"), Some("Jane Doe".to_string()));
    assert_eq!(tl.get_metadata("version"), Some("2".to_string()));
}

#[test]
fn test_timeline_builder_full_chain() {
    let start_time = RationalTime::new(0.0, 24.0);

    let tl = Timeline::builder("complete_timeline")
        .global_start_time(start_time)
        .metadata("format", "OTIO")
        .metadata("source", "Test Suite")
        .build();

    assert_eq!(tl.get_metadata("format"), Some("OTIO".to_string()));
    assert_eq!(tl.get_metadata("source"), Some("Test Suite".to_string()));
}

// ============ ExternalReferenceBuilder Tests ============

#[test]
fn test_external_ref_builder_basic() {
    let _ext_ref = ExternalReference::builder("/path/to/file.mov").build();
    // Basic external reference created successfully
}

#[test]
fn test_external_ref_builder_with_available_range() {
    let available_range = make_time_range(0.0, 1000.0, 24.0);
    let _ext_ref = ExternalReference::builder("https://cdn.example.com/video.mp4")
        .available_range(available_range)
        .build();
    // External reference with available range set
}

#[test]
fn test_external_ref_builder_with_metadata() {
    let ext_ref = ExternalReference::builder("/media/clip.mov")
        .metadata("codec", "ProRes422")
        .metadata("resolution", "1920x1080")
        .build();

    assert_eq!(ext_ref.get_metadata("codec"), Some("ProRes422".to_string()));
    assert_eq!(
        ext_ref.get_metadata("resolution"),
        Some("1920x1080".to_string())
    );
}

#[test]
fn test_external_ref_builder_full_chain() {
    let available_range = make_time_range(0.0, 500.0, 30.0);

    let ext_ref = ExternalReference::builder("s3://bucket/video.mxf")
        .available_range(available_range)
        .metadata("format", "MXF")
        .metadata("storage", "S3")
        .metadata("region", "us-west-2")
        .build();

    assert_eq!(ext_ref.get_metadata("format"), Some("MXF".to_string()));
    assert_eq!(ext_ref.get_metadata("storage"), Some("S3".to_string()));
    assert_eq!(ext_ref.get_metadata("region"), Some("us-west-2".to_string()));
}

// ============ Builder Integration Tests ============

#[test]
fn test_builders_in_timeline_construction() {
    // Build a complete timeline using builders
    let mut tl = Timeline::builder("production_timeline")
        .global_start_time(RationalTime::new(0.0, 24.0))
        .metadata("project", "Documentary")
        .build();

    let mut track = tl.add_video_track("V1");

    // Create clips using builder
    let media1 = ExternalReference::builder("/footage/shot001.mov")
        .available_range(make_time_range(0.0, 240.0, 24.0))
        .metadata("camera", "A")
        .build();

    let clip1 = Clip::builder("Interview A", make_time_range(100.0, 48.0, 24.0))
        .media_reference(media1)
        .metadata("scene", "1")
        .build();

    let media2 = ExternalReference::builder("/footage/shot002.mov")
        .available_range(make_time_range(0.0, 360.0, 24.0))
        .metadata("camera", "B")
        .build();

    let clip2 = Clip::builder("B-Roll", make_time_range(50.0, 72.0, 24.0))
        .media_reference(media2)
        .metadata("scene", "1")
        .metadata("type", "b-roll")
        .build();

    track.append_clip(clip1).unwrap();
    track.append_clip(clip2).unwrap();

    assert_eq!(track.children_count(), 2);
}

#[test]
fn test_builder_method_chaining_order_independence() {
    // Metadata can be added in any order
    let clip1 = Clip::builder("clip", make_time_range(0.0, 24.0, 24.0))
        .metadata("a", "1")
        .metadata("b", "2")
        .metadata("c", "3")
        .build();

    let clip2 = Clip::builder("clip", make_time_range(0.0, 24.0, 24.0))
        .metadata("c", "3")
        .metadata("a", "1")
        .metadata("b", "2")
        .build();

    // Both should have the same metadata regardless of order
    assert_eq!(clip1.get_metadata("a"), clip2.get_metadata("a"));
    assert_eq!(clip1.get_metadata("b"), clip2.get_metadata("b"));
    assert_eq!(clip1.get_metadata("c"), clip2.get_metadata("c"));
}

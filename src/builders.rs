//! Builder pattern implementations for OTIO types.
//!
//! Builders provide a fluent API for constructing complex OTIO objects
//! with optional fields.
//!
//! Each builder provides two build methods:
//! - `build()` - Returns `Result<T>`, propagating any errors
//! - `build_unchecked()` - Returns `T`, ignoring any errors (for convenience)

use crate::{Clip, ExternalReference, HasMetadata, RationalTime, Result, TimeRange, Timeline};

/// Builder for creating `Clip` instances.
///
/// # Example
///
/// ```no_run
/// use otio_rs::{ClipBuilder, ExternalReference, RationalTime, TimeRange};
///
/// let clip = ClipBuilder::new("My Clip", TimeRange::new(
///         RationalTime::new(0.0, 24.0),
///         RationalTime::new(48.0, 24.0),
///     ))
///     .media_reference(ExternalReference::new("/path/to/media.mov"))
///     .metadata("author", "Jane Doe")
///     .metadata("project", "Demo")
///     .build()
///     .unwrap();
/// ```
pub struct ClipBuilder {
    name: String,
    source_range: TimeRange,
    media_reference: Option<ExternalReference>,
    metadata: Vec<(String, String)>,
}

impl ClipBuilder {
    /// Create a new clip builder with the required name and source range.
    #[must_use]
    pub fn new(name: &str, source_range: TimeRange) -> Self {
        Self {
            name: name.to_string(),
            source_range,
            media_reference: None,
            metadata: Vec::new(),
        }
    }

    /// Set the media reference for this clip.
    #[must_use]
    pub fn media_reference(mut self, reference: ExternalReference) -> Self {
        self.media_reference = Some(reference);
        self
    }

    /// Add a metadata key-value pair.
    #[must_use]
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.push((key.to_string(), value.to_string()));
        self
    }

    /// Build the clip, returning an error if any operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if setting the media reference fails.
    pub fn build(self) -> Result<Clip> {
        let mut clip = Clip::new(&self.name, self.source_range);

        if let Some(reference) = self.media_reference {
            clip.set_media_reference(reference)?;
        }

        for (key, value) in self.metadata {
            clip.set_metadata(&key, &value);
        }

        Ok(clip)
    }

    /// Build the clip, ignoring any errors.
    ///
    /// Use this when you don't care about errors during construction.
    #[must_use]
    pub fn build_unchecked(self) -> Clip {
        let mut clip = Clip::new(&self.name, self.source_range);

        if let Some(reference) = self.media_reference {
            let _ = clip.set_media_reference(reference);
        }

        for (key, value) in self.metadata {
            clip.set_metadata(&key, &value);
        }

        clip
    }
}

/// Builder for creating `Timeline` instances.
///
/// # Example
///
/// ```no_run
/// use otio_rs::{TimelineBuilder, RationalTime};
///
/// let timeline = TimelineBuilder::new("My Project")
///     .global_start_time(RationalTime::new(0.0, 24.0))
///     .metadata("author", "John Smith")
///     .build()
///     .unwrap();
/// ```
pub struct TimelineBuilder {
    name: String,
    global_start_time: Option<RationalTime>,
    metadata: Vec<(String, String)>,
}

impl TimelineBuilder {
    /// Create a new timeline builder with the required name.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            global_start_time: None,
            metadata: Vec::new(),
        }
    }

    /// Set the global start time for this timeline.
    #[must_use]
    pub fn global_start_time(mut self, time: RationalTime) -> Self {
        self.global_start_time = Some(time);
        self
    }

    /// Add a metadata key-value pair.
    #[must_use]
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.push((key.to_string(), value.to_string()));
        self
    }

    /// Build the timeline, returning an error if any operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if setting the global start time fails.
    pub fn build(self) -> Result<Timeline> {
        let mut timeline = Timeline::new(&self.name);

        if let Some(time) = self.global_start_time {
            timeline.set_global_start_time(time)?;
        }

        for (key, value) in self.metadata {
            timeline.set_metadata(&key, &value);
        }

        Ok(timeline)
    }

    /// Build the timeline, ignoring any errors.
    ///
    /// Use this when you don't care about errors during construction.
    #[must_use]
    pub fn build_unchecked(self) -> Timeline {
        let mut timeline = Timeline::new(&self.name);

        if let Some(time) = self.global_start_time {
            let _ = timeline.set_global_start_time(time);
        }

        for (key, value) in self.metadata {
            timeline.set_metadata(&key, &value);
        }

        timeline
    }
}

/// Builder for creating `ExternalReference` instances.
///
/// # Example
///
/// ```no_run
/// use otio_rs::{ExternalReferenceBuilder, RationalTime, TimeRange};
///
/// let reference = ExternalReferenceBuilder::new("/path/to/media.mov")
///     .available_range(TimeRange::new(
///         RationalTime::new(0.0, 24.0),
///         RationalTime::new(720.0, 24.0),
///     ))
///     .metadata("codec", "ProRes")
///     .build()
///     .unwrap();
/// ```
pub struct ExternalReferenceBuilder {
    target_url: String,
    available_range: Option<TimeRange>,
    metadata: Vec<(String, String)>,
}

impl ExternalReferenceBuilder {
    /// Create a new external reference builder with the required URL.
    #[must_use]
    pub fn new(target_url: &str) -> Self {
        Self {
            target_url: target_url.to_string(),
            available_range: None,
            metadata: Vec::new(),
        }
    }

    /// Set the available range for this media reference.
    #[must_use]
    pub fn available_range(mut self, range: TimeRange) -> Self {
        self.available_range = Some(range);
        self
    }

    /// Add a metadata key-value pair.
    #[must_use]
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.push((key.to_string(), value.to_string()));
        self
    }

    /// Build the external reference, returning an error if any operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if setting the available range fails.
    pub fn build(self) -> Result<ExternalReference> {
        let mut reference = ExternalReference::new(&self.target_url);

        if let Some(range) = self.available_range {
            reference.set_available_range(range)?;
        }

        for (key, value) in self.metadata {
            reference.set_metadata(&key, &value);
        }

        Ok(reference)
    }

    /// Build the external reference, ignoring any errors.
    ///
    /// Use this when you don't care about errors during construction.
    #[must_use]
    pub fn build_unchecked(self) -> ExternalReference {
        let mut reference = ExternalReference::new(&self.target_url);

        if let Some(range) = self.available_range {
            let _ = reference.set_available_range(range);
        }

        for (key, value) in self.metadata {
            reference.set_metadata(&key, &value);
        }

        reference
    }
}

// Convenience methods on the types themselves

impl Clip {
    /// Create a builder for a new clip.
    #[must_use]
    pub fn builder(name: &str, source_range: TimeRange) -> ClipBuilder {
        ClipBuilder::new(name, source_range)
    }
}

impl Timeline {
    /// Create a builder for a new timeline.
    #[must_use]
    pub fn builder(name: &str) -> TimelineBuilder {
        TimelineBuilder::new(name)
    }
}

impl ExternalReference {
    /// Create a builder for a new external reference.
    #[must_use]
    pub fn builder(target_url: &str) -> ExternalReferenceBuilder {
        ExternalReferenceBuilder::new(target_url)
    }
}

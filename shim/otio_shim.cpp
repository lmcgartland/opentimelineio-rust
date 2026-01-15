#include "otio_shim.h"

// Include individual OTIO headers
#include "opentimelineio/timeline.h"
#include "opentimelineio/track.h"
#include "opentimelineio/clip.h"
#include "opentimelineio/gap.h"
#include "opentimelineio/externalReference.h"
#include "opentimelineio/errorStatus.h"
#include "opentimelineio/stack.h"
#include "opentimelineio/serializableObject.h"

#include <cstring>
#include <exception>

namespace otio = opentimelineio::OPENTIMELINEIO_VERSION;

// Use Retainer for reference-counted pointers
template <typename T>
using Retainer = otio::SerializableObject::Retainer<T>;

// ============================================================================
// Helper Macros - Reduce boilerplate and ensure consistent error handling
// ============================================================================

// Null pointer check - returns failure_val if ptr is null
#define OTIO_NULL_CHECK(ptr, failure_val) \
    do { if (!(ptr)) return (failure_val); } while(0)

// Null check with error message
#define OTIO_NULL_CHECK_ERR(ptr, err, failure_val, msg) \
    do { \
        if (!(ptr)) { \
            set_error(err, 1, msg); \
            return (failure_val); \
        } \
    } while(0)

// Try-catch wrapper for functions returning int (0 = success, -1 = error)
#define OTIO_TRY_INT(err, body) \
    try { \
        body \
        return 0; \
    } catch (const std::exception& e) { \
        set_error(err, 1, e.what()); \
        return -1; \
    } catch (...) { \
        set_error(err, 1, "Unknown exception"); \
        return -1; \
    }

// Try-catch wrapper for functions returning pointers (nullptr on error)
#define OTIO_TRY_PTR(body) \
    try { \
        body \
    } catch (...) { \
        return nullptr; \
    }

// Try-catch wrapper for functions returning int32_t (default_val on error)
#define OTIO_TRY_INT32(default_val, body) \
    try { \
        body \
    } catch (...) { \
        return (default_val); \
    }

// Check OTIO ErrorStatus and return -1 if error
#define OTIO_CHECK_STATUS(status, err) \
    do { \
        if (otio::is_error(status)) { \
            set_error(err, 1, (status).full_description.c_str()); \
            return -1; \
        } \
    } while(0)

// Cast opaque handle to OTIO type
#define OTIO_CAST(Type, var, handle) \
    auto var = reinterpret_cast<otio::Type*>(handle)

// ============================================================================
// Helper Functions
// ============================================================================

static otio::RationalTime to_otio_rt(OtioRationalTime rt) {
    return otio::RationalTime(rt.value, rt.rate);
}

static otio::TimeRange to_otio_tr(OtioTimeRange tr) {
    return otio::TimeRange(to_otio_rt(tr.start_time), to_otio_rt(tr.duration));
}

static void set_error(OtioError* err, int code, const char* msg) {
    if (err) {
        err->code = code;
        strncpy(err->message, msg, sizeof(err->message) - 1);
        err->message[sizeof(err->message) - 1] = '\0';
    }
}

// Safe strdup that returns empty string on failure
static char* safe_strdup(const char* s) {
    if (!s) return strdup("");
    char* result = strdup(s);
    return result ? result : strdup("");
}

static char* safe_strdup(const std::string& s) {
    return safe_strdup(s.c_str());
}

// ============================================================================
// Template helpers for metadata
// ============================================================================

template<typename T>
static char* get_metadata_string_impl(T* obj, const char* key) {
    if (!obj || !key) return nullptr;
    OTIO_TRY_PTR(
        auto& meta = obj->metadata();
        auto it = meta.find(std::string(key));
        if (it != meta.end()) {
            if (it->second.type() == typeid(std::string)) {
                const std::string& value = std::any_cast<const std::string&>(it->second);
                return safe_strdup(value);
            }
        }
        return nullptr;
    )
}

template<typename T>
static void set_metadata_string_impl(T* obj, const char* key, const char* value) {
    if (!obj || !key || !value) return;
    try {
        obj->metadata()[std::string(key)] = std::string(value);
    } catch (...) {
        // Ignore errors in metadata setting
    }
}

// ============================================================================
// Template helpers for child operations
// ============================================================================

template<typename Container>
static int32_t children_count_impl(Container* container) {
    if (!container) return 0;
    try {
        return static_cast<int32_t>(container->children().size());
    } catch (...) {
        return 0;
    }
}

template<typename Container>
static int32_t child_type_impl(Container* container, int32_t index) {
    if (!container) return -1;
    try {
        auto& children = container->children();
        if (index < 0 || static_cast<size_t>(index) >= children.size()) {
            return -1;
        }
        auto child = children[index].value;
        if (dynamic_cast<otio::Clip*>(child)) return OTIO_CHILD_TYPE_CLIP;
        if (dynamic_cast<otio::Gap*>(child)) return OTIO_CHILD_TYPE_GAP;
        if (dynamic_cast<otio::Stack*>(child)) return OTIO_CHILD_TYPE_STACK;
        if (dynamic_cast<otio::Track*>(child)) return OTIO_CHILD_TYPE_TRACK;
        return -1;
    } catch (...) {
        return -1;
    }
}

template<typename Container>
static void* child_at_impl(Container* container, int32_t index) {
    if (!container) return nullptr;
    try {
        auto& children = container->children();
        if (index < 0 || static_cast<size_t>(index) >= children.size()) {
            return nullptr;
        }
        return children[index].value;
    } catch (...) {
        return nullptr;
    }
}

template<typename Container, typename Child>
static int append_child_impl(Container* container, Child* child, OtioError* err) {
    OTIO_NULL_CHECK_ERR(container, err, -1, "Container is null");
    OTIO_NULL_CHECK_ERR(child, err, -1, "Child is null");
    try {
        otio::ErrorStatus status;
        container->append_child(child, &status);
        OTIO_CHECK_STATUS(status, err);
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

template<typename Container, typename Child>
static int insert_child_impl(Container* container, int32_t index, Child* child, OtioError* err) {
    OTIO_NULL_CHECK_ERR(container, err, -1, "Container is null");
    OTIO_NULL_CHECK_ERR(child, err, -1, "Child is null");
    try {
        otio::ErrorStatus status;
        container->insert_child(index, child, &status);
        OTIO_CHECK_STATUS(status, err);
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

template<typename Container>
static int remove_child_impl(Container* container, int32_t index, OtioError* err) {
    OTIO_NULL_CHECK_ERR(container, err, -1, "Container is null");
    try {
        auto& children = container->children();
        if (index < 0 || static_cast<size_t>(index) >= children.size()) {
            set_error(err, 1, "Index out of bounds");
            return -1;
        }
        otio::ErrorStatus status;
        container->remove_child(index, &status);
        OTIO_CHECK_STATUS(status, err);
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

template<typename Container>
static int clear_children_impl(Container* container, OtioError* err) {
    OTIO_NULL_CHECK_ERR(container, err, -1, "Container is null");
    try {
        container->clear_children();
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

// ============================================================================
// C API Implementation
// ============================================================================

extern "C" {

// ----------------------------------------------------------------------------
// Timeline
// ----------------------------------------------------------------------------

OtioTimeline* otio_timeline_create(const char* name) {
    OTIO_NULL_CHECK(name, nullptr);
    OTIO_TRY_PTR(
        auto tl = new otio::Timeline(name);
        tl->set_tracks(new otio::Stack());
        Retainer<otio::Timeline> retainer(tl);
        return reinterpret_cast<OtioTimeline*>(retainer.take_value());
    )
}

void otio_timeline_free(OtioTimeline* tl) {
    if (tl) {
        try {
            OTIO_CAST(Timeline, timeline, tl);
            Retainer<otio::Timeline> retainer(timeline);
        } catch (...) {
            // Ignore exceptions during cleanup
        }
    }
}

int otio_timeline_set_global_start_time(OtioTimeline* tl, OtioRationalTime time, OtioError* err) {
    OTIO_NULL_CHECK_ERR(tl, err, -1, "Timeline is null");
    OTIO_TRY_INT(err,
        OTIO_CAST(Timeline, timeline, tl);
        timeline->set_global_start_time(to_otio_rt(time));
    )
}

OtioTrack* otio_timeline_add_video_track(OtioTimeline* tl, const char* name) {
    OTIO_NULL_CHECK(tl, nullptr);
    OTIO_NULL_CHECK(name, nullptr);
    OTIO_TRY_PTR(
        OTIO_CAST(Timeline, timeline, tl);
        auto track = new otio::Track(name, std::nullopt, otio::Track::Kind::video);
        otio::ErrorStatus err;
        timeline->tracks()->append_child(track, &err);
        return reinterpret_cast<OtioTrack*>(track);
    )
}

OtioTrack* otio_timeline_add_audio_track(OtioTimeline* tl, const char* name) {
    OTIO_NULL_CHECK(tl, nullptr);
    OTIO_NULL_CHECK(name, nullptr);
    OTIO_TRY_PTR(
        OTIO_CAST(Timeline, timeline, tl);
        auto track = new otio::Track(name, std::nullopt, otio::Track::Kind::audio);
        otio::ErrorStatus err;
        timeline->tracks()->append_child(track, &err);
        return reinterpret_cast<OtioTrack*>(track);
    )
}

OtioStack* otio_timeline_get_tracks(OtioTimeline* tl) {
    OTIO_NULL_CHECK(tl, nullptr);
    OTIO_TRY_PTR(
        OTIO_CAST(Timeline, timeline, tl);
        return reinterpret_cast<OtioStack*>(timeline->tracks());
    )
}

// ----------------------------------------------------------------------------
// Track
// ----------------------------------------------------------------------------

OtioTrack* otio_track_create_video(const char* name) {
    OTIO_NULL_CHECK(name, nullptr);
    OTIO_TRY_PTR(
        auto track = new otio::Track(name, std::nullopt, otio::Track::Kind::video);
        Retainer<otio::Track> retainer(track);
        return reinterpret_cast<OtioTrack*>(retainer.take_value());
    )
}

OtioTrack* otio_track_create_audio(const char* name) {
    OTIO_NULL_CHECK(name, nullptr);
    OTIO_TRY_PTR(
        auto track = new otio::Track(name, std::nullopt, otio::Track::Kind::audio);
        Retainer<otio::Track> retainer(track);
        return reinterpret_cast<OtioTrack*>(retainer.take_value());
    )
}

void otio_track_free(OtioTrack* track) {
    if (track) {
        try {
            OTIO_CAST(Track, t, track);
            Retainer<otio::Track> retainer(t);
        } catch (...) {
            // Ignore exceptions during cleanup
        }
    }
}

int otio_track_append_clip(OtioTrack* track, OtioClip* clip, OtioError* err) {
    return append_child_impl<otio::Track, otio::Clip>(
        reinterpret_cast<otio::Track*>(track),
        reinterpret_cast<otio::Clip*>(clip), err);
}

int otio_track_append_gap(OtioTrack* track, OtioGap* gap, OtioError* err) {
    return append_child_impl<otio::Track, otio::Gap>(
        reinterpret_cast<otio::Track*>(track),
        reinterpret_cast<otio::Gap*>(gap), err);
}

int otio_track_append_stack(OtioTrack* track, OtioStack* stack, OtioError* err) {
    return append_child_impl<otio::Track, otio::Stack>(
        reinterpret_cast<otio::Track*>(track),
        reinterpret_cast<otio::Stack*>(stack), err);
}

int32_t otio_track_children_count(OtioTrack* track) {
    return children_count_impl(reinterpret_cast<otio::Track*>(track));
}

int32_t otio_track_child_type(OtioTrack* track, int32_t index) {
    return child_type_impl(reinterpret_cast<otio::Track*>(track), index);
}

void* otio_track_child_at(OtioTrack* track, int32_t index) {
    return child_at_impl(reinterpret_cast<otio::Track*>(track), index);
}

int otio_track_remove_child(OtioTrack* track, int32_t index, OtioError* err) {
    return remove_child_impl(reinterpret_cast<otio::Track*>(track), index, err);
}

int otio_track_insert_clip(OtioTrack* track, int32_t index, OtioClip* clip, OtioError* err) {
    return insert_child_impl<otio::Track, otio::Clip>(
        reinterpret_cast<otio::Track*>(track), index,
        reinterpret_cast<otio::Clip*>(clip), err);
}

int otio_track_insert_gap(OtioTrack* track, int32_t index, OtioGap* gap, OtioError* err) {
    return insert_child_impl<otio::Track, otio::Gap>(
        reinterpret_cast<otio::Track*>(track), index,
        reinterpret_cast<otio::Gap*>(gap), err);
}

int otio_track_insert_stack(OtioTrack* track, int32_t index, OtioStack* stack, OtioError* err) {
    return insert_child_impl<otio::Track, otio::Stack>(
        reinterpret_cast<otio::Track*>(track), index,
        reinterpret_cast<otio::Stack*>(stack), err);
}

int otio_track_clear_children(OtioTrack* track, OtioError* err) {
    return clear_children_impl(reinterpret_cast<otio::Track*>(track), err);
}

// ----------------------------------------------------------------------------
// Clip
// ----------------------------------------------------------------------------

OtioClip* otio_clip_create(const char* name, OtioTimeRange source_range) {
    OTIO_NULL_CHECK(name, nullptr);
    OTIO_TRY_PTR(
        auto clip = new otio::Clip(name, nullptr, to_otio_tr(source_range));
        Retainer<otio::Clip> retainer(clip);
        return reinterpret_cast<OtioClip*>(retainer.take_value());
    )
}

int otio_clip_set_media_reference(OtioClip* clip, OtioExternalRef* ref, OtioError* err) {
    OTIO_NULL_CHECK_ERR(clip, err, -1, "Clip is null");
    OTIO_NULL_CHECK_ERR(ref, err, -1, "Reference is null");
    OTIO_TRY_INT(err,
        OTIO_CAST(Clip, c, clip);
        OTIO_CAST(ExternalReference, r, ref);
        c->set_media_reference(r);
    )
}

char* otio_clip_get_name(OtioClip* clip) {
    OTIO_NULL_CHECK(clip, nullptr);
    OTIO_TRY_PTR(
        OTIO_CAST(Clip, c, clip);
        return safe_strdup(c->name());
    )
}

OtioTimeRange otio_clip_get_source_range(OtioClip* clip) {
    OtioTimeRange zero = {OtioRationalTime{0, 1}, OtioRationalTime{0, 1}};
    if (!clip) return zero;
    try {
        OTIO_CAST(Clip, c, clip);
        auto sr = c->source_range();
        if (sr.has_value()) {
            auto& range = sr.value();
            return OtioTimeRange{
                OtioRationalTime{range.start_time().value(), range.start_time().rate()},
                OtioRationalTime{range.duration().value(), range.duration().rate()}
            };
        }
    } catch (...) {
    }
    return zero;
}

// ----------------------------------------------------------------------------
// Gap
// ----------------------------------------------------------------------------

OtioGap* otio_gap_create(OtioRationalTime duration) {
    OTIO_TRY_PTR(
        auto gap = new otio::Gap(otio::TimeRange(
            otio::RationalTime(0, duration.rate),
            to_otio_rt(duration)
        ));
        Retainer<otio::Gap> retainer(gap);
        return reinterpret_cast<OtioGap*>(retainer.take_value());
    )
}

char* otio_gap_get_name(OtioGap* gap) {
    OTIO_NULL_CHECK(gap, nullptr);
    OTIO_TRY_PTR(
        OTIO_CAST(Gap, g, gap);
        return safe_strdup(g->name());
    )
}

// ----------------------------------------------------------------------------
// ExternalReference
// ----------------------------------------------------------------------------

OtioExternalRef* otio_external_ref_create(const char* target_url) {
    OTIO_NULL_CHECK(target_url, nullptr);
    OTIO_TRY_PTR(
        auto ref = new otio::ExternalReference(target_url);
        Retainer<otio::ExternalReference> retainer(ref);
        return reinterpret_cast<OtioExternalRef*>(retainer.take_value());
    )
}

int otio_external_ref_set_available_range(OtioExternalRef* ref, OtioTimeRange range, OtioError* err) {
    OTIO_NULL_CHECK_ERR(ref, err, -1, "Reference is null");
    OTIO_TRY_INT(err,
        OTIO_CAST(ExternalReference, r, ref);
        r->set_available_range(to_otio_tr(range));
    )
}

void otio_external_ref_free(OtioExternalRef* ref) {
    // ExternalReferences are typically owned by clips via set_media_reference.
    // This function exists for API completeness but the reference is managed
    // by OTIO's reference counting when attached to a clip.
    (void)ref;
}

// ----------------------------------------------------------------------------
// Stack
// ----------------------------------------------------------------------------

OtioStack* otio_stack_create(const char* name) {
    OTIO_NULL_CHECK(name, nullptr);
    OTIO_TRY_PTR(
        auto stack = new otio::Stack(name);
        Retainer<otio::Stack> retainer(stack);
        return reinterpret_cast<OtioStack*>(retainer.take_value());
    )
}

void otio_stack_free(OtioStack* stack) {
    if (stack) {
        try {
            OTIO_CAST(Stack, s, stack);
            Retainer<otio::Stack> retainer(s);
        } catch (...) {
            // Ignore exceptions during cleanup
        }
    }
}

int otio_stack_append_track(OtioStack* stack, OtioTrack* track, OtioError* err) {
    return append_child_impl<otio::Stack, otio::Track>(
        reinterpret_cast<otio::Stack*>(stack),
        reinterpret_cast<otio::Track*>(track), err);
}

int otio_stack_append_clip(OtioStack* stack, OtioClip* clip, OtioError* err) {
    return append_child_impl<otio::Stack, otio::Clip>(
        reinterpret_cast<otio::Stack*>(stack),
        reinterpret_cast<otio::Clip*>(clip), err);
}

int otio_stack_append_gap(OtioStack* stack, OtioGap* gap, OtioError* err) {
    return append_child_impl<otio::Stack, otio::Gap>(
        reinterpret_cast<otio::Stack*>(stack),
        reinterpret_cast<otio::Gap*>(gap), err);
}

int otio_stack_append_stack(OtioStack* stack, OtioStack* child, OtioError* err) {
    return append_child_impl<otio::Stack, otio::Stack>(
        reinterpret_cast<otio::Stack*>(stack),
        reinterpret_cast<otio::Stack*>(child), err);
}

int32_t otio_stack_children_count(OtioStack* stack) {
    return children_count_impl(reinterpret_cast<otio::Stack*>(stack));
}

int32_t otio_stack_child_type(OtioStack* stack, int32_t index) {
    return child_type_impl(reinterpret_cast<otio::Stack*>(stack), index);
}

void* otio_stack_child_at(OtioStack* stack, int32_t index) {
    return child_at_impl(reinterpret_cast<otio::Stack*>(stack), index);
}

int otio_stack_remove_child(OtioStack* stack, int32_t index, OtioError* err) {
    return remove_child_impl(reinterpret_cast<otio::Stack*>(stack), index, err);
}

int otio_stack_insert_track(OtioStack* stack, int32_t index, OtioTrack* track, OtioError* err) {
    return insert_child_impl<otio::Stack, otio::Track>(
        reinterpret_cast<otio::Stack*>(stack), index,
        reinterpret_cast<otio::Track*>(track), err);
}

int otio_stack_insert_clip(OtioStack* stack, int32_t index, OtioClip* clip, OtioError* err) {
    return insert_child_impl<otio::Stack, otio::Clip>(
        reinterpret_cast<otio::Stack*>(stack), index,
        reinterpret_cast<otio::Clip*>(clip), err);
}

int otio_stack_insert_gap(OtioStack* stack, int32_t index, OtioGap* gap, OtioError* err) {
    return insert_child_impl<otio::Stack, otio::Gap>(
        reinterpret_cast<otio::Stack*>(stack), index,
        reinterpret_cast<otio::Gap*>(gap), err);
}

int otio_stack_insert_stack(OtioStack* stack, int32_t index, OtioStack* child, OtioError* err) {
    return insert_child_impl<otio::Stack, otio::Stack>(
        reinterpret_cast<otio::Stack*>(stack), index,
        reinterpret_cast<otio::Stack*>(child), err);
}

int otio_stack_clear_children(OtioStack* stack, OtioError* err) {
    return clear_children_impl(reinterpret_cast<otio::Stack*>(stack), err);
}

char* otio_stack_get_name(OtioStack* stack) {
    OTIO_NULL_CHECK(stack, nullptr);
    OTIO_TRY_PTR(
        OTIO_CAST(Stack, s, stack);
        return safe_strdup(s->name());
    )
}

char* otio_track_get_name(OtioTrack* track) {
    OTIO_NULL_CHECK(track, nullptr);
    OTIO_TRY_PTR(
        OTIO_CAST(Track, t, track);
        return safe_strdup(t->name());
    )
}

// ----------------------------------------------------------------------------
// Serialization
// ----------------------------------------------------------------------------

int otio_timeline_write_to_file(OtioTimeline* tl, const char* path, OtioError* err) {
    OTIO_NULL_CHECK_ERR(tl, err, -1, "Timeline is null");
    OTIO_NULL_CHECK_ERR(path, err, -1, "Path is null");
    OTIO_TRY_INT(err,
        OTIO_CAST(Timeline, timeline, tl);
        otio::ErrorStatus status;
        bool success = timeline->to_json_file(path, &status);
        if (!success || otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
    )
}

OtioTimeline* otio_timeline_read_from_file(const char* path, OtioError* err) {
    if (!path) {
        set_error(err, 1, "Path is null");
        return nullptr;
    }
    try {
        otio::ErrorStatus status;
        auto result = otio::SerializableObject::from_json_file(path, &status);
        if (otio::is_error(status) || !result) {
            set_error(err, 1, status.full_description.c_str());
            return nullptr;
        }
        auto timeline = dynamic_cast<otio::Timeline*>(result);
        if (!timeline) {
            set_error(err, 1, "File does not contain a Timeline");
            Retainer<otio::SerializableObject> retainer(result);
            return nullptr;
        }
        Retainer<otio::Timeline> retainer(timeline);
        return reinterpret_cast<OtioTimeline*>(retainer.take_value());
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return nullptr;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return nullptr;
    }
}

// ----------------------------------------------------------------------------
// String memory management
// ----------------------------------------------------------------------------

void otio_free_string(char* str) {
    free(str);
}

// ----------------------------------------------------------------------------
// Metadata - using template helpers
// ----------------------------------------------------------------------------

void otio_timeline_set_metadata_string(OtioTimeline* tl, const char* key, const char* value) {
    set_metadata_string_impl(reinterpret_cast<otio::Timeline*>(tl), key, value);
}

char* otio_timeline_get_metadata_string(OtioTimeline* tl, const char* key) {
    return get_metadata_string_impl(reinterpret_cast<otio::Timeline*>(tl), key);
}

void otio_track_set_metadata_string(OtioTrack* track, const char* key, const char* value) {
    set_metadata_string_impl(reinterpret_cast<otio::Track*>(track), key, value);
}

char* otio_track_get_metadata_string(OtioTrack* track, const char* key) {
    return get_metadata_string_impl(reinterpret_cast<otio::Track*>(track), key);
}

void otio_clip_set_metadata_string(OtioClip* clip, const char* key, const char* value) {
    set_metadata_string_impl(reinterpret_cast<otio::Clip*>(clip), key, value);
}

char* otio_clip_get_metadata_string(OtioClip* clip, const char* key) {
    return get_metadata_string_impl(reinterpret_cast<otio::Clip*>(clip), key);
}

void otio_gap_set_metadata_string(OtioGap* gap, const char* key, const char* value) {
    set_metadata_string_impl(reinterpret_cast<otio::Gap*>(gap), key, value);
}

char* otio_gap_get_metadata_string(OtioGap* gap, const char* key) {
    return get_metadata_string_impl(reinterpret_cast<otio::Gap*>(gap), key);
}

void otio_stack_set_metadata_string(OtioStack* stack, const char* key, const char* value) {
    set_metadata_string_impl(reinterpret_cast<otio::Stack*>(stack), key, value);
}

char* otio_stack_get_metadata_string(OtioStack* stack, const char* key) {
    return get_metadata_string_impl(reinterpret_cast<otio::Stack*>(stack), key);
}

void otio_external_ref_set_metadata_string(OtioExternalRef* ref, const char* key, const char* value) {
    set_metadata_string_impl(reinterpret_cast<otio::ExternalReference*>(ref), key, value);
}

char* otio_external_ref_get_metadata_string(OtioExternalRef* ref, const char* key) {
    return get_metadata_string_impl(reinterpret_cast<otio::ExternalReference*>(ref), key);
}

} // extern "C"

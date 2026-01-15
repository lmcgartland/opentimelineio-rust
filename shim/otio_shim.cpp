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

// Helper: convert our RationalTime to OTIO's
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

extern "C" {

OtioTimeline* otio_timeline_create(const char* name) {
    try {
        auto tl = new otio::Timeline(name);
        tl->set_tracks(new otio::Stack());
        Retainer<otio::Timeline> retainer(tl);
        return reinterpret_cast<OtioTimeline*>(retainer.take_value());
    } catch (const std::exception& e) {
        return nullptr;
    } catch (...) {
        return nullptr;
    }
}

void otio_timeline_free(OtioTimeline* tl) {
    if (tl) {
        try {
            auto timeline = reinterpret_cast<otio::Timeline*>(tl);
            Retainer<otio::Timeline> retainer(timeline);
        } catch (...) {
            // Ignore exceptions during cleanup
        }
    }
}

int otio_timeline_set_global_start_time(OtioTimeline* tl, OtioRationalTime time, OtioError* err) {
    try {
        auto timeline = reinterpret_cast<otio::Timeline*>(tl);
        timeline->set_global_start_time(to_otio_rt(time));
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

OtioTrack* otio_timeline_add_video_track(OtioTimeline* tl, const char* name) {
    try {
        auto timeline = reinterpret_cast<otio::Timeline*>(tl);
        auto track = new otio::Track(name, std::nullopt, otio::Track::Kind::video);
        otio::ErrorStatus err;
        timeline->tracks()->append_child(track, &err);
        return reinterpret_cast<OtioTrack*>(track);
    } catch (...) {
        return nullptr;
    }
}

OtioTrack* otio_timeline_add_audio_track(OtioTimeline* tl, const char* name) {
    try {
        auto timeline = reinterpret_cast<otio::Timeline*>(tl);
        auto track = new otio::Track(name, std::nullopt, otio::Track::Kind::audio);
        otio::ErrorStatus err;
        timeline->tracks()->append_child(track, &err);
        return reinterpret_cast<OtioTrack*>(track);
    } catch (...) {
        return nullptr;
    }
}

OtioTrack* otio_track_create_video(const char* name) {
    try {
        auto track = new otio::Track(name, std::nullopt, otio::Track::Kind::video);
        Retainer<otio::Track> retainer(track);
        return reinterpret_cast<OtioTrack*>(retainer.take_value());
    } catch (...) {
        return nullptr;
    }
}

OtioTrack* otio_track_create_audio(const char* name) {
    try {
        auto track = new otio::Track(name, std::nullopt, otio::Track::Kind::audio);
        Retainer<otio::Track> retainer(track);
        return reinterpret_cast<OtioTrack*>(retainer.take_value());
    } catch (...) {
        return nullptr;
    }
}

void otio_track_free(OtioTrack* track) {
    if (track) {
        try {
            auto t = reinterpret_cast<otio::Track*>(track);
            Retainer<otio::Track> retainer(t);
        } catch (...) {
            // Ignore exceptions during cleanup
        }
    }
}

OtioClip* otio_clip_create(const char* name, OtioTimeRange source_range) {
    try {
        auto clip = new otio::Clip(name, nullptr, to_otio_tr(source_range));
        Retainer<otio::Clip> retainer(clip);
        return reinterpret_cast<OtioClip*>(retainer.take_value());
    } catch (...) {
        return nullptr;
    }
}

int otio_clip_set_media_reference(OtioClip* clip, OtioExternalRef* ref, OtioError* err) {
    try {
        auto c = reinterpret_cast<otio::Clip*>(clip);
        auto r = reinterpret_cast<otio::ExternalReference*>(ref);
        c->set_media_reference(r);
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_track_append_clip(OtioTrack* track, OtioClip* clip, OtioError* err) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        auto c = reinterpret_cast<otio::Clip*>(clip);
        otio::ErrorStatus status;
        t->append_child(c, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

OtioGap* otio_gap_create(OtioRationalTime duration) {
    try {
        auto gap = new otio::Gap(otio::TimeRange(
            otio::RationalTime(0, duration.rate),
            to_otio_rt(duration)
        ));
        Retainer<otio::Gap> retainer(gap);
        return reinterpret_cast<OtioGap*>(retainer.take_value());
    } catch (...) {
        return nullptr;
    }
}

int otio_track_append_gap(OtioTrack* track, OtioGap* gap, OtioError* err) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        auto g = reinterpret_cast<otio::Gap*>(gap);
        otio::ErrorStatus status;
        t->append_child(g, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

OtioExternalRef* otio_external_ref_create(const char* target_url) {
    try {
        auto ref = new otio::ExternalReference(target_url);
        Retainer<otio::ExternalReference> retainer(ref);
        return reinterpret_cast<OtioExternalRef*>(retainer.take_value());
    } catch (...) {
        return nullptr;
    }
}

int otio_external_ref_set_available_range(OtioExternalRef* ref, OtioTimeRange range, OtioError* err) {
    try {
        auto r = reinterpret_cast<otio::ExternalReference*>(ref);
        r->set_available_range(to_otio_tr(range));
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

void otio_external_ref_free(OtioExternalRef* ref) {
    // References are typically owned by clips
}

int otio_timeline_write_to_file(OtioTimeline* tl, const char* path, OtioError* err) {
    try {
        auto timeline = reinterpret_cast<otio::Timeline*>(tl);
        otio::ErrorStatus status;
        bool success = timeline->to_json_file(path, &status);
        if (!success || otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

OtioTimeline* otio_timeline_read_from_file(const char* path, OtioError* err) {
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

} // end extern "C" for basic functions

// Helper for getting metadata string - returns malloc'd string (caller must free)
template<typename T>
static char* get_metadata_string_impl(T* obj, const char* key) {
    try {
        auto& meta = obj->metadata();
        auto it = meta.find(std::string(key));
        if (it != meta.end()) {
            if (it->second.type() == typeid(std::string)) {
                const std::string& value = std::any_cast<const std::string&>(it->second);
                return strdup(value.c_str());
            }
        }
        return nullptr;
    } catch (...) {
        return nullptr;
    }
}

template<typename T>
static void set_metadata_string_impl(T* obj, const char* key, const char* value) {
    try {
        obj->metadata()[std::string(key)] = std::string(value);
    } catch (...) {
        // Ignore
    }
}

extern "C" {

// String memory management
void otio_free_string(char* str) {
    free(str);
}

// Timeline metadata
void otio_timeline_set_metadata_string(OtioTimeline* tl, const char* key, const char* value) {
    auto timeline = reinterpret_cast<otio::Timeline*>(tl);
    set_metadata_string_impl(timeline, key, value);
}

char* otio_timeline_get_metadata_string(OtioTimeline* tl, const char* key) {
    auto timeline = reinterpret_cast<otio::Timeline*>(tl);
    return get_metadata_string_impl(timeline, key);
}

// Track metadata
void otio_track_set_metadata_string(OtioTrack* track, const char* key, const char* value) {
    auto t = reinterpret_cast<otio::Track*>(track);
    set_metadata_string_impl(t, key, value);
}

char* otio_track_get_metadata_string(OtioTrack* track, const char* key) {
    auto t = reinterpret_cast<otio::Track*>(track);
    return get_metadata_string_impl(t, key);
}

// Clip metadata
void otio_clip_set_metadata_string(OtioClip* clip, const char* key, const char* value) {
    auto c = reinterpret_cast<otio::Clip*>(clip);
    set_metadata_string_impl(c, key, value);
}

char* otio_clip_get_metadata_string(OtioClip* clip, const char* key) {
    auto c = reinterpret_cast<otio::Clip*>(clip);
    return get_metadata_string_impl(c, key);
}

// Gap metadata
void otio_gap_set_metadata_string(OtioGap* gap, const char* key, const char* value) {
    auto g = reinterpret_cast<otio::Gap*>(gap);
    set_metadata_string_impl(g, key, value);
}

char* otio_gap_get_metadata_string(OtioGap* gap, const char* key) {
    auto g = reinterpret_cast<otio::Gap*>(gap);
    return get_metadata_string_impl(g, key);
}

// Stack metadata
void otio_stack_set_metadata_string(OtioStack* stack, const char* key, const char* value) {
    auto s = reinterpret_cast<otio::Stack*>(stack);
    set_metadata_string_impl(s, key, value);
}

char* otio_stack_get_metadata_string(OtioStack* stack, const char* key) {
    auto s = reinterpret_cast<otio::Stack*>(stack);
    return get_metadata_string_impl(s, key);
}

// ExternalReference metadata
void otio_external_ref_set_metadata_string(OtioExternalRef* ref, const char* key, const char* value) {
    auto r = reinterpret_cast<otio::ExternalReference*>(ref);
    set_metadata_string_impl(r, key, value);
}

char* otio_external_ref_get_metadata_string(OtioExternalRef* ref, const char* key) {
    auto r = reinterpret_cast<otio::ExternalReference*>(ref);
    return get_metadata_string_impl(r, key);
}

// Stack functions
OtioStack* otio_stack_create(const char* name) {
    try {
        auto stack = new otio::Stack(name);
        Retainer<otio::Stack> retainer(stack);
        return reinterpret_cast<OtioStack*>(retainer.take_value());
    } catch (...) {
        return nullptr;
    }
}

void otio_stack_free(OtioStack* stack) {
    if (stack) {
        try {
            auto s = reinterpret_cast<otio::Stack*>(stack);
            Retainer<otio::Stack> retainer(s);
        } catch (...) {
            // Ignore exceptions during cleanup
        }
    }
}

int otio_stack_append_track(OtioStack* stack, OtioTrack* track, OtioError* err) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto t = reinterpret_cast<otio::Track*>(track);
        otio::ErrorStatus status;
        s->append_child(t, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_stack_append_clip(OtioStack* stack, OtioClip* clip, OtioError* err) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto c = reinterpret_cast<otio::Clip*>(clip);
        otio::ErrorStatus status;
        s->append_child(c, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_stack_append_gap(OtioStack* stack, OtioGap* gap, OtioError* err) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto g = reinterpret_cast<otio::Gap*>(gap);
        otio::ErrorStatus status;
        s->append_child(g, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_stack_append_stack(OtioStack* stack, OtioStack* child, OtioError* err) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto c = reinterpret_cast<otio::Stack*>(child);
        otio::ErrorStatus status;
        s->append_child(c, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

OtioStack* otio_timeline_get_tracks(OtioTimeline* tl) {
    try {
        auto timeline = reinterpret_cast<otio::Timeline*>(tl);
        return reinterpret_cast<OtioStack*>(timeline->tracks());
    } catch (...) {
        return nullptr;
    }
}

int otio_track_append_stack(OtioTrack* track, OtioStack* stack, OtioError* err) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        auto s = reinterpret_cast<otio::Stack*>(stack);
        otio::ErrorStatus status;
        t->append_child(s, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

// Track iteration
int32_t otio_track_children_count(OtioTrack* track) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        return static_cast<int32_t>(t->children().size());
    } catch (...) {
        return 0;
    }
}

int32_t otio_track_child_type(OtioTrack* track, int32_t index) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        auto& children = t->children();
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

void* otio_track_child_at(OtioTrack* track, int32_t index) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        auto& children = t->children();
        if (index < 0 || static_cast<size_t>(index) >= children.size()) {
            return nullptr;
        }
        return children[index].value;
    } catch (...) {
        return nullptr;
    }
}

// Stack iteration
int32_t otio_stack_children_count(OtioStack* stack) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        return static_cast<int32_t>(s->children().size());
    } catch (...) {
        return 0;
    }
}

int32_t otio_stack_child_type(OtioStack* stack, int32_t index) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto& children = s->children();
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

void* otio_stack_child_at(OtioStack* stack, int32_t index) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto& children = s->children();
        if (index < 0 || static_cast<size_t>(index) >= children.size()) {
            return nullptr;
        }
        return children[index].value;
    } catch (...) {
        return nullptr;
    }
}

// Name accessors
char* otio_clip_get_name(OtioClip* clip) {
    try {
        auto c = reinterpret_cast<otio::Clip*>(clip);
        return strdup(c->name().c_str());
    } catch (...) {
        return nullptr;
    }
}

char* otio_gap_get_name(OtioGap* gap) {
    try {
        auto g = reinterpret_cast<otio::Gap*>(gap);
        return strdup(g->name().c_str());
    } catch (...) {
        return nullptr;
    }
}

char* otio_track_get_name(OtioTrack* track) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        return strdup(t->name().c_str());
    } catch (...) {
        return nullptr;
    }
}

char* otio_stack_get_name(OtioStack* stack) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        return strdup(s->name().c_str());
    } catch (...) {
        return nullptr;
    }
}

// Source range accessor
OtioTimeRange otio_clip_get_source_range(OtioClip* clip) {
    try {
        auto c = reinterpret_cast<otio::Clip*>(clip);
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
    // Return zero range on error
    return OtioTimeRange{OtioRationalTime{0, 1}, OtioRationalTime{0, 1}};
}

// Track modification operations
int otio_track_remove_child(OtioTrack* track, int32_t index, OtioError* err) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        auto& children = t->children();
        if (index < 0 || static_cast<size_t>(index) >= children.size()) {
            set_error(err, 1, "Index out of bounds");
            return -1;
        }
        otio::ErrorStatus status;
        t->remove_child(index, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_track_insert_clip(OtioTrack* track, int32_t index, OtioClip* clip, OtioError* err) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        auto c = reinterpret_cast<otio::Clip*>(clip);
        otio::ErrorStatus status;
        t->insert_child(index, c, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_track_insert_gap(OtioTrack* track, int32_t index, OtioGap* gap, OtioError* err) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        auto g = reinterpret_cast<otio::Gap*>(gap);
        otio::ErrorStatus status;
        t->insert_child(index, g, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_track_insert_stack(OtioTrack* track, int32_t index, OtioStack* stack, OtioError* err) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        auto s = reinterpret_cast<otio::Stack*>(stack);
        otio::ErrorStatus status;
        t->insert_child(index, s, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_track_clear_children(OtioTrack* track, OtioError* err) {
    try {
        auto t = reinterpret_cast<otio::Track*>(track);
        t->clear_children();
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

// Stack modification operations
int otio_stack_remove_child(OtioStack* stack, int32_t index, OtioError* err) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto& children = s->children();
        if (index < 0 || static_cast<size_t>(index) >= children.size()) {
            set_error(err, 1, "Index out of bounds");
            return -1;
        }
        otio::ErrorStatus status;
        s->remove_child(index, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_stack_insert_track(OtioStack* stack, int32_t index, OtioTrack* track, OtioError* err) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto t = reinterpret_cast<otio::Track*>(track);
        otio::ErrorStatus status;
        s->insert_child(index, t, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_stack_insert_clip(OtioStack* stack, int32_t index, OtioClip* clip, OtioError* err) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto c = reinterpret_cast<otio::Clip*>(clip);
        otio::ErrorStatus status;
        s->insert_child(index, c, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_stack_insert_gap(OtioStack* stack, int32_t index, OtioGap* gap, OtioError* err) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto g = reinterpret_cast<otio::Gap*>(gap);
        otio::ErrorStatus status;
        s->insert_child(index, g, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_stack_insert_stack(OtioStack* stack, int32_t index, OtioStack* child, OtioError* err) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        auto c = reinterpret_cast<otio::Stack*>(child);
        otio::ErrorStatus status;
        s->insert_child(index, c, &status);
        if (otio::is_error(status)) {
            set_error(err, 1, status.full_description.c_str());
            return -1;
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

int otio_stack_clear_children(OtioStack* stack, OtioError* err) {
    try {
        auto s = reinterpret_cast<otio::Stack*>(stack);
        s->clear_children();
        return 0;
    } catch (const std::exception& e) {
        set_error(err, 1, e.what());
        return -1;
    } catch (...) {
        set_error(err, 1, "Unknown exception");
        return -1;
    }
}

}


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

void otio_timeline_set_global_start_time(OtioTimeline* tl, OtioRationalTime time) {
    try {
        auto timeline = reinterpret_cast<otio::Timeline*>(tl);
        timeline->set_global_start_time(to_otio_rt(time));
    } catch (...) {
        // Ignore
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

void otio_clip_set_media_reference(OtioClip* clip, OtioExternalRef* ref) {
    try {
        auto c = reinterpret_cast<otio::Clip*>(clip);
        auto r = reinterpret_cast<otio::ExternalReference*>(ref);
        c->set_media_reference(r);
    } catch (...) {
        // Ignore
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

void otio_external_ref_set_available_range(OtioExternalRef* ref, OtioTimeRange range) {
    try {
        auto r = reinterpret_cast<otio::ExternalReference*>(ref);
        r->set_available_range(to_otio_tr(range));
    } catch (...) {
        // Ignore
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

void otio_clip_set_metadata_string(OtioClip* clip, const char* key, const char* value) {
    try {
        auto c = reinterpret_cast<otio::Clip*>(clip);
        c->metadata()[std::string(key)] = std::string(value);
    } catch (...) {
        // Ignore
    }
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

}


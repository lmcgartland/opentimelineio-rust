#ifndef OTIO_SHIM_H
#define OTIO_SHIM_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handles
typedef struct OtioTimeline OtioTimeline;
typedef struct OtioTrack OtioTrack;
typedef struct OtioClip OtioClip;
typedef struct OtioGap OtioGap;
typedef struct OtioExternalRef OtioExternalRef;
typedef struct OtioStack OtioStack;

// Error handling
typedef struct {
    int code;
    char message[256];
} OtioError;

// RationalTime (passed by value)
typedef struct {
    double value;
    double rate;
} OtioRationalTime;

typedef struct {
    OtioRationalTime start_time;
    OtioRationalTime duration;
} OtioTimeRange;

// Timeline
OtioTimeline* otio_timeline_create(const char* name);
void otio_timeline_free(OtioTimeline* tl);
void otio_timeline_set_global_start_time(OtioTimeline* tl, OtioRationalTime time);

// Tracks (0 = video, 1 = audio)
OtioTrack* otio_timeline_add_video_track(OtioTimeline* tl, const char* name);
OtioTrack* otio_timeline_add_audio_track(OtioTimeline* tl, const char* name);

// Standalone track creation (for use with Stack)
OtioTrack* otio_track_create_video(const char* name);
OtioTrack* otio_track_create_audio(const char* name);
void otio_track_free(OtioTrack* track);

// Clips
OtioClip* otio_clip_create(const char* name, OtioTimeRange source_range);
void otio_clip_set_media_reference(OtioClip* clip, OtioExternalRef* ref);
int otio_track_append_clip(OtioTrack* track, OtioClip* clip, OtioError* err);

// Gaps
OtioGap* otio_gap_create(OtioRationalTime duration);
int otio_track_append_gap(OtioTrack* track, OtioGap* gap, OtioError* err);

// Media references
OtioExternalRef* otio_external_ref_create(const char* target_url);
void otio_external_ref_set_available_range(OtioExternalRef* ref, OtioTimeRange range);
void otio_external_ref_free(OtioExternalRef* ref);

// Serialization
int otio_timeline_write_to_file(OtioTimeline* tl, const char* path, OtioError* err);
OtioTimeline* otio_timeline_read_from_file(const char* path, OtioError* err);

// Metadata (string key-value for simplicity)
void otio_clip_set_metadata_string(OtioClip* clip, const char* key, const char* value);

// Stack (composition for nested structures)
OtioStack* otio_stack_create(const char* name);
void otio_stack_free(OtioStack* stack);
int otio_stack_append_track(OtioStack* stack, OtioTrack* track, OtioError* err);
int otio_stack_append_clip(OtioStack* stack, OtioClip* clip, OtioError* err);
int otio_stack_append_gap(OtioStack* stack, OtioGap* gap, OtioError* err);
int otio_stack_append_stack(OtioStack* stack, OtioStack* child, OtioError* err);

// Timeline stack accessor
OtioStack* otio_timeline_get_tracks(OtioTimeline* tl);

// Track can also contain stacks (for versioning/alternatives)
int otio_track_append_stack(OtioTrack* track, OtioStack* stack, OtioError* err);

#ifdef __cplusplus
}
#endif

#endif

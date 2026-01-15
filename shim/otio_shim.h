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
int otio_timeline_set_global_start_time(OtioTimeline* tl, OtioRationalTime time, OtioError* err);

// Tracks (0 = video, 1 = audio)
OtioTrack* otio_timeline_add_video_track(OtioTimeline* tl, const char* name);
OtioTrack* otio_timeline_add_audio_track(OtioTimeline* tl, const char* name);

// Standalone track creation (for use with Stack)
OtioTrack* otio_track_create_video(const char* name);
OtioTrack* otio_track_create_audio(const char* name);
void otio_track_free(OtioTrack* track);

// Clips
OtioClip* otio_clip_create(const char* name, OtioTimeRange source_range);
int otio_clip_set_media_reference(OtioClip* clip, OtioExternalRef* ref, OtioError* err);
int otio_track_append_clip(OtioTrack* track, OtioClip* clip, OtioError* err);

// Gaps
OtioGap* otio_gap_create(OtioRationalTime duration);
int otio_track_append_gap(OtioTrack* track, OtioGap* gap, OtioError* err);

// Media references
OtioExternalRef* otio_external_ref_create(const char* target_url);
int otio_external_ref_set_available_range(OtioExternalRef* ref, OtioTimeRange range, OtioError* err);
void otio_external_ref_free(OtioExternalRef* ref);

// Serialization
int otio_timeline_write_to_file(OtioTimeline* tl, const char* path, OtioError* err);
OtioTimeline* otio_timeline_read_from_file(const char* path, OtioError* err);

// Metadata (string key-value pairs)
// Getter returns malloc'd string - caller must free with otio_free_string
// Returns NULL if key not found

// String memory management
void otio_free_string(char* str);

// Timeline metadata
void otio_timeline_set_metadata_string(OtioTimeline* tl, const char* key, const char* value);
char* otio_timeline_get_metadata_string(OtioTimeline* tl, const char* key);

// Track metadata
void otio_track_set_metadata_string(OtioTrack* track, const char* key, const char* value);
char* otio_track_get_metadata_string(OtioTrack* track, const char* key);

// Clip metadata
void otio_clip_set_metadata_string(OtioClip* clip, const char* key, const char* value);
char* otio_clip_get_metadata_string(OtioClip* clip, const char* key);

// Gap metadata
void otio_gap_set_metadata_string(OtioGap* gap, const char* key, const char* value);
char* otio_gap_get_metadata_string(OtioGap* gap, const char* key);

// Stack metadata
void otio_stack_set_metadata_string(OtioStack* stack, const char* key, const char* value);
char* otio_stack_get_metadata_string(OtioStack* stack, const char* key);

// ExternalReference metadata
void otio_external_ref_set_metadata_string(OtioExternalRef* ref, const char* key, const char* value);
char* otio_external_ref_get_metadata_string(OtioExternalRef* ref, const char* key);

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

// Child type enumeration (returned by child_type functions)
// 0 = Clip, 1 = Gap, 2 = Stack, 3 = Track, -1 = Unknown/Error
#define OTIO_CHILD_TYPE_CLIP  0
#define OTIO_CHILD_TYPE_GAP   1
#define OTIO_CHILD_TYPE_STACK 2
#define OTIO_CHILD_TYPE_TRACK 3

// Track iteration
int32_t otio_track_children_count(OtioTrack* track);
int32_t otio_track_child_type(OtioTrack* track, int32_t index);
void* otio_track_child_at(OtioTrack* track, int32_t index);

// Stack iteration
int32_t otio_stack_children_count(OtioStack* stack);
int32_t otio_stack_child_type(OtioStack* stack, int32_t index);
void* otio_stack_child_at(OtioStack* stack, int32_t index);

// Name accessors (returns malloc'd string - caller must free with otio_free_string)
char* otio_clip_get_name(OtioClip* clip);
char* otio_gap_get_name(OtioGap* gap);
char* otio_track_get_name(OtioTrack* track);
char* otio_stack_get_name(OtioStack* stack);

// Source range accessor
OtioTimeRange otio_clip_get_source_range(OtioClip* clip);

// Track modification operations
int otio_track_remove_child(OtioTrack* track, int32_t index, OtioError* err);
int otio_track_insert_clip(OtioTrack* track, int32_t index, OtioClip* clip, OtioError* err);
int otio_track_insert_gap(OtioTrack* track, int32_t index, OtioGap* gap, OtioError* err);
int otio_track_insert_stack(OtioTrack* track, int32_t index, OtioStack* stack, OtioError* err);
int otio_track_clear_children(OtioTrack* track, OtioError* err);

// Stack modification operations
int otio_stack_remove_child(OtioStack* stack, int32_t index, OtioError* err);
int otio_stack_insert_track(OtioStack* stack, int32_t index, OtioTrack* track, OtioError* err);
int otio_stack_insert_clip(OtioStack* stack, int32_t index, OtioClip* clip, OtioError* err);
int otio_stack_insert_gap(OtioStack* stack, int32_t index, OtioGap* gap, OtioError* err);
int otio_stack_insert_stack(OtioStack* stack, int32_t index, OtioStack* child, OtioError* err);
int otio_stack_clear_children(OtioStack* stack, OtioError* err);

#ifdef __cplusplus
}
#endif

#endif

#include <deadbeef/deadbeef.h>

typedef struct {
  char artist[256];
  char title[256];
  char album[256];

  float current_position;
  float total_duration;
  float percentage;

  ddb_playback_state_t state;

  int bitrate;
  int samplerate;
  int channels;

  int is_valid;
} playback_info_t;

static const char *APPLICATION_ID = "1440255782418387026";
static playback_info_t g_playback_info = {0};
static DB_functions_t *deadbeef;
static intptr_t timer_id = 0;
static volatile int timer_running = 0;
static uintptr_t mutex = 0;

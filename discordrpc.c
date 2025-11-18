#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <threads.h>
#include <unistd.h>

#include "discordrpc.h"
#include <discord_rpc.h>

static void discordInit() {
  DiscordEventHandlers handlers;

  memset(&handlers, 0, sizeof(handlers));

  Discord_Initialize(APPLICATION_ID, &handlers, 1, NULL);
}

static void update_playback_info(playback_info_t *info) {
  if (!info)
    return;

  memset(info, 0, sizeof(playback_info_t));

  DB_playItem_t *track = deadbeef->streamer_get_playing_track_safe();
  if (!track) {
    info->is_valid = 0;
    return;
  }

  info->is_valid = 1;

  DB_output_t *output = deadbeef->get_output();
  if (output) {
    info->state = output->state();
  }

  deadbeef->pl_lock();

  ddb_playlist_t *pl = deadbeef->plt_get_curr();
  const char *artist = deadbeef->pl_find_meta(track, "artist");
  const char *title = deadbeef->pl_find_meta(track, "title");
  const char *album = deadbeef->pl_find_meta(track, "album");
  const char *uri = deadbeef->pl_find_meta(track, ":URI");

  if (title) {
    strncpy(info->title, title, sizeof(info->title) - 1);
  } else if (uri) {
    const char *slash = strrchr(uri, '/');
    const char *title = slash ? slash + 1 : uri;

    strncpy(info->title, title, sizeof(info->title) - 1);
  } else {
    strcpy(info->title, "Unknown Title");
  }

  if (artist) {
    strncpy(info->artist, artist, sizeof(info->artist) - 1);
  } else {
    strcpy(info->artist, "Unknown Artist");
  }

  if (album) {
    strncpy(info->album, album, sizeof(info->album) - 1);
  } else {
    strcpy(info->album, "Unknown Album");
  }

  deadbeef->pl_unlock();

  info->current_position = deadbeef->streamer_get_playpos();
  info->total_duration = deadbeef->pl_get_item_duration(track);

  if (info->total_duration > 0) {
    info->percentage = (info->current_position / info->total_duration) * 100.0f;
  } else {
    info->percentage = 0.0f;
  }

  DB_fileinfo_t *fileinfo = deadbeef->streamer_get_current_fileinfo();
  if (fileinfo) {
    info->samplerate = fileinfo->fmt.samplerate;
    info->channels = fileinfo->fmt.channels;
  }

  info->bitrate = deadbeef->streamer_get_apx_bitrate();

  deadbeef->pl_item_unref(track);
}

static void update_discord_presence(const playback_info_t *info) {
  if (!info || !info->is_valid) {
    Discord_ClearPresence();
    return;
  }

  static char details_buffer[256];
  static char state_buffer[256];
  static char large_image_text[128];
  static char small_image_text[128];

  DiscordRichPresence discordPresence;
  memset(&discordPresence, 0, sizeof(discordPresence));

  snprintf(details_buffer, sizeof(details_buffer), "🎵 %s", info->title);
  discordPresence.details = details_buffer;

  snprintf(state_buffer, sizeof(state_buffer), "👤 %s • 💿 %s", info->artist,
           info->album);
  discordPresence.state = state_buffer;

  snprintf(large_image_text, sizeof(large_image_text),
           "%d kbps • %d Hz • %d ch", info->bitrate, info->samplerate,
           info->channels);
  discordPresence.largeImageText = large_image_text;

  switch (info->state) {
  case DDB_PLAYBACK_STATE_PLAYING:
    discordPresence.largeImageKey = "deadbeef-logo";
    discordPresence.smallImageKey = "play-icon";
    snprintf(small_image_text, sizeof(small_image_text), "Playing (%.1f%%)",
             info->percentage);
    discordPresence.smallImageText = small_image_text;

    time_t now = time(0);
    discordPresence.startTimestamp = now - (time_t)info->current_position;
    discordPresence.endTimestamp =
        now + (time_t)(info->total_duration - info->current_position);
    break;

  case DDB_PLAYBACK_STATE_PAUSED:
    discordPresence.largeImageKey = "deadbeef-logo";
    discordPresence.smallImageKey = "pause-icon";
    snprintf(small_image_text, sizeof(small_image_text), "Paused at %.1f%%",
             info->percentage);
    discordPresence.smallImageText = small_image_text;
    break;

  case DDB_PLAYBACK_STATE_STOPPED:
    discordPresence.largeImageKey = "deadbeef-logo";
    discordPresence.smallImageKey = "stop-icon";
    discordPresence.smallImageText = "Stopped";
    break;
  }

  discordPresence.instance = 0;

  Discord_UpdatePresence(&discordPresence);
}

static int message_handler(uint32_t id, uintptr_t ctx, uint32_t p1,
                           uint32_t p2) {

  switch (id) {
  case DB_EV_SONGSTARTED:
  case DB_EV_SONGCHANGED:
  case DB_EV_SONGFINISHED:
  case DB_EV_PAUSED:
  case DB_EV_SEEKED:
  case DB_EV_TRACKINFOCHANGED:
  case DB_EV_CONFIGCHANGED:
    update_playback_info(&g_playback_info);
    update_discord_presence(&g_playback_info);
  }

  return 0;
}

static void timer_thread(void *ctx) {
  static const struct timespec ts = {.tv_sec = 15, .tv_nsec = 0};

  while (timer_running) {
    thrd_sleep(&ts, NULL);

    if (!timer_running)
      break;

    deadbeef->mutex_lock(mutex);

    DB_output_t *output = deadbeef->get_output();
    if (output && output->state() == DDB_PLAYBACK_STATE_PLAYING) {
      update_playback_info(&g_playback_info);
      update_discord_presence(&g_playback_info);
    }

    deadbeef->mutex_unlock(mutex);
  }
}

int discordrpc_start() {
  printf("Discord RPC Timer Tick\n");
  discordInit();

  timer_running = 1;
  mutex = deadbeef->mutex_create();
  timer_id = deadbeef->thread_start(timer_thread, NULL);

  return 0;
}

int discordrpc_stop() {
  timer_running = 0;
  deadbeef->thread_join(timer_id);
  deadbeef->mutex_free(mutex);

  mutex = 0;
  timer_id = 0;

  return 0;
}

static DB_misc_t plugin = {
    .plugin.api_vmajor = 1,
    .plugin.api_vminor = 0,

    .plugin.version_major = 0,
    .plugin.version_minor = 1,

    .plugin.flags = DDB_PLUGIN_FLAG_IMPLEMENTS_DECODER2,
    .plugin.type = DB_PLUGIN_MISC,

    .plugin.id = "discordrpc",
    .plugin.name = "Discord Rich Presence",
    .plugin.descr = "Updates Discord Rich Presence with the current track info "
                    "from DeadBeef.",
    .plugin.copyright = "2025",

    .plugin.start = discordrpc_start,
    .plugin.stop = discordrpc_stop,
    .plugin.message = message_handler,
};

DB_plugin_t *discordrpc_load(DB_functions_t *api) {
  deadbeef = api;
  return DB_PLUGIN(&plugin);
}

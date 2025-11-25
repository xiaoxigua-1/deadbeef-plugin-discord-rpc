use std::time::SystemTime;

use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, ActivityType, Assets, Timestamps},
};

use crate::{
    API, DRPC,
    config::{ConfigDefault, ConfigKey, CoverSource},
    deadbeef::ddb_playback_state_e_DDB_PLAYBACK_STATE_PLAYING,
    error::{Error, Result},
    musicbrainz::get_album_cover_url_from_query,
    util::{is_streaming, nowplaying_format_string, nowplaying_length},
};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Paused = 1,
    Songchanged = 2,
    Seeked = 3,
    Start = 4,
}

pub fn clear_activity() -> Result<()> {
    let mut drpc = DRPC.lock().unwrap();

    if let Some(drpc) = &mut *drpc {
        drpc.clear_activity().map_err(Error::DiscordFailed)?;
    }

    Ok(())
}

pub fn update_activity(playback_status: Status, nextitem_length: Option<f32>) -> Result<()> {
    let mut playback_status = playback_status;
    let mut drpc = DRPC.lock().unwrap();
    let api = API.get().unwrap();

    let details_script = api.conf_get_str(ConfigKey::TITLE_SCRIPT, ConfigDefault::TITLE_SCRIPT)?;
    let state_script = api.conf_get_str(ConfigKey::STATE_SCRIPT, ConfigDefault::STATE_SCRIPT)?;
    let icon_text_script = api.conf_get_str(ConfigKey::ICON_SCRIPT, ConfigDefault::ICON_SCRIPT)?;
    let timestamp_display_mode =
        api.conf_get_int(ConfigKey::END_TIMESTAMP2, ConfigDefault::END_TIMESTAMP2)?;
    let cover_source = CoverSource::try_from(
        api.conf_get_int(ConfigKey::COVER_SOURCE, ConfigDefault::COVER_SOURCE)?,
    )?;

    let details = nowplaying_format_string(&details_script)?;
    let state = nowplaying_format_string(&state_script)?;
    let icon_text = nowplaying_format_string(&icon_text_script)?;
    let mut start_timestamp: i64 = 0;
    let mut end_timestamp: i64 = 0;

    if let Status::Seeked = playback_status
        && !api.get_output()?.is_null()
        && unsafe { &*api.get_output()? }.state()?
            != ddb_playback_state_e_DDB_PLAYBACK_STATE_PLAYING
    {
        playback_status = Status::Paused;

        // TODO: Hide on paused
    }

    match playback_status {
        Status::Paused => {
            start_timestamp = 0;
            end_timestamp = 0;
        }
        Status::Songchanged | Status::Seeked | Status::Start if timestamp_display_mode != 2 => {
            start_timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(Error::SystemTimeError)?
                .as_secs() as i64;

            if playback_status != Status::Songchanged {
                start_timestamp -= (nowplaying_length()? * api.playback_get_pos()? / 100.0) as i64;
            }

            if timestamp_display_mode == 1 && !is_streaming()? {
                if let Status::Songchanged = playback_status
                    && let Some(length) = nextitem_length
                {
                    end_timestamp = start_timestamp + length as i64;
                } else {
                    end_timestamp = start_timestamp + nowplaying_length()? as i64;
                }
            }
        }
        _ => {}
    }

    let large_image = match cover_source {
        CoverSource::MusicBrainz => {
            let album_query_script = api.conf_get_str(
                ConfigKey::QUERY_ALBUM_SCRIPT,
                ConfigDefault::QUERY_ALBUM_SCRIPT,
            )?;
            let album_query = nowplaying_format_string(&album_query_script)?;

            get_album_cover_url_from_query(&album_query).unwrap_or("default".to_string())
        }
        CoverSource::NoCover => "default".to_string(),
    };

    start_timestamp *= 1000;
    end_timestamp *= 1000;

    if let Some(drpc) = &mut *drpc {
        drpc.set_activity(
            Activity::new()
                .details(&details)
                .timestamps(Timestamps::new().start(start_timestamp).end(end_timestamp))
                .assets(
                    Assets::new()
                        .large_text(&icon_text)
                        .large_image(&large_image),
                )
                .state(&state)
                .activity_type(ActivityType::Listening),
        )
        .map_err(Error::DiscordFailed)?;
    } else {
        return Err(Error::MissingFunction);
    }

    Ok(())
}

pub fn create_discord_client() -> Result<DiscordIpcClient> {
    let client_id = API
        .get()
        .unwrap()
        .conf_get_str(ConfigKey::CLIENT_ID, ConfigDefault::CLIENT_ID)?;

    Ok(DiscordIpcClient::new(&client_id))
}

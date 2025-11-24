use std::time::SystemTime;

use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, ActivityType, Assets, Timestamps},
};

use crate::{
    API, DRPC,
    deadbeef::ddb_playback_state_e_DDB_PLAYBACK_STATE_PLAYING,
    error::{Error, Result},
    util::{is_streaming, nowplaying_format_string, nowplaying_length},
};

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Status {
    Paused = 1,
    Songchanged = 2,
    Seeked = 3,
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

    let details_script = api.conf_get_str(
        c"discordrpc.title_script".as_ptr(),
        c"%title%$if(%ispaused%,' ('paused')')".as_ptr(),
    )?;
    let state_script =
        api.conf_get_str(c"discordrpc.state_script".as_ptr(), c"%artist%".as_ptr())?;
    let icon_text_script =
        api.conf_get_str(c"discordrpc.icon_script".as_ptr(), c"%album%".as_ptr())?;
    let timestamp_display_mode = api.conf_get_int(c"discordrpc.end_timestamp2".as_ptr(), 1)?;

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
        Status::Songchanged | Status::Seeked if timestamp_display_mode != 2 => {
            start_timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(Error::SystemTimeError)?
                .as_secs() as i64;

            if let Status::Seeked = playback_status {
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

    if let Some(drpc) = &mut *drpc {
        drpc.set_activity(
            Activity::new()
                .details(&details)
                .timestamps(Timestamps::new().start(start_timestamp).end(end_timestamp))
                .assets(Assets::new().large_text(&icon_text))
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
    let client_id = API.get().unwrap().conf_get_str(
        c"discordrpc.client_id".as_ptr(),
        c"1440255782418387026".as_ptr(),
    )?;

    Ok(DiscordIpcClient::new(&client_id))
}

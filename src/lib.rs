mod config;
mod deadbeef;
mod discordrpc;
mod error;
mod musicbrainz;
mod util;

use std::{
    ffi::c_void,
    mem,
    ptr::{self, null_mut},
    sync::{Arc, Mutex},
};

use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;

use crate::{
    config::*,
    deadbeef::{
        DB_EV_CONFIGCHANGED, DB_EV_PAUSED, DB_EV_SEEKED, DB_EV_SONGCHANGED, DB_EV_STOP,
        DB_functions_t, DB_misc_t, DB_plugin_t, ddb_event_trackchange_t,
        ddb_playback_state_e_DDB_PLAYBACK_STATE_STOPPED, safe_wrapper::SafeDBPlayItem,
    },
    discordrpc::{Status, clear_activity, create_discord_client, update_activity},
    error::{Error, Result},
};

static API: OnceCell<&DB_functions_t> = OnceCell::new();
lazy_static! {
    static ref DRPC: Mutex<Option<DiscordIpcClient>> = Mutex::new(None);
    static ref DISCORD_CLIENT_ID: Mutex<Option<String>> = Mutex::new(None);
}

fn config_update() -> Result<()> {
    let mut drpc = DRPC.lock().unwrap();
    let api = API.get().unwrap();
    let enable = api.conf_get_int(ConfigKey::ENABLE, ConfigDefault::ENABLE)?;
    let client_id = api.conf_get_str(ConfigKey::CLIENT_ID, ConfigDefault::CLIENT_ID)?;
    let data = Arc::new(UpdateThreadData {
        status: Status::Seeked,
        nextitem_length: None,
    });

    if let Some(id) = DISCORD_CLIENT_ID.lock().unwrap().as_ref()
        && (id != &client_id || enable == 0)
        && let Some(mut client) = drpc.take()
    {
        api.trace(format!(
            "Disconnecting from Discord RPC (client ID changed from {} to {}).",
            id, client_id
        ));
        client.close().map_err(Error::DiscordFailed)?;
    }

    if drpc.is_none()
        && let Ok(mut client) = create_discord_client()
        && enable == 1
    {
        api.trace(format!(
            "Connecting to Discord RPC with client ID {}.",
            client_id
        ));
        client.connect().unwrap();
        *DISCORD_CLIENT_ID.lock().unwrap() = Some(client_id);
        *drpc = Some(client);
    }

    if enable == 1
        && let Some(output) = unsafe { api.get_output()?.as_ref() }
        && output.state()? != ddb_playback_state_e_DDB_PLAYBACK_STATE_STOPPED
    {
        api.thread_start(create_update_thread, Arc::into_raw(data) as *mut c_void)?;
    }

    Ok(())
}

#[repr(C)]
struct UpdateThreadData {
    status: Status,
    nextitem_length: Option<f32>,
}

#[unsafe(no_mangle)]
extern "C" fn create_update_thread(ptr: *mut c_void) {
    let api = API.get().unwrap();
    let data = unsafe { Arc::from_raw(ptr as *mut UpdateThreadData) };

    api.trace(format!("Updating Discord activity: {:?}", data.status));

    if let Err(e) = update_activity(data.status, data.nextitem_length) {
        api.trace(format!("Failed to update Discord activity: {:?}", e));
    }
}

#[unsafe(no_mangle)]
extern "C" fn clear_activity_thread(_: *mut c_void) {
    let api = API.get().unwrap();

    api.trace("Clearing Discord activity from thread.".to_string());

    if let Err(e) = clear_activity() {
        api.trace(format!("Failed to clear Discord activity: {:?}", e));
    }
}

#[unsafe(no_mangle)]
extern "C" fn message(id: u32, ctx: usize, p1: u32, _: u32) -> i32 {
    let api = API.get().unwrap();
    let enable = api.conf_get_int(ConfigKey::ENABLE, ConfigDefault::ENABLE);
    let hide_on_pause = api.conf_get_int(ConfigKey::HIDE_ON_PAUSE, ConfigDefault::HIDE_ON_PAUSE);
    let ctx = unsafe { (ctx as *mut ddb_event_trackchange_t).as_ref() };

    api.trace(format!(
        "message received: id={}, ctx={:?}, p1={}",
        id, &ctx, p1
    ));

    let ret = match id {
        DB_EV_CONFIGCHANGED => {
            if let Err(e) = config_update() {
                api.trace(format!("Failed to update config: {:?}", e));
                true
            } else {
                false
            }
        }
        DB_EV_SONGCHANGED => {
            if let Ok(enable) = enable
                && enable == 1
                && let Some(ctx) = ctx
            {
                let playlist_item = SafeDBPlayItem::new(ctx.to);
                let nextitem_length = api.pl_get_item_duration(&playlist_item).ok();

                let data = Arc::new(UpdateThreadData {
                    status: Status::Songchanged,
                    nextitem_length,
                });

                mem::forget(playlist_item);
                api.thread_start(create_update_thread, Arc::into_raw(data) as *mut c_void)
                    .ok()
                    .is_some()
            } else {
                api.thread_start(clear_activity_thread, null_mut())
                    .ok()
                    .is_some()
            }
        }
        DB_EV_SEEKED => {
            if let Ok(enable) = enable
                && enable == 1
            {
                let data = Arc::new(UpdateThreadData {
                    status: Status::Seeked,
                    nextitem_length: None,
                });
                api.thread_start(create_update_thread, Arc::into_raw(data) as *mut c_void)
                    .ok()
                    .is_some()
            } else {
                true
            }
        }
        DB_EV_PAUSED if enable.is_ok() && *enable.as_ref().ok().unwrap() == 1 => {
            if let Ok(hide_on_pause) = hide_on_pause
                && !(hide_on_pause == 1 && p1 == 1)
            {
                let data = Arc::new(UpdateThreadData {
                    status: if p1 == 1 {
                        Status::Paused
                    } else {
                        Status::Start
                    },
                    nextitem_length: None,
                });
                api.thread_start(create_update_thread, Arc::into_raw(data) as *mut c_void)
                    .ok()
                    .is_some()
            } else {
                api.thread_start(clear_activity_thread, null_mut())
                    .ok()
                    .is_some()
            }
        }
        DB_EV_STOP => {
            if let Ok(enable) = enable
                && enable == 1
            {
                api.thread_start(clear_activity_thread, null_mut())
                    .ok()
                    .is_some()
            } else {
                true
            }
        }
        _ => false,
    };

    if ret { 1 } else { -1 }
}

#[unsafe(no_mangle)]
extern "C" fn stop() -> i32 {
    let mut drpc = DRPC.lock().unwrap();

    if let Some(client) = drpc.as_mut() {
        client.close().map_err(Error::DiscordFailed).ok().is_some() as i32
    } else {
        -1
    }
}

#[unsafe(no_mangle)]
extern "C" fn start() -> i32 {
    let api = API.get().unwrap();

    if let Err(e) = config_update() {
        api.trace(format!("Failed to start Discord RPC plugin: {:?}", e));
        -1
    } else {
        0
    }
}

/// # Safety
///
/// This function is `unsafe` because it dereferences a raw pointer `ptr`.
/// The caller must ensure that:
/// - `ptr` is non-null and points to a valid `DB_functions_t`.
/// - The memory pointed to by `ptr` is valid for the duration of this function.
///
/// The returned pointer is a raw pointer to a heap-allocated `DB_plugin_t`.
/// The caller is responsible for eventually converting it back to a `Box` and dropping it,
/// otherwise a memory leak will occur.
///
/// All static strings used (PLUGIN_ID, PLUGIN_NAME, etc.) must be valid for the program's lifetime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn discordrpc_load(ptr: *const DB_functions_t) -> *mut DB_plugin_t {
    unsafe {
        API.set(&*ptr).unwrap();
    }

    ptr::addr_of!(*PLUGIN.0)
        .cast::<DB_misc_t>()
        .cast::<DB_plugin_t>()
        .cast_mut()
}

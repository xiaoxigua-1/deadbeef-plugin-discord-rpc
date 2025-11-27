mod config;
mod deadbeef;
mod discordrpc;
mod error;
mod musicbrainz;
mod util;

use std::{
    ffi::c_void,
    mem,
    ptr::null_mut,
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
        safe_wrapper::SafeDBPlayItem,
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
    let api = API.get().unwrap();
    let mut drpc = DRPC.lock().unwrap();
    let enable = api.conf_get_int(ConfigKey::ENABLE, ConfigDefault::ENABLE)?;
    let client_id = api.conf_get_str(ConfigKey::CLIENT_ID, ConfigDefault::CLIENT_ID)?;

    if let Some(id) = DISCORD_CLIENT_ID.lock().unwrap().as_ref()
        && (id != &client_id || enable == 0)
        && let Some(mut client) = drpc.take()
    {
        client.close().map_err(Error::DiscordFailed)?;
    }

    if drpc.is_none()
        && let Ok(mut client) = create_discord_client()
        && enable == 1
    {
        client.connect().unwrap();
        *DISCORD_CLIENT_ID.lock().unwrap() = Some(client_id);
        *drpc = Some(client);
    }

    // TODO: Update activity on config change

    Ok(())
}

#[repr(C)]
struct UpdateThreadData {
    status: Status,
    nextitem_length: Option<f32>,
}

#[unsafe(no_mangle)]
extern "C" fn create_update_thread(ptr: *mut c_void) {
    let data = unsafe { Arc::from_raw(ptr as *mut UpdateThreadData) };

    update_activity(data.status, data.nextitem_length).ok(); // TODO: Handle error
}

#[unsafe(no_mangle)]
extern "C" fn clear_activity_thread(_: *mut c_void) {
    clear_activity().ok(); // TODO: Handle error
}

#[unsafe(no_mangle)]
extern "C" fn message(id: u32, ctx: usize, p1: u32, _: u32) -> i32 {
    let api = API.get().unwrap();
    let enable = api.conf_get_int(ConfigKey::ENABLE, ConfigDefault::ENABLE);

    api.trace(format!(
        "message received: id={}, ctx={:?}, p1={}",
        id, ctx as *mut c_void, p1
    ));
    let ret = match id {
        DB_EV_CONFIGCHANGED => config_update().ok().is_some(),
        DB_EV_SONGCHANGED => {
            let ctx = unsafe { (ctx as *mut ddb_event_trackchange_t).as_ref() };

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
        DB_EV_PAUSED => {
            if let Ok(enable) = enable
                && enable == 1
            // TODO: Hide on paused
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
                true
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

    ret as i32
}

#[unsafe(no_mangle)]
extern "C" fn stop() -> i32 {
    let mut drpc = DRPC.lock().unwrap();

    if let Some(client) = drpc.as_mut() {
        client.close().map_err(Error::DiscordFailed).ok().is_some() as i32
    } else {
        0
    }
}

#[unsafe(no_mangle)]
extern "C" fn start() -> i32 {
    config_update().ok().is_some() as i32
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

    (&*PLUGIN.0) as *const DB_misc_t as *mut DB_plugin_t
}

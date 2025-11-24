use std::{ffi::c_char, mem};

use crate::{
    API,
    deadbeef::{PL_MAIN, ddb_tf_context_t},
    error::Result,
};

static MAX_LEN: usize = 256;

pub fn nowplaying_format_string(script: &str) -> Result<String> {
    let nowplaying = API.get().unwrap().streamer_get_playing_track()?;
    let nowplaying_plt = API.get().unwrap().plt_get_curr()?;
    let code_script = API.get().unwrap().tf_compile(script)?;

    let mut context: Box<ddb_tf_context_t> = unsafe { Box::new(mem::zeroed()) };
    let mut out: Vec<u8> = vec![0; MAX_LEN];
    let out_ptr = out.as_mut_ptr() as *mut c_char;

    context._size = std::mem::size_of::<ddb_tf_context_t>() as i32;
    context.it = nowplaying;
    context.plt = nowplaying_plt;
    context.iter = PL_MAIN as i32;

    if !code_script.is_null() {
        API.get()
            .unwrap()
            .tf_eval(Box::into_raw(context), code_script, out_ptr, MAX_LEN as i32)?;
    }

    API.get().unwrap().pl_item_unref(nowplaying)?;

    if !nowplaying.is_null() {
        API.get().unwrap().plt_unref(nowplaying_plt)?;
    }

    if !code_script.is_null() {
        API.get().unwrap().tf_free(code_script)?;
    }

    Ok(
        String::from_utf8_lossy(&out[..out.iter().position(|&c| c == 0).unwrap_or(MAX_LEN)])
            .into_owned(),
    )
}

pub fn nowplaying_length() -> Result<f32> {
    let nowplaying = API.get().unwrap().streamer_get_playing_track()?;

    if !nowplaying.is_null() {
        Ok(API.get().unwrap().pl_get_item_duration(nowplaying)?)
    } else {
        Ok(0.0)
    }
}

pub fn is_streaming() -> Result<bool> {
    let api = API.get().unwrap();
    let nowplaying = api.streamer_get_playing_track()?;
    if !nowplaying.is_null() {
        api.pl_lock()?;

        let fname = api.pl_find_meta(nowplaying, c":URI".as_ptr())?;
        let result = api.is_local_file(fname)?;

        api.pl_item_unref(nowplaying)?;
        api.pl_unlock()?;

        Ok(result)
    } else {
        Ok(false)
    }
}

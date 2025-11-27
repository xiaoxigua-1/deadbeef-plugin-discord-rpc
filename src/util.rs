use std::{
    ffi::{CStr, c_char},
    mem,
};

use crate::{
    API,
    deadbeef::{PL_MAIN, ddb_tf_context_t},
    error::{Error, Result},
};

static MAX_LEN: usize = 256;

pub fn nowplaying_format_string(script: &str) -> Result<String> {
    let api = API.get().unwrap();
    let nowplaying = api.streamer_get_playing_track()?;
    let nowplaying_plt = api.plt_get_curr()?;
    let code_script = api.tf_compile(script)?;

    let mut context: Box<ddb_tf_context_t> = unsafe { Box::new(mem::zeroed()) };
    let mut out: Vec<u8> = vec![0; MAX_LEN];
    let out_ptr = out.as_mut_ptr() as *mut c_char;

    context._size = std::mem::size_of::<ddb_tf_context_t>() as i32;
    context.it = nowplaying.as_ptr();
    context.plt = nowplaying_plt.as_ptr();
    context.iter = PL_MAIN as i32;

    if !code_script.is_null() {
        api.tf_eval(context.as_mut(), &code_script, out_ptr, MAX_LEN as i32)?;
    }

    let c_str = CStr::from_bytes_until_nul(&out).map_err(Error::FromBytesUntilNulError)?;

    Ok(c_str.to_string_lossy().to_string())
}

pub fn nowplaying_length() -> Result<f32> {
    let api = API.get().unwrap();
    let nowplaying = api.streamer_get_playing_track()?;

    if !nowplaying.is_null() {
        Ok(API.get().unwrap().pl_get_item_duration(&nowplaying)?)
    } else {
        Ok(0.0)
    }
}

pub fn is_streaming() -> Result<bool> {
    let api = API.get().unwrap();
    let nowplaying = api.streamer_get_playing_track()?;
    if !nowplaying.is_null() {
        api.pl_lock()?;

        let result = (|| -> Result<bool> {
            let fname = api.pl_find_meta(&nowplaying, c":URI".as_ptr())?;
            api.is_local_file(fname)
        })();

        api.pl_unlock()?;

        result
    } else {
        Ok(false)
    }
}

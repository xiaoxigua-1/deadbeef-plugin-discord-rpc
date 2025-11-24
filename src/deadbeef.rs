#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]
#![allow(clippy::all)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unnecessary_transmutes)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::c_void;

use crate::error::Error;
use crate::error::Result;

macro_rules! call_optional_fn {
    ($name:expr) => {
        match $name {
            Some(f) => Ok(unsafe { f() }),
            None => Err(Error::MissingFunction),
        }
    };
    ($name:expr, $($arg:expr),+ ) => {
        match $name {
            Some(f) => Ok(unsafe { f($($arg),+) }),
            None => Err(Error::MissingFunction),
        }
    };
}

impl DB_functions_t {
    pub fn streamer_get_playing_track(&self) -> Result<*mut DB_playItem_s> {
        call_optional_fn!(self.streamer_get_playing_track)
    }

    pub fn plt_get_curr(&self) -> Result<*mut ddb_playlist_t> {
        call_optional_fn!(self.plt_get_curr)
    }

    pub fn tf_compile(&self, script: &str) -> Result<*mut i8> {
        call_optional_fn!(self.tf_compile, CString::new(script).unwrap().as_ptr())
    }

    pub fn pl_item_unref(&self, item: *mut DB_playItem_s) -> Result<()> {
        call_optional_fn!(self.pl_item_unref, item)
    }

    pub fn plt_unref(&self, plt: *mut ddb_playlist_t) -> Result<()> {
        call_optional_fn!(self.plt_unref, plt)
    }

    pub fn tf_free(&self, code_script: *mut i8) -> Result<()> {
        call_optional_fn!(self.tf_free, code_script)
    }

    pub fn pl_get_item_duration(&self, item: *mut DB_playItem_s) -> Result<f32> {
        call_optional_fn!(self.pl_get_item_duration, item)
    }

    pub fn pl_lock(&self) -> Result<()> {
        call_optional_fn!(self.pl_lock)
    }

    pub fn pl_find_meta(&self, plt: *mut DB_playItem_s, value: *const i8) -> Result<*const i8> {
        call_optional_fn!(self.pl_find_meta, plt, value)
    }

    pub fn is_local_file(&self, item: *const i8) -> Result<bool> {
        let ret = { call_optional_fn!(self.is_local_file, item) }?;

        Ok(ret == 0)
    }

    pub fn pl_unlock(&self) -> Result<()> {
        call_optional_fn!(self.pl_unlock)
    }

    pub fn get_output(&self) -> Result<*mut DB_output_s> {
        call_optional_fn!(self.get_output)
    }

    pub fn playback_get_pos(&self) -> Result<f32> {
        call_optional_fn!(self.playback_get_pos)
    }

    pub fn thread_start(
        &self,
        func: unsafe extern "C" fn(*mut c_void),
        args: *mut c_void,
    ) -> Result<isize> {
        call_optional_fn!(self.thread_start, Some(func), args)
    }

    pub fn conf_get_str(&self, key: *const i8, def: *const i8) -> Result<String> {
        let mut buf = vec![0u8; 256];

        {
            call_optional_fn!(
                self.conf_get_str,
                key,
                def,
                buf.as_mut_ptr() as *mut i8,
                buf.len() as i32
            )
        }?;
        let c_str = CStr::from_bytes_until_nul(&buf).map_err(Error::FromBytesUntilNulError)?;

        Ok(c_str.to_string_lossy().to_string())
    }

    pub fn conf_get_int(&self, key: *const i8, def: i32) -> Result<i32> {
        call_optional_fn!(self.conf_get_int, key, def)
    }

    pub fn tf_eval(
        &self,
        context: *mut ddb_tf_context_s,
        code_script: *const i8,
        out: *mut i8,
        len: i32,
    ) -> Result<i32> {
        call_optional_fn!(self.tf_eval, context, code_script, out, len)
    }
}

impl DB_output_s {
    pub fn state(&self) -> Result<u32> {
        call_optional_fn!(self.state)
    }
}

impl Drop for DB_playItem_s {
    fn drop(&mut self) {
        let api = crate::API.get().unwrap();
        let _ = api.pl_item_unref(self);
    }
}

impl Drop for ddb_playlist_t {
    fn drop(&mut self) {
        let api = crate::API.get().unwrap();
        let _ = api.plt_unref(self);
    }
}

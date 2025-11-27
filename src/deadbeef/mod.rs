#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]
#![allow(clippy::all)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unnecessary_transmutes)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod http;
pub mod safe_wrapper;

use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::c_void;

use crate::config::PLUGIN;
use crate::deadbeef::safe_wrapper::SafeDBFile;
use crate::deadbeef::safe_wrapper::SafeDBPlayItem;
use crate::deadbeef::safe_wrapper::SafeDBPlayList;
use crate::deadbeef::safe_wrapper::SafeDBTitleFormat;
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
    pub fn streamer_get_playing_track(&self) -> Result<SafeDBPlayItem> {
        let ptr = call_optional_fn!(self.streamer_get_playing_track)?;

        Ok(SafeDBPlayItem::new(ptr))
    }

    pub fn plt_get_curr(&self) -> Result<SafeDBPlayList> {
        let ptr = call_optional_fn!(self.plt_get_curr)?;

        Ok(SafeDBPlayList::new(ptr))
    }

    pub fn tf_compile(&self, script: &str) -> Result<SafeDBTitleFormat> {
        let ptr = call_optional_fn!(self.tf_compile, CString::new(script).unwrap().as_ptr())?;

        Ok(SafeDBTitleFormat::new(ptr))
    }

    pub fn tf_eval(
        &self,
        context: *mut ddb_tf_context_s,
        code_script: &SafeDBTitleFormat,
        out: *mut i8,
        len: i32,
    ) -> Result<i32> {
        call_optional_fn!(self.tf_eval, context, code_script.as_ptr(), out, len)
    }

    pub fn tf_free(&self, code_script: *mut i8) -> Result<()> {
        call_optional_fn!(self.tf_free, code_script)
    }

    pub fn pl_item_unref(&self, item: *mut DB_playItem_s) -> Result<()> {
        call_optional_fn!(self.pl_item_unref, item)
    }

    pub fn plt_unref(&self, plt: *mut ddb_playlist_t) -> Result<()> {
        call_optional_fn!(self.plt_unref, plt)
    }

    pub fn pl_get_item_duration(&self, item: &SafeDBPlayItem) -> Result<f32> {
        call_optional_fn!(self.pl_get_item_duration, item.as_ptr())
    }

    pub fn pl_lock(&self) -> Result<()> {
        call_optional_fn!(self.pl_lock)
    }

    pub fn pl_find_meta(&self, plt: &SafeDBPlayItem, value: *const i8) -> Result<*const i8> {
        call_optional_fn!(self.pl_find_meta, plt.as_ptr(), value)
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

    pub fn fopen(&self, url: &str) -> Result<SafeDBFile> {
        let c_str = CString::new(url).unwrap();
        let ptr = call_optional_fn!(self.fopen, c_str.as_ptr())?;

        Ok(SafeDBFile::new(ptr))
    }

    pub fn fclose(&self, file: *mut DB_FILE) -> Result<()> {
        call_optional_fn!(self.fclose, file)
    }

    pub fn fgetlength(&self, file: &SafeDBFile) -> Result<i64> {
        call_optional_fn!(self.fgetlength, file.as_ptr())
    }

    pub fn fread(
        &self,
        ptr: *mut c_void,
        size: usize,
        nmemb: usize,
        stream: &SafeDBFile,
    ) -> Result<usize> {
        call_optional_fn!(self.fread, ptr, size, nmemb, stream.as_ptr())
    }

    fn log_detailed(
        &self,
        plugin: *mut DB_plugin_s,
        layers: u32,
        char: *const i8,
        args: va_list,
    ) -> Result<()> {
        call_optional_fn!(self.log_detailed, plugin, layers, char, args)
    }
}

impl DB_functions_t {
    pub fn trace(&self, msg: String) {
        let c_msg = CString::new(msg).unwrap();
        let plugin = &PLUGIN.0.plugin as *const DB_plugin_s as *mut DB_plugin_s;

        if cfg!(debug_assertions) {
            let plugin_id = unsafe { CStr::from_ptr(PLUGIN.0.plugin.id) }.to_string_lossy();
            let msg_str = c_msg.to_string_lossy();

            println!("[Deadbeef][{}] {}", plugin_id, msg_str);
        } else {
            // TODO: idk where this actually shows up.
            // log_detailed only appears in Deadbeef debug builds or development versions.
            // In a normal release build, this message will not be visible.
            self.log_detailed(plugin, DDB_LOG_LAYER_DEFAULT, c_msg.as_ptr(), unsafe {
                std::mem::zeroed()
            })
            .ok();
        };
    }
}

impl DB_output_s {
    pub fn state(&self) -> Result<u32> {
        call_optional_fn!(self.state)
    }
}

use crate::API;
use crate::deadbeef::{DB_FILE, DB_playItem_s, ddb_playlist_t};

macro_rules! safe_wrapper {
    ($name:ident, $inner_type:ty, $free:ident) => {
        pub struct $name {
            inner: *mut $inner_type,
        }

        impl $name {
            pub fn new(inner: *mut $inner_type) -> Self {
                Self { inner }
            }

            pub fn is_null(&self) -> bool {
                self.inner.is_null()
            }

            pub fn as_ptr(&self) -> *mut $inner_type {
                self.inner
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                let api = API.get().unwrap();

                if !self.inner.is_null() {
                    let _ = api.$free(self.inner);
                }
            }
        }
    };
}

safe_wrapper!(SafeDBFile, DB_FILE, fclose);
safe_wrapper!(SafeDBPlayItem, DB_playItem_s, pl_item_unref);
safe_wrapper!(SafeDBPlayList, ddb_playlist_t, plt_unref);

use std::ffi::CStr;

use crate::error::{Error, Result};

pub const PLUGIN_ID: &CStr = c"discordrpc";
pub const PLUGIN_NAME: &CStr = c"Discord Rich Presence";
pub const PLUGIN_DESC: &CStr =
    c"Updates Discord Rich Presence with the current track info from DeadBeef.";
pub const PLUGIN_WEBSITE: &CStr = c"https://github.com/xiaoxigua-1/deadbeef-plugin-discord-rpc";
pub const PLUGIN_COPYRIGHT: &CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(include_bytes!(concat!(env!("OUT_DIR"), "/LICENSE")))
};
pub const PLUGIN_SETTING_DLG: &CStr = cr#"
property "Enable" checkbox discordrpc.enable 1;
property "Client ID" entry discordrpc.client_id "1440255782418387026";
property "Title format" entry discordrpc.title_script "%title%$if(%ispaused%,' ('paused')')";
property "State format" entry discordrpc.state_script "%artist%";
property "Display time" select[3] discord_presence.end_timestamp2 1 "Only elapsed time" "Full track time" "Don't display time";
property "Icon text format" entry discordrpc.icon_script "%album%";
property "Display cover from" select[2] discordrpc.cover_source 1 "No cover" "MusicBrainz";
property "MusicBrainz album query format" entry discorrpc.query_album_script "release:\"%album%\" AND artist:\"%artist%\"";
"#;

pub struct ConfigKey;
pub struct ConfigDefault;

impl ConfigKey {
    pub const ENABLE: *const i8 = c"discordrpc.enable".as_ptr();
    pub const CLIENT_ID: *const i8 = c"discordrpc.client_id".as_ptr();
    pub const TITLE_SCRIPT: *const i8 = c"discordrpc.title_script".as_ptr();
    pub const STATE_SCRIPT: *const i8 = c"discordrpc.state_script".as_ptr();
    pub const END_TIMESTAMP2: *const i8 = c"discord_presence.end_timestamp2".as_ptr();
    pub const ICON_SCRIPT: *const i8 = c"discordrpc.icon_script".as_ptr();
    pub const COVER_SOURCE: *const i8 = c"discordrpc.cover_source".as_ptr();
    pub const QUERY_ALBUM_SCRIPT: *const i8 = c"discorrpc.query_album_script".as_ptr();
}

impl ConfigDefault {
    pub const ENABLE: i32 = 1;
    pub const CLIENT_ID: *const i8 = c"1440255782418387026".as_ptr();
    pub const TITLE_SCRIPT: *const i8 = c"%title%$if(%ispaused%,' ('paused')')".as_ptr();
    pub const STATE_SCRIPT: *const i8 = c"%artist%".as_ptr();
    pub const END_TIMESTAMP2: i32 = 1;
    pub const ICON_SCRIPT: *const i8 = c"%album%".as_ptr();
    pub const COVER_SOURCE: i32 = CoverSource::MusicBrainz as i32;
    pub const QUERY_ALBUM_SCRIPT: *const i8 =
        cr#"release:\"%album%\" AND artist:\"%artist%\""#.as_ptr();
}

#[repr(i32)]
pub enum CoverSource {
    NoCover = 0,
    MusicBrainz = 1,
}

impl TryFrom<i32> for CoverSource {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(CoverSource::NoCover),
            1 => Ok(CoverSource::MusicBrainz),
            _ => Err(Error::InvalidCoverSource),
        }
    }
}

use std::ffi::FromBytesUntilNulError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingFunction,
    SetActivityFailed,
    DiscordFailed(discord_rich_presence::error::Error),
    InvalidStatus,
    SystemTimeError(std::time::SystemTimeError),
    FromBytesUntilNulError(FromBytesUntilNulError),
    InvalidCoverSource,
    UreqRequestError(ureq::Error),
    JsonParseError(json::Error),

    MusicbrainzNoReleaseFound,
}

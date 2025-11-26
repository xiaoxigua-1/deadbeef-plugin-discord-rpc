use json::JsonValue;
use urlencoding::encode;

use crate::{
    API,
    error::{Error, Result},
};

pub fn query_album_json(query: &str) -> Result<String> {
    let url = format!(
        "https://musicbrainz.org/ws/2/release?query={}&fmt=json",
        encode(query)
    );
    let api = API.get().unwrap();

    api.http_get(&url)
}

pub fn query_album(query: &str) -> Result<String> {
    let json_raw = query_album_json(query)?;
    let json = json::parse(&json_raw).map_err(Error::JsonParseError)?;

    if let JsonValue::Object(object) = json
        && let Some(releases_value) = object.get("releases")
        && let JsonValue::Array(releases) = releases_value
        && let Some(relese) = releases.first()
        && let JsonValue::Object(relese) = relese
        && let Some(id_value) = relese.get("id")
        && let JsonValue::String(id) = id_value
    {
        Ok(id.clone())
    } else {
        Err(Error::MusicbrainzNoReleaseFound)
    }
}

pub fn get_album_cover_url_from_query(query: &str) -> Result<String> {
    let mb_release_id = query_album(query)?;

    Ok(get_album_cover_url(&mb_release_id))
}

pub fn get_album_cover_url(mb_release_id: &str) -> String {
    format!(
        "https://coverartarchive.org/release/{}/front-250",
        mb_release_id
    )
}

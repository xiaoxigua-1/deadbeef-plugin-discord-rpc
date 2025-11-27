use json::JsonValue;
use urlencoding::encode;

use crate::{
    API,
    error::{Error, Result},
};

fn query_album_json(query: &str) -> Result<String> {
    let url = format!(
        "https://musicbrainz.org/ws/2/release?query={}&fmt=json&limit=10",
        encode(query)
    );
    let api = API.get().unwrap();

    api.http_get(&url)
}

fn release_has_artwork(mb_release_id: &str) -> Result<bool> {
    let url = format!(
        "https://musicbrainz.org/ws/2/release/{}?fmt=json",
        mb_release_id
    );
    let api = API.get().unwrap();
    let json_raw = api.http_get(&url)?;
    let json = json::parse(&json_raw).map_err(Error::JsonParseFailed)?;

    if let JsonValue::Object(object) = json
        && let Some(cover_art_archive) = object.get("cover-art-archive")
        && let JsonValue::Object(cover_art_archive) = cover_art_archive
        && let Some(artwork) = cover_art_archive.get("artwork")
        && let JsonValue::Boolean(has_artwork) = artwork
    {
        Ok(*has_artwork)
    } else {
        Ok(false)
    }
}

fn query_releases(query: &str) -> Result<Vec<String>> {
    let json_raw = query_album_json(query)?;
    let json = json::parse(&json_raw).map_err(Error::JsonParseFailed)?;

    if let JsonValue::Object(object) = json
        && let Some(releases_value) = object.get("releases")
        && let JsonValue::Array(releases) = releases_value
    {
        Ok(releases
            .iter()
            .filter_map(|release| {
                if let JsonValue::Object(release_obj) = release
                    && let Some(id_value) = release_obj.get("id")
                    && let JsonValue::String(id) = id_value
                {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>())
    } else {
        Err(Error::MusicbrainzNoReleaseFound)
    }
}

pub fn get_album_cover_url_from_query(query: &str) -> Result<String> {
    let mb_release_ids = query_releases(query)?;
    for mb_release_id in mb_release_ids {
        if release_has_artwork(&mb_release_id)? {
            return Ok(get_album_cover_url(&mb_release_id));
        }
    }

    Err(Error::MusicbrainzNoReleaseFound)
}

fn get_album_cover_url(mb_release_id: &str) -> String {
    format!(
        "https://coverartarchive.org/release/{}/front-250",
        mb_release_id
    )
}

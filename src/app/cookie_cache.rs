use crate::app::{CrtCredentials, CrtSession};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use time::OffsetDateTime;

#[derive(Debug, Deserialize, Serialize)]
struct CookieEntry {
    created_timestamp: i64,
    value: CrtSession,
}

fn get_cache_filepath() -> PathBuf {
    temp_dir().join("crtcli_cookie_cache.bin")
}

fn hash_credentials(credentials: &CrtCredentials) -> u64 {
    let mut hasher = DefaultHasher::new();
    credentials.hash(&mut hasher);
    hasher.finish()
}

fn get_cookie_cache() -> Option<HashMap<u64, CookieEntry>> {
    let file = match File::open(get_cache_filepath()) {
        Err(_) => return None,
        Ok(file) => file,
    };

    let cache: HashMap<u64, CookieEntry> = match bincode::deserialize_from(file) {
        Err(_) => return None,
        Ok(cache) => cache,
    };

    Some(cache)
}

pub fn get_cookie_cache_entry(credentials: &CrtCredentials) -> Option<CrtSession> {
    let mut cache = match get_cookie_cache() {
        Some(cache) => cache,
        None => return None,
    };

    let hash = hash_credentials(credentials);
    let past_hour_timestamp =
        (OffsetDateTime::now_utc() - time::Duration::hours(1)).unix_timestamp();

    cache
        .remove(&hash)
        .filter(|x| x.created_timestamp > past_hour_timestamp)
        .map(|x| x.value)
}

pub fn set_cookie_cache_entry(credentials: &CrtCredentials, session: CrtSession) {
    let cache = get_cookie_cache().unwrap_or_default();
    let hash = hash_credentials(credentials);
    let past_hour_timestamp =
        (OffsetDateTime::now_utc() - time::Duration::hours(1)).unix_timestamp();

    let mut cache: HashMap<u64, CookieEntry> = HashMap::from_iter(
        cache
            .into_iter()
            .filter(|(_, e)| e.created_timestamp > past_hour_timestamp),
    );

    cache.insert(
        hash,
        CookieEntry {
            created_timestamp: OffsetDateTime::now_utc().unix_timestamp(),
            value: session,
        },
    );

    let _ = File::create(get_cache_filepath()).map(|file| bincode::serialize_into(file, &cache));
}

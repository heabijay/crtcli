use crate::app::{CrtCredentials, CrtSession};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::RwLock;
use time::OffsetDateTime;

#[derive(Debug, Deserialize, Clone, Serialize)]
struct CrtSessionCacheEntry {
    created_timestamp: i64,
    value: CrtSession,
}

trait CrtSessionCacheStorage: Send + Sync {
    fn get(&self) -> Cow<HashMap<u64, CrtSessionCacheEntry>>;

    fn set(&self, value: Cow<HashMap<u64, CrtSessionCacheEntry>>);
}

struct BinaryFileCrtSessionCacheStorage {
    filepath: PathBuf,
}

impl CrtSessionCacheStorage for BinaryFileCrtSessionCacheStorage {
    fn get(&self) -> Cow<HashMap<u64, CrtSessionCacheEntry>> {
        let cache: HashMap<u64, CrtSessionCacheEntry> = File::open(&self.filepath)
            .ok()
            .and_then(|f| bincode::deserialize_from(f).ok())
            .unwrap_or_default();

        Cow::Owned(cache)
    }

    fn set(&self, value: Cow<HashMap<u64, CrtSessionCacheEntry>>) {
        let _ = File::create(&self.filepath)
            .is_ok_and(|file| bincode::serialize_into(file, value.as_ref()).is_ok());
    }
}

#[derive(Default)]
struct MemoryCrtSessionCacheStorage {
    cache: RwLock<HashMap<u64, CrtSessionCacheEntry>>,
}

impl MemoryCrtSessionCacheStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CrtSessionCacheStorage for MemoryCrtSessionCacheStorage {
    fn get(&self) -> Cow<HashMap<u64, CrtSessionCacheEntry>> {
        Cow::Owned(self.cache.read().unwrap().clone())
    }

    fn set(&self, value: Cow<HashMap<u64, CrtSessionCacheEntry>>) {
        *self.cache.write().unwrap() = value.into_owned();
    }
}

pub trait CrtSessionCache: Send + Sync {
    fn get_entry(&self, credentials: &CrtCredentials) -> Option<CrtSession>;

    fn set_entry(&self, credentials: &CrtCredentials, session: CrtSession);
}

struct DefaultCrtSessionCache<S>
where
    S: CrtSessionCacheStorage,
{
    storage: S,
}

impl<S> DefaultCrtSessionCache<S>
where
    S: CrtSessionCacheStorage,
{
    fn hash_credentials(credentials: &CrtCredentials) -> u64 {
        let mut hasher = DefaultHasher::new();
        credentials.hash(&mut hasher);
        hasher.finish()
    }

    fn get_outdated_since_timestamp() -> i64 {
        (OffsetDateTime::now_utc() - time::Duration::hours(1)).unix_timestamp()
    }
}

impl<S> CrtSessionCache for DefaultCrtSessionCache<S>
where
    S: CrtSessionCacheStorage,
{
    fn get_entry(&self, credentials: &CrtCredentials) -> Option<CrtSession> {
        let hash = Self::hash_credentials(credentials);

        let cache = self.storage.get();
        let entry = cache.get(&hash);

        entry
            .filter(|x| x.created_timestamp > Self::get_outdated_since_timestamp())
            .map(|x| x.value.clone())
    }

    fn set_entry(&self, credentials: &CrtCredentials, session: CrtSession) {
        let hash = Self::hash_credentials(credentials);
        let outdated_since = Self::get_outdated_since_timestamp();

        let mut cache = self.storage.get().into_owned();

        cache.insert(
            hash,
            CrtSessionCacheEntry {
                created_timestamp: OffsetDateTime::now_utc().unix_timestamp(),
                value: session,
            },
        );

        cache.retain(|_, x| x.created_timestamp > outdated_since);

        self.storage.set(Cow::Owned(cache));
    }
}

pub fn create_default_session_cache() -> Box<dyn CrtSessionCache> {
    Box::new(DefaultCrtSessionCache {
        storage: BinaryFileCrtSessionCacheStorage {
            filepath: temp_dir().join("crtcli-sessions.cache"),
        },
    })
}

pub fn create_memory_session_cache() -> Box<dyn CrtSessionCache> {
    Box::new(DefaultCrtSessionCache {
        storage: MemoryCrtSessionCacheStorage::new(),
    })
}

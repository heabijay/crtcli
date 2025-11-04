use crate::app::{CrtCredentials, CrtSession};
use bincode::{Decode, Encode};
use std::borrow::Cow;
use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::RwLock;
use time::OffsetDateTime;

#[derive(Debug, Encode, Decode, Clone)]
struct CrtSessionCacheEntry {
    created_timestamp: i64,
    value: CrtSession,
}

trait CrtSessionCacheStorage: Send + Sync {
    fn get(&self) -> Cow<'_, HashMap<u64, CrtSessionCacheEntry>>;

    fn set(&self, value: Cow<HashMap<u64, CrtSessionCacheEntry>>);
}

struct BinaryFileCrtSessionCacheStorage {
    filepath: PathBuf,
}

impl CrtSessionCacheStorage for BinaryFileCrtSessionCacheStorage {
    fn get(&self) -> Cow<'_, HashMap<u64, CrtSessionCacheEntry>> {
        let cache: HashMap<u64, CrtSessionCacheEntry> = File::open(&self.filepath)
            .ok()
            .and_then(|mut f| {
                bincode::decode_from_std_read(&mut f, bincode::config::standard()).ok()
            })
            .unwrap_or_default();

        Cow::Owned(cache)
    }

    fn set(&self, value: Cow<HashMap<u64, CrtSessionCacheEntry>>) {
        let _ = File::create(&self.filepath).is_ok_and(|mut file| {
            bincode::encode_into_std_write(value.as_ref(), &mut file, bincode::config::standard())
                .is_ok()
        });
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
    fn get(&self) -> Cow<'_, HashMap<u64, CrtSessionCacheEntry>> {
        Cow::Owned(self.cache.read().unwrap().clone())
    }

    fn set(&self, value: Cow<HashMap<u64, CrtSessionCacheEntry>>) {
        *self.cache.write().unwrap() = value.into_owned();
    }
}

pub trait CrtSessionCache: Send + Sync {
    fn clear_all(&self);

    fn get_entry(&self, credentials: &CrtCredentials) -> Option<CrtSession>;

    fn set_entry(&self, credentials: &CrtCredentials, session: CrtSession);

    #[allow(dead_code)]
    fn remove_entry(&self, credentials: &CrtCredentials);
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
    fn clear_all(&self) {
        self.storage.set(Default::default());
    }

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
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let default_outdated_since = Self::get_outdated_since_timestamp();

        let mut cache = self.storage.get().into_owned();

        cache.insert(
            hash,
            CrtSessionCacheEntry {
                created_timestamp: OffsetDateTime::now_utc().unix_timestamp(),
                value: session,
            },
        );

        cache.retain(|_, x| match &x.value {
            CrtSession::OAuthSession(oauth_session) => {
                x.created_timestamp + oauth_session.expires_in() >= now
            }
            _ => x.created_timestamp > default_outdated_since,
        });

        self.storage.set(Cow::Owned(cache));
    }

    fn remove_entry(&self, credentials: &CrtCredentials) {
        let hash = Self::hash_credentials(credentials);
        let mut cache = self.storage.get().into_owned();

        cache.remove(&hash);

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

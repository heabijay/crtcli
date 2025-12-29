use crate::app::{CrtCredentials, CrtSession};
use std::collections::HashMap;
use std::env::temp_dir;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::RwLock;
use time::OffsetDateTime;

#[derive(Debug, Clone, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
struct CrtSessionCacheEntry {
    created_timestamp: i64,
    value: CrtSession,
}

trait CrtSessionCacheStorage: Send + Sync {
    fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<u64, CrtSessionCacheEntry>) -> R;

    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut HashMap<u64, CrtSessionCacheEntry>);
}

struct BinaryFileCrtSessionCacheStorage {
    filepath: PathBuf,
}

impl BinaryFileCrtSessionCacheStorage {
    fn load(&self) -> HashMap<u64, CrtSessionCacheEntry> {
        std::fs::read(&self.filepath)
            .ok()
            .and_then(|bytes| rkyv::from_bytes::<_, rkyv::rancor::Error>(&bytes).ok())
            .unwrap_or_default()
    }
}

impl CrtSessionCacheStorage for BinaryFileCrtSessionCacheStorage {
    fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<u64, CrtSessionCacheEntry>) -> R,
    {
        let cache = self.load();
        f(&cache)
    }

    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut HashMap<u64, CrtSessionCacheEntry>),
    {
        let mut cache = self.load();

        f(&mut cache);

        if let Ok(bytes) = rkyv::to_bytes::<rkyv::rancor::Error>(&cache) {
            let _ = std::fs::write(&self.filepath, bytes);
        }
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
    fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<u64, CrtSessionCacheEntry>) -> R,
    {
        let guard = self.cache.read().unwrap();
        f(&guard)
    }

    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut HashMap<u64, CrtSessionCacheEntry>),
    {
        let mut guard = self.cache.write().unwrap();
        f(&mut guard);
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
        self.storage.update(|cache| cache.clear());
    }

    fn get_entry(&self, credentials: &CrtCredentials) -> Option<CrtSession> {
        let hash = Self::hash_credentials(credentials);
        let outdated_since = Self::get_outdated_since_timestamp();

        self.storage.read(|cache| {
            cache
                .get(&hash)
                .filter(|x| x.created_timestamp > outdated_since)
                .map(|x| x.value.clone())
        })
    }

    fn set_entry(&self, credentials: &CrtCredentials, session: CrtSession) {
        let hash = Self::hash_credentials(credentials);
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let default_outdated_since = Self::get_outdated_since_timestamp();

        self.storage.update(|cache| {
            cache.insert(
                hash,
                CrtSessionCacheEntry {
                    created_timestamp: now,
                    value: session,
                },
            );

            cache.retain(|_, x| match &x.value {
                CrtSession::OAuthSession(oauth_session) => {
                    x.created_timestamp + oauth_session.expires_in() >= now
                }
                _ => x.created_timestamp > default_outdated_since,
            });
        });
    }

    fn remove_entry(&self, credentials: &CrtCredentials) {
        let hash = Self::hash_credentials(credentials);
        self.storage.update(|cache| {
            cache.remove(&hash);
        });
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

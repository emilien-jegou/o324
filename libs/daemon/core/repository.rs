use native_db::{transaction::RwTransaction, ToInput};
use native_model::Model;
use std::marker::PhantomData;

use crate::core::storage::Storage;
// Import the new cache strategy components
use super::cache_strategy::{CacheStrategy, InMemoryCache, NoCache};

/// A trait for entities that can be stored in the repository and cached.
pub trait Cacheable: Model + Clone + Send + std::fmt::Debug + Sync {
    fn cache_key(&self) -> String;
}

/// A generic repository specialized at compile time for its caching strategy.
#[derive(Clone)]
pub struct Repository<T, C>
where
    T: Cacheable,
    C: CacheStrategy<T>,
{
    db: Storage,
    cache_handler: C,
    _marker: PhantomData<T>,
}

/// A builder for creating a `Repository`, using the type-state pattern.
pub struct RepositoryBuilder<T, C> {
    db: Storage,
    _marker: PhantomData<(T, C)>,
}

// Builder implementation for a NON-CACHED repository
impl<T: Cacheable> RepositoryBuilder<T, NoCache<T>> {
    /// Enables a thread-safe, in-memory cache for the repository.
    /// This transforms the builder into a cached-repository builder.
    pub fn with_cache(self) -> RepositoryBuilder<T, InMemoryCache<T>> {
        RepositoryBuilder {
            db: self.db,
            _marker: PhantomData,
        }
    }

    /// Builds and returns a non-cached `Repository`.
    pub fn build(self) -> Repository<T, NoCache<T>> {
        Repository {
            db: self.db,
            cache_handler: NoCache::new(),
            _marker: PhantomData,
        }
    }
}

// Builder implementation for a CACHED repository
impl<T: Cacheable> RepositoryBuilder<T, InMemoryCache<T>> {
    /// Builds and returns a cached `Repository`.
    pub fn build(self) -> Repository<T, InMemoryCache<T>> {
        Repository {
            db: self.db,
            cache_handler: InMemoryCache::new(),
            _marker: PhantomData,
        }
    }
}

/// A cached transaction wrapper, generic over the cache strategy.
pub struct CachedTransaction<'a, 'b, T, C>
where
    T: Cacheable,
    C: CacheStrategy<T>,
{
    pub txn: &'a mut RwTransaction<'b>,
    cache_handler: &'a C,
    _marker: PhantomData<T>,
}

impl<'a, 'b, T, C> CachedTransaction<'a, 'b, T, C>
where
    T: Cacheable + Clone + ToInput,
    C: CacheStrategy<T>,
{
    pub fn get(&self, key: &str) -> eyre::Result<Option<T>> {
        if let Some(item) = self.cache_handler.get_from_cache(key) {
            return Ok(Some(item));
        }
        let item_from_db = self.txn.get().primary::<T>(key.to_string())?;
        if let Some(item) = &item_from_db {
            self.cache_handler
                .insert_into_cache(key.to_string(), item.clone());
        }
        Ok(item_from_db)
    }

    pub fn insert(&mut self, item: T) -> eyre::Result<()> {
        self.txn.insert(item.clone())?;
        self.cache_handler.insert_into_cache(item.cache_key(), item);
        Ok(())
    }

    pub fn upsert(&mut self, item: T) -> eyre::Result<()> {
        self.txn.upsert(item.clone())?;
        self.cache_handler.insert_into_cache(item.cache_key(), item);
        Ok(())
    }
}

// Main Repository implementation
impl<T, C> Repository<T, C>
where
    T: Cacheable + ToInput + Clone + Send + Sync + 'static,
    C: CacheStrategy<T> + 'static,
{
    /// Returns a builder to construct a `Repository`.
    /// The builder defaults to creating a non-cached repository.
    pub fn builder(db: Storage) -> RepositoryBuilder<T, NoCache<T>> {
        RepositoryBuilder {
            db,
            _marker: PhantomData,
        }
    }

    pub fn get(&self, key: &str) -> eyre::Result<Option<T>> {
        if let Some(item) = self.cache_handler.get_from_cache(key) {
            return Ok(Some(item));
        }
        let item_from_db = self
            .db
            .read(|r_txn| Ok(r_txn.get().primary::<T>(key.to_string())?))?;
        if let Some(item) = &item_from_db {
            self.cache_handler
                .insert_into_cache(key.to_string(), item.clone());
        }
        Ok(item_from_db)
    }

    pub fn write_cached<F, R>(&self, f: F) -> eyre::Result<R>
    where
        F: FnOnce(&mut CachedTransaction<T, C>) -> eyre::Result<R>,
    {
        self.db.write(|rw_txn| {
            let mut cached_txn = CachedTransaction {
                txn: rw_txn,
                cache_handler: &self.cache_handler,
                _marker: PhantomData,
            };
            f(&mut cached_txn)
        })
    }
}

// services/repository/cache_strategy.rs

use dashmap::DashMap;
use std::{marker::PhantomData, sync::Arc};

use crate::core::repository::Cacheable;

/// A trait that defines the behavior for a caching layer.
/// This allows for compile-time specialization of the repository.
pub trait CacheStrategy<T>: Send + Sync + Clone {
    /// Creates a new instance of the cache strategy.
    fn new() -> Self;

    /// Retrieves an item from the cache.
    fn get_from_cache(&self, key: &str) -> Option<T>;

    /// Inserts an item into the cache.
    fn insert_into_cache(&self, key: String, value: T);
}

// --- No-Cache Implementation ---

/// A cache strategy that performs no caching. This is a zero-sized struct,
/// so it has no runtime cost.
#[derive(Clone, Copy)]
pub struct NoCache<T>(PhantomData<T>);

impl<T: Cacheable> CacheStrategy<T> for NoCache<T> {
    fn new() -> Self {
        Self(PhantomData::default())
    }

    #[inline(always)]
    fn get_from_cache(&self, _key: &str) -> Option<T> {
        None
    }

    #[inline(always)]
    fn insert_into_cache(&self, _key: String, _value: T) {}
}

// --- In-Memory Cache Implementation ---

/// A cache strategy that uses a thread-safe in-memory DashMap.
#[derive(Clone)]
pub struct InMemoryCache<T> {
    cache: Arc<DashMap<String, T>>,
}

impl<T: Cacheable> CacheStrategy<T> for InMemoryCache<T> {
    fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    fn get_from_cache(&self, key: &str) -> Option<T> {
        //debug!("GET {key}");
        self.cache.get(key).map(|v| v.value().clone())
    }

    fn insert_into_cache(&self, key: String, value: T) {
        //debug!("INSERT {key} {value:?}");
        self.cache.insert(key, value);
    }
}

use dashmap::DashMap;
use std::sync::Arc;

use crate::storage::entities::prefix_trie_node::PrefixTrieNode;

/// The type used for the in-memory, thread-safe cache.
///
/// It's an atomically reference-counted `DashMap` which provides
/// fine-grained locking for high-concurrency access.
pub(crate) type IndexCache = Arc<DashMap<String, PrefixTrieNode>>;

/// Creates a new, empty cache instance.
pub(crate) fn new() -> IndexCache {
    Arc::new(DashMap::new())
}

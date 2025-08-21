use crate::core::cache_strategy::InMemoryCache;
use crate::core::repository::{Cacheable, Repository};
use crate::core::storage::Storage;
use crate::entities::prefix_trie_node::PrefixTrieNode;

// This implementation should be placed where PrefixTrieNode is defined,
// for example in `src/entities/prefix_trie_node.rs`.
impl Cacheable for PrefixTrieNode {
    fn cache_key(&self) -> String {
        self.prefix.clone()
    }
}

/// The main interface for the prefix index.
#[derive(Clone)]
pub struct PrefixIndex {
    // The type now explicitly includes the caching strategy
    repo: Repository<PrefixTrieNode, InMemoryCache<PrefixTrieNode>>,
}

impl PrefixIndex {
    /// Opens an existing index file or creates a new one if it doesn't exist.
    pub fn new(db: Storage) -> Self {
        // The builder API works exactly as you desired, but now it uses
        // the type system to construct the correct kind of repository.
        Self {
            repo: Repository::<PrefixTrieNode, InMemoryCache<PrefixTrieNode>>::builder(db)
                .with_cache()
                .build(),
        }
    }

    // For reference, here is one of the methods, unchanged:
    pub fn add_ids(&self, ids: &[String]) -> eyre::Result<()> {
        self.repo.write_cached(|cached_txn| {
            for id in ids {
                if let Some(node) = cached_txn.get(id)? {
                    if node.is_end_of_id {
                        continue;
                    }
                }

                for i in 1..=id.len() {
                    let prefix = &id[..i];
                    let is_final_node = i == id.len();

                    if prefix.len() < 2 && !is_final_node {
                        continue;
                    }

                    let prefix_str = prefix.to_string();
                    let node_lookup = cached_txn.get(&prefix_str)?;

                    if let Some(mut existing_node) = node_lookup {
                        let mut changed = false;
                        if existing_node.is_unique {
                            existing_node.is_unique = false;
                            changed = true;
                        }
                        if is_final_node && !existing_node.is_end_of_id {
                            existing_node.is_end_of_id = true;
                            changed = true;
                        }

                        if changed {
                            // Note: The transaction helper is now just `CachedTransaction`
                            cached_txn.upsert(existing_node)?;
                        }
                    } else {
                        let new_node = PrefixTrieNode {
                            prefix: prefix_str,
                            is_unique: true,
                            is_end_of_id: is_final_node,
                        };
                        cached_txn.insert(new_node)?;
                    }
                }
            }
            Ok(())
        })
    }

    // `contains` and `find_shortest_unique_prefix` are also unchanged.
    pub fn contains(&self, id: &str) -> eyre::Result<bool> {
        Ok(self.repo.get(id)?.map_or(false, |node| node.is_end_of_id))
    }

    pub fn find_shortest_unique_prefix(&self, id: &str) -> eyre::Result<String> {
        if !self.contains(id)? {
            return Ok(id.into());
        }

        for i in 2..=id.len() {
            let prefix = &id[..i];
            if let Some(node) = self.repo.get(prefix)? {
                if node.is_unique {
                    return Ok(prefix.to_string());
                }
            } else {
                break;
            }
        }
        Ok(id.into())
    }
}

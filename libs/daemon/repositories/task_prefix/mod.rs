use crate::core::cache_strategy::InMemoryCache;
use crate::core::repository::{Cacheable, Repository};
use crate::core::storage::Storage;
use crate::entities::prefix_trie_node::PrefixTrieNode;

#[cfg(test)]
mod tests;

impl Cacheable for PrefixTrieNode {
    fn cache_key(&self) -> String {
        self.prefix.clone()
    }
}

/// The main interface for the prefix index.
#[derive(Clone)]
pub struct TaskPrefixRepository {
    repo: Repository<PrefixTrieNode, InMemoryCache<PrefixTrieNode>>,
}

#[allow(dead_code)]
impl TaskPrefixRepository {
    pub fn new(db: Storage) -> Self {
        Self {
            repo: Repository::<PrefixTrieNode, InMemoryCache<PrefixTrieNode>>::builder(db)
                .with_cache()
                .build(),
        }
    }

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

    pub fn contains(&self, id: &str) -> eyre::Result<bool> {
        Ok(self.repo.get(id)?.is_some_and(|node| node.is_end_of_id))
    }

    pub fn search_by_prefix(&self, prefix: &str) -> eyre::Result<Vec<PrefixTrieNode>> {
        self.repo.scan_by_key(prefix)
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

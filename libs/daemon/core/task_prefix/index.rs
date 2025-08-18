use native_db::transaction::RwTransaction;

use crate::storage::entities::prefix_trie_node::PrefixTrieNode;
use crate::storage::storage::Storage;

use super::cache;
use super::cache::IndexCache;

/// The main interface for the prefix index.
#[derive(Clone)]
pub struct PrefixIndex {
    db: Storage,
    cache: IndexCache,
}

impl PrefixIndex {
    /// Opens an existing index file or creates a new one if it doesn't exist.
    pub fn new(db: Storage) -> Self {
        let cache = cache::new();
        Self { db, cache }
    }

    /// Adds a batch of new IDs to the index, run in provided transaction.
    pub fn add_ids_txn(&self, qr: &mut RwTransaction<'_>, ids: &[String]) -> eyre::Result<()> {
        for id in ids {
            let full_id_node = self.cache.get(id).map(|r| r.value().clone()).or_else(|| {
                qr.get()
                    .primary::<PrefixTrieNode>(id.clone())
                    .ok()
                    .flatten()
            });

            if let Some(node) = full_id_node {
                if node.is_end_of_id {
                    self.cache.insert(id.clone(), node);
                    continue; // ID already exists, skip.
                }
            }

            for i in 1..=id.len() {
                let prefix = &id[..i];
                let is_final_node = i == id.len();

                // Only create/update nodes for prefixes of length >= 2, or for the final full ID.
                if prefix.len() < 2 && !is_final_node {
                    continue;
                }

                let prefix_str = prefix.to_string();

                let node_lookup = self
                    .cache
                    .get(&prefix_str)
                    .map(|r| r.value().clone())
                    .or_else(|| {
                        qr.get()
                            .primary::<PrefixTrieNode>(prefix_str.clone())
                            .ok()
                            .flatten()
                    });

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
                        qr.upsert(existing_node.clone())?;
                    }
                    self.cache.insert(prefix_str, existing_node);
                } else {
                    let new_node = PrefixTrieNode {
                        prefix: prefix_str.clone(),
                        is_unique: true,
                        is_end_of_id: is_final_node,
                    };
                    qr.insert(new_node.clone())?;
                    self.cache.insert(prefix_str, new_node);
                }
            }
        }
        Ok(())
    }

    /// Adds a batch of new IDs to the index.
    pub fn add_ids(&mut self, ids: &[String]) -> eyre::Result<()> {
        self.db.write(|rw| self.add_ids_txn(rw, ids))
    }

    /// Gets a node, checking the cache first before falling back to the database.
    fn get_node(&self, key: &str) -> eyre::Result<Option<PrefixTrieNode>> {
        // DashMap's .get() is cheap and highly concurrent.
        if let Some(node_ref) = self.cache.get(key) {
            return Ok(Some(node_ref.value().clone()));
        }

        // Cache miss, go to the database.
        let node_from_db = self
            .db
            .read(|r| Ok(r.get().primary::<PrefixTrieNode>(key.to_string())?))?;

        // Populate the cache for next time.
        if let Some(ref node) = node_from_db {
            self.cache.insert(key.to_string(), node.clone());
        }

        Ok(node_from_db)
    }

    /// Checks if a full ID exists in the index.
    pub fn contains(&self, id: &str) -> eyre::Result<bool> {
        if let Some(node) = self.get_node(id)? {
            return Ok(node.is_end_of_id);
        }
        Ok(false)
    }

    /// Finds the shortest unique prefix for a given full ID.
    pub fn find_shortest_unique_prefix(&self, id: &str) -> eyre::Result<String> {
        if !self.contains(id)? {
            return Ok(id.into());
        }

        // Start searching for unique prefixes from length 2.
        for i in 2..=id.len() {
            let prefix = &id[..i];
            if let Some(node) = self.get_node(prefix)? {
                if node.is_unique {
                    return Ok(prefix.to_string());
                }
            } else {
                // This case should ideally not be reached if `contains` passed,
                // as all prefixes (>=2) and the full ID node should exist.
                break;
            }
        }
        Ok(id.into())
    }
}

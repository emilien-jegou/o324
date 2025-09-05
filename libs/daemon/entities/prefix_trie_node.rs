use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

/// We build a Trie Node of task id prefixes so that the user can
/// reference task by prefixes while handling collision gracefully.
#[native_model(id = 2, version = 1)]
#[native_db]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrefixTrieNode {
    #[primary_key]
    pub prefix: String,
    pub is_unique: bool,
    pub is_end_of_id: bool,
}

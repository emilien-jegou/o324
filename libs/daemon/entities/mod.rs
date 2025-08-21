use native_db::Models;
use once_cell::sync::Lazy;

pub mod prefix_trie_node;
pub mod task;

pub fn get_models() -> Models {
    let mut models = Models::new();
    models.define::<task::Task>().unwrap();
    models.define::<prefix_trie_node::PrefixTrieNode>().unwrap();
    models
}

pub static MODELS: Lazy<Models> = Lazy::new(get_models);

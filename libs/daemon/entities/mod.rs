use once_cell::sync::Lazy;

use crate::core::named_model::NamedModels;

pub mod prefix_trie_node;
pub mod task;

pub fn get_models() -> NamedModels {
    let mut models = NamedModels::new();
    models.define::<task::Task>("task").unwrap();
    models
        .define::<prefix_trie_node::PrefixTrieNode>("prefix-trie-node")
        .unwrap();
    models
}

pub static MODELS: Lazy<NamedModels> = Lazy::new(get_models);

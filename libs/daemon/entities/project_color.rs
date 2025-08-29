use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

/// We build a Trie Node of task id prefixes so that the user can
/// reference task by prefixes while handling collision gracefully.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct ProjectColor {
    #[primary_key]
    pub project: String,
    pub color_hue: u32,
}

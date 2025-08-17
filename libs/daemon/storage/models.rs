use native_db::Models;
use once_cell::sync::Lazy;

use super::task::Task;

pub fn get_models() -> Models {
    let mut models = Models::new();
    models.define::<Task>().unwrap();
    models
}

pub static MODELS: Lazy<Models> = Lazy::new(get_models);

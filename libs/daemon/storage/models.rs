use native_db::Models;

use super::task::Task;

pub fn get_models() -> Models {
    let mut models = Models::new();
    models.define::<Task>().unwrap();
    models
}

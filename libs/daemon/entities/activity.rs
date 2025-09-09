use native_db::{native_db, ToKey};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[native_model(id = 4, version = 1)]
#[native_db]
#[derive(Clone, Debug, Serialize, Deserialize, TypedBuilder)]
pub struct Activity {
    #[primary_key]
    pub id: String,
    pub app_title: Option<String>,
    pub app_name: String,
    #[secondary_key]
    pub at: u64,
    pub computer_name: String,
}

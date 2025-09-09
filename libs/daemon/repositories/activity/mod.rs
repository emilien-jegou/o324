use crate::{
    core::{storage::Storage, utils::generate_random_id},
    entities::activity::{Activity, ActivityKey},
};
use std::sync::Arc;
use wrap_builder::wrap_builder;

pub mod defs;

#[wrap_builder(Arc)]
pub struct ActivityRepository {
    pub computer_name: String,
    pub storage: Storage,
}

impl ActivityRepositoryInner {
    pub fn register(&self, activity: defs::StartActivity) -> eyre::Result<String> {
        let activity_id = generate_random_id(7);
        let activity = Activity {
            id: activity_id.clone(),
            //app_title: activity.app_title,
            app_name: activity.app_name,
            start: activity.at,
            last_active: activity.at,
            computer_name: self.computer_name.clone(),
        };
        self.storage.insert(activity)?;
        Ok(activity_id)
    }

    pub fn update_last_event(&self, activity_id: String, new_last_active: u64) -> eyre::Result<()> {
        self.storage.write_txn(|qr| {
            if let Some(mut current_task) = qr.get().primary::<Activity>(activity_id)? {
                current_task.last_active = new_last_active;
                qr.upsert(current_task)?;
                Ok(())
            } else {
                Err(eyre::eyre!("Task not found"))
            }
        })?;
        Ok(())
    }

    pub async fn list_activity_range(
        &self,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> eyre::Result<Vec<Activity>> {
        self.storage.read_txn(|qr| {
            let filtered_tasks = qr
                .scan()
                .secondary::<Activity>(ActivityKey::start)?
                .range(start_timestamp..end_timestamp)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(filtered_tasks)
        })
    }
}

use crate::{
    config::Config,
    core::{Core, StartTaskInput},
    storage::models::get_models,
};
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, config: &Config) -> eyre::Result<()> {
    let models = get_models();
    let core = Core::try_new(&config, &models)?;
    // Now all method calls on `core` are valid because the trait bounds are met.
    core.start_new_task(StartTaskInput {
        task_name: "Hello".to_string(),
        tags: vec![],
        project: None,
        computer_name: config.core.computer_name.clone(),
    })
    .await?;

    let x = core.list_last_tasks(5).await?;

    println!("{:?}", x);
    Ok(())
}

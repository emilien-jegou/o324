use clap::Args;
use colored::Colorize;
use o324_core::Core;

#[derive(Args, Debug)]
pub struct Command {}

pub async fn handle(_: Command, core: &Core) -> eyre::Result<()> {
    let tasks = core.list_last_tasks(20).await?;

    for task in tasks.iter() {
        println!("{}", task.task_name.cyan());
        println!("Id: {}", task.ulid);
        println!("Started on: {}", task.start);
        println!("Ended on: {}", task.end.map_or("ONGOING".into(), |x| x.to_string()));

        if let Some(project) = &task.project {
            println!("Project: {}", project);
        }

        if !task.tags.is_empty() {
            println!("Tags: {:?}", task.tags);
        }

        println!();
    }
    Ok(())
}

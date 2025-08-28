use clap::{Args, Subcommand};
use o324_dbus::{dto, proxy::O324ServiceProxy};

#[derive(Args, Debug)]
pub struct ScanCommand {
    table_name: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Operation {
    /// Show all tables in database
    Tables,
    /// List all rows of a table
    Scan(ScanCommand),
}

#[derive(Args, Debug)]
pub struct Command {
    #[command(subcommand)]
    operation: Operation,
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> eyre::Result<()> {
    let operation_dto = match command.operation {
        Operation::Tables | Operation::Scan(ScanCommand { table_name: None }) => {
            dto::DbOperationDto {
                operation_type: dto::DbOperationTypeDto::ListTables,
                table_name: None,
            }
        }
        Operation::Scan(ScanCommand {
            table_name: Some(table_name),
        }) => dto::DbOperationDto {
            operation_type: dto::DbOperationTypeDto::ScanTable,
            table_name: Some(table_name),
        },
    };

    let result = proxy.db_query(operation_dto).await?.unpack();

    match result {
        dto::DbResultDto::TableList(list) => {
            if list.is_empty() {
                println!("No tables found in the database.");
            } else {
                println!("Available tables:");
                for table_name in list {
                    println!("- {table_name}");
                }
            }
        }
        dto::DbResultDto::TableRows(rows) => {
            if rows.is_empty() {
                println!("Table is empty or does not exist.");
            } else {
                println!("Found {} row(s):", rows.len());
                for json_row_string in rows {
                    let value: serde_json::Value = serde_json::from_str(&json_row_string)?;
                    let pretty_json = serde_json::to_string_pretty(&value)?;
                    println!("{pretty_json}");
                }
            }
        }
    }

    Ok(())
}

use clap::{Args, Subcommand};
use eyre::bail;
use o324_dbus::{dto, proxy::O324ServiceProxy};

#[derive(Args, Debug)]
pub struct ScanCommand {
    table_name: String,
}

#[derive(Subcommand, Debug)]
enum Operation {
    /// Show all tables in database
    ShowTables,
    /// List all rows of a table
    Scan(ScanCommand),
}

#[derive(Args, Debug)]
pub struct Command {
    #[command(subcommand)]
    operation: Operation,
    table: Option<String>,
}

pub async fn handle(command: Command, proxy: O324ServiceProxy<'_>) -> eyre::Result<()> {
    // 1. Construct the operation DTO based on the CLI command.
    let operation_dto = match command.operation {
        Operation::ShowTables => dto::DbOperationDto {
            operation_type: dto::DbOperationTypeDto::ListTables,
            table_name: None,
        },
        Operation::Scan(scan_command) => dto::DbOperationDto {
            operation_type: dto::DbOperationTypeDto::ScanTable,
            table_name: Some(scan_command.table_name),
        },
    };

    // 2. Call the proxy with the constructed DTO.
    let result = proxy.db_query(operation_dto).await?;

    // 3. Handle the structured result by matching on its `result_type`.
    //    This section is updated to use the new field names.
    match result.result_type {
        dto::DbResultTypeDto::TableList => {
            // Use `result.table_list` instead of `result.tables`
            if let Some(table_list) = result.table_list {
                if table_list.is_empty() {
                    println!("No tables found in the database.");
                } else {
                    println!("Available tables:");
                    for table_name in table_list {
                        println!("- {}", table_name);
                    }
                }
            } else {
                bail!("Service returned TableList type but did not provide table list data.");
            }
        }
        dto::DbResultTypeDto::TableRows => {
            // Use `result.table_rows` instead of `result.rows`
            if let Some(table_rows) = result.table_rows {
                if table_rows.is_empty() {
                    println!("Table is empty or does not exist.");
                } else {
                    println!("Found {} row(s):", table_rows.len());
                    for json_row_string in table_rows {
                        // The inner logic remains the same: parse and pretty-print the JSON string.
                        let value: serde_json::Value = serde_json::from_str(&json_row_string)?;
                        let pretty_json = serde_json::to_string_pretty(&value)?;
                        println!("{}", pretty_json);
                    }
                }
            } else {
                bail!("Service returned TableRows type but did not provide table row data.");
            }
        }
        dto::DbResultTypeDto::Error => {
            // Use `result.error` instead of `result.error_message`
            let err_msg = result
                .error
                .unwrap_or_else(|| "Unknown error from service".to_string());
            bail!("Service returned an error: {}", err_msg);
        }
    }

    Ok(())
}

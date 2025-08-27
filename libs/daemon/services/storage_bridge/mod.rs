use crate::{
    core::storage::Storage,
    entities::{prefix_trie_node::PrefixTrieNode, task::Task},
};
use native_db::{transaction::RTransaction, ToInput};
use native_model::Model;
use serde::Serialize;
use std::{any::TypeId, sync::Arc};
use wrap_builder::wrap_builder;

#[wrap_builder(Arc)]
pub struct StorageBridgeService {
    pub storage: Storage,
}

/// Defines a database query operation for the storage layer.
pub enum DbOperation {
    ListTables,
    ScanTable { table_name: String },
}

/// Represents a successful result from a database query.
pub enum DbResult {
    TableList(Vec<String>),
    TableRows(Vec<String>),
}

impl StorageBridgeServiceInner {
    pub fn db_query(&self, operation: DbOperation) -> eyre::Result<DbResult> {
        match operation {
            DbOperation::ListTables => {
                let names: Vec<String> = self.storage.models.iter().map(|x| x.0.clone()).collect();

                Ok(DbResult::TableList(names))
            }
            DbOperation::ScanTable { table_name } => {
                let rows = self.storage.read(|txn| {
                    let (tid, _) = self.storage.models.get(&table_name).ok_or_else(|| {
                        eyre::eyre!("Table '{}' not found or not scannable.", table_name)
                    })?;

                    if tid == &TypeId::of::<Task>() {
                        scan_and_serialize::<Task>(&txn)
                    } else if tid == &TypeId::of::<PrefixTrieNode>() {
                        scan_and_serialize::<PrefixTrieNode>(&txn)
                    } else {
                        unreachable!("Couldn't find table");
                    }
                })?;

                Ok(DbResult::TableRows(rows))
            }
        }
    }
}

fn scan_and_serialize<T: Model + Serialize + ToInput>(
    txn: &RTransaction,
) -> eyre::Result<Vec<String>> {
    let items = txn
        .scan()
        .primary::<T>()?
        .all()?
        .collect::<Result<Vec<_>, _>>()?;

    items
        .into_iter()
        .map(|item| serde_json::to_string(&item).map_err(|e| eyre::eyre!(e)))
        .collect()
}

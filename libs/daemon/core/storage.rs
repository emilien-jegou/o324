use native_db::{
    transaction::{self},
    Builder, Database,
};
use std::{path::Path, sync::Arc};

use crate::core::named_model::NamedModels;

#[derive(Clone)]
pub struct Storage {
    db: Arc<Database<'static>>,
    pub models: &'static NamedModels,
}

impl Storage {
    pub fn try_new(path: impl AsRef<Path>, models: &'static NamedModels) -> eyre::Result<Self> {
        let builder = Builder::new();
        let db = builder.create(models.get_inner(), path)?;
        Ok(Self {
            db: Arc::new(db),
            models,
        })
    }

    /// Executes read-only operation within a transaction
    pub fn read<F, R>(&self, f: F) -> eyre::Result<R>
    where
        F: FnOnce(transaction::RTransaction) -> eyre::Result<R>,
    {
        f(self.db.r_transaction()?)
    }

    /// Executes read-write operation within a transaction
    pub fn write<F, R>(&self, f: F) -> eyre::Result<R>
    where
        F: FnOnce(&mut transaction::RwTransaction) -> eyre::Result<R>,
    {
        let mut txn = self.db.rw_transaction()?;
        match f(&mut txn) {
            Ok(result) => {
                txn.commit()?;
                Ok(result)
            }
            e => {
                // RwTransaction doesn't seem to implement drop, there may
                // be nested properties with it but w/e let's be safe and call abort.
                txn.abort()?;
                e
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use native_db::native_db;
    use native_db::ToKey;
    use native_model::{native_model, Model};
    use once_cell::sync::Lazy;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    // 1. Define Test Models

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq, PartialOrd, Ord)]
    #[native_model(id = 1, version = 1)]
    #[native_db]
    struct Item {
        #[primary_key]
        id: u32,
        // Non-unique secondary key
        #[secondary_key]
        name: String,
        value: i32,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq, Ord, PartialOrd)]
    #[native_model(id = 2, version = 1)]
    #[native_db]
    struct User {
        #[primary_key]
        username: String,
        // Unique secondary key for testing `get().secondary()`
        #[secondary_key(unique)]
        email: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    #[native_model(id = 3, version = 1)]
    #[native_db]
    struct Settings {
        // Singleton primary key must be convertible from 0_u32
        #[primary_key]
        id: u32,
        theme: String,
        notifications_enabled: bool,
    }

    // 2. Test Setup Helpers
    fn setup_database(models: &'static NamedModels) -> (tempfile::TempDir, Storage) {
        let dir = tempdir().unwrap();
        let storage = Storage::try_new(dir.path().join("test.db"), models).unwrap();
        (dir, storage)
    }

    fn get_models() -> NamedModels {
        let mut models = NamedModels::new();
        models.define::<Item>("item").unwrap();
        models.define::<User>("user").unwrap();
        models.define::<Settings>("settings").unwrap();
        models
    }

    static MODELS: Lazy<NamedModels> = Lazy::new(get_models);

    // 3. Tests

    #[test]
    fn test_set_and_get_primary() -> eyre::Result<()> {
        let (_dir, storage) = setup_database(&MODELS);
        let item1 = Item {
            id: 1,
            name: "red".to_string(),
            value: 100,
        };

        storage.write(|txn| {
            txn.upsert(item1.clone())?;
            Ok(())
        })?;

        storage.read(|txn| {
            let retrieved_item = txn.get().primary::<Item>(1_u32)?.unwrap();
            assert_eq!(item1, retrieved_item);
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_remove_primary() -> eyre::Result<()> {
        let (_dir, storage) = setup_database(&MODELS);
        let item1 = Item {
            id: 1,
            name: "red".to_string(),
            value: 100,
        };

        storage.write(|txn| {
            txn.upsert(item1.clone())?;
            Ok(())
        })?;

        storage.write(|txn| {
            let item_to_remove = txn.get().primary::<Item>(1_u32)?.unwrap();
            txn.remove(item_to_remove)?;
            Ok(())
        })?;

        storage.read(|txn| {
            let result = txn.get().primary::<Item>(1_u32)?;
            assert!(result.is_none());
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_primary_key_scans() -> eyre::Result<()> {
        let (_dir, storage) = setup_database(&MODELS);
        let item1 = Item {
            id: 10,
            name: "item10".to_string(),
            value: 10,
        };
        let item2 = Item {
            id: 20,
            name: "item20".to_string(),
            value: 20,
        };
        let item3 = Item {
            id: 30,
            name: "item30".to_string(),
            value: 30,
        };
        let item4 = Item {
            id: 40,
            name: "item40".to_string(),
            value: 40,
        };

        storage.write(|txn| {
            txn.upsert(item1.clone())?;
            txn.upsert(item2.clone())?;
            txn.upsert(item3.clone())?;
            txn.upsert(item4.clone())?;
            Ok(())
        })?;

        storage.read(|txn| {
            // Test all_primary
            let all_items = txn
                .scan()
                .primary::<Item>()?
                .all()?
                .collect::<Result<Vec<_>, _>>()?;
            assert_eq!(all_items.len(), 4);
            assert_eq!(
                all_items,
                vec![item1.clone(), item2.clone(), item3.clone(), item4.clone()]
            );

            // Test first_primary
            let first = txn
                .scan()
                .primary::<Item>()?
                .all()?
                .next()
                .transpose()?
                .unwrap();
            assert_eq!(first, item1);

            // Test last_primary
            let last = txn
                .scan()
                .primary::<Item>()?
                .all()?
                .next_back()
                .transpose()?
                .unwrap();
            assert_eq!(last, item4);

            // Test range_primary
            let range_items = txn
                .scan()
                .primary::<Item>()?
                .range(20_u32..=30_u32)?
                .collect::<Result<Vec<_>, _>>()?;
            assert_eq!(range_items.len(), 2);
            assert_eq!(range_items, vec![item2.clone(), item3.clone()]);
            Ok(())
        })?;

        // Test start_with_primary (on User model with String key)
        let user1 = User {
            username: "albert".to_string(),
            email: "a@a.com".to_string(),
        };
        let user2 = User {
            username: "alice".to_string(),
            email: "a@b.com".to_string(),
        };
        let user3 = User {
            username: "bob".to_string(),
            email: "b@b.com".to_string(),
        };
        storage.write(|txn| {
            txn.upsert(user1.clone())?;
            txn.upsert(user2.clone())?;
            txn.upsert(user3.clone())?;
            Ok(())
        })?;
        storage.read(|txn| {
            let al_users = txn
                .scan()
                .primary::<User>()?
                .start_with("al".to_string())?
                .collect::<Result<Vec<_>, _>>()?;
            assert_eq!(al_users.len(), 2);
            assert_eq!(al_users, vec![user1, user2]);
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_non_unique_secondary_key_queries() -> eyre::Result<()> {
        let (_dir, storage) = setup_database(&MODELS);
        let item1 = Item {
            id: 1,
            name: "red".to_string(),
            value: 100,
        };
        let item2 = Item {
            id: 2,
            name: "blue".to_string(),
            value: 200,
        };
        let item3 = Item {
            id: 3,
            name: "red".to_string(),
            value: 300,
        };

        storage.write(|txn| {
            txn.upsert(item1.clone())?;
            txn.upsert(item2.clone())?;
            txn.upsert(item3.clone())?;
            Ok(())
        })?;

        storage.read(|txn| {
            // all_by_secondary (using range workaround)
            let mut red_items = txn
                .scan()
                .secondary::<Item>(ItemKey::name)?
                .range("red".to_string()..="red".to_string())?
                .collect::<Result<Vec<_>, _>>()?;
            red_items.sort(); // Order is not guaranteed for secondary keys
            assert_eq!(red_items.len(), 2);
            assert_eq!(red_items, vec![item1.clone(), item3.clone()]);

            // start_with_by_secondary
            let b_items = txn
                .scan()
                .secondary::<Item>(ItemKey::name)?
                .start_with("b".to_string())?
                .collect::<Result<Vec<_>, _>>()?;
            assert_eq!(b_items.len(), 1);
            assert_eq!(b_items[0], item2);

            // range_by_secondary
            let mut range_items = txn
                .scan()
                .secondary::<Item>(ItemKey::name)?
                .range("a".to_string().."c".to_string())?
                .collect::<Result<Vec<_>, _>>()?;
            range_items.sort();
            assert_eq!(range_items.len(), 1);
            assert_eq!(range_items, vec![item2]);

            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_unique_secondary_key() -> eyre::Result<()> {
        let (_dir, storage) = setup_database(&MODELS);
        let user1 = User {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        let user2_clash = User {
            username: "bob".to_string(),
            email: "alice@example.com".to_string(),
        };

        storage.write(|txn| {
            txn.upsert(user1.clone())?;
            Ok(())
        })?;

        // Test get().secondary() which only works for unique keys
        storage.read(|txn| {
            let found_user: User = txn
                .get()
                .secondary(UserKey::email, "alice@example.com".to_string())?
                .unwrap();
            assert_eq!(found_user, user1);
            Ok(())
        })?;

        // Test uniqueness constraint
        let result = storage.write(|txn| {
            txn.upsert(user2_clash)?;
            Ok(())
        });

        assert!(result.is_err());
        let err = result
            .unwrap_err()
            .downcast::<native_db::db_type::Error>()?;
        assert!(matches!(
            err,
            native_db::db_type::Error::DuplicateKey { .. }
        ));

        Ok(())
    }

    #[test]
    fn test_singleton_operations() -> eyre::Result<()> {
        let (_dir, storage) = setup_database(&MODELS);
        let settings = Settings {
            id: 0,
            theme: "dark".to_string(),
            notifications_enabled: true,
        };

        // Initially, no singleton exists
        storage.read(|txn| {
            let s = txn.get().primary::<Settings>(0_u32)?;
            assert!(s.is_none());
            Ok(())
        })?;

        // Set the singleton
        storage.write(|txn| {
            txn.upsert(settings.clone())?;
            Ok(())
        })?;

        // Get the singleton
        storage.read(|txn| {
            let s = txn.get().primary::<Settings>(0_u32)?.unwrap();
            assert_eq!(s, settings);
            Ok(())
        })?;

        // Remove the singleton
        storage.write(|txn| {
            let s = txn.get().primary::<Settings>(0_u32)?.unwrap();
            txn.remove(s)?;
            Ok(())
        })?;

        // Singleton should be gone
        storage.read(|txn| {
            let s = txn.get().primary::<Settings>(0_u32)?;
            assert!(s.is_none());
            Ok(())
        })?;

        Ok(())
    }
}

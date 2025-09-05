use futures::future::{BoxFuture, FutureExt};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;

/// Like a Dataloader but lightweight
#[derive(Clone)]
pub struct BatchLoader<K, V> {
    batch_call: Arc<dyn BatchCall<K, V>>,
}

pub trait BatchCall<K, V>: Send + Sync + 'static {
    fn call(&self, keys: Vec<K>) -> BoxFuture<'static, eyre::Result<HashMap<K, V>>>;
}

impl<K, V, F, Fut> BatchCall<K, V> for F
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Fn(Vec<K>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = eyre::Result<HashMap<K, V>>> + Send + 'static,
{
    fn call(&self, keys: Vec<K>) -> BoxFuture<'static, eyre::Result<HashMap<K, V>>> {
        self(keys).boxed()
    }
}

impl<K, V> BatchLoader<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Creates a new `BatchLoader` with any type that implements `BatchCall`.
    /// This can be a closure or a struct.
    pub fn new<B>(batch_logic: B) -> Self
    where
        B: BatchCall<K, V>,
    {
        Self {
            batch_call: Arc::new(batch_logic),
        }
    }

    /// Runs a batch operation for the given keys.
    pub async fn run(&self, keys: &[K]) -> eyre::Result<HashMap<K, V>> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let unique_keys: Vec<K> = keys
            .iter()
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        if unique_keys.is_empty() {
            return Ok(HashMap::new());
        }

        self.batch_call.call(unique_keys).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::eyre;
    use rand::Rng;
    use std::sync::{Arc, Mutex};
    use tokio::time::{sleep, Duration};

    /// A mock implementation of `BatchCall` for testing purposes.
    /// It allows us to track which keys were passed to the `call` method
    /// and how many times it was invoked.
    #[derive(Clone)]
    struct MockBatchCall<K, V> {
        calls: Arc<Mutex<Vec<Vec<K>>>>,
        result_generator: Arc<dyn Fn(&[K]) -> eyre::Result<HashMap<K, V>> + Send + Sync>,
    }

    impl<K, V> MockBatchCall<K, V>
    where
        K: Eq + Hash + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        fn new<F>(result_generator: F) -> Self
        where
            F: Fn(&[K]) -> eyre::Result<HashMap<K, V>> + Send + Sync + 'static,
        {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
                result_generator: Arc::new(result_generator),
            }
        }

        fn get_calls(&self) -> Vec<Vec<K>> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl<K, V> BatchCall<K, V> for MockBatchCall<K, V>
    where
        K: Eq + Hash + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        fn call(&self, keys: Vec<K>) -> BoxFuture<'static, eyre::Result<HashMap<K, V>>> {
            self.calls.lock().unwrap().push(keys.clone());
            let result = (self.result_generator)(&keys);
            async move { result }.boxed()
        }
    }

    #[tokio::test]
    async fn test_batcher_with_closure_success() -> eyre::Result<()> {
        let batcher = BatchLoader::new(|keys: Vec<i32>| async move {
            let mut results = HashMap::new();
            for k in keys {
                results.insert(k, format!("Value for {}", k));
            }
            Ok(results)
        });

        let keys = vec![1, 2, 3];
        let result_map = batcher.run(&keys).await?;

        assert_eq!(result_map.len(), 3);
        assert_eq!(result_map.get(&1), Some(&"Value for 1".to_string()));
        assert_eq!(result_map.get(&2), Some(&"Value for 2".to_string()));
        assert_eq!(result_map.get(&3), Some(&"Value for 3".to_string()));

        Ok(())
    }

    struct GreeterBatchLoader {
        greeting: String,
    }

    impl BatchCall<String, String> for GreeterBatchLoader {
        fn call(
            &self,
            names: Vec<String>,
        ) -> BoxFuture<'static, eyre::Result<HashMap<String, String>>> {
            let value = self.greeting.clone();

            async move {
                println!("\n--- Running Batch Callback with {:?} ---", names);
                let sleep_duration_ms = rand::rng().random_range(20..=200);
                println!("(Batch callback will sleep for {}ms)", sleep_duration_ms);
                tokio::time::sleep(std::time::Duration::from_millis(sleep_duration_ms)).await;

                let mut greetings = HashMap::new();
                for name in names.iter() {
                    greetings.insert(name.clone(), format!("{} {}", value, name));
                }
                println!("--- Batch Callback Finished ---");
                Ok(greetings)
            }
            .boxed() // Box the future to match the trait's return type.
        }
    }

    #[tokio::test]
    async fn test_batcher_with_struct_success() -> eyre::Result<()> {
        let greeter_logic = GreeterBatchLoader {
            greeting: "Hi".to_string(),
        };
        let batcher = BatchLoader::new(greeter_logic);

        let keys = vec!["Alice".to_string(), "Bob".to_string()];
        let result_map = batcher.run(&keys).await?;

        assert_eq!(result_map.len(), 2);
        assert_eq!(result_map.get("Alice"), Some(&"Hi Alice".to_string()));
        assert_eq!(result_map.get("Bob"), Some(&"Hi Bob".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_batcher_deduplicates_keys() -> eyre::Result<()> {
        let mock_logic = MockBatchCall::new(|keys: &[String]| {
            let mut results = HashMap::new();
            for k in keys {
                results.insert(k.clone(), format!("Processed {}", k));
            }
            Ok(results)
        });
        let batcher = BatchLoader::new(mock_logic.clone());

        let keys_with_duplicates = vec![
            "A".to_string(),
            "B".to_string(),
            "A".to_string(),
            "C".to_string(),
            "B".to_string(),
        ];
        let result_map = batcher.run(&keys_with_duplicates).await?;

        assert_eq!(result_map.len(), 3);
        assert!(result_map.contains_key("A"));
        assert!(result_map.contains_key("B"));
        assert!(result_map.contains_key("C"));

        let calls = mock_logic.get_calls();
        assert_eq!(calls.len(), 1);

        let received_keys = &calls[0];
        assert_eq!(received_keys.len(), 3);
        let received_keys_set: HashSet<_> = received_keys.iter().cloned().collect();
        let expected_keys_set: HashSet<_> = vec!["A".to_string(), "B".to_string(), "C".to_string()]
            .into_iter()
            .collect();
        assert_eq!(received_keys_set, expected_keys_set);

        Ok(())
    }

    #[tokio::test]
    async fn test_run_with_empty_keys_returns_empty_map() -> eyre::Result<()> {
        let mock_logic: MockBatchCall<i32, String> = MockBatchCall::new(|_: &[i32]| {
            panic!("Batch function was called with empty keys!");
        });
        let batcher = BatchLoader::new(mock_logic.clone());

        let keys: Vec<i32> = vec![];
        let result_map = batcher.run(&keys).await?;

        assert!(result_map.is_empty());
        assert_eq!(mock_logic.get_calls().len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_batcher_propagates_error() -> eyre::Result<()> {
        let mock_logic: MockBatchCall<String, ()> = MockBatchCall::new(|_: &[String]| {
            Err(eyre!("Something went wrong in the batch call!"))
        });
        let batcher = BatchLoader::new(mock_logic);

        let keys = vec!["A".to_string()];
        let result = batcher.run(&keys).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Something went wrong in the batch call!"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_batcher_is_thread_safe_and_runs_independently() -> eyre::Result<()> {
        let mock_logic = MockBatchCall::new(|keys: &[i32]| {
            let mut results = HashMap::new();
            for k in keys {
                results.insert(*k, format!("val_{}", k));
            }
            Ok(results)
        });
        let batcher = Arc::new(BatchLoader::new(mock_logic.clone()));

        let mut handles = vec![];
        for i in 0..5 {
            let batcher_clone = Arc::clone(&batcher);
            let handle = tokio::spawn(async move {
                let keys = vec![i, i + 1, i + 2];
                let delay_ms = rand::rng().random_range(5..20);
                sleep(Duration::from_millis(delay_ms)).await;

                batcher_clone.run(&keys).await
            });
            handles.push(handle);
        }

        let results = futures::future::try_join_all(handles).await?;

        assert_eq!(results.len(), 5);

        for (i, result) in results.iter().enumerate() {
            let res_map = result.as_ref().unwrap();
            let i = i as i32;
            assert_eq!(res_map.len(), 3);
            assert_eq!(res_map.get(&i), Some(&format!("val_{}", i)));
            assert_eq!(res_map.get(&(i + 1)), Some(&format!("val_{}", i + 1)));
            assert_eq!(res_map.get(&(i + 2)), Some(&format!("val_{}", i + 2)));
        }

        assert_eq!(mock_logic.get_calls().len(), 5);

        Ok(())
    }
}

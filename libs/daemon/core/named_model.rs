use native_db::{Models, ToInput};
use std::any::TypeId;
use std::collections::hash_map::Iter as HashMapIter;
use std::collections::HashMap;

/// A wrapper around `native_db::Models` that also stores the string names of the models.
///
/// This allows you to retrieve a user-friendly name for a model type, which is not
/// possible with the base `native_db::Models` struct.
#[derive(Debug, Default)]
pub struct NamedModels {
    /// The underlying `native_db::Models` instance.
    inner: Models,
    /// A map from the TypeId of a model to its user-defined string name.
    names: HashMap<String, (TypeId, native_db::Model)>,
}

#[allow(dead_code)]
impl NamedModels {
    /// Creates a new, empty `NamedModels` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Defines a model of type `T` with a given string name.
    ///
    /// This method calls the underlying `native_db::Models::define` and, if successful,
    /// stores the provided name for later retrieval.
    ///
    /// The generic bound `T: ToInput + 'static` is required to match the underlying
    /// library's `define` method and to use `TypeId::of`.
    pub fn define<T: ToInput + 'static>(&mut self, name: &str) -> eyre::Result<()> {
        let model = T::native_db_model();
        self.inner.define::<T>()?;
        self.names
            .insert(name.to_string(), (TypeId::of::<T>(), model));
        Ok(())
    }

    /// Returns an immutable reference to the inner `native_db::Models` instance.
    ///
    /// This is essential for passing the models collection to `native_db` functions
    /// like `native_db::Builder::new().create_in_memory()`.
    pub fn get_inner(&self) -> &Models {
        &self.inner
    }

    /// Consumes the wrapper and returns the inner `native_db::Models` instance.
    pub fn into_inner(self) -> Models {
        self.inner
    }

    /// Returns an iterator over the registered models, yielding their `TypeId` and name.
    ///
    /// The iterator yields tuples of `(&TypeId, &String)`.
    pub fn iter(&self) -> HashMapIter<'_, String, (TypeId, native_db::Model)> {
        self.names.iter()
    }

    /// Retrieves the name of a model by its type `T`.
    /// Returns `None` if the model has not been defined in this collection.
    pub fn get(&self, name: &str) -> Option<&(TypeId, native_db::Model)> {
        self.names.get(name)
    }
}

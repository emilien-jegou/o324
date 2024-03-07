use crate::utils::files;
use serde::{de::DeserializeOwned, Serialize};
use shaku::Interface;
use std::sync::Arc;

use super::{config_manager::IConfigManager, file_format_manager::IFileFormatManager};

pub trait IDocumentStorageManager<T: Send + Sync + Serialize + DeserializeOwned + Default + 'static>:
    Interface
{
    fn write(&self, document_name: &str, document: &T) -> eyre::Result<()>;

    /// Read a file and deserialize it to a document, if an error occured return the document
    /// default value.
    fn read_as_struct_with_default(&self, document_name: &str) -> eyre::Result<T>;

    /// Convert a string in the current storage file format to T
    #[allow(clippy::wrong_self_convention)]
    fn from_str(&self, content: &str) -> eyre::Result<T>;

    /// Convert a document to a string in the current storage file format (e.g. json)
    #[cfg(test)]
    fn _to_string(&self, document: &T) -> eyre::Result<String>;
}

pub struct DocumentStorageManager<S: Send + Sync + Serialize + DeserializeOwned + Default + 'static>
{
    file_format_manager: Arc<dyn IFileFormatManager>,
    config_manager: Arc<dyn IConfigManager>,
    phantom: std::marker::PhantomData<S>,
}

impl<
        M: shaku::Module
            + shaku::HasComponent<dyn IConfigManager>
            + shaku::HasComponent<dyn IFileFormatManager>,
        S: Send + Sync + Serialize + DeserializeOwned + Default + 'static,
    > shaku::Component<M> for DocumentStorageManager<S>
{
    type Interface = dyn IDocumentStorageManager<S>;
    type Parameters = ();
    fn build(
        context: &mut ::shaku::ModuleBuildContext<M>,
        _: Self::Parameters,
    ) -> Box<Self::Interface> {
        Box::new(Self {
            file_format_manager: M::build_component(context),
            config_manager: M::build_component(context),
            phantom: Default::default(),
        })
    }
}

impl<T: Send + Sync + Serialize + DeserializeOwned + Default + 'static> IDocumentStorageManager<T>
    for DocumentStorageManager<T>
{
    fn write(&self, document_name: &str, document: &T) -> eyre::Result<()> {
        let mut path = self
            .config_manager
            .get_repository_path()
            .join(document_name);

        files::add_file_extension(&mut path, self.file_format_manager.file_extension());
        let serialized = self.file_format_manager.serialize(document)?;
        std::fs::write(path, serialized.as_bytes())?;
        Ok(())
    }

    fn read_as_struct_with_default(&self, document_name: &str) -> eyre::Result<T> {
        let mut path = self
            .config_manager
            .get_repository_path()
            .join(document_name);

        files::add_file_extension(&mut path, self.file_format_manager.file_extension());
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            let data = self.file_format_manager.deserialize(&contents)?;
            Ok(serde_json::from_value(data)?)
        } else {
            Ok(T::default())
        }
    }

    fn from_str(&self, contents: &str) -> eyre::Result<T> {
        let data = self.file_format_manager.deserialize(contents)?;
        Ok(serde_json::from_value(data)?)
    }

    #[cfg(test)]
    fn _to_string(&self, document: &T) -> eyre::Result<String> {
        let serialized = self.file_format_manager.serialize(document)?;
        Ok(serialized)
    }
}

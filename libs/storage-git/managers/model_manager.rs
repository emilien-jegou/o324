use serde::{de::DeserializeOwned, Serialize};
use shaku::Interface;
use std::path::{Path, PathBuf};

pub trait IModelManager<T: Send + Sync + Serialize + DeserializeOwned + Default + 'static>:
    Interface
{
    fn save(&self, relative_path: &Path, data: &T) -> eyre::Result<()>;
    fn read_as_struct_with_default(&self, relative_path: &Path) -> eyre::Result<T>;
}

pub trait ModelType: Send + Sync + 'static {}

pub struct ModelManager<
    MT: ModelType,
    Model: Send + Sync + Serialize + DeserializeOwned + Default + 'static,
> {
    phantom: std::marker::PhantomData<(MT, Model)>,
    base_path: PathBuf,
}

pub struct ModelManagerParameters {
    pub base_path: PathBuf,
}

impl Default for ModelManagerParameters {
    fn default() -> Self {
        unreachable!("There is no default value for `ModelManager::base_path`");
    }
}

impl<
        M: ::shaku::Module,
        MT: ModelType,
        Model: Send + Sync + Serialize + DeserializeOwned + Default + 'static,
    > ::shaku::Component<M> for ModelManager<MT, Model>
where
    ModelManager<MT, Model>: IModelManager<Model>,
{
    type Interface = dyn IModelManager<Model>;
    type Parameters = ModelManagerParameters;
    fn build(
        _: &mut ::shaku::ModuleBuildContext<M>,
        params: Self::Parameters,
    ) -> Box<Self::Interface> {
        Box::new(Self {
            base_path: params.base_path,
            phantom: Default::default(),
        })
    }
}

pub struct JsonModel;

impl ModelType for JsonModel {}

impl<T: Send + Sync + Serialize + DeserializeOwned + Default + 'static> IModelManager<T>
    for ModelManager<JsonModel, T>
{
    fn save(&self, relative_path: &Path, data: &T) -> eyre::Result<()> {
        let path = self.base_path.join(relative_path);
        let serialized = serde_json::to_string_pretty(data)?;
        std::fs::write(path, serialized.as_bytes())?;
        Ok(())
    }

    fn read_as_struct_with_default(&self, relative_path: &Path) -> eyre::Result<T> {
        let path = self.base_path.join(relative_path);
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&contents)?)
        } else {
            Ok(T::default())
        }
    }
}

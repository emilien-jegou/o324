use shaku::Interface;

pub trait IFileFormatManager: Interface {
    fn deserialize(&self, data: &str) -> eyre::Result<serde_json::Value>;
    fn serialize(&self, data: &dyn erased_serde::Serialize) -> eyre::Result<String>;
    fn file_extension(&self) -> &'static str;
}

pub trait FileFormatType: Send + Sync + 'static {}

pub struct FileFormatManager<ST: FileFormatType> {
    phantom: std::marker::PhantomData<ST>,
}

#[derive(Default, Clone)]
pub struct FileFormatManagerParameters {}

impl<M: ::shaku::Module, ST: FileFormatType> ::shaku::Component<M> for FileFormatManager<ST>
where
    FileFormatManager<ST>: IFileFormatManager,
{
    type Interface = dyn IFileFormatManager;
    type Parameters = FileFormatManagerParameters;
    fn build(_: &mut ::shaku::ModuleBuildContext<M>, _: Self::Parameters) -> Box<Self::Interface> {
        Box::new(Self {
            phantom: Default::default(),
        })
    }
}

pub struct JsonFileFormat;

impl FileFormatType for JsonFileFormat {}

impl IFileFormatManager for FileFormatManager<JsonFileFormat> {
    fn deserialize(&self, data: &str) -> eyre::Result<serde_json::Value> {
        let data: serde_json::Value = serde_json::from_str(data)?;
        Ok(data)
    }

    fn serialize(&self, data: &dyn erased_serde::Serialize) -> eyre::Result<String> {
        Ok(serde_json::to_string_pretty(data)?)
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}

pub struct TomlFileFormat;

impl FileFormatType for TomlFileFormat {}

impl IFileFormatManager for FileFormatManager<TomlFileFormat> {
    fn deserialize(&self, data: &str) -> eyre::Result<serde_json::Value> {
        let data: serde_json::Value = toml::from_str(data)?;
        Ok(data)
    }

    fn serialize(&self, data: &dyn erased_serde::Serialize) -> eyre::Result<String> {
        Ok(toml::to_string(data)?)
    }

    fn file_extension(&self) -> &'static str {
        "toml"
    }
}

pub struct YamlFileFormat;

impl FileFormatType for YamlFileFormat {}

impl IFileFormatManager for FileFormatManager<YamlFileFormat> {
    fn deserialize(&self, data: &str) -> eyre::Result<serde_json::Value> {
        let data: serde_json::Value = serde_yaml::from_str(data)?;
        Ok(data)
    }

    fn serialize(&self, data: &dyn erased_serde::Serialize) -> eyre::Result<String> {
        Ok(serde_yaml::to_string(data)?)
    }

    fn file_extension(&self) -> &'static str {
        "yaml"
    }
}

use std::sync::Arc;

use derive_more::Deref;

#[derive(Deref, Clone)]
#[deref(forward)]
pub struct DocumentParser(Arc<dyn IDocumentParser>);

pub trait IDocumentParser: Send + Sync {
    fn deserialize(&self, data: &str) -> eyre::Result<serde_json::Value>;
    fn serialize(&self, data: &dyn erased_serde::Serialize) -> eyre::Result<String>;
    fn file_extension(&self) -> &'static str;
}

macro_rules! impl_file_parser {
    ($FormatType:ident, $deserialize:expr, $serialize:expr, $file_extension:expr) => {
        pub struct $FormatType;
        impl $FormatType {
            pub fn get() -> DocumentParser {
                DocumentParser(Arc::new($FormatType {}))
            }
        }
        impl IDocumentParser for $FormatType {
            fn deserialize(&self, data: &str) -> eyre::Result<serde_json::Value> {
                let data: serde_json::Value = $deserialize(data)?;
                Ok(data)
            }

            fn serialize(&self, data: &dyn erased_serde::Serialize) -> eyre::Result<String> {
                Ok($serialize(data)?)
            }

            fn file_extension(&self) -> &'static str {
                $file_extension
            }
        }
    };
}

impl_file_parser!(
    JsonParser,
    serde_json::from_str,
    serde_json::to_string_pretty,
    "json"
);

impl_file_parser!(TomlParser, toml::from_str, toml::to_string, "toml");

impl_file_parser!(
    YamlParser,
    serde_yaml::from_str,
    serde_yaml::to_string,
    "yaml"
);


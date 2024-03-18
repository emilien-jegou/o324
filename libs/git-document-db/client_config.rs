use crate::document_parser::DocumentParser;
use std::path::PathBuf;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct ClientConfig {
    pub document_parser: DocumentParser,
    pub repository_path: PathBuf,
    pub remote_origin_url: String,
}

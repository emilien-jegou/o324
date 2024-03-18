use std::path::{Path, PathBuf};

use lazy_regex::Regex;

pub fn add_file_extension(path_buf: &mut PathBuf, extension: &str) {
    // Check if the current extension is the same as the provided one, if not, append the provided extension.
    match path_buf.extension() {
        Some(current_extension) if current_extension == extension => {}
        _ => {
            if let Some(stem) = path_buf.file_stem() {
                let new_name = format!("{}.{}", stem.to_string_lossy(), extension);
                path_buf.set_file_name(new_name);
            }
        }
    }
}

pub fn find_matching_files(path: &Path, re: &Regex) -> eyre::Result<Vec<String>> {
    let mut matchs: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
            if re.is_match(file_name) {
                matchs.push(file_name.to_string());
            }
        }
    }

    Ok(matchs)
}

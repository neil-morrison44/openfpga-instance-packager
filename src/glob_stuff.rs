use glob::glob;

use std::error;

use std::path::PathBuf;

pub(crate) fn get_glob_paths(glob_str: &str) -> Result<Vec<PathBuf>, Box<dyn error::Error>> {
    Ok(glob(&glob_str)?
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|p| {
            !p.file_name()
                .and_then(|f| f.to_str())
                .unwrap()
                .starts_with(".")
        })
        .collect())
}

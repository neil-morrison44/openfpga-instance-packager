use std::fs::{create_dir_all, File};
use tempfile::tempdir;

pub(crate) fn make_fake_files(files: Vec<&str>) -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    for file in files {
        let full_path = path.join(file);
        create_dir_all(&full_path.parent().unwrap()).unwrap();
        File::create(full_path).unwrap();
    }

    temp_dir
}

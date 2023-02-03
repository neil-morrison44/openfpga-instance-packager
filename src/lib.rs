use glob::glob;
use std::io::ErrorKind;
use std::path::Path;
use std::{error, fs};
use std::{io, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

pub static PACKAGER_NAME: &str = "instance-packager.json";

pub fn find_cores_with_package_json(
    root_path: &PathBuf,
) -> Result<Vec<String>, Box<dyn error::Error>> {
    let cores_path = root_path.join("Cores");
    if !cores_path.exists() {
        return Err(io::Error::new(ErrorKind::NotFound, "Unable to find Cores/ folder").into());
    }
    let paths = fs::read_dir(cores_path).unwrap();
    let mut found_cores: Vec<String> = vec![];

    for path in paths.filter_map(|x| x.ok()) {
        let core_path = path.path();
        if core_path.join(PACKAGER_NAME).exists() {
            let core_name = core_path.file_name().and_then(|f| f.to_str()).unwrap();
            found_cores.push(String::from(core_name));
        }
    }
    Ok(found_cores)
}

mod serde_structs;

pub fn build_jsons_for_core(
    root_path: &PathBuf,
    core_name: &str,
) -> Result<(), Box<dyn error::Error>> {
    let file_name = root_path.join("Cores").join(core_name).join(PACKAGER_NAME);
    let data =
        fs::read_to_string(&file_name).expect(&format!("Unable to read file {:?}", &file_name));
    let instance_packager: serde_structs::InstancePackager = serde_json::from_str(&data).unwrap();

    let asset_folder = root_path
        .join("Assets")
        .join(instance_packager.platform_id)
        .join("common");

    fn is_hidden(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
    }

    let walker = WalkDir::new(&asset_folder).into_iter();
    for entry in walker
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_dir() {
            println!("{}", path.display());

            //TODO: merge in overrides based on path.file_name()
            let matches = check_if_dir_matches_slots(&instance_packager.data_slots, &path).unwrap();
            dbg!(matches);

            // if matches, build JSON
        }
    }

    Ok(())
}

fn check_if_dir_matches_slots(
    data_slots: &Vec<serde_structs::InstancePackagerDataSlot>,
    path: &Path,
) -> Result<bool, Box<dyn error::Error>> {
    for slot in data_slots {
        if slot.required {
            let partial_glob = slot.filename.clone();
            let full_glob = String::from(path.join(partial_glob).to_str().unwrap());

            let paths: Vec<PathBuf> = glob(&full_glob)
                .unwrap()
                .into_iter()
                .filter_map(|f| f.ok())
                .collect();

            if paths.len() == 0
                || (matches!(slot.sort, serde_structs::Sort::Single) && paths.len() > 1)
            {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    fn make_fake_files(files: Vec<&str>) -> tempfile::TempDir {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        for file in files {
            File::create(path.join(file)).unwrap();
        }

        temp_dir
    }

    #[test]
    fn test_check_if_dir_matches_slots_bin_and_cue() {
        let temp_dir = make_fake_files(vec![
            "Something.bin",
            "Something (1).cue",
            "Something (2).cue",
        ]);
        let path = temp_dir.path();

        let data_slots = vec![
            serde_structs::InstancePackagerDataSlot {
                id: 101,
                filename: String::from("*.bin"),
                required: true,
                sort: serde_structs::Sort::Single,
            },
            serde_structs::InstancePackagerDataSlot {
                id: 102,
                filename: String::from("*.cue"),
                required: true,
                sort: serde_structs::Sort::Ascending,
            },
        ];

        let result = check_if_dir_matches_slots(&data_slots, &path);

        assert!(matches!(result, Ok(true)));
    }

    #[test]
    fn test_check_if_dir_matches_slots_multi_bin() {
        let temp_dir = make_fake_files(vec![
            "Something.bin",
            "Something_else.bin",
            "Something (1).cue",
            "Something (2).cue",
        ]);
        let path = temp_dir.path();

        let data_slots = vec![
            serde_structs::InstancePackagerDataSlot {
                id: 101,
                filename: String::from("*.bin"),
                required: true,
                sort: serde_structs::Sort::Single,
            },
            serde_structs::InstancePackagerDataSlot {
                id: 102,
                filename: String::from("*.cue"),
                required: true,
                sort: serde_structs::Sort::Ascending,
            },
        ];

        let result = check_if_dir_matches_slots(&data_slots, &path);

        assert!(matches!(result, Ok(false)));
    }

    #[test]
    fn test_check_if_dir_matches_slots_missing_bin_and_cue() {
        let temp_dir = make_fake_files(vec![
            "Something.pin",
            "Something (1).bue",
            "Something (2).bue",
        ]);
        let path = temp_dir.path();

        let data_slots = vec![
            serde_structs::InstancePackagerDataSlot {
                id: 101,
                filename: String::from("*.bin"),
                required: true,
                sort: serde_structs::Sort::Single,
            },
            serde_structs::InstancePackagerDataSlot {
                id: 102,
                filename: String::from("*.cue"),
                required: true,
                sort: serde_structs::Sort::Ascending,
            },
        ];

        let result = check_if_dir_matches_slots(&data_slots, &path);

        assert!(matches!(result, Ok(false)));
    }
}

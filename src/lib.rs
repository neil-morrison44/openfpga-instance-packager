use glob::glob;
use std::io::ErrorKind;
use std::path::Path;
use std::{error, fs};
use std::{io, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
struct InstancePackagerDataSlot {
    id: u32,
    filename: String,
    sort: Sort,
    required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Sort {
    Single,
    Ascending,
    Descending,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstancePackagerOverrides {
    data_slots: Vec<InstancePackagerDataSlot>,
    filename: Option<String>,
    // don't need to do memory_writes etc since those'll come in from a merge
}

#[derive(Debug, Serialize, Deserialize)]
struct InstancePackager {
    platform_id: String,
    data_slots: Vec<InstancePackagerDataSlot>,
    overrides: Option<InstancePackagerOverrides>,
}

pub fn build_jsons_for_core(
    root_path: &PathBuf,
    core_name: &str,
) -> Result<(), Box<dyn error::Error>> {
    let file_name = root_path.join("Cores").join(core_name).join(PACKAGER_NAME);
    let data = fs::read_to_string(file_name).expect("Unable to read file");
    let instance_packager: InstancePackager = serde_json::from_str(&data).unwrap();

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
    data_slots: &Vec<InstancePackagerDataSlot>,
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

            if paths.len() == 0 || (matches!(slot.sort, Sort::Single) && paths.len() > 1) {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

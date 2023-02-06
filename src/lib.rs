use std::io::ErrorKind;
use std::path::Path;
use std::{error, fs};
use std::{io, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

mod glob_stuff;

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
    on_json: impl Fn(&str) -> (),
    on_warn: impl Fn(&str, &str) -> (),
) -> Result<(), Box<dyn error::Error>> {
    let file_name = root_path.join("Cores").join(core_name).join(PACKAGER_NAME);
    let data =
        fs::read_to_string(&file_name).expect(&format!("Unable to read file {:?}", &file_name));
    let instance_packager: serde_structs::InstancePackager = serde_json::from_str(&data).unwrap();

    let asset_folder = root_path
        .join("Assets")
        .join(&instance_packager.platform_id)
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
            let folder_name = path.file_name().and_then(|f| f.to_str()).unwrap();
            let slots = instance_packager.get_slots(&folder_name);
            let matches = check_if_dir_matches_slots(&slots, &path).unwrap();

            if !matches {
                continue;
            }

            let mut instance_json = build_json(&path, &instance_packager)?;
            let output_path = root_path.join(&instance_packager.output);

            instance_json.instance.data_path =
                format!("{}/", path.strip_prefix(&asset_folder)?.to_str().unwrap());

            let file_name = instance_packager.get_filename(&path)?;
            let file_name = format!("{}.json", file_name);

            fs::create_dir_all(&output_path)?;
            let file_path = output_path.join(&file_name);

            if let Some(slot_limit) = &instance_packager.slot_limit {
                if instance_json.instance.data_slots.len() > slot_limit.count {
                    on_warn(
                        &file_path.strip_prefix(root_path)?.to_str().unwrap(),
                        &slot_limit.message,
                    );
                    continue;
                }
            }

            std::fs::write(
                &file_path,
                serde_json::to_string_pretty(&instance_json).unwrap(),
            )?;
            on_json(&file_path.strip_prefix(root_path)?.to_str().unwrap());
        }
    }

    Ok(())
}

fn build_json(
    folder_path: &Path,
    instance_packager: &serde_structs::InstancePackager,
) -> Result<serde_structs::InstanceJSON, Box<dyn error::Error>> {
    let folder_name = folder_path.file_name().and_then(|f| f.to_str()).unwrap();
    let slots = &instance_packager.get_slots(folder_name);
    let mut instance_json = serde_structs::InstanceJSON::new();

    for slot in slots {
        let partial_glob = slot.filename.clone();
        let full_glob = String::from(folder_path.join(partial_glob).to_str().unwrap());
        let paths: Vec<PathBuf> = glob_stuff::get_glob_paths(&full_glob)?;

        let sorted_paths = match slot.sort {
            serde_structs::Sort::Single => paths,
            serde_structs::Sort::Ascending => paths,
            serde_structs::Sort::Descending => paths.into_iter().rev().collect(),
        };

        for (index, path) in sorted_paths.iter().enumerate() {
            instance_json
                .instance
                .data_slots
                .push(serde_structs::SlotsCoresAndWrites::DataSlot {
                    id: (slot.id + index).into(),
                    filename: String::from(
                        path.strip_prefix(folder_path).unwrap().to_str().unwrap(),
                    ),
                })
        }
    }

    instance_json.instance.memory_writes = instance_packager.get_memory_writes(&folder_name);
    instance_json.instance.core_select = instance_packager.get_core_select(&folder_name);
    Ok(instance_json)
}

fn check_if_dir_matches_slots(
    data_slots: &Vec<serde_structs::InstancePackagerDataSlot>,
    path: &Path,
) -> Result<bool, Box<dyn error::Error>> {
    for slot in data_slots {
        if slot.required {
            let partial_glob = slot.filename.clone();
            let full_glob = String::from(path.join(partial_glob).to_str().unwrap());
            let paths: Vec<PathBuf> = glob_stuff::get_glob_paths(&full_glob)?;

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
    mod test_helpers;

    #[test]
    fn test_find_cores_with_package_json_some() {
        let temp_dir = test_helpers::make_fake_files(vec![
            "Cores/someone.core/core.json",
            "Cores/someone_else.core/core.json",
            "Cores/someone_else.core/core.json",
            "Cores/someone_else.core/instance-packager.json",
        ]);
        let path = temp_dir.path();

        let results = find_cores_with_package_json(&PathBuf::from(path)).unwrap();

        assert!(results.len() == 1);
        assert!(results[0] == "someone_else.core");
    }

    #[test]
    fn test_find_cores_with_package_json_none() {
        let temp_dir = test_helpers::make_fake_files(vec![
            "Cores/someone.core/core.json",
            "Cores/someone_else.core/core.json",
            "Cores/someone_else.core/core.json",
        ]);
        let path = temp_dir.path();
        let results = find_cores_with_package_json(&PathBuf::from(path)).unwrap();
        assert!(results.len() == 0);
    }

    #[test]
    fn test_check_if_dir_matches_slots_bin_and_cue() {
        let temp_dir = test_helpers::make_fake_files(vec![
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
                as_filename: None,
            },
            serde_structs::InstancePackagerDataSlot {
                id: 102,
                filename: String::from("*.cue"),
                required: true,
                sort: serde_structs::Sort::Ascending,
                as_filename: None,
            },
        ];

        let result = check_if_dir_matches_slots(&data_slots, &path);

        assert!(matches!(result, Ok(true)));
    }

    #[test]
    fn test_check_if_dir_matches_slots_multi_bin() {
        let temp_dir = test_helpers::make_fake_files(vec![
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
                as_filename: None,
            },
            serde_structs::InstancePackagerDataSlot {
                id: 102,
                filename: String::from("*.cue"),
                required: true,
                sort: serde_structs::Sort::Ascending,
                as_filename: None,
            },
        ];

        let result = check_if_dir_matches_slots(&data_slots, &path);

        assert!(matches!(result, Ok(false)));
    }

    #[test]
    fn test_check_if_dir_matches_slots_missing_bin_and_cue() {
        let temp_dir = test_helpers::make_fake_files(vec![
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
                as_filename: None,
            },
            serde_structs::InstancePackagerDataSlot {
                id: 102,
                filename: String::from("*.cue"),
                required: true,
                sort: serde_structs::Sort::Ascending,
                as_filename: None,
            },
        ];

        let result = check_if_dir_matches_slots(&data_slots, &path);

        assert!(matches!(result, Ok(false)));
    }
}

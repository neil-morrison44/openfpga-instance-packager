use std::{collections::HashMap, path::Path};

use glob::glob;
use serde::{Deserialize, Serialize};

use crate::glob_stuff;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct InstancePackagerDataSlot {
    pub(crate) id: usize,
    pub(crate) filename: String,
    pub(crate) sort: Sort,
    pub(crate) required: bool,
    pub(crate) as_filename: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Sort {
    Single,
    Ascending,
    Descending,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InstancePackagerOverrides {
    pub(crate) data_slots: Option<Vec<InstancePackagerDataSlot>>,
    pub(crate) filename: Option<String>,
    // don't need to do memory_writes etc since those'll come in from a merge
    pub(crate) memory_writes: Option<Vec<SlotsCoresAndWrites>>,
    pub(crate) core_select: Option<SlotsCoresAndWrites>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InstancePackager {
    pub(crate) output: String,
    pub(crate) data_slots: Vec<InstancePackagerDataSlot>,
    pub(crate) overrides: Option<HashMap<String, InstancePackagerOverrides>>,
    pub(crate) platform_id: String,
    pub(crate) memory_writes: Option<Vec<SlotsCoresAndWrites>>,
    pub(crate) core_select: Option<SlotsCoresAndWrites>,
}

impl InstancePackager {
    pub fn get_slots(&self, folder_name: &str) -> Vec<InstancePackagerDataSlot> {
        if let Some(overides_map) = &self.overrides {
            if let Some(data_slots) = overides_map
                .get(folder_name)
                .and_then(|o| o.data_slots.to_owned())
            {
                return data_slots.clone();
            }
        }
        self.data_slots.clone()
    }

    pub fn get_memory_writes(&self, folder_name: &str) -> Vec<SlotsCoresAndWrites> {
        if let Some(overides_map) = &self.overrides {
            if let Some(memory_writes) = overides_map
                .get(folder_name)
                .and_then(|m| m.memory_writes.to_owned())
            {
                return memory_writes.clone();
            }
        }

        if let Some(memory_writes) = &self.memory_writes {
            return memory_writes.clone();
        } else {
            return vec![];
        }
    }

    pub fn get_core_select(&self, folder_name: &str) -> Option<SlotsCoresAndWrites> {
        if let Some(overides_map) = &self.overrides {
            if let Some(core_select) = overides_map
                .get(folder_name)
                .and_then(|m| m.core_select.to_owned())
            {
                return Some(core_select);
            }
        }
        return self.core_select.to_owned();
    }

    pub fn get_filename(&self, folder_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let folder_name = folder_path.file_name().unwrap().to_str().unwrap();

        if let Some(overides_map) = &self.overrides {
            if let Some(filename) = overides_map
                .get(folder_name)
                .and_then(|m| m.filename.to_owned())
            {
                return Ok(String::from(filename));
            }
        }

        let as_filename_slots: Vec<InstancePackagerDataSlot> = self
            .data_slots
            .clone()
            .into_iter()
            .filter(|s| s.as_filename == Some(true))
            .collect();

        for slot in as_filename_slots {
            let full_glob = String::from(folder_path.join(slot.filename).to_str().unwrap());
            let paths = glob_stuff::get_glob_paths(&full_glob)?;

            for path in paths {
                return Ok(String::from(path.file_stem().unwrap().to_str().unwrap()));
            }
        }

        return Ok(String::from(folder_name));
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InstanceJSON {
    pub instance: InstanceJSONInstance,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InstanceJSONInstance {
    magic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core_select: Option<SlotsCoresAndWrites>,
    pub data_path: String,
    pub data_slots: Vec<SlotsCoresAndWrites>,
    pub memory_writes: Vec<SlotsCoresAndWrites>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub(crate) enum SlotsCoresAndWrites {
    CoreSelect { id: usize, select: bool },
    DataSlot { id: usize, filename: String },
    MemoryWriteNum { address: usize, data: usize },
    MemoryWriteStr { address: String, data: String },
}

impl InstanceJSON {
    pub fn new() -> InstanceJSON {
        InstanceJSON {
            instance: InstanceJSONInstance {
                magic: String::from("APF_VER_1"),
                data_path: String::from(""),
                data_slots: vec![],
                memory_writes: vec![],
                core_select: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{create_dir_all, File},
        path::{Path, PathBuf},
    };

    use crate::serde_structs::SlotsCoresAndWrites;

    use super::InstancePackager;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn test_instance_packager_get_slots() {
        let json_data = json!({
            "output": "some/folder/somewhere/",
            "platform_id": "abc",
            "data_slots": [
                {
                    "id": 22,
                    "filename": "file_A.bin",
                    "sort": "single",
                    "required": true
                }
            ],
            "overrides": {
                "overrider": {
                    "data_slots": [
                        {
                            "id": 23,
                            "filename": "file_B.bin",
                            "sort": "single",
                            "required": true
                        }
                    ]
                }
            }
        });

        let instance_packager: InstancePackager = serde_json::from_value(json_data).unwrap();

        let data_slots = instance_packager.get_slots("overrider");
        assert_eq!(data_slots.len(), 1);
        assert_eq!(data_slots[0].filename, "file_B.bin");

        let data_slots = instance_packager.get_slots("non_overrider");
        assert_eq!(data_slots.len(), 1);
        assert_eq!(data_slots[0].filename, "file_A.bin");
    }

    #[test]
    fn test_instance_packager_get_memory_writes() {
        let json_data = json!({
            "output": "some/folder/somewhere/",
            "platform_id": "abc",
            "data_slots": [
                {
                    "id": 22,
                    "filename": "file_A.bin",
                    "sort": "single",
                    "required": true
                }
            ],
            "memory_writes": [
                {
                    "data": "0x123",
                    "address": "0x1345",
                }
            ],
            "overrides": {
                "overrider": {
                    "memory_writes": [
                        {
                            "data": "0x9876",
                            "address": "0x987654",
                        },
                        {
                            "data": "0x9876",
                            "address": "0x987654",
                        }
                    ]
                }
            }
        });

        let instance_packager: InstancePackager = serde_json::from_value(json_data).unwrap();

        let expected = SlotsCoresAndWrites::MemoryWriteStr {
            data: "0x123".to_string(),
            address: "0x1345".to_string(),
        };

        let memory_writes = instance_packager.get_memory_writes("non_overrider");
        assert_eq!(memory_writes.len(), 1);
        assert_eq!(memory_writes[0], expected);

        let expected = SlotsCoresAndWrites::MemoryWriteStr {
            data: "0x9876".to_string(),
            address: "0x987654".to_string(),
        };

        let memory_writes = instance_packager.get_memory_writes("overrider");
        assert_eq!(memory_writes.len(), 2);
        assert_eq!(memory_writes[0], expected);
    }

    #[test]
    fn test_instance_packager_get_core_select() {
        let json_data = json!({
            "output": "some/folder/somewhere/",
            "platform_id": "abc",
            "core_select": {
                "id": 123,
                "select": true
            },
            "data_slots": [
            ],
            "overrides": {
                "overrider": {
                    "core_select": {
                        "id": 456,
                        "select": false
                    }
                }
            }
        });

        let instance_packager: InstancePackager = serde_json::from_value(json_data).unwrap();

        let expected = SlotsCoresAndWrites::CoreSelect {
            id: 123,
            select: true,
        };

        let core_select = instance_packager.get_core_select("non_overrider");
        assert_eq!(core_select, Some(expected));

        let expected = SlotsCoresAndWrites::CoreSelect {
            id: 456,
            select: false,
        };

        let core_select = instance_packager.get_core_select("overrider");
        assert_eq!(core_select, Some(expected));
    }

    #[test]
    fn test_instance_packager_get_file_name() {
        let json_data = json!({
            "output": "some/folder/somewhere/",
            "platform_id": "abc",
            "core_select": {
                "id": 123,
                "select": true
            },
            "data_slots": [
            ],
            "overrides": {
                "overrider": {
                    "filename": "overridden_file_name"
                }
            }
        });

        let instance_packager: InstancePackager = serde_json::from_value(json_data).unwrap();

        let file_name = instance_packager
            .get_filename(&PathBuf::from("fake/folder/game_name"))
            .unwrap();
        assert_eq!(file_name, String::from("game_name"));

        let file_name = instance_packager
            .get_filename(&PathBuf::from("fake/folder/overridden_file_name"))
            .unwrap();
        assert_eq!(file_name, String::from("overridden_file_name"));
    }

    #[test]
    fn test_instance_packager_get_file_name_as_file_name() {
        let json_data = json!({
            "output": "some/folder/somewhere/",
            "platform_id": "abc",
            "core_select": {
                "id": 123,
                "select": true
            },
            "data_slots": [
                {
                    "id": 123,
                    "filename": "*.cue",
                    "as_filename": true,
                    "sort": "single",
                    "required": true
                }
            ],
            "overrides": {
                "overrider": {
                    "filename": "overridden_file_name"
                }
            }
        });

        // TODO use the one in the `test_helpers` for this (need to move it somewhere)

        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        let files = vec![
            "fake/folder/game_name/cue_file_name.cue",
            "fake/folder/overrider/cue_file_name.cue",
        ];

        for file in files {
            let full_path = path.join(file);
            create_dir_all(&full_path.parent().unwrap()).unwrap();
            File::create(full_path).unwrap();
        }

        let instance_packager: InstancePackager = serde_json::from_value(json_data).unwrap();

        let file_name = instance_packager
            .get_filename(&path.join("fake/folder/game_name"))
            .unwrap();
        assert_eq!(file_name, String::from("cue_file_name"));

        let file_name = instance_packager
            .get_filename(&path.join("fake/folder/overrider"))
            .unwrap();
        assert_eq!(file_name, String::from("overridden_file_name"));
    }
}

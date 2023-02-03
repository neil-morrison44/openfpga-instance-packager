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
    pub(crate) data_slots: Vec<InstancePackagerDataSlot>,
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
            if let Some(overrides) = overides_map.get(folder_name) {
                return overrides.data_slots.clone();
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

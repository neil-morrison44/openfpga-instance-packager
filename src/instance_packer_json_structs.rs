use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InstancePackagerDataSlot {
    pub(crate) id: u32,
    pub(crate) filename: String,
    pub(crate) sort: Sort,
    pub(crate) required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
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
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InstancePackager {
    pub(crate) data_slots: Vec<InstancePackagerDataSlot>,
    pub(crate) overrides: Option<InstancePackagerOverrides>,
    pub(crate) platform_id: String,
}

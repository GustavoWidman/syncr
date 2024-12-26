use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SyncConfigTOML {
    config: SyncConfigInner,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncConfigInner {
    debounce: u64,
    ignored: Vec<IgnoredPath>,
}

impl Default for SyncConfigInner {
    fn default() -> Self {
        Self {
            debounce: 1000,
            ignored: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct IgnoredPath {
    path: PathBuf,
}

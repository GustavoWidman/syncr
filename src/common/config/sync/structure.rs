use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SyncConfigTOML {
    config: SyncConfigInner,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "mode")]
pub enum IgnoreMode {
    #[serde(rename = "whitelist")]
    Whitelist { whitelist: Vec<Directory> },
    #[serde(rename = "blacklist")]
    Blacklist { blacklist: Vec<Directory> },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncConfigInner {
    debounce: u64,

    #[serde(flatten)]
    mode: IgnoreMode,
}

impl Default for SyncConfigInner {
    fn default() -> Self {
        Self {
            debounce: 1000,
            mode: IgnoreMode::Blacklist {
                blacklist: Vec::new(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Directory {
    path: PathBuf,
}

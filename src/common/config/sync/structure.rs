use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Default, Clone, Eq, PartialEq)]
pub struct SyncConfigTOML {
    pub config: SyncConfigInner,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct SyncConfigInner {
    pub debounce: u64,
    pub ignore_symlinks: bool,
    pub ignore_hidden: bool,
    pub max_depth: i32,
    pub syncr_id: String,

    pub patterns: Option<Vec<Pattern>>,
}

impl Default for SyncConfigInner {
    fn default() -> Self {
        Self {
            debounce: 60000,
            patterns: Some(Vec::from([Pattern {
                pattern: "**/*".to_owned(),
            }])),
            syncr_id: Uuid::new_v4().to_string(),
            ignore_symlinks: true,
            ignore_hidden: false,
            max_depth: -1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct Pattern {
    pub pattern: String,
}

impl Into<String> for Pattern {
    fn into(self) -> String {
        self.pattern
    }
}

impl<'a> Into<&'a str> for &'a Pattern {
    fn into(self) -> &'a str {
        self.pattern.as_str()
    }
}

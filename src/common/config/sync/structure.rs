use glob::Pattern as GlobPattern;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SyncConfigTOML {
    pub config: SyncConfigInner,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "mode")]
pub enum IgnoreMode {
    #[serde(rename = "whitelist")]
    Whitelist { whitelist: Vec<Pattern> },
    #[serde(rename = "blacklist")]
    Blacklist { blacklist: Vec<Pattern> },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncConfigInner {
    pub debounce: u64,

    #[serde(flatten)]
    pub mode: IgnoreMode,
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

#[derive(Debug, Default)]
pub struct Pattern {
    pub pattern: GlobPattern,
}

impl Serialize for Pattern {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.pattern.as_str())
    }
}

impl<'de> Deserialize<'de> for Pattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let pattern = String::deserialize(deserializer)?;
        Ok(Self {
            pattern: GlobPattern::new(&pattern).map_err(serde::de::Error::custom)?,
        })
    }
}

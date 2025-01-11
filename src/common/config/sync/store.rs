use anyhow::bail;

use super::structure::{SyncConfigInner, SyncConfigTOML};
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Debug)]
pub struct SyncConfig {
    pub path: PathBuf,
    cached: SyncConfigTOML,
}

impl SyncConfig {
    pub fn read(path: PathBuf) -> Result<Self, anyhow::Error> {
        // resolve .syncr from given dir if its a dir, if its not then we leave it.
        let path = match path.is_dir() {
            true => path.join(".syncr"),
            false => path,
        };

        if !path.exists() {
            return Ok(Self::new(path)?);
        }

        if !path.is_file() {
            bail!(
                "Given path exists and is not a file... either change the path or delete the file."
            );
        }

        let config_str = std::fs::read_to_string(&path)?;

        Ok(Self {
            path,
            cached: toml::from_str(&config_str)?,
        })
    }

    pub fn as_ref(&self) -> &SyncConfigTOML {
        &self.cached
    }

    pub fn as_static_ref(&'static self) -> &'static SyncConfigTOML {
        &self.cached
    }

    pub fn update(&mut self) -> bool {
        let new = Self::read(self.path.clone()).unwrap();

        match self.cached.config == new.cached.config {
            true => false,
            false => {
                self.cached = new.cached;
                true
            }
        }
    }

    pub fn as_mut_ref(&mut self) -> &mut SyncConfigTOML {
        &mut self.cached
    }

    fn new(path: PathBuf) -> Result<Self, anyhow::Error> {
        std::fs::create_dir_all(path.parent().unwrap())?;

        let config = Self {
            path,
            cached: SyncConfigTOML::default(),
        };

        config.save()?;

        Ok(config)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        std::fs::write(&self.path, toml::to_string(&self.cached)?)?;

        Ok(())
    }

    pub async fn async_save(&self) -> Result<(), anyhow::Error> {
        tokio::fs::write(&self.path, toml::to_string(&self.cached)?).await?;

        Ok(())
    }
}

impl Deref for SyncConfig {
    type Target = SyncConfigInner;

    fn deref(&self) -> &Self::Target {
        &self.cached.config
    }
}

impl DerefMut for SyncConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cached.config
    }
}

impl Clone for SyncConfig {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            cached: self.cached.clone(),
        }
    }
}

impl PartialEq for SyncConfig {
    fn eq(&self, other: &Self) -> bool {
        self.cached.config == other.cached.config
    }
}
impl Eq for SyncConfig {}

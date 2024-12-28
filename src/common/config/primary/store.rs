use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use super::structure::{ClientConfig, ConfigTOML, ModeConfig, ServerConfig};
use anyhow::bail;

#[derive(Debug)]
pub struct Config {
    path: PathBuf,
    cached: ConfigTOML,
}

impl Config {
    pub fn read(path: Option<PathBuf>) -> Result<Self, anyhow::Error> {
        let path = path
            .or_else(|| dirs::home_dir().map(|dir| dir.join(".syncr").join("config.toml")))
            .ok_or(anyhow::anyhow!(
                "Unable to extract config path, default home directory not found."
            ))?;

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

    pub fn as_ref(&self) -> &ConfigTOML {
        &self.cached
    }

    pub fn as_mut_ref(&mut self) -> &mut ConfigTOML {
        &mut self.cached
    }

    fn new(path: PathBuf) -> Result<Self, anyhow::Error> {
        std::fs::create_dir_all(path.parent().unwrap())?;

        let config = Self {
            path,
            cached: ConfigTOML::default(),
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

    pub fn as_client(&self) -> Result<ClientOnlyConfigRef, anyhow::Error> {
        match self.cached.mode_config {
            ModeConfig::Client { .. } => Ok(ClientOnlyConfigRef {
                inner: &self.cached,
            }),
            _ => bail!("Config is not in client mode"),
        }
    }

    pub fn as_server(&self) -> Result<ServerOnlyConfigRef, anyhow::Error> {
        match self.cached.mode_config {
            ModeConfig::Server { .. } => Ok(ServerOnlyConfigRef {
                inner: &self.cached,
            }),
            _ => bail!("Config is not in server mode"),
        }
    }
}

impl Deref for Config {
    type Target = ConfigTOML;

    fn deref(&self) -> &Self::Target {
        &self.cached
    }
}

impl DerefMut for Config {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cached
    }
}

pub struct ClientOnlyConfigRef<'a> {
    inner: &'a ConfigTOML,
}

pub struct ServerOnlyConfigRef<'a> {
    inner: &'a ConfigTOML,
}

impl<'a> ClientOnlyConfigRef<'a> {
    pub fn client(&self) -> &ClientConfig {
        // We can safely unwrap here because we verified the type in as_client()
        match &self.inner.mode_config {
            ModeConfig::Client { client } => client,
            _ => unreachable!(),
        }
    }
}

impl<'a> ServerOnlyConfigRef<'a> {
    pub fn server(&self) -> &ServerConfig {
        // We can safely unwrap here because we verified the type in as_client()
        match &self.inner.mode_config {
            ModeConfig::Server { server } => server,
            _ => unreachable!(),
        }
    }
}

macro_rules! quick_config {
    () => {
        crate::common::config::Config::read(None)
    };
    ($path:expr) => {{
        let path: std::path::PathBuf = $path.into();
        crate::common::config::Config::read(Some(path))
    }};
}
pub(crate) use quick_config;

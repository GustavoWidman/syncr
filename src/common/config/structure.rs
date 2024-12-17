use std::{
    net::IpAddr,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use fixedstr::zstr;
use serde::{Deserialize, Serialize};
use toml::value::Array;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct ConfigStructure {
    config: ConfigInner,
}

impl Deref for ConfigStructure {
    type Target = ConfigInner;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl DerefMut for ConfigStructure {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct ConfigInner {
    pub secret: zstr<32>,
    #[serde(flatten)]
    pub mode_config: ModeConfig,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "mode")]
pub enum ModeConfig {
    #[serde(rename = "server")]
    Server { server: ServerConfig },
    #[serde(rename = "client")]
    Client { client: ClientConfig },
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self::Client {
            client: ClientConfig::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
    pub ip: IpAddr,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            port: 7878,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ClientConfig {
    pub server_ip: IpAddr,
    pub server_port: u16,
    pub directories: Directories,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Directories {
    pub paths: Vec<PathBuf>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_ip: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            server_port: 7878,
            directories: Directories { paths: Vec::new() },
        }
    }
}

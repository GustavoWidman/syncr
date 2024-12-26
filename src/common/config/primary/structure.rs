use std::{
    net::IpAddr,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use fixedstr::zstr;
use serde::{Deserialize, Serialize};
use toml::value::Array;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct ConfigTOML {
    config: ConfigInner,
}

impl Deref for ConfigTOML {
    type Target = ConfigInner;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl DerefMut for ConfigTOML {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConfigInner {
    pub secret: zstr<32>,
    #[serde(flatten)]
    pub mode_config: ModeConfig,
}

impl Default for ConfigInner {
    fn default() -> Self {
        Self {
            secret: "password".into(),
            mode_config: ModeConfig::Client {
                client: ClientConfig::default(),
            },
        }
    }
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
    pub directories: Vec<Directory>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Directory {
    pub path: PathBuf,
    pub active: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_ip: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            server_port: 7878,
            directories: Vec::from([
                Directory {
                    path: PathBuf::from("~/Documents/enabled"),
                    active: true,
                },
                Directory {
                    path: PathBuf::from("~/Downloads/disabled"),
                    active: false,
                },
            ]),
        }
    }
}

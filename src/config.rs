use serde::{Serialize, Deserialize};
use std::default::Default;
use std::fs;
use std::path::Path;
use log::{info, trace, warn};
use std::net::ToSocketAddrs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub id: &'static str,
    address: &'static str
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyConfig {
    pub bind: &'static str,
    pub ip_forward: bool,
    pub online_mode: bool,
    pub max_players: i32,
    pub motd: &'static str,
    pub favicon: Option<&'static str>,
    pub servers: Vec<ServerConfig>
}

impl ServerConfig {
    fn get_address(&self) -> std::net::SocketAddr {
        self.address.to_owned().to_socket_addrs().unwrap().next().unwrap()
    }
}

impl ProxyConfig {
    pub fn load(path: &Path) -> ProxyConfig {
        if path.exists() {
            let config = toml::from_str(Box::leak(fs::read_to_string(path).unwrap().into_boxed_str())).unwrap();

            info!("Successfully loaded {}!", path.file_name().unwrap().to_str().unwrap());

            config
        } else {
            warn!("Configuration file not found!");
            let config = ProxyConfig::default();

            trace!("Default configuration: {:?}", config);
            fs::write(path, toml::to_string(&config).unwrap()).unwrap();

            config
        }
    }

    pub(crate) fn set_favicon(&mut self, b64: String) {
        self.favicon = Some(Box::leak(b64.into_boxed_str()));
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        let mut servers = Vec::new();
        servers.push(ServerConfig {
            id: "lobby",
            address: "localhost:25565"
        });

        ProxyConfig {
            bind: "0.0.0.0:25570",
            ip_forward: true,
            online_mode: true,
            max_players: 20,
            motd: "&3Enter the rift.",
            favicon: None,
            servers: servers
        }
    }
}

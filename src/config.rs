use serde::{Serialize, Deserialize};
use std::default::Default;
use std::fs;
use std::path::Path;
use log::{info, trace, warn};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ProxyConfig {
    bind: &'static str,
    pub ip_forward: bool,
    pub online_mode: bool,
    pub max_players: i32
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
}

impl Default for ProxyConfig {
    fn default() -> Self {
        ProxyConfig {
            bind: "0.0.0.0:25570",
            ip_forward: true,
            online_mode: true,
            max_players: 20
        }
    }
}

// SPDX-License-Identifier: GPL-3.0-only
//! Server Config Struct and Loading/Saving
//!
//! Authors: MarioS271

use serde::{Deserialize, Serialize};

const SERVER_CONFIG_FILE: &str = "servers.toml";

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default)]
    servers: Vec<ServerEntry>
}
impl ServerConfig {
    pub fn servers(&self) -> &Vec<ServerEntry> {
        &self.servers
    }

    pub fn servers_mut(&mut self) -> &mut Vec<ServerEntry> {
        &mut self.servers
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub port: u16,
    pub key: String
}

pub fn load() -> Result<ServerConfig, String> {
    let path = dirs::home_dir()
        .ok_or("Could not find home directory".to_string())?
        .join(format!("{}/{}", super::CONFIG_DIR, SERVER_CONFIG_FILE));

    if !path.exists() {
        println!("{} does not exist, using default config", SERVER_CONFIG_FILE);
        return Ok(ServerConfig {
            servers: Vec::new()
        });
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Could not read config: {}", e))?;

    toml::from_str(&content)
        .map_err(|e| format!("Invalid Config: {}", e))
}

pub fn save(config: &ServerConfig) -> Result<(), String> {
    let path = dirs::home_dir()
        .ok_or("Could not find home directory".to_string())?
        .join(format!("{}/{}", super::CONFIG_DIR, SERVER_CONFIG_FILE));

    std::fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| format!("Could not create config dir: {}", e))?;

    let content = toml::to_string(config)
        .map_err(|e| format!("Could not serialize config: {}", e))?;

    std::fs::write(&path, content)
        .map_err(|e| format!("Could not write config: {}", e))
}

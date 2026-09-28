// SPDX-License-Identifier: GPL-3.0-only
//! Client Config Struct and Loading/Saving
//!
//! Authors: MarioS271

use serde::{Deserialize, Serialize};
use crate::helpers::ask_for_input;

const CLIENT_CONFIG_FILE: &str = "clients.toml";

#[derive(Serialize, Deserialize)]
pub struct ClientConfig {
    username: String,
    #[serde(default)]
    servers: Vec<ServerEntry>
}
impl ClientConfig {
    pub fn username(&mut self) -> Result<&String, String> {
        if self.username.is_empty() {
            let input = ask_for_input("Please enter your username")
                .map_err(|e| format!("Failed to read in username: {}", e))?;
            self.username = input;
            save(&self)?;
        }
        Ok(&self.username)
    }

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
    pub address: String,
    pub key: String
}

pub fn load() -> Result<ClientConfig, String> {
    let path = dirs::home_dir()
        .ok_or("Could not find home directory".to_string())?
        .join(format!("{}/{}", super::CONFIG_DIR, CLIENT_CONFIG_FILE));

    if !path.exists() {
        println!("{} does not exist, using default config", CLIENT_CONFIG_FILE);
        return Ok(ClientConfig {
            username: String::new(),
            servers: Vec::new()
        });
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Could not read config: {}", e))?;

    toml::from_str(&content)
        .map_err(|e| format!("Invalid Config: {}", e))
}

pub fn save(config: &ClientConfig) -> Result<(), String> {
    let path = dirs::home_dir()
        .ok_or("Could not find home directory".to_string())?
        .join(format!("{}/{}", super::CONFIG_DIR, CLIENT_CONFIG_FILE));

    std::fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| format!("Could not create config dir: {}", e))?;

    let content = toml::to_string(config)
        .map_err(|e| format!("Could not serialize config: {}", e))?;

    std::fs::write(&path, content)
        .map_err(|e| format!("Could not write config: {}", e))
}

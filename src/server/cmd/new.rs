// SPDX-License-Identifier: GPL-3.0-only
//! `server --new` Command Handler
//!
//! Authors: MarioS271

use crate::{DEFAULT_PORT, config, encrypt};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

pub fn handler(name: String, mut port: Option<u16>) -> Result<(), String> {
    let mut config = config::server::load()?;

    if config.servers.iter().any(|server| server.name == name) {
        return Err(format!("Server named '{}' already exists", name));
    }

    if port == None {
        port = Some(DEFAULT_PORT);
        println!("No port given, using default port ({})", DEFAULT_PORT);
    } else {
        println!("Using given port {}", port.unwrap());
    }

    println!("Generating encryption key");
    let key = encrypt::generate_key()?;
    let base64_key = STANDARD.encode(&key);

    let server = config::server::ServerEntry {
        name: name.clone(),
        port: port.unwrap(),
        key: base64_key,
    };

    config.servers.push(server);
    config::server::save(&config)?;

    super::show_key::handler(name)
}

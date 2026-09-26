// SPDX-License-Identifier: GPL-3.0-only
//! `client --add` Command Handler
//!
//! Authors: MarioS271

use crate::DEFAULT_PORT;
use crate::helpers::ask_for_input;

pub fn handler(name: String, mut address: String) -> Result<(), String> {
    let mut config = crate::config::client::load()?;

    if config.servers.iter().any(|server| server.name == name) {
        return Err(format!("Server named '{}' already exists", name));
    }

    if !address.contains(':') {
        println!("No port given, using default port {}", DEFAULT_PORT);
        address.push_str(
            format!(":{}", DEFAULT_PORT).as_str()
        );
    } else {
        println!("Using given port {}", address.rsplit_once(':').unwrap().1)
    }

    let key = ask_for_input("Please enter the server's encryption key")
        .map_err(|e| format!("Failed to read input: {}", e))?;

    if key.trim().is_empty() {
        return Err("No input received, aborting".to_string());
    }

    let server = crate::config::client::ServerEntry {
        name,
        address,
        key
    };

    config.servers.push(server);
    crate::config::client::save(&config)?;

    Ok(())
}

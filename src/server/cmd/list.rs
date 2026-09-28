// SPDX-License-Identifier: GPL-3.0-only
//! `server --list` Command Handler
//!
//! Authors: MarioS271

pub fn handler() -> Result<(), String> {
    let config = crate::config::server::load()?;

    if !config.servers().is_empty() {
        println!("Existing Servers:");
    } else {
        println!("No servers exist");
        return Ok(());
    }

    for server in config.servers() {
        println!("  {} on port {}", server.name, server.port);
    }

    Ok(())
}

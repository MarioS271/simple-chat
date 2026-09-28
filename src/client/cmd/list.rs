// SPDX-License-Identifier: GPL-3.0-only
//! `client --list` Command Handler
//!
//! Authors: MarioS271

pub fn handler() -> Result<(), String> {
    let config = crate::config::client::load()?;

    if !config.servers().is_empty() {
        println!("Known Servers:");
    } else {
        println!("No servers currently known");
        return Ok(());
    }

    for server in config.servers() {
        println!("  {} at {}", server.name, server.address);
    }

    Ok(())
}

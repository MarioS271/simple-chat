// SPDX-License-Identifier: GPL-3.0-only
//! `client --remove` Command Handler
//!
//! Authors: MarioS271

use crate::helpers::ask_for_confirmation;

pub fn handler(name: String) -> Result<(), String> {
    let mut config = crate::config::client::load()?;

    if !config.servers.iter().any(|server| server.name == name) {
        return Err(format!("Server named '{}' doesn't exist", name));
    }

    let confirm = ask_for_confirmation(
        format!("Do you want to delete the server '{}'?", name).as_str(),
        false
    ).map_err(|e| format!("Failed to read choice: {}", e))?;

    if !confirm {
        return Err("Aborted.".to_string());
    }

    config.servers.retain(|server| server.name != name);
    crate::config::client::save(&config)?;

    Ok(())
}

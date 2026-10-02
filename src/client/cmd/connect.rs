// SPDX-License-Identifier: GPL-3.0-only
//! `client <name>` Command Handling
//!
//! Authors: MarioS271

use crate::TCP_TIMEOUT;
use crate::client::helpers::sender_name_string_to_bytes;
use crate::client::state::ClientState;
use crate::client::ui::start_tui;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub fn handler(name: String) -> Result<(), String> {
    let mut config = crate::config::client::load()?;

    let username = config.username()?.clone();
    let server = config.servers().iter().find(|server| server.name == name)
        .ok_or_else(|| format!("Server named '{}' doesn't exist", name))?;

    let client_state = ClientState::new(username, server.address.clone());

    drop(config);

    println!("Attempting to connect to {}", client_state.remote);

    let mut stream = TcpStream::connect(&client_state.remote)
        .map_err(|e| format!("Failed to open TCP stream: {}", e))?;
    stream.set_nodelay(true).map_err(|e| format!("Failed to set TCP stream into no delay mode: {}", e))?;
    stream.set_read_timeout(Some(TCP_TIMEOUT)).map_err(|e| format!("Failed to set TCP stream read timeout: {}", e))?;

    println!("Successfully connected to {}", client_state.remote);

    let mut server_version = [0u8; 2];
    stream.read_exact(&mut server_version)
        .map_err(|e| format!("Failed to receive server's protocol version: {}", e))?;

    if u16::from_be_bytes(server_version) != crate::PROTOCOL_VERSION {
        return Err("This client is incompatible with the server's protocol version".to_string());
    }

    stream.write_all(&crate::PROTOCOL_VERSION.to_be_bytes())
        .map_err(|e| format!("Failed to transmit own protocol version: {}", e))?;
    stream.write_all(&sender_name_string_to_bytes(&client_state.name))
        .map_err(|e| format!("Failed to transmit username: {}", e))?;

    start_tui(
        Arc::new(Mutex::new(client_state)),
        stream
    ).map_err(|e| format!("{}", e))
}

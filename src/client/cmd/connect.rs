// SPDX-License-Identifier: GPL-3.0-only
//! `client <name>` Command Handling
//!
//! Authors: MarioS271

use crate::client::net::receive::cipher_to_packet;
use crate::client::net::send::send_packet;
use crate::client::state::ClientState;
use crate::client::ui::start_tui;
use crate::encrypt::EncryptionKey;
use crate::packets::framing;
use crate::packets::handshake::Handshake;
use crate::packets::packet::Packet;
use crate::{PROTOCOL_VERSION, TCP_TIMEOUT};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub fn handler(name: String) -> Result<(), String> {
    let mut config = crate::config::client::load()?;

    let username = config.username()?.clone();
    let server = config.servers().iter().find(|server| server.name == name)
        .ok_or_else(|| format!("Server named '{}' doesn't exist", name))?;

    let client_state = ClientState::new(username, server.address.clone());

    let encryption_key = STANDARD.decode(server.key.trim())
        .map_err(|e| format!("Could not decode encryption key: {}", e))?;
    let encryption_key: EncryptionKey = encryption_key.try_into()
        .map_err(|_| "Could not convert Vec<u8> to EncryptionKey".to_string())?;

    drop(config);

    println!("Attempting to connect to {}", client_state.remote);

    let mut stream = TcpStream::connect(&client_state.remote)
        .map_err(|e| format!("Failed to open TCP stream: {}", e))?;
    stream.set_nodelay(true).map_err(|e| format!("Failed to set TCP stream into no delay mode: {}", e))?;
    stream.set_read_timeout(Some(TCP_TIMEOUT)).map_err(|e| format!("Failed to set TCP stream read timeout: {}", e))?;

    println!("Successfully connected to {}", client_state.remote);

    match framing::read_message(&mut stream) {
        Ok(cipher) => {
            match cipher_to_packet(cipher, encryption_key) {
                Ok(Packet::Heartbeat) => {},
                Ok(_) => {
                    eprintln!("Received non-heartbeat packet as first packet from server");
                    std::process::exit(1);
                },
                Err(err) => {
                    eprintln!("Deserialize Error: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
            eprintln!("Disconnected");
            std::process::exit(1);
        }
        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock
            || err.kind() == std::io::ErrorKind::TimedOut => {
            eprintln!("Receive Timed Out: {}", err);
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Receive Error: {}", err);
            std::process::exit(1);
        }
    }

    let handshake = Handshake::new(
        PROTOCOL_VERSION,
        client_state.name.clone()
    );
    if let Err(e) = send_packet(&stream, Packet::Handshake(handshake), encryption_key) {
        return Err(e.to_string());
    };

    start_tui(
        Arc::new(Mutex::new(client_state)),
        stream,
        encryption_key
    ).map_err(|e| format!("{}", e))
}

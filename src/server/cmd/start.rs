// SPDX-License-Identifier: GPL-3.0-only
//! `server <name>` Command Handler
//!
//! Authors: MarioS271

use crate::encrypt::EncryptionKey;
use crate::message::Message;
use crate::server::handle_client::handle_client;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
pub async fn handler(name: String) -> Result<(), String> {
    let config = crate::config::server::load()?;
    let server = config.servers().iter().find(|server| server.name == name)
        .ok_or_else(|| format!("Server named '{}' doesn't exist", name))?;

    let encryption_key = STANDARD.decode(server.key.trim())
        .map_err(|e| format!("Could not decode encryption key: {}", e))?;
    let encryption_key: EncryptionKey = encryption_key.try_into()
        .map_err(|_| "Could not convert Vec<u8> to EncryptionKey".to_string())?;

    println!("Running as server on port {}", server.port);

    let listener = TcpListener::bind(("0.0.0.0", server.port))
        .await
        .map_err(|e| format!("Could not create TcpListener: {}", e))?;

    let (broadcast_sender, _) = broadcast::channel::<Message>(256);

    loop {
        let (tcp_stream, socket_addr) = listener.accept()
            .await
            .map_err(|e| format!("Could not accept connection: {}", e))?;

        let broadcast_sender = broadcast_sender.clone();
        tokio::spawn(async move {
            let result = handle_client(tcp_stream, broadcast_sender, socket_addr, encryption_key).await;
            if let Err(e) = result {
                eprintln!("handle_client failed: {}", e);
            }
        });
    }
}

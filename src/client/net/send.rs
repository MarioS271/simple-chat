// SPDX-License-Identifier: GPL-3.0-only
//! Message Send Logic
//!
//! Authors: MarioS271

use crate::client::state::SenderNameArray;
use crate::encrypt::{EncryptionKey, encrypt};
use crate::framing;
use crate::message::{ChatMessage, Message};
use std::net::TcpStream;

pub fn send(mut stream: &TcpStream, sender_name: SenderNameArray, message: String, key: EncryptionKey) -> std::io::Result<()> {
    if !message.is_empty() {
        let msg = Message::Chat(ChatMessage::new(sender_name, message));
        let encrypted = match encrypt(key, msg.serialize().as_slice()) {
            Ok(cipher) => cipher,
            Err(e) => {
                eprintln!("Failed to encrypt message: {}", e);
                std::process::exit(1);
            }
        };
        framing::write_message(
            &mut stream,
            encrypted.as_slice()
        )?;
    }
    Ok(())
}

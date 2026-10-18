// SPDX-License-Identifier: GPL-3.0-only
//! Message Send Logic
//!
//! Authors: MarioS271

use crate::client::state::SenderNameArray;
use crate::encrypt::{EncryptionKey, encrypt};
use crate::packets::framing;
use crate::packets::message::chat_message::ChatMessage;
use crate::packets::packet::Packet;
use std::net::TcpStream;

pub fn send_message(stream: &TcpStream, sender_name: SenderNameArray, message: String, encryption_key: EncryptionKey) -> std::io::Result<()> {
    if message.is_empty() {
        return Ok(());
    }

    let packet = Packet::ChatMessage(ChatMessage::new(sender_name, message));
    send_packet(stream, packet, encryption_key)
}

pub fn send_packet(mut stream: &TcpStream, packet: Packet, encryption_key: EncryptionKey) -> std::io::Result<()> {
    let encrypted = match encrypt(encryption_key, packet.serialize().as_slice()) {
        Ok(cipher) => cipher,
        Err(e) => {
            eprintln!("Failed to encrypt message: {}", e);
            std::process::exit(1);
        }
    };

    framing::write_message(
        &mut stream,
        encrypted.as_slice()
    )
}

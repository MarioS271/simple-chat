// SPDX-License-Identifier: GPL-3.0-only
//! Message Send Logic
//!
//! Authors: MarioS271

use crate::framing;
use crate::message::{ChatMessage, Message};
use std::net::TcpStream;

pub fn send(mut stream: &TcpStream, sender_name: String, message: String) -> std::io::Result<()> {
    if !message.is_empty() {
        let msg = Message::Chat(ChatMessage::new(sender_name, message));
        framing::write_message(
            &mut stream,
            msg.serialize().as_slice()
        )?;
    }
    Ok(())
}

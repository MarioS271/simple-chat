// SPDX-License-Identifier: GPL-3.0-only
//! Chat Message Type
//!
//! Authors: MarioS271

use super::formatted::Formatted;
use crate::client::state::{MAX_NAME_LEN, SenderNameArray};
use crate::helpers::get_timestamp;

#[derive(Clone)]
pub struct ChatMessage {
    pub timestamp: u64,
    pub sender_name: SenderNameArray,
    pub content: Vec<u8>
}

impl ChatMessage {
    const METADATA_SIZE: usize = 8 + MAX_NAME_LEN;

    pub fn new(sender_name: SenderNameArray, content: String) -> Self {
        Self {
            timestamp: get_timestamp(),
            sender_name,
            content: content.into_bytes()
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(Self::METADATA_SIZE + self.content.len());

        buffer.extend_from_slice(&self.timestamp.to_be_bytes());
        buffer.extend_from_slice(&self.sender_name);
        buffer.extend_from_slice(&self.content);

        buffer
    }

    pub fn deserialize(buffer: &[u8]) -> std::io::Result<Self> {
        if buffer.len() < Self::METADATA_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer too short"
            ));
        }

        let timestamp = u64::from_be_bytes(buffer[..8].try_into().unwrap());
        let sender_name = buffer[8..8 + MAX_NAME_LEN].try_into().unwrap();
        let content = buffer[Self::METADATA_SIZE..].to_vec();

        Ok(Self {
            timestamp,
            sender_name,
            content
        })
    }

    pub fn formatted_sender_name(&self) -> &str {
        std::str::from_utf8(&self.sender_name)
            .unwrap_or("\u{FFFD}")
            .trim_end_matches('\0')
            .trim_end()
    }
}

impl Formatted for ChatMessage {
    fn __timestamp(&self) -> u64 {
        self.timestamp
    }
    fn __content(&self) -> &[u8] {
        &self.content
    }
}

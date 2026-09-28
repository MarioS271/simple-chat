// SPDX-License-Identifier: GPL-3.0-only
//! Message Definition
//!
//! Authors: MarioS271

use crate::client::state::{MAX_NAME_LEN, SenderNameArray};
use crate::helpers::get_timestamp;
use std::borrow::Cow;

pub enum Message {
    Chat(ChatMessage),
    System(SystemMessage)
}
impl Message {
    pub const TYPE_BYTES: usize = 1;

    const TYPE_CHAT_U8: u8 = 0;
    const TYPE_SYSTEM_U8: u8 = 1;

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            Message::Chat(inner) => {
                buf.extend_from_slice(&Self::TYPE_CHAT_U8.to_be_bytes());
                buf.extend_from_slice(&inner.serialize());
            }
            Message::System(inner) => {
                buf.extend_from_slice(&Self::TYPE_SYSTEM_U8.to_be_bytes());
                buf.extend_from_slice(&inner.serialize());
            }
        }

        buf
    }

    pub fn deserialize(buffer: &[u8]) -> std::io::Result<Self> {
        if buffer.len() <= Self::TYPE_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer too short"
            ));
        }

        match buffer[0] {
            Self::TYPE_CHAT_U8 => Ok(Message::Chat(ChatMessage::deserialize(&buffer[Self::TYPE_BYTES..])?)),
            Self::TYPE_SYSTEM_U8 => Ok(Message::System(SystemMessage::deserialize(&buffer[Self::TYPE_BYTES..])?)),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Message Type"
            ))
        }
    }
}

pub trait Formatted {
    fn __timestamp(&self) -> u64;
    fn __content(&self) -> &[u8];

    fn formatted_timestamp(&self) -> String {
        chrono::DateTime::from_timestamp(self.__timestamp() as i64, 0)
            .unwrap()
            .with_timezone(&chrono::Local)
            .format("%H:%M:%S")
            .to_string()
    }

    fn formatted_content(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.__content())
    }
}

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

pub struct SystemMessage {
    pub timestamp: u64,
    pub content: Vec<u8>
}
impl SystemMessage {
    const METADATA_SIZE: usize = 8;

    pub fn new(content: String) -> Self {
        Self {
            timestamp: get_timestamp(),
            content: content.into_bytes()
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(Self::METADATA_SIZE + self.content.len());

        buffer.extend_from_slice(&self.timestamp.to_be_bytes());
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
        let content = buffer[Self::METADATA_SIZE..].to_vec();

        Ok(Self {
            timestamp,
            content
        })
    }
}
impl Formatted for SystemMessage {
    fn __timestamp(&self) -> u64 {
        self.timestamp
    }
    fn __content(&self) -> &[u8] {
        &self.content
    }
}

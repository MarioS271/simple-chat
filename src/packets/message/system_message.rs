// SPDX-License-Identifier: GPL-3.0-only
//! System Message Type
//!
//! Authors: MarioS271

use super::formatted::Formatted;
use crate::helpers::get_timestamp;

#[derive(Clone)]
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

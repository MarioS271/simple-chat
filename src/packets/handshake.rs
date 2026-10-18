// SPDX-License-Identifier: GPL-3.0-only
//! The Handshake Packet
//!
//! Authors: MarioS271

use crate::ProtocolVersion;

#[derive(Clone)]
pub struct Handshake {
    pub protocol_version: ProtocolVersion,
    pub client_name: String
}

impl Handshake {
    const PROTOCOL_VERSION_LEN: usize = size_of::<ProtocolVersion>();

    pub fn new(protocol_version: ProtocolVersion, client_name: String) -> Self {
        Self {
            protocol_version,
            client_name
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(size_of::<ProtocolVersion>() + self.client_name.len());

        buffer.extend_from_slice(&self.protocol_version.to_be_bytes());
        buffer.extend_from_slice(self.client_name.as_bytes());

        buffer
    }

    pub fn deserialize(buffer: &[u8]) -> std::io::Result<Self> {
        if buffer.len() < Self::PROTOCOL_VERSION_LEN {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer too short"
            ));
        } else if buffer.len() == Self::PROTOCOL_VERSION_LEN {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Zero-size name given"
            ));
        }

        let protocol_version = ProtocolVersion::from_be_bytes(buffer[..Self::PROTOCOL_VERSION_LEN].try_into().unwrap());
        let client_name = String::from_utf8_lossy(&buffer[Self::PROTOCOL_VERSION_LEN..]).into_owned();

        Ok(Self {
            protocol_version,
            client_name
        })
    }
}

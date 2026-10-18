// SPDX-License-Identifier: GPL-3.0-only
//! Packet Base; each packet is a packet type header followed by the data (like a serialized message
//! or a handshake)
//!
//! Authors: MarioS271

use super::handshake::Handshake;
use super::message::chat_message::ChatMessage;
use super::message::system_message::SystemMessage;

type PacketType = u8;

#[derive(Clone)]
pub enum Packet {
    Heartbeat,
    Handshake(Handshake),
    ChatMessage(ChatMessage),
    SystemMessage(SystemMessage)
}

impl Packet {
    const TYPE_MARKER_LEN: usize = size_of::<PacketType>();

    const HEARTBEAT_MARKER: PacketType = 0;
    const HANDSHAKE_MARKER: PacketType = 1;
    const CHAT_MESSAGE_MARKER: PacketType = 2;
    const SYSTEM_MESSAGE_MARKER: PacketType = 3;

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::<u8>::new();

        match self {
            Packet::Heartbeat => {
                buf.extend_from_slice(&Self::HEARTBEAT_MARKER.to_be_bytes());
            }
            Packet::Handshake(inner) => {
                buf.extend_from_slice(&Self::HANDSHAKE_MARKER.to_be_bytes());
                buf.extend_from_slice(&inner.serialize());
            }
            Packet::ChatMessage(inner) => {
                buf.extend_from_slice(&Self::CHAT_MESSAGE_MARKER.to_be_bytes());
                buf.extend_from_slice(&inner.serialize());
            }
            Packet::SystemMessage(inner) => {
                buf.extend_from_slice(&Self::SYSTEM_MESSAGE_MARKER.to_be_bytes());
                buf.extend_from_slice(&inner.serialize());
            }
        }

        buf
    }

    pub fn deserialize(buffer: &[u8]) -> std::io::Result<Self> {
        let Some((marker_bytes, payload)) = buffer.split_at_checked(Self::TYPE_MARKER_LEN) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer too short"
            ));
        };

        let marker = PacketType::from_be_bytes(marker_bytes.try_into().unwrap());

        match marker {
            Self::HEARTBEAT_MARKER => Ok(Packet::Heartbeat),
            Self::HANDSHAKE_MARKER => Ok(Packet::Handshake(Handshake::deserialize(payload)?)),
            Self::CHAT_MESSAGE_MARKER => Ok(Packet::ChatMessage(ChatMessage::deserialize(payload)?)),
            Self::SYSTEM_MESSAGE_MARKER => Ok(Packet::SystemMessage(SystemMessage::deserialize(payload)?)),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid or Unknown Message Type"
            ))
        }
    }
}

// SPDX-License-Identifier: GPL-3.0-only
//! Client State Struct
//!
//! Authors: MarioS271

use crate::MAX_NAME_LEN;
use crate::packets::packet::Packet;

pub type SenderNameArray = [u8; MAX_NAME_LEN];
pub const EMPTY_SENDER_NAME_ARRAY: SenderNameArray = [0u8; MAX_NAME_LEN];

pub struct ClientState {
    pub name: String,
    pub remote: String,
    pub messages: Vec<Packet>,
    pub input: String
}

impl ClientState {
    pub fn new(name: String, remote: String) -> Self {
        Self {
            name,
            remote,
            messages: Vec::new(),
            input: String::new()
        }
    }
}

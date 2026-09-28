// SPDX-License-Identifier: GPL-3.0-only
//! Client State Struct
//!
//! Authors: MarioS271

use crate::message::Message;

pub const MAX_NAME_LEN: usize = 32;

pub type SenderNameArray = [u8; MAX_NAME_LEN];
pub const EMPTY_SENDER_NAME_ARRAY: SenderNameArray = [0u8; MAX_NAME_LEN];

pub struct ClientState {
    pub name: String,
    pub remote: String,
    pub messages: Vec<Message>,
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

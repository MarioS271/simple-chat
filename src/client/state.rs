// SPDX-License-Identifier: GPL-3.0-only
//! Client State Struct
//!
//! Authors: MarioS271

use crate::MAX_NAME_LEN;
use crate::packets::packet::Packet;
use std::collections::VecDeque;

pub type SenderNameArray = [u8; MAX_NAME_LEN];
pub const EMPTY_SENDER_NAME_ARRAY: SenderNameArray = [0u8; MAX_NAME_LEN];

const MAX_HISTORY_LEN: usize = 1000;

pub struct ClientState {
    pub name: String,
    pub remote: String,
    pub messages: VecDeque<Packet>,
    pub input: String
}

impl ClientState {
    pub fn new(name: String, remote: String) -> Self {
        Self {
            name,
            remote,
            messages: VecDeque::with_capacity(MAX_HISTORY_LEN),
            input: String::new()
        }
    }

    pub fn push_message(&mut self, message: Packet) {
        if self.messages.len() >= MAX_HISTORY_LEN {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }
}

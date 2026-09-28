// SPDX-License-Identifier: GPL-3.0-only
//! Client-Side Helpers
//!
//! Authors: MarioS271

use crate::client::state::{EMPTY_SENDER_NAME_ARRAY, MAX_NAME_LEN, SenderNameArray};

pub fn sender_name_string_to_bytes(sender_name: &str) -> SenderNameArray {
    let mut name_bytes = EMPTY_SENDER_NAME_ARRAY;
    let end = sender_name
        .char_indices()
        .map(|(index, character)| index + character.len_utf8())
        .take_while(|&end| end <= MAX_NAME_LEN)
        .last()
        .unwrap_or(0);

    let bytes = &sender_name.as_bytes()[..end];

    name_bytes[..bytes.len()].copy_from_slice(bytes);

    name_bytes
}

pub fn sender_name_bytes_to_string(sender_name: &SenderNameArray) -> String {
    let trimmed = sender_name.iter()
        .position(|&byte| byte == 0)
        .map(|len| &sender_name[..len])
        .unwrap_or(sender_name);

    String::from_utf8_lossy(trimmed).into_owned()
}

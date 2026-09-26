// SPDX-License-Identifier: GPL-3.0-only
//! Client-Side Helpers
//!
//! Authors: MarioS271

use crate::client::ui::state::{EMPTY_SENDER_NAME_ARRAY, SenderNameArray};

pub fn sender_name_string_to_bytes(sender_name: String) -> SenderNameArray {
    let mut name_bytes = EMPTY_SENDER_NAME_ARRAY;
    let bytes = sender_name.as_bytes();

    name_bytes[..bytes.len()].copy_from_slice(bytes);

    name_bytes
}

pub fn sender_name_bytes_to_string(sender_name: SenderNameArray) -> String {
    let trimmed = sender_name.iter()
        .position(|&byte| byte == 0)
        .map(|len| &sender_name[..len])
        .unwrap_or(&sender_name);

    String::from_utf8_lossy(trimmed).into_owned()
}

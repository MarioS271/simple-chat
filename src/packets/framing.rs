// SPDX-License-Identifier: GPL-3.0-only
//! TCP framing logic
//!
//! Authors: MarioS271

use crate::MAX_MESSAGE_SIZE;
use std::io::{Read, Write};

pub fn write_message(writer: &mut impl Write, data: &[u8]) -> std::io::Result<()> {
    if data.len() > MAX_MESSAGE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Message too large"
        ));
    }

    let mut buffer = Vec::with_capacity(4 + data.len());

    buffer.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buffer.extend_from_slice(data);

    writer.write_all(buffer.as_slice())?;
    Ok(())
}

pub fn read_message(reader: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;

    let len = u32::from_be_bytes(len_buf) as usize;

    if len > MAX_MESSAGE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Message too large"
        ));
    }

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;

    Ok(buf)
}

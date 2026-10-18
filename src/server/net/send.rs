// SPDX-License-Identifier: GPL-3.0-only
//! Helper to send packets
//!
//! Authors: MarioS271

use crate::TCP_TIMEOUT;
use crate::encrypt::{EncryptionKey, encrypt};
use crate::packets::framing;
use crate::packets::packet::Packet;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::WriteHalf;
use tokio::time::timeout;

pub async fn send(
    packet: Packet,
    encryption_key: EncryptionKey,
    write_stream: &mut WriteHalf<'_>,
) -> std::io::Result<()> {
    let bytes = packet.serialize();
    let cipher = match encrypt(encryption_key, bytes.as_slice()) {
        Ok(cipher) => cipher,
        Err(err) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, err));
        }
    };

    let mut framed = Vec::<u8>::new();
    framing::write_message(&mut framed, cipher.as_slice())?;

    match timeout(TCP_TIMEOUT, write_stream.write_all(framed.as_slice())).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Socket Write Timed Out",
            ))
        }
    }
}
